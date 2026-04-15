//! Structural continuity — formalizing what the causal chain provides.
//!
//! The causal chain IS identity. `ContinuityStatus` describes the state
//! of an entity's chain from an observer's perspective. `ContinuityProof`
//! is a compact (40-byte) transmittable proof of chain integrity.

use crate::adapter::nltp::state::causal::compute_parent_hash;
use crate::adapter::nltp::state::log::EntityLog;

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
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < CONTINUITY_PROOF_SIZE {
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
pub fn assess_continuity(log: &EntityLog) -> ContinuityStatus {
    let events = log.range(0, u64::MAX);

    if events.is_empty() {
        return ContinuityStatus::Continuous {
            genesis_hash: 0,
            head_seq: log.head_seq(),
            head_hash: 0,
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
    use crate::adapter::nltp::identity::EntityKeypair;
    use crate::adapter::nltp::state::causal::CausalChainBuilder;
    use bytes::Bytes;

    fn build_log(count: usize) -> (EntityLog, CausalChainBuilder) {
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let mut log = EntityLog::new(kp.entity_id().clone());
        let mut builder = CausalChainBuilder::new(origin);

        for i in 0..count {
            let event = builder.append(Bytes::from(format!("event-{}", i)), 0);
            log.append(event).unwrap();
        }

        (log, builder)
    }

    #[test]
    fn test_assess_continuous() {
        let (log, _) = build_log(10);
        let status = assess_continuity(&log);
        assert!(matches!(
            status,
            ContinuityStatus::Continuous { head_seq: 10, .. }
        ));
    }

    #[test]
    fn test_assess_empty_log() {
        let kp = EntityKeypair::generate();
        let log = EntityLog::new(kp.entity_id().clone());
        let status = assess_continuity(&log);
        assert!(matches!(status, ContinuityStatus::Continuous { .. }));
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
