//! Tasks — the first CortEX model built on top of the CortEX adapter.
//!
//! Demonstrates the full three-way handoff:
//!
//! - `TasksState` — materialized HashMap of tasks (CortEX-owned state).
//! - `TasksFold` — decodes `EventMeta` + payload, routes by dispatch,
//!   mutates state.
//! - `TasksAdapter` — typed wrapper over `CortexAdapter<TasksState>`
//!   with domain-level ingest helpers (`create` / `rename` /
//!   `complete` / `delete`).
//!
//! All events ride a single RedEX file at `cortex/tasks` (see
//! [`TASKS_CHANNEL`]). Dispatches use the CortEX-internal range
//! `0x01..=0x04` (created / renamed / completed / deleted).
//!
//! # Example
//!
//! ```ignore
//! use net::adapter::net::redex::Redex;
//! use net::adapter::net::cortex::tasks::TasksAdapter;
//!
//! # async fn demo(now_ns: u64) -> Result<(), Box<dyn std::error::Error>> {
//! let redex = Redex::new();
//! let tasks = TasksAdapter::open(&redex, 0xABCD_EF01)?;
//!
//! let seq = tasks.create(1, "write the plan", now_ns)?;
//! tasks.wait_for_seq(seq).await;
//!
//! let state = tasks.state();
//! let guard = state.read();
//! assert_eq!(guard.len(), 1);
//! # Ok(()) }
//! ```

mod adapter;
mod dispatch;
mod filter;
mod fold;
mod query;
mod state;
mod types;
mod watch;

pub use adapter::TasksAdapter;
pub use dispatch::{
    DISPATCH_TASK_COMPLETED, DISPATCH_TASK_CREATED, DISPATCH_TASK_DELETED, DISPATCH_TASK_RENAMED,
    TASKS_CHANNEL,
};
pub use filter::TasksFilter;
pub use fold::TasksFold;
pub use query::{OrderBy, TasksQuery};
pub use state::TasksState;
pub use types::{Task, TaskId, TaskStatus};
pub use watch::TasksWatcher;
