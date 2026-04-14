//! Distributed entity event log.
//!
//! Each entity's events form an append-only causal chain. Nodes store
//! segments of entity logs they are responsible for. The `LogIndex`
//! provides O(1) lookup by origin_hash.

use bytes::Bytes;
use dashmap::DashMap;

use super::causal::{validate_chain_link, CausalEvent, CausalLink, ChainError};
use crate::adapter::bltp::identity::EntityId;

/// Local view of an entity's event log.
pub struct EntityLog {
    /// Entity identity.
    entity_id: EntityId,
    /// Truncated entity hash.
    origin_hash: u32,
    /// Events in causal order (by sequence number).
    events: Vec<CausalEvent>,
    /// Highest contiguous sequence held locally.
    head_seq: u64,
    /// Payload of the head event (for chain validation of next append).
    head_payload: Bytes,
    /// Latest snapshot sequence (events before this can be pruned).
    snapshot_seq: u64,
}

impl EntityLog {
    /// Create a new empty log for an entity.
    pub fn new(entity_id: EntityId) -> Self {
        let origin_hash = entity_id.origin_hash();
        Self {
            entity_id,
            origin_hash,
            events: Vec::new(),
            head_seq: 0,
            head_payload: Bytes::new(),
            snapshot_seq: 0,
        }
    }

    /// Create from a snapshot (for catchup — events after snapshot_seq will be appended).
    pub fn from_snapshot(
        entity_id: EntityId,
        snapshot_seq: u64,
        head_link: CausalLink,
        head_payload: Bytes,
    ) -> Self {
        let origin_hash = entity_id.origin_hash();
        Self {
            entity_id,
            origin_hash,
            events: Vec::new(),
            head_seq: head_link.sequence,
            head_payload,
            snapshot_seq,
        }
    }

    /// Append a causal event to the log.
    ///
    /// Validates chain integrity (origin, sequence, parent_hash).
    /// Returns an error if the chain is broken.
    pub fn append(&mut self, event: CausalEvent) -> Result<(), LogError> {
        if event.link.origin_hash != self.origin_hash {
            return Err(LogError::Chain(ChainError::OriginMismatch {
                expected: self.origin_hash,
                got: event.link.origin_hash,
            }));
        }

        if self.events.is_empty() && self.head_seq == 0 {
            // First event — accept genesis or seq 1
            if !event.link.is_genesis() && event.link.sequence != 1 {
                // If we have a snapshot, expect snapshot_seq + 1
                if self.snapshot_seq > 0 && event.link.sequence != self.snapshot_seq + 1 {
                    return Err(LogError::Chain(ChainError::SequenceGap {
                        expected: self.snapshot_seq + 1,
                        got: event.link.sequence,
                    }));
                }
            }
        } else {
            // Validate against the current head
            let head_link = self.head_link();
            validate_chain_link(&head_link, &self.head_payload, &event.link)
                .map_err(LogError::Chain)?;
        }

        // Duplicate check
        if event.link.sequence <= self.head_seq && self.head_seq > 0 {
            return Err(LogError::Duplicate(event.link.sequence));
        }

        self.head_seq = event.link.sequence;
        self.head_payload = event.payload.clone();
        self.events.push(event);
        Ok(())
    }

    /// Get events in a sequence range (inclusive).
    pub fn range(&self, from_seq: u64, to_seq: u64) -> Vec<&CausalEvent> {
        self.events
            .iter()
            .filter(|e| e.link.sequence >= from_seq && e.link.sequence <= to_seq)
            .collect()
    }

    /// Get all events after a given sequence.
    pub fn after(&self, seq: u64) -> Vec<&CausalEvent> {
        self.events
            .iter()
            .filter(|e| e.link.sequence > seq)
            .collect()
    }

    /// Get the head (latest) link.
    pub fn head_link(&self) -> CausalLink {
        self.events
            .last()
            .map(|e| e.link)
            .unwrap_or_else(|| CausalLink::genesis(self.origin_hash, 0))
    }

    /// Get the head sequence number.
    #[inline]
    pub fn head_seq(&self) -> u64 {
        self.head_seq
    }

    /// Get the entity ID.
    #[inline]
    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    /// Get the origin hash.
    #[inline]
    pub fn origin_hash(&self) -> u32 {
        self.origin_hash
    }

    /// Number of events in the log.
    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the log is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Prune events up to and including a sequence number.
    ///
    /// Called after a snapshot is taken at that sequence.
    pub fn prune_through(&mut self, seq: u64) {
        self.events.retain(|e| e.link.sequence > seq);
        if seq > self.snapshot_seq {
            self.snapshot_seq = seq;
        }
    }

    /// Get the snapshot sequence (events before this have been pruned).
    #[inline]
    pub fn snapshot_seq(&self) -> u64 {
        self.snapshot_seq
    }
}

impl std::fmt::Debug for EntityLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityLog")
            .field("entity_id", &self.entity_id)
            .field("origin_hash", &format!("{:#x}", self.origin_hash))
            .field("events", &self.events.len())
            .field("head_seq", &self.head_seq)
            .field("snapshot_seq", &self.snapshot_seq)
            .finish()
    }
}

/// Index of entity logs by origin_hash.
///
/// O(1) lookup for per-packet routing to the correct entity log.
pub struct LogIndex {
    logs: DashMap<u32, EntityLog>,
}

impl LogIndex {
    /// Create an empty index.
    pub fn new() -> Self {
        Self {
            logs: DashMap::new(),
        }
    }

    /// Get or create the log for an entity.
    pub fn get_or_create(
        &self,
        entity_id: EntityId,
    ) -> dashmap::mapref::one::RefMut<'_, u32, EntityLog> {
        let origin_hash = entity_id.origin_hash();
        self.logs
            .entry(origin_hash)
            .or_insert_with(|| EntityLog::new(entity_id))
    }

    /// Get the log for an entity (read-only).
    pub fn get(&self, origin_hash: u32) -> Option<dashmap::mapref::one::Ref<'_, u32, EntityLog>> {
        self.logs.get(&origin_hash)
    }

    /// Number of tracked entities.
    pub fn entity_count(&self) -> usize {
        self.logs.len()
    }

    /// Remove an entity's log.
    pub fn remove(&self, origin_hash: u32) -> Option<EntityLog> {
        self.logs.remove(&origin_hash).map(|(_, log)| log)
    }
}

impl Default for LogIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LogIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogIndex")
            .field("entities", &self.logs.len())
            .finish()
    }
}

/// Errors from log operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogError {
    /// Chain validation failed.
    Chain(ChainError),
    /// Duplicate sequence number.
    Duplicate(u64),
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chain(e) => write!(f, "chain error: {}", e),
            Self::Duplicate(seq) => write!(f, "duplicate sequence: {}", seq),
        }
    }
}

impl std::error::Error for LogError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::bltp::identity::EntityKeypair;
    use crate::adapter::bltp::state::causal::CausalChainBuilder;

    fn make_entity() -> (EntityKeypair, EntityId) {
        let kp = EntityKeypair::generate();
        let id = kp.entity_id().clone();
        (kp, id)
    }

    #[test]
    fn test_append_chain() {
        let (_, entity_id) = make_entity();
        let origin_hash = entity_id.origin_hash();
        let mut log = EntityLog::new(entity_id);
        let mut builder = CausalChainBuilder::new(origin_hash);

        for i in 0..5 {
            let event = builder.append(Bytes::from(format!("event-{}", i)), 0);
            assert!(log.append(event).is_ok());
        }

        assert_eq!(log.len(), 5);
        assert_eq!(log.head_seq(), 5);
    }

    #[test]
    fn test_rejects_broken_chain() {
        let (_, entity_id) = make_entity();
        let origin_hash = entity_id.origin_hash();
        let mut log = EntityLog::new(entity_id);
        let mut builder = CausalChainBuilder::new(origin_hash);

        let e1 = builder.append(Bytes::from_static(b"event1"), 0);
        log.append(e1).unwrap();

        // Skip an event and try to append e3 directly
        let _e2 = builder.append(Bytes::from_static(b"event2"), 0);
        let e3 = builder.append(Bytes::from_static(b"event3"), 0);

        assert!(matches!(log.append(e3), Err(LogError::Chain(_))));
    }

    #[test]
    fn test_rejects_wrong_origin() {
        let (_, entity_a) = make_entity();
        let (_, entity_b) = make_entity();
        let mut log = EntityLog::new(entity_a);

        let mut builder = CausalChainBuilder::new(entity_b.origin_hash());
        let event = builder.append(Bytes::from_static(b"wrong origin"), 0);

        assert!(matches!(
            log.append(event),
            Err(LogError::Chain(ChainError::OriginMismatch { .. }))
        ));
    }

    #[test]
    fn test_range_query() {
        let (_, entity_id) = make_entity();
        let origin_hash = entity_id.origin_hash();
        let mut log = EntityLog::new(entity_id);
        let mut builder = CausalChainBuilder::new(origin_hash);

        for i in 0..10 {
            let event = builder.append(Bytes::from(format!("e{}", i)), 0);
            log.append(event).unwrap();
        }

        let range = log.range(3, 7);
        assert_eq!(range.len(), 5);
        assert_eq!(range[0].link.sequence, 3);
        assert_eq!(range[4].link.sequence, 7);
    }

    #[test]
    fn test_after_query() {
        let (_, entity_id) = make_entity();
        let origin_hash = entity_id.origin_hash();
        let mut log = EntityLog::new(entity_id);
        let mut builder = CausalChainBuilder::new(origin_hash);

        for i in 0..5 {
            let event = builder.append(Bytes::from(format!("e{}", i)), 0);
            log.append(event).unwrap();
        }

        let after = log.after(3);
        assert_eq!(after.len(), 2); // seq 4 and 5
    }

    #[test]
    fn test_prune() {
        let (_, entity_id) = make_entity();
        let origin_hash = entity_id.origin_hash();
        let mut log = EntityLog::new(entity_id);
        let mut builder = CausalChainBuilder::new(origin_hash);

        for i in 0..10 {
            let event = builder.append(Bytes::from(format!("e{}", i)), 0);
            log.append(event).unwrap();
        }

        log.prune_through(5);
        assert_eq!(log.len(), 5); // events 6-10 remain
        assert_eq!(log.snapshot_seq(), 5);
    }

    #[test]
    fn test_log_index() {
        let index = LogIndex::new();
        let (_, entity_a) = make_entity();
        let (_, entity_b) = make_entity();

        {
            let mut log_a = index.get_or_create(entity_a.clone());
            let mut builder = CausalChainBuilder::new(log_a.origin_hash());
            let event = builder.append(Bytes::from_static(b"hello"), 0);
            log_a.append(event).unwrap();
        }

        {
            let mut log_b = index.get_or_create(entity_b.clone());
            let mut builder = CausalChainBuilder::new(log_b.origin_hash());
            let event = builder.append(Bytes::from_static(b"world"), 0);
            log_b.append(event).unwrap();
        }

        assert_eq!(index.entity_count(), 2);

        let log = index.get(entity_a.origin_hash()).unwrap();
        assert_eq!(log.len(), 1);
    }
}
