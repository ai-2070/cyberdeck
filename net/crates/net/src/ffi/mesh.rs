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

use crate::adapter::net::identity::{
    EntityId, PermissionToken, TokenCache, TokenError as CoreTokenError, TokenScope,
};
use crate::adapter::net::{
    ChannelConfig as InnerChannelConfig, ChannelConfigRegistry, ChannelId,
    ChannelName as InnerChannelName, ChannelPublisher, EntityKeypair, MeshNode, MeshNodeConfig,
    OnFailure as InnerOnFailure, PublishConfig as InnerPublishConfig,
    PublishReport as InnerPublishReport, Reliability, Stream as CoreStream, StreamConfig,
    StreamError, Visibility as InnerVisibility, DEFAULT_STREAM_WINDOW_BYTES,
};
use crate::adapter::net::{SubnetId, SubnetPolicy, SubnetRule};
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

// Identity + token error codes. Block -120..-129 mirrors the
// `"identity: ..."` / `"token: <kind>"` prefix convention used by
// PyO3 and NAPI; each `kind` gets its own integer so Go callers can
// `errors.Is(err, net.ErrTokenExpired)` without parsing strings.
pub(crate) const NET_ERR_IDENTITY: c_int = -120;
pub(crate) const NET_ERR_TOKEN_INVALID_FORMAT: c_int = -121;
pub(crate) const NET_ERR_TOKEN_INVALID_SIGNATURE: c_int = -122;
pub(crate) const NET_ERR_TOKEN_EXPIRED: c_int = -123;
pub(crate) const NET_ERR_TOKEN_NOT_YET_VALID: c_int = -124;
pub(crate) const NET_ERR_TOKEN_DELEGATION_EXHAUSTED: c_int = -125;
pub(crate) const NET_ERR_TOKEN_DELEGATION_NOT_ALLOWED: c_int = -126;
pub(crate) const NET_ERR_TOKEN_NOT_AUTHORIZED: c_int = -127;

fn token_err_to_code(e: &CoreTokenError) -> c_int {
    match e {
        CoreTokenError::InvalidFormat => NET_ERR_TOKEN_INVALID_FORMAT,
        CoreTokenError::InvalidSignature => NET_ERR_TOKEN_INVALID_SIGNATURE,
        CoreTokenError::Expired => NET_ERR_TOKEN_EXPIRED,
        CoreTokenError::NotYetValid => NET_ERR_TOKEN_NOT_YET_VALID,
        CoreTokenError::DelegationExhausted => NET_ERR_TOKEN_DELEGATION_EXHAUSTED,
        CoreTokenError::DelegationNotAllowed => NET_ERR_TOKEN_DELEGATION_NOT_ALLOWED,
        CoreTokenError::NotAuthorized => NET_ERR_TOKEN_NOT_AUTHORIZED,
    }
}

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
struct SubnetPolicyJson {
    #[serde(default)]
    rules: Vec<SubnetRuleJson>,
}

#[derive(Deserialize)]
struct SubnetRuleJson {
    tag_prefix: String,
    level: u32,
    #[serde(default)]
    values: std::collections::HashMap<String, u32>,
}

fn u8_from_u32(value: u32) -> Option<u8> {
    if value > 255 {
        None
    } else {
        Some(value as u8)
    }
}

fn subnet_id_from_json(levels: Vec<u32>) -> Option<SubnetId> {
    if levels.is_empty() || levels.len() > 4 {
        return None;
    }
    let mut bytes = [0u8; 4];
    for (i, raw) in levels.iter().enumerate() {
        bytes[i] = u8_from_u32(*raw)?;
    }
    Some(SubnetId::new(&bytes[..levels.len()]))
}

fn subnet_policy_from_json(p: SubnetPolicyJson) -> Option<SubnetPolicy> {
    let mut policy = SubnetPolicy::new();
    for rule_json in p.rules {
        let level = u8_from_u32(rule_json.level)?;
        if level > 3 {
            return None;
        }
        let mut rule = SubnetRule::new(rule_json.tag_prefix, level);
        for (tag_value, raw_val) in rule_json.values {
            let v = u8_from_u32(raw_val)?;
            rule = rule.map(tag_value, v);
        }
        policy = policy.add_rule(rule);
    }
    Some(policy)
}

#[derive(Deserialize)]
struct MeshNewConfig {
    bind_addr: String,
    /// Hex-encoded 32-byte pre-shared key.
    psk_hex: String,
    heartbeat_ms: Option<u64>,
    session_timeout_ms: Option<u64>,
    num_shards: Option<u16>,
    /// Capability GC interval (ms). Drives eviction of stale
    /// capability index entries.
    capability_gc_interval_ms: Option<u64>,
    /// Reject unsigned capability announcements when `true`.
    /// Defaults to the core's default (`false` in v1).
    require_signed_capabilities: Option<bool>,
    /// 1–4 bytes, each 0–255. Leave unset for `SubnetId::GLOBAL`.
    subnet: Option<Vec<u32>>,
    /// Optional `{"rules": [{"tag_prefix", "level", "values"}]}` policy.
    subnet_policy: Option<SubnetPolicyJson>,
    /// Hex-encoded 32-byte ed25519 seed — when present, the mesh
    /// reproduces the same `entity_id` as
    /// `IdentityFromSeed(sameSeed)`. Leave unset to generate a fresh
    /// keypair.
    identity_seed_hex: Option<String>,
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
    if let Some(ms) = cfg.capability_gc_interval_ms {
        node_cfg = node_cfg.with_capability_gc_interval(std::time::Duration::from_millis(ms));
    }
    if let Some(b) = cfg.require_signed_capabilities {
        node_cfg = node_cfg.with_require_signed_capabilities(b);
    }
    if let Some(levels) = cfg.subnet {
        let Some(id) = subnet_id_from_json(levels) else {
            return NET_ERR_MESH_INIT;
        };
        node_cfg = node_cfg.with_subnet(id);
    }
    if let Some(policy_js) = cfg.subnet_policy {
        let Some(policy) = subnet_policy_from_json(policy_js) else {
            return NET_ERR_MESH_INIT;
        };
        node_cfg = node_cfg.with_subnet_policy(Arc::new(policy));
    }

    let identity = match cfg.identity_seed_hex {
        Some(seed_hex) => {
            let bytes = match hex::decode(&seed_hex) {
                Ok(b) => b,
                Err(_) => return NET_ERR_MESH_INIT,
            };
            if bytes.len() != 32 {
                return NET_ERR_MESH_INIT;
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            EntityKeypair::from_bytes(arr)
        }
        None => EntityKeypair::generate(),
    };
    let rt = runtime();
    let result = rt.block_on(async move { MeshNode::new(identity, node_cfg).await });
    match result {
        Ok(mut node) => {
            let channel_configs = Arc::new(ChannelConfigRegistry::new());
            node.set_channel_configs(channel_configs.clone());
            // Install a fresh TokenCache — channel auth needs
            // somewhere to stash tokens presented on subscribe.
            // Matches the PyO3 / NAPI behaviour.
            node.set_token_cache(Arc::new(TokenCache::new()));
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

/// Writes the 32-byte ed25519 entity id of this mesh into `out[32]`.
/// Matches `Identity::from_seed(seed).entity_id` when the mesh was
/// constructed with `identity_seed_hex = hex::encode(seed)`.
#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_entity_id(handle: *mut MeshNodeHandle, out: *mut u8) -> c_int {
    if handle.is_null() || out.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let bytes = h.inner.entity_id().as_bytes();
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, 32);
    }
    0
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
    /// Capability filter restricting who may publish on this
    /// channel. Same POJO shape as `CapabilityFilter` (see
    /// `net_mesh_find_peers`).
    publish_caps: Option<CapabilityFilterJson>,
    /// Capability filter restricting who may subscribe. Subscribers
    /// whose announced caps miss this filter are rejected with
    /// `NET_ERR_CHANNEL_AUTH`.
    subscribe_caps: Option<CapabilityFilterJson>,
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
    if let Some(filter_json) = input.publish_caps {
        cfg = cfg.with_publish_caps(capability_filter_from_json(filter_json));
    }
    if let Some(filter_json) = input.subscribe_caps {
        cfg = cfg.with_subscribe_caps(capability_filter_from_json(filter_json));
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

/// Subscribe with a serialized `PermissionToken` attached. Parses
/// the token client-side (rejecting malformed bytes with
/// `NET_ERR_TOKEN_INVALID_FORMAT`) before dispatching the request
/// to the publisher. Signature verification happens on the
/// publisher side; a tampered token will surface as
/// `NET_ERR_CHANNEL_AUTH` rather than a token error in this call.
#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_subscribe_channel_with_token(
    handle: *mut MeshNodeHandle,
    publisher_node_id: u64,
    channel: *const c_char,
    token: *const u8,
    token_len: usize,
) -> c_int {
    if handle.is_null() || channel.is_null() || token.is_null() {
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
    let slice = unsafe { std::slice::from_raw_parts(token, token_len) };
    let parsed = match PermissionToken::from_bytes(slice) {
        Ok(t) => t,
        Err(e) => return token_err_to_code(&e),
    };
    let rt = runtime();
    let node = h.inner.clone();
    match rt.block_on(async move {
        node.subscribe_channel_with_token(publisher_node_id, name, parsed)
            .await
    }) {
        Ok(()) => 0,
        Err(e) => adapter_err_to_channel_code(&e),
    }
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

// =========================================================================
// Identity + permission tokens
// =========================================================================

/// Opaque handle holding an ed25519 keypair plus a local
/// `TokenCache`. Matches the PyO3 / NAPI `Identity` pyclass layout —
/// cheap to clone (both fields are `Arc`s inside the core), and the
/// cache is owned by the handle rather than shared across peers.
pub struct IdentityHandle {
    keypair: Arc<EntityKeypair>,
    cache: Arc<TokenCache>,
}

fn alloc_bytes(src: &[u8], out_ptr: *mut *mut u8, out_len: *mut usize) -> c_int {
    let mut vec = src.to_vec();
    vec.shrink_to_fit();
    let len = vec.len();
    let ptr = vec.as_mut_ptr();
    std::mem::forget(vec);
    unsafe {
        *out_ptr = ptr;
        *out_len = len;
    }
    0
}

/// Free a byte buffer allocated by the Rust side (tokens, entity ids
/// returned by reference, etc.). Uses the length returned in the
/// matching out parameter — do not pass `0` or a mismatched length or
/// the Vec reconstruction will tear the allocator state down.
#[unsafe(no_mangle)]
pub extern "C" fn net_free_bytes(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

fn entity_id_from_bytes(bytes: *const u8, len: usize) -> Option<EntityId> {
    if bytes.is_null() || len != 32 {
        return None;
    }
    let slice = unsafe { std::slice::from_raw_parts(bytes, 32) };
    let mut arr = [0u8; 32];
    arr.copy_from_slice(slice);
    Some(EntityId::from_bytes(arr))
}

fn parse_scope_list(raw: &str) -> Option<TokenScope> {
    // JSON array of string scope names — same shape as PyO3's
    // `Vec<String>` parsing. Keeps the ABI aligned to the Python /
    // NAPI surfaces for round-trip fixtures.
    let values: Vec<String> = serde_json::from_str(raw).ok()?;
    let mut acc = TokenScope::NONE;
    for s in &values {
        acc = acc.union(match s.as_str() {
            "publish" => TokenScope::PUBLISH,
            "subscribe" => TokenScope::SUBSCRIBE,
            "admin" => TokenScope::ADMIN,
            "delegate" => TokenScope::DELEGATE,
            _ => return None,
        });
    }
    Some(acc)
}

fn scope_to_strings(scope: TokenScope) -> Vec<&'static str> {
    let mut out = Vec::new();
    if scope.contains(TokenScope::PUBLISH) {
        out.push("publish");
    }
    if scope.contains(TokenScope::SUBSCRIBE) {
        out.push("subscribe");
    }
    if scope.contains(TokenScope::ADMIN) {
        out.push("admin");
    }
    if scope.contains(TokenScope::DELEGATE) {
        out.push("delegate");
    }
    out
}

fn channel_name_to_hash(channel: &str) -> Option<u16> {
    InnerChannelName::new(channel).ok().map(|n| n.hash())
}

/// Generate a fresh ed25519 identity. Writes an owned handle to
/// `*out_handle`. Free via `net_identity_free`.
#[unsafe(no_mangle)]
pub extern "C" fn net_identity_generate(out_handle: *mut *mut IdentityHandle) -> c_int {
    if out_handle.is_null() {
        return NetError::NullPointer.into();
    }
    let handle = Box::new(IdentityHandle {
        keypair: Arc::new(EntityKeypair::generate()),
        cache: Arc::new(TokenCache::new()),
    });
    unsafe {
        *out_handle = Box::into_raw(handle);
    }
    0
}

/// Construct an identity from a caller-owned 32-byte ed25519 seed.
/// Installs a fresh, empty `TokenCache` — reinstall tokens via
/// `net_identity_install_token` after rehydrating from disk.
#[unsafe(no_mangle)]
pub extern "C" fn net_identity_from_seed(
    seed: *const u8,
    seed_len: usize,
    out_handle: *mut *mut IdentityHandle,
) -> c_int {
    if seed.is_null() || out_handle.is_null() {
        return NetError::NullPointer.into();
    }
    if seed_len != 32 {
        return NET_ERR_IDENTITY;
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(unsafe { std::slice::from_raw_parts(seed, 32) });
    let handle = Box::new(IdentityHandle {
        keypair: Arc::new(EntityKeypair::from_bytes(arr)),
        cache: Arc::new(TokenCache::new()),
    });
    unsafe {
        *out_handle = Box::into_raw(handle);
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn net_identity_free(handle: *mut IdentityHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

/// Write the 32-byte ed25519 seed into `out[32]`. Caller must pass
/// a buffer of at least 32 bytes.
#[unsafe(no_mangle)]
pub extern "C" fn net_identity_to_seed(handle: *mut IdentityHandle, out: *mut u8) -> c_int {
    if handle.is_null() || out.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let seed = h.keypair.secret_bytes();
    unsafe {
        std::ptr::copy_nonoverlapping(seed.as_ptr(), out, 32);
    }
    0
}

/// Write the 32-byte entity id into `out[32]`.
#[unsafe(no_mangle)]
pub extern "C" fn net_identity_entity_id(handle: *mut IdentityHandle, out: *mut u8) -> c_int {
    if handle.is_null() || out.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let id = h.keypair.entity_id().as_bytes();
    unsafe {
        std::ptr::copy_nonoverlapping(id.as_ptr(), out, 32);
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn net_identity_node_id(handle: *mut IdentityHandle) -> u64 {
    if handle.is_null() {
        return 0;
    }
    let h = unsafe { &*handle };
    h.keypair.node_id()
}

#[unsafe(no_mangle)]
pub extern "C" fn net_identity_origin_hash(handle: *mut IdentityHandle) -> u32 {
    if handle.is_null() {
        return 0;
    }
    let h = unsafe { &*handle };
    h.keypair.origin_hash()
}

/// Sign `msg[len]` with the identity's ed25519 secret key. Writes a
/// 64-byte signature into `out_sig[64]`.
#[unsafe(no_mangle)]
pub extern "C" fn net_identity_sign(
    handle: *mut IdentityHandle,
    msg: *const u8,
    len: usize,
    out_sig: *mut u8,
) -> c_int {
    if handle.is_null() || out_sig.is_null() {
        return NetError::NullPointer.into();
    }
    if len > 0 && msg.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let slice = if len == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(msg, len) }
    };
    let sig = h.keypair.sign(slice).to_bytes();
    unsafe {
        std::ptr::copy_nonoverlapping(sig.as_ptr(), out_sig, 64);
    }
    0
}

/// Issue a token to `subject`. Writes a newly-allocated blob to
/// `*out_token`; caller frees via `net_free_bytes(ptr, *out_len)`.
#[unsafe(no_mangle)]
pub extern "C" fn net_identity_issue_token(
    signer: *mut IdentityHandle,
    subject: *const u8,
    subject_len: usize,
    scope_json: *const c_char,
    channel: *const c_char,
    ttl_seconds: u32,
    delegation_depth: u8,
    out_token: *mut *mut u8,
    out_token_len: *mut usize,
) -> c_int {
    if signer.is_null() || out_token.is_null() || out_token_len.is_null() {
        return NetError::NullPointer.into();
    }
    let Some(subject_id) = entity_id_from_bytes(subject, subject_len) else {
        return NET_ERR_IDENTITY;
    };
    let Some(scope_s) = (unsafe { c_str_to_str(scope_json) }) else {
        return NetError::InvalidUtf8.into();
    };
    let Some(scope) = parse_scope_list(scope_s) else {
        return NET_ERR_IDENTITY;
    };
    let Some(channel_s) = (unsafe { c_str_to_str(channel) }) else {
        return NetError::InvalidUtf8.into();
    };
    let Some(channel_hash) = channel_name_to_hash(channel_s) else {
        return NET_ERR_IDENTITY;
    };
    let h = unsafe { &*signer };
    let token = PermissionToken::issue(
        &h.keypair,
        subject_id,
        scope,
        channel_hash,
        u64::from(ttl_seconds),
        delegation_depth,
    );
    alloc_bytes(&token.to_bytes(), out_token, out_token_len)
}

/// Install a token received from another issuer. Signature +
/// structural checks run on insert; malformed or tampered tokens
/// return the relevant `NET_ERR_TOKEN_*` code.
#[unsafe(no_mangle)]
pub extern "C" fn net_identity_install_token(
    handle: *mut IdentityHandle,
    token: *const u8,
    len: usize,
) -> c_int {
    if handle.is_null() || token.is_null() {
        return NetError::NullPointer.into();
    }
    let slice = unsafe { std::slice::from_raw_parts(token, len) };
    let parsed = match PermissionToken::from_bytes(slice) {
        Ok(t) => t,
        Err(e) => return token_err_to_code(&e),
    };
    let h = unsafe { &*handle };
    match h.cache.insert(parsed) {
        Ok(()) => 0,
        Err(e) => token_err_to_code(&e),
    }
}

/// Look up a cached token by `(subject, channel)`. Writes a newly-
/// allocated blob to `*out_token` on hit; writes `NULL` / `0` on
/// miss. Caller must always free on hit via `net_free_bytes`.
#[unsafe(no_mangle)]
pub extern "C" fn net_identity_lookup_token(
    handle: *mut IdentityHandle,
    subject: *const u8,
    subject_len: usize,
    channel: *const c_char,
    out_token: *mut *mut u8,
    out_token_len: *mut usize,
) -> c_int {
    if handle.is_null() || out_token.is_null() || out_token_len.is_null() {
        return NetError::NullPointer.into();
    }
    let Some(subject_id) = entity_id_from_bytes(subject, subject_len) else {
        return NET_ERR_IDENTITY;
    };
    let Some(channel_s) = (unsafe { c_str_to_str(channel) }) else {
        return NetError::InvalidUtf8.into();
    };
    let Some(channel_hash) = channel_name_to_hash(channel_s) else {
        return NET_ERR_IDENTITY;
    };
    let h = unsafe { &*handle };
    match h.cache.get(&subject_id, channel_hash) {
        Some(token) => alloc_bytes(&token.to_bytes(), out_token, out_token_len),
        None => {
            unsafe {
                *out_token = std::ptr::null_mut();
                *out_token_len = 0;
            }
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_identity_token_cache_len(handle: *mut IdentityHandle) -> u32 {
    if handle.is_null() {
        return 0;
    }
    let h = unsafe { &*handle };
    h.cache.len() as u32
}

// -------------------------------------------------------------------------
// Module-level token helpers
// -------------------------------------------------------------------------

#[derive(Serialize)]
struct ParsedTokenJson {
    issuer_hex: String,
    subject_hex: String,
    scope: Vec<&'static str>,
    channel_hash: u16,
    not_before: u64,
    not_after: u64,
    delegation_depth: u8,
    nonce: u64,
    signature_hex: String,
}

/// Parse a serialized `PermissionToken` into a JSON dict. Fields are
/// hex-encoded on the wire (`issuer_hex`, `subject_hex`,
/// `signature_hex`) so the JSON round-trips cleanly. Binary variants
/// live on the `Identity` handle.
#[unsafe(no_mangle)]
pub extern "C" fn net_parse_token(
    token: *const u8,
    len: usize,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if token.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let slice = unsafe { std::slice::from_raw_parts(token, len) };
    let parsed = match PermissionToken::from_bytes(slice) {
        Ok(t) => t,
        Err(e) => return token_err_to_code(&e),
    };
    let out = ParsedTokenJson {
        issuer_hex: hex::encode(parsed.issuer.as_bytes()),
        subject_hex: hex::encode(parsed.subject.as_bytes()),
        scope: scope_to_strings(parsed.scope),
        channel_hash: parsed.channel_hash,
        not_before: parsed.not_before,
        not_after: parsed.not_after,
        delegation_depth: parsed.delegation_depth,
        nonce: parsed.nonce,
        signature_hex: hex::encode(parsed.signature),
    };
    write_json_out(&out, out_json, out_len)
}

/// Verify a serialized token's ed25519 signature. Writes `1` for
/// valid / `0` for tampered-or-wrong-subject. Time-bound validity is
/// a separate check — see `net_token_is_expired`.
#[unsafe(no_mangle)]
pub extern "C" fn net_verify_token(token: *const u8, len: usize, out_ok: *mut c_int) -> c_int {
    if token.is_null() || out_ok.is_null() {
        return NetError::NullPointer.into();
    }
    let slice = unsafe { std::slice::from_raw_parts(token, len) };
    let parsed = match PermissionToken::from_bytes(slice) {
        Ok(t) => t,
        Err(e) => return token_err_to_code(&e),
    };
    unsafe {
        *out_ok = if parsed.verify().is_ok() { 1 } else { 0 };
    }
    0
}

/// Writes `1` to `*out_expired` if the token's `not_after` has
/// passed; `0` otherwise. Pure time check — a tampered-but-expired
/// token still reports `1`. Use `net_verify_token` for signature
/// integrity.
#[unsafe(no_mangle)]
pub extern "C" fn net_token_is_expired(
    token: *const u8,
    len: usize,
    out_expired: *mut c_int,
) -> c_int {
    if token.is_null() || out_expired.is_null() {
        return NetError::NullPointer.into();
    }
    let slice = unsafe { std::slice::from_raw_parts(token, len) };
    let parsed = match PermissionToken::from_bytes(slice) {
        Ok(t) => t,
        Err(e) => return token_err_to_code(&e),
    };
    unsafe {
        *out_expired = match parsed.is_valid() {
            Err(CoreTokenError::Expired) => 1,
            _ => 0,
        };
    }
    0
}

/// Delegate a token to a new subject. Returns the child token blob;
/// caller frees via `net_free_bytes`.
#[unsafe(no_mangle)]
pub extern "C" fn net_delegate_token(
    signer: *mut IdentityHandle,
    parent: *const u8,
    parent_len: usize,
    new_subject: *const u8,
    new_subject_len: usize,
    restricted_scope_json: *const c_char,
    out_token: *mut *mut u8,
    out_token_len: *mut usize,
) -> c_int {
    if signer.is_null()
        || parent.is_null()
        || new_subject.is_null()
        || restricted_scope_json.is_null()
        || out_token.is_null()
        || out_token_len.is_null()
    {
        return NetError::NullPointer.into();
    }
    let parent_slice = unsafe { std::slice::from_raw_parts(parent, parent_len) };
    let parent_tok = match PermissionToken::from_bytes(parent_slice) {
        Ok(t) => t,
        Err(e) => return token_err_to_code(&e),
    };
    let Some(subject_id) = entity_id_from_bytes(new_subject, new_subject_len) else {
        return NET_ERR_IDENTITY;
    };
    let Some(scope_s) = (unsafe { c_str_to_str(restricted_scope_json) }) else {
        return NetError::InvalidUtf8.into();
    };
    let Some(scope) = parse_scope_list(scope_s) else {
        return NET_ERR_IDENTITY;
    };
    let h = unsafe { &*signer };
    match parent_tok.delegate(&h.keypair, subject_id, scope) {
        Ok(child) => alloc_bytes(&child.to_bytes(), out_token, out_token_len),
        Err(e) => token_err_to_code(&e),
    }
}

/// Hash a channel name to its 16-bit wire-format value. Returns
/// `NET_ERR_IDENTITY` for invalid names.
#[unsafe(no_mangle)]
pub extern "C" fn net_channel_hash(channel: *const c_char, out_hash: *mut u16) -> c_int {
    if channel.is_null() || out_hash.is_null() {
        return NetError::NullPointer.into();
    }
    let Some(s) = (unsafe { c_str_to_str(channel) }) else {
        return NetError::InvalidUtf8.into();
    };
    let Some(hash) = channel_name_to_hash(s) else {
        return NET_ERR_IDENTITY;
    };
    unsafe {
        *out_hash = hash;
    }
    0
}

// =========================================================================
// Capabilities (announce / find_peers)
// =========================================================================

// Local alias to keep the capability helpers out of the mesh module's
// import list when the Go surface doesn't need them.
use crate::adapter::net::behavior::capability::{
    AcceleratorInfo, AcceleratorType, CapabilityFilter, CapabilitySet, GpuInfo, GpuVendor,
    HardwareCapabilities, Modality, ModelCapability, ResourceLimits, SoftwareCapabilities,
    ToolCapability,
};

// ----- enum helpers (byte-for-byte mirrors of PyO3/NAPI) ---------------------

fn parse_gpu_vendor_cap(s: &str) -> GpuVendor {
    match s.to_ascii_lowercase().as_str() {
        "nvidia" => GpuVendor::Nvidia,
        "amd" => GpuVendor::Amd,
        "intel" => GpuVendor::Intel,
        "apple" => GpuVendor::Apple,
        "qualcomm" => GpuVendor::Qualcomm,
        _ => GpuVendor::Unknown,
    }
}

fn gpu_vendor_to_string_cap(v: GpuVendor) -> &'static str {
    match v {
        GpuVendor::Nvidia => "nvidia",
        GpuVendor::Amd => "amd",
        GpuVendor::Intel => "intel",
        GpuVendor::Apple => "apple",
        GpuVendor::Qualcomm => "qualcomm",
        GpuVendor::Unknown => "unknown",
    }
}

fn parse_modality_cap(s: &str) -> Modality {
    match s.to_ascii_lowercase().as_str() {
        "text" => Modality::Text,
        "image" => Modality::Image,
        "audio" => Modality::Audio,
        "video" => Modality::Video,
        "code" => Modality::Code,
        "embedding" => Modality::Embedding,
        "tool-use" | "tool_use" | "tooluse" => Modality::ToolUse,
        _ => Modality::Text,
    }
}

fn parse_accelerator_type_cap(s: &str) -> AcceleratorType {
    match s.to_ascii_lowercase().as_str() {
        "tpu" => AcceleratorType::Tpu,
        "npu" => AcceleratorType::Npu,
        "fpga" => AcceleratorType::Fpga,
        "asic" => AcceleratorType::Asic,
        "dsp" => AcceleratorType::Dsp,
        _ => AcceleratorType::Unknown,
    }
}

// ----- JSON shapes -----------------------------------------------------------

#[derive(Deserialize, Default)]
struct CapabilitySetJson {
    #[serde(default)]
    hardware: Option<HardwareJson>,
    #[serde(default)]
    software: Option<SoftwareJson>,
    #[serde(default)]
    models: Vec<ModelJson>,
    #[serde(default)]
    tools: Vec<ToolJson>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    limits: Option<LimitsJson>,
}

#[derive(Deserialize, Default)]
struct HardwareJson {
    cpu_cores: Option<u32>,
    cpu_threads: Option<u32>,
    memory_mb: Option<u32>,
    gpu: Option<GpuJson>,
    #[serde(default)]
    additional_gpus: Vec<GpuJson>,
    storage_mb: Option<u64>,
    network_mbps: Option<u32>,
    #[serde(default)]
    accelerators: Vec<AcceleratorJson>,
}

#[derive(Deserialize)]
struct GpuJson {
    vendor: Option<String>,
    #[serde(default)]
    model: String,
    #[serde(default)]
    vram_mb: u32,
    compute_units: Option<u32>,
    tensor_cores: Option<u32>,
    fp16_tflops_x10: Option<u32>,
}

#[derive(Deserialize)]
struct AcceleratorJson {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    model: String,
    memory_mb: Option<u32>,
    tops_x10: Option<u32>,
}

#[derive(Deserialize, Default)]
struct SoftwareJson {
    os: Option<String>,
    os_version: Option<String>,
    #[serde(default)]
    runtimes: Vec<Vec<String>>,
    #[serde(default)]
    frameworks: Vec<Vec<String>>,
    cuda_version: Option<String>,
    #[serde(default)]
    drivers: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct ModelJson {
    #[serde(default)]
    model_id: String,
    #[serde(default)]
    family: String,
    parameters_b_x10: Option<u32>,
    context_length: Option<u32>,
    quantization: Option<String>,
    #[serde(default)]
    modalities: Vec<String>,
    tokens_per_sec: Option<u32>,
    loaded: Option<bool>,
}

#[derive(Deserialize)]
struct ToolJson {
    #[serde(default)]
    tool_id: String,
    #[serde(default)]
    name: String,
    version: Option<String>,
    input_schema: Option<String>,
    output_schema: Option<String>,
    #[serde(default)]
    requires: Vec<String>,
    estimated_time_ms: Option<u32>,
    stateless: Option<bool>,
}

#[derive(Deserialize, Default)]
struct LimitsJson {
    max_concurrent_requests: Option<u32>,
    max_tokens_per_request: Option<u32>,
    rate_limit_rpm: Option<u32>,
    max_batch_size: Option<u32>,
    max_input_bytes: Option<u32>,
    max_output_bytes: Option<u32>,
}

#[derive(Deserialize, Default)]
struct CapabilityFilterJson {
    #[serde(default)]
    require_tags: Vec<String>,
    #[serde(default)]
    require_models: Vec<String>,
    #[serde(default)]
    require_tools: Vec<String>,
    min_memory_mb: Option<u32>,
    require_gpu: Option<bool>,
    gpu_vendor: Option<String>,
    min_vram_mb: Option<u32>,
    min_context_length: Option<u32>,
    #[serde(default)]
    require_modalities: Vec<String>,
}

// ----- Conversions -----------------------------------------------------------

fn pair_vec(xs: Vec<Vec<String>>) -> Vec<(String, String)> {
    xs.into_iter()
        .filter_map(|mut p| {
            if p.len() >= 2 {
                Some((std::mem::take(&mut p[0]), std::mem::take(&mut p[1])))
            } else {
                None
            }
        })
        .collect()
}

fn gpu_info_from_json(g: GpuJson) -> GpuInfo {
    let vendor = g
        .vendor
        .as_deref()
        .map(parse_gpu_vendor_cap)
        .unwrap_or(GpuVendor::Unknown);
    let mut info = GpuInfo::new(vendor, g.model, g.vram_mb);
    if let Some(cu) = g.compute_units {
        info = info.with_compute_units(cu as u16);
    }
    if let Some(tc) = g.tensor_cores {
        info = info.with_tensor_cores(tc as u16);
    }
    if let Some(tf) = g.fp16_tflops_x10 {
        info = info.with_fp16_tflops(tf as f32 / 10.0);
    }
    info
}

fn accelerator_from_json(a: AcceleratorJson) -> AcceleratorInfo {
    AcceleratorInfo {
        accel_type: parse_accelerator_type_cap(&a.kind),
        model: a.model,
        memory_mb: a.memory_mb.unwrap_or(0),
        tops_x10: a
            .tops_x10
            .map(|v| v.min(u16::MAX as u32) as u16)
            .unwrap_or(0),
    }
}

fn hardware_from_json(h: HardwareJson) -> HardwareCapabilities {
    let mut hw = HardwareCapabilities::new();
    match (h.cpu_cores, h.cpu_threads) {
        (Some(c), Some(t)) => hw = hw.with_cpu(c as u16, t as u16),
        (Some(c), None) => hw = hw.with_cpu(c as u16, c as u16),
        _ => {}
    }
    if let Some(mb) = h.memory_mb {
        hw = hw.with_memory(mb);
    }
    if let Some(g) = h.gpu {
        hw = hw.with_gpu(gpu_info_from_json(g));
    }
    for g in h.additional_gpus {
        hw = hw.add_gpu(gpu_info_from_json(g));
    }
    if let Some(mb) = h.storage_mb {
        hw = hw.with_storage(mb);
    }
    if let Some(mbps) = h.network_mbps {
        hw = hw.with_network(mbps);
    }
    for a in h.accelerators {
        hw = hw.add_accelerator(accelerator_from_json(a));
    }
    hw
}

fn software_from_json(s: SoftwareJson) -> SoftwareCapabilities {
    let mut sw = SoftwareCapabilities::new()
        .with_os(s.os.unwrap_or_default(), s.os_version.unwrap_or_default());
    for (k, v) in pair_vec(s.runtimes) {
        sw = sw.add_runtime(k, v);
    }
    for (k, v) in pair_vec(s.frameworks) {
        sw = sw.add_framework(k, v);
    }
    if let Some(c) = s.cuda_version {
        sw = sw.with_cuda(c);
    }
    sw.drivers = pair_vec(s.drivers);
    sw
}

fn model_from_json(m: ModelJson) -> ModelCapability {
    let mut mc = ModelCapability::new(m.model_id, m.family);
    if let Some(p) = m.parameters_b_x10 {
        mc.parameters_b_x10 = p;
    }
    if let Some(c) = m.context_length {
        mc = mc.with_context_length(c);
    }
    if let Some(q) = m.quantization {
        mc = mc.with_quantization(q);
    }
    for modality in m.modalities {
        mc = mc.add_modality(parse_modality_cap(&modality));
    }
    if let Some(t) = m.tokens_per_sec {
        mc = mc.with_tokens_per_sec(t);
    }
    if let Some(l) = m.loaded {
        mc = mc.with_loaded(l);
    }
    mc
}

fn tool_from_json(t: ToolJson) -> ToolCapability {
    let mut tc = ToolCapability::new(t.tool_id, t.name);
    if let Some(v) = t.version {
        tc = tc.with_version(v);
    }
    if let Some(s) = t.input_schema {
        tc = tc.with_input_schema(s);
    }
    if let Some(s) = t.output_schema {
        tc = tc.with_output_schema(s);
    }
    for r in t.requires {
        tc = tc.requires(r);
    }
    if let Some(ms) = t.estimated_time_ms {
        tc = tc.with_estimated_time(ms);
    }
    if let Some(st) = t.stateless {
        tc = tc.with_stateless(st);
    }
    tc
}

fn limits_from_json(l: LimitsJson) -> ResourceLimits {
    let mut rl = ResourceLimits::new();
    if let Some(n) = l.max_concurrent_requests {
        rl = rl.with_max_concurrent(n);
    }
    if let Some(n) = l.max_tokens_per_request {
        rl = rl.with_max_tokens(n);
    }
    if let Some(n) = l.rate_limit_rpm {
        rl = rl.with_rate_limit(n);
    }
    if let Some(n) = l.max_batch_size {
        rl = rl.with_max_batch(n);
    }
    if let Some(n) = l.max_input_bytes {
        rl.max_input_bytes = n;
    }
    if let Some(n) = l.max_output_bytes {
        rl.max_output_bytes = n;
    }
    rl
}

fn capability_set_from_json(caps: CapabilitySetJson) -> CapabilitySet {
    let mut cs = CapabilitySet::new();
    if let Some(h) = caps.hardware {
        cs = cs.with_hardware(hardware_from_json(h));
    }
    if let Some(s) = caps.software {
        cs = cs.with_software(software_from_json(s));
    }
    for m in caps.models {
        cs = cs.add_model(model_from_json(m));
    }
    for t in caps.tools {
        cs = cs.add_tool(tool_from_json(t));
    }
    for tag in caps.tags {
        cs = cs.add_tag(tag);
    }
    if let Some(l) = caps.limits {
        cs = cs.with_limits(limits_from_json(l));
    }
    cs
}

fn capability_filter_from_json(f: CapabilityFilterJson) -> CapabilityFilter {
    let mut cf = CapabilityFilter::new();
    for t in f.require_tags {
        cf = cf.require_tag(t);
    }
    for m in f.require_models {
        cf = cf.require_model(m);
    }
    for t in f.require_tools {
        cf = cf.require_tool(t);
    }
    if let Some(mb) = f.min_memory_mb {
        cf = cf.with_min_memory(mb);
    }
    if f.require_gpu.unwrap_or(false) {
        cf = cf.require_gpu();
    }
    if let Some(v) = f.gpu_vendor {
        cf = cf.with_gpu_vendor(parse_gpu_vendor_cap(&v));
    }
    if let Some(mb) = f.min_vram_mb {
        cf = cf.with_min_vram(mb);
    }
    if let Some(n) = f.min_context_length {
        cf = cf.with_min_context(n);
    }
    for m in f.require_modalities {
        cf = cf.require_modality(parse_modality_cap(&m));
    }
    cf
}

// ----- Exports ---------------------------------------------------------------

pub(crate) const NET_ERR_CAPABILITY: c_int = -128;

/// Announce this node's capabilities to every directly-connected
/// peer. Also self-indexes, so `find_peers` on the same node matches
/// on the announcement. Multi-hop propagation is deferred.
///
/// `caps_json` is the same POJO shape as PyO3 / NAPI:
/// `{hardware, software, models, tools, tags, limits}`.
#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_announce_capabilities(
    handle: *mut MeshNodeHandle,
    caps_json: *const c_char,
) -> c_int {
    if handle.is_null() || caps_json.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let Some(s) = (unsafe { c_str_to_str(caps_json) }) else {
        return NetError::InvalidUtf8.into();
    };
    let parsed: CapabilitySetJson = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return NetError::InvalidJson.into(),
    };
    let caps = capability_set_from_json(parsed);
    let rt = runtime();
    let node = h.inner.clone();
    match rt.block_on(async move { node.announce_capabilities(caps).await }) {
        Ok(()) => 0,
        Err(_) => NET_ERR_CAPABILITY,
    }
}

/// Query the local capability index. Writes a JSON array of node
/// ids (u64) to `*out_json`; caller frees via `net_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn net_mesh_find_peers(
    handle: *mut MeshNodeHandle,
    filter_json: *const c_char,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if handle.is_null() || filter_json.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let h = unsafe { &*handle };
    let Some(s) = (unsafe { c_str_to_str(filter_json) }) else {
        return NetError::InvalidUtf8.into();
    };
    let parsed: CapabilityFilterJson = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return NetError::InvalidJson.into(),
    };
    let filter = capability_filter_from_json(parsed);
    let ids = h.inner.find_peers_by_filter(&filter);
    write_json_out(&ids, out_json, out_len)
}

/// Normalize a GPU vendor string to its canonical lowercase form.
#[unsafe(no_mangle)]
pub extern "C" fn net_normalize_gpu_vendor(
    raw: *const c_char,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if raw.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let Some(s) = (unsafe { c_str_to_str(raw) }) else {
        return NetError::InvalidUtf8.into();
    };
    let canonical = gpu_vendor_to_string_cap(parse_gpu_vendor_cap(s));
    write_string_out(canonical.to_string(), out_json, out_len)
}
