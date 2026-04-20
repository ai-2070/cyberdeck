//! C FFI bindings for CortEX + RedEX, behind the `cortex` feature.
//!
//! Surface targeted at the Go SDK: NetDb, TasksAdapter, MemoriesAdapter,
//! and raw RedexFile access. Everything crosses the boundary as:
//!
//! - Opaque handles (`*mut T`) freed via dedicated `_free` functions.
//! - Scalar IDs / timestamps as `u64`.
//! - Everything else as JSON strings allocated with `CString::into_raw`,
//!   freed by the caller via [`crate::ffi::net_free_string`].
//!
//! Watch / tail iterators use a cursor pattern:
//! `net_*_next(cursor, timeout_ms, out_json, out_len) -> c_int` returns
//! `0 = event delivered`, `1 = timeout`, `2 = stream ended cleanly`,
//! or a negative `NetError`. The Go layer wraps the cursor in a
//! goroutine that pumps into a channel, calling `close` on `ctx.Done()`.

use std::ffi::{c_char, c_int, CStr, CString};
use std::os::raw::c_void;
use std::ptr;
use std::sync::Arc;

use futures::stream::BoxStream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;

use crate::adapter::net::channel::ChannelName;
use crate::adapter::net::cortex::memories::{
    MemoriesAdapter as InnerMemoriesAdapter, MemoriesFilter, MemoriesWatcher, Memory,
    OrderBy as MemoriesOrderBy,
};
use crate::adapter::net::cortex::tasks::{
    OrderBy as TasksOrderBy, Task, TaskStatus, TasksAdapter as InnerTasksAdapter, TasksFilter,
    TasksWatcher,
};
use crate::adapter::net::redex::{
    FsyncPolicy, Redex as InnerRedex, RedexError, RedexEvent, RedexFile as InnerRedexFile,
    RedexFileConfig,
};

use super::NetError;

// =========================================================================
// Extended error codes for the CortEX surface. Keep numbering below -99
// (NetError::Unknown) so they never collide with the base surface.
// =========================================================================

pub(crate) const NET_ERR_CORTEX_CLOSED: c_int = -100;
pub(crate) const NET_ERR_CORTEX_FOLD: c_int = -101;
pub(crate) const NET_ERR_NETDB: c_int = -102;
pub(crate) const NET_ERR_REDEX: c_int = -103;
pub(crate) const NET_ERR_TIMEOUT: c_int = 1;
pub(crate) const NET_ERR_STREAM_ENDED: c_int = 2;

// =========================================================================
// Shared utilities
// =========================================================================

/// One tokio runtime, lazily initialized, used by every CortEX /
/// RedEX FFI call. The watch / tail cursors rely on a single runtime
/// so the spawned forwarding tasks survive across cursor calls.
fn runtime() -> &'static Arc<Runtime> {
    use std::sync::OnceLock;
    static RT: OnceLock<Arc<Runtime>> = OnceLock::new();
    RT.get_or_init(|| {
        Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("cortex ffi tokio runtime"),
        )
    })
}

/// Copy a C string into a Rust string. Returns `None` on null /
/// non-UTF-8.
unsafe fn c_str_to_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok()
}

/// Serialize `value` as JSON into a C-owned string + length. On
/// success writes the pointer to `*out_ptr` and the length to
/// `*out_len` (excluding the null terminator) and returns `0`. The
/// caller must free the string with `net_free_string`.
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

// =========================================================================
// Redex manager
// =========================================================================

pub struct RedexHandle {
    inner: Arc<InnerRedex>,
}

/// Create a new Redex manager. `persistent_dir` may be NULL for
/// heap-only. Returns a heap-allocated handle the caller must free
/// with `net_redex_free`.
#[unsafe(no_mangle)]
pub extern "C" fn net_redex_new(persistent_dir: *const c_char) -> *mut RedexHandle {
    let dir = if persistent_dir.is_null() {
        None
    } else {
        unsafe { c_str_to_str(persistent_dir) }.map(|s| s.to_owned())
    };
    let inner = match dir {
        Some(d) => InnerRedex::new().with_persistent_dir(d),
        None => InnerRedex::new(),
    };
    Box::into_raw(Box::new(RedexHandle {
        inner: Arc::new(inner),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn net_redex_free(handle: *mut RedexHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

// =========================================================================
// RedexFile
// =========================================================================

#[derive(Deserialize, Default)]
struct RedexFileConfigJson {
    #[serde(default)]
    persistent: bool,
    fsync_every_n: Option<u64>,
    fsync_interval_ms: Option<u64>,
    retention_max_events: Option<u64>,
    retention_max_bytes: Option<u64>,
    retention_max_age_ms: Option<u64>,
}

pub struct RedexFileHandle {
    inner: Arc<InnerRedexFile>,
}

/// Open (or get) a RedEX file for raw append / tail / read-range.
/// `config_json` may be NULL for defaults. Writes the file handle to
/// `*out_handle` on success. Caller frees with `net_redex_file_free`.
#[unsafe(no_mangle)]
pub extern "C" fn net_redex_open_file(
    redex: *mut RedexHandle,
    name: *const c_char,
    config_json: *const c_char,
    out_handle: *mut *mut RedexFileHandle,
) -> c_int {
    if redex.is_null() || name.is_null() || out_handle.is_null() {
        return NetError::NullPointer.into();
    }
    let redex = unsafe { &*redex };
    let Some(name_str) = (unsafe { c_str_to_str(name) }) else {
        return NetError::InvalidUtf8.into();
    };
    let Ok(channel) = ChannelName::new(name_str) else {
        return NET_ERR_REDEX;
    };
    let cfg_json: RedexFileConfigJson = if config_json.is_null() {
        RedexFileConfigJson::default()
    } else {
        let Some(s) = (unsafe { c_str_to_str(config_json) }) else {
            return NetError::InvalidUtf8.into();
        };
        match serde_json::from_str(s) {
            Ok(v) => v,
            Err(_) => return NetError::InvalidJson.into(),
        }
    };
    let mut cfg = RedexFileConfig::default();
    cfg.persistent = cfg_json.persistent;
    match (cfg_json.fsync_every_n, cfg_json.fsync_interval_ms) {
        (Some(_), Some(_)) | (Some(0), _) | (_, Some(0)) => return NET_ERR_REDEX,
        (Some(n), None) => cfg.fsync_policy = FsyncPolicy::EveryN(n),
        (None, Some(ms)) => {
            cfg.fsync_policy = FsyncPolicy::Interval(std::time::Duration::from_millis(ms))
        }
        _ => {}
    }
    cfg.retention_max_events = cfg_json.retention_max_events;
    cfg.retention_max_bytes = cfg_json.retention_max_bytes;
    if let Some(ms) = cfg_json.retention_max_age_ms {
        cfg.retention_max_age_ns = Some(ms.saturating_mul(1_000_000));
    }
    match redex.inner.open_file(&channel, cfg) {
        Ok(file) => {
            let handle = Box::new(RedexFileHandle {
                inner: Arc::new(file),
            });
            unsafe {
                *out_handle = Box::into_raw(handle);
            }
            0
        }
        Err(_) => NET_ERR_REDEX,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_redex_file_free(handle: *mut RedexFileHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

/// Append one payload. Writes the assigned seq to `*out_seq`.
#[unsafe(no_mangle)]
pub extern "C" fn net_redex_file_append(
    handle: *mut RedexFileHandle,
    payload: *const u8,
    len: usize,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || payload.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let file = unsafe { &*handle };
    let slice = unsafe { std::slice::from_raw_parts(payload, len) };
    match file.inner.append(slice) {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_REDEX,
    }
}

#[derive(Serialize)]
struct RedexEventJson {
    seq: u64,
    /// Hex-encoded payload so JSON transport is safe for binary data.
    payload_hex: String,
    checksum: u32,
    is_inline: bool,
}

impl From<RedexEvent> for RedexEventJson {
    fn from(ev: RedexEvent) -> Self {
        RedexEventJson {
            seq: ev.entry.seq,
            payload_hex: hex_encode(&ev.payload),
            checksum: ev.entry.checksum(),
            is_inline: ev.entry.is_inline(),
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[unsafe(no_mangle)]
pub extern "C" fn net_redex_file_len(handle: *mut RedexFileHandle) -> u64 {
    if handle.is_null() {
        return 0;
    }
    let file = unsafe { &*handle };
    file.inner.len() as u64
}

/// Read the half-open range `[start, end)` into a JSON array.
#[unsafe(no_mangle)]
pub extern "C" fn net_redex_file_read_range(
    handle: *mut RedexFileHandle,
    start: u64,
    end: u64,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if handle.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let file = unsafe { &*handle };
    let events: Vec<RedexEventJson> = file
        .inner
        .read_range(start, end)
        .into_iter()
        .map(RedexEventJson::from)
        .collect();
    write_json_out(&events, out_json, out_len)
}

#[unsafe(no_mangle)]
pub extern "C" fn net_redex_file_sync(handle: *mut RedexFileHandle) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }
    let file = unsafe { &*handle };
    match file.inner.sync() {
        Ok(()) => 0,
        Err(_) => NET_ERR_REDEX,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_redex_file_close(handle: *mut RedexFileHandle) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }
    let file = unsafe { &*handle };
    match file.inner.close() {
        Ok(()) => 0,
        Err(_) => NET_ERR_REDEX,
    }
}

// RedEX tail cursor

pub struct RedexTailHandle {
    stream: TokioMutex<Option<BoxStream<'static, std::result::Result<RedexEvent, RedexError>>>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn net_redex_file_tail(
    handle: *mut RedexFileHandle,
    from_seq: u64,
    out_cursor: *mut *mut RedexTailHandle,
) -> c_int {
    if handle.is_null() || out_cursor.is_null() {
        return NetError::NullPointer.into();
    }
    let file = unsafe { &*handle };
    let stream = file.inner.tail(from_seq);
    let boxed: BoxStream<'static, std::result::Result<RedexEvent, RedexError>> = stream.boxed();
    let cursor = Box::new(RedexTailHandle {
        stream: TokioMutex::new(Some(boxed)),
    });
    unsafe {
        *out_cursor = Box::into_raw(cursor);
    }
    0
}

/// Pull the next tail event. `timeout_ms == 0` blocks indefinitely.
/// Returns:
/// * `0`  — event delivered; JSON written to `*out_json` (caller frees
///   via `net_free_string`).
/// * `1`  — timeout (no event available within `timeout_ms`).
/// * `2`  — stream ended (file closed or dropped).
/// * negative — error.
#[unsafe(no_mangle)]
pub extern "C" fn net_redex_tail_next(
    cursor: *mut RedexTailHandle,
    timeout_ms: u32,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if cursor.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let cursor = unsafe { &*cursor };
    let rt = runtime();
    rt.block_on(async move {
        let mut guard = cursor.stream.lock().await;
        let Some(stream) = guard.as_mut() else {
            return NET_ERR_STREAM_ENDED;
        };
        let next_fut = stream.next();
        let outcome = if timeout_ms == 0 {
            next_fut.await
        } else {
            match tokio::time::timeout(
                std::time::Duration::from_millis(timeout_ms as u64),
                next_fut,
            )
            .await
            {
                Ok(v) => v,
                Err(_) => return NET_ERR_TIMEOUT,
            }
        };
        match outcome {
            Some(Ok(ev)) => {
                let js = RedexEventJson::from(ev);
                write_json_out(&js, out_json, out_len)
            }
            Some(Err(RedexError::Closed)) | None => {
                *guard = None;
                NET_ERR_STREAM_ENDED
            }
            Some(Err(_)) => {
                *guard = None;
                NET_ERR_REDEX
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn net_redex_tail_free(cursor: *mut RedexTailHandle) {
    if cursor.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(cursor));
    }
}

// =========================================================================
// Tasks adapter — standalone open. Go-side `NetDb` struct composes
// Redex + Tasks + Memories without a dedicated FFI handle.
// =========================================================================

pub struct TasksAdapterHandle {
    inner: Arc<InnerTasksAdapter>,
}

/// Open a tasks adapter against a Redex. `persistent != 0` routes
/// writes through the Redex's persistent directory (requires the
/// Redex to have been created with a `persistent_dir`).
#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_adapter_open(
    redex: *mut RedexHandle,
    origin_hash: u32,
    persistent: c_int,
    out_handle: *mut *mut TasksAdapterHandle,
) -> c_int {
    if redex.is_null() || out_handle.is_null() {
        return NetError::NullPointer.into();
    }
    let redex = unsafe { &*redex };
    let cfg = if persistent != 0 {
        RedexFileConfig::default().with_persistent(true)
    } else {
        RedexFileConfig::default()
    };
    // `open_with_config` spawns the fold task via `tokio::spawn` and
    // needs a live reactor; run under our runtime.
    let rt = runtime();
    let redex_inner = redex.inner.clone();
    let result = rt.block_on(async move {
        InnerTasksAdapter::open_with_config(&redex_inner, origin_hash, cfg)
    });
    match result {
        Ok(adapter) => {
            let handle = Box::new(TasksAdapterHandle {
                inner: Arc::new(adapter),
            });
            unsafe {
                *out_handle = Box::into_raw(handle);
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_adapter_close(handle: *mut TasksAdapterHandle) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }
    let tasks = unsafe { &*handle };
    match tasks.inner.close() {
        Ok(()) => 0,
        Err(_) => NET_ERR_CORTEX_CLOSED,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_adapter_free(handle: *mut TasksAdapterHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[derive(Serialize)]
struct TaskJson {
    id: u64,
    title: String,
    status: &'static str,
    created_ns: u64,
    updated_ns: u64,
}

impl From<Task> for TaskJson {
    fn from(t: Task) -> Self {
        TaskJson {
            id: t.id,
            title: t.title,
            status: match t.status {
                TaskStatus::Pending => "pending",
                TaskStatus::Completed => "completed",
            },
            created_ns: t.created_ns,
            updated_ns: t.updated_ns,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_create(
    handle: *mut TasksAdapterHandle,
    id: u64,
    title: *const c_char,
    now_ns: u64,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || title.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let tasks = unsafe { &*handle };
    let Some(title) = (unsafe { c_str_to_str(title) }) else {
        return NetError::InvalidUtf8.into();
    };
    match tasks.inner.create(id, title, now_ns) {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_rename(
    handle: *mut TasksAdapterHandle,
    id: u64,
    new_title: *const c_char,
    now_ns: u64,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || new_title.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let tasks = unsafe { &*handle };
    let Some(nt) = (unsafe { c_str_to_str(new_title) }) else {
        return NetError::InvalidUtf8.into();
    };
    match tasks.inner.rename(id, nt, now_ns) {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_complete(
    handle: *mut TasksAdapterHandle,
    id: u64,
    now_ns: u64,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let tasks = unsafe { &*handle };
    match tasks.inner.complete(id, now_ns) {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_delete(
    handle: *mut TasksAdapterHandle,
    id: u64,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let tasks = unsafe { &*handle };
    match tasks.inner.delete(id) {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

/// Block until fold has applied every event up through `seq`. Pass
/// `timeout_ms == 0` to wait indefinitely. Returns `0` on success,
/// `1` on timeout, or negative on error.
#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_wait_for_seq(
    handle: *mut TasksAdapterHandle,
    seq: u64,
    timeout_ms: u32,
) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }
    let tasks = unsafe { &*handle };
    let adapter = tasks.inner.clone();
    let rt = runtime();
    rt.block_on(async move {
        let fut = adapter.wait_for_seq(seq);
        if timeout_ms == 0 {
            fut.await;
            0
        } else {
            match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms as u64), fut)
                .await
            {
                Ok(_) => 0,
                Err(_) => NET_ERR_TIMEOUT,
            }
        }
    })
}

#[derive(Deserialize, Default)]
struct TasksFilterJson {
    status: Option<String>,
    title_contains: Option<String>,
    created_after_ns: Option<u64>,
    created_before_ns: Option<u64>,
    updated_after_ns: Option<u64>,
    updated_before_ns: Option<u64>,
    order_by: Option<String>,
    limit: Option<u32>,
}

fn build_tasks_watcher(
    adapter: &InnerTasksAdapter,
    filter_json: *const c_char,
) -> Result<TasksWatcher, c_int> {
    let mut w = adapter.watch();
    if filter_json.is_null() {
        return Ok(w);
    }
    let Some(s) = (unsafe { c_str_to_str(filter_json) }) else {
        return Err(NetError::InvalidUtf8.into());
    };
    let f: TasksFilterJson = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return Err(NetError::InvalidJson.into()),
    };
    w = match f.status.as_deref() {
        Some("pending") => w.where_status(TaskStatus::Pending),
        Some("completed") => w.where_status(TaskStatus::Completed),
        Some(_) => return Err(NetError::InvalidJson.into()),
        None => w,
    };
    if let Some(s) = f.title_contains {
        w = w.title_contains(s);
    }
    if let Some(ns) = f.created_after_ns {
        w = w.created_after(ns);
    }
    if let Some(ns) = f.created_before_ns {
        w = w.created_before(ns);
    }
    if let Some(ns) = f.updated_after_ns {
        w = w.updated_after(ns);
    }
    if let Some(ns) = f.updated_before_ns {
        w = w.updated_before(ns);
    }
    if let Some(o) = f.order_by.as_deref() {
        w = match o {
            "id_asc" => w.order_by(TasksOrderBy::IdAsc),
            "id_desc" => w.order_by(TasksOrderBy::IdDesc),
            "created_asc" => w.order_by(TasksOrderBy::CreatedAsc),
            "created_desc" => w.order_by(TasksOrderBy::CreatedDesc),
            "updated_asc" => w.order_by(TasksOrderBy::UpdatedAsc),
            "updated_desc" => w.order_by(TasksOrderBy::UpdatedDesc),
            _ => return Err(NetError::InvalidJson.into()),
        };
    }
    if let Some(l) = f.limit {
        w = w.limit(l as usize);
    }
    Ok(w)
}

/// Apply JSON filter to a query-side filter (used by `list_tasks`).
fn build_tasks_list_filter(filter_json: *const c_char) -> Result<TasksFilter, c_int> {
    if filter_json.is_null() {
        return Ok(TasksFilter::default());
    }
    let Some(s) = (unsafe { c_str_to_str(filter_json) }) else {
        return Err(NetError::InvalidUtf8.into());
    };
    let f: TasksFilterJson = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return Err(NetError::InvalidJson.into()),
    };
    let mut out = TasksFilter::default();
    out.status = match f.status.as_deref() {
        Some("pending") => Some(TaskStatus::Pending),
        Some("completed") => Some(TaskStatus::Completed),
        Some(_) => return Err(NetError::InvalidJson.into()),
        None => None,
    };
    out.title_contains = f.title_contains;
    out.created_after_ns = f.created_after_ns;
    out.created_before_ns = f.created_before_ns;
    out.updated_after_ns = f.updated_after_ns;
    out.updated_before_ns = f.updated_before_ns;
    out.order_by = f.order_by.as_deref().and_then(|o| match o {
        "id_asc" => Some(TasksOrderBy::IdAsc),
        "id_desc" => Some(TasksOrderBy::IdDesc),
        "created_asc" => Some(TasksOrderBy::CreatedAsc),
        "created_desc" => Some(TasksOrderBy::CreatedDesc),
        "updated_asc" => Some(TasksOrderBy::UpdatedAsc),
        "updated_desc" => Some(TasksOrderBy::UpdatedDesc),
        _ => None,
    });
    out.limit = f.limit.map(|l| l as usize);
    Ok(out)
}

fn run_tasks_list(tasks: &InnerTasksAdapter, filter: &TasksFilter) -> Vec<Task> {
    let state = tasks.state();
    let guard = state.read();
    let mut q = guard.query();
    if let Some(s) = filter.status {
        q = q.where_status(s);
    }
    if let Some(s) = &filter.title_contains {
        q = q.title_contains(s.clone());
    }
    if let Some(ns) = filter.created_after_ns {
        q = q.created_after(ns);
    }
    if let Some(ns) = filter.created_before_ns {
        q = q.created_before(ns);
    }
    if let Some(ns) = filter.updated_after_ns {
        q = q.updated_after(ns);
    }
    if let Some(ns) = filter.updated_before_ns {
        q = q.updated_before(ns);
    }
    if let Some(o) = filter.order_by {
        q = q.order_by(o);
    }
    if let Some(l) = filter.limit {
        q = q.limit(l);
    }
    q.collect()
}

/// List tasks matching `filter_json` (may be NULL). Writes a JSON
/// array of tasks to `*out_json`; caller frees via `net_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_list(
    handle: *mut TasksAdapterHandle,
    filter_json: *const c_char,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if handle.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let tasks = unsafe { &*handle };
    let filter = match build_tasks_list_filter(filter_json) {
        Ok(f) => f,
        Err(code) => return code,
    };
    let items: Vec<TaskJson> = run_tasks_list(&tasks.inner, &filter)
        .into_iter()
        .map(TaskJson::from)
        .collect();
    write_json_out(&items, out_json, out_len)
}

pub struct TasksWatchHandle {
    stream: TokioMutex<Option<BoxStream<'static, Vec<Task>>>>,
}

/// Atomic snapshot + watch. Writes:
/// * `*out_snapshot` — JSON array of tasks in the current filter result.
/// * `*out_cursor` — watch cursor; iterate via `net_tasks_watch_next`
///   and free via `net_tasks_watch_free`.
#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_snapshot_and_watch(
    handle: *mut TasksAdapterHandle,
    filter_json: *const c_char,
    out_snapshot: *mut *mut c_char,
    out_snapshot_len: *mut usize,
    out_cursor: *mut *mut TasksWatchHandle,
) -> c_int {
    if handle.is_null()
        || out_snapshot.is_null()
        || out_snapshot_len.is_null()
        || out_cursor.is_null()
    {
        return NetError::NullPointer.into();
    }
    let tasks = unsafe { &*handle };
    let watcher = match build_tasks_watcher(&tasks.inner, filter_json) {
        Ok(w) => w,
        Err(code) => return code,
    };
    // `watcher.stream()` spawns a forwarding task — needs a live
    // reactor.
    let adapter = tasks.inner.clone();
    let rt = runtime();
    let (snapshot, stream) = rt.block_on(async move { adapter.snapshot_and_watch(watcher) });
    let snapshot_json: Vec<TaskJson> = snapshot.into_iter().map(TaskJson::from).collect();
    let code = write_json_out(&snapshot_json, out_snapshot, out_snapshot_len);
    if code != 0 {
        return code;
    }
    let handle = Box::new(TasksWatchHandle {
        stream: TokioMutex::new(Some(stream)),
    });
    unsafe {
        *out_cursor = Box::into_raw(handle);
    }
    0
}

/// Pull the next tasks-watch batch. Semantics match
/// [`net_redex_tail_next`] — `0` on event (JSON array written),
/// `1` on timeout, `2` on stream end, negative on error.
#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_watch_next(
    cursor: *mut TasksWatchHandle,
    timeout_ms: u32,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if cursor.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let cursor = unsafe { &*cursor };
    let rt = runtime();
    rt.block_on(async move {
        let mut guard = cursor.stream.lock().await;
        let Some(stream) = guard.as_mut() else {
            return NET_ERR_STREAM_ENDED;
        };
        let next_fut = stream.next();
        let outcome = if timeout_ms == 0 {
            next_fut.await
        } else {
            match tokio::time::timeout(
                std::time::Duration::from_millis(timeout_ms as u64),
                next_fut,
            )
            .await
            {
                Ok(v) => v,
                Err(_) => return NET_ERR_TIMEOUT,
            }
        };
        match outcome {
            Some(batch) => {
                let js: Vec<TaskJson> = batch.into_iter().map(TaskJson::from).collect();
                write_json_out(&js, out_json, out_len)
            }
            None => {
                *guard = None;
                NET_ERR_STREAM_ENDED
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn net_tasks_watch_free(cursor: *mut TasksWatchHandle) {
    if cursor.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(cursor));
    }
}

// =========================================================================
// Memories adapter (same shape as tasks)
// =========================================================================

pub struct MemoriesAdapterHandle {
    inner: Arc<InnerMemoriesAdapter>,
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_adapter_open(
    redex: *mut RedexHandle,
    origin_hash: u32,
    persistent: c_int,
    out_handle: *mut *mut MemoriesAdapterHandle,
) -> c_int {
    if redex.is_null() || out_handle.is_null() {
        return NetError::NullPointer.into();
    }
    let redex = unsafe { &*redex };
    let cfg = if persistent != 0 {
        RedexFileConfig::default().with_persistent(true)
    } else {
        RedexFileConfig::default()
    };
    let rt = runtime();
    let redex_inner = redex.inner.clone();
    let result = rt.block_on(async move {
        InnerMemoriesAdapter::open_with_config(&redex_inner, origin_hash, cfg)
    });
    match result {
        Ok(adapter) => {
            let handle = Box::new(MemoriesAdapterHandle {
                inner: Arc::new(adapter),
            });
            unsafe {
                *out_handle = Box::into_raw(handle);
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_adapter_close(handle: *mut MemoriesAdapterHandle) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }
    let mem = unsafe { &*handle };
    match mem.inner.close() {
        Ok(()) => 0,
        Err(_) => NET_ERR_CORTEX_CLOSED,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_adapter_free(handle: *mut MemoriesAdapterHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[derive(Serialize)]
struct MemoryJson {
    id: u64,
    content: String,
    tags: Vec<String>,
    source: String,
    created_ns: u64,
    updated_ns: u64,
    pinned: bool,
}

impl From<Memory> for MemoryJson {
    fn from(m: Memory) -> Self {
        MemoryJson {
            id: m.id,
            content: m.content,
            tags: m.tags,
            source: m.source,
            created_ns: m.created_ns,
            updated_ns: m.updated_ns,
            pinned: m.pinned,
        }
    }
}

#[derive(Deserialize)]
struct MemoryStoreInput {
    id: u64,
    content: String,
    tags: Vec<String>,
    source: String,
    now_ns: u64,
}

/// Store a memory. Input is a JSON object
/// `{id, content, tags, source, now_ns}`.
#[unsafe(no_mangle)]
pub extern "C" fn net_memories_store(
    handle: *mut MemoriesAdapterHandle,
    input_json: *const c_char,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || input_json.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let mem = unsafe { &*handle };
    let Some(s) = (unsafe { c_str_to_str(input_json) }) else {
        return NetError::InvalidUtf8.into();
    };
    let input: MemoryStoreInput = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return NetError::InvalidJson.into(),
    };
    match mem
        .inner
        .store(input.id, input.content, input.tags, input.source, input.now_ns)
    {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[derive(Deserialize)]
struct MemoryRetagInput {
    id: u64,
    tags: Vec<String>,
    now_ns: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_retag(
    handle: *mut MemoriesAdapterHandle,
    input_json: *const c_char,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || input_json.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let mem = unsafe { &*handle };
    let Some(s) = (unsafe { c_str_to_str(input_json) }) else {
        return NetError::InvalidUtf8.into();
    };
    let input: MemoryRetagInput = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return NetError::InvalidJson.into(),
    };
    match mem.inner.retag(input.id, input.tags, input.now_ns) {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_pin(
    handle: *mut MemoriesAdapterHandle,
    id: u64,
    now_ns: u64,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let mem = unsafe { &*handle };
    match mem.inner.pin(id, now_ns) {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_unpin(
    handle: *mut MemoriesAdapterHandle,
    id: u64,
    now_ns: u64,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let mem = unsafe { &*handle };
    match mem.inner.unpin(id, now_ns) {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_delete(
    handle: *mut MemoriesAdapterHandle,
    id: u64,
    out_seq: *mut u64,
) -> c_int {
    if handle.is_null() || out_seq.is_null() {
        return NetError::NullPointer.into();
    }
    let mem = unsafe { &*handle };
    match mem.inner.delete(id) {
        Ok(seq) => {
            unsafe {
                *out_seq = seq;
            }
            0
        }
        Err(_) => NET_ERR_CORTEX_FOLD,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_wait_for_seq(
    handle: *mut MemoriesAdapterHandle,
    seq: u64,
    timeout_ms: u32,
) -> c_int {
    if handle.is_null() {
        return NetError::NullPointer.into();
    }
    let mem = unsafe { &*handle };
    let adapter = mem.inner.clone();
    let rt = runtime();
    rt.block_on(async move {
        let fut = adapter.wait_for_seq(seq);
        if timeout_ms == 0 {
            fut.await;
            0
        } else {
            match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms as u64), fut)
                .await
            {
                Ok(_) => 0,
                Err(_) => NET_ERR_TIMEOUT,
            }
        }
    })
}

#[derive(Deserialize, Default)]
struct MemoriesFilterJson {
    source: Option<String>,
    content_contains: Option<String>,
    tag: Option<String>,
    any_tag: Option<Vec<String>>,
    all_tags: Option<Vec<String>>,
    pinned: Option<bool>,
    created_after_ns: Option<u64>,
    created_before_ns: Option<u64>,
    updated_after_ns: Option<u64>,
    updated_before_ns: Option<u64>,
    order_by: Option<String>,
    limit: Option<u32>,
}

fn parse_memories_order_by(s: &str) -> Option<MemoriesOrderBy> {
    match s {
        "id_asc" => Some(MemoriesOrderBy::IdAsc),
        "id_desc" => Some(MemoriesOrderBy::IdDesc),
        "created_asc" => Some(MemoriesOrderBy::CreatedAsc),
        "created_desc" => Some(MemoriesOrderBy::CreatedDesc),
        "updated_asc" => Some(MemoriesOrderBy::UpdatedAsc),
        "updated_desc" => Some(MemoriesOrderBy::UpdatedDesc),
        _ => None,
    }
}

fn build_memories_watcher(
    adapter: &InnerMemoriesAdapter,
    filter_json: *const c_char,
) -> Result<MemoriesWatcher, c_int> {
    let mut w = adapter.watch();
    if filter_json.is_null() {
        return Ok(w);
    }
    let Some(s) = (unsafe { c_str_to_str(filter_json) }) else {
        return Err(NetError::InvalidUtf8.into());
    };
    let f: MemoriesFilterJson = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return Err(NetError::InvalidJson.into()),
    };
    if let Some(s) = f.source {
        w = w.where_source(s);
    }
    if let Some(s) = f.content_contains {
        w = w.content_contains(s);
    }
    if let Some(t) = f.tag {
        w = w.where_tag(t);
    }
    if let Some(tags) = f.any_tag {
        w = w.where_any_tag(tags);
    }
    if let Some(tags) = f.all_tags {
        w = w.where_all_tags(tags);
    }
    if let Some(p) = f.pinned {
        w = w.where_pinned(p);
    }
    if let Some(ns) = f.created_after_ns {
        w = w.created_after(ns);
    }
    if let Some(ns) = f.created_before_ns {
        w = w.created_before(ns);
    }
    if let Some(ns) = f.updated_after_ns {
        w = w.updated_after(ns);
    }
    if let Some(ns) = f.updated_before_ns {
        w = w.updated_before(ns);
    }
    if let Some(o) = f.order_by.as_deref() {
        if let Some(ob) = parse_memories_order_by(o) {
            w = w.order_by(ob);
        } else {
            return Err(NetError::InvalidJson.into());
        }
    }
    if let Some(l) = f.limit {
        w = w.limit(l as usize);
    }
    Ok(w)
}

fn build_memories_list_filter(filter_json: *const c_char) -> Result<MemoriesFilter, c_int> {
    if filter_json.is_null() {
        return Ok(MemoriesFilter::default());
    }
    let Some(s) = (unsafe { c_str_to_str(filter_json) }) else {
        return Err(NetError::InvalidUtf8.into());
    };
    let f: MemoriesFilterJson = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return Err(NetError::InvalidJson.into()),
    };
    let mut out = MemoriesFilter::default();
    out.source = f.source;
    out.content_contains = f.content_contains;
    out.tag = f.tag;
    out.any_tag = f.any_tag;
    out.all_tags = f.all_tags;
    out.pinned = f.pinned;
    out.created_after_ns = f.created_after_ns;
    out.created_before_ns = f.created_before_ns;
    out.updated_after_ns = f.updated_after_ns;
    out.updated_before_ns = f.updated_before_ns;
    out.order_by = f
        .order_by
        .as_deref()
        .and_then(parse_memories_order_by);
    out.limit = f.limit.map(|l| l as usize);
    Ok(out)
}

fn run_memories_list(mem: &InnerMemoriesAdapter, filter: &MemoriesFilter) -> Vec<Memory> {
    let state = mem.state();
    let guard = state.read();
    let mut q = guard.query();
    if let Some(s) = &filter.source {
        q = q.where_source(s.clone());
    }
    if let Some(s) = &filter.content_contains {
        q = q.content_contains(s.clone());
    }
    if let Some(t) = &filter.tag {
        q = q.where_tag(t.clone());
    }
    if let Some(tags) = &filter.any_tag {
        q = q.where_any_tag(tags.clone());
    }
    if let Some(tags) = &filter.all_tags {
        q = q.where_all_tags(tags.clone());
    }
    if let Some(p) = filter.pinned {
        q = q.where_pinned(p);
    }
    if let Some(ns) = filter.created_after_ns {
        q = q.created_after(ns);
    }
    if let Some(ns) = filter.created_before_ns {
        q = q.created_before(ns);
    }
    if let Some(ns) = filter.updated_after_ns {
        q = q.updated_after(ns);
    }
    if let Some(ns) = filter.updated_before_ns {
        q = q.updated_before(ns);
    }
    if let Some(o) = filter.order_by {
        q = q.order_by(o);
    }
    if let Some(l) = filter.limit {
        q = q.limit(l);
    }
    q.collect()
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_list(
    handle: *mut MemoriesAdapterHandle,
    filter_json: *const c_char,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if handle.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let mem = unsafe { &*handle };
    let filter = match build_memories_list_filter(filter_json) {
        Ok(f) => f,
        Err(code) => return code,
    };
    let items: Vec<MemoryJson> = run_memories_list(&mem.inner, &filter)
        .into_iter()
        .map(MemoryJson::from)
        .collect();
    write_json_out(&items, out_json, out_len)
}

pub struct MemoriesWatchHandle {
    stream: TokioMutex<Option<BoxStream<'static, Vec<Memory>>>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_snapshot_and_watch(
    handle: *mut MemoriesAdapterHandle,
    filter_json: *const c_char,
    out_snapshot: *mut *mut c_char,
    out_snapshot_len: *mut usize,
    out_cursor: *mut *mut MemoriesWatchHandle,
) -> c_int {
    if handle.is_null()
        || out_snapshot.is_null()
        || out_snapshot_len.is_null()
        || out_cursor.is_null()
    {
        return NetError::NullPointer.into();
    }
    let mem = unsafe { &*handle };
    let watcher = match build_memories_watcher(&mem.inner, filter_json) {
        Ok(w) => w,
        Err(code) => return code,
    };
    let adapter = mem.inner.clone();
    let rt = runtime();
    let (snapshot, stream) = rt.block_on(async move { adapter.snapshot_and_watch(watcher) });
    let snapshot_json: Vec<MemoryJson> = snapshot.into_iter().map(MemoryJson::from).collect();
    let code = write_json_out(&snapshot_json, out_snapshot, out_snapshot_len);
    if code != 0 {
        return code;
    }
    let handle = Box::new(MemoriesWatchHandle {
        stream: TokioMutex::new(Some(stream)),
    });
    unsafe {
        *out_cursor = Box::into_raw(handle);
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_watch_next(
    cursor: *mut MemoriesWatchHandle,
    timeout_ms: u32,
    out_json: *mut *mut c_char,
    out_len: *mut usize,
) -> c_int {
    if cursor.is_null() || out_json.is_null() || out_len.is_null() {
        return NetError::NullPointer.into();
    }
    let cursor = unsafe { &*cursor };
    let rt = runtime();
    rt.block_on(async move {
        let mut guard = cursor.stream.lock().await;
        let Some(stream) = guard.as_mut() else {
            return NET_ERR_STREAM_ENDED;
        };
        let next_fut = stream.next();
        let outcome = if timeout_ms == 0 {
            next_fut.await
        } else {
            match tokio::time::timeout(
                std::time::Duration::from_millis(timeout_ms as u64),
                next_fut,
            )
            .await
            {
                Ok(v) => v,
                Err(_) => return NET_ERR_TIMEOUT,
            }
        };
        match outcome {
            Some(batch) => {
                let js: Vec<MemoryJson> = batch.into_iter().map(MemoryJson::from).collect();
                write_json_out(&js, out_json, out_len)
            }
            None => {
                *guard = None;
                NET_ERR_STREAM_ENDED
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn net_memories_watch_free(cursor: *mut MemoriesWatchHandle) {
    if cursor.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(cursor));
    }
}

// ABI-visible no-op to force the linker to keep `c_void` happy on
// some older linkers; harmless otherwise.
#[doc(hidden)]
pub fn _ffi_cortex_keep_alive() -> *mut c_void {
    ptr::null_mut()
}
