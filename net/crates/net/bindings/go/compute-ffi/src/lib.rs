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

use dashmap::DashMap;
use net::adapter::net::channel::ChannelConfigRegistry;
use net::adapter::net::MeshNode;
use net_sdk::compute::DaemonRuntime as SdkDaemonRuntime;
use net_sdk::mesh::Mesh as SdkMesh;
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
