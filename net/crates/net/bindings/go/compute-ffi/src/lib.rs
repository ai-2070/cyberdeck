//! C ABI for the compute (MeshDaemon + migration) surface — Stage 6 of
//! `SDK_COMPUTE_SURFACE_PLAN.md`. Consumed by the Go binding at
//! `bindings/go/net/compute.go`.
//!
//! **Sub-step 1** (this file): lifecycle skeleton. A Go caller can
//! build a `DaemonRuntime` bound to an existing `MeshNodeHandle`
//! (from `net::ffi::mesh`), transition it to ready, register a
//! placeholder kind, and shut it down. Event delivery, spawn,
//! migration, and snapshot/restore land in sub-steps 2-4.
//!
//! # Handle model
//!
//! Every Rust object that crosses the FFI boundary is wrapped in a
//! heap-allocated box and handed to the caller as `*mut T`. Go owns
//! the pointer (runtime-finalizer pattern in `compute.go`) and MUST
//! call the matching `_free` function exactly once.
//!
//! # Error codes
//!
//! `c_int` return values (0 = success, < 0 = error) follow the
//! convention established by `net::ffi::mesh` +
//! `net::ffi::cortex`. Structured error information is surfaced via
//! an out-param `*mut *mut c_char` that the caller frees with
//! [`net_compute_free_cstring`].
//!
//! # Tokio runtime
//!
//! This crate owns a lazy `OnceLock<Arc<Runtime>>` for the async
//! SDK calls, mirroring the pattern used by `net::ffi::mesh`. The
//! mesh's internal operations run on their own global runtime; the
//! compute runtime needs its own tokio context for `block_on`
//! because we cross the FFI boundary inside it.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, OnceLock};

use bytes::Bytes;
use dashmap::DashMap;
use net::adapter::net::behavior::capability::CapabilityFilter;
use net::adapter::net::channel::ChannelConfigRegistry;
use net::adapter::net::compute::{DaemonError as CoreDaemonError, DaemonHostConfig, MeshDaemon};
use net::adapter::net::state::causal::CausalEvent;
use net::adapter::net::MeshNode;
use net_sdk::compute::{DaemonHandle as SdkDaemonHandle, DaemonRuntime as SdkDaemonRuntime};
use net_sdk::mesh::Mesh as SdkMesh;
use net_sdk::Identity as SdkIdentity;
use tokio::runtime::Runtime;

// =========================================================================
// Error codes
// =========================================================================

/// Operation succeeded.
pub const NET_COMPUTE_OK: c_int = 0;
/// Null or invalid pointer passed where a live handle was expected.
pub const NET_COMPUTE_ERR_NULL: c_int = -1;
/// Generic catch-all for errors whose detail is returned via the
/// out-param `*mut *mut c_char` on the same call.
pub const NET_COMPUTE_ERR_CALL_FAILED: c_int = -2;
/// Kind already registered on `net_compute_register_factory`.
pub const NET_COMPUTE_ERR_DUPLICATE_KIND: c_int = -3;

// =========================================================================
// Shared tokio runtime
// =========================================================================

/// Lazy global runtime. Matches the pattern in `net::ffi::mesh` /
/// `net::ffi::cortex`. Panics on initial construction failure —
/// the process is unusable without a runtime, so failing fast is
/// preferable to silently returning errors on every subsequent
/// call.
fn runtime() -> &'static Arc<Runtime> {
    static RT: OnceLock<Arc<Runtime>> = OnceLock::new();
    RT.get_or_init(|| {
        Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("net-compute-ffi")
                .build()
                .expect("failed to construct compute-ffi tokio runtime"),
        )
    })
}

// =========================================================================
// Handle types
// =========================================================================

/// Opaque pointer exposed to Go. Wraps the SDK's `DaemonRuntime`
/// plus the shared `factories` set the compute-ffi crate uses to
/// dedupe `register_factory` calls on this side of the FFI.
pub struct DaemonRuntimeHandle {
    inner: Arc<SdkDaemonRuntime>,
    /// Tracks registered kinds (same role as the `factories`
    /// DashMap on NAPI / PyO3). Sub-step 2 will swap the value
    /// type to a Go-callback trampoline table.
    factories: Arc<DashMap<String, ()>>,
}

// =========================================================================
// Free helpers — misc out-of-band allocations
// =========================================================================

/// Free a CString previously returned out-of-band by this crate
/// (e.g., structured error detail). Idempotent on NULL.
#[no_mangle]
pub extern "C" fn net_compute_free_cstring(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}

// =========================================================================
// DaemonRuntime lifecycle
// =========================================================================

/// Build a new `DaemonRuntime` from Arc-cloned handles to the
/// source `MeshNode` and its shared `ChannelConfigRegistry`. The
/// pointers MUST come from `net_mesh_arc_clone` /
/// `net_mesh_channel_configs_arc_clone` (defined in
/// `net::ffi::mesh`).
///
/// Ownership semantics:
/// - `node_arc` and `channel_configs_arc` are CONSUMED by this
///   call — the compute runtime takes their `Arc` content via
///   `Box::from_raw` and re-boxes it into its own fields. Callers
///   MUST NOT free them after a successful call.
/// - On failure (either input NULL), the pointers are left intact
///   so the caller's deferred `_free` stays correct.
///
/// Returns a boxed `DaemonRuntimeHandle` — caller owns it and
/// frees with [`net_compute_runtime_free`]. Returns NULL if any
/// input is NULL.
#[no_mangle]
pub extern "C" fn net_compute_runtime_new(
    node_arc: *mut Arc<MeshNode>,
    channel_configs_arc: *mut Arc<ChannelConfigRegistry>,
) -> *mut DaemonRuntimeHandle {
    if node_arc.is_null() || channel_configs_arc.is_null() {
        return std::ptr::null_mut();
    }
    // Take ownership of the boxed Arcs — the caller's LDFLAGS
    // expectation (paired `_free`) is handled by the fact that we
    // document consumption in the docstring. Go callers release
    // these pointers from their finalizer list after the call
    // succeeds.
    let node = unsafe { *Box::from_raw(node_arc) };
    let cc = unsafe { *Box::from_raw(channel_configs_arc) };

    let sdk_mesh = SdkMesh::from_node_arc(node, cc, None);
    let sdk_rt = SdkDaemonRuntime::new(Arc::new(sdk_mesh));

    Box::into_raw(Box::new(DaemonRuntimeHandle {
        inner: Arc::new(sdk_rt),
        factories: Arc::new(DashMap::new()),
    }))
}

/// Free a runtime handle. The underlying `MeshNode` stays alive
/// so long as the caller holds another `Arc` to it (typically via
/// its own `MeshNodeHandle`). Idempotent on NULL.
#[no_mangle]
pub extern "C" fn net_compute_runtime_free(handle: *mut DaemonRuntimeHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(handle);
    }
}

/// Transition the runtime to `Ready`. Installs the migration
/// subprotocol handler on the underlying mesh. Idempotent on an
/// already-ready runtime.
///
/// On failure, writes a heap-allocated `char*` error detail to
/// `*err_out` (caller frees with [`net_compute_free_cstring`]).
/// Returns [`NET_COMPUTE_OK`] / [`NET_COMPUTE_ERR_*`].
#[no_mangle]
pub extern "C" fn net_compute_runtime_start(
    handle: *mut DaemonRuntimeHandle,
    err_out: *mut *mut c_char,
) -> c_int {
    let Some(h) = (unsafe { handle.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL;
    };
    let inner = h.inner.clone();
    let rt = runtime();
    let res = rt.block_on(async move { inner.start().await });
    match res {
        Ok(()) => NET_COMPUTE_OK,
        Err(e) => {
            write_err(err_out, &e.to_string());
            NET_COMPUTE_ERR_CALL_FAILED
        }
    }
}

/// Tear down the runtime. Drains daemons + factory registrations +
/// uninstalls the migration handler. The underlying `MeshNode` is
/// untouched.
#[no_mangle]
pub extern "C" fn net_compute_runtime_shutdown(
    handle: *mut DaemonRuntimeHandle,
    err_out: *mut *mut c_char,
) -> c_int {
    let Some(h) = (unsafe { handle.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL;
    };
    let inner = h.inner.clone();
    let factories = h.factories.clone();
    let rt = runtime();
    let res = rt.block_on(async move { inner.shutdown().await });
    match res {
        Ok(()) => {
            factories.clear();
            NET_COMPUTE_OK
        }
        Err(e) => {
            write_err(err_out, &e.to_string());
            NET_COMPUTE_ERR_CALL_FAILED
        }
    }
}

/// Return `1` if the runtime has transitioned to `Ready` and not
/// yet begun shutting down, else `0`. Returns [`NET_COMPUTE_ERR_NULL`]
/// on a NULL handle.
#[no_mangle]
pub extern "C" fn net_compute_runtime_is_ready(handle: *mut DaemonRuntimeHandle) -> c_int {
    let Some(h) = (unsafe { handle.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL;
    };
    if h.inner.is_ready() {
        1
    } else {
        0
    }
}

/// Return the number of daemons currently registered on this
/// runtime. Returns `-1` on NULL handle (cast of
/// [`NET_COMPUTE_ERR_NULL`]).
#[no_mangle]
pub extern "C" fn net_compute_runtime_daemon_count(handle: *mut DaemonRuntimeHandle) -> i64 {
    let Some(h) = (unsafe { handle.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL as i64;
    };
    h.inner.daemon_count() as i64
}

/// Register a placeholder factory entry under `kind`. Sub-step 1
/// only stores the kind string in the runtime's registered set;
/// sub-step 2 will wire the Go callback table so the SDK's
/// factory map can construct daemons on demand.
///
/// Returns [`NET_COMPUTE_OK`] on success,
/// [`NET_COMPUTE_ERR_DUPLICATE_KIND`] if the kind was already
/// registered, or [`NET_COMPUTE_ERR_NULL`] for invalid arguments.
///
/// # Safety
///
/// `kind_ptr` must point to `kind_len` bytes of valid UTF-8. NUL
/// termination is NOT required — `kind_len` is the exact byte
/// count.
#[no_mangle]
pub extern "C" fn net_compute_register_factory(
    handle: *mut DaemonRuntimeHandle,
    kind_ptr: *const c_char,
    kind_len: usize,
) -> c_int {
    let Some(h) = (unsafe { handle.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL;
    };
    if kind_ptr.is_null() {
        return NET_COMPUTE_ERR_NULL;
    }
    let kind = match cstr_to_string(kind_ptr, kind_len) {
        Some(s) => s,
        None => return NET_COMPUTE_ERR_NULL,
    };
    use dashmap::mapref::entry::Entry;
    match h.factories.entry(kind) {
        Entry::Occupied(_) => NET_COMPUTE_ERR_DUPLICATE_KIND,
        Entry::Vacant(slot) => {
            slot.insert(());
            NET_COMPUTE_OK
        }
    }
}

// =========================================================================
// Callback dispatcher — Go side registers these once
// =========================================================================

/// C-ABI type: invoke Go's `Process` for the daemon identified by
/// `daemon_id`. Outputs push into the `OutputsVec` handed in;
/// return code `0` on success, non-zero if the Go side failed.
pub type ProcessFn = unsafe extern "C" fn(
    daemon_id: u64,
    origin_hash: u32,
    sequence: u64,
    payload_ptr: *const u8,
    payload_len: usize,
    outputs: *mut OutputsVec,
) -> c_int;

/// C-ABI type: invoke Go's `Snapshot` for `daemon_id`. On success,
/// writes either `(NULL, 0)` (stateless) or a heap-allocated
/// `(ptr, len)` pair that the Rust side frees with
/// [`net_compute_snapshot_bytes_free`]. Return `0` on success;
/// non-zero if Snapshot threw.
pub type SnapshotFn =
    unsafe extern "C" fn(daemon_id: u64, out_ptr: *mut *mut u8, out_len: *mut usize) -> c_int;

/// C-ABI type: invoke Go's `Restore` for `daemon_id`. Returns `0`
/// on success, non-zero if Restore threw.
pub type RestoreFn =
    unsafe extern "C" fn(daemon_id: u64, state_ptr: *const u8, state_len: usize) -> c_int;

/// C-ABI type: Go releases its registry entry for `daemon_id`.
/// Called when the Rust side drops the last reference to the
/// bridge (e.g., daemon stopped, migration source cutover).
pub type FreeFn = unsafe extern "C" fn(daemon_id: u64);

/// The four trampolines registered once from Go's `init()`. Stored
/// in a `OnceLock` so invocation is lock-free.
struct DispatcherFns {
    process: ProcessFn,
    snapshot: SnapshotFn,
    restore: RestoreFn,
    free: FreeFn,
}

static DISPATCHER: OnceLock<DispatcherFns> = OnceLock::new();

/// Register the Go-side dispatcher trampolines. MUST be called
/// exactly once before any `net_compute_spawn` or
/// `net_compute_deliver`. A second call is ignored (the first
/// registration wins — `OnceLock` semantics).
///
/// # Safety
///
/// The four function pointers MUST have C linkage, must be valid
/// for the remaining lifetime of the process, and must follow the
/// contracts of [`ProcessFn`] / [`SnapshotFn`] / [`RestoreFn`] /
/// [`FreeFn`]. Passing NULL is a hard error.
#[no_mangle]
pub extern "C" fn net_compute_set_dispatcher(
    process: Option<ProcessFn>,
    snapshot: Option<SnapshotFn>,
    restore: Option<RestoreFn>,
    free: Option<FreeFn>,
) -> c_int {
    let Some(process) = process else {
        return NET_COMPUTE_ERR_NULL;
    };
    let Some(snapshot) = snapshot else {
        return NET_COMPUTE_ERR_NULL;
    };
    let Some(restore) = restore else {
        return NET_COMPUTE_ERR_NULL;
    };
    let Some(free) = free else {
        return NET_COMPUTE_ERR_NULL;
    };
    let _ = DISPATCHER.set(DispatcherFns {
        process,
        snapshot,
        restore,
        free,
    });
    NET_COMPUTE_OK
}

// =========================================================================
// OutputsVec — growable container for `Vec<Bytes>` populated by Go
// =========================================================================

/// Opaque buffer the Rust bridge hands to Go during a `process`
/// callback. Go calls [`net_compute_outputs_push`] once per output
/// payload.
///
/// Exposed as `*mut OutputsVec` to C for pass-through only — fields
/// stay private so the Rust side controls the memory lifecycle.
#[repr(C)]
pub struct OutputsVec {
    inner: Vec<Bytes>,
}

/// Push a single output payload into the vec. Copies `len` bytes
/// from `data` into a freshly-allocated `Bytes`. Returns
/// [`NET_COMPUTE_OK`] on success or [`NET_COMPUTE_ERR_NULL`] on bad
/// input.
///
/// # Safety
///
/// `vec` must point to an `OutputsVec` owned by the Rust bridge
/// (lifetime bound to the caller's `net_compute_deliver` /
/// `process` callback frame). `data` must be a valid read-only
/// pointer to at least `len` bytes.
#[no_mangle]
pub extern "C" fn net_compute_outputs_push(
    vec: *mut OutputsVec,
    data: *const u8,
    len: usize,
) -> c_int {
    if vec.is_null() {
        return NET_COMPUTE_ERR_NULL;
    }
    if len > 0 && data.is_null() {
        return NET_COMPUTE_ERR_NULL;
    }
    let v = unsafe { &mut *vec };
    let slice = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, len) }
    };
    v.inner.push(Bytes::copy_from_slice(slice));
    NET_COMPUTE_OK
}

/// Free a `(ptr, len)` snapshot payload heap-allocated by Go
/// inside [`SnapshotFn`]. Called by the Rust bridge once the
/// `Bytes` copy is taken.
///
/// # Safety
///
/// Pairs with Go-side `C.malloc`-equivalent allocations. Go
/// snapshot trampolines must allocate via `C.malloc` and hand us
/// the pointer; we free via `libc::free`. NULL is a no-op.
#[no_mangle]
pub extern "C" fn net_compute_snapshot_bytes_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    // The Go side allocated via `C.malloc`; `libc::free` matches.
    unsafe {
        libc::free(ptr as *mut std::ffi::c_void);
    }
}

// =========================================================================
// GoBridge — `MeshDaemon` impl driven by the registered dispatcher
// =========================================================================

/// Daemon bridge holding a `daemon_id` (the Go-side registry key)
/// and the kind name for debug. `MeshDaemon` impls invoke the
/// registered dispatcher trampolines with this ID.
///
/// Drop triggers [`FreeFn`] so the Go side can release its entry.
struct GoBridge {
    name: String,
    daemon_id: u64,
}

impl Drop for GoBridge {
    fn drop(&mut self) {
        if let Some(d) = DISPATCHER.get() {
            unsafe { (d.free)(self.daemon_id) };
        }
    }
}

impl MeshDaemon for GoBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn requirements(&self) -> CapabilityFilter {
        CapabilityFilter::default()
    }

    fn process(
        &mut self,
        event: &CausalEvent,
    ) -> std::result::Result<Vec<Bytes>, CoreDaemonError> {
        let Some(d) = DISPATCHER.get() else {
            return Err(CoreDaemonError::ProcessFailed(
                "Go dispatcher not registered — call net_compute_set_dispatcher at init"
                    .to_string(),
            ));
        };
        let mut outputs = OutputsVec { inner: Vec::new() };
        let code = unsafe {
            (d.process)(
                self.daemon_id,
                event.link.origin_hash,
                event.link.sequence,
                event.payload.as_ptr(),
                event.payload.len(),
                &mut outputs,
            )
        };
        if code != NET_COMPUTE_OK {
            return Err(CoreDaemonError::ProcessFailed(format!(
                "Go process callback returned {code}"
            )));
        }
        Ok(outputs.inner)
    }

    fn snapshot(&self) -> Option<Bytes> {
        let d = DISPATCHER.get()?;
        let mut ptr: *mut u8 = std::ptr::null_mut();
        let mut len: usize = 0;
        let code = unsafe { (d.snapshot)(self.daemon_id, &mut ptr, &mut len) };
        if code != NET_COMPUTE_OK {
            eprintln!(
                "GoBridge::snapshot: dispatcher returned {code}; treating as None"
            );
            return None;
        }
        if ptr.is_null() || len == 0 {
            return None;
        }
        // Copy the Go-allocated buffer into a Rust `Bytes` and
        // free the original so Go's malloc pool stays tidy.
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        let out = Bytes::copy_from_slice(slice);
        unsafe { libc::free(ptr as *mut std::ffi::c_void) };
        Some(out)
    }

    fn restore(
        &mut self,
        state: Bytes,
    ) -> std::result::Result<(), CoreDaemonError> {
        let Some(d) = DISPATCHER.get() else {
            return Err(CoreDaemonError::RestoreFailed(
                "Go dispatcher not registered".to_string(),
            ));
        };
        let code = unsafe { (d.restore)(self.daemon_id, state.as_ptr(), state.len()) };
        if code != NET_COMPUTE_OK {
            return Err(CoreDaemonError::RestoreFailed(format!(
                "Go restore callback returned {code}"
            )));
        }
        Ok(())
    }
}

// =========================================================================
// DaemonHandle — thin C-ABI wrapper around the SDK handle
// =========================================================================

/// Opaque handle returned by `net_compute_spawn`. Holds the SDK
/// handle plus the `origin_hash` / `entity_id` accessors the Go
/// side surfaces as methods.
pub struct DaemonHandleC {
    origin_hash: u32,
    entity_id: [u8; 32],
    #[allow(dead_code)]
    inner: SdkDaemonHandle,
}

/// Read the daemon's `origin_hash`. Returns 0 on NULL handle
/// (callers can't hit this normally — a spawned daemon always
/// has a non-zero origin_hash).
#[no_mangle]
pub extern "C" fn net_compute_daemon_handle_origin_hash(h: *const DaemonHandleC) -> u32 {
    let Some(h) = (unsafe { h.as_ref() }) else {
        return 0;
    };
    h.origin_hash
}

/// Copy the 32-byte `entity_id` into `out[0..32]`. Returns
/// [`NET_COMPUTE_OK`] / [`NET_COMPUTE_ERR_NULL`].
#[no_mangle]
pub extern "C" fn net_compute_daemon_handle_entity_id(
    h: *const DaemonHandleC,
    out: *mut u8,
) -> c_int {
    let Some(h) = (unsafe { h.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL;
    };
    if out.is_null() {
        return NET_COMPUTE_ERR_NULL;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(h.entity_id.as_ptr(), out, 32);
    }
    NET_COMPUTE_OK
}

/// Free the handle. The daemon itself keeps running — call
/// [`net_compute_runtime_stop`] first to tear it down.
#[no_mangle]
pub extern "C" fn net_compute_daemon_handle_free(h: *mut DaemonHandleC) {
    if h.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(h);
    }
}

// =========================================================================
// Spawn / stop / deliver
// =========================================================================

/// Spawn a Go daemon. `daemon_id` is the Go-side registry key that
/// the dispatcher trampolines will use to look up the daemon
/// instance on every `process` / `snapshot` / `restore` callback.
///
/// `identity_seed` must point to 32 bytes of ed25519 seed.
/// `kind_ptr` + `kind_len`: UTF-8 name the caller registered via
/// [`net_compute_register_factory`] (sub-step 2 doesn't actually
/// require prior registration; the lookup happens on the SDK side
/// via `spawn_with_daemon`, which takes the bridge directly).
///
/// On success, writes the `DaemonHandleC*` to `*out_handle` and
/// returns [`NET_COMPUTE_OK`]. On failure, leaves `*out_handle =
/// NULL` and populates `*err_out` with a structured detail.
///
/// # Safety
///
/// See field docs. `daemon_id` must match a live Go-side entry;
/// we don't validate but the dispatcher trampolines will treat an
/// unknown ID as a daemon-side error.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn net_compute_spawn(
    handle: *mut DaemonRuntimeHandle,
    kind_ptr: *const c_char,
    kind_len: usize,
    identity_seed: *const u8,
    daemon_id: u64,
    auto_snapshot_interval: u64,
    max_log_entries: u32,
    out_handle: *mut *mut DaemonHandleC,
    err_out: *mut *mut c_char,
) -> c_int {
    let Some(h) = (unsafe { handle.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL;
    };
    if kind_ptr.is_null() || identity_seed.is_null() || out_handle.is_null() {
        return NET_COMPUTE_ERR_NULL;
    }
    let Some(kind) = cstr_to_string(kind_ptr, kind_len) else {
        return NET_COMPUTE_ERR_NULL;
    };
    // Copy the 32-byte seed — the SDK `Identity::from_seed` takes
    // `[u8; 32]` by value and we don't want to hold onto Go's
    // backing memory past the call.
    let mut seed = [0u8; 32];
    unsafe {
        std::ptr::copy_nonoverlapping(identity_seed, seed.as_mut_ptr(), 32);
    }
    let sdk_identity = SdkIdentity::from_seed(seed);

    let mut cfg = DaemonHostConfig::default();
    cfg.auto_snapshot_interval = auto_snapshot_interval;
    if max_log_entries > 0 {
        cfg.max_log_entries = max_log_entries;
    }

    let bridge = Box::new(GoBridge {
        name: kind.clone(),
        daemon_id,
    });
    // kind_factory for migration-target reconstruction — returns
    // a NoopBridge fallback because we don't currently call back
    // into Go's factory on the target side. Sub-step 4 addresses
    // this for migration-capable Go targets.
    let kind_for_noop = kind.clone();
    let kind_factory = move || -> Box<dyn MeshDaemon> {
        Box::new(NoopBridge::new(kind_for_noop.clone()))
    };

    let inner = h.inner.clone();
    let rt = runtime();
    let result = rt.block_on(async move {
        inner
            .spawn_with_daemon(sdk_identity, cfg, bridge, kind_factory)
            .await
    });
    match result {
        Ok(sdk_handle) => {
            let origin_hash = sdk_handle.origin_hash;
            let entity_id = *sdk_handle.entity_id.as_bytes();
            let boxed = Box::new(DaemonHandleC {
                origin_hash,
                entity_id,
                inner: sdk_handle,
            });
            unsafe {
                *out_handle = Box::into_raw(boxed);
            }
            NET_COMPUTE_OK
        }
        Err(e) => {
            unsafe {
                *out_handle = std::ptr::null_mut();
            }
            write_err(err_out, &e.to_string());
            NET_COMPUTE_ERR_CALL_FAILED
        }
    }
}

/// Stop a daemon by `origin_hash`. Idempotent during shutdown.
/// Returns [`NET_COMPUTE_OK`] / [`NET_COMPUTE_ERR_*`].
#[no_mangle]
pub extern "C" fn net_compute_runtime_stop(
    handle: *mut DaemonRuntimeHandle,
    origin_hash: u32,
    err_out: *mut *mut c_char,
) -> c_int {
    let Some(h) = (unsafe { handle.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL;
    };
    let inner = h.inner.clone();
    let rt = runtime();
    match rt.block_on(async move { inner.stop(origin_hash).await }) {
        Ok(()) => NET_COMPUTE_OK,
        Err(e) => {
            write_err(err_out, &e.to_string());
            NET_COMPUTE_ERR_CALL_FAILED
        }
    }
}

/// Deliver one event to the daemon at `origin_hash`. The Go
/// dispatcher's `process` callback fires with the same event
/// fields; the daemon's outputs push into a fresh `OutputsVec`
/// and the caller receives them via `outputs` (must be non-NULL).
///
/// The caller is responsible for reading the outputs before the
/// next deliver — [`net_compute_outputs_take`] drains them.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn net_compute_runtime_deliver(
    handle: *mut DaemonRuntimeHandle,
    origin_hash: u32,
    event_origin_hash: u32,
    event_sequence: u64,
    event_payload: *const u8,
    event_payload_len: usize,
    out_outputs: *mut *mut OutputsVec,
    err_out: *mut *mut c_char,
) -> c_int {
    let Some(h) = (unsafe { handle.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL;
    };
    if out_outputs.is_null() {
        return NET_COMPUTE_ERR_NULL;
    }
    if event_payload_len > 0 && event_payload.is_null() {
        return NET_COMPUTE_ERR_NULL;
    }
    let payload = if event_payload_len == 0 {
        Bytes::new()
    } else {
        let slice =
            unsafe { std::slice::from_raw_parts(event_payload, event_payload_len) };
        Bytes::copy_from_slice(slice)
    };
    let event = CausalEvent {
        link: net::adapter::net::state::causal::CausalLink {
            origin_hash: event_origin_hash,
            horizon_encoded: 0,
            sequence: event_sequence,
            parent_hash: 0,
        },
        payload,
        received_at: 0,
    };

    let inner = h.inner.clone();
    let rt = runtime();
    let result = rt.block_on(async move { inner.deliver(origin_hash, &event) });
    match result {
        Ok(outputs) => {
            let vec = OutputsVec {
                inner: outputs
                    .into_iter()
                    .map(|ev| ev.payload.clone())
                    .collect(),
            };
            unsafe {
                *out_outputs = Box::into_raw(Box::new(vec));
            }
            NET_COMPUTE_OK
        }
        Err(e) => {
            unsafe {
                *out_outputs = std::ptr::null_mut();
            }
            write_err(err_out, &e.to_string());
            NET_COMPUTE_ERR_CALL_FAILED
        }
    }
}

/// Return the number of outputs stored in `vec`. Returns `0` on
/// NULL.
#[no_mangle]
pub extern "C" fn net_compute_outputs_len(vec: *const OutputsVec) -> usize {
    let Some(v) = (unsafe { vec.as_ref() }) else {
        return 0;
    };
    v.inner.len()
}

/// Copy the `idx`-th output payload's `(ptr, len)` into
/// `*out_ptr` / `*out_len`. The pointer is borrowed from the vec —
/// caller must copy before the next [`net_compute_outputs_free`].
#[no_mangle]
pub extern "C" fn net_compute_outputs_at(
    vec: *const OutputsVec,
    idx: usize,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> c_int {
    let Some(v) = (unsafe { vec.as_ref() }) else {
        return NET_COMPUTE_ERR_NULL;
    };
    if out_ptr.is_null() || out_len.is_null() {
        return NET_COMPUTE_ERR_NULL;
    }
    let Some(b) = v.inner.get(idx) else {
        return NET_COMPUTE_ERR_NULL;
    };
    unsafe {
        *out_ptr = b.as_ptr();
        *out_len = b.len();
    }
    NET_COMPUTE_OK
}

/// Free the outputs vec produced by
/// [`net_compute_runtime_deliver`]. Idempotent on NULL.
#[no_mangle]
pub extern "C" fn net_compute_outputs_free(vec: *mut OutputsVec) {
    if vec.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(vec);
    }
}

// =========================================================================
// NoopBridge — migration-target fallback
// =========================================================================

struct NoopBridge {
    name: String,
}

impl NoopBridge {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl MeshDaemon for NoopBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn requirements(&self) -> CapabilityFilter {
        CapabilityFilter::default()
    }

    fn process(
        &mut self,
        _event: &CausalEvent,
    ) -> std::result::Result<Vec<Bytes>, CoreDaemonError> {
        Ok(Vec::new())
    }
}

// =========================================================================
// Helpers
// =========================================================================

fn write_err(out: *mut *mut c_char, msg: &str) {
    if out.is_null() {
        return;
    }
    match CString::new(msg) {
        Ok(c) => unsafe {
            *out = c.into_raw();
        },
        Err(_) => unsafe {
            // Message contained an interior NUL — shouldn't happen
            // with our own errors; null-out rather than panic.
            *out = std::ptr::null_mut();
        },
    }
}

/// # Safety
///
/// Caller guarantees `ptr` is a valid read-only pointer to a UTF-8
/// byte sequence of `len` bytes (no trailing NUL required).
fn cstr_to_string(ptr: *const c_char, len: usize) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    std::str::from_utf8(bytes).ok().map(|s| s.to_string())
}
