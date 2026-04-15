//! Log reconciliation after partition healing.
//!
//! Merges divergent EntityLogs from two sides of a partition using
//! CausalLink chain verification. Longest-chain-wins for conflicts,
//! with deterministic tiebreak. Losing chains become ForkRecords.

use crate::adapter::net::continuity::discontinuity::{fork_entity, ForkRecord};
use crate::adapter::net::state::causal::{validate_chain_link, CausalEvent, ChainError};
use crate::adapter::net::state::log::EntityLog;

/// Which side of the partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Our side (local).
    Ours,
    /// Their side (remote).
    Theirs,
}

/// How a conflict was resolved.
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    /// One side wins (longer chain or deterministic tiebreak).
    Winner {
        /// Which side won.
        winning_side: Side,
        /// Fork record for the losing chain.
        fork_record: ForkRecord,
    },
}

/// Outcome of reconciling one entity's logs.
#[derive(Debug, Clone)]
pub enum ReconcileOutcome {
    /// Logs are identical or one is a strict prefix — no action needed.
    AlreadyConverged,
    /// One side has events the other doesn't, no conflict.
    Catchup {
        /// Entity origin hash.
        origin_hash: u32,
        /// Events to replay on the behind side.
        missing_events: Vec<CausalEvent>,
        /// Which side needs the events.
        behind_side: Side,
    },
    /// Both sides produced events during the split.
    Conflict {
        /// Entity origin hash.
        origin_hash: u32,
        /// Sequence where divergence begins.
        diverge_seq: u64,
        /// Resolution applied.
        resolution: ConflictResolution,
    },
}

/// Reconcile a single entity's log against remote events.
///
/// `our_log`: local EntityLog.
/// `their_events`: events from the remote side (must be chain-valid).
/// `split_seq`: sequence number at the partition point (from horizon snapshot).
pub fn reconcile_entity(
    our_log: &EntityLog,
    their_events: &[CausalEvent],
    split_seq: u64,
) -> ReconcileOutcome {
    let our_events = our_log.after(split_seq);

    // Both sides empty after split — converged
    if our_events.is_empty() && their_events.is_empty() {
        return ReconcileOutcome::AlreadyConverged;
    }

    // Only one side has events — catchup
    if our_events.is_empty() {
        return ReconcileOutcome::Catchup {
            origin_hash: our_log.origin_hash(),
            missing_events: their_events.to_vec(),
            behind_side: Side::Ours,
        };
    }

    if their_events.is_empty() {
        return ReconcileOutcome::Catchup {
            origin_hash: our_log.origin_hash(),
            missing_events: our_events.into_iter().cloned().collect(),
            behind_side: Side::Theirs,
        };
    }

    // Both sides have events — find divergence point
    let mut diverge_idx = None;
    let min_len = our_events.len().min(their_events.len());

    for i in 0..min_len {
        if our_events[i].link.parent_hash != their_events[i].link.parent_hash
            || our_events[i].link.sequence != their_events[i].link.sequence
        {
            diverge_idx = Some(i);
            break;
        }
        // Same link — check payload too
        if our_events[i].payload != their_events[i].payload {
            diverge_idx = Some(i);
            break;
        }
    }

    match diverge_idx {
        None if our_events.len() == their_events.len() => {
            // Identical chains
            ReconcileOutcome::AlreadyConverged
        }
        None if our_events.len() > their_events.len() => {
            // Our chain is longer — they need catchup
            let missing: Vec<CausalEvent> = our_events[their_events.len()..]
                .iter()
                .map(|e| (*e).clone())
                .collect();
            ReconcileOutcome::Catchup {
                origin_hash: our_log.origin_hash(),
                missing_events: missing,
                behind_side: Side::Theirs,
            }
        }
        None => {
            // Their chain is longer — we need catchup
            let missing: Vec<CausalEvent> = their_events[our_events.len()..].to_vec();
            ReconcileOutcome::Catchup {
                origin_hash: our_log.origin_hash(),
                missing_events: missing,
                behind_side: Side::Ours,
            }
        }
        Some(idx) => {
            // Conflict at idx
            let diverge_seq = if idx < our_events.len() {
                our_events[idx].link.sequence
            } else {
                their_events[idx].link.sequence
            };

            let our_len = our_events.len() - idx;
            let their_len = their_events.len() - idx;

            // Longest chain wins. Tie: lower payload hash wins.
            // parent_hash is identical at the divergence point (both diverge
            // from the same parent), so we hash each side's divergent payload
            // for a perspective-independent tiebreak.
            let winning_side = if our_len > their_len {
                Side::Ours
            } else if their_len > our_len {
                Side::Theirs
            } else {
                let our_payload_hash = xxhash_rust::xxh3::xxh3_64(&our_events[idx].payload);
                let their_payload_hash = xxhash_rust::xxh3::xxh3_64(&their_events[idx].payload);
                if our_payload_hash <= their_payload_hash {
                    Side::Ours
                } else {
                    Side::Theirs
                }
            };

            // Fork the losing chain
            let origin_hash = our_log.origin_hash();
            let (_, fork_record, _) = fork_entity(origin_hash, diverge_seq, None);

            ReconcileOutcome::Conflict {
                origin_hash,
                diverge_seq,
                resolution: ConflictResolution::Winner {
                    winning_side,
                    fork_record,
                },
            }
        }
    }
}

/// Validate that a sequence of remote events forms a valid chain.
///
/// Checks parent_hash linkage between consecutive events.
/// Returns Ok if valid, or the first chain error found.
pub fn verify_remote_chain(events: &[CausalEvent]) -> Result<(), ChainError> {
    for i in 1..events.len() {
        validate_chain_link(&events[i - 1].link, &events[i - 1].payload, &events[i].link)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::identity::EntityKeypair;
    use crate::adapter::net::state::causal::CausalChainBuilder;
    use bytes::Bytes;

    fn build_divergent_logs(
        shared_events: usize,
        our_extra: usize,
        their_extra: usize,
    ) -> (EntityLog, Vec<CausalEvent>, u64) {
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let mut log = EntityLog::new(kp.entity_id().clone());
        let mut builder = CausalChainBuilder::new(origin);

        // Shared prefix
        for i in 0..shared_events {
            let event = builder.append(Bytes::from(format!("shared-{}", i)), 0);
            log.append(event).unwrap();
        }

        let split_seq = builder.sequence();

        // Our side continues
        let mut our_builder = CausalChainBuilder::from_head(
            *builder.head(),
            Bytes::from(format!("shared-{}", shared_events - 1)),
        );
        for i in 0..our_extra {
            let event = our_builder.append(Bytes::from(format!("ours-{}", i)), 0);
            log.append(event).unwrap();
        }

        // Their side diverges from the same point
        let mut their_builder = CausalChainBuilder::from_head(
            *builder.head(),
            Bytes::from(format!("shared-{}", shared_events - 1)),
        );
        let mut their_events = Vec::new();
        for i in 0..their_extra {
            let event = their_builder.append(Bytes::from(format!("theirs-{}", i)), 0);
            their_events.push(event);
        }

        (log, their_events, split_seq)
    }

    #[test]
    fn test_already_converged() {
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let mut log = EntityLog::new(kp.entity_id().clone());
        let mut builder = CausalChainBuilder::new(origin);

        for i in 0..5 {
            let event = builder.append(Bytes::from(format!("e{}", i)), 0);
            log.append(event).unwrap();
        }

        // No events after split
        let result = reconcile_entity(&log, &[], 5);
        assert!(matches!(result, ReconcileOutcome::AlreadyConverged));
    }

    #[test]
    fn test_catchup_we_are_behind() {
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let log = EntityLog::new(kp.entity_id().clone());
        let mut builder = CausalChainBuilder::new(origin);

        // They have events, we don't
        let their_events: Vec<CausalEvent> = (0..3)
            .map(|i| builder.append(Bytes::from(format!("theirs-{}", i)), 0))
            .collect();

        let result = reconcile_entity(&log, &their_events, 0);
        match result {
            ReconcileOutcome::Catchup {
                behind_side,
                missing_events,
                ..
            } => {
                assert_eq!(behind_side, Side::Ours);
                assert_eq!(missing_events.len(), 3);
            }
            other => panic!("expected Catchup, got {:?}", other),
        }
    }

    #[test]
    fn test_catchup_they_are_behind() {
        let (log, _, split_seq) = build_divergent_logs(3, 2, 0);

        let result = reconcile_entity(&log, &[], split_seq);
        match result {
            ReconcileOutcome::Catchup {
                behind_side,
                missing_events,
                ..
            } => {
                assert_eq!(behind_side, Side::Theirs);
                assert_eq!(missing_events.len(), 2);
            }
            other => panic!("expected Catchup, got {:?}", other),
        }
    }

    #[test]
    fn test_conflict_longest_wins() {
        let (log, their_events, split_seq) = build_divergent_logs(3, 5, 2);

        let result = reconcile_entity(&log, &their_events, split_seq);
        match result {
            ReconcileOutcome::Conflict {
                resolution:
                    ConflictResolution::Winner {
                        winning_side,
                        fork_record,
                    },
                ..
            } => {
                assert_eq!(winning_side, Side::Ours); // 5 > 2
                assert!(fork_record.verify());
            }
            other => panic!("expected Conflict, got {:?}", other),
        }
    }

    #[test]
    fn test_conflict_they_win() {
        let (log, their_events, split_seq) = build_divergent_logs(3, 1, 4);

        let result = reconcile_entity(&log, &their_events, split_seq);
        match result {
            ReconcileOutcome::Conflict {
                resolution: ConflictResolution::Winner { winning_side, .. },
                ..
            } => {
                assert_eq!(winning_side, Side::Theirs); // 4 > 1
            }
            other => panic!("expected Conflict, got {:?}", other),
        }
    }

    #[test]
    fn test_conflict_tiebreak_deterministic() {
        // Equal length chains — deterministic tiebreak on parent_hash
        let (log, their_events, split_seq) = build_divergent_logs(3, 2, 2);

        let result = reconcile_entity(&log, &their_events, split_seq);
        assert!(matches!(
            result,
            ReconcileOutcome::Conflict {
                resolution: ConflictResolution::Winner { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_verify_remote_chain_valid() {
        let kp = EntityKeypair::generate();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());

        let events: Vec<CausalEvent> = (0..5)
            .map(|i| builder.append(Bytes::from(format!("e{}", i)), 0))
            .collect();

        assert!(verify_remote_chain(&events).is_ok());
    }

    #[test]
    fn test_verify_remote_chain_broken() {
        let kp = EntityKeypair::generate();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());

        let mut events: Vec<CausalEvent> = (0..3)
            .map(|i| builder.append(Bytes::from(format!("e{}", i)), 0))
            .collect();

        // Tamper with the middle event
        events[1].link.parent_hash = 0xBADBADBAD;

        assert!(verify_remote_chain(&events).is_err());
    }

    // ---- Regression tests for Cubic AI findings ----

    #[test]
    fn test_regression_tiebreak_perspective_independent() {
        // Regression: tiebreak used parent_hash, which is identical on both
        // sides of a divergence (both diverge from the same parent). Each
        // side would declare itself the winner. Now uses payload hash.
        let (log, their_events, split_seq) = build_divergent_logs(3, 2, 2);

        let result = reconcile_entity(&log, &their_events, split_seq);

        // The result must be a conflict with a winner
        let winning_side = match &result {
            ReconcileOutcome::Conflict {
                resolution: ConflictResolution::Winner { winning_side, .. },
                ..
            } => *winning_side,
            other => panic!("expected Conflict, got {:?}", other),
        };

        // Now simulate the OTHER side's perspective: they have their_events
        // in their log, and our post-split events are "theirs"
        let our_post_split: Vec<CausalEvent> =
            log.after(split_seq).iter().map(|e| (*e).clone()).collect();

        // Build the other side's log
        let kp = EntityKeypair::from_bytes([0x42u8; 32]); // deterministic for both sides
        let origin = log.origin_hash();
        let mut their_log = EntityLog::new(log.entity_id().clone());
        let mut builder = CausalChainBuilder::new(origin);

        // Replay shared prefix
        for i in 0..3 {
            let event = builder.append(Bytes::from(format!("shared-{}", i)), 0);
            their_log.append(event).unwrap();
        }

        // Replay their divergent events
        let mut their_builder =
            CausalChainBuilder::from_head(*builder.head(), Bytes::from("shared-2".to_string()));
        for event in &their_events {
            their_log.append(event.clone()).unwrap();
        }

        let other_result = reconcile_entity(&their_log, &our_post_split, split_seq);

        let other_winning_side = match &other_result {
            ReconcileOutcome::Conflict {
                resolution: ConflictResolution::Winner { winning_side, .. },
                ..
            } => *winning_side,
            other => panic!("expected Conflict from other side, got {:?}", other),
        };

        // Both sides must agree on the same winner (from their own perspective)
        // If we say "Ours wins", they must say "Theirs wins" (= us), and vice versa.
        assert_ne!(
            winning_side, other_winning_side,
            "both sides must agree: if we say Ours, they must say Theirs"
        );
    }
}
