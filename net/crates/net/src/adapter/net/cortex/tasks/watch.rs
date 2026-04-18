//! Reactive watcher over `TasksState`.
//!
//! Fluent builder mirroring [`super::query::TasksQuery`] that produces
//! a `Stream<Item = Vec<Task>>`. The stream yields the current filter
//! result on open, then yields again whenever a fold tick produces a
//! different filter result (deduplicated by `Vec<Task>` equality).
//!
//! ```ignore
//! let mut pending_stream = Box::pin(
//!     tasks.watch()
//!         .where_status(TaskStatus::Pending)
//!         .order_by(OrderBy::CreatedDesc)
//!         .stream()
//! );
//!
//! while let Some(current) = pending_stream.next().await {
//!     // `current` is the freshly-evaluated pending list.
//! }
//! ```

use std::sync::Arc;

use futures::stream::BoxStream;
use futures::{Stream, StreamExt};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::query::{OrderBy, TasksFilterSpec};
use super::state::TasksState;
use super::types::{Task, TaskId, TaskStatus};

/// Reactive filter over `TasksState`. Created via
/// [`super::TasksAdapter::watch`].
pub struct TasksWatcher {
    state: Arc<RwLock<TasksState>>,
    changes: BoxStream<'static, u64>,
    spec: TasksFilterSpec,
}

impl TasksWatcher {
    /// Build a watcher from the adapter's state handle + change stream.
    /// Intended to be called only by [`super::TasksAdapter::watch`].
    pub(super) fn new(
        state: Arc<RwLock<TasksState>>,
        changes: BoxStream<'static, u64>,
    ) -> Self {
        Self {
            state,
            changes,
            spec: TasksFilterSpec::default(),
        }
    }

    /// Restrict to tasks with the given status.
    pub fn where_status(mut self, status: TaskStatus) -> Self {
        self.spec.status = Some(status);
        self
    }

    /// Restrict to tasks whose id is in the provided collection.
    pub fn where_id_in(mut self, ids: impl IntoIterator<Item = TaskId>) -> Self {
        self.spec.id_in = Some(ids.into_iter().collect());
        self
    }

    /// Restrict to `created_ns > ns`.
    pub fn created_after(mut self, ns: u64) -> Self {
        self.spec.created_after_ns = Some(ns);
        self
    }

    /// Restrict to `created_ns < ns`.
    pub fn created_before(mut self, ns: u64) -> Self {
        self.spec.created_before_ns = Some(ns);
        self
    }

    /// Restrict to `updated_ns > ns`.
    pub fn updated_after(mut self, ns: u64) -> Self {
        self.spec.updated_after_ns = Some(ns);
        self
    }

    /// Restrict to `updated_ns < ns`.
    pub fn updated_before(mut self, ns: u64) -> Self {
        self.spec.updated_before_ns = Some(ns);
        self
    }

    /// Restrict to tasks whose title contains `needle` (case-insensitive).
    pub fn title_contains(mut self, needle: impl Into<String>) -> Self {
        self.spec.title_contains = Some(needle.into().to_lowercase());
        self
    }

    /// Order each emitted result set.
    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.spec.order_by = Some(order);
        self
    }

    /// Truncate each emitted result set to `n` after ordering.
    pub fn limit(mut self, n: usize) -> Self {
        self.spec.limit = Some(n);
        self
    }

    /// Start emitting. The stream yields:
    ///
    /// - The current filter result immediately (first element).
    /// - A new result vector on each subsequent fold tick where the
    ///   filter's result differs from the previously emitted one.
    ///
    /// The stream ends when the adapter's change stream ends (e.g.
    /// when all adapter handles drop and the fold task exits).
    pub fn stream(self) -> impl Stream<Item = Vec<Task>> + Send + 'static {
        let TasksWatcher {
            state,
            mut changes,
            spec,
        } = self;
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            // Initial emission.
            let initial = {
                let guard = state.read();
                spec.execute(&guard)
            };
            if tx.send(initial.clone()).is_err() {
                return;
            }
            let mut last = initial;

            while let Some(_seq) = changes.next().await {
                let current = {
                    let guard = state.read();
                    spec.execute(&guard)
                };
                if current != last {
                    if tx.send(current.clone()).is_err() {
                        return;
                    }
                    last = current;
                }
            }
        });

        UnboundedReceiverStream::new(rx)
    }
}
