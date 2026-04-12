//! Python bindings for Blackstream event bus.
//!
//! Provides high-performance event ingestion and consumption for Python.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::runtime::Runtime;

use blackstream::{
    config::{AdapterConfig, BackpressureMode, EventBusConfig},
    consumer::Ordering,
    event::RawEvent,
    ConsumeRequest, EventBus, Filter,
};

#[cfg(feature = "redis")]
use blackstream::config::RedisAdapterConfig;

#[cfg(feature = "jetstream")]
use blackstream::config::JetStreamAdapterConfig;

#[cfg(feature = "bltp")]
use blackstream::adapter::bltp::{BltpAdapterConfig, ReliabilityConfig, StaticKeypair};

/// Result of an ingestion operation.
#[pyclass]
#[derive(Clone)]
pub struct IngestResult {
    #[pyo3(get)]
    pub shard_id: u16,
    #[pyo3(get)]
    pub timestamp: u64,
}

#[pymethods]
impl IngestResult {
    fn __repr__(&self) -> String {
        format!(
            "IngestResult(shard_id={}, timestamp={})",
            self.shard_id, self.timestamp
        )
    }
}

/// A stored event returned from polling.
#[pyclass]
#[derive(Clone)]
pub struct StoredEvent {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub raw: String,
    #[pyo3(get)]
    pub insertion_ts: u64,
    #[pyo3(get)]
    pub shard_id: u16,
}

#[pymethods]
impl StoredEvent {
    fn __repr__(&self) -> String {
        format!(
            "StoredEvent(id='{}', shard_id={}, insertion_ts={})",
            self.id, self.shard_id, self.insertion_ts
        )
    }

    /// Parse the raw JSON into a Python dict.
    fn parse(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let json_module = py.import("json")?;
        let result = json_module.call_method1("loads", (&self.raw,))?;
        Ok(result.into())
    }
}

/// Poll response containing events and cursor.
#[pyclass]
#[derive(Clone)]
pub struct PollResponse {
    #[pyo3(get)]
    pub events: Vec<StoredEvent>,
    #[pyo3(get)]
    pub next_id: Option<String>,
    #[pyo3(get)]
    pub has_more: bool,
}

#[pymethods]
impl PollResponse {
    fn __repr__(&self) -> String {
        format!(
            "PollResponse(events=[...{}], next_id={:?}, has_more={})",
            self.events.len(),
            self.next_id,
            self.has_more
        )
    }

    fn __len__(&self) -> usize {
        self.events.len()
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<EventIterator>> {
        let iter = EventIterator {
            events: slf.events.clone(),
            index: 0,
        };
        Py::new(slf.py(), iter)
    }
}

#[pyclass]
struct EventIterator {
    events: Vec<StoredEvent>,
    index: usize,
}

#[pymethods]
impl EventIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<StoredEvent> {
        if slf.index < slf.events.len() {
            let event = slf.events[slf.index].clone();
            slf.index += 1;
            Some(event)
        } else {
            None
        }
    }
}

/// Ingestion statistics.
#[pyclass]
#[derive(Clone)]
pub struct Stats {
    #[pyo3(get)]
    pub events_ingested: u64,
    #[pyo3(get)]
    pub events_dropped: u64,
}

#[pymethods]
impl Stats {
    fn __repr__(&self) -> String {
        format!(
            "Stats(events_ingested={}, events_dropped={})",
            self.events_ingested, self.events_dropped
        )
    }
}

/// BLTP keypair for encrypted UDP transport.
#[pyclass]
#[derive(Clone)]
pub struct BltpKeypair {
    /// Hex-encoded 32-byte public key
    #[pyo3(get)]
    pub public_key: String,
    /// Hex-encoded 32-byte secret key
    #[pyo3(get)]
    pub secret_key: String,
}

#[pymethods]
impl BltpKeypair {
    fn __repr__(&self) -> String {
        format!(
            "BltpKeypair(public_key='{}...', secret_key='[REDACTED]')",
            &self.public_key[..8]
        )
    }
}

/// Generate a new BLTP keypair for encrypted UDP transport.
///
/// Returns a BltpKeypair with hex-encoded public and secret keys.
/// Use this to generate keys for a responder, then share the public key
/// with the initiator.
///
/// Returns:
///     BltpKeypair with public_key and secret_key attributes
#[cfg(feature = "bltp")]
#[pyfunction]
fn generate_bltp_keypair() -> BltpKeypair {
    let keypair = StaticKeypair::generate();
    BltpKeypair {
        public_key: hex::encode(keypair.public_key()),
        secret_key: hex::encode(keypair.secret_key()),
    }
}

/// High-performance event bus for Python.
///
/// Example usage:
/// ```python
/// from blackstream import Blackstream
///
/// # Create event bus
/// bus = Blackstream(num_shards=4)
///
/// # Ingest events (fast path with raw JSON string)
/// bus.ingest_raw('{"token": "hello", "index": 0}')
///
/// # Or ingest a dict (convenience method)
/// bus.ingest({"token": "world", "index": 1})
///
/// # Poll events
/// response = bus.poll(limit=100)
/// for event in response:
///     print(event.raw)
///
/// bus.shutdown()
/// ```
#[pyclass]
pub struct Blackstream {
    bus: Arc<RwLock<Option<EventBus>>>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl Blackstream {
    /// Create a new Blackstream event bus.
    ///
    /// Args:
    ///     num_shards: Number of shards (defaults to CPU core count)
    ///     ring_buffer_capacity: Ring buffer capacity per shard (must be power of 2)
    ///     backpressure_mode: One of "drop_newest", "drop_oldest", "fail_producer"
    ///     redis_url: Redis connection URL (e.g., "redis://localhost:6379")
    ///     redis_prefix: Stream key prefix (default: "blackstream")
    ///     redis_pipeline_size: Maximum commands per pipeline (default: 1000)
    ///     redis_pool_size: Connection pool size (default: num_shards)
    ///     redis_connect_timeout_ms: Connection timeout in milliseconds (default: 5000)
    ///     redis_command_timeout_ms: Command timeout in milliseconds (default: 1000)
    ///     redis_max_stream_len: Maximum stream length, unlimited if not set
    ///     jetstream_url: NATS JetStream URL (e.g., "nats://localhost:4222")
    ///     jetstream_prefix: Stream name prefix (default: "blackstream")
    ///     jetstream_connect_timeout_ms: Connection timeout in milliseconds (default: 5000)
    ///     jetstream_request_timeout_ms: Request timeout in milliseconds (default: 5000)
    ///     jetstream_max_messages: Maximum messages per stream, unlimited if not set
    ///     jetstream_max_bytes: Maximum bytes per stream, unlimited if not set
    ///     jetstream_max_age_ms: Maximum age for messages in milliseconds, unlimited if not set
    ///     jetstream_replicas: Number of stream replicas (default: 1)
    ///     bltp_bind_addr: BLTP local bind address (e.g., "127.0.0.1:9000")
    ///     bltp_peer_addr: BLTP remote peer address (e.g., "127.0.0.1:9001")
    ///     bltp_psk: Hex-encoded 32-byte pre-shared key
    ///     bltp_role: Connection role - "initiator" or "responder"
    ///     bltp_peer_public_key: Hex-encoded peer's public key (required for initiator)
    ///     bltp_secret_key: Hex-encoded secret key (required for responder)
    ///     bltp_public_key: Hex-encoded public key (required for responder)
    ///     bltp_reliability: Reliability mode - "none", "light", or "full" (default: "none")
    ///     bltp_heartbeat_interval_ms: Heartbeat interval in milliseconds (default: 5000)
    ///     bltp_session_timeout_ms: Session timeout in milliseconds (default: 30000)
    ///     bltp_batched_io: Enable batched I/O for Linux (default: False)
    ///     bltp_packet_pool_size: Packet pool size (default: 64)
    #[new]
    #[pyo3(signature = (num_shards=None, ring_buffer_capacity=None, backpressure_mode=None, redis_url=None, redis_prefix=None, redis_pipeline_size=None, redis_pool_size=None, redis_connect_timeout_ms=None, redis_command_timeout_ms=None, redis_max_stream_len=None, jetstream_url=None, jetstream_prefix=None, jetstream_connect_timeout_ms=None, jetstream_request_timeout_ms=None, jetstream_max_messages=None, jetstream_max_bytes=None, jetstream_max_age_ms=None, jetstream_replicas=None, bltp_bind_addr=None, bltp_peer_addr=None, bltp_psk=None, bltp_role=None, bltp_peer_public_key=None, bltp_secret_key=None, bltp_public_key=None, bltp_reliability=None, bltp_heartbeat_interval_ms=None, bltp_session_timeout_ms=None, bltp_batched_io=None, bltp_packet_pool_size=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        num_shards: Option<u16>,
        ring_buffer_capacity: Option<usize>,
        backpressure_mode: Option<&str>,
        redis_url: Option<&str>,
        redis_prefix: Option<&str>,
        redis_pipeline_size: Option<usize>,
        redis_pool_size: Option<usize>,
        redis_connect_timeout_ms: Option<u64>,
        redis_command_timeout_ms: Option<u64>,
        redis_max_stream_len: Option<usize>,
        jetstream_url: Option<&str>,
        jetstream_prefix: Option<&str>,
        jetstream_connect_timeout_ms: Option<u64>,
        jetstream_request_timeout_ms: Option<u64>,
        jetstream_max_messages: Option<i64>,
        jetstream_max_bytes: Option<i64>,
        jetstream_max_age_ms: Option<u64>,
        jetstream_replicas: Option<usize>,
        bltp_bind_addr: Option<&str>,
        bltp_peer_addr: Option<&str>,
        bltp_psk: Option<&str>,
        bltp_role: Option<&str>,
        bltp_peer_public_key: Option<&str>,
        bltp_secret_key: Option<&str>,
        bltp_public_key: Option<&str>,
        bltp_reliability: Option<&str>,
        bltp_heartbeat_interval_ms: Option<u64>,
        bltp_session_timeout_ms: Option<u64>,
        bltp_batched_io: Option<bool>,
        bltp_packet_pool_size: Option<usize>,
    ) -> PyResult<Self> {
        let runtime = Runtime::new().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let mut builder = EventBusConfig::builder();

        if let Some(n) = num_shards {
            builder = builder.num_shards(n);
        }
        if let Some(cap) = ring_buffer_capacity {
            builder = builder.ring_buffer_capacity(cap);
        }
        if let Some(mode) = backpressure_mode {
            let bp_mode = match mode {
                "drop_newest" => BackpressureMode::DropNewest,
                "drop_oldest" => BackpressureMode::DropOldest,
                "fail_producer" => BackpressureMode::FailProducer,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid backpressure mode: {}",
                        mode
                    )));
                }
            };
            builder = builder.backpressure_mode(bp_mode);
        }

        // Configure Redis adapter if URL is provided
        if let Some(url) = redis_url {
            #[cfg(feature = "redis")]
            {
                use std::time::Duration;
                let mut redis_config = RedisAdapterConfig::new(url);
                if let Some(prefix) = redis_prefix {
                    redis_config = redis_config.with_prefix(prefix);
                }
                if let Some(pipeline_size) = redis_pipeline_size {
                    redis_config = redis_config.with_pipeline_size(pipeline_size);
                }
                if let Some(pool_size) = redis_pool_size {
                    redis_config = redis_config.with_pool_size(pool_size);
                }
                if let Some(connect_timeout_ms) = redis_connect_timeout_ms {
                    redis_config = redis_config
                        .with_connect_timeout(Duration::from_millis(connect_timeout_ms));
                }
                if let Some(command_timeout_ms) = redis_command_timeout_ms {
                    redis_config = redis_config
                        .with_command_timeout(Duration::from_millis(command_timeout_ms));
                }
                if let Some(max_stream_len) = redis_max_stream_len {
                    redis_config = redis_config.with_max_stream_len(max_stream_len);
                }
                builder = builder.adapter(AdapterConfig::Redis(redis_config));
            }
            #[cfg(not(feature = "redis"))]
            {
                let _ = (
                    url,
                    redis_prefix,
                    redis_pipeline_size,
                    redis_pool_size,
                    redis_connect_timeout_ms,
                    redis_command_timeout_ms,
                    redis_max_stream_len,
                );
                return Err(PyRuntimeError::new_err(
                    "Redis support not enabled. Rebuild with --features redis",
                ));
            }
        } else if let Some(url) = jetstream_url {
            #[cfg(feature = "jetstream")]
            {
                use std::time::Duration;
                let mut js_config = JetStreamAdapterConfig::new(url);
                if let Some(prefix) = jetstream_prefix {
                    js_config = js_config.with_prefix(prefix);
                }
                if let Some(connect_timeout_ms) = jetstream_connect_timeout_ms {
                    js_config =
                        js_config.with_connect_timeout(Duration::from_millis(connect_timeout_ms));
                }
                if let Some(request_timeout_ms) = jetstream_request_timeout_ms {
                    js_config =
                        js_config.with_request_timeout(Duration::from_millis(request_timeout_ms));
                }
                if let Some(max_messages) = jetstream_max_messages {
                    js_config = js_config.with_max_messages(max_messages);
                }
                if let Some(max_bytes) = jetstream_max_bytes {
                    js_config = js_config.with_max_bytes(max_bytes);
                }
                if let Some(max_age_ms) = jetstream_max_age_ms {
                    js_config = js_config.with_max_age(Duration::from_millis(max_age_ms));
                }
                if let Some(replicas) = jetstream_replicas {
                    js_config = js_config.with_replicas(replicas);
                }
                builder = builder.adapter(AdapterConfig::JetStream(js_config));
            }
            #[cfg(not(feature = "jetstream"))]
            {
                let _ = (
                    url,
                    jetstream_prefix,
                    jetstream_connect_timeout_ms,
                    jetstream_request_timeout_ms,
                    jetstream_max_messages,
                    jetstream_max_bytes,
                    jetstream_max_age_ms,
                    jetstream_replicas,
                );
                return Err(PyRuntimeError::new_err(
                    "JetStream support not enabled. Rebuild with --features jetstream",
                ));
            }
        } else if let Some(bind_addr_str) = bltp_bind_addr {
            #[cfg(feature = "bltp")]
            {
                use std::time::Duration;

                let bind_addr: std::net::SocketAddr = bind_addr_str
                    .parse()
                    .map_err(|e| PyValueError::new_err(format!("Invalid bltp_bind_addr: {}", e)))?;

                let peer_addr: std::net::SocketAddr = bltp_peer_addr
                    .ok_or_else(|| PyValueError::new_err("bltp_peer_addr is required"))?
                    .parse()
                    .map_err(|e| PyValueError::new_err(format!("Invalid bltp_peer_addr: {}", e)))?;

                let psk_hex =
                    bltp_psk.ok_or_else(|| PyValueError::new_err("bltp_psk is required"))?;
                let psk: [u8; 32] = hex::decode(psk_hex)
                    .map_err(|e| PyValueError::new_err(format!("Invalid bltp_psk hex: {}", e)))?
                    .try_into()
                    .map_err(|_| PyValueError::new_err("bltp_psk must be exactly 32 bytes"))?;

                let role =
                    bltp_role.ok_or_else(|| PyValueError::new_err("bltp_role is required"))?;

                let mut bltp_config = match role {
                    "initiator" => {
                        let peer_pubkey_hex = bltp_peer_public_key.ok_or_else(|| {
                            PyValueError::new_err("bltp_peer_public_key is required for initiator")
                        })?;
                        let peer_pubkey: [u8; 32] = hex::decode(peer_pubkey_hex)
                            .map_err(|e| {
                                PyValueError::new_err(format!(
                                    "Invalid bltp_peer_public_key hex: {}",
                                    e
                                ))
                            })?
                            .try_into()
                            .map_err(|_| {
                                PyValueError::new_err(
                                    "bltp_peer_public_key must be exactly 32 bytes",
                                )
                            })?;
                        BltpAdapterConfig::initiator(bind_addr, peer_addr, psk, peer_pubkey)
                    }
                    "responder" => {
                        let secret_key_hex = bltp_secret_key.ok_or_else(|| {
                            PyValueError::new_err("bltp_secret_key is required for responder")
                        })?;
                        let public_key_hex = bltp_public_key.ok_or_else(|| {
                            PyValueError::new_err("bltp_public_key is required for responder")
                        })?;
                        let secret_key: [u8; 32] = hex::decode(secret_key_hex)
                            .map_err(|e| {
                                PyValueError::new_err(format!("Invalid bltp_secret_key hex: {}", e))
                            })?
                            .try_into()
                            .map_err(|_| {
                                PyValueError::new_err("bltp_secret_key must be exactly 32 bytes")
                            })?;
                        let public_key: [u8; 32] = hex::decode(public_key_hex)
                            .map_err(|e| {
                                PyValueError::new_err(format!("Invalid bltp_public_key hex: {}", e))
                            })?
                            .try_into()
                            .map_err(|_| {
                                PyValueError::new_err("bltp_public_key must be exactly 32 bytes")
                            })?;
                        let keypair = StaticKeypair::from_keys(secret_key, public_key);
                        BltpAdapterConfig::responder(bind_addr, peer_addr, psk, keypair)
                    }
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid bltp_role: {}. Use 'initiator' or 'responder'",
                            role
                        )));
                    }
                };

                // Apply optional settings
                if let Some(reliability) = bltp_reliability {
                    bltp_config = bltp_config.with_reliability(match reliability {
                        "light" => ReliabilityConfig::Light,
                        "full" => ReliabilityConfig::Full,
                        _ => ReliabilityConfig::None,
                    });
                }
                if let Some(interval_ms) = bltp_heartbeat_interval_ms {
                    bltp_config =
                        bltp_config.with_heartbeat_interval(Duration::from_millis(interval_ms));
                }
                if let Some(timeout_ms) = bltp_session_timeout_ms {
                    bltp_config =
                        bltp_config.with_session_timeout(Duration::from_millis(timeout_ms));
                }
                if let Some(batched) = bltp_batched_io {
                    bltp_config = bltp_config.with_batched_io(batched);
                }
                if let Some(pool_size) = bltp_packet_pool_size {
                    bltp_config = bltp_config.with_pool_size(pool_size);
                }

                builder = builder.adapter(AdapterConfig::Bltp(Box::new(bltp_config)));
            }
            #[cfg(not(feature = "bltp"))]
            {
                let _ = (
                    bind_addr_str,
                    bltp_peer_addr,
                    bltp_psk,
                    bltp_role,
                    bltp_peer_public_key,
                    bltp_secret_key,
                    bltp_public_key,
                    bltp_reliability,
                    bltp_heartbeat_interval_ms,
                    bltp_session_timeout_ms,
                    bltp_batched_io,
                    bltp_packet_pool_size,
                );
                return Err(PyRuntimeError::new_err(
                    "BLTP support not enabled. Rebuild with --features bltp",
                ));
            }
        }

        let config = builder
            .build()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let bus = runtime
            .block_on(EventBus::new(config))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(Blackstream {
            bus: Arc::new(RwLock::new(Some(bus))),
            runtime: Arc::new(runtime),
        })
    }

    /// Ingest a raw JSON string (fastest path).
    ///
    /// This is the recommended method for high-throughput ingestion.
    /// The JSON string is stored directly without parsing.
    ///
    /// Args:
    ///     json: JSON string to ingest
    ///
    /// Returns:
    ///     IngestResult with shard_id and timestamp
    fn ingest_raw(&self, json: &str) -> PyResult<IngestResult> {
        let bus_guard = self
            .bus
            .read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        let bus = bus_guard
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("EventBus has been shut down"))?;

        let raw = RawEvent::from_str(json);
        let (shard_id, ts) = bus
            .ingest_raw(raw)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(IngestResult {
            shard_id,
            timestamp: ts,
        })
    }

    /// Ingest a Python dict (convenience method).
    ///
    /// The dict is serialized to JSON before ingestion.
    /// For maximum performance, use `ingest_raw` with pre-serialized JSON.
    ///
    /// Args:
    ///     event: Dict to ingest (will be JSON serialized)
    ///
    /// Returns:
    ///     IngestResult with shard_id and timestamp
    fn ingest(&self, py: Python<'_>, event: &Bound<'_, PyDict>) -> PyResult<IngestResult> {
        let json_module = py.import("json")?;
        let json_str: String = json_module.call_method1("dumps", (event,))?.extract()?;
        self.ingest_raw(&json_str)
    }

    /// Ingest multiple raw JSON strings in a batch.
    ///
    /// Args:
    ///     events: List of JSON strings to ingest
    ///
    /// Returns:
    ///     Number of successfully ingested events
    fn ingest_raw_batch(&self, events: Vec<String>) -> PyResult<usize> {
        let bus_guard = self
            .bus
            .read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        let bus = bus_guard
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("EventBus has been shut down"))?;

        let raw_events: Vec<RawEvent> = events.iter().map(|s| RawEvent::from_str(s)).collect();
        let count = bus.ingest_raw_batch(raw_events);

        Ok(count)
    }

    /// Poll events from the bus.
    ///
    /// Args:
    ///     limit: Maximum number of events to return
    ///     cursor: Optional cursor to resume from
    ///     filter: Optional JSON filter expression
    ///     ordering: Event ordering - "none" (default, fastest) or "insertion_ts" (cross-shard ordering)
    ///
    /// Returns:
    ///     PollResponse with events and pagination cursor
    #[pyo3(signature = (limit, cursor=None, filter=None, ordering=None))]
    fn poll(
        &self,
        limit: usize,
        cursor: Option<&str>,
        filter: Option<&str>,
        ordering: Option<&str>,
    ) -> PyResult<PollResponse> {
        let bus_guard = self
            .bus
            .read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        let bus = bus_guard
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("EventBus has been shut down"))?;

        let mut request = ConsumeRequest::new(limit);

        if let Some(c) = cursor {
            request = request.from(c);
        }

        if let Some(f) = filter {
            let filter_obj: Filter =
                serde_json::from_str(f).map_err(|e| PyValueError::new_err(e.to_string()))?;
            request = request.filter(filter_obj);
        }

        if let Some(ord) = ordering {
            let ordering_mode = match ord {
                "none" => Ordering::None,
                "insertion_ts" => Ordering::InsertionTs,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid ordering: {}. Use 'none' or 'insertion_ts'",
                        ord
                    )));
                }
            };
            request = request.ordering(ordering_mode);
        }

        let response = self
            .runtime
            .block_on(bus.poll(request))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let events: Vec<StoredEvent> = response
            .events
            .into_iter()
            .map(|e| StoredEvent {
                id: e.id,
                raw: e.raw_str().to_owned(),
                insertion_ts: e.insertion_ts,
                shard_id: e.shard_id,
            })
            .collect();

        Ok(PollResponse {
            events,
            next_id: response.next_id,
            has_more: response.has_more,
        })
    }

    /// Get the number of active shards.
    fn num_shards(&self) -> PyResult<u16> {
        let bus_guard = self
            .bus
            .read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        let bus = bus_guard
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("EventBus has been shut down"))?;

        Ok(bus.num_shards())
    }

    /// Get ingestion statistics.
    fn stats(&self) -> PyResult<Stats> {
        let bus_guard = self
            .bus
            .read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        let bus = bus_guard
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("EventBus has been shut down"))?;

        let stats = bus.stats();
        Ok(Stats {
            events_ingested: stats
                .events_ingested
                .load(std::sync::atomic::Ordering::Relaxed),
            events_dropped: stats
                .events_dropped
                .load(std::sync::atomic::Ordering::Relaxed),
        })
    }

    /// Gracefully shutdown the event bus.
    fn shutdown(&self) -> PyResult<()> {
        let mut bus_guard = self
            .bus
            .write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        if let Some(bus) = bus_guard.take() {
            self.runtime
                .block_on(bus.shutdown())
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    fn __repr__(&self) -> String {
        let bus_guard = self.bus.read().ok();
        if bus_guard.map(|g| g.is_some()).unwrap_or(false) {
            "Blackstream(active)".to_string()
        } else {
            "Blackstream(shutdown)".to_string()
        }
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[pyo3(signature = (_exc_type=None, _exc_val=None, _exc_tb=None))]
    fn __exit__(
        &self,
        _exc_type: Option<&Bound<'_, pyo3::types::PyType>>,
        _exc_val: Option<&Bound<'_, pyo3::types::PyAny>>,
        _exc_tb: Option<&Bound<'_, pyo3::types::PyAny>>,
    ) -> PyResult<bool> {
        self.shutdown()?;
        Ok(false)
    }
}

/// Blackstream Python module.
#[pymodule]
fn _blackstream(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Blackstream>()?;
    m.add_class::<IngestResult>()?;
    m.add_class::<StoredEvent>()?;
    m.add_class::<PollResponse>()?;
    m.add_class::<Stats>()?;
    m.add_class::<BltpKeypair>()?;
    #[cfg(feature = "bltp")]
    m.add_function(wrap_pyfunction!(generate_bltp_keypair, m)?)?;
    Ok(())
}
