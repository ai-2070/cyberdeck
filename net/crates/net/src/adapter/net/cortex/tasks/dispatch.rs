//! Dispatch byte constants for task events.
//!
//! Falls in the `0x00..0x7F` range reserved for CortEX-internal
//! dispatches per the adapter plan. Values `0x01..0x0F` are allocated
//! to the tasks model here.

/// A task was created.
pub const DISPATCH_TASK_CREATED: u8 = 0x01;
/// A task's title was changed.
pub const DISPATCH_TASK_RENAMED: u8 = 0x02;
/// A task was marked completed.
pub const DISPATCH_TASK_COMPLETED: u8 = 0x03;
/// A task was deleted.
pub const DISPATCH_TASK_DELETED: u8 = 0x04;

/// Canonical channel name for the tasks model.
pub const TASKS_CHANNEL: &str = "cortex/tasks";

// Static assertions that the allocated dispatches fall in CortEX's
// reserved range (0x00..0x7F).
const _: () = {
    assert!(DISPATCH_TASK_CREATED < 0x80);
    assert!(DISPATCH_TASK_RENAMED < 0x80);
    assert!(DISPATCH_TASK_COMPLETED < 0x80);
    assert!(DISPATCH_TASK_DELETED < 0x80);
};
