//! Node.js bindings for Net event bus.
//!
//! Provides high-performance event ingestion and consumption for Node.js/TypeScript.

#[cfg(feature = "cortex")]
mod cortex;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use arc_swap::ArcSwapOption;

use net::{
    config::{AdapterConfig, BackpressureMode, EventBusConfig},
    consumer::Ordering,
    event::RawEvent,
    ConsumeRequest, EventBus, Filter,
};

/// Pre-computed hash for events that will be reused.
/// Store on JS side and pass back to avoid recomputing xxhash.
///
/// Hash is exposed as BigInt to preserve full 64-bit xxh3 precision.
#[napi(object)]
pub struct HashedEvent {
    pub data: Buffer,
    pub hash: BigInt,
}

#[cfg(feature = "redis")]
use net::config::RedisAdapterConfig;

#[cfg(feature = "jetstream")]
use net::config::JetStreamAdapterConfig;

#[cfg(feature = "net")]
use net::adapter::net::{NetAdapterConfig, ReliabilityConfig, StaticKeypair};

/// Redis adapter configuration.
#[napi(object)]
pub struct RedisOptions {
    /// Redis connection URL (e.g., "redis://localhost:6379")
    pub url: String,
    /// Stream key prefix (default: "net")
    pub prefix: Option<String>,
    /// Maximum commands per pipeline (default: 1000)
    pub pipeline_size: Option<u32>,
    /// Connection pool size (default: num_shards)
    pub pool_size: Option<u32>,
    /// Connection timeout in milliseconds (default: 5000)
    pub connect_timeout_ms: Option<u32>,
    /// Command timeout in milliseconds (default: 1000)
    pub command_timeout_ms: Option<u32>,
    /// Maximum stream length, unlimited if not set
    pub max_stream_len: Option<u32>,
}

/// NATS JetStream adapter configuration.
#[napi(object)]
pub struct JetStreamOptions {
    /// NATS server URL (e.g., "nats://localhost:4222")
    pub url: String,
    /// Stream name prefix (default: "net")
    pub prefix: Option<String>,
    /// Connection timeout in milliseconds (default: 5000)
    pub connect_timeout_ms: Option<u32>,
    /// Request timeout in milliseconds (default: 5000)
    pub request_timeout_ms: Option<u32>,
    /// Maximum messages per stream, unlimited if not set
    pub max_messages: Option<i64>,
    /// Maximum bytes per stream, unlimited if not set
    pub max_bytes: Option<i64>,
    /// Maximum age for messages in milliseconds, unlimited if not set
    pub max_age_ms: Option<u32>,
    /// Number of stream replicas (default: 1)
    pub replicas: Option<u32>,
}

/// Net keypair for encrypted UDP transport.
#[napi(object)]
pub struct NetKeypair {
    /// Hex-encoded 32-byte public key
    pub public_key: String,
    /// Hex-encoded 32-byte secret key
    pub secret_key: String,
}

/// Net adapter configuration for encrypted UDP transport.
#[napi(object)]
pub struct NetOptions {
    /// Local bind address (e.g., "127.0.0.1:9000")
    pub bind_addr: String,
    /// Remote peer address (e.g., "127.0.0.1:9001")
    pub peer_addr: String,
    /// Hex-encoded 32-byte pre-shared key
    pub psk: String,
    /// Connection role: "initiator" or "responder"
    pub role: String,
    /// Hex-encoded peer's static public key (required for initiator)
    pub peer_public_key: Option<String>,
    /// Hex-encoded secret key (required for responder)
    pub secret_key: Option<String>,
    /// Hex-encoded public key (required for responder)
    pub public_key: Option<String>,
    /// Reliability mode: "none" (default), "light", or "full"
    pub reliability: Option<String>,
    /// Heartbeat interval in milliseconds (default: 5000)
    pub heartbeat_interval_ms: Option<u32>,
    /// Session timeout in milliseconds (default: 30000)
    pub session_timeout_ms: Option<u32>,
    /// Enable batched I/O for Linux (default: false)
    pub batched_io: Option<bool>,
    /// Packet pool size (default: 64)
    pub packet_pool_size: Option<u32>,
}

/// Configuration options for creating an EventBus.
#[napi(object)]
pub struct EventBusOptions {
    /// Number of shards (defaults to CPU core count)
    pub num_shards: Option<u32>,
    /// Ring buffer capacity per shard (must be power of 2)
    pub ring_buffer_capacity: Option<u32>,
    /// Backpressure mode: "drop_newest", "drop_oldest", "fail_producer"
    pub backpressure_mode: Option<String>,
    /// Redis adapter configuration (if not set, uses in-memory noop adapter)
    pub redis: Option<RedisOptions>,
    /// NATS JetStream adapter configuration (alternative to Redis)
    pub jetstream: Option<JetStreamOptions>,
    /// Net adapter configuration for encrypted UDP transport
    pub net: Option<NetOptions>,
}

/// Options for polling events.
#[napi(object)]
pub struct PollOptions {
    /// Maximum number of events to return
    pub limit: u32,
    /// Optional cursor to resume from
    pub cursor: Option<String>,
    /// Optional JSON filter expression
    pub filter: Option<String>,
    /// Event ordering: "none" (default, fastest) or "insertion_ts" (cross-shard ordering)
    pub ordering: Option<String>,
}

/// A stored event returned from polling.
#[napi(object)]
pub struct StoredEvent {
    /// Backend-specific event ID
    pub id: String,
    /// Raw payload as UTF-8. When the payload is not valid UTF-8
    /// (binary payloads), this is the empty string and the original
    /// bytes are in `raw_bytes` instead.
    pub raw: String,
    /// Raw payload bytes. Always populated — consumers that need binary
    /// fidelity should prefer this over `raw`. For UTF-8 payloads the
    /// two fields carry the same content in different representations.
    pub raw_bytes: Buffer,
    /// Insertion timestamp (nanoseconds)
    pub insertion_ts: i64,
    /// Shard ID
    pub shard_id: u32,
}

/// Poll response containing events and cursor.
#[napi(object)]
pub struct PollResponse {
    /// List of events
    pub events: Vec<StoredEvent>,
    /// Cursor for pagination (pass to next poll)
    pub next_id: Option<String>,
    /// Whether there are more events available
    pub has_more: bool,
}

/// Ingestion result.
#[napi(object)]
pub struct IngestResult {
    /// Shard the event was assigned to
    pub shard_id: u32,
    /// Insertion timestamp
    pub timestamp: i64,
}

/// Ingestion statistics.
#[napi(object)]
pub struct Stats {
    /// Total events ingested
    pub events_ingested: i64,
    /// Events dropped due to backpressure
    pub events_dropped: i64,
}

/// High-performance event bus for Node.js.
///
/// Example usage:
/// ```typescript
/// import { Net } from '@ai2070/net';
///
/// const bus = await Net.create({ numShards: 4 });
///
/// // Fast sync ingestion (no async overhead)
/// bus.ingestRawSync('{"token": "hello", "index": 0}');
///
/// // Or batch for maximum throughput
/// bus.ingestRawBatchSync(['{"a":1}', '{"a":2}']);
///
/// // Poll events (async)
/// const response = await bus.poll({ limit: 100 });
///
/// await bus.shutdown();
/// ```
#[napi]
pub struct Net {
    /// Lock-free bus handle using ArcSwap for maximum performance.
    /// ArcSwapOption allows atomic load/store without mutex overhead.
    bus: Arc<ArcSwapOption<EventBus>>,
}

#[napi]
impl Net {
    /// Create a new Net event bus.
    #[napi(factory)]
    pub async fn create(options: Option<EventBusOptions>) -> Result<Net> {
        let config = build_config(options)?;

        let bus = EventBus::new(config)
            .await
            .map_err(|e| Error::from_reason(format!("Failed to create EventBus: {}", e)))?;

        Ok(Net {
            bus: Arc::new(ArcSwapOption::from_pointee(bus)),
        })
    }

    // =========================================================================
    // ULTRA FAST PATH - Minimal overhead methods
    // =========================================================================

    /// Pre-compute hash for an event buffer.
    ///
    /// Use this for events that will be ingested multiple times (e.g., templates).
    /// The returned HashedEvent can be passed to pushHashed() to skip hash computation.
    #[napi]
    pub fn prehash(&self, data: Buffer) -> HashedEvent {
        let hash = xxhash_rust::xxh3::xxh3_64(data.as_ref());
        HashedEvent {
            data,
            hash: BigInt::from(hash),
        }
    }

    /// Ingest with a pre-computed hash (fastest single-event path).
    ///
    /// Pass the buffer and its pre-computed xxhash (as BigInt) to skip hash
    /// computation. BigInt preserves full 64-bit precision.
    #[napi]
    pub fn push_with_hash(&self, data: Buffer, hash: BigInt) -> bool {
        let guard = self.bus.load();
        let bus = match guard.as_ref() {
            Some(b) => b,
            None => return false,
        };

        let (sign, value, lossless) = hash.get_u64();
        // Reject negative or out-of-range BigInts to avoid silent wrong-shard routing
        if sign || !lossless {
            return false;
        }
        let raw =
            RawEvent::from_bytes_with_hash(bytes::Bytes::copy_from_slice(data.as_ref()), value);
        bus.ingest_raw(raw).is_ok()
    }

    /// Ingest raw bytes (fastest single-event path).
    ///
    /// Accepts a Buffer directly from JS - no string conversion overhead.
    /// Returns true on success, false on failure.
    #[napi]
    pub fn push(&self, data: Buffer) -> bool {
        let guard = self.bus.load();
        let bus = match guard.as_ref() {
            Some(b) => b,
            None => return false,
        };

        let raw = RawEvent::from_bytes(bytes::Bytes::copy_from_slice(data.as_ref()));
        bus.ingest_raw(raw).is_ok()
    }

    /// Batch push raw buffers (fastest batch path).
    ///
    /// Each buffer is one event. Returns count of successfully ingested.
    #[napi]
    pub fn push_batch(&self, events: Vec<Buffer>) -> u32 {
        let guard = self.bus.load();
        let bus = match guard.as_ref() {
            Some(b) => b,
            None => return 0,
        };

        let raw_events: Vec<RawEvent> = events
            .iter()
            .map(|b| RawEvent::from_bytes(bytes::Bytes::copy_from_slice(b.as_ref())))
            .collect();
        bus.ingest_raw_batch(raw_events) as u32
    }

    // =========================================================================
    // SYNC FAST PATH - Use these for maximum throughput
    // =========================================================================

    /// Ingest a raw JSON string synchronously (fastest path).
    ///
    /// This is the recommended method for high-throughput ingestion.
    /// No async overhead - directly calls into Rust core.
    #[napi]
    pub fn ingest_raw_sync(&self, json: String) -> Result<IngestResult> {
        let guard = self.bus.load();
        let bus = guard
            .as_ref()
            .ok_or_else(|| Error::from_reason("EventBus has been shut down"))?;

        let raw = RawEvent::from_str(&json);
        let (shard_id, ts) = bus
            .ingest_raw(raw)
            .map_err(|e| Error::from_reason(format!("Ingestion failed: {}", e)))?;

        Ok(IngestResult {
            shard_id: shard_id as u32,
            timestamp: ts as i64,
        })
    }

    /// Ingest multiple raw JSON strings in a batch synchronously.
    ///
    /// Most efficient method for bulk ingestion - single FFI boundary crossing.
    #[napi]
    pub fn ingest_raw_batch_sync(&self, events: Vec<String>) -> Result<u32> {
        let guard = self.bus.load();
        let bus = guard
            .as_ref()
            .ok_or_else(|| Error::from_reason("EventBus has been shut down"))?;

        let raw_events: Vec<RawEvent> = events.iter().map(|s| RawEvent::from_str(s)).collect();
        let count = bus.ingest_raw_batch(raw_events);

        Ok(count as u32)
    }

    /// Fire-and-forget ingestion - returns immediately, no result.
    ///
    /// Fastest possible path when you don't need confirmation.
    #[napi]
    pub fn ingest_fire(&self, json: String) -> bool {
        let guard = self.bus.load();
        let bus = match guard.as_ref() {
            Some(b) => b,
            None => return false,
        };

        let raw = RawEvent::from_str(&json);
        bus.ingest_raw(raw).is_ok()
    }

    /// Fire-and-forget batch ingestion - returns count only.
    #[napi]
    pub fn ingest_batch_fire(&self, events: Vec<String>) -> u32 {
        let guard = self.bus.load();
        let bus = match guard.as_ref() {
            Some(b) => b,
            None => return 0,
        };

        let raw_events: Vec<RawEvent> = events.iter().map(|s| RawEvent::from_str(s)).collect();
        bus.ingest_raw_batch(raw_events) as u32
    }

    // =========================================================================
    // ASYNC METHODS - For compatibility, use sync methods for perf
    // =========================================================================

    /// Ingest a raw JSON string (async version).
    ///
    /// For maximum performance, use `ingestRawSync` instead.
    #[napi]
    pub async fn ingest_raw(&self, json: String) -> Result<IngestResult> {
        self.ingest_raw_sync(json)
    }

    /// Ingest a JavaScript object (convenience method).
    ///
    /// The object is serialized to JSON before ingestion.
    /// For maximum performance, use `ingestRawSync` with pre-serialized JSON.
    #[napi]
    pub async fn ingest(&self, event: String) -> Result<IngestResult> {
        // Accept JSON string, parse to validate, then use raw path
        let _: serde_json::Value = serde_json::from_str(&event)
            .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;
        self.ingest_raw_sync(event)
    }

    /// Ingest multiple raw JSON strings in a batch (async version).
    ///
    /// For maximum performance, use `ingestRawBatchSync` instead.
    #[napi]
    pub async fn ingest_raw_batch(&self, events: Vec<String>) -> Result<u32> {
        self.ingest_raw_batch_sync(events)
    }

    /// Poll events from the bus.
    #[napi]
    pub async fn poll(&self, options: PollOptions) -> Result<PollResponse> {
        // Load the Arc - this is lock-free with ArcSwap
        let bus_arc = self
            .bus
            .load_full()
            .ok_or_else(|| Error::from_reason("EventBus has been shut down"))?;

        let mut request = ConsumeRequest::new(options.limit as usize);

        if let Some(cursor) = options.cursor {
            request = request.from(cursor);
        }

        if let Some(filter_json) = options.filter {
            let filter: Filter = serde_json::from_str(&filter_json)
                .map_err(|e| Error::from_reason(format!("Invalid filter: {}", e)))?;
            request = request.filter(filter);
        }

        if let Some(ordering) = options.ordering {
            let ord = match ordering.as_str() {
                "none" => Ordering::None,
                "insertion_ts" => Ordering::InsertionTs,
                _ => {
                    return Err(Error::from_reason(format!(
                        "Invalid ordering: {}. Use 'none' or 'insertion_ts'",
                        ordering
                    )));
                }
            };
            request = request.ordering(ord);
        }

        let response = bus_arc
            .poll(request)
            .await
            .map_err(|e| Error::from_reason(format!("Poll failed: {}", e)))?;

        let events: Vec<StoredEvent> = response
            .events
            .into_iter()
            .map(|e| {
                let raw = e.raw_str().unwrap_or("").to_string();
                let raw_bytes = Buffer::from(e.raw.to_vec());
                StoredEvent {
                    id: e.id,
                    raw,
                    raw_bytes,
                    insertion_ts: e.insertion_ts as i64,
                    shard_id: e.shard_id as u32,
                }
            })
            .collect();

        Ok(PollResponse {
            events,
            next_id: response.next_id,
            has_more: response.has_more,
        })
    }

    /// Get the number of active shards.
    #[napi]
    pub fn num_shards(&self) -> Result<u32> {
        let guard = self.bus.load();
        let bus = guard
            .as_ref()
            .ok_or_else(|| Error::from_reason("EventBus has been shut down"))?;

        Ok(bus.num_shards() as u32)
    }

    /// Get ingestion statistics.
    #[napi]
    pub fn stats(&self) -> Result<Stats> {
        let guard = self.bus.load();
        let bus = guard
            .as_ref()
            .ok_or_else(|| Error::from_reason("EventBus has been shut down"))?;

        let stats = bus.stats();
        Ok(Stats {
            events_ingested: stats
                .events_ingested
                .load(std::sync::atomic::Ordering::Relaxed)
                .min(i64::MAX as u64) as i64,
            events_dropped: stats
                .events_dropped
                .load(std::sync::atomic::Ordering::Relaxed)
                .min(i64::MAX as u64) as i64,
        })
    }

    /// Flush all pending batches to the backend.
    ///
    /// Call this after ingesting events to ensure they are persisted
    /// before polling.
    #[napi]
    pub async fn flush(&self) -> Result<()> {
        let bus_arc = self
            .bus
            .load_full()
            .ok_or_else(|| Error::from_reason("EventBus has been shut down"))?;

        bus_arc
            .flush()
            .await
            .map_err(|e| Error::from_reason(format!("Flush failed: {}", e)))?;
        Ok(())
    }

    /// Gracefully shutdown the event bus.
    ///
    /// Returns an error if there are outstanding references to the bus
    /// (e.g., from concurrent async operations).
    #[napi]
    pub async fn shutdown(&self) -> Result<()> {
        // Swap out the bus atomically - no lock needed
        let bus_arc = self.bus.swap(None);

        if let Some(bus) = bus_arc {
            // Try to unwrap the Arc - if we're the only holder, we can shutdown
            match Arc::try_unwrap(bus) {
                Ok(bus) => {
                    bus.shutdown()
                        .await
                        .map_err(|e| Error::from_reason(format!("Shutdown failed: {}", e)))?;
                }
                Err(arc) => {
                    // Put the bus back so it isn't permanently lost.
                    // The caller can retry after outstanding operations complete.
                    self.bus.store(Some(arc));
                    return Err(Error::from_reason(
                        "Cannot shutdown: outstanding references to EventBus exist. \
                         Ensure all async operations have completed before calling shutdown()."
                            .to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Generate a new Net keypair for encrypted UDP transport.
///
/// Returns a keypair with hex-encoded public and secret keys.
/// Use this to generate keys for a responder, then share the public key
/// with the initiator.
#[cfg(feature = "net")]
#[napi]
pub fn generate_net_keypair() -> NetKeypair {
    let keypair = StaticKeypair::generate();
    NetKeypair {
        public_key: hex::encode(keypair.public_key()),
        secret_key: hex::encode(keypair.secret_key()),
    }
}

fn build_config(options: Option<EventBusOptions>) -> Result<EventBusConfig> {
    let mut builder = EventBusConfig::builder();

    if let Some(opts) = options {
        if let Some(num_shards) = opts.num_shards {
            let shards: u16 = num_shards.try_into().map_err(|_| {
                Error::from_reason(format!(
                    "num_shards must be <= {}, got {}",
                    u16::MAX,
                    num_shards
                ))
            })?;
            builder = builder.num_shards(shards);
        }
        if let Some(capacity) = opts.ring_buffer_capacity {
            builder = builder.ring_buffer_capacity(capacity as usize);
        }
        if let Some(mode) = opts.backpressure_mode {
            let bp_mode = match mode.as_str() {
                "drop_newest" => BackpressureMode::DropNewest,
                "drop_oldest" => BackpressureMode::DropOldest,
                "fail_producer" => BackpressureMode::FailProducer,
                _ => {
                    return Err(Error::from_reason(format!(
                        "Invalid backpressure mode: {}",
                        mode
                    )));
                }
            };
            builder = builder.backpressure_mode(bp_mode);
        }

        // Configure adapter
        if let Some(redis) = opts.redis {
            #[cfg(feature = "redis")]
            {
                use std::time::Duration;
                let mut redis_config = RedisAdapterConfig::new(&redis.url);
                if let Some(prefix) = redis.prefix {
                    redis_config = redis_config.with_prefix(&prefix);
                }
                if let Some(pipeline_size) = redis.pipeline_size {
                    redis_config = redis_config.with_pipeline_size(pipeline_size as usize);
                }
                if let Some(pool_size) = redis.pool_size {
                    redis_config = redis_config.with_pool_size(pool_size as usize);
                }
                if let Some(connect_timeout_ms) = redis.connect_timeout_ms {
                    redis_config = redis_config
                        .with_connect_timeout(Duration::from_millis(connect_timeout_ms as u64));
                }
                if let Some(command_timeout_ms) = redis.command_timeout_ms {
                    redis_config = redis_config
                        .with_command_timeout(Duration::from_millis(command_timeout_ms as u64));
                }
                if let Some(max_stream_len) = redis.max_stream_len {
                    redis_config = redis_config.with_max_stream_len(max_stream_len as usize);
                }
                builder = builder.adapter(AdapterConfig::Redis(redis_config));
            }
            #[cfg(not(feature = "redis"))]
            {
                let _ = redis;
                return Err(Error::from_reason(
                    "Redis support not enabled. Rebuild with --features redis".to_string(),
                ));
            }
        } else if let Some(jetstream) = opts.jetstream {
            #[cfg(feature = "jetstream")]
            {
                use std::time::Duration;
                let mut js_config = JetStreamAdapterConfig::new(&jetstream.url);
                if let Some(prefix) = jetstream.prefix {
                    js_config = js_config.with_prefix(&prefix);
                }
                if let Some(connect_timeout_ms) = jetstream.connect_timeout_ms {
                    js_config = js_config
                        .with_connect_timeout(Duration::from_millis(connect_timeout_ms as u64));
                }
                if let Some(request_timeout_ms) = jetstream.request_timeout_ms {
                    js_config = js_config
                        .with_request_timeout(Duration::from_millis(request_timeout_ms as u64));
                }
                if let Some(max_messages) = jetstream.max_messages {
                    js_config = js_config.with_max_messages(max_messages);
                }
                if let Some(max_bytes) = jetstream.max_bytes {
                    js_config = js_config.with_max_bytes(max_bytes);
                }
                if let Some(max_age_ms) = jetstream.max_age_ms {
                    js_config = js_config.with_max_age(Duration::from_millis(max_age_ms as u64));
                }
                if let Some(replicas) = jetstream.replicas {
                    js_config = js_config.with_replicas(replicas as usize);
                }
                builder = builder.adapter(AdapterConfig::JetStream(js_config));
            }
            #[cfg(not(feature = "jetstream"))]
            {
                let _ = jetstream;
                return Err(Error::from_reason(
                    "JetStream support not enabled. Rebuild with --features jetstream".to_string(),
                ));
            }
        } else if let Some(net) = opts.net {
            #[cfg(feature = "net")]
            {
                use std::time::Duration;

                let bind_addr: std::net::SocketAddr = net
                    .bind_addr
                    .parse()
                    .map_err(|e| Error::from_reason(format!("Invalid bind_addr: {}", e)))?;

                let peer_addr: std::net::SocketAddr = net
                    .peer_addr
                    .parse()
                    .map_err(|e| Error::from_reason(format!("Invalid peer_addr: {}", e)))?;

                let psk: [u8; 32] = hex::decode(&net.psk)
                    .map_err(|e| Error::from_reason(format!("Invalid psk hex: {}", e)))?
                    .try_into()
                    .map_err(|_| Error::from_reason("psk must be exactly 32 bytes".to_string()))?;

                let mut net_config = match net.role.as_str() {
                    "initiator" => {
                        let peer_pubkey_hex = net.peer_public_key.ok_or_else(|| {
                            Error::from_reason(
                                "peer_public_key is required for initiator".to_string(),
                            )
                        })?;
                        let peer_pubkey: [u8; 32] = hex::decode(&peer_pubkey_hex)
                            .map_err(|e| {
                                Error::from_reason(format!("Invalid peer_public_key hex: {}", e))
                            })?
                            .try_into()
                            .map_err(|_| {
                                Error::from_reason(
                                    "peer_public_key must be exactly 32 bytes".to_string(),
                                )
                            })?;
                        NetAdapterConfig::initiator(bind_addr, peer_addr, psk, peer_pubkey)
                    }
                    "responder" => {
                        let secret_key_hex = net.secret_key.ok_or_else(|| {
                            Error::from_reason("secret_key is required for responder".to_string())
                        })?;
                        let public_key_hex = net.public_key.ok_or_else(|| {
                            Error::from_reason("public_key is required for responder".to_string())
                        })?;
                        let secret_key: [u8; 32] = hex::decode(&secret_key_hex)
                            .map_err(|e| {
                                Error::from_reason(format!("Invalid secret_key hex: {}", e))
                            })?
                            .try_into()
                            .map_err(|_| {
                                Error::from_reason(
                                    "secret_key must be exactly 32 bytes".to_string(),
                                )
                            })?;
                        let public_key: [u8; 32] = hex::decode(&public_key_hex)
                            .map_err(|e| {
                                Error::from_reason(format!("Invalid public_key hex: {}", e))
                            })?
                            .try_into()
                            .map_err(|_| {
                                Error::from_reason(
                                    "public_key must be exactly 32 bytes".to_string(),
                                )
                            })?;
                        let keypair = StaticKeypair::from_keys(secret_key, public_key);
                        NetAdapterConfig::responder(bind_addr, peer_addr, psk, keypair)
                    }
                    _ => {
                        return Err(Error::from_reason(format!(
                            "Invalid role: {}. Use 'initiator' or 'responder'",
                            net.role
                        )));
                    }
                };

                // Apply optional settings
                if let Some(reliability) = net.reliability {
                    net_config = net_config.with_reliability(match reliability.as_str() {
                        "light" => ReliabilityConfig::Light,
                        "full" => ReliabilityConfig::Full,
                        _ => ReliabilityConfig::None,
                    });
                }
                if let Some(interval_ms) = net.heartbeat_interval_ms {
                    net_config = net_config
                        .with_heartbeat_interval(Duration::from_millis(interval_ms as u64));
                }
                if let Some(timeout_ms) = net.session_timeout_ms {
                    net_config =
                        net_config.with_session_timeout(Duration::from_millis(timeout_ms as u64));
                }
                if let Some(batched) = net.batched_io {
                    net_config = net_config.with_batched_io(batched);
                }
                if let Some(pool_size) = net.packet_pool_size {
                    net_config = net_config.with_pool_size(pool_size as usize);
                }

                builder = builder.adapter(AdapterConfig::Net(Box::new(net_config)));
            }
            #[cfg(not(feature = "net"))]
            {
                let _ = net;
                return Err(Error::from_reason(
                    "Net support not enabled. Rebuild with --features net".to_string(),
                ));
            }
        }
    }

    builder
        .build()
        .map_err(|e| Error::from_reason(format!("Invalid configuration: {}", e)))
}

// ============================================================================
// MeshNode bindings
// ============================================================================

#[cfg(feature = "net")]
mod mesh_bindings {
    use super::*;
    use net::adapter::net::{
        EntityKeypair, MeshNode, MeshNodeConfig, Reliability, Stream as CoreStream, StreamConfig,
        StreamError, DEFAULT_STREAM_WINDOW_BYTES,
    };
    use net::adapter::Adapter;
    use std::time::Duration;

    // ─── Stream API type bridges ─────────────────────────────────────

    /// Reliability mode for a stream. Wire value is a plain tag string:
    /// `"fire_and_forget"` (default) or `"reliable"`. Anything else
    /// errors at stream-open time.
    #[napi(object)]
    pub struct StreamOptions {
        /// Caller-chosen `u64` stream id. Stream IDs are opaque; no
        /// range has transport-level meaning.
        pub stream_id: i64,
        /// `"fire_and_forget"` | `"reliable"`. Default: `"fire_and_forget"`.
        pub reliability: Option<String>,
        /// Initial send-credit window in bytes. Defaults to
        /// `DEFAULT_STREAM_WINDOW_BYTES` (64 KB) when unset — v2
        /// backpressure is ON out of the box. Pass `0` to restore the
        /// v1 unbounded-queue behavior for a specific stream.
        pub window_bytes: Option<u32>,
        /// Fair-scheduler weight (1 = equal share). Default: 1.
        pub fairness_weight: Option<u8>,
    }

    /// Handle to an open stream. Opaque to JS callers; pass back to
    /// `sendOnStream` / `sendWithRetry` / `sendBlocking` / `closeStream`.
    #[napi]
    pub struct NetStream {
        peer_node_id: u64,
        stream_id: u64,
        core: CoreStream,
    }

    #[napi]
    impl NetStream {
        /// The peer this stream terminates at.
        #[napi(getter)]
        pub fn peer_node_id(&self) -> Result<i64> {
            i64::try_from(self.peer_node_id).map_err(|_| {
                Error::from_reason(format!(
                    "peer_node_id {} exceeds i64::MAX; JS has no lossless u64",
                    self.peer_node_id
                ))
            })
        }
        /// The caller-chosen stream id.
        #[napi(getter)]
        pub fn stream_id(&self) -> Result<i64> {
            i64::try_from(self.stream_id).map_err(|_| {
                Error::from_reason(format!(
                    "stream_id {} exceeds i64::MAX; JS has no lossless u64",
                    self.stream_id
                ))
            })
        }
    }

    /// Snapshot of per-stream stats.
    ///
    /// u64 fields are exposed as `BigInt` so values outside the JS
    /// safe-integer range (notably `last_activity_ns`, which is
    /// Unix-epoch nanoseconds and always well above `2^53`) don't
    /// lose precision or trip the TS SDK's safe-integer guard. The
    /// u32 fields are safe as regular numbers.
    #[napi(object)]
    pub struct NetStreamStats {
        pub tx_seq: BigInt,
        pub rx_seq: BigInt,
        pub inbound_pending: BigInt,
        pub last_activity_ns: BigInt,
        pub active: bool,
        pub backpressure_events: BigInt,
        pub tx_credit_remaining: u32,
        pub tx_window: u32,
        pub credit_grants_received: BigInt,
        pub credit_grants_sent: BigInt,
    }

    /// Prefix convention for JS SDK error-class routing. The TS wrapper
    /// matches on the message prefix to re-throw a `BackpressureError`
    /// or `NotConnectedError` subclass. The rest of the message is
    /// human-readable detail. Keep these strings stable — they are part
    /// of the SDK contract.
    pub(crate) const ERR_BACKPRESSURE_PREFIX: &str = "stream would block";
    pub(crate) const ERR_NOT_CONNECTED_PREFIX: &str = "stream not connected";

    pub(crate) fn stream_error_to_napi(e: StreamError) -> Error {
        // Map each variant to a stable, prefix-sniffable message. The
        // TS `sendOnStream` wrapper (`sdk-ts`) inspects this prefix to
        // re-throw `BackpressureError` or `NotConnectedError`.
        match e {
            StreamError::Backpressure => {
                Error::from_reason(format!("{}: stream queue full", ERR_BACKPRESSURE_PREFIX))
            }
            StreamError::NotConnected => {
                Error::from_reason(format!("{}: peer session gone", ERR_NOT_CONNECTED_PREFIX))
            }
            StreamError::Transport(msg) => {
                Error::from_reason(format!("stream transport error: {}", msg))
            }
        }
    }

    pub(crate) fn stream_config_from_opts(opts: &StreamOptions) -> Result<StreamConfig> {
        let reliability = match opts.reliability.as_deref() {
            None | Some("fire_and_forget") => Reliability::FireAndForget,
            Some("reliable") => Reliability::Reliable,
            Some(other) => {
                return Err(Error::from_reason(format!(
                    "unknown reliability mode {:?}; expected \"fire_and_forget\" or \"reliable\"",
                    other
                )));
            }
        };
        Ok(StreamConfig::new()
            .with_reliability(reliability)
            .with_window_bytes(opts.window_bytes.unwrap_or(DEFAULT_STREAM_WINDOW_BYTES))
            .with_fairness_weight(opts.fairness_weight.unwrap_or(1)))
    }

    /// Configuration for creating a MeshNode.
    #[napi(object)]
    pub struct MeshOptions {
        /// Local bind address (e.g., "127.0.0.1:9000")
        pub bind_addr: String,
        /// Hex-encoded 32-byte pre-shared key
        pub psk: String,
        /// Heartbeat interval in milliseconds (default: 5000)
        pub heartbeat_interval_ms: Option<u32>,
        /// Session timeout in milliseconds (default: 30000)
        pub session_timeout_ms: Option<u32>,
        /// Number of inbound shards (default: 4)
        pub num_shards: Option<u32>,
    }

    /// A multi-peer mesh node for Node.js.
    ///
    /// Manages encrypted connections to multiple peers over a single
    /// UDP socket with automatic failure detection and rerouting.
    ///
    /// ```typescript
    /// import { NetMesh } from '@ai2070/net';
    ///
    /// const node = await NetMesh.create({
    ///   bindAddr: '127.0.0.1:9000',
    ///   psk: '0'.repeat(64), // 32-byte hex
    /// });
    ///
    /// console.log('public key:', node.publicKey());
    ///
    /// await node.connect('127.0.0.1:9001', peerPubkey, 0x2222);
    /// node.start();
    ///
    /// node.pushTo('127.0.0.1:9001', Buffer.from('{"token":"hi"}'));
    ///
    /// const events = await node.poll(100);
    ///
    /// await node.shutdown();
    /// ```
    #[napi]
    pub struct NetMesh {
        node: Arc<ArcSwapOption<MeshNode>>,
    }

    #[napi]
    impl NetMesh {
        /// Create a new mesh node.
        #[napi(factory)]
        pub async fn create(options: MeshOptions) -> Result<NetMesh> {
            let bind_addr: std::net::SocketAddr = options
                .bind_addr
                .parse()
                .map_err(|e| Error::from_reason(format!("invalid bind address: {}", e)))?;

            let psk_bytes = hex::decode(&options.psk)
                .map_err(|e| Error::from_reason(format!("invalid PSK hex: {}", e)))?;
            if psk_bytes.len() != 32 {
                return Err(Error::from_reason("PSK must be 32 bytes (64 hex chars)"));
            }
            let mut psk = [0u8; 32];
            psk.copy_from_slice(&psk_bytes);

            let mut config = MeshNodeConfig::new(bind_addr, psk);
            if let Some(ms) = options.heartbeat_interval_ms {
                config = config.with_heartbeat_interval(Duration::from_millis(ms as u64));
            }
            if let Some(ms) = options.session_timeout_ms {
                config = config.with_session_timeout(Duration::from_millis(ms as u64));
            }
            if let Some(n) = options.num_shards {
                let n = u16::try_from(n).map_err(|_| {
                    Error::from_reason(format!("num_shards must be in [0, 65535]; got {}", n))
                })?;
                config = config.with_num_shards(n);
            }

            let identity = EntityKeypair::generate();

            let node = MeshNode::new(identity, config)
                .await
                .map_err(|e| Error::from_reason(format!("MeshNode creation failed: {}", e)))?;

            Ok(NetMesh {
                node: Arc::new(ArcSwapOption::from_pointee(node)),
            })
        }

        /// Get this node's Noise public key (hex-encoded).
        #[napi]
        pub fn public_key(&self) -> Result<String> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            Ok(hex::encode(node.public_key()))
        }

        /// Get this node's ID.
        ///
        /// Returned as `i64` to fit JavaScript's `number` semantics. Fails
        /// rather than wraps when the `u64` node_id exceeds `i64::MAX`,
        /// which would otherwise silently flip sign on the JS side.
        #[napi]
        pub fn node_id(&self) -> Result<i64> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            i64::try_from(node.node_id()).map_err(|_| {
                Error::from_reason(format!(
                    "node_id {} exceeds i64::MAX; JS has no lossless u64",
                    node.node_id()
                ))
            })
        }

        /// Connect to a peer (initiator side).
        #[napi]
        pub async fn connect(
            &self,
            peer_addr: String,
            peer_public_key: String,
            peer_node_id: i64,
        ) -> Result<()> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let addr: std::net::SocketAddr = peer_addr
                .parse()
                .map_err(|e| Error::from_reason(format!("invalid peer address: {}", e)))?;

            let pubkey_bytes = hex::decode(&peer_public_key)
                .map_err(|e| Error::from_reason(format!("invalid public key hex: {}", e)))?;
            if pubkey_bytes.len() != 32 {
                return Err(Error::from_reason("public key must be 32 bytes"));
            }
            let mut pubkey = [0u8; 32];
            pubkey.copy_from_slice(&pubkey_bytes);

            let peer_node_id = u64::try_from(peer_node_id).map_err(|_| {
                Error::from_reason(format!(
                    "peer_node_id must be non-negative; got {}",
                    peer_node_id
                ))
            })?;
            node.connect(addr, &pubkey, peer_node_id)
                .await
                .map_err(|e| Error::from_reason(format!("connect failed: {}", e)))?;
            Ok(())
        }

        /// Accept an incoming connection (responder side).
        #[napi]
        pub async fn accept(&self, peer_node_id: i64) -> Result<String> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let peer_node_id = u64::try_from(peer_node_id).map_err(|_| {
                Error::from_reason(format!(
                    "peer_node_id must be non-negative; got {}",
                    peer_node_id
                ))
            })?;
            let (addr, _) = node
                .accept(peer_node_id)
                .await
                .map_err(|e| Error::from_reason(format!("accept failed: {}", e)))?;
            Ok(addr.to_string())
        }

        /// Start the receive loop, heartbeats, and router.
        #[napi]
        pub fn start(&self) -> Result<()> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            node.start();
            Ok(())
        }

        /// Send raw bytes to a direct peer.
        #[napi]
        pub async fn push_to(&self, peer_addr: String, data: Buffer) -> Result<bool> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let addr: std::net::SocketAddr = peer_addr
                .parse()
                .map_err(|e| Error::from_reason(format!("invalid address: {}", e)))?;

            let batch = net::event::Batch {
                shard_id: 0,
                events: vec![net::event::InternalEvent::new(
                    bytes::Bytes::copy_from_slice(data.as_ref()),
                    0,
                    0,
                )],
                sequence_start: 0,
            };

            node.send_to_peer(addr, batch)
                .await
                .map_err(|e| Error::from_reason(format!("send failed: {}", e)))?;
            Ok(true)
        }

        /// Poll for received events.
        #[napi]
        pub async fn poll(&self, limit: u32) -> Result<Vec<StoredEvent>> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let result = node
                .poll_shard(0, None, limit as usize)
                .await
                .map_err(|e| Error::from_reason(format!("poll failed: {}", e)))?;

            Ok(result
                .events
                .into_iter()
                .map(|e| {
                    // Preserve binary payloads in `raw_bytes`. `raw` is
                    // kept for back-compat with UTF-8 consumers but is
                    // deliberately empty (not a silent UTF-8-lossy
                    // substitution) when the payload isn't valid UTF-8 —
                    // callers that need fidelity must use `raw_bytes`.
                    let raw = e.raw_str().unwrap_or("").to_string();
                    let raw_bytes = Buffer::from(e.raw.to_vec());
                    StoredEvent {
                        id: e.id,
                        raw,
                        raw_bytes,
                        insertion_ts: e.insertion_ts as i64,
                        shard_id: e.shard_id as u32,
                    }
                })
                .collect())
        }

        /// Add a route to a destination node.
        #[napi]
        pub fn add_route(&self, dest_node_id: i64, next_hop_addr: String) -> Result<()> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let addr: std::net::SocketAddr = next_hop_addr
                .parse()
                .map_err(|e| Error::from_reason(format!("invalid address: {}", e)))?;
            let dest_node_id = u64::try_from(dest_node_id).map_err(|_| {
                Error::from_reason(format!(
                    "dest_node_id must be non-negative; got {}",
                    dest_node_id
                ))
            })?;
            node.router().add_route(dest_node_id, addr);
            Ok(())
        }

        /// Number of connected peers.
        #[napi]
        pub fn peer_count(&self) -> Result<u32> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            Ok(node.peer_count() as u32)
        }

        /// Number of nodes discovered via pingwave.
        #[napi]
        pub fn discovered_nodes(&self) -> Result<u32> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            Ok(node.proximity_graph().node_count() as u32)
        }

        // ─── Stream API ────────────────────────────────────────────

        /// Open (or look up) a stream to a connected peer. Repeated
        /// calls for the same `(peer, streamId)` return handles to the
        /// same underlying state (first-open wins; differing configs
        /// are logged and ignored).
        #[napi]
        pub fn open_stream(&self, peer_node_id: i64, opts: StreamOptions) -> Result<NetStream> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let peer_u64 = u64::try_from(peer_node_id).map_err(|_| {
                Error::from_reason(format!(
                    "peer_node_id must be non-negative; got {}",
                    peer_node_id
                ))
            })?;
            let stream_u64 = u64::try_from(opts.stream_id).map_err(|_| {
                Error::from_reason(format!(
                    "stream_id must be non-negative; got {}",
                    opts.stream_id
                ))
            })?;
            let config = stream_config_from_opts(&opts)?;
            let core = node
                .open_stream(peer_u64, stream_u64, config)
                .map_err(|e| Error::from_reason(format!("open_stream failed: {}", e)))?;
            Ok(NetStream {
                peer_node_id: peer_u64,
                stream_id: stream_u64,
                core,
            })
        }

        /// Close a stream. Idempotent.
        #[napi]
        pub fn close_stream(&self, peer_node_id: i64, stream_id: i64) -> Result<()> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let peer_u64 = u64::try_from(peer_node_id).map_err(|_| {
                Error::from_reason(format!(
                    "peer_node_id must be non-negative; got {}",
                    peer_node_id
                ))
            })?;
            let stream_u64 = u64::try_from(stream_id).map_err(|_| {
                Error::from_reason(format!("stream_id must be non-negative; got {}", stream_id))
            })?;
            node.close_stream(peer_u64, stream_u64);
            Ok(())
        }

        /// Send a batch of events on an explicit stream.
        ///
        /// **Error contract for SDK wrappers:** message prefixes are
        /// stable. `"stream would block"` = `BackpressureError`;
        /// `"stream not connected"` = `NotConnectedError`; anything
        /// else is a real transport failure. See `sdk-ts` for the
        /// class-based re-throw layer.
        #[napi]
        pub async fn send_on_stream(&self, stream: &NetStream, events: Vec<Buffer>) -> Result<()> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let payloads: Vec<bytes::Bytes> = events
                .into_iter()
                .map(|b| bytes::Bytes::copy_from_slice(b.as_ref()))
                .collect();
            node.send_on_stream(&stream.core, &payloads)
                .await
                .map_err(stream_error_to_napi)
        }

        /// Send events, retrying on `Backpressure` with 5 ms → 200 ms
        /// exponential backoff up to `maxRetries` times. Transport
        /// errors are returned immediately (not retried).
        #[napi]
        pub async fn send_with_retry(
            &self,
            stream: &NetStream,
            events: Vec<Buffer>,
            max_retries: u32,
        ) -> Result<()> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let payloads: Vec<bytes::Bytes> = events
                .into_iter()
                .map(|b| bytes::Bytes::copy_from_slice(b.as_ref()))
                .collect();
            node.send_with_retry(&stream.core, &payloads, max_retries as usize)
                .await
                .map_err(stream_error_to_napi)
        }

        /// Block the calling JS task until the send succeeds or a
        /// transport error occurs. Retries `Backpressure` with 5 ms →
        /// 200 ms exponential backoff up to 4096 times (~13 min worst
        /// case) — effectively "block until the network lets up" for
        /// practical workloads, but with a hard upper bound so runaway
        /// pressure can't hang a caller forever. Use `sendWithRetry`
        /// directly if you need a tighter bound.
        #[napi]
        pub async fn send_blocking(&self, stream: &NetStream, events: Vec<Buffer>) -> Result<()> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let payloads: Vec<bytes::Bytes> = events
                .into_iter()
                .map(|b| bytes::Bytes::copy_from_slice(b.as_ref()))
                .collect();
            node.send_blocking(&stream.core, &payloads)
                .await
                .map_err(stream_error_to_napi)
        }

        /// Snapshot of per-stream stats. Returns `null` if the peer or
        /// stream isn't registered.
        #[napi]
        pub fn stream_stats(
            &self,
            peer_node_id: i64,
            stream_id: i64,
        ) -> Result<Option<NetStreamStats>> {
            let guard = self.load_node()?;
            let node = guard.as_ref().unwrap();
            let peer_u64 = u64::try_from(peer_node_id).map_err(|_| {
                Error::from_reason(format!(
                    "peer_node_id must be non-negative; got {}",
                    peer_node_id
                ))
            })?;
            let stream_u64 = u64::try_from(stream_id).map_err(|_| {
                Error::from_reason(format!("stream_id must be non-negative; got {}", stream_id))
            })?;
            Ok(node
                .stream_stats(peer_u64, stream_u64)
                .map(|s| NetStreamStats {
                    tx_seq: BigInt::from(s.tx_seq),
                    rx_seq: BigInt::from(s.rx_seq),
                    inbound_pending: BigInt::from(s.inbound_pending),
                    last_activity_ns: BigInt::from(s.last_activity_ns),
                    active: s.active,
                    backpressure_events: BigInt::from(s.backpressure_events),
                    tx_credit_remaining: s.tx_credit_remaining,
                    tx_window: s.tx_window,
                    credit_grants_received: BigInt::from(s.credit_grants_received),
                    credit_grants_sent: BigInt::from(s.credit_grants_sent),
                }))
        }

        /// Shutdown the mesh node.
        #[napi]
        pub async fn shutdown(&self) -> Result<()> {
            let node_arc = self
                .node
                .swap(None)
                .ok_or_else(|| Error::from_reason("already shut down"))?;

            match Arc::try_unwrap(node_arc) {
                Ok(node) => {
                    node.shutdown()
                        .await
                        .map_err(|e| Error::from_reason(format!("shutdown failed: {}", e)))?;
                }
                Err(arc) => {
                    // Put it back if there are outstanding references
                    self.node.store(Some(arc));
                    return Err(Error::from_reason(
                        "cannot shutdown: outstanding references exist",
                    ));
                }
            }
            Ok(())
        }

        fn load_node(&self) -> Result<arc_swap::Guard<Option<Arc<MeshNode>>>> {
            let guard = self.node.load();
            if guard.is_none() {
                return Err(Error::from_reason("MeshNode has been shut down"));
            }
            Ok(guard)
        }
    }
}

#[cfg(feature = "net")]
pub use mesh_bindings::*;
