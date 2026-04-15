//! Partition detection and healing.
//!
//! Detects when a mass failure is actually a network partition (asymmetric
//! visibility), tracks partition state, and detects healing when nodes
//! from the other side reappear.

use std::time::Instant;

use super::correlation::{CorrelationVerdict, FailureCause};
use crate::adapter::nltp::state::horizon::ObservedHorizon;
use crate::adapter::nltp::subnet::SubnetId;

/// Lifecycle phase of a detected partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionPhase {
    /// Partition suspected but not confirmed.
    Suspected,
    /// Partition confirmed (other side is alive but unreachable).
    Confirmed,
    /// Partition healing — some nodes reappearing.
    Healing {
        /// Nodes from the other side that have reappeared.
        reappeared: Vec<u64>,
    },
    /// Partition healed, reconciliation needed.
    Healed,
}

/// Record of a detected partition.
#[derive(Debug, Clone)]
pub struct PartitionRecord {
    /// Unique partition ID (timestamp-based).
    id: u64,
    /// When the partition was detected.
    detected_at: Instant,
    /// Nodes we can still reach.
    our_side: Vec<u64>,
    /// Nodes we lost.
    other_side: Vec<u64>,
    /// Subnet where the failure was concentrated (if identified).
    partition_subnet: Option<SubnetId>,
    /// Current phase.
    phase: PartitionPhase,
    /// Snapshot of our ObservedHorizon at partition time (reconciliation baseline).
    our_horizon_at_split: ObservedHorizon,
}

impl PartitionRecord {
    /// Get the partition ID.
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get our side nodes.
    pub fn our_side(&self) -> &[u64] {
        &self.our_side
    }

    /// Get the other side nodes.
    pub fn other_side(&self) -> &[u64] {
        &self.other_side
    }

    /// Get the partition subnet.
    pub fn partition_subnet(&self) -> Option<SubnetId> {
        self.partition_subnet
    }

    /// Get the current phase.
    pub fn phase(&self) -> &PartitionPhase {
        &self.phase
    }

    /// Get the horizon snapshot at split time.
    pub fn horizon_at_split(&self) -> &ObservedHorizon {
        &self.our_horizon_at_split
    }

    /// How long the partition has been active.
    pub fn duration(&self) -> std::time::Duration {
        self.detected_at.elapsed()
    }

    /// Fraction of the other side that has reappeared.
    pub fn healing_progress(&self) -> f32 {
        if self.other_side.is_empty() {
            return 1.0;
        }
        match &self.phase {
            PartitionPhase::Healing { reappeared } => {
                reappeared.len() as f32 / self.other_side.len() as f32
            }
            PartitionPhase::Healed => 1.0,
            _ => 0.0,
        }
    }
}

/// Partition detector.
///
/// Tracks active partitions and detects healing when nodes reappear.
pub struct PartitionDetector {
    /// Active partition records.
    active_partitions: Vec<PartitionRecord>,
    /// Healing threshold — fraction of other_side that must reappear
    /// for the partition to be considered healed.
    healing_threshold: f32,
    /// Counter for generating partition IDs.
    next_id: u64,
}

impl PartitionDetector {
    /// Create a new partition detector.
    pub fn new() -> Self {
        Self {
            active_partitions: Vec::new(),
            healing_threshold: 0.50,
            next_id: 1,
        }
    }

    /// Set the healing threshold (fraction of other_side that must reappear).
    pub fn with_healing_threshold(mut self, threshold: f32) -> Self {
        self.healing_threshold = threshold;
        self
    }

    /// Attempt to detect a partition from a correlation verdict.
    ///
    /// Only creates a partition record for `MassFailure` with `SubnetFailure` cause.
    /// Returns the partition ID if created.
    pub fn detect(
        &mut self,
        verdict: &CorrelationVerdict,
        healthy_nodes: &[u64],
        current_horizon: &ObservedHorizon,
    ) -> Option<u64> {
        let (failed_nodes, cause) = match verdict {
            CorrelationVerdict::MassFailure {
                failed_nodes,
                suspected_cause,
                ..
            } => (failed_nodes, suspected_cause),
            _ => return None,
        };

        let partition_subnet = match cause {
            FailureCause::SubnetFailure { subnet, .. } => Some(*subnet),
            _ => return None, // broad outage, not a partition
        };

        let id = self.next_id;
        self.next_id += 1;

        let record = PartitionRecord {
            id,
            detected_at: Instant::now(),
            our_side: healthy_nodes.to_vec(),
            other_side: failed_nodes.clone(),
            partition_subnet,
            phase: PartitionPhase::Suspected,
            our_horizon_at_split: current_horizon.clone(),
        };

        self.active_partitions.push(record);
        Some(id)
    }

    /// Confirm a partition (e.g., received gossip that other side is alive).
    pub fn confirm(&mut self, partition_id: u64) -> bool {
        if let Some(record) = self.find_mut(partition_id) {
            if record.phase == PartitionPhase::Suspected {
                record.phase = PartitionPhase::Confirmed;
                return true;
            }
        }
        false
    }

    /// Record that a node has recovered (reappeared after failure).
    ///
    /// If the node was in any partition's `other_side`, transitions
    /// the partition toward healing.
    pub fn on_node_recovery(&mut self, node_id: u64) {
        for record in &mut self.active_partitions {
            if !record.other_side.contains(&node_id) {
                continue;
            }

            match &mut record.phase {
                PartitionPhase::Suspected | PartitionPhase::Confirmed => {
                    record.phase = PartitionPhase::Healing {
                        reappeared: vec![node_id],
                    };
                }
                PartitionPhase::Healing { reappeared } => {
                    if !reappeared.contains(&node_id) {
                        reappeared.push(node_id);
                    }
                }
                PartitionPhase::Healed => {}
            }

            // Check if healed (after any phase transition)
            if let PartitionPhase::Healing { reappeared } = &record.phase {
                let ratio = reappeared.len() as f32 / record.other_side.len() as f32;
                if ratio >= self.healing_threshold {
                    record.phase = PartitionPhase::Healed;
                }
            }
        }
    }

    /// Take all partitions that have healed (drains them from active list).
    pub fn take_healed(&mut self) -> Vec<PartitionRecord> {
        let mut healed = Vec::new();
        self.active_partitions.retain(|r| {
            if r.phase == PartitionPhase::Healed {
                healed.push(r.clone());
                false
            } else {
                true
            }
        });
        healed
    }

    /// Number of active partitions.
    pub fn active_count(&self) -> usize {
        self.active_partitions.len()
    }

    /// Get an active partition by ID.
    pub fn get(&self, partition_id: u64) -> Option<&PartitionRecord> {
        self.active_partitions.iter().find(|r| r.id == partition_id)
    }

    fn find_mut(&mut self, partition_id: u64) -> Option<&mut PartitionRecord> {
        self.active_partitions
            .iter_mut()
            .find(|r| r.id == partition_id)
    }
}

impl Default for PartitionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PartitionDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartitionDetector")
            .field("active_partitions", &self.active_partitions.len())
            .field("healing_threshold", &self.healing_threshold)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_verdict_subnet(failed: Vec<u64>, subnet: SubnetId) -> CorrelationVerdict {
        CorrelationVerdict::MassFailure {
            failed_nodes: failed,
            failure_ratio: 0.5,
            suspected_cause: FailureCause::SubnetFailure {
                subnet,
                affected_ratio: 1.0,
            },
        }
    }

    fn make_verdict_broad(failed: Vec<u64>) -> CorrelationVerdict {
        CorrelationVerdict::MassFailure {
            failed_nodes: failed,
            failure_ratio: 0.5,
            suspected_cause: FailureCause::BroadOutage,
        }
    }

    #[test]
    fn test_detect_partition() {
        let mut det = PartitionDetector::new();
        let horizon = ObservedHorizon::new();
        let verdict = make_verdict_subnet(vec![1, 2, 3], SubnetId::new(&[2]));

        let id = det.detect(&verdict, &[4, 5, 6], &horizon);
        assert!(id.is_some());
        assert_eq!(det.active_count(), 1);

        let record = det.get(id.unwrap()).unwrap();
        assert_eq!(record.other_side(), &[1, 2, 3]);
        assert_eq!(record.our_side(), &[4, 5, 6]);
        assert_eq!(record.phase(), &PartitionPhase::Suspected);
    }

    #[test]
    fn test_no_partition_for_broad_outage() {
        let mut det = PartitionDetector::new();
        let horizon = ObservedHorizon::new();
        let verdict = make_verdict_broad(vec![1, 2, 3]);

        let id = det.detect(&verdict, &[4, 5, 6], &horizon);
        assert!(id.is_none());
        assert_eq!(det.active_count(), 0);
    }

    #[test]
    fn test_no_partition_for_independent() {
        let mut det = PartitionDetector::new();
        let horizon = ObservedHorizon::new();
        let verdict = CorrelationVerdict::Independent {
            failed_nodes: vec![1],
        };

        let id = det.detect(&verdict, &[2, 3], &horizon);
        assert!(id.is_none());
    }

    #[test]
    fn test_confirm() {
        let mut det = PartitionDetector::new();
        let horizon = ObservedHorizon::new();
        let verdict = make_verdict_subnet(vec![1, 2], SubnetId::new(&[2]));

        let id = det.detect(&verdict, &[3, 4], &horizon).unwrap();
        assert!(det.confirm(id));
        assert_eq!(det.get(id).unwrap().phase(), &PartitionPhase::Confirmed);
    }

    #[test]
    fn test_healing() {
        let mut det = PartitionDetector::new().with_healing_threshold(0.50);
        let horizon = ObservedHorizon::new();
        let verdict = make_verdict_subnet(vec![1, 2, 3, 4], SubnetId::new(&[2]));

        let id = det.detect(&verdict, &[5, 6], &horizon).unwrap();

        // First recovery — not healed yet
        det.on_node_recovery(1);
        assert!(matches!(
            det.get(id).unwrap().phase(),
            PartitionPhase::Healing { .. }
        ));

        // Second recovery — 2/4 = 50% >= threshold
        det.on_node_recovery(2);
        assert_eq!(det.get(id).unwrap().phase(), &PartitionPhase::Healed);
    }

    #[test]
    fn test_take_healed() {
        let mut det = PartitionDetector::new().with_healing_threshold(0.50);
        let horizon = ObservedHorizon::new();
        let verdict = make_verdict_subnet(vec![1, 2], SubnetId::new(&[2]));

        det.detect(&verdict, &[3, 4], &horizon);

        det.on_node_recovery(1); // 1/2 = 50% >= threshold → healed

        let healed = det.take_healed();
        assert_eq!(healed.len(), 1);
        assert_eq!(det.active_count(), 0);
    }

    #[test]
    fn test_healing_progress() {
        let mut det = PartitionDetector::new().with_healing_threshold(0.75);
        let horizon = ObservedHorizon::new();
        let verdict = make_verdict_subnet(vec![1, 2, 3, 4], SubnetId::new(&[2]));

        let id = det.detect(&verdict, &[5], &horizon).unwrap();
        assert_eq!(det.get(id).unwrap().healing_progress(), 0.0);

        det.on_node_recovery(1);
        assert_eq!(det.get(id).unwrap().healing_progress(), 0.25);

        det.on_node_recovery(2);
        assert_eq!(det.get(id).unwrap().healing_progress(), 0.50);
    }

    #[test]
    fn test_duplicate_recovery_ignored() {
        let mut det = PartitionDetector::new().with_healing_threshold(0.75);
        let horizon = ObservedHorizon::new();
        let verdict = make_verdict_subnet(vec![1, 2, 3, 4], SubnetId::new(&[2]));

        let id = det.detect(&verdict, &[5], &horizon).unwrap();

        det.on_node_recovery(1);
        det.on_node_recovery(1); // duplicate
        assert_eq!(det.get(id).unwrap().healing_progress(), 0.25); // still 1/4
    }
}
