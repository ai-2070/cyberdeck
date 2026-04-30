//! Distributed entity event log.
//!
//! Each entity's events form an append-only causal chain. Nodes store
//! segments of entity logs they are responsible for. The `LogIndex`
//! provides O(1) lookup by origin_hash.

use bytes::Bytes;
use dashmap::DashMap;

use super::causal::{validate_chain_link, CausalEvent, CausalLink, ChainError};
use crate::adapter::net::identity::EntityId;

/// Local view of an entity's event log.
pub struct EntityLog {
    /// Entity identity.
    entity_id: EntityId,
    /// Truncated entity hash.
    origin_hash: u32,
    /// Events in causal order (by sequence number).
    events: Vec<CausalEvent>,
    /// Base link — genesis for new logs, snapshot head for restored logs.
    /// Used as the validation anchor when `events` is empty.
    base_link: CausalLink,
    /// Payload of the base/head event (for chain validation of next append).
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
            base_link: CausalLink::genesis(origin_hash, 0),
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
            base_link: head_link,
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

        let current_head = self.head_link();
        let current_seq = current_head.sequence;

        // Duplicate check.
        //
        // Previously this was guarded by `current_seq > 0`, which
        // silently skipped the duplicate check for any incoming
        // event when the head was at sequence `0` (i.e. immediately
        // after genesis). The chain validator backstopped the case
        // in practice (a duplicate genesis would fail
        // `validate_chain_link`), but that's a structural-incidental
        // defense. Tighten the guard so we only skip the check when
        // the log is genuinely empty (no events yet — head_link
        // returns the sentinel).
        if !self.events.is_empty() && event.link.sequence <= current_seq {
            return Err(LogError::Duplicate(event.link.sequence));
        }

        // For genesis on a fresh log, accept without parent validation.
        // All other appends validate chain linkage (parent_hash, sequence, origin).
        if self.events.is_empty() && current_head.is_genesis() && event.link.is_genesis() {
            // Accept genesis event
        } else {
            validate_chain_link(&current_head, &self.head_payload, &event.link)
                .map_err(LogError::Chain)?;
        }

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
        self.events.last().map(|e| e.link).unwrap_or(self.base_link)
    }

    /// Get the head sequence number.
    #[inline]
    pub fn head_seq(&self) -> u64 {
        self.events
            .last()
            .map(|e| e.link.sequence)
            .unwrap_or(self.base_link.sequence)
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
    ///
    /// BUG #129: pre-fix `snapshot_seq` was unconditionally bumped
    /// to `seq` whenever `seq > self.snapshot_seq`, even when no
    /// events matched the prune (e.g. on an empty log, or when
    /// `seq < first_event.sequence`). The pruning side-effects
    /// (`base_link`, `head_payload`) only fired when at least one
    /// event was actually removed — so calling `prune_through` on
    /// an empty log advanced `snapshot_seq` while `base_link.sequence`
    /// stayed put, producing a permanent desync where `head_seq()
    /// .max(snapshot_seq())` returned a value the next append
    /// couldn't agree with. Now `snapshot_seq` is only advanced
    /// when `last_pruned.is_some()` — i.e. a real event was
    /// pruned. Callers that need to install an externally-
    /// coordinated snapshot anchor on an empty log should use
    /// `from_snapshot` instead.
    pub fn prune_through(&mut self, seq: u64) {
        // Capture the last pruned event's link and payload so that base_link
        // remains a valid chain anchor if all events are removed. Without this,
        // the next append would fail chain validation because base_link wouldn't
        // match the expected parent_hash.
        let last_pruned = self
            .events
            .iter()
            .rev()
            .find(|e| e.link.sequence <= seq)
            .map(|e| (e.link, e.payload.clone()));

        self.events.retain(|e| e.link.sequence > seq);
        // BUG #129: gate the snapshot_seq bump on having actually
        // pruned something. A no-op prune (empty log, or seq below
        // the first event) must not advance the marker — otherwise
        // `snapshot_seq` desyncs from `base_link.sequence` and
        // future appends are rejected against the implied gap.
        if last_pruned.is_some() && seq > self.snapshot_seq {
            self.snapshot_seq = seq;
        }
        // Update base_link (used as fallback when events is empty) and
        // head_payload (used for chain validation of the next append).
        //
        // When events remain: head_payload is already correct — it tracks
        // the last event's payload (set during append), and partial pruning
        // only removes from the front. We don't need to update it.
        //
        // When all events are removed: set base_link and head_payload to
        // the last pruned event so the next append can chain correctly.
        if self.events.is_empty() {
            if let Some((link, payload)) = last_pruned {
                self.base_link = link;
                self.head_payload = payload;
            }
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
            .field("head_seq", &self.head_seq())
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
    use crate::adapter::net::identity::EntityKeypair;
    use crate::adapter::net::state::causal::CausalChainBuilder;

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
            let event = builder
                .append(Bytes::from(format!("event-{}", i)), 0)
                .unwrap();
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

        let e1 = builder.append(Bytes::from_static(b"event1"), 0).unwrap();
        log.append(e1).unwrap();

        // Skip an event and try to append e3 directly
        let _e2 = builder.append(Bytes::from_static(b"event2"), 0).unwrap();
        let e3 = builder.append(Bytes::from_static(b"event3"), 0).unwrap();

        assert!(matches!(log.append(e3), Err(LogError::Chain(_))));
    }

    #[test]
    fn test_rejects_wrong_origin() {
        let (_, entity_a) = make_entity();
        let (_, entity_b) = make_entity();
        let mut log = EntityLog::new(entity_a);

        let mut builder = CausalChainBuilder::new(entity_b.origin_hash());
        let event = builder
            .append(Bytes::from_static(b"wrong origin"), 0)
            .unwrap();

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
            let event = builder.append(Bytes::from(format!("e{}", i)), 0).unwrap();
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
            let event = builder.append(Bytes::from(format!("e{}", i)), 0).unwrap();
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
            let event = builder.append(Bytes::from(format!("e{}", i)), 0).unwrap();
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
            let event = builder.append(Bytes::from_static(b"hello"), 0).unwrap();
            log_a.append(event).unwrap();
        }

        {
            let mut log_b = index.get_or_create(entity_b.clone());
            let mut builder = CausalChainBuilder::new(log_b.origin_hash());
            let event = builder.append(Bytes::from_static(b"world"), 0).unwrap();
            log_b.append(event).unwrap();
        }

        assert_eq!(index.entity_count(), 2);

        let log = index.get(entity_a.origin_hash()).unwrap();
        assert_eq!(log.len(), 1);
    }

    // ---- Regression tests for Cubic AI findings ----

    #[test]
    fn test_regression_prune_all_then_append() {
        // Regression: prune_through with all events removed used to reset
        // base_link to genesis, breaking chain validation for the next append.
        let (_, entity_id) = make_entity();
        let origin_hash = entity_id.origin_hash();
        let mut log = EntityLog::new(entity_id);
        let mut builder = CausalChainBuilder::new(origin_hash);

        for i in 0..5 {
            let event = builder.append(Bytes::from(format!("e{}", i)), 0).unwrap();
            log.append(event).unwrap();
        }

        // Prune everything
        log.prune_through(5);
        assert_eq!(log.len(), 0);

        // Append the next event — must succeed because base_link was set
        // to the last pruned event's link, not reset to genesis.
        let next = builder
            .append(Bytes::from_static(b"after-prune"), 0)
            .unwrap();
        assert!(
            log.append(next).is_ok(),
            "append after full prune must succeed"
        );
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_regression_duplicate_genesis_rejected() {
        // Regression: genesis events could be appended repeatedly because
        // duplicate detection was skipped at seq 0 and genesis acceptance
        // was not restricted to an empty log.
        let (_, entity_id) = make_entity();
        let origin_hash = entity_id.origin_hash();
        let mut log = EntityLog::new(entity_id);
        let mut builder = CausalChainBuilder::new(origin_hash);

        let e1 = builder.append(Bytes::from_static(b"first"), 0).unwrap();
        log.append(e1).unwrap();

        // Try to append another genesis — must be rejected
        let genesis = CausalEvent {
            link: CausalLink::genesis(origin_hash, 0),
            payload: Bytes::from_static(b"fake genesis"),
            received_at: 0,
        };
        assert!(
            log.append(genesis).is_err(),
            "duplicate genesis must be rejected after log has events"
        );
    }

    // ========================================================================
    // BUG #129: prune_through(seq) on empty / out-of-range logs must not
    // desync snapshot_seq from base_link.sequence
    // ========================================================================

    /// `prune_through` on an empty log is a no-op — `snapshot_seq`
    /// must NOT advance past `base_link.sequence`. Pre-fix it
    /// blindly bumped the marker, leaving a phantom snapshot
    /// reference that no append could honor.
    #[test]
    fn prune_through_on_empty_log_does_not_advance_snapshot_seq() {
        let (_, entity_id) = make_entity();
        let mut log = EntityLog::new(entity_id);

        // Pre-condition: fresh log has snapshot_seq == 0.
        assert_eq!(log.snapshot_seq(), 0);
        assert!(log.is_empty());

        // Caller supplies an externally-coordinated seq; with no
        // events to prune, the marker must stay put. The correct
        // way to install an external snapshot anchor is
        // `from_snapshot`, not this no-op call.
        log.prune_through(1000);

        assert_eq!(
            log.snapshot_seq(),
            0,
            "no-op prune_through must not advance snapshot_seq",
        );
        // base_link.sequence remained at 0 (genesis), so head_seq()
        // and snapshot_seq() agree.
        assert_eq!(log.head_seq(), 0);
    }

    /// `prune_through(seq)` where `seq` is below the first event's
    /// sequence must also be a no-op for `snapshot_seq` — pruning
    /// found nothing to remove, so there's no recoverable anchor
    /// at `seq`.
    #[test]
    fn prune_through_below_first_event_does_not_advance_snapshot_seq() {
        let (_, entity_id) = make_entity();
        let origin_hash = entity_id.origin_hash();
        let mut log = EntityLog::new(entity_id);
        let mut builder = CausalChainBuilder::new(origin_hash);
        for i in 0..5 {
            let event = builder.append(Bytes::from(format!("e{}", i)), 0).unwrap();
            log.append(event).unwrap();
        }
        // The log holds events 1..=5. Try to prune at seq=0
        // (below the first event) — nothing matches.
        log.prune_through(0);

        assert_eq!(log.len(), 5, "no events were pruned");
        assert_eq!(
            log.snapshot_seq(),
            0,
            "prune that touched no events must not advance snapshot_seq",
        );
    }

    /// A successful prune still advances `snapshot_seq` — pins the
    /// happy path so the BUG #129 gate doesn't accidentally lock
    /// out legitimate pruning.
    #[test]
    fn prune_through_advances_snapshot_seq_when_events_pruned() {
        let (_, entity_id) = make_entity();
        let origin_hash = entity_id.origin_hash();
        let mut log = EntityLog::new(entity_id);
        let mut builder = CausalChainBuilder::new(origin_hash);
        for i in 0..5 {
            let event = builder.append(Bytes::from(format!("e{}", i)), 0).unwrap();
            log.append(event).unwrap();
        }
        log.prune_through(3);
        assert_eq!(log.len(), 2, "events 4 and 5 remain");
        assert_eq!(
            log.snapshot_seq(),
            3,
            "snapshot_seq must advance when prune actually removed events",
        );
    }
}
