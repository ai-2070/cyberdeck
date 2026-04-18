//! Node.js bindings for the CortEX adapter slice — tasks + memories.
//!
//! Feature-gated behind `cortex` in this crate (which turns on the
//! core's `cortex-tasks` + `cortex-memories`). Exposes:
//!
//! - [`Redex`] — local RedEX manager handle
//! - [`TasksAdapter`] / [`MemoriesAdapter`] — typed adapters with CRUD
//!   plus a synchronous `list*(filter)` snapshot query
//!
//! u64 fields (ids, timestamps, RedEX sequences) cross the napi
//! boundary as `BigInt` to preserve full 64-bit precision.
//!
//! Watch / `AsyncIterator` is deliberately deferred — the JS async
//! iterator glue lands in a follow-up session.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::stream::BoxStream;
use futures::StreamExt;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use tokio::sync::{Mutex as TokioMutex, Notify};

use ::net::adapter::net::cortex::memories::{
    MemoriesAdapter as InnerMemoriesAdapter, Memory as InnerMemory, OrderBy as InnerMemoriesOrderBy,
};
use ::net::adapter::net::cortex::tasks::{
    OrderBy as InnerTasksOrderBy, Task as InnerTask, TaskStatus as InnerTaskStatus,
    TasksAdapter as InnerTasksAdapter,
};
use ::net::adapter::net::redex::{Redex as InnerRedex, RedexFileConfig};

// =========================================================================
// Shared helpers
// =========================================================================

#[inline]
fn bigint_u64(b: BigInt) -> u64 {
    b.get_u64().1
}

// =========================================================================
// Redex manager
// =========================================================================

/// Local RedEX manager. Holds the set of open files on this node.
///
/// Cheap to share — methods take `&self`.
#[napi]
pub struct Redex {
    inner: Arc<InnerRedex>,
}

#[napi]
impl Redex {
    /// Open a new Redex manager.
    ///
    /// `persistentDir`: if provided, files opened through adapters
    /// with `persistent: true` write to `<persistentDir>/<channel_path>/{idx,dat}`
    /// and replay from those files on reopen. Heap-only when omitted.
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new(persistent_dir: Option<String>) -> Self {
        let inner = match persistent_dir {
            Some(dir) => InnerRedex::new().with_persistent_dir(dir),
            None => InnerRedex::new(),
        };
        Self {
            inner: Arc::new(inner),
        }
    }
}

// =========================================================================
// Tasks
// =========================================================================

/// Task lifecycle status.
#[napi(string_enum)]
pub enum TaskStatus {
    Pending,
    Completed,
}

impl From<InnerTaskStatus> for TaskStatus {
    fn from(s: InnerTaskStatus) -> Self {
        match s {
            InnerTaskStatus::Pending => TaskStatus::Pending,
            InnerTaskStatus::Completed => TaskStatus::Completed,
        }
    }
}

impl From<TaskStatus> for InnerTaskStatus {
    fn from(s: TaskStatus) -> Self {
        match s {
            TaskStatus::Pending => InnerTaskStatus::Pending,
            TaskStatus::Completed => InnerTaskStatus::Completed,
        }
    }
}

/// Ordering for task queries.
#[napi(string_enum)]
pub enum TasksOrderBy {
    IdAsc,
    IdDesc,
    CreatedAsc,
    CreatedDesc,
    UpdatedAsc,
    UpdatedDesc,
}

impl From<TasksOrderBy> for InnerTasksOrderBy {
    fn from(o: TasksOrderBy) -> Self {
        match o {
            TasksOrderBy::IdAsc => InnerTasksOrderBy::IdAsc,
            TasksOrderBy::IdDesc => InnerTasksOrderBy::IdDesc,
            TasksOrderBy::CreatedAsc => InnerTasksOrderBy::CreatedAsc,
            TasksOrderBy::CreatedDesc => InnerTasksOrderBy::CreatedDesc,
            TasksOrderBy::UpdatedAsc => InnerTasksOrderBy::UpdatedAsc,
            TasksOrderBy::UpdatedDesc => InnerTasksOrderBy::UpdatedDesc,
        }
    }
}

/// A materialized task record.
#[napi(object)]
pub struct Task {
    pub id: BigInt,
    pub title: String,
    pub status: TaskStatus,
    pub created_ns: BigInt,
    pub updated_ns: BigInt,
}

impl From<InnerTask> for Task {
    fn from(t: InnerTask) -> Self {
        Task {
            id: BigInt::from(t.id),
            title: t.title,
            status: t.status.into(),
            created_ns: BigInt::from(t.created_ns),
            updated_ns: BigInt::from(t.updated_ns),
        }
    }
}

/// Filter for [`TasksAdapter::list_tasks`] and
/// [`TasksAdapter::watch_tasks`].
#[napi(object)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub title_contains: Option<String>,
    pub created_after_ns: Option<BigInt>,
    pub created_before_ns: Option<BigInt>,
    pub updated_after_ns: Option<BigInt>,
    pub updated_before_ns: Option<BigInt>,
    pub order_by: Option<TasksOrderBy>,
    pub limit: Option<u32>,
}

// =========================================================================
// Task watch iterator (napi)
// =========================================================================

struct TaskWatchIterInner {
    stream: TokioMutex<Option<BoxStream<'static, Vec<InnerTask>>>>,
    shutdown: Notify,
    /// Set by `close()` before notifying. `next()` pre-checks this
    /// flag so a close that races ahead of `next()` is still observed
    /// (raw `Notify::notify_waiters` only wakes currently-registered
    /// waiters).
    is_shutdown: AtomicBool,
}

/// Async iterator over a live task filter.
///
/// Rust returns `null` from [`Self::next`] when the underlying
/// watcher ends; JS should treat that as `done: true`. Paired with
/// the JS helper in the test suite below, this cleanly wraps into a
/// `for await (const tasks of ...)` loop.
#[napi]
pub struct TaskWatchIter {
    inner: Arc<TaskWatchIterInner>,
}

#[napi]
impl TaskWatchIter {
    /// Wait for the next filter result. Returns `null` when the
    /// iterator has been closed or the underlying stream has ended.
    #[napi]
    pub async fn next(&self) -> Option<Vec<Task>> {
        if self.inner.is_shutdown.load(Ordering::Acquire) {
            return None;
        }
        let mut guard = self.inner.stream.lock().await;
        let stream = match guard.as_mut() {
            Some(s) => s,
            None => return None,
        };

        // Pre-register the shutdown waker before re-checking the flag
        // so a close that fires between the check and the await is
        // still observed.
        let shutdown_fut = self.inner.shutdown.notified();
        tokio::pin!(shutdown_fut);
        shutdown_fut.as_mut().enable();

        if self.inner.is_shutdown.load(Ordering::Acquire) {
            *guard = None;
            return None;
        }

        tokio::select! {
            biased;
            _ = shutdown_fut => {
                *guard = None;
                None
            }
            msg = stream.next() => match msg {
                Some(items) => Some(items.into_iter().map(Task::from).collect()),
                None => {
                    *guard = None;
                    None
                }
            }
        }
    }

    /// Terminate the iterator early. Any pending `next()` call
    /// resolves to `null`. Subsequent `next()` calls also return
    /// `null`. Idempotent.
    #[napi]
    pub fn close(&self) {
        self.inner.is_shutdown.store(true, Ordering::Release);
        self.inner.shutdown.notify_waiters();
    }
}

/// Typed tasks adapter handle.
#[napi]
pub struct TasksAdapter {
    inner: Arc<InnerTasksAdapter>,
}

#[napi]
impl TasksAdapter {
    /// Open the tasks adapter against a Redex manager.
    ///
    /// `persistent` — when `true`, the file writes to disk under the
    /// Redex's configured persistent directory and replays from disk
    /// on reopen. Requires the Redex to have been constructed with
    /// `persistentDir`; otherwise `open()` errors.
    ///
    /// Declared `async` so napi-rs runs it with its tokio runtime
    /// active — the underlying `CortexAdapter::open` spawns the
    /// fold task via `tokio::spawn` and needs a live reactor.
    #[napi(factory)]
    pub async fn open(redex: &Redex, origin_hash: u32, persistent: Option<bool>) -> Result<Self> {
        let cfg = if persistent.unwrap_or(false) {
            RedexFileConfig::default().with_persistent(true)
        } else {
            RedexFileConfig::default()
        };
        let inner = InnerTasksAdapter::open_with_config(&redex.inner, origin_hash, cfg)
            .map_err(|e| Error::from_reason(format!("TasksAdapter open failed: {}", e)))?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Create a new task. Returns the RedEX sequence of the append.
    #[napi]
    pub fn create(&self, id: BigInt, title: String, now_ns: BigInt) -> Result<BigInt> {
        self.inner
            .create(bigint_u64(id), title, bigint_u64(now_ns))
            .map(BigInt::from)
            .map_err(|e| Error::from_reason(format!("create failed: {}", e)))
    }

    /// Rename an existing task. No-op at fold time if `id` is unknown.
    #[napi]
    pub fn rename(&self, id: BigInt, new_title: String, now_ns: BigInt) -> Result<BigInt> {
        self.inner
            .rename(bigint_u64(id), new_title, bigint_u64(now_ns))
            .map(BigInt::from)
            .map_err(|e| Error::from_reason(format!("rename failed: {}", e)))
    }

    /// Mark a task completed. No-op at fold time if `id` is unknown.
    #[napi]
    pub fn complete(&self, id: BigInt, now_ns: BigInt) -> Result<BigInt> {
        self.inner
            .complete(bigint_u64(id), bigint_u64(now_ns))
            .map(BigInt::from)
            .map_err(|e| Error::from_reason(format!("complete failed: {}", e)))
    }

    /// Delete a task.
    #[napi]
    pub fn delete(&self, id: BigInt) -> Result<BigInt> {
        self.inner
            .delete(bigint_u64(id))
            .map(BigInt::from)
            .map_err(|e| Error::from_reason(format!("delete failed: {}", e)))
    }

    /// Block until every event up through `seq` has been folded into
    /// state. Use as a read-after-write barrier.
    #[napi]
    pub async fn wait_for_seq(&self, seq: BigInt) {
        self.inner.wait_for_seq(bigint_u64(seq)).await;
    }

    /// Close the adapter. Idempotent.
    #[napi]
    pub fn close(&self) -> Result<()> {
        self.inner
            .close()
            .map_err(|e| Error::from_reason(format!("close failed: {}", e)))
    }

    /// True if the fold task is currently running.
    #[napi]
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    /// Snapshot query over current state. Clones out matching tasks
    /// as a Vec. Pass `null` / `undefined` for no filter (returns all).
    #[napi]
    pub fn list_tasks(&self, filter: Option<TaskFilter>) -> Vec<Task> {
        let state_handle = self.inner.state();
        let state = state_handle.read();
        let mut q = state.query();
        if let Some(f) = filter {
            if let Some(s) = f.status {
                q = q.where_status(s.into());
            }
            if let Some(s) = f.title_contains {
                q = q.title_contains(s);
            }
            if let Some(ns) = f.created_after_ns {
                q = q.created_after(bigint_u64(ns));
            }
            if let Some(ns) = f.created_before_ns {
                q = q.created_before(bigint_u64(ns));
            }
            if let Some(ns) = f.updated_after_ns {
                q = q.updated_after(bigint_u64(ns));
            }
            if let Some(ns) = f.updated_before_ns {
                q = q.updated_before(bigint_u64(ns));
            }
            if let Some(o) = f.order_by {
                q = q.order_by(o.into());
            }
            if let Some(l) = f.limit {
                q = q.limit(l as usize);
            }
        }
        q.collect().into_iter().map(Task::from).collect()
    }

    /// Total task count in current state (ignores any filter).
    #[napi]
    pub fn count(&self) -> u32 {
        self.inner.state().read().len() as u32
    }

    /// Open a reactive watcher over the filter. Returns an iterator
    /// whose `.next()` yields the current filter result on first
    /// call, then yields again whenever a fold tick produces a
    /// different filter result (deduplicated).
    ///
    /// Declared `async` so the underlying watcher's `tokio::spawn`
    /// fold-forwarding task runs inside napi's tokio runtime.
    #[napi]
    pub async fn watch_tasks(&self, filter: Option<TaskFilter>) -> TaskWatchIter {
        let mut w = self.inner.watch();
        if let Some(f) = filter {
            if let Some(s) = f.status {
                w = w.where_status(s.into());
            }
            if let Some(s) = f.title_contains {
                w = w.title_contains(s);
            }
            if let Some(ns) = f.created_after_ns {
                w = w.created_after(bigint_u64(ns));
            }
            if let Some(ns) = f.created_before_ns {
                w = w.created_before(bigint_u64(ns));
            }
            if let Some(ns) = f.updated_after_ns {
                w = w.updated_after(bigint_u64(ns));
            }
            if let Some(ns) = f.updated_before_ns {
                w = w.updated_before(bigint_u64(ns));
            }
            if let Some(o) = f.order_by {
                w = w.order_by(o.into());
            }
            if let Some(l) = f.limit {
                w = w.limit(l as usize);
            }
        }
        let stream: BoxStream<'static, Vec<InnerTask>> = w.stream().boxed();
        TaskWatchIter {
            inner: Arc::new(TaskWatchIterInner {
                stream: TokioMutex::new(Some(stream)),
                shutdown: Notify::new(),
                is_shutdown: AtomicBool::new(false),
            }),
        }
    }
}

// =========================================================================
// Memories
// =========================================================================

/// Ordering for memory queries.
#[napi(string_enum)]
pub enum MemoriesOrderBy {
    IdAsc,
    IdDesc,
    CreatedAsc,
    CreatedDesc,
    UpdatedAsc,
    UpdatedDesc,
}

impl From<MemoriesOrderBy> for InnerMemoriesOrderBy {
    fn from(o: MemoriesOrderBy) -> Self {
        match o {
            MemoriesOrderBy::IdAsc => InnerMemoriesOrderBy::IdAsc,
            MemoriesOrderBy::IdDesc => InnerMemoriesOrderBy::IdDesc,
            MemoriesOrderBy::CreatedAsc => InnerMemoriesOrderBy::CreatedAsc,
            MemoriesOrderBy::CreatedDesc => InnerMemoriesOrderBy::CreatedDesc,
            MemoriesOrderBy::UpdatedAsc => InnerMemoriesOrderBy::UpdatedAsc,
            MemoriesOrderBy::UpdatedDesc => InnerMemoriesOrderBy::UpdatedDesc,
        }
    }
}

/// A materialized memory record.
#[napi(object)]
pub struct Memory {
    pub id: BigInt,
    pub content: String,
    pub tags: Vec<String>,
    pub source: String,
    pub created_ns: BigInt,
    pub updated_ns: BigInt,
    pub pinned: bool,
}

impl From<InnerMemory> for Memory {
    fn from(m: InnerMemory) -> Self {
        Memory {
            id: BigInt::from(m.id),
            content: m.content,
            tags: m.tags,
            source: m.source,
            created_ns: BigInt::from(m.created_ns),
            updated_ns: BigInt::from(m.updated_ns),
            pinned: m.pinned,
        }
    }
}

/// Filter for [`MemoriesAdapter::list_memories`] and
/// [`MemoriesAdapter::watch_memories`]. Tag predicates:
///
/// - `tag` — must include this exact tag.
/// - `any_tag` — must include at least one tag from the array.
/// - `all_tags` — must include every tag in the array.
#[napi(object)]
pub struct MemoryFilter {
    pub source: Option<String>,
    pub content_contains: Option<String>,
    pub tag: Option<String>,
    pub any_tag: Option<Vec<String>>,
    pub all_tags: Option<Vec<String>>,
    pub pinned: Option<bool>,
    pub created_after_ns: Option<BigInt>,
    pub created_before_ns: Option<BigInt>,
    pub updated_after_ns: Option<BigInt>,
    pub updated_before_ns: Option<BigInt>,
    pub order_by: Option<MemoriesOrderBy>,
    pub limit: Option<u32>,
}

// =========================================================================
// Memory watch iterator (napi)
// =========================================================================

struct MemoryWatchIterInner {
    stream: TokioMutex<Option<BoxStream<'static, Vec<InnerMemory>>>>,
    shutdown: Notify,
    is_shutdown: AtomicBool,
}

/// Async iterator over a live memory filter.
#[napi]
pub struct MemoryWatchIter {
    inner: Arc<MemoryWatchIterInner>,
}

#[napi]
impl MemoryWatchIter {
    /// Wait for the next filter result. Returns `null` when the
    /// iterator has been closed or the underlying stream has ended.
    #[napi]
    pub async fn next(&self) -> Option<Vec<Memory>> {
        if self.inner.is_shutdown.load(Ordering::Acquire) {
            return None;
        }
        let mut guard = self.inner.stream.lock().await;
        let stream = match guard.as_mut() {
            Some(s) => s,
            None => return None,
        };

        let shutdown_fut = self.inner.shutdown.notified();
        tokio::pin!(shutdown_fut);
        shutdown_fut.as_mut().enable();

        if self.inner.is_shutdown.load(Ordering::Acquire) {
            *guard = None;
            return None;
        }

        tokio::select! {
            biased;
            _ = shutdown_fut => {
                *guard = None;
                None
            }
            msg = stream.next() => match msg {
                Some(items) => Some(items.into_iter().map(Memory::from).collect()),
                None => {
                    *guard = None;
                    None
                }
            }
        }
    }

    /// Terminate the iterator early. Idempotent.
    #[napi]
    pub fn close(&self) {
        self.inner.is_shutdown.store(true, Ordering::Release);
        self.inner.shutdown.notify_waiters();
    }
}

/// Typed memories adapter handle.
#[napi]
pub struct MemoriesAdapter {
    inner: Arc<InnerMemoriesAdapter>,
}

#[napi]
impl MemoriesAdapter {
    /// Open the memories adapter against a Redex manager. See
    /// [`TasksAdapter::open`] for `persistent` semantics.
    #[napi(factory)]
    pub async fn open(redex: &Redex, origin_hash: u32, persistent: Option<bool>) -> Result<Self> {
        let cfg = if persistent.unwrap_or(false) {
            RedexFileConfig::default().with_persistent(true)
        } else {
            RedexFileConfig::default()
        };
        let inner = InnerMemoriesAdapter::open_with_config(&redex.inner, origin_hash, cfg)
            .map_err(|e| Error::from_reason(format!("MemoriesAdapter open failed: {}", e)))?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Store a new memory.
    #[napi]
    pub fn store(
        &self,
        id: BigInt,
        content: String,
        tags: Vec<String>,
        source: String,
        now_ns: BigInt,
    ) -> Result<BigInt> {
        self.inner
            .store(bigint_u64(id), content, tags, source, bigint_u64(now_ns))
            .map(BigInt::from)
            .map_err(|e| Error::from_reason(format!("store failed: {}", e)))
    }

    /// Replace the tag set on an existing memory. No-op at fold time
    /// if `id` is unknown.
    #[napi]
    pub fn retag(&self, id: BigInt, tags: Vec<String>, now_ns: BigInt) -> Result<BigInt> {
        self.inner
            .retag(bigint_u64(id), tags, bigint_u64(now_ns))
            .map(BigInt::from)
            .map_err(|e| Error::from_reason(format!("retag failed: {}", e)))
    }

    /// Pin a memory.
    #[napi]
    pub fn pin(&self, id: BigInt, now_ns: BigInt) -> Result<BigInt> {
        self.inner
            .pin(bigint_u64(id), bigint_u64(now_ns))
            .map(BigInt::from)
            .map_err(|e| Error::from_reason(format!("pin failed: {}", e)))
    }

    /// Unpin a memory.
    #[napi]
    pub fn unpin(&self, id: BigInt, now_ns: BigInt) -> Result<BigInt> {
        self.inner
            .unpin(bigint_u64(id), bigint_u64(now_ns))
            .map(BigInt::from)
            .map_err(|e| Error::from_reason(format!("unpin failed: {}", e)))
    }

    /// Delete a memory.
    #[napi]
    pub fn delete(&self, id: BigInt) -> Result<BigInt> {
        self.inner
            .delete(bigint_u64(id))
            .map(BigInt::from)
            .map_err(|e| Error::from_reason(format!("delete failed: {}", e)))
    }

    /// Block until every event up through `seq` has been folded.
    #[napi]
    pub async fn wait_for_seq(&self, seq: BigInt) {
        self.inner.wait_for_seq(bigint_u64(seq)).await;
    }

    /// Close the adapter. Idempotent.
    #[napi]
    pub fn close(&self) -> Result<()> {
        self.inner
            .close()
            .map_err(|e| Error::from_reason(format!("close failed: {}", e)))
    }

    /// True if the fold task is currently running.
    #[napi]
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    /// Snapshot query. See [`MemoryFilter`] for available predicates.
    #[napi]
    pub fn list_memories(&self, filter: Option<MemoryFilter>) -> Vec<Memory> {
        let state_handle = self.inner.state();
        let state = state_handle.read();
        let mut q = state.query();
        if let Some(f) = filter {
            if let Some(s) = f.source {
                q = q.where_source(s);
            }
            if let Some(s) = f.content_contains {
                q = q.content_contains(s);
            }
            if let Some(tag) = f.tag {
                q = q.where_tag(tag);
            }
            if let Some(tags) = f.any_tag {
                q = q.where_any_tag(tags);
            }
            if let Some(tags) = f.all_tags {
                q = q.where_all_tags(tags);
            }
            if let Some(pinned) = f.pinned {
                q = q.where_pinned(pinned);
            }
            if let Some(ns) = f.created_after_ns {
                q = q.created_after(bigint_u64(ns));
            }
            if let Some(ns) = f.created_before_ns {
                q = q.created_before(bigint_u64(ns));
            }
            if let Some(ns) = f.updated_after_ns {
                q = q.updated_after(bigint_u64(ns));
            }
            if let Some(ns) = f.updated_before_ns {
                q = q.updated_before(bigint_u64(ns));
            }
            if let Some(o) = f.order_by {
                q = q.order_by(o.into());
            }
            if let Some(l) = f.limit {
                q = q.limit(l as usize);
            }
        }
        q.collect().into_iter().map(Memory::from).collect()
    }

    /// Total memory count in current state (ignores any filter).
    #[napi]
    pub fn count(&self) -> u32 {
        self.inner.state().read().len() as u32
    }

    /// Open a reactive watcher over the filter. See
    /// [`TasksAdapter::watch_tasks`] for emission semantics.
    #[napi]
    pub async fn watch_memories(&self, filter: Option<MemoryFilter>) -> MemoryWatchIter {
        let mut w = self.inner.watch();
        if let Some(f) = filter {
            if let Some(s) = f.source {
                w = w.where_source(s);
            }
            if let Some(s) = f.content_contains {
                w = w.content_contains(s);
            }
            if let Some(tag) = f.tag {
                w = w.where_tag(tag);
            }
            if let Some(tags) = f.any_tag {
                w = w.where_any_tag(tags);
            }
            if let Some(tags) = f.all_tags {
                w = w.where_all_tags(tags);
            }
            if let Some(pinned) = f.pinned {
                w = w.where_pinned(pinned);
            }
            if let Some(ns) = f.created_after_ns {
                w = w.created_after(bigint_u64(ns));
            }
            if let Some(ns) = f.created_before_ns {
                w = w.created_before(bigint_u64(ns));
            }
            if let Some(ns) = f.updated_after_ns {
                w = w.updated_after(bigint_u64(ns));
            }
            if let Some(ns) = f.updated_before_ns {
                w = w.updated_before(bigint_u64(ns));
            }
            if let Some(o) = f.order_by {
                w = w.order_by(o.into());
            }
            if let Some(l) = f.limit {
                w = w.limit(l as usize);
            }
        }
        let stream: BoxStream<'static, Vec<InnerMemory>> = w.stream().boxed();
        MemoryWatchIter {
            inner: Arc::new(MemoryWatchIterInner {
                stream: TokioMutex::new(Some(stream)),
                shutdown: Notify::new(),
                is_shutdown: AtomicBool::new(false),
            }),
        }
    }
}
