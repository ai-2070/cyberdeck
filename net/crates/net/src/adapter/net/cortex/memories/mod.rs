//! Memories — a second CortEX model built on the adapter, exercising
//! the pattern on a non-CRUD shape.
//!
//! Where [tasks](super::tasks) are a mutate-by-id CRUD store,
//! memories are a content-addressed log with set-valued tag metadata
//! and a pin toggle. That difference is what the module is for:
//! validating that the adapter pattern accommodates domains beyond
//! per-id status transitions.
//!
//! Structure mirrors the tasks module:
//!
//! - `MemoriesState` — `HashMap<MemoryId, Memory>`.
//! - `MemoriesFold` — decodes `EventMeta` + payload, routes on dispatch.
//! - `MemoriesAdapter` — typed wrapper over `CortexAdapter<MemoriesState>`.
//! - `MemoriesQuery` — fluent query with single-tag / any-tag /
//!   all-tags predicates and a pin filter.
//!
//! All events ride a single RedEX file at `cortex/memories` (see
//! [`MEMORIES_CHANNEL`]). Dispatches use the CortEX-internal range
//! `0x10..=0x14` (stored / retagged / pinned / unpinned / deleted).

mod adapter;
mod dispatch;
mod filter;
mod fold;
mod query;
mod state;
mod types;
mod watch;

pub use adapter::MemoriesAdapter;
pub use dispatch::{
    DISPATCH_MEMORY_DELETED, DISPATCH_MEMORY_PINNED, DISPATCH_MEMORY_RETAGGED,
    DISPATCH_MEMORY_STORED, DISPATCH_MEMORY_UNPINNED, MEMORIES_CHANNEL,
};
pub use filter::MemoriesFilter;
pub use fold::MemoriesFold;
pub use query::{MemoriesQuery, OrderBy};
pub use state::MemoriesState;
pub use types::{Memory, MemoryId};
pub use watch::MemoriesWatcher;
