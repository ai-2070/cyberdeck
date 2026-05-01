//! NATS JetStream adapter for durable event storage.
//!
//! This adapter uses NATS JetStream for persistent storage.
//!
//! # Design
//!
//! - Each shard maps to one JetStream stream: `{prefix}_shard_{shard_id}`
//! - Writes use async publish for high throughput
//! - Reads use direct get with sequence-based cursors for efficient pagination
//! - Reusable serialization buffers to avoid per-event allocation
//!
//! # Throughput Expectations
//!
//! JetStream throughput depends on deployment:
//! - Single node: 100K-500K messages/sec
//! - Clustered: Lower due to replication overhead
//!
//! The batch aggregation layer smooths bursts before they reach JetStream.

use async_nats::jetstream::{self, stream::Stream};
use async_nats::Client;
use async_trait::async_trait;
use bytes::Bytes;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::adapter::{Adapter, ShardPollResult};
use crate::config::JetStreamAdapterConfig;
use crate::error::AdapterError;
use crate::event::{Batch, InternalEvent, StoredEvent};

/// NATS JetStream adapter.
pub struct JetStreamAdapter {
    /// NATS client.
    client: Option<Client>,
    /// JetStream context.
    jetstream: Option<jetstream::Context>,
    /// Configuration.
    config: JetStreamAdapterConfig,
    /// Per-shard stream cache.
    ///
    /// Each shard's slot is an `Arc<OnceCell<Stream>>` so concurrent
    /// `on_batch` callers for the same cold shard race only on the
    /// outer `Mutex` (a brief get-or-insert) and then serialize on
    /// `OnceCell::get_or_try_init`. Without the per-shard `OnceCell`,
    /// two concurrent callers could both miss a flat
    /// `HashMap<u16, Stream>` cache, both call `get_stream` /
    /// `create_stream`, and both insert — extra RPCs on cold start
    /// and a hazard if create_stream configs ever diverge between
    /// callers.
    streams: Mutex<HashMap<u16, Arc<OnceCell<Stream>>>>,
    /// Whether the adapter has been initialized.
    initialized: AtomicBool,
}

impl JetStreamAdapter {
    /// Create a new JetStream adapter.
    pub fn new(config: JetStreamAdapterConfig) -> Result<Self, AdapterError> {
        Ok(Self {
            client: None,
            jetstream: None,
            config,
            streams: Mutex::new(HashMap::new()),
            initialized: AtomicBool::new(false),
        })
    }

    /// Get the stream name for a shard.
    #[inline]
    fn stream_name(&self, shard_id: u16) -> String {
        format!("{}_shard_{}", self.config.prefix, shard_id)
    }

    /// Get the subject name for a shard.
    #[inline]
    fn subject(&self, shard_id: u16) -> String {
        format!("{}.shard.{}", self.config.prefix, shard_id)
    }

    /// Serialize an event for storage.
    ///
    /// Format: JSON with `raw` and `ts` fields.
    /// Since `event.raw` is already pre-serialized JSON bytes, we embed it directly
    /// using `RawValue` semantics to avoid double-serialization.
    fn serialize_event(event: &InternalEvent) -> Result<Vec<u8>, AdapterError> {
        // Build JSON manually to avoid re-parsing/re-serializing the raw bytes
        // Format: {"r":<raw_json>,"t":<ts>,"s":<shard_id>}
        let mut buf = Vec::with_capacity(event.raw.len() + 32);
        buf.extend_from_slice(b"{\"r\":");
        buf.extend_from_slice(&event.raw); // Already valid JSON
        buf.extend_from_slice(b",\"t\":");
        buf.extend_from_slice(event.insertion_ts.to_string().as_bytes());
        buf.extend_from_slice(b",\"s\":");
        buf.extend_from_slice(event.shard_id.to_string().as_bytes());
        buf.push(b'}');
        Ok(buf)
    }

    /// Deserialize a stored event.
    fn deserialize_event(seq: u64, data: &[u8]) -> Result<StoredEvent, AdapterError> {
        let value: serde_json::Value = serde_json::from_slice(data)?;

        let raw = value.get("r").cloned().unwrap_or(serde_json::Value::Null);
        let raw_bytes = Bytes::from(serde_json::to_vec(&raw).unwrap_or_default());
        let insertion_ts = value.get("t").and_then(|v| v.as_u64()).unwrap_or(0);
        let shard_id = value.get("s").and_then(|v| v.as_u64()).unwrap_or(0) as u16;

        Ok(StoredEvent::new(
            seq.to_string(),
            raw_bytes,
            insertion_ts,
            shard_id,
        ))
    }

    /// Get or create a stream for a shard.
    ///
    /// Cold-start single-flight: only one `get_stream`/`create_stream`
    /// pair runs per shard regardless of how many concurrent
    /// `on_batch` calls land here at once. The brief outer `Mutex`
    /// just resolves "which `OnceCell` does this shard map to";
    /// the actual create-once happens inside
    /// `OnceCell::get_or_try_init`, which serializes initialization
    /// across all callers and surfaces the same `Stream` clone (or
    /// the same error) to each. On error the cell stays
    /// uninitialized and a subsequent call will retry.
    async fn get_or_create_stream(&self, shard_id: u16) -> Result<Stream, AdapterError> {
        let cell = {
            let mut streams = self.streams.lock();
            streams
                .entry(shard_id)
                .or_insert_with(|| Arc::new(OnceCell::new()))
                .clone()
        };

        let stream = cell
            .get_or_try_init(|| async {
                let stream_name = self.stream_name(shard_id);
                let js = self
                    .jetstream
                    .as_ref()
                    .ok_or_else(|| AdapterError::Connection("adapter not initialized".into()))?;

                // Try to get existing stream first; only create if missing.
                match js.get_stream(&stream_name).await {
                    Ok(stream) => Ok(stream),
                    Err(_) => {
                        let mut stream_config = jetstream::stream::Config {
                            name: stream_name.clone(),
                            subjects: vec![self.subject(shard_id)],
                            retention: jetstream::stream::RetentionPolicy::Limits,
                            storage: jetstream::stream::StorageType::File,
                            num_replicas: self.config.replicas,
                            discard: jetstream::stream::DiscardPolicy::Old,
                            allow_direct: true, // Required for direct_get API
                            ..Default::default()
                        };

                        if let Some(max_messages) = self.config.max_messages {
                            stream_config.max_messages = max_messages;
                        }
                        if let Some(max_bytes) = self.config.max_bytes {
                            stream_config.max_bytes = max_bytes;
                        }
                        if let Some(max_age) = self.config.max_age {
                            stream_config.max_age = max_age;
                        }

                        js.create_stream(stream_config)
                            .await
                            .map_err(|e| AdapterError::Connection(e.to_string()))
                    }
                }
            })
            .await?;

        Ok(stream.clone())
    }
}

impl std::fmt::Debug for JetStreamAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JetStreamAdapter")
            .field("url", &self.config.url)
            .field("prefix", &self.config.prefix)
            .field("initialized", &self.initialized.load(Ordering::Relaxed))
            .finish()
    }
}

#[async_trait]
impl Adapter for JetStreamAdapter {
    async fn init(&mut self) -> Result<(), AdapterError> {
        // Idempotency: no-op when already initialized and log at
        // warn so a misbehaving caller is observable. A second
        // `init` call would otherwise overwrite `client` /
        // `jetstream`, dropping the prior client and any in-flight
        // publishes — an orchestrator calling `init` defensively
        // after a perceived failure would silently lose messages.
        // The trait says "Called once before any other methods"
        // but doesn't enforce it.
        if self.initialized.load(Ordering::Acquire) {
            tracing::warn!(
                adapter = "jetstream",
                "JetStream adapter::init called twice; ignoring"
            );
            return Ok(());
        }

        let client = async_nats::ConnectOptions::new()
            .connection_timeout(self.config.connect_timeout)
            .request_timeout(Some(self.config.request_timeout))
            .connect(&self.config.url)
            .await
            .map_err(|e| AdapterError::Connection(e.to_string()))?;

        let jetstream = jetstream::new(client.clone());

        self.client = Some(client);
        self.jetstream = Some(jetstream);
        self.initialized.store(true, Ordering::Release);

        tracing::info!(
            adapter = "jetstream",
            url = %self.config.url,
            prefix = %self.config.prefix,
            "JetStream adapter initialized"
        );

        Ok(())
    }

    async fn on_batch(&self, batch: Batch) -> Result<(), AdapterError> {
        if batch.is_empty() {
            return Ok(());
        }

        let js = self
            .jetstream
            .as_ref()
            .ok_or_else(|| AdapterError::Connection("adapter not initialized".into()))?;

        // Convert to `async_nats::Subject` once — internally `Bytes`-
        // backed, so per-iteration `subject.clone()` is an Arc-style
        // refcount bump rather than a fresh `String` allocation.
        let subject: async_nats::Subject = self.subject(batch.shard_id).into();

        // Ensure stream exists
        let _ = self.get_or_create_stream(batch.shard_id).await?;

        // Serialize all events first
        let serialized: Vec<Vec<u8>> = batch
            .events
            .iter()
            .map(Self::serialize_event)
            .collect::<Result<Vec<_>, _>>()?;

        // Publish each event with a deterministic message ID for dedup.
        // If a retry resends the same batch, NATS discards duplicates
        // within its dedup window (default 2 minutes).
        //
        // Two-phase publish: enqueue all messages in order (each await
        // returns a `PublishAckFuture` once enqueued — fast), then
        // await every server ack in parallel. With one ack per event
        // the prior serial-await loop paid 1 RTT *per event*;
        // pipelining drops that to ~1 RTT per batch.
        //
        // Mid-batch failure is safe: if `publish_with_headers` returns
        // `Err` for event N, we drop the in-flight `PublishAckFuture`s
        // for events 0..N — but dropping them does not cancel the
        // publishes (the bytes are already on the wire to the server).
        // The caller retries the whole batch, and the JetStream dedup
        // window discards the prior copies via `Nats-Msg-Id`.
        //
        // The message-id buffer (`Nats-Msg-Id` header) is reused
        // across iterations: the `{nonce}:{shard_id}:{seq_start}`
        // prefix is the same for every event in the batch, so we
        // render it once and only rewrite the trailing `:{i}` per
        // event, eliminating the per-event `format!` allocation.
        //
        // The leading `{nonce}` segment is the bus's producer nonce
        // — sourced from `EventBusConfig::producer_nonce_path` when
        // the caller wants persistence across restarts, or from
        // the per-process default `event::batch_process_nonce`
        // otherwise. Without it, a producer that restarted within
        // JetStream's dedup window collided with its prior
        // incarnation's `shard:0:0…` ids and the new batches were
        // silently discarded; with it, the dedup window correctly
        // recognizes mid-batch retries from a crashed-and-restarted
        // producer when the persistent path is configured.
        // Use the batch's process_nonce field — bus-loaded once
        // and consistent across every batch from this bus instance.
        let mut acks = Vec::with_capacity(serialized.len());
        let mut msg_id_buf = String::new();
        let _ = write!(
            msg_id_buf,
            "{:x}:{}:{}",
            batch.process_nonce, batch.shard_id, batch.sequence_start
        );
        let prefix_len = msg_id_buf.len();

        for (i, data) in serialized.into_iter().enumerate() {
            // Reset to the cached prefix and append `:{i}`.
            msg_id_buf.truncate(prefix_len);
            let _ = write!(msg_id_buf, ":{i}");

            let mut headers = async_nats::HeaderMap::new();
            // `From<&str> for HeaderValue` copies the bytes into the
            // header, so reusing `msg_id_buf` on the next iteration is
            // safe — the header now owns its own copy.
            headers.insert("Nats-Msg-Id", msg_id_buf.as_str());

            let ack = js
                .publish_with_headers(subject.clone(), headers, data.into())
                .await
                .map_err(|e| {
                    if is_transient_error(&e) {
                        AdapterError::Transient(e.to_string())
                    } else {
                        AdapterError::Fatal(e.to_string())
                    }
                })?;
            // `PublishAckFuture` implements `IntoFuture`, not `Future`,
            // so it can't go straight into `try_join_all`. Wrap each
            // in an async block that handles the await + error
            // mapping in one place.
            acks.push(async move {
                ack.await
                    .map_err(|e| AdapterError::Transient(e.to_string()))
            });
        }

        // Await all server acks in parallel. `try_join_all` short-
        // circuits on the first error, which is the desired retry
        // semantic — the JetStream stream's dedup window will discard
        // duplicates on the retry.
        futures::future::try_join_all(acks).await?;

        tracing::trace!(
            shard_id = batch.shard_id,
            event_count = batch.events.len(),
            "Batch written to JetStream"
        );

        Ok(())
    }

    async fn flush(&self) -> Result<(), AdapterError> {
        // JetStream writes are synchronous (acked), nothing to flush
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), AdapterError> {
        self.initialized.store(false, Ordering::Release);

        // Clear stream cache
        {
            let mut streams = self.streams.lock();
            streams.clear();
        }

        // Drain the NATS client to flush pending messages and close
        // cleanly. Surface drain errors as `Transient` rather than
        // discarding them — the trait contract is "shutdown should
        // flush", and a silent failure here means in-flight messages
        // are quietly lost.
        if let Some(client) = &self.client {
            if let Err(e) = client.drain().await {
                tracing::error!(
                    adapter = "jetstream",
                    error = %e,
                    "drain() failed during JetStream shutdown"
                );
                return Err(AdapterError::Transient(format!("nats drain: {e}")));
            }
        }

        tracing::info!(adapter = "jetstream", "JetStream adapter shut down");
        Ok(())
    }

    async fn poll_shard(
        &self,
        shard_id: u16,
        from_id: Option<&str>,
        limit: usize,
    ) -> Result<ShardPollResult, AdapterError> {
        let mut stream = self.get_or_create_stream(shard_id).await?;

        // Parse the cursor (sequence number)
        let start_seq = from_id
            .and_then(|id| id.parse::<u64>().ok())
            .map(|seq| seq + 1) // Exclusive: start after the given sequence
            .unwrap_or(1); // Start from beginning if no cursor

        // Fetch one extra to detect has_more
        let fetch_limit = limit + 1;

        // Get messages directly from the stream
        let mut events = Vec::with_capacity(limit);
        let mut current_seq = start_seq;

        // Use the stream's actual last sequence to bound the search,
        // rather than an arbitrary multiplier that can miss events in
        // streams with large gaps (deletions, compaction). The
        // fallback saturates so a caller-supplied `limit` near
        // `usize::MAX` cannot wrap to a tiny `max_seq` and silently
        // cap the poll at zero events.
        //
        // Also extract `first_sequence` so the per-event walk below
        // can skip past a long retention-rollover gap in a single
        // jump. Without it, `direct_get(seq)` returns NotFound for
        // every deleted seq and the loop would increment by one
        // and try again — after a MAXLEN trim of the first 10M
        // sequences, `poll_shard(from_id=None)` resumes at
        // `start_seq=1` and the consumer would do 10M sequential
        // network RTTs before returning a single event (hung for
        // minutes, request timeout fires, next poll resumes from
        // where it left off — never made progress). With
        // `first_sequence` captured up-front, the first `NotFound`
        // jumps `current_seq` to `first_sequence` in one step.
        let (max_seq, first_seq) = match stream.info().await {
            Ok(info) => (info.state.last_sequence, info.state.first_sequence),
            Err(_) => {
                let span = (fetch_limit as u64).saturating_mul(10);
                (start_seq.saturating_add(span), 0)
            }
        };
        // Apply the retention-rollover jump up-front: if
        // `start_seq` is below the stream's first retained
        // sequence, advance the cursor immediately rather than
        // walking the deleted range one-by-one.
        if first_seq > current_seq {
            current_seq = first_seq;
        }

        // Use direct get to fetch messages by sequence
        // Use while loop so gaps don't consume our fetch count.
        // Track every sequence we observe (regardless of deserialize
        // outcome) so the cursor can advance past corrupt entries
        // instead of stalling on them.
        //
        // The loop's `current_seq > max_seq` short-circuit re-reads
        // `info()` once before declaring drain, to catch concurrent
        // writes that appeared after our initial sample. Without
        // the re-read, a producer writing new messages during the
        // read would be silently truncated — `has_more=false` would
        // come back even though the stream tail had more events.
        // `max_seq_re_checked` tracks whether we've already paid
        // the one bounded re-read, so a relentless producer can't
        // spin us forever in a re-info loop.
        let mut last_seen_seq: Option<u64> = None;
        let mut max_seq = max_seq;
        let mut max_seq_re_checked = false;
        while events.len() < fetch_limit {
            if current_seq > max_seq {
                // Before declaring drain, re-read `info()` once to
                // catch concurrent writes that appeared after our
                // initial sample. One bounded re-read per poll
                // preserves the loop's O(span) worst-case while
                // closing the truncation hole.
                if !max_seq_re_checked {
                    max_seq_re_checked = true;
                    if let Ok(info) = stream.info().await {
                        if info.state.last_sequence > max_seq {
                            max_seq = info.state.last_sequence;
                            continue;
                        }
                    }
                }
                // Searched too far without finding enough events
                break;
            }

            match stream.direct_get(current_seq).await {
                Ok(msg) => {
                    last_seen_seq = Some(current_seq);
                    match Self::deserialize_event(current_seq, &msg.payload) {
                        Ok(event) => events.push(event),
                        Err(e) => {
                            tracing::warn!(
                                stream = %self.stream_name(shard_id),
                                seq = current_seq,
                                error = %e,
                                "Failed to deserialize event, skipping"
                            );
                        }
                    }
                    current_seq += 1;
                }
                Err(e) => {
                    use async_nats::jetstream::stream::DirectGetErrorKind;
                    match e.kind() {
                        DirectGetErrorKind::NotFound => {
                            // Try next sequence (there might be gaps due to deletions)
                            current_seq += 1;
                        }
                        DirectGetErrorKind::InvalidSubject => {
                            // No more messages or invalid request
                            break;
                        }
                        _ => {
                            // For other errors, check if we have any events
                            if events.is_empty() {
                                return Err(AdapterError::Transient(e.to_string()));
                            }
                            break;
                        }
                    }
                }
            }
        }

        let has_more = events.len() > limit;
        let events: Vec<_> = events.into_iter().take(limit).collect();
        // Prefer the last *seen* sequence over the last successfully
        // deserialized event id. Otherwise a run of trailing corrupt
        // entries leaves the cursor stuck, re-fetching them forever
        // (analog of the Redis adapter's `last_seen_seq` fix for the
        // JetStream path).
        let next_id = last_seen_seq
            .map(|s| s.to_string())
            .or_else(|| events.last().map(|e| e.id.clone()));

        Ok(ShardPollResult {
            events,
            next_id,
            has_more,
        })
    }

    fn name(&self) -> &'static str {
        "jetstream"
    }

    async fn is_healthy(&self) -> bool {
        if !self.initialized.load(Ordering::Acquire) {
            return false;
        }

        if let Some(client) = &self.client {
            // Check connection state
            matches!(
                client.connection_state(),
                async_nats::connection::State::Connected
            )
        } else {
            false
        }
    }
}

/// Check if a NATS publish error is transient (retryable).
///
/// Enumerates the retryable kinds explicitly rather than treating
/// every error other than `WrongLastSequence` as retryable. The
/// permissive default amplified misconfiguration into infinite
/// retry storms — `StreamNotFound` and the `WrongLast*` variants
/// are structural problems that do not become recoverable on retry.
fn is_transient_error(e: &async_nats::jetstream::context::PublishError) -> bool {
    use async_nats::jetstream::context::PublishErrorKind;
    match e.kind() {
        // Network / connection / timing / backpressure — retry is meaningful.
        PublishErrorKind::TimedOut
        | PublishErrorKind::BrokenPipe
        | PublishErrorKind::MaxAckPending
        | PublishErrorKind::Other => true,
        // Structural failures: missing stream and optimistic-concurrency
        // mismatches don't fix themselves under retry.
        PublishErrorKind::StreamNotFound
        | PublishErrorKind::WrongLastMessageId
        | PublishErrorKind::WrongLastSequence => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_event() {
        let event =
            InternalEvent::from_value(json!({"token": "hello", "index": 42}), 1702123456789, 3);

        let buffer = JetStreamAdapter::serialize_event(&event).unwrap();

        let parsed: serde_json::Value = serde_json::from_slice(&buffer).unwrap();
        assert_eq!(parsed["t"], 1702123456789u64);
        assert_eq!(parsed["s"], 3);
        assert_eq!(parsed["r"]["token"], "hello");
        assert_eq!(parsed["r"]["index"], 42);
    }

    #[test]
    fn test_deserialize_event() {
        let data = br#"{"r":{"token":"world"},"t":9999,"s":7}"#;
        let event = JetStreamAdapter::deserialize_event(42, data).unwrap();

        assert_eq!(event.id, "42");
        assert_eq!(event.insertion_ts, 9999);
        assert_eq!(event.shard_id, 7);
        let raw: serde_json::Value = serde_json::from_slice(&event.raw).unwrap();
        assert_eq!(raw["token"], "world");
    }

    #[test]
    fn test_stream_name() {
        let config = JetStreamAdapterConfig::new("nats://localhost:4222").with_prefix("myapp");
        let adapter = JetStreamAdapter::new(config).unwrap();

        assert_eq!(adapter.stream_name(0), "myapp_shard_0");
        assert_eq!(adapter.stream_name(15), "myapp_shard_15");
    }

    #[test]
    fn test_subject() {
        let config = JetStreamAdapterConfig::new("nats://localhost:4222").with_prefix("myapp");
        let adapter = JetStreamAdapter::new(config).unwrap();

        assert_eq!(adapter.subject(0), "myapp.shard.0");
        assert_eq!(adapter.subject(15), "myapp.shard.15");
    }

    /// Regression: BUG_REPORT.md #10 — `is_transient_error` previously
    /// classified every error other than `WrongLastSequence` as
    /// retryable, which meant config errors like `StreamNotFound`
    /// triggered infinite retry storms. The fix enumerates retryable
    /// kinds explicitly and treats structural failures as fatal.
    #[test]
    fn is_transient_error_classifies_kinds() {
        use async_nats::jetstream::context::{PublishError, PublishErrorKind};

        // Retryable: network / timing / backpressure.
        assert!(is_transient_error(&PublishError::new(
            PublishErrorKind::TimedOut
        )));
        assert!(is_transient_error(&PublishError::new(
            PublishErrorKind::BrokenPipe
        )));
        assert!(is_transient_error(&PublishError::new(
            PublishErrorKind::MaxAckPending
        )));
        assert!(is_transient_error(&PublishError::new(
            PublishErrorKind::Other
        )));

        // Fatal: structural / config / concurrency.
        assert!(!is_transient_error(&PublishError::new(
            PublishErrorKind::StreamNotFound
        )));
        assert!(!is_transient_error(&PublishError::new(
            PublishErrorKind::WrongLastMessageId
        )));
        assert!(!is_transient_error(&PublishError::new(
            PublishErrorKind::WrongLastSequence
        )));
    }
}
