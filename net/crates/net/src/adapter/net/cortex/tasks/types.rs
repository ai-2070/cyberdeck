//! Domain types and event payloads for the tasks model.

use serde::{Deserialize, Serialize};

/// Identifier for a task. Opaque `u64`; callers derive however they want
/// (sequential, hash of title, snowflake id, etc.).
pub type TaskId = u64;

/// Lifecycle state of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Created, not yet completed.
    Pending,
    /// Marked completed.
    Completed,
}

/// The canonical task record held in [`super::state::TasksState`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// Stable identifier.
    pub id: TaskId,
    /// Human-readable title.
    pub title: String,
    /// Current status.
    pub status: TaskStatus,
    /// Wall-clock creation time (unix nanos).
    pub created_ns: u64,
    /// Wall-clock last-update time (unix nanos). Equals `created_ns`
    /// until the task is renamed or completed.
    pub updated_ns: u64,
}

// ---- Event payload structs (serialized after the 20-byte EventMeta) ----

/// Payload for `DISPATCH_TASK_CREATED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct TaskCreatedPayload {
    pub id: TaskId,
    pub title: String,
    pub now_ns: u64,
}

/// Payload for `DISPATCH_TASK_RENAMED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct TaskRenamedPayload {
    pub id: TaskId,
    pub new_title: String,
    pub now_ns: u64,
}

/// Payload for `DISPATCH_TASK_COMPLETED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct TaskCompletedPayload {
    pub id: TaskId,
    pub now_ns: u64,
}

/// Payload for `DISPATCH_TASK_DELETED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct TaskDeletedPayload {
    pub id: TaskId,
}
