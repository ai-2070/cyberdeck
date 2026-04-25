//! Redis Streams adapter for durable event storage.
//!
//! This adapter uses Redis Streams (XADD/XRANGE) for persistent storage.
//!
//! # Design
//!
//! - Each shard maps to one Redis Stream: `{prefix}:shard:{shard_id}`
//! - Writes use pipelined XADD for high throughput
//! - Reads use XRANGE with exclusive cursors for efficient pagination
//! - Reusable serialization buffers to avoid per-event allocation
//!
//! # Throughput Expectations
//!
//! Redis throughput is LOWER than ingestion throughput:
//! - Ingestion: 10M-100M events/sec (in-memory)
//! - Redis: 100K-500K events/sec (network-bound)
//!
//! The batch aggregation layer smooths bursts before they reach Redis.

use async_trait::async_trait;
use bytes::Bytes;
use redis::aio::ConnectionManager;
use redis::{Client, RedisError, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::adapter::{Adapter, ShardPollResult};
use crate::config::RedisAdapterConfig;
use crate::error::AdapterError;
use crate::event::{Batch, InternalEvent, StoredEvent};

/// Redis Streams adapter.
pub struct RedisAdapter {
    /// Redis client.
    client: Client,
    /// Connection manager (pooled connections).
    conn: Option<ConnectionManager>,
    /// Configuration.
    config: RedisAdapterConfig,
    /// Whether the adapter has been initialized.
    initialized: AtomicBool,
    /// Interned stream keys indexed by shard id. Avoids rebuilding
    /// `"{prefix}:shard:{n}"` on every `on_batch` / `poll_shard`.
    /// `RwLock<Vec<_>>` because shards can be added at runtime via
    /// dynamic scaling; growth is rare and amortized.
    stream_keys: parking_lot::RwLock<Vec<Arc<str>>>,
}

impl RedisAdapter {
    /// Create a new Redis adapter.
    pub fn new(config: RedisAdapterConfig) -> Result<Self, AdapterError> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| AdapterError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            conn: None,
            config,
            initialized: AtomicBool::new(false),
            stream_keys: parking_lot::RwLock::new(Vec::new()),
        })
    }

    /// Get the stream key for a shard, populating the cache on first
    /// access for that shard id.
    #[inline]
    fn stream_key(&self, shard_id: u16) -> Arc<str> {
        let idx = shard_id as usize;
        // Fast path: cache hit under read lock.
        if let Some(k) = self.stream_keys.read().get(idx) {
            return k.clone();
        }
        // Slow path: lazily fill the cache up to and including `idx`.
        let mut cache = self.stream_keys.write();
        while cache.len() <= idx {
            let id = cache.len();
            cache.push(Arc::from(
                format!("{}:shard:{}", self.config.prefix, id).as_str(),
            ));
        }
        cache[idx].clone()
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
    ///
    /// Borrows `id` so the caller can defer the owned-String
    /// allocation until success. Uses `RawValue` to slice the `r`
    /// field directly out of the stored bytes — no full JSON tree
    /// allocation, no re-serialize.
    fn deserialize_event(id: &str, data: &[u8]) -> Result<StoredEvent, AdapterError> {
        #[derive(serde::Deserialize)]
        struct StoredFormat<'a> {
            #[serde(borrow)]
            r: &'a serde_json::value::RawValue,
            #[serde(default)]
            t: u64,
            #[serde(default)]
            s: u16,
        }

        let parsed: StoredFormat =
            serde_json::from_slice(data).map_err(|e| AdapterError::Serialization(e.to_string()))?;

        let raw_bytes = Bytes::copy_from_slice(parsed.r.get().as_bytes());

        Ok(StoredEvent::new(
            id.to_string(),
            raw_bytes,
            parsed.t,
            parsed.s,
        ))
    }

    /// Get a connection (with error handling).
    async fn get_conn(&self) -> Result<ConnectionManager, AdapterError> {
        self.conn
            .clone()
            .ok_or_else(|| AdapterError::Connection("adapter not initialized".into()))
    }
}

impl std::fmt::Debug for RedisAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisAdapter")
            .field("url", &self.config.url)
            .field("prefix", &self.config.prefix)
            .field("initialized", &self.initialized.load(Ordering::Relaxed))
            .finish()
    }
}

#[async_trait]
impl Adapter for RedisAdapter {
    async fn init(&mut self) -> Result<(), AdapterError> {
        let conn = self
            .client
            .get_connection_manager()
            .await
            .map_err(|e| AdapterError::Connection(e.to_string()))?;

        // Test the connection
        let mut test_conn = conn.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut test_conn)
            .await
            .map_err(|e| AdapterError::Connection(e.to_string()))?;

        self.conn = Some(conn);
        self.initialized.store(true, Ordering::Release);

        tracing::info!(
            adapter = "redis",
            url = %self.config.url,
            prefix = %self.config.prefix,
            "Redis adapter initialized"
        );

        Ok(())
    }

    async fn on_batch(&self, batch: Batch) -> Result<(), AdapterError> {
        if batch.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_conn().await?;
        let stream_key = self.stream_key(batch.shard_id);

        // Build pipeline with serialized events
        // Serialize all events first (no await while holding data)
        let serialized: Vec<Vec<u8>> = batch
            .events
            .iter()
            .map(Self::serialize_event)
            .collect::<Result<Vec<_>, _>>()?;

        // Build atomic pipeline (MULTI/EXEC) so retries are safe —
        // either all XADDs succeed or none do.
        let mut pipe = redis::pipe();
        pipe.atomic();

        for data in &serialized {
            // Build XADD command
            let mut cmd = redis::cmd("XADD");
            cmd.arg(&*stream_key);

            // Add MAXLEN if configured
            if let Some(max_len) = self.config.max_stream_len {
                cmd.arg("MAXLEN").arg("~").arg(max_len);
            }

            cmd.arg("*"); // Auto-generate ID
            cmd.arg("d").arg(data.as_slice()); // "d" = data field

            pipe.add_command(cmd);
        }

        // Execute pipeline with command timeout
        let fut = pipe.query_async::<()>(&mut conn);
        tokio::time::timeout(self.config.command_timeout, fut)
            .await
            .map_err(|_| AdapterError::Transient("Redis command timeout".into()))?
            .map_err(|e: RedisError| {
                if is_transient_error(&e) {
                    AdapterError::Transient(e.to_string())
                } else {
                    AdapterError::Fatal(e.to_string())
                }
            })?;

        tracing::trace!(
            shard_id = batch.shard_id,
            event_count = batch.events.len(),
            "Batch written to Redis"
        );

        Ok(())
    }

    async fn flush(&self) -> Result<(), AdapterError> {
        // Redis writes are synchronous in the pipeline, nothing to flush
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), AdapterError> {
        self.initialized.store(false, Ordering::Release);
        // ConnectionManager handles cleanup automatically
        tracing::info!(adapter = "redis", "Redis adapter shut down");
        Ok(())
    }

    async fn poll_shard(
        &self,
        shard_id: u16,
        from_id: Option<&str>,
        limit: usize,
    ) -> Result<ShardPollResult, AdapterError> {
        let mut conn = self.get_conn().await?;
        let stream_key = self.stream_key(shard_id);

        // Use exclusive range start to avoid re-reading the last event
        // Redis XRANGE is inclusive, so we use "(" prefix for exclusive
        let start = from_id
            .map(|id| format!("({}", id)) // Exclusive: "(1702123456789-0"
            .unwrap_or_else(|| "-".to_string()); // "-" = from beginning

        // Fetch one extra to detect has_more
        let fetch_limit = limit + 1;

        // XRANGE key start + COUNT limit
        // Returns array of [id, [field, value, field, value, ...]]
        let results: Value = redis::cmd("XRANGE")
            .arg(&*stream_key)
            .arg(&start)
            .arg("+") // To end
            .arg("COUNT")
            .arg(fetch_limit)
            .query_async(&mut conn)
            .await
            .map_err(|e| AdapterError::Transient(e.to_string()))?;

        // Parse results manually
        let mut events = Vec::with_capacity(limit);

        if let Value::Array(entries) = results {
            for entry in entries.iter().take(limit) {
                if let Value::Array(parts) = entry {
                    if parts.len() >= 2 {
                        // First element is the ID. Borrow it as `&str`
                        // until we know we'll keep the event — defers
                        // the owned `String` allocation to the success
                        // path inside `deserialize_event`.
                        let id: std::borrow::Cow<str> = match &parts[0] {
                            Value::BulkString(bytes) => String::from_utf8_lossy(bytes),
                            Value::SimpleString(s) => std::borrow::Cow::Borrowed(s.as_str()),
                            _ => continue,
                        };

                        // Second element is array of field-value pairs.
                        if let Value::Array(ref fields) = parts[1] {
                            // Find the "d" field. Compare against the
                            // byte literal directly — no allocation
                            // for what is otherwise a constant-name
                            // probe on every entry.
                            let mut i = 0;
                            while i + 1 < fields.len() {
                                let is_data_field = match &fields[i] {
                                    Value::BulkString(bytes) => bytes.as_slice() == b"d",
                                    Value::SimpleString(s) => s == "d",
                                    _ => false,
                                };

                                if is_data_field {
                                    if let Value::BulkString(data) = &fields[i + 1] {
                                        match Self::deserialize_event(&id, data) {
                                            Ok(event) => events.push(event),
                                            Err(e) => {
                                                tracing::warn!(
                                                    stream = %stream_key,
                                                    id = %id,
                                                    error = %e,
                                                    "Failed to deserialize event, skipping"
                                                );
                                            }
                                        }
                                    }
                                    break;
                                }
                                i += 2;
                            }
                        }
                    }
                }
            }

            let has_more = entries.len() > limit;
            let next_id = events.last().map(|e| e.id.clone());

            Ok(ShardPollResult {
                events,
                next_id,
                has_more,
            })
        } else {
            Ok(ShardPollResult::empty())
        }
    }

    fn name(&self) -> &'static str {
        "redis"
    }

    async fn is_healthy(&self) -> bool {
        if !self.initialized.load(Ordering::Acquire) {
            return false;
        }

        if let Ok(mut conn) = self.get_conn().await {
            redis::cmd("PING")
                .query_async::<String>(&mut conn)
                .await
                .is_ok()
        } else {
            false
        }
    }
}

/// Check if a Redis error is transient (retryable).
fn is_transient_error(e: &RedisError) -> bool {
    use redis::ErrorKind;
    match e.kind() {
        // I/O errors are always transient
        ErrorKind::Io => true,
        // Cluster connection issues are transient
        ErrorKind::ClusterConnectionNotFound => true,
        // Server errors may be transient - check the message
        ErrorKind::Server(_) => {
            let msg = e.to_string().to_uppercase();
            msg.contains("LOADING")
                || msg.contains("BUSY")
                || msg.contains("TRYAGAIN")
                || msg.contains("MASTERDOWN")
        }
        _ => false,
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

        let buffer = RedisAdapter::serialize_event(&event).unwrap();

        let parsed: serde_json::Value = serde_json::from_slice(&buffer).unwrap();
        assert_eq!(parsed["t"], 1702123456789u64);
        assert_eq!(parsed["s"], 3);
        assert_eq!(parsed["r"]["token"], "hello");
        assert_eq!(parsed["r"]["index"], 42);
    }

    #[test]
    fn test_deserialize_event() {
        let data = br#"{"r":{"token":"world"},"t":9999,"s":7}"#;
        let event = RedisAdapter::deserialize_event("1702123456789-0", data).unwrap();

        assert_eq!(event.id, "1702123456789-0");
        assert_eq!(event.insertion_ts, 9999);
        assert_eq!(event.shard_id, 7);
        let raw: serde_json::Value = serde_json::from_slice(&event.raw).unwrap();
        assert_eq!(raw["token"], "world");
    }

    #[test]
    fn test_stream_key() {
        let config = RedisAdapterConfig::new("redis://localhost:6379").with_prefix("myapp");
        let adapter = RedisAdapter::new(config).unwrap();

        assert_eq!(&*adapter.stream_key(0), "myapp:shard:0");
        assert_eq!(&*adapter.stream_key(15), "myapp:shard:15");
        // Repeat access should hit the interned cache rather than
        // re-running `format!`.
        assert_eq!(&*adapter.stream_key(0), "myapp:shard:0");
    }
}
