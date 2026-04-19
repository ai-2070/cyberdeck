//! Domain types and event payloads for the memories model.

use serde::{Deserialize, Serialize};

/// Identifier for a memory. Opaque `u64`; callers derive however they
/// want (content hash, snowflake id, sequential).
pub type MemoryId = u64;

/// A content-addressable memory record: a piece of content plus tag
/// metadata, source identity, and a pinned flag.
///
/// Memories differ structurally from [`super::super::tasks::Task`] in
/// two ways that exercise the CortEX adapter pattern:
///
/// - `tags: Vec<String>` — multi-valued field with set-like query
///   semantics (`where_tag`, `where_any_tag`, `where_all_tags`).
/// - `pinned: bool` — toggled via explicit events rather than as a
///   terminal state like `TaskStatus::Completed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    /// Stable identifier.
    pub id: MemoryId,
    /// Free-form payload. RedEX stores the raw bytes; the adapter
    /// makes no assumption about structure.
    pub content: String,
    /// Tag set. Insertion order is preserved; duplicates are not
    /// enforced as unique by the adapter (fold relies on the event
    /// stream to keep it sane).
    pub tags: Vec<String>,
    /// Producer / origin label. Free-form string chosen by callers.
    pub source: String,
    /// Wall-clock creation time (unix nanos).
    pub created_ns: u64,
    /// Wall-clock last-update time (unix nanos).
    pub updated_ns: u64,
    /// Whether the memory is pinned (callers decide what "pinned"
    /// means for their domain — typically "keep prominent in queries").
    pub pinned: bool,
}

// ---- Event payload structs (serialized after the 20-byte EventMeta) ----

/// Payload for `DISPATCH_MEMORY_STORED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MemoryStoredPayload {
    pub id: MemoryId,
    pub content: String,
    pub tags: Vec<String>,
    pub source: String,
    pub now_ns: u64,
}

/// Payload for `DISPATCH_MEMORY_RETAGGED`. Replaces the full tag set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MemoryRetaggedPayload {
    pub id: MemoryId,
    pub tags: Vec<String>,
    pub now_ns: u64,
}

/// Payload for `DISPATCH_MEMORY_PINNED` and `DISPATCH_MEMORY_UNPINNED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MemoryPinTogglePayload {
    pub id: MemoryId,
    pub now_ns: u64,
}

/// Payload for `DISPATCH_MEMORY_DELETED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MemoryDeletedPayload {
    pub id: MemoryId,
}
