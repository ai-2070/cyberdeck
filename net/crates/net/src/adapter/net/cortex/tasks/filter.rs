//! Plain-data `TasksFilter` struct for the Prisma-ish NetDB surface.
//!
//! Mirrors the filter shape used by the Node and Python SDK bindings.
//! Callers can construct one directly and pass it to
//! [`super::state::TasksState::find_many`] / `count_where` /
//! `exists_where`, or to [`super::TasksAdapter::watch_with_filter`]
//! (when added).
//!
//! For fluent chaining, use the existing [`super::query::TasksQuery`]
//! builder instead.

use super::query::{OrderBy, TasksQuery};
use super::types::TaskStatus;

/// Filter for `find_many` / `count_where` / `exists_where` over
/// [`super::state::TasksState`]. All fields are optional and
/// default to "no constraint" on that axis.
#[derive(Debug, Clone, Default)]
pub struct TasksFilter {
    /// Restrict to this status.
    pub status: Option<TaskStatus>,
    /// Case-insensitive substring match on `title`.
    pub title_contains: Option<String>,
    /// `created_ns >= ns` (inclusive).
    pub created_after_ns: Option<u64>,
    /// `created_ns <= ns` (inclusive).
    pub created_before_ns: Option<u64>,
    /// `updated_ns >= ns` (inclusive).
    pub updated_after_ns: Option<u64>,
    /// `updated_ns <= ns` (inclusive).
    pub updated_before_ns: Option<u64>,
    /// Result ordering.
    pub order_by: Option<OrderBy>,
    /// Truncate results to `limit` after ordering.
    pub limit: Option<usize>,
}

impl TasksFilter {
    /// Apply this filter's constraints to an existing query builder.
    /// Used by the convenience state methods; callers don't normally
    /// call this directly.
    pub(super) fn apply<'a>(&self, mut q: TasksQuery<'a>) -> TasksQuery<'a> {
        if let Some(s) = self.status {
            q = q.where_status(s);
        }
        if let Some(s) = &self.title_contains {
            q = q.title_contains(s.clone());
        }
        if let Some(ns) = self.created_after_ns {
            q = q.created_after(ns);
        }
        if let Some(ns) = self.created_before_ns {
            q = q.created_before(ns);
        }
        if let Some(ns) = self.updated_after_ns {
            q = q.updated_after(ns);
        }
        if let Some(ns) = self.updated_before_ns {
            q = q.updated_before(ns);
        }
        if let Some(o) = self.order_by {
            q = q.order_by(o);
        }
        if let Some(l) = self.limit {
            q = q.limit(l);
        }
        q
    }
}
