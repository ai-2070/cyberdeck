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
                    bincode::deserialize(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
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
            DISPATCH_MEMORY_RETAGGED => {
                let p: MemoryRetaggedPayload =
                    bincode::deserialize(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                if let Some(m) = state.memories.get_mut(&p.id) {
                    m.tags = p.tags;
                    m.updated_ns = p.now_ns;
                }
            }
            DISPATCH_MEMORY_PINNED => {
                let p: MemoryPinTogglePayload =
                    bincode::deserialize(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                if let Some(m) = state.memories.get_mut(&p.id) {
                    m.pinned = true;
                    m.updated_ns = p.now_ns;
                }
            }
            DISPATCH_MEMORY_UNPINNED => {
                let p: MemoryPinTogglePayload =
                    bincode::deserialize(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
                if let Some(m) = state.memories.get_mut(&p.id) {
                    m.pinned = false;
                    m.updated_ns = p.now_ns;
                }
            }
            DISPATCH_MEMORY_DELETED => {
                let p: MemoryDeletedPayload =
                    bincode::deserialize(tail).map_err(|e| RedexError::Encode(e.to_string()))?;
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
