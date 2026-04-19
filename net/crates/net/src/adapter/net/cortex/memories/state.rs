//! `MemoriesState` — the materialized view held behind the
//! `CortexAdapter<MemoriesState>`'s `RwLock`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::filter::MemoriesFilter;
use super::types::{Memory, MemoryId};

/// Materialized view over the memories log.
///
/// `Serialize` / `Deserialize` are derived so the state can be
/// snapshotted via [`super::super::CortexAdapter::snapshot`] and
/// restored via [`super::super::CortexAdapter::open_from_snapshot`].
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoriesState {
    pub(super) memories: HashMap<MemoryId, Memory>,
}

impl MemoriesState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a memory by id.
    pub fn get(&self, id: MemoryId) -> Option<&Memory> {
        self.memories.get(&id)
    }

    /// Total number of memories currently retained.
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    /// True if no memories are retained.
    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }

    /// True if a memory with `id` exists.
    pub fn contains(&self, id: MemoryId) -> bool {
        self.memories.contains_key(&id)
    }

    /// Iterate over every retained memory.
    pub fn all(&self) -> impl Iterator<Item = &Memory> {
        self.memories.values()
    }

    /// Iterate over currently-pinned memories.
    pub fn pinned(&self) -> impl Iterator<Item = &Memory> {
        self.memories.values().filter(|m| m.pinned)
    }

    /// Iterate over memories that are NOT pinned.
    pub fn unpinned(&self) -> impl Iterator<Item = &Memory> {
        self.memories.values().filter(|m| !m.pinned)
    }

    // -- Prisma-ish convenience surface (NetDB layer) -------------------

    /// Look up a memory by id. Alias of [`Self::get`].
    pub fn find_unique(&self, id: MemoryId) -> Option<&Memory> {
        self.get(id)
    }

    /// Collect all memories matching `filter`, respecting order + limit.
    pub fn find_many(&self, filter: &MemoriesFilter) -> Vec<Memory> {
        filter.apply(self.query()).collect()
    }

    /// Count memories matching `filter`. Ignores `limit`.
    pub fn count_where(&self, filter: &MemoriesFilter) -> usize {
        filter.apply(self.query()).count()
    }

    /// True if any memory matches `filter`. Short-circuits.
    pub fn exists_where(&self, filter: &MemoriesFilter) -> bool {
        filter.apply(self.query()).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem(id: MemoryId, pinned: bool) -> Memory {
        Memory {
            id,
            content: format!("mem-{}", id),
            tags: Vec::new(),
            source: "test".into(),
            created_ns: 0,
            updated_ns: 0,
            pinned,
        }
    }

    #[test]
    fn test_empty_state() {
        let s = MemoriesState::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert!(s.get(1).is_none());
        assert!(!s.contains(1));
        assert_eq!(s.pinned().count(), 0);
        assert_eq!(s.unpinned().count(), 0);
    }

    #[test]
    fn test_pin_split() {
        let mut s = MemoriesState::new();
        s.memories.insert(1, mem(1, true));
        s.memories.insert(2, mem(2, false));
        s.memories.insert(3, mem(3, true));

        assert_eq!(s.len(), 3);
        assert_eq!(s.pinned().count(), 2);
        assert_eq!(s.unpinned().count(), 1);
        assert!(s.contains(2));
        assert!(!s.contains(99));
    }
}
