//! C FFI bindings for cross-language integration.
//!
//! This module provides a C-compatible API for using Blackstream from
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
//! - Handles are not used after `blackstream_shutdown`
//!
//! # Thread Safety
//!
//! All FFI functions are thread-safe. The event bus handle can be shared
//! across threads.
//!
//! # Memory Management
//!
//! - Handles returned by `blackstream_init` must be freed with `blackstream_shutdown`
//! - String buffers passed to `blackstream_poll` are owned by the caller
//! - Error codes are returned as integers (0 = success, negative = error)
//!
//! # Example (C)
//!
//! ```c
//! #include "blackstream.h"
//!
//! int main() {
//!     // Initialize with default config
//!     void* bus = blackstream_init("{\"num_shards\": 4}");
//!     if (!bus) return 1;
//!
//!     // Ingest an event
//!     int result = blackstream_ingest(bus, "{\"token\": \"hello\"}", 19);
//!     if (result < 0) { /* handle error */ }
//!
//!     // Poll events
//!     char buffer[65536];
//!     result = blackstream_poll(bus, "{\"limit\": 100}", buffer, sizeof(buffer));
//!
//!     // Shutdown
//!     blackstream_shutdown(bus);
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
use crate::config::{AdapterConfig, EventBusConfig};
use crate::consumer::ConsumeRequest;
use crate::event::{Event, RawEvent};

#[cfg(feature = "bltp")]
use crate::adapter::bltp::{BltpAdapterConfig, ReliabilityConfig, StaticKeypair};
#[cfg(feature = "jetstream")]
use crate::config::JetStreamAdapterConfig;
#[cfg(feature = "redis")]
use crate::config::RedisAdapterConfig;
#[cfg(feature = "bltp")]
use std::ffi::CString;

/// Opaque handle to an event bus instance.
///
/// This wraps the EventBus along with a Tokio runtime for async operations.
pub struct BlackstreamHandle {
    bus: EventBus,
    runtime: Runtime,
}

/// Error codes returned by FFI functions.
#[repr(C)]
pub enum BlackstreamError {
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
    /// Unknown error.
    Unknown = -99,
}

impl From<BlackstreamError> for c_int {
    fn from(e: BlackstreamError) -> Self {
        e as c_int
    }
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
/// The handle must be freed with `blackstream_shutdown`.
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
pub extern "C" fn blackstream_init(config_json: *const c_char) -> *mut BlackstreamHandle {
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

fn create_with_default(runtime: Runtime) -> *mut BlackstreamHandle {
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

    // Parse BLTP config
    #[cfg(feature = "bltp")]
    if let Some(bltp) = value.get("bltp") {
        let bind_addr: std::net::SocketAddr = bltp
            .get("bind_addr")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())?;

        let peer_addr: std::net::SocketAddr = bltp
            .get("peer_addr")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())?;

        let psk: [u8; 32] = bltp
            .get("psk")
            .and_then(|v| v.as_str())
            .and_then(|s| hex::decode(s).ok())
            .and_then(|v| v.try_into().ok())?;

        let role = bltp
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("initiator");

        let mut bltp_config = match role {
            "initiator" => {
                let peer_pubkey: [u8; 32] = bltp
                    .get("peer_public_key")
                    .and_then(|v| v.as_str())
                    .and_then(|s| hex::decode(s).ok())
                    .and_then(|v| v.try_into().ok())?;
                BltpAdapterConfig::initiator(bind_addr, peer_addr, psk, peer_pubkey)
            }
            "responder" => {
                let secret_key: [u8; 32] = bltp
                    .get("secret_key")
                    .and_then(|v| v.as_str())
                    .and_then(|s| hex::decode(s).ok())
                    .and_then(|v| v.try_into().ok())?;
                let public_key: [u8; 32] = bltp
                    .get("public_key")
                    .and_then(|v| v.as_str())
                    .and_then(|s| hex::decode(s).ok())
                    .and_then(|v| v.try_into().ok())?;
                let keypair = StaticKeypair::from_keys(secret_key, public_key);
                BltpAdapterConfig::responder(bind_addr, peer_addr, psk, keypair)
            }
            _ => return None,
        };

        // Apply optional settings
        if let Some(reliability) = bltp.get("reliability").and_then(|v| v.as_str()) {
            bltp_config = bltp_config.with_reliability(match reliability {
                "light" => ReliabilityConfig::Light,
                "full" => ReliabilityConfig::Full,
                _ => ReliabilityConfig::None,
            });
        }

        if let Some(pool_size) = bltp.get("packet_pool_size").and_then(|v| v.as_u64()) {
            if let Ok(size) = usize::try_from(pool_size) {
                bltp_config = bltp_config.with_pool_size(size);
            }
        }

        if let Some(interval_ms) = bltp.get("heartbeat_interval_ms").and_then(|v| v.as_u64()) {
            bltp_config =
                bltp_config.with_heartbeat_interval(std::time::Duration::from_millis(interval_ms));
        }

        if let Some(timeout_ms) = bltp.get("session_timeout_ms").and_then(|v| v.as_u64()) {
            bltp_config =
                bltp_config.with_session_timeout(std::time::Duration::from_millis(timeout_ms));
        }

        if let Some(batched) = bltp.get("batched_io").and_then(|v| v.as_bool()) {
            bltp_config = bltp_config.with_batched_io(batched);
        }

        builder = builder.adapter(AdapterConfig::Bltp(Box::new(bltp_config)));
    }

    builder.build().ok()
}

fn create_with_config(runtime: Runtime, config: EventBusConfig) -> *mut BlackstreamHandle {
    let bus = match runtime.block_on(EventBus::new(config)) {
        Ok(bus) => bus,
        Err(_) => return ptr::null_mut(),
    };

    let handle = Box::new(BlackstreamHandle { bus, runtime });

    Box::into_raw(handle)
}

/// Ingest a single event.
///
/// # Parameters
///
/// - `handle`: Event bus handle from `blackstream_init`.
/// - `event_json`: JSON event string (UTF-8).
/// - `len`: Length of the event string in bytes.
///
/// # Returns
///
/// - `0` on success
/// - Negative error code on failure
#[unsafe(no_mangle)]
pub extern "C" fn blackstream_ingest(
    handle: *mut BlackstreamHandle,
    event_json: *const c_char,
    len: usize,
) -> c_int {
    if handle.is_null() || event_json.is_null() {
        return BlackstreamError::NullPointer.into();
    }

    let handle = unsafe { &*handle };

    // Parse event JSON
    let json_bytes = unsafe { std::slice::from_raw_parts(event_json as *const u8, len) };
    let json_str = match std::str::from_utf8(json_bytes) {
        Ok(s) => s,
        Err(_) => return BlackstreamError::InvalidUtf8.into(),
    };

    let event = match Event::from_str(json_str) {
        Ok(e) => e,
        Err(_) => return BlackstreamError::InvalidJson.into(),
    };

    // Ingest
    match handle.bus.ingest(event) {
        Ok(_) => BlackstreamError::Success.into(),
        Err(_) => BlackstreamError::IngestionFailed.into(),
    }
}

/// Ingest a raw JSON string (fastest path).
///
/// The JSON string is stored directly without parsing.
/// This is the recommended method for high-throughput ingestion.
///
/// # Parameters
///
/// - `handle`: Event bus handle from `blackstream_init`.
/// - `json`: JSON string (UTF-8).
/// - `len`: Length of the JSON string in bytes.
///
/// # Returns
///
/// - `0` on success
/// - Negative error code on failure
#[unsafe(no_mangle)]
pub extern "C" fn blackstream_ingest_raw(
    handle: *mut BlackstreamHandle,
    json: *const c_char,
    len: usize,
) -> c_int {
    if handle.is_null() || json.is_null() {
        return BlackstreamError::NullPointer.into();
    }

    let handle = unsafe { &*handle };

    let json_bytes = unsafe { std::slice::from_raw_parts(json as *const u8, len) };
    let json_str = match std::str::from_utf8(json_bytes) {
        Ok(s) => s,
        Err(_) => return BlackstreamError::InvalidUtf8.into(),
    };

    let raw = RawEvent::from_str(json_str);

    match handle.bus.ingest_raw(raw) {
        Ok(_) => BlackstreamError::Success.into(),
        Err(_) => BlackstreamError::IngestionFailed.into(),
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
pub extern "C" fn blackstream_ingest_raw_batch(
    handle: *mut BlackstreamHandle,
    jsons: *const *const c_char,
    lens: *const usize,
    count: usize,
) -> c_int {
    if handle.is_null() || jsons.is_null() || lens.is_null() {
        return BlackstreamError::NullPointer.into();
    }
    if count == 0 {
        return 0;
    }

    let handle = unsafe { &*handle };
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
    c_int::try_from(count).unwrap_or(c_int::MAX)
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
pub extern "C" fn blackstream_ingest_batch(
    handle: *mut BlackstreamHandle,
    events_json: *const c_char,
) -> c_int {
    if handle.is_null() || events_json.is_null() {
        return BlackstreamError::NullPointer.into();
    }

    let handle = unsafe { &*handle };

    let json_str = match unsafe { CStr::from_ptr(events_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return BlackstreamError::InvalidUtf8.into(),
    };

    // Parse as JSON array
    let array: Vec<serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(a) => a,
        Err(_) => return BlackstreamError::InvalidJson.into(),
    };

    let events: Vec<Event> = array.into_iter().map(Event::new).collect();
    let count = handle.bus.ingest_batch(events);

    c_int::try_from(count).unwrap_or(c_int::MAX)
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
pub extern "C" fn blackstream_poll(
    handle: *mut BlackstreamHandle,
    request_json: *const c_char,
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    if handle.is_null() || out_buffer.is_null() {
        return BlackstreamError::NullPointer.into();
    }

    let handle = unsafe { &*handle };

    // Parse request
    let request = if request_json.is_null() {
        ConsumeRequest::new(100)
    } else {
        let json_str = match unsafe { CStr::from_ptr(request_json) }.to_str() {
            Ok(s) => s,
            Err(_) => return BlackstreamError::InvalidUtf8.into(),
        };

        // Parse limit from JSON
        let value: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => return BlackstreamError::InvalidJson.into(),
        };

        let limit = value.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
        ConsumeRequest::new(limit)
    };

    // Poll
    let response = match handle.runtime.block_on(handle.bus.poll(request)) {
        Ok(r) => r,
        Err(_) => return BlackstreamError::PollFailed.into(),
    };

    // Serialize response
    let response_json = match serde_json::to_string(&serde_json::json!({
        "events": response.events.iter().map(|e| e.parse().unwrap_or(serde_json::Value::Null)).collect::<Vec<_>>(),
        "next_id": response.next_id,
        "has_more": response.has_more,
        "count": response.events.len(),
    })) {
        Ok(s) => s,
        Err(_) => return BlackstreamError::Unknown.into(),
    };

    // Check buffer size
    if response_json.len() + 1 > buffer_len {
        return BlackstreamError::BufferTooSmall.into();
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

    response_json.len() as c_int
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
pub extern "C" fn blackstream_stats(
    handle: *mut BlackstreamHandle,
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> c_int {
    if handle.is_null() || out_buffer.is_null() {
        return BlackstreamError::NullPointer.into();
    }

    let handle = unsafe { &*handle };
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
        Err(_) => return BlackstreamError::Unknown.into(),
    };

    if stats_json.len() + 1 > buffer_len {
        return BlackstreamError::BufferTooSmall.into();
    }

    unsafe {
        ptr::copy_nonoverlapping(
            stats_json.as_ptr() as *const c_char,
            out_buffer,
            stats_json.len(),
        );
        *out_buffer.add(stats_json.len()) = 0;
    }

    stats_json.len() as c_int
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
pub extern "C" fn blackstream_flush(handle: *mut BlackstreamHandle) -> c_int {
    if handle.is_null() {
        return BlackstreamError::NullPointer.into();
    }

    let handle = unsafe { &*handle };

    match handle.runtime.block_on(handle.bus.flush()) {
        Ok(_) => BlackstreamError::Success.into(),
        Err(_) => BlackstreamError::Unknown.into(),
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
pub extern "C" fn blackstream_shutdown(handle: *mut BlackstreamHandle) -> c_int {
    if handle.is_null() {
        return BlackstreamError::NullPointer.into();
    }

    let handle = unsafe { Box::from_raw(handle) };

    // Shutdown is consuming, so we need to handle this carefully
    // For now, just drop the handle which will clean up resources
    drop(handle);

    BlackstreamError::Success.into()
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
pub extern "C" fn blackstream_num_shards(handle: *mut BlackstreamHandle) -> u16 {
    if handle.is_null() {
        return 0;
    }
    let handle = unsafe { &*handle };
    handle.bus.num_shards()
}

/// Get the library version.
///
/// # Returns
///
/// Version string (static, do not free).
#[unsafe(no_mangle)]
pub extern "C" fn blackstream_version() -> *const c_char {
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
}

/// Generate a new BLTP keypair.
///
/// # Returns
///
/// JSON string with hex-encoded public_key and secret_key.
/// The caller must free the returned string with `blackstream_free_string`.
/// Returns NULL if BLTP feature is not enabled.
#[cfg(feature = "bltp")]
#[unsafe(no_mangle)]
pub extern "C" fn bltp_generate_keypair() -> *mut c_char {
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

/// Free a string returned by BLTP functions.
///
/// # Parameters
///
/// - `s`: String pointer returned by `bltp_generate_keypair` or similar.
#[cfg(feature = "bltp")]
#[unsafe(no_mangle)]
pub extern "C" fn blackstream_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
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
}
