//! `MemoriesFold` — decodes `EventMeta` + payload, routes on dispatch,
//! mutates [`super::state::MemoriesState`].

use super::super::super::redex::{RedexError, RedexEvent, RedexFold};
use super::super::meta::{compute_checksum, EventMeta, EVENT_META_SIZE};
use super::dispatch::{
    DISPATCH_MEMORY_DELETED, DISPATCH_MEMORY_PINNED, DISPATCH_MEMORY_RETAGGED,
    DISPATCH_MEMORY_STORED, DISPATCH_MEMORY_UNPINNED,
};
use super::state::MemoriesState;
use super::types::{
    Memory, MemoryDeletedPayload, MemoryPinTogglePayload, MemoryRetaggedPayload,
    MemoryStoredPayload,
};

/// Fold implementation for the memories model.
pub struct MemoriesFold;

impl RedexFold<MemoriesState> for MemoriesFold {
    fn apply(&mut self, ev: &RedexEvent, state: &mut MemoriesState) -> Result<(), RedexError> {
        if ev.payload.len() < EVENT_META_SIZE {
            return Err(RedexError::Encode(format!(
                "memories payload too short: {} bytes (need >= {})",
                ev.payload.len(),
                EVENT_META_SIZE
            )));
        }
        let meta = EventMeta::from_bytes(&ev.payload[..EVENT_META_SIZE])
            .ok_or_else(|| RedexError::Encode("bad EventMeta prefix".into()))?;
        let tail = &ev.payload[EVENT_META_SIZE..];

        // Verify the checksum stamped at ingest against the tail we
        // received from RedEX. Catches disk corruption, tampered
        // on-disk files, and truncated tails. Under
        // `FoldErrorPolicy::Stop` this halts the fold task; under
        // `LogAndContinue` the event is counted and skipped.
        let expected = compute_checksum(tail);
        if meta.checksum != expected {
            return Err(RedexError::Encode(format!(
                "memories fold: EventMeta checksum mismatch at seq {} (got {:#010x}, tail hashes to {:#010x})",
                ev.entry.seq, meta.checksum, expected
            )));
        }

        match meta.dispatch {
            DISPATCH_MEMORY_STORED => {
                let p: MemoryStoredPayload =
                    postcard::from_bytes(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                // BUG #115: pre-fix this constructed a fresh
                // `Memory { pinned: false, created_ns: p.now_ns,
                // ... }` and `insert`ed it, silently replacing
                // any existing entry. So `memories.store(42,
                // "updated", ...)` after `memories.pin(42)`
                // dropped the pin flag and overwrote the
                // original creation timestamp — operator had no
                // observable signal. Treat STORED as a content-
                // update for an existing id: preserve `pinned`
                // and `created_ns`, advance `updated_ns`, and
                // overwrite the rest.
                if let Some(existing) = state.memories.get_mut(&p.id) {
                    existing.content = p.content;
                    existing.tags = p.tags;
                    existing.source = p.source;
                    existing.updated_ns = p.now_ns;
                    // pinned + created_ns intentionally preserved.
                } else {
                    state.memories.insert(
                        p.id,
                        Memory {
                            id: p.id,
                            content: p.content,
                            tags: p.tags,
                            source: p.source,
                            created_ns: p.now_ns,
                            updated_ns: p.now_ns,
                            pinned: false,
                        },
                    );
                }
            }
            DISPATCH_MEMORY_RETAGGED => {
                let p: MemoryRetaggedPayload =
                    postcard::from_bytes(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                if let Some(m) = state.memories.get_mut(&p.id) {
                    m.tags = p.tags;
                    m.updated_ns = p.now_ns;
                }
            }
            DISPATCH_MEMORY_PINNED => {
                let p: MemoryPinTogglePayload =
                    postcard::from_bytes(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                if let Some(m) = state.memories.get_mut(&p.id) {
                    m.pinned = true;
                    m.updated_ns = p.now_ns;
                }
            }
            DISPATCH_MEMORY_UNPINNED => {
                let p: MemoryPinTogglePayload =
                    postcard::from_bytes(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                if let Some(m) = state.memories.get_mut(&p.id) {
                    m.pinned = false;
                    m.updated_ns = p.now_ns;
                }
            }
            DISPATCH_MEMORY_DELETED => {
                let p: MemoryDeletedPayload =
                    postcard::from_bytes(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                state.memories.remove(&p.id);
            }
            other => {
                tracing::debug!(
                    dispatch = other,
                    seq = ev.entry.seq,
                    "memories fold: ignoring unknown dispatch"
                );
            }
        }
        Ok(())
    }
}
