//! C FFI bindings for the encrypted-UDP mesh transport.
//!
//! Surface targeted at the Go SDK. Mirrors the Rust SDK's `Mesh`
//! type (not the full core `MeshNode`) — just the common path:
//! handshake, per-peer streams, channels, shard receive.
//!
//! Everything crosses the boundary as:
//!
//! - Opaque handles (`*mut T`) freed via dedicated `_free` functions.
//! - Scalar ids as `u64`.
//! - Everything else as JSON strings allocated with
//!   `CString::into_raw`, freed by the caller via `net_free_string`.
//!
//! Handshake + per-peer sends are async on the core side; the FFI
//! drives them via a shared `tokio::runtime::Runtime` (lazy OnceLock)
//! identical to the one used by `ffi/cortex.rs`.

use std::ffi::{c_char, c_int, CStr, CString};
use std::sync::Arc;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::adapter::net::{
    ChannelConfig as InnerChannelConfig, ChannelConfigRegistry, ChannelId,
    ChannelName as InnerChannelName, ChannelPublisher, EntityKeypair, MeshNode, MeshNodeConfig,
    OnFailure as InnerOnFailure, PublishConfig as InnerPublishConfig,
    PublishReport as InnerPublishReport, Reliability, Stream as CoreStream, StreamConfig,
    StreamError, Visibility as InnerVisibility, DEFAULT_STREAM_WINDOW_BYTES,
};
use crate::adapter::Adapter;
use crate::error::AdapterError;

use super::NetError;

// =========================================================================
// Mesh-specific error codes. Continues the -100..-99 range used by
// `ffi/cortex.rs`. The Go layer maps these to typed sentinels.
// =========================================================================

pub(crate) const NET_ERR_MESH_INIT: c_int = -110;
pub(crate) const NET_ERR_MESH_HANDSHAKE: c_int = -111;
pub(crate) const NET_ERR_MESH_BACKPRESSURE: c_int = -112;
pub(crate) const NET_ERR_MESH_NOT_CONNECTED: c_int = -113;
pub(crate) const NET_ERR_MESH_TRANSPORT: c_int = -114;
pub(crate) const NET_ERR_CHANNEL: c_int = -115;
pub(crate) const NET_ERR_CHANNEL_AUTH: c_int = -116;

// =========================================================================
// Shared utilities
// =========================================================================

/// Shared tokio runtime. One per process, lazy-initialized.
fn runtime() -> &'static Arc<Runtime> {
    use std::sync::OnceLock;
    static RT: OnceLock<Arc<Runtime>> = OnceLock::new();
    RT.get_or_init(|| {
        Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("mesh ffi tokio runtime"),
        )
    })
}

#[inline]
unsafe fn c_str_to_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok()
}

fn write_json_out<T: Serialize>(
    value: &T,
    out_ptr: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    let Ok(s) = serde_json::to_string(value) else {
        return NetError::Unknown.into();
    };
    let len = s.len();
    let Ok(cs) = CString::new(s) else {
        return NetError::Unknown.into();
    };
    unsafe {
        *out_ptr = cs.into_raw();
        *out_len = len;
    }
    0
}

fn write_string_out(s: String, out_ptr: *mut *mut c_char, out_len: *mut usize) -> c_int {
    let len = s.len();
    let Ok(cs) = CString::new(s) else {
        return NetError::Unknown.into();
    };
    unsafe {
        *out_ptr = cs.into_raw();
        *out_len = len;
    }
    0
}

fn adapter_err_to_code(err: &AdapterError) -> c_int {
    match err {
        AdapterError::Connection(_) => NET_ERR_MESH_HANDSHAKE,
        _ => NET_ERR_MESH_TRANSPORT,
    }
}

fn stream_err_to_code(err: &StreamError) -> c_int {
    match err {
        StreamError::Backpressure => NET_ERR_MESH_BACKPRESSURE,
        StreamError::NotConnected => NET_ERR_MESH_NOT_CONNECTED,
        StreamError::Transport(_) => NET_ERR_MESH_TRANSPORT,
    }
}

// =========================================================================
// MeshNode
// =========================================================================

#[derive(Deserialize)]
struct MeshNewConfig {
    bind_addr: String,
    /// Hex-encoded 32-byte pre-shared key.
    psk_hex: String,
    heartbeat_ms: Option<u64>,
    session_timeout_ms: Option<u64>,
    num_shards: Option<u16>,
}

pub struct MeshNodeHandle {
    inner: Arc<MeshNode>,
    channel_configs: Arc<ChannelConfigRegistry>,
}

/// Create a new mesh node. `config_json` is:
///
/// ```json
/// {
///   "bind_addr": "127.0.0.1:9000",
///   "psk_hex":   "42424242...",   // 64 hex chars
///   "heartbeat_ms": 5000,
///   "session_timeout_ms": 30000,
///   "num_shards": 4
/// }
/// ```
///
/// Installs an empty `ChannelConfigRegistry` at creation time so
/// `net_mesh_register_channel` can insert without a mutable ref.
#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_new(
    config_json: *const c_char,
    out_handle: *mut *mut MeshNodeHandle,
) -> c_int {
    if config_json.is_null() || out_handle.is_null() {
        return NetError::NullPointer.into();
    }
    let Some(s) = (unsafe { c_str_to_str(config_json) }) else {
        return NetError::InvalidUtf8.into();
    };
    let cfg: MeshNewConfig = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return NetError::InvalidJson.into(),
    };
    let bind_addr: std::net::SocketAddr = match cfg.bind_addr.parse() {
        Ok(a) => a,
        Err(_) => return NET_ERR_MESH_INIT,
    };
    let psk_bytes = match hex::decode(&cfg.psk_hex) {
        Ok(b) => b,
        Err(_) => return NET_ERR_MESH_INIT,
    };
    if psk_bytes.len() != 32 {
        return NET_ERR_MESH_INIT;
    }
    let mut psk = [0u8; 32];
    psk.copy_from_slice(&psk_bytes);

    let mut node_cfg = MeshNodeConfig::new(bind_addr, psk);
    if let Some(ms) = cfg.heartbeat_ms {
        node_cfg = node_cfg.with_heartbeat_interval(std::time::Duration::from_millis(ms));
    }
    if let Some(ms) = cfg.session_timeout_ms {
        node_cfg = node_cfg.with_session_timeout(std::time::Duration::from_millis(ms));
    }
    if let Some(n) = cfg.num_shards {
        node_cfg = node_cfg.with_num_shards(n);
    }

    let identity = EntityKeypair::generate();
    let rt = runtime();
    let result = rt.block_on(async move { MeshNode::new(identity, node_cfg).await });
    match result {
        Ok(mut node) => {
            let channel_configs = Arc::new(ChannelConfigRegistry::new());
            node.set_channel_configs(channel_configs.clone());
            let handle = Box::new(MeshNodeHandle {
                inner: Arc::new(node),
                channel_configs,
            });
            unsafe {
                *out_handle = Box::into_raw(handle);
            }
            0
        }
        Err(_) => NET_ERR_MESH_INIT,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_free(handle: *mut MeshNodeHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

/// Write the hex-encoded 32-byte Noise static public key of this
/// node to `*out`. Caller frees via `net_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_public_key_hex(
    handle: *mut MeshNodeHandle,
    out_ptr: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if handle.is_null() || out_ptr.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let s = hex::encode(h.inner.public_key());
    write_string_out(s, out_ptr, out_len)
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_node_id(handle: *mut MeshNodeHandle) -> u64 {
    if handle.is_null() {
        return 0;
    }
    let h = unsafe { &*handle };
    h.inner.node_id()
}

/// Connect (initiator). Blocks until the handshake completes.
#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_connect(
    handle: *mut MeshNodeHandle,
    peer_addr: *const c_char,
    peer_pubkey_hex: *const c_char,
    peer_node_id: u64,
) -> c_int {
    if handle.is_null() || peer_addr.is_null() || peer_pubkey_hex.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let Some(addr_s) = (unsafe { c_str_to_str(peer_addr) }) else {
        return NetError::InvalidUtf8.into();
    };
    let addr: std::net::SocketAddr = match addr_s.parse() {
        Ok(a) => a,
        Err(_) => return NET_ERR_MESH_HANDSHAKE,
    };
    let Some(pk_s) = (unsafe { c_str_to_str(peer_pubkey_hex) }) else {
        return NetError::InvalidUtf8.into();
    };
    let pk_bytes = match hex::decode(pk_s) {
        Ok(b) => b,
        Err(_) => return NET_ERR_MESH_HANDSHAKE,
    };
    if pk_bytes.len() != 32 {
        return NET_ERR_MESH_HANDSHAKE;
    }
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&pk_bytes);

    let rt = runtime();
    let node = h.inner.clone();
    match rt.block_on(async move { node.connect(addr, &pk, peer_node_id).await }) {
        Ok(_) => 0,
        Err(e) => adapter_err_to_code(&e),
    }
}

/// Accept an incoming connection (responder). Writes the peer's wire
/// address to `*out_addr` (caller frees via `net_free_string`).
#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_accept(
    handle: *mut MeshNodeHandle,
    peer_node_id: u64,
    out_addr: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if handle.is_null() || out_addr.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let rt = runtime();
    let node = h.inner.clone();
    match rt.block_on(async move { node.accept(peer_node_id).await }) {
        Ok((addr, _)) => write_string_out(addr.to_string(), out_addr, out_len),
        Err(e) => adapter_err_to_code(&e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_start(handle: *mut MeshNodeHandle) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let rt = runtime();
    let node = h.inner.clone();
    // `start` spawns internal tasks via tokio::spawn; run under the
    // shared runtime.
    rt.block_on(async move { node.start() });
    0
}

/// Shut down the node. Must be called before `net_mesh_free` to
/// release network resources. Idempotent.
#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_shutdown(handle: *mut MeshNodeHandle) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let rt = runtime();
    // Only actually shut down if we're the sole holder of the Arc
    // (no outstanding stream handles etc.). Otherwise return OK —
    // the remaining references will keep the node alive until they
    // drop. `shutdown` takes `&self`, so no ownership transfer is
    // needed; the earlier `try_unwrap` against an extra clone of the
    // same `Arc` was unreachable.
    if Arc::strong_count(&h.inner) == 1 {
        match rt.block_on(async { h.inner.shutdown().await }) {
            Ok(()) => 0,
            Err(e) => adapter_err_to_code(&e),
        }
    } else {
        0
    }
}

// =========================================================================
// Streams
// =========================================================================

#[derive(Deserialize, Default)]
struct StreamOpenConfig {
    /// `"reliable" | "fire_and_forget"`. Default `"fire_and_forget"`.
    reliability: Option<String>,
    /// Initial send-credit window in bytes. 0 disables backpressure.
    /// Default: `DEFAULT_STREAM_WINDOW_BYTES` (64 KB).
    window_bytes: Option<u32>,
    fairness_weight: Option<u8>,
}

pub struct MeshStreamHandle {
    stream: CoreStream,
    // Keep the node alive as long as the stream is alive so sends
    // don't race a concurrent shutdown.
    _node: Arc<MeshNode>,
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_open_stream(
    handle: *mut MeshNodeHandle,
    peer_node_id: u64,
    stream_id: u64,
    config_json: *const c_char,
    out_stream: *mut *mut MeshStreamHandle,
) -> c_int {
    if handle.is_null() || out_stream.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let cfg_json: StreamOpenConfig = if config_json.is_null() {
        StreamOpenConfig::default()
    } else {
        let Some(s) = (unsafe { c_str_to_str(config_json) }) else {
            return NetError::InvalidUtf8.into();
        };
        match serde_json::from_str(s) {
            Ok(v) => v,
            Err(_) => return NetError::InvalidJson.into(),
        }
    };
    let reliability = match cfg_json.reliability.as_deref() {
        None | Some("fire_and_forget") => Reliability::FireAndForget,
        Some("reliable") => Reliability::Reliable,
        Some(_) => return NET_ERR_MESH_TRANSPORT,
    };
    let window = cfg_json.window_bytes.unwrap_or(DEFAULT_STREAM_WINDOW_BYTES);
    let weight = cfg_json.fairness_weight.unwrap_or(1);
    let cfg = StreamConfig::new()
        .with_reliability(reliability)
        .with_window_bytes(window)
        .with_fairness_weight(weight);
    match h.inner.open_stream(peer_node_id, stream_id, cfg) {
        Ok(stream) => {
            let sh = Box::new(MeshStreamHandle {
                stream,
                _node: h.inner.clone(),
            });
            unsafe {
                *out_stream = Box::into_raw(sh);
            }
            0
        }
        Err(e) => adapter_err_to_code(&e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_stream_free(handle: *mut MeshStreamHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

/// Collect an array of borrowed `(ptr, len)` pairs into a
/// `Vec<Bytes>`. Caller must keep the pointer / length arrays alive
/// for the duration of the C call.
unsafe fn collect_payloads(
    payloads: *const *const u8,
    lens: *const usize,
    count: usize,
) -> Vec<Bytes> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let ptr = *payloads.add(i);
        let len = *lens.add(i);
        let slice = std::slice::from_raw_parts(ptr, len);
        out.push(Bytes::copy_from_slice(slice));
    }
    out
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_send(
    handle: *mut MeshStreamHandle,
    payloads: *const *const u8,
    lens: *const usize,
    count: usize,
    node_handle: *mut MeshNodeHandle,
) -> c_int {
    if handle.is_null() || node_handle.is_null() {
        return NetError::NullPointer.into();
    }
    if count > 0 && (payloads.is_null() || lens.is_null()) {
        return NetError::NullPointer.into();
    }
    let sh = unsafe { &*handle };
    let nh = unsafe { &*node_handle };
    let rt = runtime();
    let payloads = unsafe { collect_payloads(payloads, lens, count) };
    let node = nh.inner.clone();
    let stream = sh.stream.clone();
    match rt.block_on(async move { node.send_on_stream(&stream, &payloads).await }) {
        Ok(()) => 0,
        Err(e) => stream_err_to_code(&e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_send_with_retry(
    handle: *mut MeshStreamHandle,
    payloads: *const *const u8,
    lens: *const usize,
    count: usize,
    max_retries: u32,
    node_handle: *mut MeshNodeHandle,
) -> c_int {
    if handle.is_null() || node_handle.is_null() {
        return NetError::NullPointer.into();
    }
    if count > 0 && (payloads.is_null() || lens.is_null()) {
        return NetError::NullPointer.into();
    }
    let sh = unsafe { &*handle };
    let nh = unsafe { &*node_handle };
    let rt = runtime();
    let payloads = unsafe { collect_payloads(payloads, lens, count) };
    let node = nh.inner.clone();
    let stream = sh.stream.clone();
    match rt.block_on(async move {
        node.send_with_retry(&stream, &payloads, max_retries as usize)
            .await
    }) {
        Ok(()) => 0,
        Err(e) => stream_err_to_code(&e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_send_blocking(
    handle: *mut MeshStreamHandle,
    payloads: *const *const u8,
    lens: *const usize,
    count: usize,
    node_handle: *mut MeshNodeHandle,
) -> c_int {
    if handle.is_null() || node_handle.is_null() {
        return NetError::NullPointer.into();
    }
    if count > 0 && (payloads.is_null() || lens.is_null()) {
        return NetError::NullPointer.into();
    }
    let sh = unsafe { &*handle };
    let nh = unsafe { &*node_handle };
    let rt = runtime();
    let payloads = unsafe { collect_payloads(payloads, lens, count) };
    let node = nh.inner.clone();
    let stream = sh.stream.clone();
    match rt.block_on(async move { node.send_blocking(&stream, &payloads).await }) {
        Ok(()) => 0,
        Err(e) => stream_err_to_code(&e),
    }
}

#[derive(Serialize)]
struct StreamStatsJson {
    tx_seq: u64,
    rx_seq: u64,
    inbound_pending: u64,
    last_activity_ns: u64,
    active: bool,
    backpressure_events: u64,
    tx_credit_remaining: u32,
    tx_window: u32,
    credit_grants_received: u64,
    credit_grants_sent: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_stream_stats(
    node_handle: *mut MeshNodeHandle,
    peer_node_id: u64,
    stream_id: u64,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if node_handle.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*node_handle };
    match h.inner.stream_stats(peer_node_id, stream_id) {
        Some(s) => {
            let js = StreamStatsJson {
                tx_seq: s.tx_seq,
                rx_seq: s.rx_seq,
                inbound_pending: s.inbound_pending,
                last_activity_ns: s.last_activity_ns,
                active: s.active,
                backpressure_events: s.backpressure_events,
                tx_credit_remaining: s.tx_credit_remaining,
                tx_window: s.tx_window,
                credit_grants_received: s.credit_grants_received,
                credit_grants_sent: s.credit_grants_sent,
            };
            write_json_out(&js, out_json, out_len)
        }
        None => {
            // Encode `null` so Go can distinguish "no such stream"
            // from an error.
            write_string_out("null".to_string(), out_json, out_len)
        }
    }
}

// =========================================================================
// Shard receive
// =========================================================================

#[derive(Serialize)]
struct RecvEventJson {
    id: String,
    /// Base64 payload (binary-safe across the JSON boundary).
    payload_b64: String,
    insertion_ts: u64,
    shard_id: u16,
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_recv_shard(
    handle: *mut MeshNodeHandle,
    shard_id: u16,
    limit: u32,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if handle.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let rt = runtime();
    let node = h.inner.clone();
    let result = rt.block_on(async move { node.poll_shard(shard_id, None, limit as usize).await });
    let result = match result {
        Ok(r) => r,
        Err(e) => return adapter_err_to_code(&e),
    };
    let events: Vec<RecvEventJson> = result
        .events
        .into_iter()
        .map(|e| RecvEventJson {
            id: e.id,
            payload_b64: encode_b64(&e.raw),
            insertion_ts: e.insertion_ts,
            shard_id: e.shard_id,
        })
        .collect();
    write_json_out(&events, out_json, out_len)
}

fn encode_b64(bytes: &[u8]) -> String {
    // Small stdlib-free base64. Net already pulls in `base64` via
    // other deps, but a local encoder keeps this module independent.
    const ALPH: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let chunk = &bytes[i..i + 3];
        s.push(ALPH[(chunk[0] >> 2) as usize] as char);
        s.push(ALPH[(((chunk[0] & 0b11) << 4) | (chunk[1] >> 4)) as usize] as char);
        s.push(ALPH[(((chunk[1] & 0b1111) << 2) | (chunk[2] >> 6)) as usize] as char);
        s.push(ALPH[(chunk[2] & 0b111111) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let b = bytes[i];
        s.push(ALPH[(b >> 2) as usize] as char);
        s.push(ALPH[((b & 0b11) << 4) as usize] as char);
        s.push('=');
        s.push('=');
    } else if rem == 2 {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        s.push(ALPH[(b0 >> 2) as usize] as char);
        s.push(ALPH[(((b0 & 0b11) << 4) | (b1 >> 4)) as usize] as char);
        s.push(ALPH[((b1 & 0b1111) << 2) as usize] as char);
        s.push('=');
    }
    s
}

// =========================================================================
// Channels (distributed pub/sub)
// =========================================================================

#[derive(Deserialize)]
struct ChannelConfigInput {
    name: String,
    visibility: Option<String>,
    reliable: Option<bool>,
    require_token: Option<bool>,
    priority: Option<u8>,
    max_rate_pps: Option<u32>,
}

fn parse_visibility(s: &str) -> Option<InnerVisibility> {
    match s {
        "subnet-local" => Some(InnerVisibility::SubnetLocal),
        "parent-visible" => Some(InnerVisibility::ParentVisible),
        "exported" => Some(InnerVisibility::Exported),
        "global" => Some(InnerVisibility::Global),
        _ => None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_register_channel(
    handle: *mut MeshNodeHandle,
    config_json: *const c_char,
) -> c_int {
    if handle.is_null() || config_json.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let Some(s) = (unsafe { c_str_to_str(config_json) }) else {
        return NetError::InvalidUtf8.into();
    };
    let input: ChannelConfigInput = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return NetError::InvalidJson.into(),
    };
    let name = match InnerChannelName::new(&input.name) {
        Ok(n) => n,
        Err(_) => return NET_ERR_CHANNEL,
    };
    let mut cfg = InnerChannelConfig::new(ChannelId::new(name));
    if let Some(v) = input.visibility {
        let Some(vis) = parse_visibility(&v) else {
            return NET_ERR_CHANNEL;
        };
        cfg = cfg.with_visibility(vis);
    }
    if let Some(r) = input.reliable {
        cfg = cfg.with_reliable(r);
    }
    if let Some(t) = input.require_token {
        cfg = cfg.with_require_token(t);
    }
    if let Some(p) = input.priority {
        cfg = cfg.with_priority(p);
    }
    if let Some(pps) = input.max_rate_pps {
        cfg = cfg.with_rate_limit(pps);
    }
    h.channel_configs.insert(cfg);
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_subscribe_channel(
    handle: *mut MeshNodeHandle,
    publisher_node_id: u64,
    channel: *const c_char,
) -> c_int {
    subscribe_or_unsubscribe(handle, publisher_node_id, channel, true)
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_unsubscribe_channel(
    handle: *mut MeshNodeHandle,
    publisher_node_id: u64,
    channel: *const c_char,
) -> c_int {
    subscribe_or_unsubscribe(handle, publisher_node_id, channel, false)
}

fn subscribe_or_unsubscribe(
    handle: *mut MeshNodeHandle,
    publisher_node_id: u64,
    channel: *const c_char,
    subscribe: bool,
) -> c_int {
    if handle.is_null() || channel.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let Some(s) = (unsafe { c_str_to_str(channel) }) else {
        return NetError::InvalidUtf8.into();
    };
    let name = match InnerChannelName::new(s) {
        Ok(n) => n,
        Err(_) => return NET_ERR_CHANNEL,
    };
    let rt = runtime();
    let node = h.inner.clone();
    let outcome = if subscribe {
        rt.block_on(async move { node.subscribe_channel(publisher_node_id, name).await })
    } else {
        rt.block_on(async move { node.unsubscribe_channel(publisher_node_id, name).await })
    };
    match outcome {
        Ok(()) => 0,
        Err(e) => adapter_err_to_channel_code(&e),
    }
}

fn adapter_err_to_channel_code(err: &AdapterError) -> c_int {
    if let AdapterError::Connection(msg) = err {
        let prefix = "membership request rejected: ";
        if let Some(tail) = msg.strip_prefix(prefix) {
            if tail.trim() == "Some(Unauthorized)" {
                return NET_ERR_CHANNEL_AUTH;
            }
        }
    }
    NET_ERR_CHANNEL
}

#[derive(Deserialize, Default)]
struct PublishConfigInput {
    reliability: Option<String>,
    on_failure: Option<String>,
    max_inflight: Option<u32>,
}

#[derive(Serialize)]
struct PublishReportJson {
    attempted: u32,
    delivered: u32,
    errors: Vec<PublishFailureJson>,
}

#[derive(Serialize)]
struct PublishFailureJson {
    node_id: u64,
    message: String,
}

fn to_publish_report_json(r: InnerPublishReport) -> PublishReportJson {
    PublishReportJson {
        attempted: r.attempted as u32,
        delivered: r.delivered as u32,
        errors: r
            .errors
            .into_iter()
            .map(|(id, e)| PublishFailureJson {
                node_id: id,
                message: format!("{}", e),
            })
            .collect(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_publish(
    handle: *mut MeshNodeHandle,
    channel: *const c_char,
    payload: *const u8,
    len: usize,
    config_json: *const c_char,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if handle.is_null() || channel.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let Some(ch) = (unsafe { c_str_to_str(channel) }) else {
        return NetError::InvalidUtf8.into();
    };
    let name = match InnerChannelName::new(ch) {
        Ok(n) => n,
        Err(_) => return NET_ERR_CHANNEL,
    };
    let cfg_in: PublishConfigInput = if config_json.is_null() {
        PublishConfigInput::default()
    } else {
        let Some(s) = (unsafe { c_str_to_str(config_json) }) else {
            return NetError::InvalidUtf8.into();
        };
        match serde_json::from_str(s) {
            Ok(v) => v,
            Err(_) => return NetError::InvalidJson.into(),
        }
    };
    let reliability = match cfg_in.reliability.as_deref() {
        None | Some("fire_and_forget") => Reliability::FireAndForget,
        Some("reliable") => Reliability::Reliable,
        Some(_) => return NET_ERR_CHANNEL,
    };
    let on_failure = match cfg_in.on_failure.as_deref() {
        None | Some("best_effort") => InnerOnFailure::BestEffort,
        Some("fail_fast") => InnerOnFailure::FailFast,
        Some("collect") => InnerOnFailure::Collect,
        Some(_) => return NET_ERR_CHANNEL,
    };
    let max_inflight = cfg_in.max_inflight.unwrap_or(32) as usize;
    let publish_cfg = InnerPublishConfig {
        reliability,
        on_failure,
        max_inflight,
    };
    let publisher = ChannelPublisher::new(name, publish_cfg);

    // Payload may be NULL only when len == 0.
    let bytes = if len == 0 {
        Bytes::new()
    } else if payload.is_null() {
        return NetError::NullPointer.into();
    } else {
        Bytes::copy_from_slice(unsafe { std::slice::from_raw_parts(payload, len) })
    };

    let rt = runtime();
    let node = h.inner.clone();
    match rt.block_on(async move { node.publish(&publisher, bytes).await }) {
        Ok(report) => {
            let js = to_publish_report_json(report);
            write_json_out(&js, out_json, out_len)
        }
        Err(e) => adapter_err_to_channel_code(&e),
    }
}
