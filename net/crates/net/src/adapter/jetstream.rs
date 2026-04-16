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
use std::sync::atomic::{AtomicBool, Ordering};

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
    /// Stream cache (shard_id -> stream).
    streams: Mutex<HashMap<u16, Stream>>,
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
        let value: serde_json::Value =
            serde_json::from_slice(data).map_err(|e| AdapterError::Serialization(e.to_string()))?;

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
    async fn get_or_create_stream(&self, shard_id: u16) -> Result<Stream, AdapterError> {
        let stream_name = self.stream_name(shard_id);

        // Check cache first
        {
            let streams = self.streams.lock();
            if let Some(stream) = streams.get(&shard_id) {
                return Ok(stream.clone());
            }
        }

        let js = self
            .jetstream
            .as_ref()
            .ok_or_else(|| AdapterError::Connection("adapter not initialized".into()))?;

        // Try to get existing stream
        let stream = match js.get_stream(&stream_name).await {
            Ok(stream) => stream,
            Err(_) => {
                // Create new stream
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
                    .map_err(|e| AdapterError::Connection(e.to_string()))?
            }
        };

        // Cache the stream
        {
            let mut streams = self.streams.lock();
            streams.insert(shard_id, stream.clone());
        }

        Ok(stream)
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

        let subject = self.subject(batch.shard_id);

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
        for (i, data) in serialized.into_iter().enumerate() {
            let msg_id = format!("{}:{}:{}", batch.shard_id, batch.sequence_start, i);
            let mut headers = async_nats::HeaderMap::new();
            headers.insert("Nats-Msg-Id", msg_id.as_str());

            js.publish_with_headers(subject.clone(), headers, data.into())
                .await
                .map_err(|e| {
                    if is_transient_error(&e) {
                        AdapterError::Transient(e.to_string())
                    } else {
                        AdapterError::Fatal(e.to_string())
                    }
                })?
                .await
                .map_err(|e| AdapterError::Transient(e.to_string()))?;
        }

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

        // Client cleanup is automatic on drop
        tracing::info!(adapter = "jetstream", "JetStream adapter shut down");
        Ok(())
    }

    async fn poll_shard(
        &self,
        shard_id: u16,
        from_id: Option<&str>,
        limit: usize,
    ) -> Result<ShardPollResult, AdapterError> {
        let stream = self.get_or_create_stream(shard_id).await?;

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
        // streams with large gaps (deletions, compaction).
        let max_seq = match stream.info().await {
            Ok(info) => info.state.last_sequence,
            Err(_) => start_seq.saturating_add(fetch_limit as u64 * 10),
        };

        // Use direct get to fetch messages by sequence
        // Use while loop so gaps don't consume our fetch count
        while events.len() < fetch_limit {
            if current_seq > max_seq {
                // Searched too far without finding enough events
                break;
            }

            match stream.direct_get(current_seq).await {
                Ok(msg) => {
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
        let next_id = events.last().map(|e| e.id.clone());

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

/// Check if a NATS error is transient (retryable).
fn is_transient_error(e: &async_nats::jetstream::context::PublishError) -> bool {
    // Most JetStream errors are transient (network, timeout)
    // Only configuration errors are fatal
    !matches!(
        e.kind(),
        async_nats::jetstream::context::PublishErrorKind::WrongLastSequence
    )
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
}
