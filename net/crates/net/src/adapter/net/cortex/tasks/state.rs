//! `TasksState` — the materialized view held behind the
//! `CortexAdapter<TasksState>`'s `RwLock`.

use std::collections::HashMap;

use super::types::{Task, TaskId, TaskStatus};

/// Materialized view over the tasks log.
#[derive(Debug, Default, Clone)]
pub struct TasksState {
    pub(super) tasks: HashMap<TaskId, Task>,
}

impl TasksState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a task by id.
    pub fn get(&self, id: TaskId) -> Option<&Task> {
        self.tasks.get(&id)
    }

    /// Total number of tasks currently retained.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// True if no tasks are retained.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// True if a task with `id` exists.
    pub fn contains(&self, id: TaskId) -> bool {
        self.tasks.contains_key(&id)
    }

    /// Iterate over every retained task.
    pub fn all(&self) -> impl Iterator<Item = &Task> {
        self.tasks.values()
    }

    /// Iterate over tasks currently `Pending`.
    pub fn pending(&self) -> impl Iterator<Item = &Task> {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Pending)
    }

    /// Iterate over tasks currently `Completed`.
    pub fn completed(&self) -> impl Iterator<Item = &Task> {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: TaskId, status: TaskStatus) -> Task {
        Task {
            id,
            title: format!("task-{}", id),
            status,
            created_ns: 0,
            updated_ns: 0,
        }
    }

    #[test]
    fn test_empty_state() {
        let s = TasksState::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert!(s.get(1).is_none());
        assert!(!s.contains(1));
    }

    #[test]
    fn test_queries_filter_by_status() {
        let mut s = TasksState::new();
        s.tasks.insert(1, task(1, TaskStatus::Pending));
        s.tasks.insert(2, task(2, TaskStatus::Pending));
        s.tasks.insert(3, task(3, TaskStatus::Completed));

        assert_eq!(s.len(), 3);
        assert_eq!(s.pending().count(), 2);
        assert_eq!(s.completed().count(), 1);
        assert_eq!(s.all().count(), 3);
        assert!(s.contains(2));
        assert!(!s.contains(99));
    }
}
