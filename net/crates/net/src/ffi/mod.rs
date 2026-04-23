//! C FFI bindings for cross-language integration.
//!
//! This module provides a C-compatible API for using Net from
//! other languages (Python, Node.js, Go, etc.).
//!
//! # Safety
//!
//! All public FFI functions in this module accept raw pointers from C code.
//! While they are not marked `unsafe` (to maintain C ABI compatibility),
//! callers must ensure:
//! - Pointers are valid and properly aligned
//! - String pointers point to valid UTF-8 data
//! - Buffer sizes are accurate
//! - Handles are not used after `net_shutdown`
//!
//! # Thread Safety
//!
//! All FFI functions are thread-safe. The event bus handle can be shared
//! across threads.
//!
//! # Memory Management
//!
//! - Handles returned by `net_init` must be freed with `net_shutdown`
//! - String buffers passed to `net_poll` are owned by the caller
//! - Error codes are returned as integers (0 = success, negative = error)
//!
//! # Example (C)
//!
//! ```c
//! #include "net.h"
//!
//! int main() {
//!     // Initialize with default config
//!     void* bus = net_init("{\"num_shards\": 4}");
//!     if (!bus) return 1;
//!
//!     // Ingest an event
//!     int result = net_ingest(bus, "{\"token\": \"hello\"}", 19);
//!     if (result < 0) { /* handle error */ }
//!
//!     // Poll events
//!     char buffer[65536];
//!     result = net_poll(bus, "{\"limit\": 100}", buffer, sizeof(buffer));
//!
//!     // Shutdown
//!     net_shutdown(bus);
//!     return 0;
//! }
//! ```

// FFI functions accept raw pointers but are not marked `unsafe` to maintain
// C ABI compatibility. Safety is documented in the module-level docs.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;

use tokio::runtime::Runtime;

use crate::bus::EventBus;
use crate::config::EventBusConfig;
use crate::consumer::ConsumeRequest;
use crate::event::{Event, RawEvent};

/// C FFI for CortEX / NetDb / RedexFile. Requires `netdb` (for the
/// unified facade) and `redex-disk` (for persistent storage paths on
/// `Redex` / `RedexFile`). Go / cgo consumers target this surface.
///
/// `missing_docs` is suppressed on this module: these are extern "C"
/// shims over already-documented Rust adapters, and the per-function
/// contract is documented in the binding-side READMEs (Go / TS / Py).
/// Re-documenting each shim would duplicate with drift risk.
#[cfg(all(feature = "netdb", feature = "redex-disk"))]
#[allow(missing_docs)]
pub mod cortex;

/// C FFI for the encrypted-UDP mesh transport + channels. Requires
/// the `net` feature (which brings in the crypto + transport). Go /
/// cgo consumers target this surface alongside `ffi::cortex`. See
/// the `ffi::cortex` note for why `missing_docs` is suppressed here.
#[cfg(feature = "net")]
#[allow(missing_docs)]
pub mod mesh;

#[cfg(feature = "net")]
use crate::adapter::net::{NetAdapterConfig, ReliabilityConfig, StaticKeypair};
#[cfg(any(feature = "redis", feature = "jetstream", feature = "net"))]
use crate::config::AdapterConfig;
#[cfg(feature = "jetstream")]
use crate::config::JetStreamAdapterConfig;
#[cfg(feature = "redis")]
use crate::config::RedisAdapterConfig;
#[cfg(feature = "net")]
use std::ffi::CString;

/// Opaque handle to an event bus instance.
///
/// This wraps the EventBus along with a Tokio runtime for async operations.
/// The handle is reference-counted so that concurrent FFI calls keep it alive
/// even if `net_shutdown` races against them.
pub struct NetHandle {
    bus: EventBus,
    runtime: Runtime,
    /// Set to `true` once `net_shutdown` begins. All other FFI functions
    /// check this flag and return `ShuttingDown` early, reducing the
    /// window for use-after-free when a C caller races shutdown against
    /// concurrent operations.
    shutting_down: std::sync::atomic::AtomicBool,
    /// Number of in-flight FFI operations (excluding shutdown itself).
    /// `net_shutdown` spins until this drops to zero before deallocating.
    active_ops: std::sync::atomic::AtomicU32,
}

/// RAII guard that increments `active_ops` on creation and decrements on drop.
struct FfiOpGuard<'a> {
    handle: &'a NetHandle,
}

impl<'a> FfiOpGuard<'a> {
    /// Try to enter an FFI operation. Returns `None` if the handle is shutting down.
    ///
    /// The fetch_add on `active_ops` and the load of `shutting_down` here,
    /// together with the store on `shutting_down` and the load of
    /// `active_ops` in `net_shutdown`, form a Dekker-style handshake
    /// across two separate atomics. Release/Acquire only orders operations
    /// on the *same* variable, so it is legal under that ordering for both
    /// sides to see the "pre" value of the other atomic simultaneously —
    /// the caller would then proceed into an FFI op while the shutdown
    /// frees the handle, causing a use-after-free. `SeqCst` imposes a
    /// single total order over these four operations, eliminating that
    /// possibility.
    fn try_enter(handle: &'a NetHandle) -> Option<Self> {
        handle
            .active_ops
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if handle
            .shutting_down
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            handle
                .active_ops
                .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
            None
        } else {
            Some(Self { handle })
        }
    }
}

impl Drop for FfiOpGuard<'_> {
    fn drop(&mut self) {
        self.handle
            .active_ops
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
}

/// Error codes returned by FFI functions.
#[repr(C)]
pub enum NetError {
    /// Success (no error).
    Success = 0,
    /// Null pointer passed.
    NullPointer = -1,
    /// Invalid UTF-8 string.
    InvalidUtf8 = -2,
    /// Invalid JSON.
    InvalidJson = -3,
    /// Initialization failed.
    InitFailed = -4,
    /// Ingestion failed (backpressure).
    IngestionFailed = -5,
    /// Poll failed.
    PollFailed = -6,
    /// Buffer too small.
    BufferTooSmall = -7,
    /// Shutting down.
    ShuttingDown = -8,
    /// Integer overflow: result does not fit in `c_int`.
    IntOverflow = -9,
    /// Unknown error.
    Unknown = -99,
}

impl From<NetError> for c_int {
    fn from(e: NetError) -> Self {
        e as c_int
    }
}

/// Enter an FFI operation with lifetime protection. Returns an `FfiOpGuard`
/// that prevents `net_shutdown` from deallocating the handle until the guard
/// is dropped. Returns `Err` with the error code if shutdown is in progress.
#[inline]
fn enter_ffi_op(handle: &NetHandle) -> Result<FfiOpGuard<'_>, c_int> {
    FfiOpGuard::try_enter(handle).ok_or(NetError::ShuttingDown.into())
}

/// Initialize a new event bus.
///
/// # Parameters
///
/// - `config_json`: JSON configuration string (UTF-8, null-terminated).
///   Pass NULL or empty string for default configuration.
///
/// # Returns
///
/// Opaque handle to the event bus, or NULL on failure.
/// The handle must be freed with `net_shutdown`.
///
/// # Example Configuration
///
/// ```json
/// {
///   "num_shards": 8,
///   "ring_buffer_capacity": 1048576,
///   "backpressure_mode": "DropOldest",
///   "batch": {
///     "min_size": 1000,
///     "max_size": 10000,
///     "max_delay_ms": 10
///   }
/// }
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn net_init(config_json: *const c_char) -> *mut NetHandle {
    // Create runtime
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    // Parse config
    let config = if config_json.is_null() {
        EventBusConfig::default()
    } else {
        let config_str = match unsafe { CStr::from_ptr(config_json) }.to_str() {
            Ok("") => return create_with_default(runtime),
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };

        match parse_config_json(config_str) {
            Some(cfg) => cfg,
            None => return ptr::null_mut(),
        }
    };

    create_with_config(runtime, config)
}

fn create_with_default(runtime: Runtime) -> *mut NetHandle {
    create_with_config(runtime, EventBusConfig::default())
}

/// Parse JSON configuration into EventBusConfig.
///
/// Supports:
/// - `num_shards`: number of shards
/// - `ring_buffer_capacity`: ring buffer size per shard
/// - `backpressure_mode`: "DropNewest", "DropOldest", "FailProducer"
fn parse_config_json(json_str: &str) -> Option<EventBusConfig> {
    let value: serde_json::Value = serde_json::from_str(json_str).ok()?;

    let mut builder = EventBusConfig::builder();

    if let Some(num_shards) = value.get("num_shards").and_then(|v| v.as_u64()) {
        let num_shards = u16::try_from(num_shards).ok()?;
        builder = builder.num_shards(num_shards);
    }

    if let Some(capacity) = value.get("ring_buffer_capacity").and_then(|v| v.as_u64()) {
        let capacity = usize::try_from(capacity).ok()?;
        builder = builder.ring_buffer_capacity(capacity);
    }

    if let Some(mode) = value.get("backpressure_mode").and_then(|v| v.as_str()) {
        let bp_mode = match mode {
            "DropNewest" | "drop_newest" => crate::config::BackpressureMode::DropNewest,
            "DropOldest" | "drop_oldest" => crate::config::BackpressureMode::DropOldest,
            "FailProducer" | "fail_producer" => crate::config::BackpressureMode::FailProducer,
            _ => crate::config::BackpressureMode::DropNewest,
        };
        builder = builder.backpressure_mode(bp_mode);
    }

    // Parse Redis config
    #[cfg(feature = "redis")]
    if let Some(redis) = value.get("redis") {
        if let Some(url) = redis.get("url").and_then(|v| v.as_str()) {
            let mut redis_config = RedisAdapterConfig::new(url);

            if let Some(prefix) = redis.get("prefix").and_then(|v| v.as_str()) {
                redis_config = redis_config.with_prefix(prefix);
            }
            if let Some(max_len) = redis.get("max_stream_len").and_then(|v| v.as_u64()) {
                let max_len = usize::try_from(max_len).ok()?;
                redis_config = redis_config.with_max_stream_len(max_len);
            }
            if let Some(pipeline_size) = redis.get("pipeline_size").and_then(|v| v.as_u64()) {
                let pipeline_size = usize::try_from(pipeline_size).ok()?;
                redis_config = redis_config.with_pipeline_size(pipeline_size);
            }

            builder = builder.adapter(AdapterConfig::Redis(redis_config));
        }
    }

    // Parse JetStream config
    #[cfg(feature = "jetstream")]
    if let Some(jetstream) = value.get("jetstream") {
        if let Some(url) = jetstream.get("url").and_then(|v| v.as_str()) {
            let mut js_config = JetStreamAdapterConfig::new(url);

            if let Some(prefix) = jetstream.get("prefix").and_then(|v| v.as_str()) {
                js_config = js_config.with_prefix(prefix);
            }
            if let Some(max_messages) = jetstream.get("max_messages").and_then(|v| v.as_i64()) {
                js_config = js_config.with_max_messages(max_messages);
            }
            if let Some(replicas) = jetstream.get("replicas").and_then(|v| v.as_u64()) {
                let replicas = usize::try_from(replicas).ok()?;
                js_config = js_config.with_replicas(replicas);
            }

            builder = builder.adapter(AdapterConfig::JetStream(js_config));
        }
    }

    // Parse Net config
    #[cfg(feature = "net")]
    if let Some(net) = value.get("net") {
        let bind_addr: std::net::SocketAddr = net
            .get("bind_addr")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())?;

        let peer_addr: std::net::SocketAddr = net
            .get("peer_addr")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())?;

        let psk: [u8; 32] = net
            .get("psk")
            .and_then(|v| v.as_str())
            .and_then(|s| hex::decode(s).ok())
            .and_then(|v| v.try_into().ok())?;

        let role = net
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("initiator");

        let mut net_config = match role {
            "initiator" => {
                let peer_pubkey: [u8; 32] = net
                    .get("peer_public_key")
                    .and_then(|v| v.as_str())
                    .and_then(|s| hex::decode(s).ok())
                    .and_then(|v| v.try_into().ok())?;
                NetAdapterConfig::initiator(bind_addr, peer_addr, psk, peer_pubkey)
            }
            "responder" => {
                let secret_key: [u8; 32] = net
                    .get("secret_key")
                    .and_then(|v| v.as_str())
                    .and_then(|s| hex::decode(s).ok())
                    .and_then(|v| v.try_into().ok())?;
                let public_key: [u8; 32] = net
                    .get("public_key")
                    .and_then(|v| v.as_str())
                    .and_then(|s| hex::decode(s).ok())
                    .and_then(|v| v.try_into().ok())?;
                let keypair = StaticKeypair::from_keys(secret_key, public_key);
                NetAdapterConfig::responder(bind_addr, peer_addr, psk, keypair)
            }
            _ => return None,
        };

        // Apply optional settings
        if let Some(reliability) = net.get("reliability").and_then(|v| v.as_str()) {
            net_config = net_config.with_reliability(match reliability {
                "light" => ReliabilityConfig::Light,
                "full" => ReliabilityConfig::Full,
                _ => ReliabilityConfig::None,
            });
        }

        if let Some(pool_size) = net.get("packet_pool_size").and_then(|v| v.as_u64()) {
            if let Ok(size) = usize::try_from(pool_size) {
                net_config = net_config.with_pool_size(size);
            }
        }

        if let Some(interval_ms) = net.get("heartbeat_interval_ms").and_then(|v| v.as_u64()) {
            net_config =
                net_config.with_heartbeat_interval(std::time::Duration::from_millis(interval_ms));
        }

        if let Some(timeout_ms) = net.get("session_timeout_ms").and_then(|v| v.as_u64()) {
            net_config =
                net_config.with_session_timeout(std::time::Duration::from_millis(timeout_ms));
        }

        if let Some(batched) = net.get("batched_io").and_then(|v| v.as_bool()) {
            net_config = net_config.with_batched_io(batched);
        }

        builder = builder.adapter(AdapterConfig::Net(Box::new(net_config)));
    }

    builder.build().ok()
}

fn create_with_config(runtime: Runtime, config: EventBusConfig) -> *mut NetHandle {
    let bus = match runtime.block_on(EventBus::new(config)) {
        Ok(bus) => bus,
        Err(_) => return ptr::null_mut(),
    };

    let handle = Box::new(NetHandle {
        bus,
        runtime,
        shutting_down: std::sync::atomic::AtomicBool::new(false),
        active_ops: std::sync::atomic::AtomicU32::new(0),
    });

    Box::into_raw(handle)
}

/// Ingest a single event.
///
/// # Parameters
///
/// - `handle`: Event bus handle from `net_init`.
/// - `event_json`: JSON event string (UTF-8).
/// - `len`: Length of the event string in bytes.
///
/// # Returns
///
/// - `0` on success
/// - Negative error code on failure
#[unsafe(no_mangle)]
pub extern "C" fn net_ingest(
    handle: *mut NetHandle,
    event_json: *const c_char,
    len: usize,
) -> c_int {
    if handle.is_null() || event_json.is_null() {
        return NetError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };

    // Parse event JSON
    let json_bytes = unsafe { std::slice::from_raw_parts(event_json as *const u8, len) };
    let json_str = match std::str::from_utf8(json_bytes) {
        Ok(s) => s,
        Err(_) => return NetError::InvalidUtf8.into(),
    };

    let event = match Event::from_str(json_str) {
        Ok(e) => e,
        Err(_) => return NetError::InvalidJson.into(),
    };

    // Ingest
    match handle.bus.ingest(event) {
        Ok(_) => NetError::Success.into(),
        Err(_) => NetError::IngestionFailed.into(),
    }
}

/// Ingest a raw JSON string (fastest path).
///
/// The JSON string is stored directly without parsing.
/// This is the recommended method for high-throughput ingestion.
///
/// # Parameters
///
/// - `handle`: Event bus handle from `net_init`.
/// - `json`: JSON string (UTF-8).
/// - `len`: Length of the JSON string in bytes.
///
/// # Returns
///
/// - `0` on success
/// - Negative error code on failure
#[unsafe(no_mangle)]
pub extern "C" fn net_ingest_raw(handle: *mut NetHandle, json: *const c_char, len: usize) -> c_int {
    if handle.is_null() || json.is_null() {
        return NetError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };

    let json_bytes = unsafe { std::slice::from_raw_parts(json as *const u8, len) };
    let json_str = match std::str::from_utf8(json_bytes) {
        Ok(s) => s,
        Err(_) => return NetError::InvalidUtf8.into(),
    };

    let raw = RawEvent::from_str(json_str);

    match handle.bus.ingest_raw(raw) {
        Ok(_) => NetError::Success.into(),
        Err(_) => NetError::IngestionFailed.into(),
    }
}

/// Ingest multiple raw JSON strings (fastest batch path).
///
/// # Parameters
///
/// - `handle`: Event bus handle.
/// - `jsons`: Array of pointers to JSON strings.
/// - `lens`: Array of lengths for each JSON string.
/// - `count`: Number of events in the arrays.
///
/// # Returns
///
/// Number of successfully ingested events, or negative error code.
#[unsafe(no_mangle)]
pub extern "C" fn net_ingest_raw_batch(
    handle: *mut NetHandle,
    jsons: *const *const c_char,
    lens: *const usize,
    count: usize,
) -> c_int {
    if handle.is_null() || jsons.is_null() || lens.is_null() {
        return NetError::NullPointer.into();
    }
    if count == 0 {
        return 0;
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };
    let mut events = Vec::with_capacity(count);

    for i in 0..count {
        let json_ptr = unsafe { *jsons.add(i) };
        let len = unsafe { *lens.add(i) };

        if json_ptr.is_null() {
            continue;
        }

        let json_bytes = unsafe { std::slice::from_raw_parts(json_ptr as *const u8, len) };
        if let Ok(json_str) = std::str::from_utf8(json_bytes) {
            events.push(RawEvent::from_str(json_str));
        }
    }

    let count = handle.bus.ingest_raw_batch(events);
    // Returning `c_int::MAX` on overflow would be ambiguous with a real
    // `INT_MAX` ingest. Signal overflow explicitly so callers doing
    // accounting in high-throughput paths do not silently miscount.
    c_int::try_from(count).unwrap_or_else(|_| NetError::IntOverflow.into())
}

/// Ingest multiple events.
///
/// # Parameters
///
/// - `handle`: Event bus handle.
/// - `events_json`: JSON array of events (UTF-8, null-terminated).
///
/// # Returns
///
/// Number of successfully ingested events, or negative error code.
#[unsafe(no_mangle)]
pub extern "C" fn net_ingest_batch(handle: *mut NetHandle, events_json: *const c_char) -> c_int {
    if handle.is_null() || events_json.is_null() {
        return NetError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };

    let json_str = match unsafe { CStr::from_ptr(events_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return NetError::InvalidUtf8.into(),
    };

    // Parse as JSON array
    let array: Vec<serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(a) => a,
        Err(_) => return NetError::InvalidJson.into(),
    };

    let events: Vec<Event> = array.into_iter().map(Event::new).collect();
    let count = handle.bus.ingest_batch(events);

    // Returning `c_int::MAX` on overflow would be ambiguous with a real
    // `INT_MAX` ingest. Signal overflow explicitly — matches the
    // `net_ingest_raw_batch` contract.
    c_int::try_from(count).unwrap_or_else(|_| NetError::IntOverflow.into())
}

/// Parse the JSON request body passed to `net_poll` into a
/// `ConsumeRequest`. Returns the negative `NetError` code on parse
/// failure so the caller can surface it back across FFI. Both `limit`
/// and `cursor` are optional, but if either key is present with the
/// wrong JSON type it is an explicit error — silently falling back to
/// the default would hide caller bugs (e.g. the Go binding that
/// previously serialized `cursor` but had it dropped server-side).
fn parse_poll_request_json(json_str: &str) -> Result<ConsumeRequest, c_int> {
    let value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|_| c_int::from(NetError::InvalidJson))?;

    let limit = match value.get("limit") {
        None | Some(serde_json::Value::Null) => 100usize,
        Some(v) => match v.as_u64() {
            Some(n) => n as usize,
            None => return Err(NetError::InvalidJson.into()),
        },
    };
    let cursor = match value.get("cursor") {
        None | Some(serde_json::Value::Null) => None,
        Some(v) => match v.as_str() {
            Some(s) => Some(s.to_owned()),
            None => return Err(NetError::InvalidJson.into()),
        },
    };
    let mut req = ConsumeRequest::new(limit);
    req.from_id = cursor;
    Ok(req)
}

/// Poll events from the bus.
///
/// # Parameters
///
/// - `handle`: Event bus handle.
/// - `request_json`: JSON request string (UTF-8, null-terminated).
///   Example: `{"limit": 100, "ordering": "InsertionTs"}`
/// - `out_buffer`: Output buffer for JSON response.
/// - `buffer_len`: Size of the output buffer.
///
/// # Returns
///
/// - Number of bytes written to buffer on success
/// - Negative error code on failure
#[unsafe(no_mangle)]
pub extern "C" fn net_poll(
    handle: *mut NetHandle,
    request_json: *const c_char,
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    if handle.is_null() || out_buffer.is_null() {
        return NetError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };

    // Parse request
    let request = if request_json.is_null() {
        ConsumeRequest::new(100)
    } else {
        let json_str = match unsafe { CStr::from_ptr(request_json) }.to_str() {
            Ok(s) => s,
            Err(_) => return NetError::InvalidUtf8.into(),
        };
        match parse_poll_request_json(json_str) {
            Ok(req) => req,
            Err(code) => return code,
        }
    };

    // Poll
    let response = match handle.runtime.block_on(handle.bus.poll(request)) {
        Ok(r) => r,
        Err(_) => return NetError::PollFailed.into(),
    };

    // Serialize response. Events that fail to parse are included as raw
    // strings so the caller can see all events and detect parse failures.
    let total_events = response.events.len();
    let mut parsed_events: Vec<serde_json::Value> = Vec::with_capacity(total_events);
    let mut parse_errors: usize = 0;
    for e in &response.events {
        match e.parse() {
            Ok(v) => parsed_events.push(v),
            Err(_) => {
                parse_errors += 1;
                // Include the raw bytes as a string so the caller doesn't silently lose events
                if let Ok(raw) = e.raw_str() {
                    parsed_events.push(serde_json::Value::String(raw.to_string()));
                }
            }
        }
    }
    let response_json = match serde_json::to_string(&serde_json::json!({
        "events": parsed_events,
        "next_id": response.next_id,
        "has_more": response.has_more,
        "count": parsed_events.len(),
        "parse_errors": parse_errors,
    })) {
        Ok(s) => s,
        Err(_) => return NetError::Unknown.into(),
    };

    // Check buffer size
    if response_json.len() + 1 > buffer_len {
        return NetError::BufferTooSmall.into();
    }

    // Copy to output buffer
    unsafe {
        ptr::copy_nonoverlapping(
            response_json.as_ptr() as *const c_char,
            out_buffer,
            response_json.len(),
        );
        *out_buffer.add(response_json.len()) = 0; // Null terminate
    }

    match c_int::try_from(response_json.len()) {
        Ok(n) => n,
        Err(_) => NetError::BufferTooSmall.into(),
    }
}

/// Get event bus statistics.
///
/// # Parameters
///
/// - `handle`: Event bus handle.
/// - `out_buffer`: Output buffer for JSON statistics.
/// - `buffer_len`: Size of the output buffer.
///
/// # Returns
///
/// Number of bytes written, or negative error code.
#[unsafe(no_mangle)]
pub extern "C" fn net_stats(
    handle: *mut NetHandle,
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    if handle.is_null() || out_buffer.is_null() {
        return NetError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };
    let stats = handle.bus.stats();
    let shard_stats = handle.bus.shard_stats();

    let stats_json = match serde_json::to_string(&serde_json::json!({
        "events_ingested": stats.events_ingested.load(std::sync::atomic::Ordering::Relaxed),
        "events_dropped": stats.events_dropped.load(std::sync::atomic::Ordering::Relaxed),
        "batches_dispatched": stats.batches_dispatched.load(std::sync::atomic::Ordering::Relaxed),
        "shard_events_ingested": shard_stats.events_ingested,
        "shard_events_dropped": shard_stats.events_dropped,
        "shard_batches_dispatched": shard_stats.batches_dispatched,
    })) {
        Ok(s) => s,
        Err(_) => return NetError::Unknown.into(),
    };

    if stats_json.len() + 1 > buffer_len {
        return NetError::BufferTooSmall.into();
    }

    unsafe {
        ptr::copy_nonoverlapping(
            stats_json.as_ptr() as *const c_char,
            out_buffer,
            stats_json.len(),
        );
        *out_buffer.add(stats_json.len()) = 0;
    }

    match c_int::try_from(stats_json.len()) {
        Ok(n) => n,
        Err(_) => NetError::BufferTooSmall.into(),
    }
}

/// Flush all pending batches to the adapter.
///
/// # Parameters
///
/// - `handle`: Event bus handle.
///
/// # Returns
///
/// - `0` on success
/// - Negative error code on failure
#[unsafe(no_mangle)]
pub extern "C" fn net_flush(handle: *mut NetHandle) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };

    match handle.runtime.block_on(handle.bus.flush()) {
        Ok(_) => NetError::Success.into(),
        Err(_) => NetError::Unknown.into(),
    }
}

/// Shut down the event bus and free resources.
///
/// # Parameters
///
/// - `handle`: Event bus handle. After this call, the handle is invalid.
///
/// # Returns
///
/// - `0` on success
/// - Negative error code on failure
#[unsafe(no_mangle)]
pub extern "C" fn net_shutdown(handle: *mut NetHandle) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }

    // Signal shutdown *before* taking ownership so that concurrent FFI
    // calls on other threads see the flag and bail out early.
    //
    // The store on `shutting_down` and the subsequent load on `active_ops`
    // both use `SeqCst` to pair with `FfiOpGuard::try_enter` — see the
    // comment there for why Release/Acquire is not sufficient for this
    // Dekker-style handshake across two atomics.
    let handle_ref = unsafe { &*handle };
    handle_ref
        .shutting_down
        .store(true, std::sync::atomic::Ordering::SeqCst);

    // Spin-wait until all in-flight FFI operations have completed.
    // Each operation holds an FfiOpGuard that decrements active_ops on drop,
    // so this loop is bounded by the longest concurrent FFI call.
    while handle_ref
        .active_ops
        .load(std::sync::atomic::Ordering::SeqCst)
        > 0
    {
        std::hint::spin_loop();
    }

    let handle = unsafe { Box::from_raw(handle) };
    let NetHandle { bus, runtime, .. } = *handle;

    // Flush pending batches and gracefully shut down the adapter
    // before dropping the runtime. Without this, pending events in
    // ring buffers and batch workers would be silently lost.
    let result = runtime.block_on(bus.shutdown());

    match result {
        Ok(()) => NetError::Success.into(),
        Err(_) => NetError::Unknown.into(),
    }
}

/// Get the number of shards.
///
/// # Parameters
///
/// - `handle`: Event bus handle.
///
/// # Returns
///
/// Number of shards, or 0 if handle is null.
#[unsafe(no_mangle)]
pub extern "C" fn net_num_shards(handle: *mut NetHandle) -> u16 {
    if handle.is_null() {
        return 0;
    }
    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(_) => return 0,
    };
    handle.bus.num_shards()
}

/// Get the library version.
///
/// # Returns
///
/// Version string (static, do not free).
#[unsafe(no_mangle)]
pub extern "C" fn net_version() -> *const c_char {
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
}

/// Generate a new Net keypair.
///
/// # Returns
///
/// JSON string with hex-encoded public_key and secret_key.
/// The caller must free the returned string with `net_free_string`.
/// Returns NULL if Net feature is not enabled.
#[cfg(feature = "net")]
#[unsafe(no_mangle)]
pub extern "C" fn net_generate_keypair() -> *mut c_char {
    let keypair = StaticKeypair::generate();
    let json = serde_json::json!({
        "public_key": hex::encode(keypair.public_key()),
        "secret_key": hex::encode(keypair.secret_key()),
    });

    match CString::new(json.to_string()) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a string returned by Net functions.
///
/// # Parameters
///
/// - `s`: String pointer returned by `net_generate_keypair` or similar.
#[cfg(feature = "net")]
#[unsafe(no_mangle)]
pub extern "C" fn net_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

// =========================================================================
// Structured (non-JSON) API — _ex variants
// =========================================================================

/// Ingestion receipt for C consumers.
#[repr(C)]
pub struct NetReceipt {
    /// Shard the event was assigned to.
    pub shard_id: u16,
    /// Insertion timestamp (nanoseconds).
    pub timestamp: u64,
}

/// A single stored event for C consumers.
#[repr(C)]
pub struct NetEvent {
    /// Event ID (not null-terminated, use `id_len`).
    pub id: *const c_char,
    /// Length of the event ID.
    pub id_len: usize,
    /// Raw JSON payload (not null-terminated, use `raw_len`).
    pub raw: *const c_char,
    /// Length of the raw JSON payload.
    pub raw_len: usize,
    /// Insertion timestamp (nanoseconds).
    pub insertion_ts: u64,
    /// Shard ID.
    pub shard_id: u16,
}

/// Poll result for C consumers.
#[repr(C)]
pub struct NetPollResult {
    /// Array of events. Free with `net_free_poll_result`.
    pub events: *mut NetEvent,
    /// Number of events in the array.
    pub count: usize,
    /// Cursor for the next poll (null-terminated). NULL if no more.
    pub next_id: *mut c_char,
    /// 1 if more events are available, 0 otherwise.
    pub has_more: c_int,
}

/// Stats for C consumers.
#[repr(C)]
pub struct NetStats {
    /// Total events ingested.
    pub events_ingested: u64,
    /// Events dropped due to backpressure.
    pub events_dropped: u64,
    /// Batches dispatched to the adapter.
    pub batches_dispatched: u64,
}

/// Ingest raw JSON with structured receipt.
#[unsafe(no_mangle)]
pub extern "C" fn net_ingest_raw_ex(
    handle: *mut NetHandle,
    json: *const c_char,
    len: usize,
    out: *mut NetReceipt,
) -> c_int {
    if handle.is_null() || json.is_null() {
        return NetError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };

    let json_bytes = unsafe { std::slice::from_raw_parts(json as *const u8, len) };
    let json_str = match std::str::from_utf8(json_bytes) {
        Ok(s) => s,
        Err(_) => return NetError::InvalidUtf8.into(),
    };

    let raw = RawEvent::from_str(json_str);

    match handle.bus.ingest_raw(raw) {
        Ok((shard_id, timestamp)) => {
            if !out.is_null() {
                unsafe {
                    (*out).shard_id = shard_id;
                    (*out).timestamp = timestamp;
                }
            }
            NetError::Success.into()
        }
        Err(_) => NetError::IngestionFailed.into(),
    }
}

/// Poll events with structured result (no JSON overhead).
///
/// The caller must free the result with `net_free_poll_result`.
#[unsafe(no_mangle)]
pub extern "C" fn net_poll_ex(
    handle: *mut NetHandle,
    limit: usize,
    cursor: *const c_char,
    out: *mut NetPollResult,
) -> c_int {
    if handle.is_null() || out.is_null() {
        return NetError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };

    let mut request = ConsumeRequest::new(limit);
    if !cursor.is_null() {
        if let Ok(s) = unsafe { CStr::from_ptr(cursor) }.to_str() {
            if !s.is_empty() {
                request = request.from(s);
            }
        }
    }

    let response = match handle.runtime.block_on(handle.bus.poll(request)) {
        Ok(r) => r,
        Err(_) => return NetError::PollFailed.into(),
    };

    let count = response.events.len();

    // Allocate events array.
    let events_ptr = if count > 0 {
        let layout = match std::alloc::Layout::array::<NetEvent>(count) {
            Ok(l) => l,
            Err(_) => return NetError::Unknown.into(),
        };
        let ptr = unsafe { std::alloc::alloc(layout) as *mut NetEvent };
        if ptr.is_null() {
            return NetError::Unknown.into();
        }

        for (i, event) in response.events.iter().enumerate() {
            // Leak id and raw strings so they live until net_free_poll_result.
            let id_bytes = event.id.as_bytes().to_vec().into_boxed_slice();
            let id_len = id_bytes.len();
            let id_ptr = Box::into_raw(id_bytes) as *const c_char;

            let raw_bytes = event.raw.to_vec().into_boxed_slice();
            let raw_len = raw_bytes.len();
            let raw_ptr = Box::into_raw(raw_bytes) as *const c_char;

            unsafe {
                ptr.add(i).write(NetEvent {
                    id: id_ptr,
                    id_len,
                    raw: raw_ptr,
                    raw_len,
                    insertion_ts: event.insertion_ts,
                    shard_id: event.shard_id,
                });
            }
        }
        ptr
    } else {
        ptr::null_mut()
    };

    // Leak next_id if present.
    let next_id_ptr = match response.next_id {
        Some(ref s) => match std::ffi::CString::new(s.as_str()) {
            Ok(c) => c.into_raw(),
            Err(_) => {
                // Free already-allocated events before returning error
                free_events_array(events_ptr, count);
                return NetError::InvalidUtf8.into();
            }
        },
        None => ptr::null_mut(),
    };

    unsafe {
        (*out).events = events_ptr;
        (*out).count = count;
        (*out).next_id = next_id_ptr;
        (*out).has_more = if response.has_more { 1 } else { 0 };
    }

    NetError::Success.into()
}

/// Free an events array and all its id/raw allocations.
fn free_events_array(events: *mut NetEvent, count: usize) {
    if events.is_null() || count == 0 {
        return;
    }
    for i in 0..count {
        let event = unsafe { &*events.add(i) };
        if !event.id.is_null() {
            unsafe {
                let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                    event.id as *mut u8,
                    event.id_len,
                ));
            }
        }
        if !event.raw.is_null() {
            unsafe {
                let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                    event.raw as *mut u8,
                    event.raw_len,
                ));
            }
        }
    }
    if let Ok(layout) = std::alloc::Layout::array::<NetEvent>(count) {
        unsafe {
            std::alloc::dealloc(events as *mut u8, layout);
        }
    }
}

/// Free the internal allocations of a poll result returned by `net_poll_ex`.
///
/// This frees the events array (including each event's `id` and `raw` buffers)
/// and the `next_id` string. It does **not** free the `NetPollResult` struct
/// itself, which is caller-provided (typically stack-allocated or managed by
/// the caller).
#[unsafe(no_mangle)]
pub extern "C" fn net_free_poll_result(result: *mut NetPollResult) {
    if result.is_null() {
        return;
    }

    let result = unsafe { &*result };

    // Free events array and all id/raw allocations.
    free_events_array(result.events, result.count);

    // Free next_id.
    if !result.next_id.is_null() {
        unsafe {
            drop(std::ffi::CString::from_raw(result.next_id));
        }
    }
}

/// Get stats without JSON serialization.
#[unsafe(no_mangle)]
pub extern "C" fn net_stats_ex(handle: *mut NetHandle, out: *mut NetStats) -> c_int {
    if handle.is_null() || out.is_null() {
        return NetError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
    let _guard = match enter_ffi_op(handle) {
        Ok(g) => g,
        Err(err) => return err,
    };
    let stats = handle.bus.stats();

    unsafe {
        (*out).events_ingested = stats
            .events_ingested
            .load(std::sync::atomic::Ordering::Relaxed);
        (*out).events_dropped = stats
            .events_dropped
            .load(std::sync::atomic::Ordering::Relaxed);
        (*out).batches_dispatched = stats
            .batches_dispatched
            .load(std::sync::atomic::Ordering::Relaxed);
    }

    NetError::Success.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_valid() {
        let config = parse_config_json(r#"{"num_shards": 8}"#);
        assert!(config.is_some());
    }

    #[test]
    fn test_parse_config_num_shards_overflow() {
        // u16::MAX is 65535, so 65536 should fail
        let config = parse_config_json(r#"{"num_shards": 65536}"#);
        assert!(
            config.is_none(),
            "num_shards exceeding u16::MAX should fail"
        );

        // Much larger value should also fail
        let config = parse_config_json(r#"{"num_shards": 100000}"#);
        assert!(
            config.is_none(),
            "num_shards exceeding u16::MAX should fail"
        );
    }

    #[test]
    fn test_parse_config_num_shards_max_valid() {
        // u16::MAX (65535) should be valid
        let config = parse_config_json(r#"{"num_shards": 65535}"#);
        assert!(config.is_some(), "num_shards at u16::MAX should be valid");
    }

    #[test]
    fn test_parse_config_invalid_json() {
        let config = parse_config_json(r#"{"num_shards": invalid}"#);
        assert!(config.is_none());
    }

    #[test]
    fn test_parse_config_empty() {
        let config = parse_config_json(r#"{}"#);
        assert!(config.is_some(), "empty config should use defaults");
    }

    // Regression: the Go binding's `Poll(limit, cursor)` serializes a
    // `"cursor"` field that the FFI JSON path previously ignored —
    // cross-shard pagination silently broke. `parse_poll_request_json`
    // must round-trip the cursor into `ConsumeRequest.from_id`.
    #[test]
    fn test_parse_poll_request_preserves_cursor() {
        let req = parse_poll_request_json(r#"{"limit": 50, "cursor": "abc:123"}"#).unwrap();
        assert_eq!(req.limit, 50);
        assert_eq!(req.from_id.as_deref(), Some("abc:123"));
    }

    #[test]
    fn test_parse_poll_request_no_cursor_defaults_to_none() {
        let req = parse_poll_request_json(r#"{"limit": 10}"#).unwrap();
        assert_eq!(req.limit, 10);
        assert_eq!(req.from_id, None);
    }

    #[test]
    fn test_parse_poll_request_empty_uses_default_limit() {
        let req = parse_poll_request_json(r#"{}"#).unwrap();
        assert_eq!(req.limit, 100);
        assert_eq!(req.from_id, None);
    }

    // Regression: a wrong-typed `"limit"` previously hit
    // `.as_u64().unwrap_or(100)` and silently defaulted. Caller bugs
    // (e.g. sending a string or a negative number) must surface as
    // `InvalidJson` instead.
    #[test]
    fn test_parse_poll_request_wrong_type_limit_errors() {
        let err = parse_poll_request_json(r#"{"limit": "50"}"#).unwrap_err();
        assert_eq!(err, c_int::from(NetError::InvalidJson));
        let err = parse_poll_request_json(r#"{"limit": -1}"#).unwrap_err();
        assert_eq!(err, c_int::from(NetError::InvalidJson));
    }

    #[test]
    fn test_parse_poll_request_wrong_type_cursor_errors() {
        let err = parse_poll_request_json(r#"{"cursor": 123}"#).unwrap_err();
        assert_eq!(err, c_int::from(NetError::InvalidJson));
    }

    #[test]
    fn test_parse_poll_request_null_fields_use_defaults() {
        let req = parse_poll_request_json(r#"{"limit": null, "cursor": null}"#).unwrap();
        assert_eq!(req.limit, 100);
        assert_eq!(req.from_id, None);
    }
}
