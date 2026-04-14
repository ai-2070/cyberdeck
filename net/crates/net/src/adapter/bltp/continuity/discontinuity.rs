//! Honest discontinuity — chain breaks and entity forks.
//!
//! When a chain breaks (node crash, data loss, corruption), the system
//! creates a new entity with documented lineage via a `ForkRecord`.
//! The mesh knows the original entity is discontinued and a fork has
//! taken its place. No silent recovery — only honest forking.

use xxhash_rust::xxh3::xxh3_64;

use crate::adapter::bltp::identity::EntityKeypair;
use crate::adapter::bltp::state::causal::{CausalChainBuilder, CausalLink, ChainError};

/// A detected discontinuity in an entity's causal chain.
#[derive(Debug, Clone)]
pub struct Discontinuity {
    /// Entity whose chain broke.
    pub origin_hash: u32,
    /// Last verified event in the original chain.
    pub last_verified: CausalLink,
    /// The event that could not be linked (if available).
    pub failed_link: Option<CausalLink>,
    /// Why the chain broke.
    pub reason: DiscontinuityReason,
    /// When the break was detected (local nanos).
    pub detected_at: u64,
}

/// Why a chain broke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscontinuityReason {
    /// Node crashed, state lost between snapshot and head.
    NodeCrash {
        /// Last snapshot sequence before the crash.
        last_snapshot_seq: u64,
    },
    /// Chain validation failed.
    ChainBreak(ChainError),
    /// Conflicting chains from same origin (split brain).
    ConflictingChains {
        /// Sequence where conflict was detected.
        seq: u64,
        /// Hash from chain A.
        hash_a: u64,
        /// Hash from chain B.
        hash_b: u64,
    },
    /// Data corruption detected.
    Corruption,
}

impl std::fmt::Display for DiscontinuityReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeCrash { last_snapshot_seq } => {
                write!(f, "node crash (last snapshot at seq {})", last_snapshot_seq)
            }
            Self::ChainBreak(e) => write!(f, "chain break: {}", e),
            Self::ConflictingChains {
                seq,
                hash_a,
                hash_b,
            } => {
                write!(
                    f,
                    "conflicting chains at seq {}: {:#x} vs {:#x}",
                    seq, hash_a, hash_b
                )
            }
            Self::Corruption => write!(f, "data corruption"),
        }
    }
}

/// Record of an entity fork — created when a chain breaks.
///
/// The forked entity gets a new keypair and a new chain. The fork genesis
/// link has a deterministic sentinel `parent_hash` so any node can verify
/// the fork is legitimate.
#[derive(Debug, Clone)]
pub struct ForkRecord {
    /// The original entity's origin hash.
    pub original_origin: u32,
    /// The forked entity's origin hash (new keypair).
    pub forked_origin: u32,
    /// Sequence at which the fork occurred.
    pub fork_seq: u64,
    /// The forked entity's genesis link.
    pub fork_genesis: CausalLink,
    /// Snapshot sequence used to seed the fork (if any).
    pub from_snapshot_seq: Option<u64>,
}

/// Compute the deterministic fork sentinel hash.
///
/// `parent_hash = xxh3(original_origin ++ fork_seq ++ "fork")`
///
/// Any node can verify a fork record by recomputing this sentinel.
pub fn fork_sentinel(original_origin: u32, fork_seq: u64) -> u64 {
    let mut buf = Vec::with_capacity(4 + 8 + 4);
    buf.extend_from_slice(&original_origin.to_le_bytes());
    buf.extend_from_slice(&fork_seq.to_le_bytes());
    buf.extend_from_slice(b"fork");
    xxh3_64(&buf)
}

/// Create a forked entity from a discontinuity.
///
/// Generates a new keypair for the forked entity and creates a genesis
/// link with the deterministic fork sentinel as parent_hash.
///
/// Returns the new keypair, fork record, and chain builder ready to
/// produce events.
pub fn fork_entity(
    original_origin: u32,
    fork_seq: u64,
    from_snapshot_seq: Option<u64>,
) -> (EntityKeypair, ForkRecord, CausalChainBuilder) {
    let new_keypair = EntityKeypair::generate();
    let new_origin = new_keypair.origin_hash();

    // Fork genesis has the sentinel as parent_hash
    let sentinel = fork_sentinel(original_origin, fork_seq);
    let fork_genesis = CausalLink {
        origin_hash: new_origin,
        horizon_encoded: 0,
        sequence: 0,
        parent_hash: sentinel,
    };

    let record = ForkRecord {
        original_origin,
        forked_origin: new_origin,
        fork_seq,
        fork_genesis,
        from_snapshot_seq,
    };

    // Chain builder starts from the fork genesis
    let builder = CausalChainBuilder::from_head(fork_genesis, bytes::Bytes::new());

    (new_keypair, record, builder)
}

impl ForkRecord {
    /// Verify that this fork record is structurally valid.
    ///
    /// Checks:
    /// - Sentinel parent_hash matches the deterministic fork hash
    /// - Fork genesis origin matches forked_origin
    /// - Fork genesis sequence is 0 (genesis)
    /// - Original and forked origins differ
    pub fn verify(&self) -> bool {
        let expected = fork_sentinel(self.original_origin, self.fork_seq);
        self.fork_genesis.parent_hash == expected
            && self.fork_genesis.origin_hash == self.forked_origin
            && self.fork_genesis.sequence == 0
            && self.original_origin != self.forked_origin
    }

    /// Wire size: 4 + 4 + 8 + 24 + 1 + 8 = 49 bytes max.
    pub const WIRE_SIZE: usize = 49;

    /// Serialize to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::WIRE_SIZE);
        buf.extend_from_slice(&self.original_origin.to_le_bytes());
        buf.extend_from_slice(&self.forked_origin.to_le_bytes());
        buf.extend_from_slice(&self.fork_seq.to_le_bytes());
        buf.extend_from_slice(&self.fork_genesis.to_bytes());
        match self.from_snapshot_seq {
            Some(seq) => {
                buf.push(1);
                buf.extend_from_slice(&seq.to_le_bytes());
            }
            None => {
                buf.push(0);
                buf.extend_from_slice(&0u64.to_le_bytes());
            }
        }
        buf
    }

    /// Deserialize from bytes.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::WIRE_SIZE {
            return None;
        }
        let original_origin = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let forked_origin = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let fork_seq = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let fork_genesis = CausalLink::from_bytes(&data[16..40])?;
        let has_snapshot = data[40] != 0;
        let snapshot_seq = u64::from_le_bytes(data[41..49].try_into().unwrap());
        let from_snapshot_seq = if has_snapshot {
            Some(snapshot_seq)
        } else {
            None
        };

        Some(Self {
            original_origin,
            forked_origin,
            fork_seq,
            fork_genesis,
            from_snapshot_seq,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_sentinel_deterministic() {
        let s1 = fork_sentinel(0xAAAA, 42);
        let s2 = fork_sentinel(0xAAAA, 42);
        assert_eq!(s1, s2);
        assert_ne!(s1, 0);
    }

    #[test]
    fn test_fork_sentinel_differs() {
        let s1 = fork_sentinel(0xAAAA, 42);
        let s2 = fork_sentinel(0xBBBB, 42);
        let s3 = fork_sentinel(0xAAAA, 43);
        assert_ne!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_fork_entity() {
        let (keypair, record, builder) = fork_entity(0xAAAA, 100, Some(90));

        assert_eq!(record.original_origin, 0xAAAA);
        assert_eq!(record.forked_origin, keypair.origin_hash());
        assert_eq!(record.fork_seq, 100);
        assert_eq!(record.from_snapshot_seq, Some(90));
        assert!(record.verify());

        // Builder should be ready to produce events
        assert_eq!(builder.origin_hash(), keypair.origin_hash());
    }

    #[test]
    fn test_fork_record_roundtrip() {
        let (_, record, _) = fork_entity(0xDEAD, 500, None);

        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), ForkRecord::WIRE_SIZE);

        let parsed = ForkRecord::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.original_origin, record.original_origin);
        assert_eq!(parsed.forked_origin, record.forked_origin);
        assert_eq!(parsed.fork_seq, record.fork_seq);
        assert_eq!(parsed.fork_genesis, record.fork_genesis);
        assert_eq!(parsed.from_snapshot_seq, None);
        assert!(parsed.verify());
    }

    #[test]
    fn test_fork_record_with_snapshot_seq() {
        let (_, record, _) = fork_entity(0xBEEF, 200, Some(150));

        let bytes = record.to_bytes();
        let parsed = ForkRecord::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.from_snapshot_seq, Some(150));
    }

    #[test]
    fn test_tampered_sentinel_fails_verification() {
        let (_, mut record, _) = fork_entity(0xAAAA, 100, None);
        record.fork_genesis.parent_hash = 0xBADBADBAD;
        assert!(!record.verify());
    }
}
