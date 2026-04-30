//! Structural continuity — formalizing what the causal chain provides.
//!
//! The causal chain IS identity. `ContinuityStatus` describes the state
//! of an entity's chain from an observer's perspective. `ContinuityProof`
//! is a compact (40-byte) transmittable proof of chain integrity.

use crate::adapter::net::state::causal::compute_parent_hash;
use crate::adapter::net::state::log::EntityLog;
use crate::adapter::net::state::snapshot::StateSnapshot;

/// The continuity status of an entity from an observer's perspective.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuityStatus {
    /// Chain is unbroken from genesis to head. Entity is continuous.
    Continuous {
        /// xxh3 hash that would be the parent_hash of the genesis link's successor.
        genesis_hash: u64,
        /// Head sequence number.
        head_seq: u64,
        /// parent_hash of the next event that would follow the head.
        head_hash: u64,
    },
    /// Chain has a verified gap. Entity forked.
    Forked {
        /// Sequence where the fork occurred.
        fork_point: u64,
        /// The original chain's parent_hash at the fork point.
        original_hash: u64,
        /// The new chain's parent_hash at the fork point.
        fork_hash: u64,
    },
    /// Chain cannot be verified (missing data).
    Unverifiable {
        /// Last verified sequence.
        last_verified_seq: u64,
        /// First unverified sequence.
        gap_start: u64,
    },
    /// Entity was explicitly migrated (chain transferred, not broken).
    Migrated {
        /// Sequence at migration point.
        migration_seq: u64,
        /// Source node that held the chain before migration.
        source_node: u64,
        /// Target node that holds the chain after migration.
        target_node: u64,
    },
}

/// Compact proof of continuity that can be transmitted (36 bytes).
///
/// A node can send this to another node to prove its chain is intact
/// over a given sequence range, without transferring the full log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContinuityProof {
    /// Entity origin hash.
    pub origin_hash: u32,
    /// Start of the proven range.
    pub from_seq: u64,
    /// End of the proven range.
    pub to_seq: u64,
    /// parent_hash computed from the event at from_seq.
    pub from_hash: u64,
    /// parent_hash computed from the event at to_seq.
    pub to_hash: u64,
}

/// Wire size of a ContinuityProof.
pub const CONTINUITY_PROOF_SIZE: usize = 36; // 4 + 8 + 8 + 8 + 8

impl ContinuityProof {
    /// Extract a proof from a local entity log.
    ///
    /// Returns `None` if the log is empty.
    pub fn from_log(log: &EntityLog) -> Option<Self> {
        if log.is_empty() {
            return None;
        }

        let events = log.range(0, u64::MAX);
        if events.is_empty() {
            return None;
        }

        let first = &events[0];
        let last = events.last().unwrap();

        let from_hash = compute_parent_hash(&first.link, &first.payload);
        let to_hash = compute_parent_hash(&last.link, &last.payload);

        Some(Self {
            origin_hash: log.origin_hash(),
            from_seq: first.link.sequence,
            to_seq: last.link.sequence,
            from_hash,
            to_hash,
        })
    }

    /// Verify this proof against a local entity log.
    ///
    /// Checks that the hash values match what the local log has
    /// for the same sequence numbers.
    pub fn verify_against(&self, log: &EntityLog) -> Result<(), ProofError> {
        if self.origin_hash != log.origin_hash() {
            return Err(ProofError::OriginMismatch);
        }

        let from_events = log.range(self.from_seq, self.from_seq);
        if let Some(event) = from_events.first() {
            let local_hash = compute_parent_hash(&event.link, &event.payload);
            if local_hash != self.from_hash {
                return Err(ProofError::HashMismatch {
                    seq: self.from_seq,
                    expected: self.from_hash,
                    got: local_hash,
                });
            }
        } else {
            return Err(ProofError::MissingEvent(self.from_seq));
        }

        let to_events = log.range(self.to_seq, self.to_seq);
        if let Some(event) = to_events.first() {
            let local_hash = compute_parent_hash(&event.link, &event.payload);
            if local_hash != self.to_hash {
                return Err(ProofError::HashMismatch {
                    seq: self.to_seq,
                    expected: self.to_hash,
                    got: local_hash,
                });
            }
        } else {
            return Err(ProofError::MissingEvent(self.to_seq));
        }

        Ok(())
    }

    /// Serialize to bytes.
    pub fn to_bytes(&self) -> [u8; CONTINUITY_PROOF_SIZE] {
        let mut buf = [0u8; CONTINUITY_PROOF_SIZE];
        buf[0..4].copy_from_slice(&self.origin_hash.to_le_bytes());
        buf[4..12].copy_from_slice(&self.from_seq.to_le_bytes());
        buf[12..20].copy_from_slice(&self.to_seq.to_le_bytes());
        buf[20..28].copy_from_slice(&self.from_hash.to_le_bytes());
        buf[28..36].copy_from_slice(&self.to_hash.to_le_bytes());
        buf
    }

    /// Deserialize from bytes.
    ///
    /// Rejects buffers whose length differs from
    /// [`CONTINUITY_PROOF_SIZE`] so trailing bytes aren't silently
    /// accepted (the old `< SIZE` guard let concatenated proofs or
    /// framing garbage parse as the first proof).
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() != CONTINUITY_PROOF_SIZE {
            return None;
        }
        Some(Self {
            origin_hash: u32::from_le_bytes(data[0..4].try_into().unwrap()),
            from_seq: u64::from_le_bytes(data[4..12].try_into().unwrap()),
            to_seq: u64::from_le_bytes(data[12..20].try_into().unwrap()),
            from_hash: u64::from_le_bytes(data[20..28].try_into().unwrap()),
            to_hash: u64::from_le_bytes(data[28..36].try_into().unwrap()),
        })
    }
}

/// Assess the continuity status of an entity log.
///
/// Walks the log and validates every consecutive pair. Returns the
/// first problem found, or `Continuous` if the chain is intact.
///
/// # Genesis / snapshot anchoring (BUG #114)
///
/// Pair-wise linkage alone is not enough: after `prune_through(N)`,
/// a log only contains events with `seq > N`, and a corrupt restore
/// (or a malicious party) could ship a log starting at e.g. seq 100
/// with consistent pair-wise hashes but no evidence that events
/// `0..99` ever existed. To detect that, this function requires the
/// log to be anchored either at genesis (the first event has
/// `sequence == 1`, the genesis-successor) or at a known snapshot
/// (`snapshot.through_seq + 1 == first_event.sequence`). If neither
/// holds, returns `Unverifiable { last_verified_seq: 0, gap_start: 0 }`.
///
/// Pass `None` for `snapshot` when the log is expected to start at
/// genesis (no prior pruning); pass `Some(&snapshot)` when the log
/// was restored from `snapshot` and should pick up at the next event
/// after the snapshot's `through_seq`.
pub fn assess_continuity(
    log: &EntityLog,
    snapshot: Option<&StateSnapshot>,
) -> ContinuityStatus {
    let events = log.range(0, u64::MAX);

    if events.is_empty() {
        return ContinuityStatus::Continuous {
            genesis_hash: 0,
            head_seq: log.head_seq(),
            head_hash: 0,
        };
    }

    // BUG #114: anchor check. A pair-wise-consistent chain is not
    // continuous if it doesn't start at genesis (seq 1, post-genesis
    // successor) or at a verified snapshot boundary.
    let first_seq = events[0].link.sequence;
    let anchored = first_seq == 1
        || snapshot
            .map(|s| s.through_seq.saturating_add(1) == first_seq)
            .unwrap_or(false);
    if !anchored {
        return ContinuityStatus::Unverifiable {
            last_verified_seq: 0,
            gap_start: 0,
        };
    }

    // Validate consecutive pairs
    for i in 1..events.len() {
        let prev = &events[i - 1];
        let curr = &events[i];

        // Check sequence continuity
        if curr.link.sequence != prev.link.sequence + 1 {
            return ContinuityStatus::Unverifiable {
                last_verified_seq: prev.link.sequence,
                gap_start: prev.link.sequence + 1,
            };
        }

        // Check parent hash linkage
        let expected_parent = compute_parent_hash(&prev.link, &prev.payload);
        if curr.link.parent_hash != expected_parent {
            return ContinuityStatus::Forked {
                fork_point: curr.link.sequence,
                original_hash: expected_parent,
                fork_hash: curr.link.parent_hash,
            };
        }
    }

    let first = &events[0];
    let last = events.last().unwrap();
    let genesis_hash = compute_parent_hash(&first.link, &first.payload);
    let head_hash = compute_parent_hash(&last.link, &last.payload);

    ContinuityStatus::Continuous {
        genesis_hash,
        head_seq: last.link.sequence,
        head_hash,
    }
}

/// Errors from proof verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofError {
    /// Origin hash doesn't match the log.
    OriginMismatch,
    /// Hash at a given sequence doesn't match.
    HashMismatch {
        /// Sequence number where mismatch occurred.
        seq: u64,
        /// Expected hash from the proof.
        expected: u64,
        /// Actual hash from the local log.
        got: u64,
    },
    /// Event at the given sequence is missing from the local log.
    MissingEvent(u64),
}

impl std::fmt::Display for ProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OriginMismatch => write!(f, "origin hash mismatch"),
            Self::HashMismatch { seq, expected, got } => {
                write!(
                    f,
                    "hash mismatch at seq {}: expected {:#x}, got {:#x}",
                    seq, expected, got
                )
            }
            Self::MissingEvent(seq) => write!(f, "missing event at seq {}", seq),
        }
    }
}

impl std::error::Error for ProofError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::identity::EntityKeypair;
    use crate::adapter::net::state::causal::CausalChainBuilder;
    use bytes::Bytes;

    fn build_log(count: usize) -> (EntityLog, CausalChainBuilder) {
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let mut log = EntityLog::new(kp.entity_id().clone());
        let mut builder = CausalChainBuilder::new(origin);

        for i in 0..count {
            let event = builder
                .append(Bytes::from(format!("event-{}", i)), 0)
                .unwrap();
            log.append(event).unwrap();
        }

        (log, builder)
    }

    #[test]
    fn test_assess_continuous() {
        let (log, _) = build_log(10);
        let status = assess_continuity(&log, None);
        assert!(matches!(
            status,
            ContinuityStatus::Continuous { head_seq: 10, .. }
        ));
    }

    #[test]
    fn test_assess_empty_log() {
        let kp = EntityKeypair::generate();
        let log = EntityLog::new(kp.entity_id().clone());
        let status = assess_continuity(&log, None);
        assert!(matches!(status, ContinuityStatus::Continuous { .. }));
    }

    // ========================================================================
    // BUG #114: pruned-no-snapshot logs must not report Continuous
    // ========================================================================

    /// A log whose first event has `sequence > 1` (e.g. because
    /// earlier events were pruned, or the log was reconstructed
    /// from a partial backup) must be reported as
    /// `Unverifiable { gap_start: 0 }` when no snapshot is supplied
    /// to bridge the gap. Pre-fix this returned `Continuous` and
    /// downstream peers believed the chain was intact even when
    /// genesis-to-first events were entirely missing.
    #[test]
    fn assess_continuity_unverifiable_when_log_starts_past_genesis_without_snapshot() {
        let (mut log, _) = build_log(20);
        // Prune through seq 10 — the log now starts at seq 11 with
        // no snapshot reference.
        log.prune_through(10);
        assert!(!log.is_empty(), "test setup: log must still have events 11..20");

        let status = assess_continuity(&log, None);
        assert!(
            matches!(
                status,
                ContinuityStatus::Unverifiable {
                    last_verified_seq: 0,
                    gap_start: 0,
                }
            ),
            "pruned log without snapshot must be Unverifiable, got {:?}",
            status,
        );
    }

    /// Same pruned log, but with a snapshot whose `through_seq`
    /// matches the gap — must report `Continuous`. Pins the
    /// snapshot-bridges-gap acceptance path so a future tightening
    /// can't reject legitimately-restored logs.
    #[test]
    fn assess_continuity_continuous_when_snapshot_bridges_gap() {
        use crate::adapter::net::state::causal::CausalLink;
        use crate::adapter::net::state::horizon::ObservedHorizon;

        let (mut log, _) = build_log(20);
        log.prune_through(10);

        // Build a fake snapshot whose `through_seq == 10` so the
        // first remaining event (seq 11) is anchored. The snapshot's
        // other fields are irrelevant to the continuity-anchor check
        // — we only inspect `through_seq`.
        let snapshot = StateSnapshot {
            version: 1,
            entity_id: log.entity_id().clone(),
            through_seq: 10,
            chain_link: CausalLink::genesis(log.origin_hash(), 0),
            state: bytes::Bytes::new(),
            horizon: ObservedHorizon::default(),
            created_at: 0,
            bindings_bytes: Vec::new(),
            identity_envelope: None,
            head_payload: bytes::Bytes::new(),
        };

        let status = assess_continuity(&log, Some(&snapshot));
        assert!(
            matches!(status, ContinuityStatus::Continuous { head_seq: 20, .. }),
            "snapshot.through_seq + 1 == first_event.sequence must anchor, got {:?}",
            status,
        );
    }

    /// A snapshot whose `through_seq` does NOT match the log's gap
    /// (e.g. caller passed the wrong snapshot) must NOT anchor —
    /// this would let a forged "I have this snapshot" claim
    /// silently bypass the genesis check.
    #[test]
    fn assess_continuity_unverifiable_when_snapshot_through_seq_does_not_bridge() {
        use crate::adapter::net::state::causal::CausalLink;
        use crate::adapter::net::state::horizon::ObservedHorizon;

        let (mut log, _) = build_log(20);
        log.prune_through(10);

        // Mismatched snapshot — claims through_seq=5 but log starts at 11.
        let snapshot = StateSnapshot {
            version: 1,
            entity_id: log.entity_id().clone(),
            through_seq: 5,
            chain_link: CausalLink::genesis(log.origin_hash(), 0),
            state: bytes::Bytes::new(),
            horizon: ObservedHorizon::default(),
            created_at: 0,
            bindings_bytes: Vec::new(),
            identity_envelope: None,
            head_payload: bytes::Bytes::new(),
        };

        let status = assess_continuity(&log, Some(&snapshot));
        assert!(
            matches!(
                status,
                ContinuityStatus::Unverifiable {
                    last_verified_seq: 0,
                    gap_start: 0,
                }
            ),
            "mismatched snapshot must not anchor, got {:?}",
            status,
        );
    }

    #[test]
    fn test_proof_roundtrip() {
        let (log, _) = build_log(5);
        let proof = ContinuityProof::from_log(&log).unwrap();

        let bytes = proof.to_bytes();
        assert_eq!(bytes.len(), CONTINUITY_PROOF_SIZE);

        let parsed = ContinuityProof::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, proof);
    }

    #[test]
    fn test_proof_verify_against_same_log() {
        let (log, _) = build_log(5);
        let proof = ContinuityProof::from_log(&log).unwrap();

        assert!(proof.verify_against(&log).is_ok());
    }

    #[test]
    fn test_proof_verify_wrong_origin() {
        let (log_a, _) = build_log(5);
        let (log_b, _) = build_log(5);

        let proof = ContinuityProof::from_log(&log_a).unwrap();
        assert_eq!(
            proof.verify_against(&log_b).unwrap_err(),
            ProofError::OriginMismatch,
        );
    }

    #[test]
    fn test_proof_from_empty_log() {
        let kp = EntityKeypair::generate();
        let log = EntityLog::new(kp.entity_id().clone());
        assert!(ContinuityProof::from_log(&log).is_none());
    }
}
