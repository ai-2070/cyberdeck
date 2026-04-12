//! Consumer API for polling and filtering events.
//!
//! This module provides:
//! - JSON predicate filtering
//! - Cross-shard poll merging
//! - Cursor-based pagination
//! - Optional timestamp ordering

pub mod filter;
pub mod merge;

pub use filter::{json_path_get, Filter, FilterBuilder};
pub use merge::{CompositeCursor, ConsumeRequest, ConsumeResponse, Ordering, PollMerger};
