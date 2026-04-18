//! Python bindings for Net event bus.
//!
//! Provides high-performance event ingestion and consumption for Python.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::runtime::Runtime;

use net::{
    config::{AdapterConfig, BackpressureMode, EventBusConfig},
    consumer::Ordering,
    event::RawEvent,
    ConsumeRequest, EventBus, Filter,
};

#[cfg(feature = "redis")]
use net::config::RedisAdapterConfig;

#[cfg(feature = "jetstream")]
use net::config::JetStreamAdapterConfig;

#[cfg(feature = "net")]
use net::adapter::net::{NetAdapterConfig, ReliabilityConfig, StaticKeypair};

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

/// Net keypair for encrypted UDP transport.
#[pyclass]
#[derive(Clone)]
pub struct NetKeypair {
    /// Hex-encoded 32-byte public key
    #[pyo3(get)]
    pub public_key: String,
    /// Hex-encoded 32-byte secret key
    #[pyo3(get)]
    pub secret_key: String,
}

#[pymethods]
impl NetKeypair {
    fn __repr__(&self) -> String {
        format!(
            "NetKeypair(public_key='{}...', secret_key='[REDACTED]')",
            &self.public_key[..self.public_key.len().min(8)]
        )
    }
}

/// Generate a new Net keypair for encrypted UDP transport.
///
/// Returns a NetKeypair with hex-encoded public and secret keys.
/// Use this to generate keys for a responder, then share the public key
/// with the initiator.
///
/// Returns:
///     NetKeypair with public_key and secret_key attributes
#[cfg(feature = "net")]
#[pyfunction]
fn generate_net_keypair() -> NetKeypair {
    let keypair = StaticKeypair::generate();
    NetKeypair {
        public_key: hex::encode(keypair.public_key()),
        secret_key: hex::encode(keypair.secret_key()),
    }
}

/// High-performance event bus for Python.
///
/// Example usage:
/// ```python
/// from net import Net
///
/// # Create event bus
/// bus = Net(num_shards=4)
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
pub struct Net {
    bus: Arc<RwLock<Option<EventBus>>>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl Net {
    /// Create a new Net event bus.
    ///
    /// Args:
    ///     num_shards: Number of shards (defaults to CPU core count)
    ///     ring_buffer_capacity: Ring buffer capacity per shard (must be power of 2)
    ///     backpressure_mode: One of "drop_newest", "drop_oldest", "fail_producer"
    ///     redis_url: Redis connection URL (e.g., "redis://localhost:6379")
    ///     redis_prefix: Stream key prefix (default: "net")
    ///     redis_pipeline_size: Maximum commands per pipeline (default: 1000)
    ///     redis_pool_size: Connection pool size (default: num_shards)
    ///     redis_connect_timeout_ms: Connection timeout in milliseconds (default: 5000)
    ///     redis_command_timeout_ms: Command timeout in milliseconds (default: 1000)
    ///     redis_max_stream_len: Maximum stream length, unlimited if not set
    ///     jetstream_url: NATS JetStream URL (e.g., "nats://localhost:4222")
    ///     jetstream_prefix: Stream name prefix (default: "net")
    ///     jetstream_connect_timeout_ms: Connection timeout in milliseconds (default: 5000)
    ///     jetstream_request_timeout_ms: Request timeout in milliseconds (default: 5000)
    ///     jetstream_max_messages: Maximum messages per stream, unlimited if not set
    ///     jetstream_max_bytes: Maximum bytes per stream, unlimited if not set
    ///     jetstream_max_age_ms: Maximum age for messages in milliseconds, unlimited if not set
    ///     jetstream_replicas: Number of stream replicas (default: 1)
    ///     net_bind_addr: Net local bind address (e.g., "127.0.0.1:9000")
    ///     net_peer_addr: Net remote peer address (e.g., "127.0.0.1:9001")
    ///     net_psk: Hex-encoded 32-byte pre-shared key
    ///     net_role: Connection role - "initiator" or "responder"
    ///     net_peer_public_key: Hex-encoded peer's public key (required for initiator)
    ///     net_secret_key: Hex-encoded secret key (required for responder)
    ///     net_public_key: Hex-encoded public key (required for responder)
    ///     net_reliability: Reliability mode - "none", "light", or "full" (default: "none")
    ///     net_heartbeat_interval_ms: Heartbeat interval in milliseconds (default: 5000)
    ///     net_session_timeout_ms: Session timeout in milliseconds (default: 30000)
    ///     net_batched_io: Enable batched I/O for Linux (default: False)
    ///     net_packet_pool_size: Packet pool size (default: 64)
    #[new]
    #[pyo3(signature = (num_shards=None, ring_buffer_capacity=None, backpressure_mode=None, redis_url=None, redis_prefix=None, redis_pipeline_size=None, redis_pool_size=None, redis_connect_timeout_ms=None, redis_command_timeout_ms=None, redis_max_stream_len=None, jetstream_url=None, jetstream_prefix=None, jetstream_connect_timeout_ms=None, jetstream_request_timeout_ms=None, jetstream_max_messages=None, jetstream_max_bytes=None, jetstream_max_age_ms=None, jetstream_replicas=None, net_bind_addr=None, net_peer_addr=None, net_psk=None, net_role=None, net_peer_public_key=None, net_secret_key=None, net_public_key=None, net_reliability=None, net_heartbeat_interval_ms=None, net_session_timeout_ms=None, net_batched_io=None, net_packet_pool_size=None))]
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
        net_bind_addr: Option<&str>,
        net_peer_addr: Option<&str>,
        net_psk: Option<&str>,
        net_role: Option<&str>,
        net_peer_public_key: Option<&str>,
        net_secret_key: Option<&str>,
        net_public_key: Option<&str>,
        net_reliability: Option<&str>,
        net_heartbeat_interval_ms: Option<u64>,
        net_session_timeout_ms: Option<u64>,
        net_batched_io: Option<bool>,
        net_packet_pool_size: Option<usize>,
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
        } else if let Some(bind_addr_str) = net_bind_addr {
            #[cfg(feature = "net")]
            {
                use std::time::Duration;

                let bind_addr: std::net::SocketAddr = bind_addr_str
                    .parse()
                    .map_err(|e| PyValueError::new_err(format!("Invalid net_bind_addr: {}", e)))?;

                let peer_addr: std::net::SocketAddr = net_peer_addr
                    .ok_or_else(|| PyValueError::new_err("net_peer_addr is required"))?
                    .parse()
                    .map_err(|e| PyValueError::new_err(format!("Invalid net_peer_addr: {}", e)))?;

                let psk_hex =
                    net_psk.ok_or_else(|| PyValueError::new_err("net_psk is required"))?;
                let psk: [u8; 32] = hex::decode(psk_hex)
                    .map_err(|e| PyValueError::new_err(format!("Invalid net_psk hex: {}", e)))?
                    .try_into()
                    .map_err(|_| PyValueError::new_err("net_psk must be exactly 32 bytes"))?;

                let role = net_role.ok_or_else(|| PyValueError::new_err("net_role is required"))?;

                let mut net_config = match role {
                    "initiator" => {
                        let peer_pubkey_hex = net_peer_public_key.ok_or_else(|| {
                            PyValueError::new_err("net_peer_public_key is required for initiator")
                        })?;
                        let peer_pubkey: [u8; 32] = hex::decode(peer_pubkey_hex)
                            .map_err(|e| {
                                PyValueError::new_err(format!(
                                    "Invalid net_peer_public_key hex: {}",
                                    e
                                ))
                            })?
                            .try_into()
                            .map_err(|_| {
                                PyValueError::new_err(
                                    "net_peer_public_key must be exactly 32 bytes",
                                )
                            })?;
                        NetAdapterConfig::initiator(bind_addr, peer_addr, psk, peer_pubkey)
                    }
                    "responder" => {
                        let secret_key_hex = net_secret_key.ok_or_else(|| {
                            PyValueError::new_err("net_secret_key is required for responder")
                        })?;
                        let public_key_hex = net_public_key.ok_or_else(|| {
                            PyValueError::new_err("net_public_key is required for responder")
                        })?;
                        let secret_key: [u8; 32] = hex::decode(secret_key_hex)
                            .map_err(|e| {
                                PyValueError::new_err(format!("Invalid net_secret_key hex: {}", e))
                            })?
                            .try_into()
                            .map_err(|_| {
                                PyValueError::new_err("net_secret_key must be exactly 32 bytes")
                            })?;
                        let public_key: [u8; 32] = hex::decode(public_key_hex)
                            .map_err(|e| {
                                PyValueError::new_err(format!("Invalid net_public_key hex: {}", e))
                            })?
                            .try_into()
                            .map_err(|_| {
                                PyValueError::new_err("net_public_key must be exactly 32 bytes")
                            })?;
                        let keypair = StaticKeypair::from_keys(secret_key, public_key);
                        NetAdapterConfig::responder(bind_addr, peer_addr, psk, keypair)
                    }
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid net_role: {}. Use 'initiator' or 'responder'",
                            role
                        )));
                    }
                };

                // Apply optional settings
                if let Some(reliability) = net_reliability {
                    net_config = net_config.with_reliability(match reliability {
                        "light" => ReliabilityConfig::Light,
                        "full" => ReliabilityConfig::Full,
                        _ => ReliabilityConfig::None,
                    });
                }
                if let Some(interval_ms) = net_heartbeat_interval_ms {
                    net_config =
                        net_config.with_heartbeat_interval(Duration::from_millis(interval_ms));
                }
                if let Some(timeout_ms) = net_session_timeout_ms {
                    net_config = net_config.with_session_timeout(Duration::from_millis(timeout_ms));
                }
                if let Some(batched) = net_batched_io {
                    net_config = net_config.with_batched_io(batched);
                }
                if let Some(pool_size) = net_packet_pool_size {
                    net_config = net_config.with_pool_size(pool_size);
                }

                builder = builder.adapter(AdapterConfig::Net(Box::new(net_config)));
            }
            #[cfg(not(feature = "net"))]
            {
                let _ = (
                    bind_addr_str,
                    net_peer_addr,
                    net_psk,
                    net_role,
                    net_peer_public_key,
                    net_secret_key,
                    net_public_key,
                    net_reliability,
                    net_heartbeat_interval_ms,
                    net_session_timeout_ms,
                    net_batched_io,
                    net_packet_pool_size,
                );
                return Err(PyRuntimeError::new_err(
                    "Net support not enabled. Rebuild with --features net",
                ));
            }
        }

        let config = builder
            .build()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let bus = runtime
            .block_on(EventBus::new(config))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(Net {
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
            .map(|e| {
                let raw = e.raw_str().unwrap_or("").to_string();
                StoredEvent {
                    id: e.id,
                    raw,
                    insertion_ts: e.insertion_ts,
                    shard_id: e.shard_id,
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
            "Net(active)".to_string()
        } else {
            "Net(shutdown)".to_string()
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

// ============================================================================
// MeshNode bindings
// ============================================================================

#[cfg(feature = "net")]
pyo3::create_exception!(
    _net,
    BackpressureError,
    pyo3::exceptions::PyException,
    "Raised when a stream's per-stream in-flight window is full. The \
     caller's events were NOT sent — decide whether to drop, retry, \
     or buffer at the app layer. See send_with_retry / send_blocking \
     for two built-in policies."
);

#[cfg(feature = "net")]
pyo3::create_exception!(
    _net,
    NotConnectedError,
    pyo3::exceptions::PyException,
    "Raised when a stream's peer session is gone (never connected, \
     disconnected, or the stream was closed)."
);

#[cfg(feature = "net")]
mod mesh_bindings {
    use super::*;
    use net::adapter::net::{
        EntityKeypair, MeshNode, MeshNodeConfig, Reliability, Stream as CoreStream, StreamConfig,
        StreamError,
    };
    use net::adapter::Adapter;
    use std::time::Duration;

    pub(crate) fn stream_error_to_py(e: StreamError) -> PyErr {
        match e {
            StreamError::Backpressure => {
                super::BackpressureError::new_err("stream would block (queue full)")
            }
            StreamError::NotConnected => super::NotConnectedError::new_err("stream not connected"),
            StreamError::Transport(msg) => {
                PyRuntimeError::new_err(format!("stream transport error: {}", msg))
            }
        }
    }

    /// Handle to an open stream. Opaque to Python callers.
    #[pyclass]
    pub struct NetStream {
        pub(crate) peer_node_id: u64,
        pub(crate) stream_id: u64,
        pub(crate) core: CoreStream,
    }

    #[pymethods]
    impl NetStream {
        #[getter]
        fn peer_node_id(&self) -> u64 {
            self.peer_node_id
        }
        #[getter]
        fn stream_id(&self) -> u64 {
            self.stream_id
        }
        fn __repr__(&self) -> String {
            format!(
                "NetStream(peer_node_id={:#x}, stream_id={:#x})",
                self.peer_node_id, self.stream_id
            )
        }
    }

    /// Snapshot of per-stream stats.
    #[pyclass]
    #[derive(Clone)]
    pub struct NetStreamStats {
        #[pyo3(get)]
        pub tx_seq: u64,
        #[pyo3(get)]
        pub rx_seq: u64,
        #[pyo3(get)]
        pub inbound_pending: u64,
        #[pyo3(get)]
        pub last_activity_ns: u64,
        #[pyo3(get)]
        pub active: bool,
        #[pyo3(get)]
        pub backpressure_events: u64,
        #[pyo3(get)]
        pub tx_inflight: u32,
        #[pyo3(get)]
        pub tx_window: u32,
    }

    pub(crate) fn parse_reliability(s: Option<&str>) -> PyResult<Reliability> {
        match s {
            None | Some("fire_and_forget") => Ok(Reliability::FireAndForget),
            Some("reliable") => Ok(Reliability::Reliable),
            Some(other) => Err(PyValueError::new_err(format!(
                "unknown reliability mode {:?}; expected \"fire_and_forget\" or \"reliable\"",
                other
            ))),
        }
    }

    /// A multi-peer mesh node for Python.
    ///
    /// Manages encrypted connections to multiple peers over a single
    /// UDP socket with automatic failure detection and rerouting.
    ///
    /// ```python
    /// from net import NetMesh
    ///
    /// node = NetMesh("127.0.0.1:9000", "00" * 32)
    /// print(f"public key: {node.public_key}")
    ///
    /// node.connect("127.0.0.1:9001", peer_pubkey, 0x2222)
    /// node.start()
    ///
    /// node.push_to("127.0.0.1:9001", '{"token":"hi"}')
    ///
    /// events = node.poll(100)
    /// node.shutdown()
    /// ```
    #[pyclass]
    pub struct NetMesh {
        node: Option<MeshNode>,
        runtime: Arc<Runtime>,
    }

    #[pymethods]
    impl NetMesh {
        /// Create a new mesh node.
        ///
        /// Args:
        ///     bind_addr: Local bind address (e.g., "127.0.0.1:9000")
        ///     psk: Hex-encoded 32-byte pre-shared key
        ///     heartbeat_interval_ms: Heartbeat interval (default: 5000)
        ///     session_timeout_ms: Session timeout (default: 30000)
        ///     num_shards: Number of inbound shards (default: 4)
        #[new]
        #[pyo3(signature = (bind_addr, psk, heartbeat_interval_ms=None, session_timeout_ms=None, num_shards=None))]
        fn new(
            bind_addr: &str,
            psk: &str,
            heartbeat_interval_ms: Option<u64>,
            session_timeout_ms: Option<u64>,
            num_shards: Option<u16>,
        ) -> PyResult<Self> {
            let addr: std::net::SocketAddr = bind_addr
                .parse()
                .map_err(|e| PyValueError::new_err(format!("invalid address: {}", e)))?;

            let psk_bytes = hex::decode(psk)
                .map_err(|e| PyValueError::new_err(format!("invalid PSK hex: {}", e)))?;
            if psk_bytes.len() != 32 {
                return Err(PyValueError::new_err("PSK must be 32 bytes (64 hex chars)"));
            }
            let mut psk_arr = [0u8; 32];
            psk_arr.copy_from_slice(&psk_bytes);

            let mut config = MeshNodeConfig::new(addr, psk_arr);
            if let Some(ms) = heartbeat_interval_ms {
                config = config.with_heartbeat_interval(Duration::from_millis(ms));
            }
            if let Some(ms) = session_timeout_ms {
                config = config.with_session_timeout(Duration::from_millis(ms));
            }
            if let Some(n) = num_shards {
                config = config.with_num_shards(n);
            }

            let runtime = Arc::new(
                Runtime::new().map_err(|e| PyRuntimeError::new_err(format!("runtime: {}", e)))?,
            );

            let identity = EntityKeypair::generate();
            let node = runtime
                .block_on(MeshNode::new(identity, config))
                .map_err(|e| PyRuntimeError::new_err(format!("MeshNode: {}", e)))?;

            Ok(Self {
                node: Some(node),
                runtime,
            })
        }

        /// Get this node's Noise public key (hex-encoded).
        #[getter]
        fn public_key(&self) -> PyResult<String> {
            let node = self.get_node()?;
            Ok(hex::encode(node.public_key()))
        }

        /// Get this node's ID.
        #[getter]
        fn node_id(&self) -> PyResult<u64> {
            let node = self.get_node()?;
            Ok(node.node_id())
        }

        /// Connect to a peer (initiator side).
        #[pyo3(signature = (peer_addr, peer_public_key, peer_node_id))]
        fn connect(
            &self,
            peer_addr: &str,
            peer_public_key: &str,
            peer_node_id: u64,
        ) -> PyResult<()> {
            let node = self.get_node()?;
            let addr: std::net::SocketAddr = peer_addr
                .parse()
                .map_err(|e| PyValueError::new_err(format!("invalid address: {}", e)))?;

            let pubkey_bytes = hex::decode(peer_public_key)
                .map_err(|e| PyValueError::new_err(format!("invalid hex: {}", e)))?;
            if pubkey_bytes.len() != 32 {
                return Err(PyValueError::new_err("public key must be 32 bytes"));
            }
            let mut pubkey = [0u8; 32];
            pubkey.copy_from_slice(&pubkey_bytes);

            self.runtime
                .block_on(node.connect(addr, &pubkey, peer_node_id))
                .map_err(|e| PyRuntimeError::new_err(format!("connect: {}", e)))?;
            Ok(())
        }

        /// Accept an incoming connection (responder side).
        fn accept(&self, peer_node_id: u64) -> PyResult<String> {
            let node = self.get_node()?;
            let (addr, _) = self
                .runtime
                .block_on(node.accept(peer_node_id))
                .map_err(|e| PyRuntimeError::new_err(format!("accept: {}", e)))?;
            Ok(addr.to_string())
        }

        /// Start the receive loop and heartbeats.
        fn start(&self) -> PyResult<()> {
            let node = self.get_node()?;
            node.start();
            Ok(())
        }

        /// Send raw JSON to a direct peer.
        fn push_to(&self, peer_addr: &str, json: &str) -> PyResult<bool> {
            let node = self.get_node()?;
            let addr: std::net::SocketAddr = peer_addr
                .parse()
                .map_err(|e| PyValueError::new_err(format!("invalid address: {}", e)))?;

            let batch = net::event::Batch {
                shard_id: 0,
                events: vec![net::event::InternalEvent::new(
                    bytes::Bytes::copy_from_slice(json.as_bytes()),
                    0,
                    0,
                )],
                sequence_start: 0,
            };

            self.runtime
                .block_on(node.send_to_peer(addr, batch))
                .map_err(|e| PyRuntimeError::new_err(format!("send: {}", e)))?;
            Ok(true)
        }

        /// Poll for received events.
        fn poll(&self, limit: usize) -> PyResult<Vec<StoredEvent>> {
            let node = self.get_node()?;
            let result = self
                .runtime
                .block_on(node.poll_shard(0, None, limit))
                .map_err(|e| PyRuntimeError::new_err(format!("poll: {}", e)))?;

            Ok(result
                .events
                .into_iter()
                .map(|e| {
                    let raw = e.raw_str().unwrap_or("").to_string();
                    StoredEvent {
                        id: e.id,
                        raw,
                        insertion_ts: e.insertion_ts,
                        shard_id: e.shard_id,
                    }
                })
                .collect())
        }

        /// Add a route.
        fn add_route(&self, dest_node_id: u64, next_hop_addr: &str) -> PyResult<()> {
            let node = self.get_node()?;
            let addr: std::net::SocketAddr = next_hop_addr
                .parse()
                .map_err(|e| PyValueError::new_err(format!("invalid address: {}", e)))?;
            node.router().add_route(dest_node_id, addr);
            Ok(())
        }

        /// Number of connected peers.
        fn peer_count(&self) -> PyResult<usize> {
            Ok(self.get_node()?.peer_count())
        }

        /// Number of nodes discovered via pingwave.
        fn discovered_nodes(&self) -> PyResult<usize> {
            Ok(self.get_node()?.proximity_graph().node_count())
        }

        // ─── Stream API ────────────────────────────────────────────

        /// Open (or look up) a logical stream to a connected peer.
        ///
        /// Repeated calls for the same (peer, stream_id) are
        /// idempotent — the first open wins; later differing configs
        /// are logged and ignored.
        ///
        /// Args:
        ///     peer_node_id: node_id of a peer this node is connected to.
        ///     stream_id: caller-chosen opaque u64.
        ///     reliability: "fire_and_forget" (default) or "reliable".
        ///     window_bytes: per-stream in-flight window cap. 0 = unbounded.
        ///     fairness_weight: fair-scheduler quantum multiplier.
        #[pyo3(signature = (
            peer_node_id,
            stream_id,
            reliability=None,
            window_bytes=0,
            fairness_weight=1
        ))]
        fn open_stream(
            &self,
            peer_node_id: u64,
            stream_id: u64,
            reliability: Option<&str>,
            window_bytes: u32,
            fairness_weight: u8,
        ) -> PyResult<NetStream> {
            let node = self.get_node()?;
            let rel = parse_reliability(reliability)?;
            let config = StreamConfig::new()
                .with_reliability(rel)
                .with_window_bytes(window_bytes)
                .with_fairness_weight(fairness_weight);
            let core = node
                .open_stream(peer_node_id, stream_id, config)
                .map_err(|e| PyRuntimeError::new_err(format!("open_stream: {}", e)))?;
            Ok(NetStream {
                peer_node_id,
                stream_id,
                core,
            })
        }

        /// Close a stream. Idempotent.
        fn close_stream(&self, peer_node_id: u64, stream_id: u64) -> PyResult<()> {
            let node = self.get_node()?;
            node.close_stream(peer_node_id, stream_id);
            Ok(())
        }

        /// Send a batch of events on an explicit stream. Each event is
        /// a `bytes` payload.
        ///
        /// Raises:
        ///     BackpressureError: stream's in-flight window is full (no
        ///         events sent — the caller decides what to do).
        ///     NotConnectedError: stream's peer session is gone.
        ///     RuntimeError: underlying transport failure.
        fn send_on_stream(&self, stream: &NetStream, events: Vec<Vec<u8>>) -> PyResult<()> {
            let node = self.get_node()?;
            let payloads: Vec<bytes::Bytes> = events.into_iter().map(bytes::Bytes::from).collect();
            self.runtime
                .block_on(node.send_on_stream(&stream.core, &payloads))
                .map_err(stream_error_to_py)
        }

        /// Send events, retrying on `BackpressureError` with 5 ms → 200 ms
        /// exponential backoff up to `max_retries` times. Transport
        /// errors and `NotConnectedError` are raised immediately.
        #[pyo3(signature = (stream, events, max_retries=8))]
        fn send_with_retry(
            &self,
            stream: &NetStream,
            events: Vec<Vec<u8>>,
            max_retries: u32,
        ) -> PyResult<()> {
            let node = self.get_node()?;
            let payloads: Vec<bytes::Bytes> = events.into_iter().map(bytes::Bytes::from).collect();
            self.runtime
                .block_on(node.send_with_retry(&stream.core, &payloads, max_retries as usize))
                .map_err(stream_error_to_py)
        }

        /// Block the calling task until the send succeeds or a
        /// transport error occurs. Unbounded retry on `BackpressureError`.
        fn send_blocking(&self, stream: &NetStream, events: Vec<Vec<u8>>) -> PyResult<()> {
            let node = self.get_node()?;
            let payloads: Vec<bytes::Bytes> = events.into_iter().map(bytes::Bytes::from).collect();
            self.runtime
                .block_on(node.send_blocking(&stream.core, &payloads))
                .map_err(stream_error_to_py)
        }

        /// Snapshot of per-stream stats. Returns None if the peer or
        /// stream isn't registered.
        fn stream_stats(
            &self,
            peer_node_id: u64,
            stream_id: u64,
        ) -> PyResult<Option<NetStreamStats>> {
            let node = self.get_node()?;
            Ok(node
                .stream_stats(peer_node_id, stream_id)
                .map(|s| NetStreamStats {
                    tx_seq: s.tx_seq,
                    rx_seq: s.rx_seq,
                    inbound_pending: s.inbound_pending,
                    last_activity_ns: s.last_activity_ns,
                    active: s.active,
                    backpressure_events: s.backpressure_events,
                    tx_inflight: s.tx_inflight,
                    tx_window: s.tx_window,
                }))
        }

        /// Shutdown the mesh node.
        fn shutdown(&mut self) -> PyResult<()> {
            let node = self
                .node
                .take()
                .ok_or_else(|| PyRuntimeError::new_err("already shut down"))?;
            self.runtime
                .block_on(node.shutdown())
                .map_err(|e| PyRuntimeError::new_err(format!("shutdown: {}", e)))?;
            Ok(())
        }

        fn __repr__(&self) -> String {
            if let Some(node) = &self.node {
                format!(
                    "NetMesh(addr={}, peers={}, nodes={})",
                    node.local_addr(),
                    node.peer_count(),
                    node.proximity_graph().node_count()
                )
            } else {
                "NetMesh(shutdown)".to_string()
            }
        }

        fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
            slf
        }

        fn __exit__(
            &mut self,
            _exc_type: Option<&Bound<'_, PyAny>>,
            _exc_val: Option<&Bound<'_, PyAny>>,
            _exc_tb: Option<&Bound<'_, PyAny>>,
        ) -> PyResult<bool> {
            self.shutdown()?;
            Ok(false)
        }
    }

    impl NetMesh {
        fn get_node(&self) -> PyResult<&MeshNode> {
            self.node
                .as_ref()
                .ok_or_else(|| PyRuntimeError::new_err("MeshNode has been shut down"))
        }
    }
}

/// Net Python module.
#[pymodule]
fn _net(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Net>()?;
    m.add_class::<IngestResult>()?;
    m.add_class::<StoredEvent>()?;
    m.add_class::<PollResponse>()?;
    m.add_class::<Stats>()?;
    m.add_class::<NetKeypair>()?;
    #[cfg(feature = "net")]
    m.add_function(wrap_pyfunction!(generate_net_keypair, m)?)?;
    #[cfg(feature = "net")]
    m.add_class::<mesh_bindings::NetMesh>()?;
    #[cfg(feature = "net")]
    m.add_class::<mesh_bindings::NetStream>()?;
    #[cfg(feature = "net")]
    m.add_class::<mesh_bindings::NetStreamStats>()?;
    #[cfg(feature = "net")]
    m.add("BackpressureError", m.py().get_type::<BackpressureError>())?;
    #[cfg(feature = "net")]
    m.add("NotConnectedError", m.py().get_type::<NotConnectedError>())?;
    Ok(())
}
