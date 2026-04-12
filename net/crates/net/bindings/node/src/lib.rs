//! Node.js bindings for Blackstream event bus.
//!
//! Provides high-performance event ingestion and consumption for Node.js/TypeScript.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use arc_swap::ArcSwapOption;

use blackstream::{
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
use blackstream::config::RedisAdapterConfig;

#[cfg(feature = "jetstream")]
use blackstream::config::JetStreamAdapterConfig;

#[cfg(feature = "bltp")]
use blackstream::adapter::bltp::{BltpAdapterConfig, ReliabilityConfig, StaticKeypair};

/// Redis adapter configuration.
#[napi(object)]
pub struct RedisOptions {
    /// Redis connection URL (e.g., "redis://localhost:6379")
    pub url: String,
    /// Stream key prefix (default: "blackstream")
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
    /// Stream name prefix (default: "blackstream")
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

/// BLTP keypair for encrypted UDP transport.
#[napi(object)]
pub struct BltpKeypair {
    /// Hex-encoded 32-byte public key
    pub public_key: String,
    /// Hex-encoded 32-byte secret key
    pub secret_key: String,
}

/// BLTP adapter configuration for encrypted UDP transport.
#[napi(object)]
pub struct BltpOptions {
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
    /// BLTP adapter configuration for encrypted UDP transport
    pub bltp: Option<BltpOptions>,
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
    /// Raw JSON payload as string
    pub raw: String,
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
/// import { Blackstream } from '@ai2070/blackstream';
///
/// const bus = await Blackstream.create({ numShards: 4 });
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
pub struct Blackstream {
    /// Lock-free bus handle using ArcSwap for maximum performance.
    /// ArcSwapOption allows atomic load/store without mutex overhead.
    bus: Arc<ArcSwapOption<EventBus>>,
}

#[napi]
impl Blackstream {
    /// Create a new Blackstream event bus.
    #[napi(factory)]
    pub async fn create(options: Option<EventBusOptions>) -> Result<Blackstream> {
        let config = build_config(options)?;

        let bus = EventBus::new(config)
            .await
            .map_err(|e| Error::from_reason(format!("Failed to create EventBus: {}", e)))?;

        Ok(Blackstream {
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

        let (_, value, _) = hash.get_u64();
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
            .map(|e| StoredEvent {
                id: e.id,
                raw: e.raw_str().to_owned(),
                insertion_ts: e.insertion_ts as i64,
                shard_id: e.shard_id as u32,
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
                .load(std::sync::atomic::Ordering::Relaxed) as i64,
            events_dropped: stats
                .events_dropped
                .load(std::sync::atomic::Ordering::Relaxed) as i64,
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
                Err(_arc) => {
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

/// Generate a new BLTP keypair for encrypted UDP transport.
///
/// Returns a keypair with hex-encoded public and secret keys.
/// Use this to generate keys for a responder, then share the public key
/// with the initiator.
#[cfg(feature = "bltp")]
#[napi]
pub fn generate_bltp_keypair() -> BltpKeypair {
    let keypair = StaticKeypair::generate();
    BltpKeypair {
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
        } else if let Some(bltp) = opts.bltp {
            #[cfg(feature = "bltp")]
            {
                use std::time::Duration;

                let bind_addr: std::net::SocketAddr = bltp
                    .bind_addr
                    .parse()
                    .map_err(|e| Error::from_reason(format!("Invalid bind_addr: {}", e)))?;

                let peer_addr: std::net::SocketAddr = bltp
                    .peer_addr
                    .parse()
                    .map_err(|e| Error::from_reason(format!("Invalid peer_addr: {}", e)))?;

                let psk: [u8; 32] = hex::decode(&bltp.psk)
                    .map_err(|e| Error::from_reason(format!("Invalid psk hex: {}", e)))?
                    .try_into()
                    .map_err(|_| Error::from_reason("psk must be exactly 32 bytes".to_string()))?;

                let mut bltp_config = match bltp.role.as_str() {
                    "initiator" => {
                        let peer_pubkey_hex = bltp.peer_public_key.ok_or_else(|| {
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
                        BltpAdapterConfig::initiator(bind_addr, peer_addr, psk, peer_pubkey)
                    }
                    "responder" => {
                        let secret_key_hex = bltp.secret_key.ok_or_else(|| {
                            Error::from_reason("secret_key is required for responder".to_string())
                        })?;
                        let public_key_hex = bltp.public_key.ok_or_else(|| {
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
                        BltpAdapterConfig::responder(bind_addr, peer_addr, psk, keypair)
                    }
                    _ => {
                        return Err(Error::from_reason(format!(
                            "Invalid role: {}. Use 'initiator' or 'responder'",
                            bltp.role
                        )));
                    }
                };

                // Apply optional settings
                if let Some(reliability) = bltp.reliability {
                    bltp_config = bltp_config.with_reliability(match reliability.as_str() {
                        "light" => ReliabilityConfig::Light,
                        "full" => ReliabilityConfig::Full,
                        _ => ReliabilityConfig::None,
                    });
                }
                if let Some(interval_ms) = bltp.heartbeat_interval_ms {
                    bltp_config = bltp_config
                        .with_heartbeat_interval(Duration::from_millis(interval_ms as u64));
                }
                if let Some(timeout_ms) = bltp.session_timeout_ms {
                    bltp_config =
                        bltp_config.with_session_timeout(Duration::from_millis(timeout_ms as u64));
                }
                if let Some(batched) = bltp.batched_io {
                    bltp_config = bltp_config.with_batched_io(batched);
                }
                if let Some(pool_size) = bltp.packet_pool_size {
                    bltp_config = bltp_config.with_pool_size(pool_size as usize);
                }

                builder = builder.adapter(AdapterConfig::Bltp(Box::new(bltp_config)));
            }
            #[cfg(not(feature = "bltp"))]
            {
                let _ = bltp;
                return Err(Error::from_reason(
                    "BLTP support not enabled. Rebuild with --features bltp".to_string(),
                ));
            }
        }
    }

    builder
        .build()
        .map_err(|e| Error::from_reason(format!("Invalid configuration: {}", e)))
}
