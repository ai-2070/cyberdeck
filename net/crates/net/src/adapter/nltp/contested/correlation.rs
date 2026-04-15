//! Correlated failure detection.
//!
//! Wraps `FailureDetector` with a time-windowed correlation layer.
//! Classifies failures as independent or correlated (mass failure),
//! and identifies whether failures are concentrated in a subnet
//! (likely partition) or spread broadly (likely infrastructure outage).

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::adapter::nltp::subnet::SubnetId;

/// Configuration for correlated failure detection.
#[derive(Debug, Clone)]
pub struct CorrelatedFailureConfig {
    /// Time window for correlating failures.
    pub correlation_window: Duration,
    /// Fraction of tracked nodes failing within the window to trigger
    /// mass-failure classification (0.0 - 1.0).
    pub mass_failure_threshold: f32,
    /// If this fraction of failures in the window share a common subnet
    /// ancestor, classify as subnet-correlated (likely partition).
    pub subnet_correlation_threshold: f32,
    /// Maximum concurrent recovery actions during mass failure.
    pub max_concurrent_migrations: usize,
}

impl Default for CorrelatedFailureConfig {
    fn default() -> Self {
        Self {
            correlation_window: Duration::from_secs(2),
            mass_failure_threshold: 0.30,
            subnet_correlation_threshold: 0.80,
            max_concurrent_migrations: 3,
        }
    }
}

/// A recorded failure event within the correlation window.
#[derive(Debug, Clone)]
struct FailureEvent {
    node_id: u64,
    detected_at: Instant,
    _subnet: Option<SubnetId>,
}

/// Verdict from correlated failure analysis.
#[derive(Debug, Clone)]
pub enum CorrelationVerdict {
    /// Independent failures — handle normally via RecoveryManager.
    Independent {
        /// Nodes that failed.
        failed_nodes: Vec<u64>,
    },
    /// Mass correlated failure — throttle recovery.
    MassFailure {
        /// Nodes that failed.
        failed_nodes: Vec<u64>,
        /// Fraction of tracked nodes that failed.
        failure_ratio: f32,
        /// Suspected root cause.
        suspected_cause: FailureCause,
    },
}

impl CorrelationVerdict {
    /// Get the failed nodes regardless of verdict type.
    pub fn failed_nodes(&self) -> &[u64] {
        match self {
            Self::Independent { failed_nodes } => failed_nodes,
            Self::MassFailure { failed_nodes, .. } => failed_nodes,
        }
    }

    /// Whether this is a mass failure.
    pub fn is_mass_failure(&self) -> bool {
        matches!(self, Self::MassFailure { .. })
    }
}

/// Suspected cause of a mass failure.
#[derive(Debug, Clone, PartialEq)]
pub enum FailureCause {
    /// Failures concentrated in a single subnet (likely partition).
    SubnetFailure {
        /// The subnet ancestor where failures are concentrated.
        subnet: SubnetId,
        /// Fraction of failures in this subnet.
        affected_ratio: f32,
    },
    /// Failures spread across subnets (likely infrastructure outage).
    BroadOutage,
    /// Insufficient subnet data to determine cause.
    Unknown,
}

/// Correlated failure detector.
///
/// Sits alongside `FailureDetector` as a correlation layer. Consumes
/// failure events and classifies them as independent or correlated.
pub struct CorrelatedFailureDetector {
    config: CorrelatedFailureConfig,
    /// Recent failures within the correlation window.
    recent_failures: VecDeque<FailureEvent>,
    /// Node -> subnet mapping for correlation analysis.
    node_subnets: HashMap<u64, SubnetId>,
    /// Whether we're currently in mass-failure mode.
    in_mass_failure: bool,
}

impl CorrelatedFailureDetector {
    /// Create a new detector with the given configuration.
    pub fn new(config: CorrelatedFailureConfig) -> Self {
        Self {
            config,
            recent_failures: VecDeque::new(),
            node_subnets: HashMap::new(),
            in_mass_failure: false,
        }
    }

    /// Register a node's subnet for correlation analysis.
    pub fn register_node(&mut self, node_id: u64, subnet: SubnetId) {
        self.node_subnets.insert(node_id, subnet);
    }

    /// Record new failures and classify them.
    ///
    /// Call this after `FailureDetector::check_all()` with the newly
    /// failed nodes and the total number of tracked nodes.
    pub fn record_failures(
        &mut self,
        failed_nodes: &[u64],
        total_tracked: usize,
    ) -> CorrelationVerdict {
        let now = Instant::now();

        // Record new failures
        for &node_id in failed_nodes {
            self.recent_failures.push_back(FailureEvent {
                node_id,
                detected_at: now,
                _subnet: self.node_subnets.get(&node_id).copied(),
            });
        }

        // Prune events older than the correlation window
        let cutoff = now - self.config.correlation_window;
        while self
            .recent_failures
            .front()
            .is_some_and(|e| e.detected_at < cutoff)
        {
            self.recent_failures.pop_front();
        }

        // Count unique failures in the window
        let window_failures: Vec<u64> = self
            .recent_failures
            .iter()
            .map(|e| e.node_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if total_tracked == 0 {
            return CorrelationVerdict::Independent {
                failed_nodes: failed_nodes.to_vec(),
            };
        }

        let failure_ratio = window_failures.len() as f32 / total_tracked as f32;

        if failure_ratio < self.config.mass_failure_threshold {
            self.in_mass_failure = false;
            return CorrelationVerdict::Independent {
                failed_nodes: failed_nodes.to_vec(),
            };
        }

        // Mass failure detected — analyze subnet correlation
        self.in_mass_failure = true;
        let cause = self.analyze_subnet_correlation(&window_failures);

        CorrelationVerdict::MassFailure {
            failed_nodes: window_failures,
            failure_ratio,
            suspected_cause: cause,
        }
    }

    /// How many concurrent recovery actions are allowed.
    ///
    /// Throttled during mass failure to avoid overloading survivors.
    pub fn recovery_budget(&self) -> usize {
        if self.in_mass_failure {
            self.config.max_concurrent_migrations
        } else {
            usize::MAX
        }
    }

    /// Whether we're currently in mass-failure mode.
    pub fn in_mass_failure(&self) -> bool {
        self.in_mass_failure
    }

    /// Clear the failure window (e.g., when conditions normalize).
    pub fn clear_window(&mut self) {
        self.recent_failures.clear();
        self.in_mass_failure = false;
    }

    /// Number of failures in the current window.
    pub fn window_size(&self) -> usize {
        self.recent_failures.len()
    }

    /// Analyze whether failures are concentrated in a subnet subtree.
    fn analyze_subnet_correlation(&self, failed_nodes: &[u64]) -> FailureCause {
        let mut subnet_counts: HashMap<SubnetId, usize> = HashMap::new();
        let mut with_subnet = 0usize;

        for &node_id in failed_nodes {
            if let Some(&subnet) = self.node_subnets.get(&node_id) {
                with_subnet += 1;
                // Count at each hierarchy level
                let mut current = subnet;
                loop {
                    *subnet_counts.entry(current).or_insert(0) += 1;
                    let parent = current.parent();
                    if parent == current || parent.is_global() {
                        break;
                    }
                    current = parent;
                }
            }
        }

        if with_subnet == 0 {
            return FailureCause::Unknown;
        }

        // Find the most specific subnet with the highest concentration
        // Ceiling to avoid false subnet correlation from rounding down
        let threshold =
            (with_subnet as f32 * self.config.subnet_correlation_threshold).ceil() as usize;

        let mut best_subnet = None;
        let mut best_depth = 0u8;

        for (&subnet, &count) in &subnet_counts {
            if count >= threshold && subnet.depth() >= best_depth {
                best_subnet = Some(subnet);
                best_depth = subnet.depth();
            }
        }

        match best_subnet {
            Some(subnet) => {
                let ratio = *subnet_counts.get(&subnet).unwrap() as f32 / with_subnet as f32;
                FailureCause::SubnetFailure {
                    subnet,
                    affected_ratio: ratio,
                }
            }
            None => FailureCause::BroadOutage,
        }
    }
}

impl std::fmt::Debug for CorrelatedFailureDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CorrelatedFailureDetector")
            .field("window_size", &self.recent_failures.len())
            .field("tracked_nodes", &self.node_subnets.len())
            .field("in_mass_failure", &self.in_mass_failure)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_detector(threshold: f32) -> CorrelatedFailureDetector {
        CorrelatedFailureDetector::new(CorrelatedFailureConfig {
            mass_failure_threshold: threshold,
            ..Default::default()
        })
    }

    #[test]
    fn test_independent_failures() {
        let mut det = make_detector(0.30);
        for i in 0..10 {
            det.register_node(i, SubnetId::new(&[1]));
        }

        // 1 out of 10 fails = 10% < 30%
        let verdict = det.record_failures(&[0], 10);
        assert!(!verdict.is_mass_failure());
        assert!(!det.in_mass_failure());
        assert_eq!(det.recovery_budget(), usize::MAX);
    }

    #[test]
    fn test_mass_failure() {
        let mut det = make_detector(0.30);
        for i in 0..10 {
            det.register_node(i, SubnetId::new(&[1]));
        }

        // 4 out of 10 fails = 40% > 30%
        let verdict = det.record_failures(&[0, 1, 2, 3], 10);
        assert!(verdict.is_mass_failure());
        assert!(det.in_mass_failure());
        assert_eq!(det.recovery_budget(), 3); // default max_concurrent_migrations
    }

    #[test]
    fn test_subnet_correlated() {
        let mut det = make_detector(0.30);
        // 5 nodes in subnet [1, 1], 5 in subnet [1, 2]
        for i in 0..5 {
            det.register_node(i, SubnetId::new(&[1, 1]));
        }
        for i in 5..10 {
            det.register_node(i, SubnetId::new(&[1, 2]));
        }

        // All 5 nodes in subnet [1, 1] fail
        let verdict = det.record_failures(&[0, 1, 2, 3, 4], 10);
        assert!(verdict.is_mass_failure());

        if let CorrelationVerdict::MassFailure {
            suspected_cause, ..
        } = &verdict
        {
            match suspected_cause {
                FailureCause::SubnetFailure { subnet, .. } => {
                    // Should identify subnet [1, 1] as the correlated subnet
                    assert_eq!(*subnet, SubnetId::new(&[1, 1]));
                }
                other => panic!("expected SubnetFailure, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_broad_outage() {
        let mut det = make_detector(0.30);
        // Nodes spread across 4 different subnets
        det.register_node(0, SubnetId::new(&[1]));
        det.register_node(1, SubnetId::new(&[2]));
        det.register_node(2, SubnetId::new(&[3]));
        det.register_node(3, SubnetId::new(&[4]));
        for i in 4..10 {
            det.register_node(i, SubnetId::new(&[(i + 1) as u8]));
        }

        // Failures spread across all subnets
        let verdict = det.record_failures(&[0, 1, 2, 3], 10);
        assert!(verdict.is_mass_failure());

        if let CorrelationVerdict::MassFailure {
            suspected_cause, ..
        } = &verdict
        {
            assert_eq!(*suspected_cause, FailureCause::BroadOutage);
        }
    }

    #[test]
    fn test_clear_window() {
        let mut det = make_detector(0.30);
        for i in 0..10 {
            det.register_node(i, SubnetId::new(&[1]));
        }

        det.record_failures(&[0, 1, 2, 3], 10);
        assert!(det.in_mass_failure());

        det.clear_window();
        assert!(!det.in_mass_failure());
        assert_eq!(det.window_size(), 0);
    }

    #[test]
    fn test_no_subnet_data() {
        let mut det = make_detector(0.30);
        // Don't register any subnets

        let verdict = det.record_failures(&[0, 1, 2, 3], 10);
        assert!(verdict.is_mass_failure());

        if let CorrelationVerdict::MassFailure {
            suspected_cause, ..
        } = &verdict
        {
            assert_eq!(*suspected_cause, FailureCause::Unknown);
        }
    }
}
