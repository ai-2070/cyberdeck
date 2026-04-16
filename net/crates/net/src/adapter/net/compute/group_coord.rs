//! Group coordinator — shared coordination logic for daemon groups.
//!
//! Extracted from `ReplicaGroup` so that both `ReplicaGroup` and `ForkGroup`
//! can reuse the same load balancing, health tracking, member management,
//! and scaling mechanics without duplication.

use std::collections::HashSet;

use crate::adapter::net::behavior::capability::CapabilityFilter;
use crate::adapter::net::behavior::loadbalance::{
    Endpoint, HealthStatus, LoadBalancer, RequestContext, Strategy,
};
use crate::adapter::net::behavior::metadata::NodeId;
use crate::adapter::net::compute::daemon::DaemonError;
use crate::adapter::net::compute::scheduler::{Scheduler, SchedulerError};

// ── Member info ──────────────────────────────────────────────────────────────

/// Per-member metadata within a group.
#[derive(Debug, Clone)]
pub struct MemberInfo {
    /// Member index (0-based).
    pub index: u8,
    /// The member's origin_hash (from its keypair).
    pub origin_hash: u32,
    /// Node where this member is placed.
    pub node_id: u64,
    /// The member's entity ID bytes (used as LoadBalancer NodeId).
    pub entity_id_bytes: NodeId,
    /// Whether this member is currently healthy.
    pub healthy: bool,
}

// ── Group health ─────────────────────────────────────────────────────────────

/// Aggregate health of a daemon group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupHealth {
    /// All members healthy.
    Healthy,
    /// Some members down but at least one healthy.
    Degraded {
        /// Number of healthy members.
        healthy: u8,
        /// Total member count.
        total: u8,
    },
    /// All members down.
    Dead,
}

// ── Errors ───────────────────────────────────────────────────────────────────

/// Errors from group operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupError {
    /// No healthy member available for routing.
    NoHealthyMember,
    /// Placement failed.
    PlacementFailed(String),
    /// Registry operation failed.
    RegistryFailed(String),
    /// Invalid configuration.
    InvalidConfig(String),
}

impl std::fmt::Display for GroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHealthyMember => write!(f, "no healthy member available"),
            Self::PlacementFailed(msg) => write!(f, "placement failed: {}", msg),
            Self::RegistryFailed(msg) => write!(f, "registry operation failed: {}", msg),
            Self::InvalidConfig(msg) => write!(f, "invalid config: {}", msg),
        }
    }
}

impl std::error::Error for GroupError {}

impl From<SchedulerError> for GroupError {
    fn from(e: SchedulerError) -> Self {
        Self::PlacementFailed(e.to_string())
    }
}

impl From<DaemonError> for GroupError {
    fn from(e: DaemonError) -> Self {
        Self::RegistryFailed(e.to_string())
    }
}

// ── Group coordinator ────────────────────────────────────────────────────────

/// Shared coordination logic for daemon groups.
///
/// Manages the `LoadBalancer`, member tracking, health aggregation,
/// and routing. Both `ReplicaGroup` and `ForkGroup` own a coordinator
/// and delegate group-level operations to it.
pub struct GroupCoordinator {
    /// Per-member state, indexed by member index.
    pub members: Vec<MemberInfo>,
    /// Load balancer for routing events to healthy members.
    pub lb: LoadBalancer,
}

impl GroupCoordinator {
    /// Create a new empty coordinator with the given LB strategy.
    pub fn new(strategy: Strategy) -> Self {
        Self {
            members: Vec::new(),
            lb: LoadBalancer::with_strategy(strategy),
        }
    }

    /// Add a member that has already been registered in the DaemonRegistry.
    pub fn add_member(&mut self, info: MemberInfo) {
        self.lb.add_endpoint(Endpoint::new(info.entity_id_bytes));
        self.members.push(info);
    }

    /// Remove and return the highest-index member.
    ///
    /// Also removes from the LoadBalancer. Caller is responsible for
    /// unregistering from DaemonRegistry.
    pub fn remove_last(&mut self) -> Option<MemberInfo> {
        let info = self.members.pop()?;
        self.lb.remove_endpoint(&info.entity_id_bytes);
        Some(info)
    }

    /// Route an event to the best available member.
    ///
    /// Returns the `origin_hash` for delivery via `DaemonRegistry::deliver()`.
    pub fn route_event(&self, ctx: &RequestContext) -> Result<u32, GroupError> {
        let selection = self
            .lb
            .select(ctx)
            .map_err(|_| GroupError::NoHealthyMember)?;

        self.origin_hash_for_entity_id(&selection.node_id)
            .ok_or(GroupError::NoHealthyMember)
    }

    /// Mark a member unhealthy in the LoadBalancer.
    pub fn mark_unhealthy(&mut self, index: u8) {
        if let Some(member) = self.members.iter_mut().find(|m| m.index == index) {
            member.healthy = false;
            self.lb
                .update_health(&member.entity_id_bytes, HealthStatus::Unhealthy);
        }
    }

    /// Mark a member healthy in the LoadBalancer.
    pub fn mark_healthy(&mut self, index: u8) {
        if let Some(member) = self.members.iter_mut().find(|m| m.index == index) {
            member.healthy = true;
            self.lb
                .update_health(&member.entity_id_bytes, HealthStatus::Healthy);
        }
    }

    /// Update a member's placement after failure recovery.
    ///
    /// Updates the node_id, re-marks healthy, and updates the LB endpoint.
    pub fn update_member_placement(
        &mut self,
        index: u8,
        new_node_id: u64,
        new_entity_id_bytes: NodeId,
    ) {
        if let Some(member) = self.members.iter_mut().find(|m| m.index == index) {
            // Remove old endpoint, add new one
            self.lb.remove_endpoint(&member.entity_id_bytes);
            member.node_id = new_node_id;
            member.entity_id_bytes = new_entity_id_bytes;
            member.healthy = true;
            self.lb.add_endpoint(Endpoint::new(new_entity_id_bytes));
        }
    }

    /// Re-mark all members on a recovered node as healthy.
    pub fn on_node_recovery(&mut self, recovered_node_id: u64) {
        for member in &mut self.members {
            if member.node_id == recovered_node_id && !member.healthy {
                member.healthy = true;
                self.lb
                    .update_health(&member.entity_id_bytes, HealthStatus::Healthy);
            }
        }
    }

    /// Aggregate health of the group.
    pub fn health(&self) -> GroupHealth {
        let healthy = self.members.iter().filter(|m| m.healthy).count() as u8;
        let total = self.members.len() as u8;
        match healthy {
            0 => GroupHealth::Dead,
            n if n == total => GroupHealth::Healthy,
            n => GroupHealth::Degraded { healthy: n, total },
        }
    }

    /// Get all member info.
    pub fn members(&self) -> &[MemberInfo] {
        &self.members
    }

    /// Number of members.
    pub fn member_count(&self) -> u8 {
        self.members.len() as u8
    }

    /// Number of healthy members.
    pub fn healthy_count(&self) -> u8 {
        self.members.iter().filter(|m| m.healthy).count() as u8
    }

    /// Indices of members on a given node.
    pub fn members_on_node(&self, node_id: u64) -> Vec<u8> {
        self.members
            .iter()
            .filter(|m| m.node_id == node_id)
            .map(|m| m.index)
            .collect()
    }

    /// Look up origin_hash from a LoadBalancer entity ID.
    fn origin_hash_for_entity_id(&self, entity_id: &NodeId) -> Option<u32> {
        self.members
            .iter()
            .find(|m| m.entity_id_bytes == *entity_id)
            .map(|m| m.origin_hash)
    }

    /// Place a daemon with best-effort spread across nodes.
    pub fn place_with_spread(
        scheduler: &Scheduler,
        requirements: &CapabilityFilter,
        _exclude: &HashSet<u64>,
    ) -> Result<crate::adapter::net::compute::scheduler::PlacementDecision, GroupError> {
        let placement = scheduler.place(requirements)?;
        Ok(placement)
    }
}

impl std::fmt::Debug for GroupCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupCoordinator")
            .field("members", &self.members.len())
            .field("healthy", &self.healthy_count())
            .finish()
    }
}
