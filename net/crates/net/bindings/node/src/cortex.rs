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

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use ::net::adapter::net::cortex::memories::{
    MemoriesAdapter as InnerMemoriesAdapter, Memory as InnerMemory, OrderBy as InnerMemoriesOrderBy,
};
use ::net::adapter::net::cortex::tasks::{
    OrderBy as InnerTasksOrderBy, Task as InnerTask, TaskStatus as InnerTaskStatus,
    TasksAdapter as InnerTasksAdapter,
};
use ::net::adapter::net::redex::Redex as InnerRedex;

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
    /// Open a new, local-only Redex manager. Persistent directories
    /// (`redex-disk`) are not exposed from the Node bindings yet.
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerRedex::new()),
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

/// Filter for [`TasksAdapter::list_tasks`].
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

/// Typed tasks adapter handle.
#[napi]
pub struct TasksAdapter {
    inner: Arc<InnerTasksAdapter>,
}

#[napi]
impl TasksAdapter {
    /// Open the tasks adapter against a Redex manager.
    ///
    /// Declared `async` so napi-rs runs it with its tokio runtime
    /// active — the underlying `CortexAdapter::open` spawns the
    /// fold task via `tokio::spawn` and needs a live reactor.
    #[napi(factory)]
    pub async fn open(redex: &Redex, origin_hash: u32) -> Result<Self> {
        let inner = InnerTasksAdapter::open(&redex.inner, origin_hash)
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

/// Filter for [`MemoriesAdapter::list_memories`]. Tag predicates:
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

/// Typed memories adapter handle.
#[napi]
pub struct MemoriesAdapter {
    inner: Arc<InnerMemoriesAdapter>,
}

#[napi]
impl MemoriesAdapter {
    /// Open the memories adapter against a Redex manager.
    ///
    /// Declared `async` so napi-rs runs it with its tokio runtime
    /// active — the underlying `CortexAdapter::open` spawns the
    /// fold task via `tokio::spawn` and needs a live reactor.
    #[napi(factory)]
    pub async fn open(redex: &Redex, origin_hash: u32) -> Result<Self> {
        let inner = InnerMemoriesAdapter::open(&redex.inner, origin_hash)
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
}
