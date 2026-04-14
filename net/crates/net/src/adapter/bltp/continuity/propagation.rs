//! Propagation-speed awareness.
//!
//! Models estimated latency between subnets based on hierarchy depth
//! and hop count. Self-calibrating from actual RTT measurements.

use std::time::Duration;

use crate::adapter::bltp::subnet::SubnetId;

/// Estimated propagation latency model.
///
/// Maps subnet hierarchy distance + hop count to estimated latency.
/// Default multipliers reflect typical mesh deployments:
/// - Same subsystem: 1x (sub-millisecond)
/// - Cross-subsystem within vehicle: 5x
/// - Cross-vehicle within fleet: 50x
/// - Cross-region: 500x
pub struct PropagationModel {
    /// Base per-hop latency in nanoseconds.
    pub base_hop_latency_nanos: u64,
    /// Multiplier per subnet level crossed (index = crossing depth).
    pub level_multipliers: [f32; 4],
    /// Accumulated calibration samples for self-tuning.
    sample_count: u64,
}

impl PropagationModel {
    /// Default base latency: 100 microseconds per hop.
    pub const DEFAULT_BASE_HOP_NANOS: u64 = 100_000;

    /// Default level multipliers.
    pub const DEFAULT_MULTIPLIERS: [f32; 4] = [1.0, 5.0, 50.0, 500.0];

    /// Create with default parameters.
    pub fn new() -> Self {
        Self {
            base_hop_latency_nanos: Self::DEFAULT_BASE_HOP_NANOS,
            level_multipliers: Self::DEFAULT_MULTIPLIERS,
            sample_count: 0,
        }
    }

    /// Create with custom base latency.
    pub fn with_base_latency(base_nanos: u64) -> Self {
        Self {
            base_hop_latency_nanos: base_nanos,
            level_multipliers: Self::DEFAULT_MULTIPLIERS,
            sample_count: 0,
        }
    }

    /// Estimate latency between two subnets given a hop count.
    pub fn estimate_latency(&self, source: SubnetId, dest: SubnetId, hop_count: u8) -> Duration {
        let depth = crossing_depth(source, dest);
        let multiplier = if (depth as usize) < self.level_multipliers.len() {
            self.level_multipliers[depth as usize]
        } else {
            *self.level_multipliers.last().unwrap_or(&1.0)
        };

        let hops = if hop_count == 0 { 1 } else { hop_count as u64 };
        let nanos = (self.base_hop_latency_nanos as f64 * hops as f64 * multiplier as f64) as u64;
        Duration::from_nanos(nanos)
    }

    /// How many subnet levels differ between two IDs.
    ///
    /// 0 = same subnet, 1 = same parent different at deepest level, etc.
    /// Uses `crossing_depth` free function.
    pub fn crossing_depth(source: SubnetId, dest: SubnetId) -> u8 {
        crossing_depth(source, dest)
    }

    /// Calibrate from an actual RTT measurement.
    ///
    /// Adjusts `base_hop_latency_nanos` as an exponentially weighted
    /// moving average of observed measurements.
    pub fn calibrate(
        &mut self,
        _source: SubnetId,
        _dest: SubnetId,
        hop_count: u8,
        measured_rtt_nanos: u64,
    ) {
        if hop_count == 0 {
            return;
        }

        // Factor out the depth multiplier before updating base_hop_latency
        let depth = crossing_depth(_source, _dest);
        let multiplier = if (depth as usize) < self.level_multipliers.len() {
            self.level_multipliers[depth as usize]
        } else {
            *self.level_multipliers.last().unwrap_or(&1.0)
        };
        if multiplier == 0.0 {
            return;
        }

        // Compute implied per-hop base latency: RTT / 2 / hops / multiplier
        let per_hop =
            (measured_rtt_nanos as f64 / (2.0 * hop_count as f64 * multiplier as f64)) as u64;
        let alpha = if self.sample_count < 10 { 0.5 } else { 0.1 };

        self.base_hop_latency_nanos =
            (self.base_hop_latency_nanos as f64 * (1.0 - alpha) + per_hop as f64 * alpha) as u64;
        self.sample_count += 1;
    }

    /// Maximum subnet depth reachable within a latency budget.
    pub fn max_depth_within(&self, max_latency: Duration, hop_count: u8) -> u8 {
        let budget_nanos = max_latency.as_nanos() as u64;
        let hops = if hop_count == 0 { 1 } else { hop_count as u64 };

        for depth in (0..4u8).rev() {
            let multiplier = self.level_multipliers[depth as usize];
            let estimated =
                (self.base_hop_latency_nanos as f64 * hops as f64 * multiplier as f64) as u64;
            if estimated <= budget_nanos {
                return depth;
            }
        }
        0
    }
}

impl Default for PropagationModel {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PropagationModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropagationModel")
            .field("base_hop_nanos", &self.base_hop_latency_nanos)
            .field("multipliers", &self.level_multipliers)
            .field("samples", &self.sample_count)
            .finish()
    }
}

/// How many subnet levels differ between two subnet IDs.
///
/// Returns 0 if identical, 1 if they differ at the deepest common level, etc.
/// Global (0) is considered depth 0 from everything.
pub fn crossing_depth(a: SubnetId, b: SubnetId) -> u8 {
    if a.is_same_subnet(b) {
        return 0;
    }
    if a.is_global() || b.is_global() {
        return a.depth().max(b.depth());
    }

    // Find the first level where they differ
    for level in 0..4u8 {
        if a.level(level) != b.level(level) {
            // They differ at this level. Crossing depth = max_depth - level.
            let max_depth = a.depth().max(b.depth());
            return max_depth.saturating_sub(level);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossing_depth_same() {
        let a = SubnetId::new(&[1, 2, 3]);
        assert_eq!(crossing_depth(a, a), 0);
    }

    #[test]
    fn test_crossing_depth_sibling() {
        let a = SubnetId::new(&[1, 2]);
        let b = SubnetId::new(&[1, 3]);
        assert_eq!(crossing_depth(a, b), 1); // differ at level 1, depth 2
    }

    #[test]
    fn test_crossing_depth_different_region() {
        let a = SubnetId::new(&[1, 2]);
        let b = SubnetId::new(&[2, 3]);
        assert_eq!(crossing_depth(a, b), 2); // differ at level 0, depth 2
    }

    #[test]
    fn test_crossing_depth_global() {
        let a = SubnetId::GLOBAL;
        let b = SubnetId::new(&[1, 2, 3]);
        assert_eq!(crossing_depth(a, b), 3);
    }

    #[test]
    fn test_estimate_latency_same_subnet() {
        let model = PropagationModel::new();
        let subnet = SubnetId::new(&[1, 2]);
        let latency = model.estimate_latency(subnet, subnet, 1);
        assert_eq!(latency, Duration::from_nanos(100_000)); // 1 hop * 1.0x
    }

    #[test]
    fn test_estimate_latency_cross_region() {
        let model = PropagationModel::new();
        let a = SubnetId::new(&[1, 1]);
        let b = SubnetId::new(&[2, 1]);
        // crossing_depth = 2 (differ at level 0, max depth 2), multiplier[2] = 50.0
        // 5 hops * 100us * 50 = 25ms
        let latency = model.estimate_latency(a, b, 5);
        assert!(latency > Duration::from_millis(20));
    }

    #[test]
    fn test_calibrate() {
        let mut model = PropagationModel::new();
        let a = SubnetId::new(&[1]);
        let b = SubnetId::new(&[1, 2]);

        // Measure a 20us RTT over 2 hops → implied per-hop = 5us
        // Base should move toward 5us from default 100us
        model.calibrate(a, b, 2, 20_000);
        assert!(model.base_hop_latency_nanos < PropagationModel::DEFAULT_BASE_HOP_NANOS);
    }

    #[test]
    fn test_max_depth_within() {
        let model = PropagationModel::new();
        // With default 100us base and 1 hop:
        // depth 0 = 100us, depth 1 = 500us, depth 2 = 5ms, depth 3 = 50ms
        let depth = model.max_depth_within(Duration::from_millis(1), 1);
        assert!(depth >= 1); // 500us fits within 1ms
    }
}
