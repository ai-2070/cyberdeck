//! Prisma-style query builder over `TasksState`.
//!
//! Fluent filters compose in any order. `collect` / `first` / `count`
//! terminate the chain against the live state.
//!
//! ```ignore
//! let state_handle = tasks.state();
//! let state = state_handle.read();
//!
//! let results = state.query()
//!     .where_status(TaskStatus::Pending)
//!     .created_after(cutoff_ns)
//!     .order_by(OrderBy::CreatedDesc)
//!     .limit(10)
//!     .collect();
//! ```
//!
//! The builder borrows the state read guard, so iteration sees a
//! consistent snapshot.

use std::collections::HashSet;

use super::state::TasksState;
use super::types::{Task, TaskId, TaskStatus};

/// Filter / order / limit configuration. Shared by
/// [`TasksQuery`] (immediate execution over a borrowed state snapshot)
/// and [`super::watch::TasksWatcher`] (repeated execution driven by
/// the adapter's change stream).
#[derive(Debug, Clone, Default)]
pub(super) struct TasksFilterSpec {
    pub status: Option<TaskStatus>,
    pub id_in: Option<HashSet<TaskId>>,
    pub created_after_ns: Option<u64>,
    pub created_before_ns: Option<u64>,
    pub updated_after_ns: Option<u64>,
    pub updated_before_ns: Option<u64>,
    pub title_contains: Option<String>,
    pub order_by: Option<OrderBy>,
    pub limit: Option<usize>,
}

impl TasksFilterSpec {
    /// Apply all filter predicates to a single task.
    pub(super) fn matches(&self, t: &Task) -> bool {
        if let Some(s) = self.status {
            if t.status != s {
                return false;
            }
        }
        if let Some(ids) = &self.id_in {
            if !ids.contains(&t.id) {
                return false;
            }
        }
        if let Some(ns) = self.created_after_ns {
            if t.created_ns <= ns {
                return false;
            }
        }
        if let Some(ns) = self.created_before_ns {
            if t.created_ns >= ns {
                return false;
            }
        }
        if let Some(ns) = self.updated_after_ns {
            if t.updated_ns <= ns {
                return false;
            }
        }
        if let Some(ns) = self.updated_before_ns {
            if t.updated_ns >= ns {
                return false;
            }
        }
        if let Some(needle) = &self.title_contains {
            if !t.title.to_lowercase().contains(needle) {
                return false;
            }
        }
        true
    }

    /// Collect matching tasks from state, applying order + limit.
    pub(super) fn execute(&self, state: &TasksState) -> Vec<Task> {
        let mut out: Vec<Task> = state
            .tasks
            .values()
            .filter(|t| self.matches(t))
            .cloned()
            .collect();
        if let Some(order) = self.order_by {
            sort_tasks(&mut out, order);
        }
        if let Some(limit) = self.limit {
            out.truncate(limit);
        }
        out
    }
}

/// Ordering for query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderBy {
    /// By `id`, ascending.
    IdAsc,
    /// By `id`, descending.
    IdDesc,
    /// By `created_ns`, ascending (oldest first).
    CreatedAsc,
    /// By `created_ns`, descending (newest first).
    CreatedDesc,
    /// By `updated_ns`, ascending.
    UpdatedAsc,
    /// By `updated_ns`, descending.
    UpdatedDesc,
}

/// Fluent query over `TasksState`.
///
/// Created via [`TasksState::query`].
pub struct TasksQuery<'a> {
    state: &'a TasksState,
    spec: TasksFilterSpec,
}

impl TasksState {
    /// Start a fluent query over this state snapshot.
    pub fn query(&self) -> TasksQuery<'_> {
        TasksQuery {
            state: self,
            spec: TasksFilterSpec::default(),
        }
    }
}

impl<'a> TasksQuery<'a> {
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

    /// Order results. If unset, iteration order is unspecified (hash map).
    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.spec.order_by = Some(order);
        self
    }

    /// Truncate to `n` results after ordering.
    pub fn limit(mut self, n: usize) -> Self {
        self.spec.limit = Some(n);
        self
    }

    /// Execute the query and collect matching tasks (cloned).
    pub fn collect(self) -> Vec<Task> {
        self.spec.execute(self.state)
    }

    /// Return the number of matches. Ignores `limit`.
    pub fn count(self) -> usize {
        self.state
            .tasks
            .values()
            .filter(|t| self.spec.matches(t))
            .count()
    }

    /// Return the first matching task in iteration order (after
    /// applying `order_by` if set).
    pub fn first(mut self) -> Option<Task> {
        // Force a limit of 1 but still respect ordering.
        self.spec.limit = Some(1);
        self.collect().into_iter().next()
    }

    /// True if any task matches. Short-circuits on first hit.
    pub fn exists(self) -> bool {
        self.state.tasks.values().any(|t| self.spec.matches(t))
    }
}

pub(super) fn sort_tasks(tasks: &mut [Task], order: OrderBy) {
    match order {
        OrderBy::IdAsc => tasks.sort_by_key(|t| t.id),
        OrderBy::IdDesc => tasks.sort_by_key(|t| std::cmp::Reverse(t.id)),
        OrderBy::CreatedAsc => tasks.sort_by_key(|t| t.created_ns),
        OrderBy::CreatedDesc => tasks.sort_by_key(|t| std::cmp::Reverse(t.created_ns)),
        OrderBy::UpdatedAsc => tasks.sort_by_key(|t| t.updated_ns),
        OrderBy::UpdatedDesc => tasks.sort_by_key(|t| std::cmp::Reverse(t.updated_ns)),
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{Task, TaskStatus};
    use super::*;

    fn mk(id: TaskId, title: &str, status: TaskStatus, created: u64, updated: u64) -> Task {
        Task {
            id,
            title: title.to_string(),
            status,
            created_ns: created,
            updated_ns: updated,
        }
    }

    fn state_with(tasks: impl IntoIterator<Item = Task>) -> TasksState {
        let mut s = TasksState::new();
        for t in tasks {
            s.tasks.insert(t.id, t);
        }
        s
    }

    fn sample() -> TasksState {
        state_with([
            mk(1, "Write plan", TaskStatus::Pending, 100, 100),
            mk(2, "Ship adapter", TaskStatus::Completed, 200, 250),
            mk(3, "Review PR", TaskStatus::Pending, 300, 310),
            mk(4, "Update docs", TaskStatus::Pending, 400, 410),
            mk(5, "Deploy v1", TaskStatus::Completed, 500, 520),
        ])
    }

    #[test]
    fn test_no_filters_returns_all() {
        let s = sample();
        assert_eq!(s.query().count(), 5);
    }

    #[test]
    fn test_where_status_pending() {
        let s = sample();
        let mut ids: Vec<_> = s
            .query()
            .where_status(TaskStatus::Pending)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        ids.sort();
        assert_eq!(ids, vec![1, 3, 4]);
    }

    #[test]
    fn test_where_id_in() {
        let s = sample();
        let mut ids: Vec<_> = s
            .query()
            .where_id_in([2, 4, 99])
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        ids.sort();
        assert_eq!(ids, vec![2, 4]);
    }

    #[test]
    fn test_created_after() {
        let s = sample();
        let mut ids: Vec<_> = s
            .query()
            .created_after(300)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        ids.sort();
        assert_eq!(ids, vec![4, 5]);
    }

    #[test]
    fn test_created_before() {
        let s = sample();
        let mut ids: Vec<_> = s
            .query()
            .created_before(300)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        ids.sort();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn test_updated_after_and_before() {
        let s = sample();
        let mut ids: Vec<_> = s
            .query()
            .updated_after(250)
            .updated_before(500)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        ids.sort();
        // 3 (310), 4 (410) pass. 5 (520) is not strictly before 500. 2 (250) is not strictly after 250.
        assert_eq!(ids, vec![3, 4]);
    }

    #[test]
    fn test_title_contains_case_insensitive() {
        let s = sample();
        let mut ids: Vec<_> = s
            .query()
            .title_contains("DEPLOY")
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        ids.sort();
        assert_eq!(ids, vec![5]);

        let ids_plural: Vec<_> = s.query().title_contains("e").collect();
        // All titles contain "e" (Write, adapter, Review, update, Deploy).
        assert_eq!(ids_plural.len(), 5);
    }

    #[test]
    fn test_order_by_id_asc_desc() {
        let s = sample();
        let asc: Vec<_> = s
            .query()
            .order_by(OrderBy::IdAsc)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(asc, vec![1, 2, 3, 4, 5]);

        let desc: Vec<_> = s
            .query()
            .order_by(OrderBy::IdDesc)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(desc, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_order_by_created() {
        let s = sample();
        let asc: Vec<_> = s
            .query()
            .order_by(OrderBy::CreatedAsc)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(asc, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_order_by_updated_desc() {
        let s = sample();
        let desc: Vec<_> = s
            .query()
            .order_by(OrderBy::UpdatedDesc)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        // updated_ns: 100, 250, 310, 410, 520 → desc order → ids 5, 4, 3, 2, 1
        assert_eq!(desc, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_limit_truncates_after_order() {
        let s = sample();
        let top2: Vec<_> = s
            .query()
            .order_by(OrderBy::CreatedDesc)
            .limit(2)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(top2, vec![5, 4]);
    }

    #[test]
    fn test_composed_filters() {
        let s = sample();
        // Pending tasks created after 200, ordered by id ascending.
        let ids: Vec<_> = s
            .query()
            .where_status(TaskStatus::Pending)
            .created_after(200)
            .order_by(OrderBy::IdAsc)
            .collect()
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec![3, 4]);
    }

    #[test]
    fn test_first_returns_ordered_head() {
        let s = sample();
        let first = s
            .query()
            .where_status(TaskStatus::Pending)
            .order_by(OrderBy::CreatedDesc)
            .first()
            .unwrap();
        // Pending: 1, 3, 4 → created_ns 100, 300, 400 → desc head is id 4.
        assert_eq!(first.id, 4);
    }

    #[test]
    fn test_first_none_when_no_match() {
        let s = sample();
        assert!(s.query().title_contains("unicorn").first().is_none());
    }

    #[test]
    fn test_count_ignores_limit() {
        let s = sample();
        let q = s.query().where_status(TaskStatus::Pending).limit(1);
        // 3 pending tasks total; limit does not affect count.
        assert_eq!(q.count(), 3);
    }

    #[test]
    fn test_exists_short_circuits() {
        let s = sample();
        assert!(s.query().where_status(TaskStatus::Completed).exists());
        assert!(!s.query().title_contains("unicorn").exists());
    }

    #[test]
    fn test_empty_state_queries_return_empty() {
        let s = TasksState::new();
        assert_eq!(s.query().count(), 0);
        assert!(s.query().first().is_none());
        assert!(!s.query().exists());
    }
}
