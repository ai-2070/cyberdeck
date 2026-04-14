//! Observation window — enriched horizon with temporal bounds.
//!
//! Wraps `ObservedHorizon` with propagation context: when each entity
//! was last observed, estimated delay based on hop count and subnet distance.

use std::collections::HashMap;
use std::time::Duration;

use crate::adapter::bltp::state::horizon::ObservedHorizon;
use crate::adapter::bltp::subnet::SubnetId;

use super::propagation::PropagationModel;

/// Enriched observation horizon with temporal context.
///
/// Tracks not just what was observed (origin_hash, sequence) but when
/// and how far away each entity is (staleness, estimated delay).
pub struct ObservationWindow {
    /// The underlying horizon data.
    horizon: ObservedHorizon,
    /// This observer's subnet.
    local_subnet: SubnetId,
    /// Last time each entity was directly observed (local nanos timestamp).
    last_observed: HashMap<u32, u64>,
    /// Estimated propagation delay per entity (nanos).
    estimated_delay: HashMap<u32, u64>,
}

impl ObservationWindow {
    /// Create a new observation window.
    pub fn new(local_subnet: SubnetId) -> Self {
        Self {
            horizon: ObservedHorizon::new(),
            local_subnet,
            last_observed: HashMap::new(),
            estimated_delay: HashMap::new(),
        }
    }

    /// Create from an existing horizon.
    pub fn from_horizon(horizon: ObservedHorizon, local_subnet: SubnetId) -> Self {
        Self {
            horizon,
            local_subnet,
            last_observed: HashMap::new(),
            estimated_delay: HashMap::new(),
        }
    }

    /// Record an observation with propagation context.
    pub fn observe_with_context(
        &mut self,
        origin_hash: u32,
        sequence: u64,
        hop_count: u8,
        source_subnet: SubnetId,
        model: &PropagationModel,
    ) {
        self.horizon.observe(origin_hash, sequence);

        let now = current_timestamp();
        self.last_observed.insert(origin_hash, now);

        let delay = model
            .estimate_latency(source_subnet, self.local_subnet, hop_count)
            .as_nanos() as u64;
        self.estimated_delay.insert(origin_hash, delay);
    }

    /// Simple observation (no propagation context).
    pub fn observe(&mut self, origin_hash: u32, sequence: u64) {
        self.horizon.observe(origin_hash, sequence);
        self.last_observed.insert(origin_hash, current_timestamp());
    }

    /// How long since an entity was last observed.
    pub fn staleness(&self, origin_hash: u32) -> Option<Duration> {
        self.last_observed.get(&origin_hash).map(|&ts| {
            let now = current_timestamp();
            Duration::from_nanos(now.saturating_sub(ts))
        })
    }

    /// Whether an entity is within the observer's causal cone
    /// (observed recently enough given propagation delay).
    pub fn is_within_cone(&self, origin_hash: u32, max_delay_nanos: u64) -> bool {
        let delay = self.estimated_delay.get(&origin_hash).copied().unwrap_or(0);
        let staleness = match self.staleness(origin_hash) {
            Some(s) => s.as_nanos() as u64,
            None => return false,
        };
        staleness <= max_delay_nanos.saturating_add(delay)
    }

    /// Entities observed within a given hop radius.
    pub fn reachable_entities(&self, max_delay_nanos: u64) -> Vec<u32> {
        self.last_observed
            .keys()
            .filter(|&&origin| self.is_within_cone(origin, max_delay_nanos))
            .copied()
            .collect()
    }

    /// Quantify how different two observers' views are.
    pub fn divergence_from(&self, other: &ObservationWindow) -> HorizonDivergence {
        let mut only_self = 0u32;
        let mut only_other = 0u32;
        let mut seq_diff_sum = 0u64;
        let mut common = 0u32;

        for (&origin, &self_seq) in self.horizon.iter() {
            match other.horizon.get(origin) {
                Some(other_seq) => {
                    common += 1;
                    seq_diff_sum += self_seq.abs_diff(other_seq);
                }
                None => only_self += 1,
            }
        }

        for &origin in other.horizon.iter().map(|(k, _)| k) {
            if self.horizon.get(origin).is_none() {
                only_other += 1;
            }
        }

        HorizonDivergence {
            entities_only_self: only_self,
            entities_only_other: only_other,
            common_entities: common,
            total_seq_difference: seq_diff_sum,
        }
    }

    /// Get the underlying horizon.
    pub fn horizon(&self) -> &ObservedHorizon {
        &self.horizon
    }

    /// Get the local subnet.
    pub fn local_subnet(&self) -> SubnetId {
        self.local_subnet
    }

    /// Number of observed entities.
    pub fn entity_count(&self) -> usize {
        self.horizon.entity_count()
    }
}

impl std::fmt::Debug for ObservationWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObservationWindow")
            .field("subnet", &self.local_subnet)
            .field("entities", &self.horizon.entity_count())
            .field("tracked", &self.last_observed.len())
            .finish()
    }
}

/// Quantified divergence between two observation windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HorizonDivergence {
    /// Entities seen by self but not other.
    pub entities_only_self: u32,
    /// Entities seen by other but not self.
    pub entities_only_other: u32,
    /// Entities seen by both.
    pub common_entities: u32,
    /// Sum of sequence number differences for common entities.
    pub total_seq_difference: u64,
}

impl HorizonDivergence {
    /// Whether the two windows are completely in agreement.
    pub fn is_converged(&self) -> bool {
        self.entities_only_self == 0
            && self.entities_only_other == 0
            && self.total_seq_difference == 0
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe_and_staleness() {
        let mut window = ObservationWindow::new(SubnetId::new(&[1]));
        window.observe(0xAAAA, 10);

        let staleness = window.staleness(0xAAAA);
        assert!(staleness.is_some());
        // Should be very recent
        assert!(staleness.unwrap() < Duration::from_secs(1));

        // Unknown entity
        assert!(window.staleness(0xBBBB).is_none());
    }

    #[test]
    fn test_observe_with_context() {
        let model = PropagationModel::new();
        let mut window = ObservationWindow::new(SubnetId::new(&[1]));

        window.observe_with_context(0xAAAA, 42, 2, SubnetId::new(&[1, 2]), &model);

        assert_eq!(window.entity_count(), 1);
        assert!(window.horizon().has_observed(0xAAAA, 42));
    }

    #[test]
    fn test_is_within_cone() {
        let mut window = ObservationWindow::new(SubnetId::new(&[1]));
        window.observe(0xAAAA, 10);

        // Just observed — should be within any reasonable cone
        assert!(window.is_within_cone(0xAAAA, 1_000_000_000)); // 1 second

        // Unobserved entity
        assert!(!window.is_within_cone(0xBBBB, 1_000_000_000));
    }

    #[test]
    fn test_divergence_identical() {
        let mut w1 = ObservationWindow::new(SubnetId::new(&[1]));
        let mut w2 = ObservationWindow::new(SubnetId::new(&[1]));

        w1.observe(0xAAAA, 10);
        w2.observe(0xAAAA, 10);

        let div = w1.divergence_from(&w2);
        assert!(div.is_converged());
    }

    #[test]
    fn test_divergence_different() {
        let mut w1 = ObservationWindow::new(SubnetId::new(&[1]));
        let mut w2 = ObservationWindow::new(SubnetId::new(&[2]));

        w1.observe(0xAAAA, 10);
        w1.observe(0xBBBB, 5);
        w2.observe(0xAAAA, 15);
        w2.observe(0xCCCC, 20);

        let div = w1.divergence_from(&w2);
        assert_eq!(div.common_entities, 1); // 0xAAAA
        assert_eq!(div.entities_only_self, 1); // 0xBBBB
        assert_eq!(div.entities_only_other, 1); // 0xCCCC
        assert_eq!(div.total_seq_difference, 5); // |10 - 15|
    }

    #[test]
    fn test_reachable_entities() {
        let mut window = ObservationWindow::new(SubnetId::new(&[1]));
        window.observe(0xAAAA, 10);
        window.observe(0xBBBB, 20);

        let reachable = window.reachable_entities(1_000_000_000);
        assert_eq!(reachable.len(), 2);
    }
}
