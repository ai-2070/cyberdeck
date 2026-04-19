//! Dispatch byte constants for memory events.
//!
//! Falls in the `0x00..0x7F` range reserved for CortEX-internal
//! dispatches per the adapter plan. Values `0x10..0x1F` are allocated
//! to the memories model here.
//!
//! Current CortEX-internal dispatch allocations:
//!
//! | Range       | Model   |
//! |-------------|---------|
//! | `0x01..0x0F` | tasks  |
//! | `0x10..0x1F` | memories |

/// A memory was stored.
pub const DISPATCH_MEMORY_STORED: u8 = 0x10;
/// A memory's tag set was replaced.
pub const DISPATCH_MEMORY_RETAGGED: u8 = 0x11;
/// A memory was pinned.
pub const DISPATCH_MEMORY_PINNED: u8 = 0x12;
/// A memory was unpinned.
pub const DISPATCH_MEMORY_UNPINNED: u8 = 0x13;
/// A memory was deleted.
pub const DISPATCH_MEMORY_DELETED: u8 = 0x14;

/// Canonical channel name for the memories model.
pub const MEMORIES_CHANNEL: &str = "cortex/memories";

// Static assertions: allocated dispatches fall in CortEX's reserved
// range (0x00..0x7F).
const _: () = {
    assert!(DISPATCH_MEMORY_STORED < 0x80);
    assert!(DISPATCH_MEMORY_RETAGGED < 0x80);
    assert!(DISPATCH_MEMORY_PINNED < 0x80);
    assert!(DISPATCH_MEMORY_UNPINNED < 0x80);
    assert!(DISPATCH_MEMORY_DELETED < 0x80);
};
