//! Python bindings for the CortEX adapter slice — tasks + memories.
//!
//! Sync surface: every method blocks on the underlying tokio runtime
//! and releases the GIL via `py.detach()` around async waits. Watch
//! iterators use Python's native sync iterator protocol (`__iter__` /
//! `__next__` — `StopIteration` on end).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::stream::BoxStream;
use futures::StreamExt;
use pyo3::exceptions::{PyRuntimeError, PyStopIteration, PyValueError};
use pyo3::prelude::*;
use tokio::runtime::Runtime;
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

fn make_runtime() -> PyResult<Arc<Runtime>> {
    Runtime::new()
        .map(Arc::new)
        .map_err(|e| PyRuntimeError::new_err(format!("failed to create tokio runtime: {}", e)))
}

fn parse_task_status(s: &str) -> PyResult<InnerTaskStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Ok(InnerTaskStatus::Pending),
        "completed" => Ok(InnerTaskStatus::Completed),
        other => Err(PyValueError::new_err(format!(
            "invalid status {:?} (expected 'pending' or 'completed')",
            other
        ))),
    }
}

fn task_status_str(s: InnerTaskStatus) -> &'static str {
    match s {
        InnerTaskStatus::Pending => "pending",
        InnerTaskStatus::Completed => "completed",
    }
}

fn parse_tasks_order_by(s: &str) -> PyResult<InnerTasksOrderBy> {
    match s.to_lowercase().as_str() {
        "id_asc" => Ok(InnerTasksOrderBy::IdAsc),
        "id_desc" => Ok(InnerTasksOrderBy::IdDesc),
        "created_asc" => Ok(InnerTasksOrderBy::CreatedAsc),
        "created_desc" => Ok(InnerTasksOrderBy::CreatedDesc),
        "updated_asc" => Ok(InnerTasksOrderBy::UpdatedAsc),
        "updated_desc" => Ok(InnerTasksOrderBy::UpdatedDesc),
        other => Err(PyValueError::new_err(format!(
            "invalid order_by {:?} (expected one of id_asc|id_desc|created_asc|created_desc|updated_asc|updated_desc)",
            other
        ))),
    }
}

fn parse_memories_order_by(s: &str) -> PyResult<InnerMemoriesOrderBy> {
    match s.to_lowercase().as_str() {
        "id_asc" => Ok(InnerMemoriesOrderBy::IdAsc),
        "id_desc" => Ok(InnerMemoriesOrderBy::IdDesc),
        "created_asc" => Ok(InnerMemoriesOrderBy::CreatedAsc),
        "created_desc" => Ok(InnerMemoriesOrderBy::CreatedDesc),
        "updated_asc" => Ok(InnerMemoriesOrderBy::UpdatedAsc),
        "updated_desc" => Ok(InnerMemoriesOrderBy::UpdatedDesc),
        other => Err(PyValueError::new_err(format!(
            "invalid order_by {:?}",
            other
        ))),
    }
}

// =========================================================================
// Redex manager
// =========================================================================

/// Local RedEX manager. One handle shared across all adapters on
/// this node.
///
/// `persistent_dir`: if provided, files opened through adapters with
/// `persistent=True` write to `<persistent_dir>/<channel_path>/{idx,dat}`
/// and replay from those files on reopen. Heap-only when `None`.
#[pyclass(name = "Redex")]
pub struct PyRedex {
    inner: Arc<InnerRedex>,
    persistent_dir: Option<String>,
}

#[pymethods]
impl PyRedex {
    #[new]
    #[pyo3(signature = (persistent_dir = None))]
    fn new(persistent_dir: Option<String>) -> Self {
        let inner = match &persistent_dir {
            Some(dir) => InnerRedex::new().with_persistent_dir(dir),
            None => InnerRedex::new(),
        };
        Self {
            inner: Arc::new(inner),
            persistent_dir,
        }
    }

    fn __repr__(&self) -> String {
        match &self.persistent_dir {
            Some(dir) => format!("Redex(persistent_dir={:?})", dir),
            None => "Redex(local)".into(),
        }
    }
}

// =========================================================================
// Tasks
// =========================================================================

/// A materialized task record.
#[pyclass(name = "Task")]
#[derive(Clone)]
pub struct PyTask {
    #[pyo3(get)]
    pub id: u64,
    #[pyo3(get)]
    pub title: String,
    #[pyo3(get)]
    pub status: String,
    #[pyo3(get)]
    pub created_ns: u64,
    #[pyo3(get)]
    pub updated_ns: u64,
}

impl From<InnerTask> for PyTask {
    fn from(t: InnerTask) -> Self {
        PyTask {
            id: t.id,
            title: t.title,
            status: task_status_str(t.status).into(),
            created_ns: t.created_ns,
            updated_ns: t.updated_ns,
        }
    }
}

#[pymethods]
impl PyTask {
    fn __repr__(&self) -> String {
        format!(
            "Task(id={}, title={:?}, status={:?}, created_ns={}, updated_ns={})",
            self.id, self.title, self.status, self.created_ns, self.updated_ns
        )
    }
}

/// Typed tasks adapter handle.
#[pyclass(name = "TasksAdapter")]
pub struct PyTasksAdapter {
    inner: Arc<InnerTasksAdapter>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyTasksAdapter {
    /// Open the tasks adapter against a Redex manager.
    ///
    /// `persistent`: if `True`, the file writes to disk under the
    /// Redex's configured `persistent_dir` and replays from disk on
    /// reopen. Requires the Redex to have been constructed with
    /// `persistent_dir`; otherwise raises `RuntimeError`.
    #[staticmethod]
    #[pyo3(signature = (redex, origin_hash, persistent = false))]
    fn open(redex: &PyRedex, origin_hash: u32, persistent: bool) -> PyResult<Self> {
        let runtime = make_runtime()?;
        let redex_inner = redex.inner.clone();
        let cfg = if persistent {
            RedexFileConfig::default().with_persistent(true)
        } else {
            RedexFileConfig::default()
        };
        let inner = runtime
            .block_on(
                async move { InnerTasksAdapter::open_with_config(&redex_inner, origin_hash, cfg) },
            )
            .map_err(|e| PyRuntimeError::new_err(format!("TasksAdapter open failed: {}", e)))?;
        Ok(Self {
            inner: Arc::new(inner),
            runtime,
        })
    }

    /// Create a new task. Returns the RedEX sequence.
    fn create(&self, id: u64, title: String, now_ns: u64) -> PyResult<u64> {
        self.inner
            .create(id, title, now_ns)
            .map_err(|e| PyRuntimeError::new_err(format!("create failed: {}", e)))
    }

    /// Rename an existing task. No-op at fold time if `id` is unknown.
    fn rename(&self, id: u64, new_title: String, now_ns: u64) -> PyResult<u64> {
        self.inner
            .rename(id, new_title, now_ns)
            .map_err(|e| PyRuntimeError::new_err(format!("rename failed: {}", e)))
    }

    /// Mark a task completed.
    fn complete(&self, id: u64, now_ns: u64) -> PyResult<u64> {
        self.inner
            .complete(id, now_ns)
            .map_err(|e| PyRuntimeError::new_err(format!("complete failed: {}", e)))
    }

    /// Delete a task.
    fn delete(&self, id: u64) -> PyResult<u64> {
        self.inner
            .delete(id)
            .map_err(|e| PyRuntimeError::new_err(format!("delete failed: {}", e)))
    }

    /// Block until every event up through `seq` has been folded.
    /// Releases the GIL for the duration of the wait.
    fn wait_for_seq(&self, py: Python<'_>, seq: u64) {
        let inner = self.inner.clone();
        let runtime = self.runtime.clone();
        py.detach(|| {
            runtime.block_on(async move { inner.wait_for_seq(seq).await });
        });
    }

    /// Close the adapter. Idempotent.
    fn close(&self) -> PyResult<()> {
        self.inner
            .close()
            .map_err(|e| PyRuntimeError::new_err(format!("close failed: {}", e)))
    }

    /// True if the fold task is currently running.
    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    /// Total task count (ignores filters).
    fn count(&self) -> u32 {
        self.inner.state().read().len() as u32
    }

    /// Snapshot query. All filter args are keyword-only.
    #[pyo3(signature = (
        *,
        status=None,
        title_contains=None,
        created_after_ns=None,
        created_before_ns=None,
        updated_after_ns=None,
        updated_before_ns=None,
        order_by=None,
        limit=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn list_tasks(
        &self,
        status: Option<&str>,
        title_contains: Option<String>,
        created_after_ns: Option<u64>,
        created_before_ns: Option<u64>,
        updated_after_ns: Option<u64>,
        updated_before_ns: Option<u64>,
        order_by: Option<&str>,
        limit: Option<u32>,
    ) -> PyResult<Vec<PyTask>> {
        let state_handle = self.inner.state();
        let state = state_handle.read();
        let mut q = state.query();
        if let Some(s) = status {
            q = q.where_status(parse_task_status(s)?);
        }
        if let Some(s) = title_contains {
            q = q.title_contains(s);
        }
        if let Some(ns) = created_after_ns {
            q = q.created_after(ns);
        }
        if let Some(ns) = created_before_ns {
            q = q.created_before(ns);
        }
        if let Some(ns) = updated_after_ns {
            q = q.updated_after(ns);
        }
        if let Some(ns) = updated_before_ns {
            q = q.updated_before(ns);
        }
        if let Some(o) = order_by {
            q = q.order_by(parse_tasks_order_by(o)?);
        }
        if let Some(l) = limit {
            q = q.limit(l as usize);
        }
        Ok(q.collect().into_iter().map(PyTask::from).collect())
    }

    /// Open a reactive watcher. Returns a Python iterator — use with
    /// `for tasks in adapter.watch_tasks(status='pending'):`. Cancel
    /// iteration early with `iter.close()`.
    #[pyo3(signature = (
        *,
        status=None,
        title_contains=None,
        created_after_ns=None,
        created_before_ns=None,
        updated_after_ns=None,
        updated_before_ns=None,
        order_by=None,
        limit=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn watch_tasks(
        &self,
        status: Option<&str>,
        title_contains: Option<String>,
        created_after_ns: Option<u64>,
        created_before_ns: Option<u64>,
        updated_after_ns: Option<u64>,
        updated_before_ns: Option<u64>,
        order_by: Option<&str>,
        limit: Option<u32>,
    ) -> PyResult<PyTaskWatchIter> {
        let mut w = self.inner.watch();
        if let Some(s) = status {
            w = w.where_status(parse_task_status(s)?);
        }
        if let Some(s) = title_contains {
            w = w.title_contains(s);
        }
        if let Some(ns) = created_after_ns {
            w = w.created_after(ns);
        }
        if let Some(ns) = created_before_ns {
            w = w.created_before(ns);
        }
        if let Some(ns) = updated_after_ns {
            w = w.updated_after(ns);
        }
        if let Some(ns) = updated_before_ns {
            w = w.updated_before(ns);
        }
        if let Some(o) = order_by {
            w = w.order_by(parse_tasks_order_by(o)?);
        }
        if let Some(l) = limit {
            w = w.limit(l as usize);
        }
        // `stream()` requires an active tokio runtime (it spawns a
        // forwarding task); run via block_on to install the context.
        let runtime = self.runtime.clone();
        let stream: BoxStream<'static, Vec<InnerTask>> =
            runtime.block_on(async move { w.stream().boxed() });
        Ok(PyTaskWatchIter {
            inner: Arc::new(TaskWatchIterInner {
                stream: TokioMutex::new(Some(stream)),
                shutdown: Notify::new(),
                is_shutdown: AtomicBool::new(false),
            }),
            runtime: self.runtime.clone(),
        })
    }
}

struct TaskWatchIterInner {
    stream: TokioMutex<Option<BoxStream<'static, Vec<InnerTask>>>>,
    shutdown: Notify,
    is_shutdown: AtomicBool,
}

/// Sync Python iterator over a live task filter. `__next__` blocks
/// on the underlying stream and raises `StopIteration` when the
/// iterator is closed or the stream ends.
#[pyclass(name = "TaskWatchIter")]
pub struct PyTaskWatchIter {
    inner: Arc<TaskWatchIterInner>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyTaskWatchIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<Vec<PyTask>> {
        let inner = self.inner.clone();
        let runtime = self.runtime.clone();
        let result = py.detach(move || {
            runtime.block_on(async move {
                if inner.is_shutdown.load(Ordering::Acquire) {
                    return None;
                }
                let mut guard = inner.stream.lock().await;
                let stream = guard.as_mut()?;

                let shutdown_fut = inner.shutdown.notified();
                tokio::pin!(shutdown_fut);
                shutdown_fut.as_mut().enable();

                if inner.is_shutdown.load(Ordering::Acquire) {
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
                        Some(items) => Some(items),
                        None => {
                            *guard = None;
                            None
                        }
                    }
                }
            })
        });
        match result {
            Some(items) => Ok(items.into_iter().map(PyTask::from).collect()),
            None => Err(PyStopIteration::new_err(())),
        }
    }

    /// Terminate the iterator. Subsequent `__next__` raises
    /// `StopIteration`. Idempotent.
    fn close(&self) {
        self.inner.is_shutdown.store(true, Ordering::Release);
        self.inner.shutdown.notify_waiters();
    }
}

// =========================================================================
// Memories
// =========================================================================

/// A materialized memory record.
#[pyclass(name = "Memory")]
#[derive(Clone)]
pub struct PyMemory {
    #[pyo3(get)]
    pub id: u64,
    #[pyo3(get)]
    pub content: String,
    #[pyo3(get)]
    pub tags: Vec<String>,
    #[pyo3(get)]
    pub source: String,
    #[pyo3(get)]
    pub created_ns: u64,
    #[pyo3(get)]
    pub updated_ns: u64,
    #[pyo3(get)]
    pub pinned: bool,
}

impl From<InnerMemory> for PyMemory {
    fn from(m: InnerMemory) -> Self {
        PyMemory {
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

#[pymethods]
impl PyMemory {
    fn __repr__(&self) -> String {
        format!(
            "Memory(id={}, content={:?}, tags={:?}, source={:?}, pinned={}, created_ns={}, updated_ns={})",
            self.id,
            self.content,
            self.tags,
            self.source,
            self.pinned,
            self.created_ns,
            self.updated_ns
        )
    }
}

/// Typed memories adapter handle.
#[pyclass(name = "MemoriesAdapter")]
pub struct PyMemoriesAdapter {
    inner: Arc<InnerMemoriesAdapter>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyMemoriesAdapter {
    /// Open the memories adapter against a Redex manager. See
    /// `TasksAdapter.open` for `persistent` semantics.
    #[staticmethod]
    #[pyo3(signature = (redex, origin_hash, persistent = false))]
    fn open(redex: &PyRedex, origin_hash: u32, persistent: bool) -> PyResult<Self> {
        let runtime = make_runtime()?;
        let redex_inner = redex.inner.clone();
        let cfg = if persistent {
            RedexFileConfig::default().with_persistent(true)
        } else {
            RedexFileConfig::default()
        };
        let inner = runtime
            .block_on(async move {
                InnerMemoriesAdapter::open_with_config(&redex_inner, origin_hash, cfg)
            })
            .map_err(|e| PyRuntimeError::new_err(format!("MemoriesAdapter open failed: {}", e)))?;
        Ok(Self {
            inner: Arc::new(inner),
            runtime,
        })
    }

    #[pyo3(signature = (id, content, tags, source, now_ns))]
    fn store(
        &self,
        id: u64,
        content: String,
        tags: Vec<String>,
        source: String,
        now_ns: u64,
    ) -> PyResult<u64> {
        self.inner
            .store(id, content, tags, source, now_ns)
            .map_err(|e| PyRuntimeError::new_err(format!("store failed: {}", e)))
    }

    fn retag(&self, id: u64, tags: Vec<String>, now_ns: u64) -> PyResult<u64> {
        self.inner
            .retag(id, tags, now_ns)
            .map_err(|e| PyRuntimeError::new_err(format!("retag failed: {}", e)))
    }

    fn pin(&self, id: u64, now_ns: u64) -> PyResult<u64> {
        self.inner
            .pin(id, now_ns)
            .map_err(|e| PyRuntimeError::new_err(format!("pin failed: {}", e)))
    }

    fn unpin(&self, id: u64, now_ns: u64) -> PyResult<u64> {
        self.inner
            .unpin(id, now_ns)
            .map_err(|e| PyRuntimeError::new_err(format!("unpin failed: {}", e)))
    }

    fn delete(&self, id: u64) -> PyResult<u64> {
        self.inner
            .delete(id)
            .map_err(|e| PyRuntimeError::new_err(format!("delete failed: {}", e)))
    }

    fn wait_for_seq(&self, py: Python<'_>, seq: u64) {
        let inner = self.inner.clone();
        let runtime = self.runtime.clone();
        py.detach(|| {
            runtime.block_on(async move { inner.wait_for_seq(seq).await });
        });
    }

    fn close(&self) -> PyResult<()> {
        self.inner
            .close()
            .map_err(|e| PyRuntimeError::new_err(format!("close failed: {}", e)))
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    fn count(&self) -> u32 {
        self.inner.state().read().len() as u32
    }

    #[pyo3(signature = (
        *,
        source=None,
        content_contains=None,
        tag=None,
        any_tag=None,
        all_tags=None,
        pinned=None,
        created_after_ns=None,
        created_before_ns=None,
        updated_after_ns=None,
        updated_before_ns=None,
        order_by=None,
        limit=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn list_memories(
        &self,
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
        order_by: Option<&str>,
        limit: Option<u32>,
    ) -> PyResult<Vec<PyMemory>> {
        let state_handle = self.inner.state();
        let state = state_handle.read();
        let mut q = state.query();
        if let Some(s) = source {
            q = q.where_source(s);
        }
        if let Some(s) = content_contains {
            q = q.content_contains(s);
        }
        if let Some(t) = tag {
            q = q.where_tag(t);
        }
        if let Some(tags) = any_tag {
            q = q.where_any_tag(tags);
        }
        if let Some(tags) = all_tags {
            q = q.where_all_tags(tags);
        }
        if let Some(p) = pinned {
            q = q.where_pinned(p);
        }
        if let Some(ns) = created_after_ns {
            q = q.created_after(ns);
        }
        if let Some(ns) = created_before_ns {
            q = q.created_before(ns);
        }
        if let Some(ns) = updated_after_ns {
            q = q.updated_after(ns);
        }
        if let Some(ns) = updated_before_ns {
            q = q.updated_before(ns);
        }
        if let Some(o) = order_by {
            q = q.order_by(parse_memories_order_by(o)?);
        }
        if let Some(l) = limit {
            q = q.limit(l as usize);
        }
        Ok(q.collect().into_iter().map(PyMemory::from).collect())
    }

    #[pyo3(signature = (
        *,
        source=None,
        content_contains=None,
        tag=None,
        any_tag=None,
        all_tags=None,
        pinned=None,
        created_after_ns=None,
        created_before_ns=None,
        updated_after_ns=None,
        updated_before_ns=None,
        order_by=None,
        limit=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn watch_memories(
        &self,
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
        order_by: Option<&str>,
        limit: Option<u32>,
    ) -> PyResult<PyMemoryWatchIter> {
        let mut w = self.inner.watch();
        if let Some(s) = source {
            w = w.where_source(s);
        }
        if let Some(s) = content_contains {
            w = w.content_contains(s);
        }
        if let Some(t) = tag {
            w = w.where_tag(t);
        }
        if let Some(tags) = any_tag {
            w = w.where_any_tag(tags);
        }
        if let Some(tags) = all_tags {
            w = w.where_all_tags(tags);
        }
        if let Some(p) = pinned {
            w = w.where_pinned(p);
        }
        if let Some(ns) = created_after_ns {
            w = w.created_after(ns);
        }
        if let Some(ns) = created_before_ns {
            w = w.created_before(ns);
        }
        if let Some(ns) = updated_after_ns {
            w = w.updated_after(ns);
        }
        if let Some(ns) = updated_before_ns {
            w = w.updated_before(ns);
        }
        if let Some(o) = order_by {
            w = w.order_by(parse_memories_order_by(o)?);
        }
        if let Some(l) = limit {
            w = w.limit(l as usize);
        }
        let runtime = self.runtime.clone();
        let stream: BoxStream<'static, Vec<InnerMemory>> =
            runtime.block_on(async move { w.stream().boxed() });
        Ok(PyMemoryWatchIter {
            inner: Arc::new(MemoryWatchIterInner {
                stream: TokioMutex::new(Some(stream)),
                shutdown: Notify::new(),
                is_shutdown: AtomicBool::new(false),
            }),
            runtime: self.runtime.clone(),
        })
    }
}

struct MemoryWatchIterInner {
    stream: TokioMutex<Option<BoxStream<'static, Vec<InnerMemory>>>>,
    shutdown: Notify,
    is_shutdown: AtomicBool,
}

/// Sync Python iterator over a live memory filter.
#[pyclass(name = "MemoryWatchIter")]
pub struct PyMemoryWatchIter {
    inner: Arc<MemoryWatchIterInner>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyMemoryWatchIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<Vec<PyMemory>> {
        let inner = self.inner.clone();
        let runtime = self.runtime.clone();
        let result = py.detach(move || {
            runtime.block_on(async move {
                if inner.is_shutdown.load(Ordering::Acquire) {
                    return None;
                }
                let mut guard = inner.stream.lock().await;
                let stream = guard.as_mut()?;

                let shutdown_fut = inner.shutdown.notified();
                tokio::pin!(shutdown_fut);
                shutdown_fut.as_mut().enable();

                if inner.is_shutdown.load(Ordering::Acquire) {
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
                        Some(items) => Some(items),
                        None => {
                            *guard = None;
                            None
                        }
                    }
                }
            })
        });
        match result {
            Some(items) => Ok(items.into_iter().map(PyMemory::from).collect()),
            None => Err(PyStopIteration::new_err(())),
        }
    }

    fn close(&self) {
        self.inner.is_shutdown.store(true, Ordering::Release);
        self.inner.shutdown.notify_waiters();
    }
}
