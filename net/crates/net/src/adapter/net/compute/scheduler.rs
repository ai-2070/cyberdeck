//! Daemon placement scheduler.
//!
//! Connects `CapabilityFilter` requirements to `CapabilityIndex` queries
//! to decide where to run a daemon. Prefers local placement, falls back
//! to the least-loaded candidate.

use std::sync::Arc;

use crate::adapter::net::behavior::capability::{CapabilityFilter, CapabilityIndex, CapabilitySet};
use crate::adapter::net::subprotocol::SubprotocolRegistry;

/// Why a particular node was chosen for placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementReason {
    /// Only node matching the filter.
    OnlyCandidate,
    /// Preferred because it's the local node.
    LocalPreferred,
    /// First candidate from the index (tie-breaking).
    FirstMatch,
    /// Explicitly pinned to a specific node.
    Pinned,
}

/// Result of a placement decision.
#[derive(Debug, Clone)]
pub struct PlacementDecision {
    /// Selected node ID.
    pub node_id: u64,
    /// Why this node was chosen.
    pub reason: PlacementReason,
}

/// Errors from scheduling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerError {
    /// No nodes match the capability filter.
    NoCandidate,
    /// Capability index unavailable.
    IndexUnavailable,
}

impl std::fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCandidate => write!(f, "no nodes match capability requirements"),
            Self::IndexUnavailable => write!(f, "capability index unavailable"),
        }
    }
}

impl std::error::Error for SchedulerError {}

/// Daemon placement scheduler.
///
/// Queries the `CapabilityIndex` to find nodes matching a daemon's
/// requirements. Prefers local placement when possible.
pub struct Scheduler {
    /// Reference to the shared capability index.
    capability_index: Arc<CapabilityIndex>,
    /// This node's ID (for local preference).
    local_node_id: u64,
    /// This node's capabilities (for fast local check).
    local_caps: CapabilitySet,
}

impl Scheduler {
    /// Create a new scheduler.
    pub fn new(
        capability_index: Arc<CapabilityIndex>,
        local_node_id: u64,
        local_caps: CapabilitySet,
    ) -> Self {
        Self {
            capability_index,
            local_node_id,
            local_caps,
        }
    }

    /// Check if a daemon can run locally.
    #[inline]
    pub fn can_run_locally(&self, filter: &CapabilityFilter) -> bool {
        filter.matches(&self.local_caps)
    }

    /// Place a daemon given its capability requirements.
    ///
    /// Strategy:
    /// 1. If local node matches, prefer local (zero network hop).
    /// 2. Otherwise, query the capability index for candidates.
    /// 3. Return the first match (future: least-loaded via LoadBalancer).
    pub fn place(&self, filter: &CapabilityFilter) -> Result<PlacementDecision, SchedulerError> {
        // Fast path: try local
        if self.can_run_locally(filter) {
            return Ok(PlacementDecision {
                node_id: self.local_node_id,
                reason: PlacementReason::LocalPreferred,
            });
        }

        // Query the index for matching nodes
        let candidates = self.capability_index.query(filter);

        if candidates.is_empty() {
            return Err(SchedulerError::NoCandidate);
        }

        if candidates.len() == 1 {
            return Ok(PlacementDecision {
                node_id: candidates[0],
                reason: PlacementReason::OnlyCandidate,
            });
        }

        // Multiple candidates — pick first (future: load-aware)
        Ok(PlacementDecision {
            node_id: candidates[0],
            reason: PlacementReason::FirstMatch,
        })
    }

    /// Place a daemon on a specific node (pinning).
    pub fn pin(&self, node_id: u64) -> PlacementDecision {
        PlacementDecision {
            node_id,
            reason: PlacementReason::Pinned,
        }
    }

    /// Find nodes that support a given subprotocol.
    ///
    /// Queries the capability index for nodes advertising the tag
    /// `subprotocol:0x{id:04x}`. Returns node IDs of all matches.
    pub fn find_subprotocol_nodes(&self, subprotocol_id: u16) -> Vec<u64> {
        let filter = SubprotocolRegistry::capability_filter_for(subprotocol_id);
        self.capability_index.query(&filter)
    }

    /// Find nodes capable of receiving a daemon migration.
    ///
    /// Queries for nodes advertising the migration subprotocol tag
    /// (`subprotocol:0x0500`), combined with the daemon's own capability
    /// requirements. Excludes the source node.
    pub fn find_migration_targets(
        &self,
        daemon_filter: &CapabilityFilter,
        source_node: u64,
    ) -> Vec<u64> {
        // Build a combined filter: must support migration AND daemon requirements
        let mut combined = daemon_filter.clone();
        combined =
            combined.require_tag(format!("subprotocol:{:#06x}", super::SUBPROTOCOL_MIGRATION,));

        self.capability_index
            .query(&combined)
            .into_iter()
            .filter(|&node_id| node_id != source_node)
            .collect()
    }

    /// Place a daemon for migration — find the best target node.
    ///
    /// Combines the daemon's capability requirements with the migration
    /// subprotocol tag to find eligible target nodes. Excludes the source.
    /// Prefers local node if eligible, otherwise first match.
    pub fn place_migration(
        &self,
        daemon_filter: &CapabilityFilter,
        source_node: u64,
    ) -> Result<PlacementDecision, SchedulerError> {
        let candidates = self.find_migration_targets(daemon_filter, source_node);

        if candidates.is_empty() {
            return Err(SchedulerError::NoCandidate);
        }

        // Prefer local if it's a candidate (and not the source)
        if self.local_node_id != source_node && candidates.contains(&self.local_node_id) {
            return Ok(PlacementDecision {
                node_id: self.local_node_id,
                reason: PlacementReason::LocalPreferred,
            });
        }

        if candidates.len() == 1 {
            return Ok(PlacementDecision {
                node_id: candidates[0],
                reason: PlacementReason::OnlyCandidate,
            });
        }

        Ok(PlacementDecision {
            node_id: candidates[0],
            reason: PlacementReason::FirstMatch,
        })
    }
}

impl std::fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scheduler")
            .field("local_node_id", &format!("{:#x}", self.local_node_id))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::behavior::capability::{
        CapabilityAnnouncement, GpuInfo, GpuVendor, HardwareCapabilities,
    };

    fn make_index_with_nodes(nodes: Vec<(u64, CapabilitySet)>) -> Arc<CapabilityIndex> {
        let index = CapabilityIndex::new();
        for (node_id, caps) in nodes {
            let ad = CapabilityAnnouncement::new(node_id, 1, caps);
            index.index(ad);
        }
        Arc::new(index)
    }

    fn caps_with_gpu() -> CapabilitySet {
        let gpu = GpuInfo {
            vendor: GpuVendor::Nvidia,
            model: "test".into(),
            vram_mb: 8192,
            compute_units: 0,
            tensor_cores: 0,
            fp16_tflops_x10: 0,
        };
        CapabilitySet::new().with_hardware(HardwareCapabilities::new().with_gpu(gpu))
    }

    fn caps_no_gpu() -> CapabilitySet {
        CapabilitySet::new()
    }

    #[test]
    fn test_local_preferred() {
        let local_caps = caps_no_gpu();
        let index = make_index_with_nodes(vec![]);
        let scheduler = Scheduler::new(index, 0x1111, local_caps);

        // Empty filter = runs anywhere, including local
        let decision = scheduler.place(&CapabilityFilter::default()).unwrap();
        assert_eq!(decision.node_id, 0x1111);
        assert_eq!(decision.reason, PlacementReason::LocalPreferred);
    }

    #[test]
    fn test_remote_when_local_insufficient() {
        let local_caps = caps_no_gpu(); // no GPU
        let remote_caps = caps_with_gpu();
        let index = make_index_with_nodes(vec![(0x2222, remote_caps)]);
        let scheduler = Scheduler::new(index, 0x1111, local_caps);

        let filter = CapabilityFilter::new().require_gpu();
        let decision = scheduler.place(&filter).unwrap();
        assert_eq!(decision.node_id, 0x2222);
        assert_eq!(decision.reason, PlacementReason::OnlyCandidate);
    }

    #[test]
    fn test_no_candidate() {
        let local_caps = caps_no_gpu();
        let index = make_index_with_nodes(vec![]);
        let scheduler = Scheduler::new(index, 0x1111, local_caps);

        let filter = CapabilityFilter::new().require_gpu();
        assert_eq!(
            scheduler.place(&filter).unwrap_err(),
            SchedulerError::NoCandidate
        );
    }

    #[test]
    fn test_pin() {
        let index = make_index_with_nodes(vec![]);
        let scheduler = Scheduler::new(index, 0x1111, caps_no_gpu());

        let decision = scheduler.pin(0x9999);
        assert_eq!(decision.node_id, 0x9999);
        assert_eq!(decision.reason, PlacementReason::Pinned);
    }

    #[test]
    fn test_can_run_locally() {
        let local_caps = caps_with_gpu();
        let index = make_index_with_nodes(vec![]);
        let scheduler = Scheduler::new(index, 0x1111, local_caps);

        assert!(scheduler.can_run_locally(&CapabilityFilter::new().require_gpu()));
        assert!(scheduler.can_run_locally(&CapabilityFilter::default()));
    }

    fn caps_with_migration_tag() -> CapabilitySet {
        CapabilitySet::new().add_tag("subprotocol:0x0500")
    }

    #[test]
    fn test_find_migration_targets() {
        let index = make_index_with_nodes(vec![
            (0x2222, caps_with_migration_tag()),
            (0x3333, caps_with_migration_tag()),
            (0x4444, caps_no_gpu()), // no migration tag
        ]);
        let scheduler = Scheduler::new(index, 0x1111, caps_no_gpu());

        let targets = scheduler.find_migration_targets(&CapabilityFilter::default(), 0x1111);
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&0x2222));
        assert!(targets.contains(&0x3333));
    }

    #[test]
    fn test_find_migration_targets_excludes_source() {
        let index = make_index_with_nodes(vec![
            (0x1111, caps_with_migration_tag()), // source
            (0x2222, caps_with_migration_tag()),
        ]);
        let scheduler = Scheduler::new(index, 0x1111, caps_no_gpu());

        let targets = scheduler.find_migration_targets(&CapabilityFilter::default(), 0x1111);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], 0x2222);
    }

    #[test]
    fn test_place_migration() {
        let index = make_index_with_nodes(vec![(0x2222, caps_with_migration_tag())]);
        let scheduler = Scheduler::new(index, 0x1111, caps_no_gpu());

        let decision = scheduler
            .place_migration(&CapabilityFilter::default(), 0x1111)
            .unwrap();
        assert_eq!(decision.node_id, 0x2222);
        assert_eq!(decision.reason, PlacementReason::OnlyCandidate);
    }

    #[test]
    fn test_place_migration_no_targets() {
        let index = make_index_with_nodes(vec![
            (0x2222, caps_no_gpu()), // no migration tag
        ]);
        let scheduler = Scheduler::new(index, 0x1111, caps_no_gpu());

        let err = scheduler
            .place_migration(&CapabilityFilter::default(), 0x1111)
            .unwrap_err();
        assert_eq!(err, SchedulerError::NoCandidate);
    }

    #[test]
    fn test_place_migration_prefers_local() {
        let local_caps = caps_with_migration_tag();
        let index = make_index_with_nodes(vec![
            (0x1111, caps_with_migration_tag()),
            (0x2222, caps_with_migration_tag()),
        ]);
        let scheduler = Scheduler::new(index, 0x1111, local_caps);

        // Source is 0x3333, so local (0x1111) is a valid target
        let decision = scheduler
            .place_migration(&CapabilityFilter::default(), 0x3333)
            .unwrap();
        assert_eq!(decision.node_id, 0x1111);
        assert_eq!(decision.reason, PlacementReason::LocalPreferred);
    }

    #[test]
    fn test_find_subprotocol_nodes() {
        let index = make_index_with_nodes(vec![
            (0x2222, CapabilitySet::new().add_tag("subprotocol:0x0400")),
            (0x3333, CapabilitySet::new().add_tag("subprotocol:0x0500")),
        ]);
        let scheduler = Scheduler::new(index, 0x1111, caps_no_gpu());

        let causal_nodes = scheduler.find_subprotocol_nodes(0x0400);
        assert_eq!(causal_nodes.len(), 1);
        assert_eq!(causal_nodes[0], 0x2222);

        let migration_nodes = scheduler.find_subprotocol_nodes(0x0500);
        assert_eq!(migration_nodes.len(), 1);
        assert_eq!(migration_nodes[0], 0x3333);
    }
}
