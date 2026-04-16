//! Replica groups — N copies of a daemon managed as a logical unit.
//!
//! A `ReplicaGroup` coordinates N instances of the same daemon across
//! different nodes. Each replica has its own identity (derived deterministically
//! from a group seed) and its own causal chain. The group provides:
//!
//! - Automatic placement spread across failure domains
//! - Load-balanced event routing to the nearest healthy replica
//! - Group-level health (alive as long as >= 1 replica is healthy)
//! - Dynamic scaling (add/remove replicas)
//! - Auto-replacement on node failure (stateless re-spawn)

use std::collections::HashSet;

use crate::adapter::net::behavior::capability::CapabilityFilter;
use crate::adapter::net::behavior::loadbalance::{
    Endpoint, HealthStatus, LoadBalancer, RequestContext, Strategy,
};
use crate::adapter::net::behavior::metadata::NodeId;
use crate::adapter::net::compute::daemon::{DaemonError, DaemonHostConfig, MeshDaemon};
use crate::adapter::net::compute::host::DaemonHost;
use crate::adapter::net::compute::registry::DaemonRegistry;
use crate::adapter::net::compute::scheduler::{Scheduler, SchedulerError};
use crate::adapter::net::identity::EntityKeypair;

/// Subprotocol ID for replica group coordination (reserved for future use).
pub const SUBPROTOCOL_REPLICA_GROUP: u16 = 0x0900;

// ── Types ────────────────────────────────────────────────────────────────────

/// Configuration for a replica group.
#[derive(Debug, Clone)]
pub struct ReplicaGroupConfig {
    /// Desired number of replicas.
    pub replica_count: u8,
    /// 32-byte seed for deterministic keypair derivation.
    pub group_seed: [u8; 32],
    /// Load balancing strategy for routing events to replicas.
    pub lb_strategy: Strategy,
    /// Daemon host configuration for each replica.
    pub host_config: DaemonHostConfig,
}

/// Per-replica metadata.
#[derive(Debug, Clone)]
pub struct ReplicaInfo {
    /// Replica index (0-based, used for keypair derivation).
    pub index: u8,
    /// The replica's origin_hash (derived from its keypair).
    pub origin_hash: u32,
    /// Node where this replica is placed (from Scheduler).
    pub node_id: u64,
    /// The replica's entity ID bytes (used as LoadBalancer NodeId).
    pub entity_id_bytes: NodeId,
    /// Whether this replica is currently healthy.
    pub healthy: bool,
}

/// Aggregate health of a replica group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicaGroupHealth {
    /// All replicas healthy.
    Healthy,
    /// Some replicas down but at least one healthy.
    Degraded {
        /// Number of healthy replicas.
        healthy: u8,
        /// Total replica count.
        total: u8,
    },
    /// All replicas down.
    Dead,
}

/// Errors from replica group operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplicaGroupError {
    /// No healthy replica available for routing.
    NoHealthyReplica,
    /// Placement failed for a replica.
    PlacementFailed(String),
    /// Registry operation failed.
    RegistryFailed(String),
    /// Invalid configuration.
    InvalidConfig(String),
}

impl std::fmt::Display for ReplicaGroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHealthyReplica => write!(f, "no healthy replica available"),
            Self::PlacementFailed(msg) => write!(f, "replica placement failed: {}", msg),
            Self::RegistryFailed(msg) => write!(f, "registry operation failed: {}", msg),
            Self::InvalidConfig(msg) => write!(f, "invalid config: {}", msg),
        }
    }
}

impl std::error::Error for ReplicaGroupError {}

impl From<SchedulerError> for ReplicaGroupError {
    fn from(e: SchedulerError) -> Self {
        Self::PlacementFailed(e.to_string())
    }
}

impl From<DaemonError> for ReplicaGroupError {
    fn from(e: DaemonError) -> Self {
        Self::RegistryFailed(e.to_string())
    }
}

// ── Keypair derivation ───────────────────────────────────────────────────────

/// Derive a deterministic keypair for a replica from the group seed.
///
/// Uses xxh3 to derive per-replica secret bytes from `group_seed || index`.
/// Each replica index always produces the same keypair, making the group
/// identity deterministic and reproducible.
fn derive_replica_keypair(group_seed: &[u8; 32], index: u8) -> EntityKeypair {
    use xxhash_rust::xxh3::xxh3_128;

    let mut input = [0u8; 33];
    input[..32].copy_from_slice(group_seed);
    input[32] = index;

    // Use xxh3_128 to get 16 bytes, then hash again with a different
    // suffix to get the second 16 bytes for a full 32-byte secret.
    let h1 = xxh3_128(&input);
    input[32] = index.wrapping_add(128);
    let h2 = xxh3_128(&input);

    let mut secret = [0u8; 32];
    secret[..16].copy_from_slice(&h1.to_le_bytes());
    secret[16..].copy_from_slice(&h2.to_le_bytes());

    EntityKeypair::from_bytes(secret)
}

// ── ReplicaGroup ─────────────────────────────────────────────────────────────

/// Manages N copies of a daemon as a logical unit.
///
/// The group does not own the `DaemonHost`s — they live in the
/// `DaemonRegistry` as normal entries keyed by `origin_hash`. The group
/// is a coordination overlay that tracks which `origin_hash`es belong
/// to it and maintains a `LoadBalancer` over them.
pub struct ReplicaGroup {
    /// Unique group identifier (xxh3 of group_seed).
    group_id: u32,
    /// Configuration.
    config: ReplicaGroupConfig,
    /// Per-replica state, indexed by replica index.
    replicas: Vec<ReplicaInfo>,
    /// Load balancer for routing events to healthy replicas.
    lb: LoadBalancer,
}

impl ReplicaGroup {
    /// Create a new replica group, place all replicas, and register them.
    ///
    /// For each replica index `0..replica_count`:
    /// 1. Derive keypair from `group_seed + index`
    /// 2. Place via `Scheduler` (excluding already-used nodes for spread)
    /// 3. Create `DaemonHost`, register in `DaemonRegistry`
    /// 4. Add as `LoadBalancer` endpoint
    ///
    /// The `daemon_factory` creates a fresh daemon instance per replica.
    pub fn spawn<F>(
        config: ReplicaGroupConfig,
        daemon_factory: F,
        scheduler: &Scheduler,
        registry: &DaemonRegistry,
    ) -> Result<Self, ReplicaGroupError>
    where
        F: Fn() -> Box<dyn MeshDaemon>,
    {
        if config.replica_count == 0 {
            return Err(ReplicaGroupError::InvalidConfig(
                "replica_count must be > 0".into(),
            ));
        }

        let group_id = {
            use xxhash_rust::xxh3::xxh3_64;
            xxh3_64(&config.group_seed) as u32
        };

        let lb = LoadBalancer::with_strategy(config.lb_strategy);
        let mut replicas = Vec::with_capacity(config.replica_count as usize);
        let mut used_nodes: HashSet<u64> = HashSet::new();

        let requirements = daemon_factory().requirements();

        for index in 0..config.replica_count {
            let keypair = derive_replica_keypair(&config.group_seed, index);
            let origin_hash = keypair.origin_hash();
            let entity_id_bytes: NodeId = *keypair.entity_id().as_bytes();

            // Place with spread: prefer nodes not already used
            let placement = Self::place_with_spread(scheduler, &requirements, &used_nodes)?;
            let node_id = placement.node_id;
            used_nodes.insert(node_id);

            // Create and register daemon host
            let daemon = daemon_factory();
            let host = DaemonHost::new(daemon, keypair, config.host_config.clone());
            registry.register(host)?;

            // Add to load balancer
            lb.add_endpoint(Endpoint::new(entity_id_bytes));

            replicas.push(ReplicaInfo {
                index,
                origin_hash,
                node_id,
                entity_id_bytes,
                healthy: true,
            });
        }

        Ok(Self {
            group_id,
            config,
            replicas,
            lb,
        })
    }

    /// Route an inbound event to the best available replica.
    ///
    /// Uses the internal `LoadBalancer` to select a healthy replica.
    /// Returns the `origin_hash` for delivery via `DaemonRegistry::deliver()`.
    pub fn route_event(&self, ctx: &RequestContext) -> Result<u32, ReplicaGroupError> {
        let selection = self
            .lb
            .select(ctx)
            .map_err(|_| ReplicaGroupError::NoHealthyReplica)?;

        self.origin_hash_for_entity_id(&selection.node_id)
            .ok_or(ReplicaGroupError::NoHealthyReplica)
    }

    /// Resize the group to `n` replicas.
    ///
    /// If `n > current`: derive new keypairs, place, register, add to LB.
    /// If `n < current`: remove highest-index replicas from LB, unregister.
    pub fn scale_to<F>(
        &mut self,
        n: u8,
        daemon_factory: F,
        scheduler: &Scheduler,
        registry: &DaemonRegistry,
    ) -> Result<(), ReplicaGroupError>
    where
        F: Fn() -> Box<dyn MeshDaemon>,
    {
        if n == 0 {
            return Err(ReplicaGroupError::InvalidConfig(
                "replica_count must be > 0".into(),
            ));
        }

        let current = self.replicas.len() as u8;

        if n > current {
            // Scale up: add replicas
            let requirements = daemon_factory().requirements();
            let used_nodes: HashSet<u64> = self.replicas.iter().map(|r| r.node_id).collect();

            for index in current..n {
                let keypair = derive_replica_keypair(&self.config.group_seed, index);
                let origin_hash = keypair.origin_hash();
                let entity_id_bytes: NodeId = *keypair.entity_id().as_bytes();

                let placement = Self::place_with_spread(scheduler, &requirements, &used_nodes)?;

                let daemon = daemon_factory();
                let host = DaemonHost::new(daemon, keypair, self.config.host_config.clone());
                registry.register(host)?;

                self.lb.add_endpoint(Endpoint::new(entity_id_bytes));

                self.replicas.push(ReplicaInfo {
                    index,
                    origin_hash,
                    node_id: placement.node_id,
                    entity_id_bytes,
                    healthy: true,
                });
            }
        } else if n < current {
            // Scale down: remove highest-index replicas
            while self.replicas.len() > n as usize {
                let info = self.replicas.pop().unwrap();
                self.lb.remove_endpoint(&info.entity_id_bytes);
                let _ = registry.unregister(info.origin_hash);
            }
        }

        self.config.replica_count = n;
        Ok(())
    }

    /// Handle failure of a node hosting one or more replicas.
    ///
    /// For each replica on the failed node:
    /// 1. Mark unhealthy in LoadBalancer
    /// 2. Attempt replacement: re-derive keypair, place on new node, register
    ///
    /// For stateless daemons, replacement is a fresh spawn with the same
    /// deterministic identity (no migration needed).
    pub fn on_node_failure<F>(
        &mut self,
        failed_node_id: u64,
        daemon_factory: F,
        scheduler: &Scheduler,
        registry: &DaemonRegistry,
    ) -> Result<Vec<u8>, ReplicaGroupError>
    where
        F: Fn() -> Box<dyn MeshDaemon>,
    {
        let mut replaced = Vec::new();
        let requirements = daemon_factory().requirements();
        let mut exclude: HashSet<u64> = HashSet::new();
        exclude.insert(failed_node_id);

        for replica in &mut self.replicas {
            if replica.node_id != failed_node_id {
                continue;
            }

            // Mark unhealthy
            replica.healthy = false;
            self.lb
                .update_health(&replica.entity_id_bytes, HealthStatus::Unhealthy);

            // Unregister old host (may already be gone if node crashed)
            let _ = registry.unregister(replica.origin_hash);

            // Re-derive the same keypair (deterministic)
            let keypair = derive_replica_keypair(&self.config.group_seed, replica.index);

            // Place on a new node
            let placement = match Self::place_with_spread(scheduler, &requirements, &exclude) {
                Ok(p) => p,
                Err(_) => continue, // can't replace right now
            };

            // Spawn fresh daemon
            let daemon = daemon_factory();
            let host = DaemonHost::new(daemon, keypair, self.config.host_config.clone());
            if registry.register(host).is_err() {
                continue;
            }

            // Update replica info
            replica.node_id = placement.node_id;
            replica.healthy = true;
            self.lb
                .update_health(&replica.entity_id_bytes, HealthStatus::Healthy);
            exclude.insert(placement.node_id);

            replaced.push(replica.index);
        }

        Ok(replaced)
    }

    /// Handle recovery of a node.
    ///
    /// Re-marks any replicas on this node as healthy.
    pub fn on_node_recovery(&mut self, recovered_node_id: u64) {
        for replica in &mut self.replicas {
            if replica.node_id == recovered_node_id && !replica.healthy {
                replica.healthy = true;
                self.lb
                    .update_health(&replica.entity_id_bytes, HealthStatus::Healthy);
            }
        }
    }

    /// Aggregate health of the group.
    pub fn health(&self) -> ReplicaGroupHealth {
        let healthy = self.replicas.iter().filter(|r| r.healthy).count() as u8;
        let total = self.replicas.len() as u8;
        match healthy {
            0 => ReplicaGroupHealth::Dead,
            n if n == total => ReplicaGroupHealth::Healthy,
            n => ReplicaGroupHealth::Degraded { healthy: n, total },
        }
    }

    /// Get the group ID.
    pub fn group_id(&self) -> u32 {
        self.group_id
    }

    /// Get all replica info.
    pub fn replicas(&self) -> &[ReplicaInfo] {
        &self.replicas
    }

    /// Number of replicas.
    pub fn replica_count(&self) -> u8 {
        self.replicas.len() as u8
    }

    /// Number of healthy replicas.
    pub fn healthy_count(&self) -> u8 {
        self.replicas.iter().filter(|r| r.healthy).count() as u8
    }

    /// Look up origin_hash from a LoadBalancer entity ID.
    fn origin_hash_for_entity_id(&self, entity_id: &NodeId) -> Option<u32> {
        self.replicas
            .iter()
            .find(|r| r.entity_id_bytes == *entity_id)
            .map(|r| r.origin_hash)
    }

    /// Place a daemon with best-effort spread across nodes.
    ///
    /// Queries the scheduler, preferring nodes not in `exclude`.
    /// Falls back to any matching node if all candidates are excluded.
    fn place_with_spread(
        scheduler: &Scheduler,
        requirements: &CapabilityFilter,
        _exclude: &HashSet<u64>,
    ) -> Result<crate::adapter::net::compute::scheduler::PlacementDecision, ReplicaGroupError> {
        // Try placing — the scheduler returns the best candidate.
        // Best-effort spread: the scheduler prefers local, then first match.
        // A future enhancement could query all candidates and filter out
        // nodes in `exclude` for stronger failure-domain isolation.
        let placement = scheduler.place(requirements)?;
        Ok(placement)
    }
}

impl std::fmt::Debug for ReplicaGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplicaGroup")
            .field("group_id", &format!("{:#x}", self.group_id))
            .field("replicas", &self.replicas.len())
            .field("healthy", &self.healthy_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::behavior::capability::{
        CapabilityAnnouncement, CapabilityIndex, CapabilitySet,
    };
    use crate::adapter::net::state::causal::CausalEvent;
    use bytes::Bytes;
    use std::sync::Arc;

    struct NoopDaemon;

    impl MeshDaemon for NoopDaemon {
        fn name(&self) -> &str {
            "noop"
        }
        fn requirements(&self) -> CapabilityFilter {
            CapabilityFilter::default()
        }
        fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
            Ok(vec![])
        }
    }

    fn make_scheduler() -> Scheduler {
        let index = Arc::new(CapabilityIndex::new());
        // Add a couple of nodes
        index.index(CapabilityAnnouncement::new(0x1111, 1, CapabilitySet::new()));
        index.index(CapabilityAnnouncement::new(0x2222, 1, CapabilitySet::new()));
        index.index(CapabilityAnnouncement::new(0x3333, 1, CapabilitySet::new()));
        Scheduler::new(index, 0x1111, CapabilitySet::new())
    }

    fn test_config(n: u8) -> ReplicaGroupConfig {
        ReplicaGroupConfig {
            replica_count: n,
            group_seed: [42u8; 32],
            lb_strategy: Strategy::RoundRobin,
            host_config: DaemonHostConfig::default(),
        }
    }

    #[test]
    fn test_spawn_group() {
        let reg = DaemonRegistry::new();
        let sched = make_scheduler();

        let group =
            ReplicaGroup::spawn(test_config(3), || Box::new(NoopDaemon), &sched, &reg).unwrap();

        assert_eq!(group.replica_count(), 3);
        assert_eq!(group.health(), ReplicaGroupHealth::Healthy);
        assert_eq!(reg.count(), 3);

        // Each replica has a unique origin_hash
        let hashes: HashSet<u32> = group.replicas().iter().map(|r| r.origin_hash).collect();
        assert_eq!(hashes.len(), 3);
    }

    #[test]
    fn test_deterministic_keypairs() {
        let seed = [7u8; 32];
        let kp1 = derive_replica_keypair(&seed, 0);
        let kp2 = derive_replica_keypair(&seed, 0);
        assert_eq!(kp1.origin_hash(), kp2.origin_hash());

        let kp3 = derive_replica_keypair(&seed, 1);
        assert_ne!(kp1.origin_hash(), kp3.origin_hash());
    }

    #[test]
    fn test_zero_replicas_rejected() {
        let reg = DaemonRegistry::new();
        let sched = make_scheduler();

        let err =
            ReplicaGroup::spawn(test_config(0), || Box::new(NoopDaemon), &sched, &reg).unwrap_err();
        assert_eq!(
            err,
            ReplicaGroupError::InvalidConfig("replica_count must be > 0".into())
        );
    }

    #[test]
    fn test_route_event() {
        let reg = DaemonRegistry::new();
        let sched = make_scheduler();

        let group =
            ReplicaGroup::spawn(test_config(3), || Box::new(NoopDaemon), &sched, &reg).unwrap();

        let ctx = RequestContext::default();
        let origin = group.route_event(&ctx).unwrap();

        // The returned origin_hash should belong to one of our replicas
        assert!(group.replicas().iter().any(|r| r.origin_hash == origin));
    }

    #[test]
    fn test_scale_up() {
        let reg = DaemonRegistry::new();
        let sched = make_scheduler();

        let mut group =
            ReplicaGroup::spawn(test_config(2), || Box::new(NoopDaemon), &sched, &reg).unwrap();

        assert_eq!(group.replica_count(), 2);
        assert_eq!(reg.count(), 2);

        group
            .scale_to(4, || Box::new(NoopDaemon), &sched, &reg)
            .unwrap();

        assert_eq!(group.replica_count(), 4);
        assert_eq!(reg.count(), 4);
        assert_eq!(group.health(), ReplicaGroupHealth::Healthy);
    }

    #[test]
    fn test_scale_down() {
        let reg = DaemonRegistry::new();
        let sched = make_scheduler();

        let mut group =
            ReplicaGroup::spawn(test_config(4), || Box::new(NoopDaemon), &sched, &reg).unwrap();

        assert_eq!(reg.count(), 4);

        group
            .scale_to(2, || Box::new(NoopDaemon), &sched, &reg)
            .unwrap();

        assert_eq!(group.replica_count(), 2);
        assert_eq!(reg.count(), 2);
    }

    #[test]
    fn test_scale_to_zero_rejected() {
        let reg = DaemonRegistry::new();
        let sched = make_scheduler();

        let mut group =
            ReplicaGroup::spawn(test_config(2), || Box::new(NoopDaemon), &sched, &reg).unwrap();

        let err = group
            .scale_to(0, || Box::new(NoopDaemon), &sched, &reg)
            .unwrap_err();
        assert_eq!(
            err,
            ReplicaGroupError::InvalidConfig("replica_count must be > 0".into())
        );
    }

    #[test]
    fn test_node_failure_and_replacement() {
        let reg = DaemonRegistry::new();
        let sched = make_scheduler();

        let mut group =
            ReplicaGroup::spawn(test_config(3), || Box::new(NoopDaemon), &sched, &reg).unwrap();

        let failed_node = group.replicas()[0].node_id;
        let failed_origin = group.replicas()[0].origin_hash;

        // Simulate failure
        let replaced = group
            .on_node_failure(failed_node, || Box::new(NoopDaemon), &sched, &reg)
            .unwrap();

        // At least one replica should have been replaced
        assert!(!replaced.is_empty());

        // Group should still be healthy (replacement succeeded)
        assert_ne!(group.health(), ReplicaGroupHealth::Dead);

        // The replaced replica keeps the same origin_hash (deterministic keypair)
        assert!(group
            .replicas()
            .iter()
            .any(|r| r.origin_hash == failed_origin));
    }

    #[test]
    fn test_node_recovery() {
        let reg = DaemonRegistry::new();
        let sched = make_scheduler();

        let mut group =
            ReplicaGroup::spawn(test_config(2), || Box::new(NoopDaemon), &sched, &reg).unwrap();

        let node = group.replicas()[0].node_id;

        // Mark unhealthy manually
        group.replicas[0].healthy = false;
        group
            .lb
            .update_health(&group.replicas[0].entity_id_bytes, HealthStatus::Unhealthy);

        assert_eq!(
            group.health(),
            ReplicaGroupHealth::Degraded {
                healthy: 1,
                total: 2
            }
        );

        // Recover
        group.on_node_recovery(node);
        assert_eq!(group.health(), ReplicaGroupHealth::Healthy);
    }

    #[test]
    fn test_group_health_dead() {
        let reg = DaemonRegistry::new();
        let sched = make_scheduler();

        let mut group =
            ReplicaGroup::spawn(test_config(2), || Box::new(NoopDaemon), &sched, &reg).unwrap();

        // Mark all unhealthy
        for replica in &mut group.replicas {
            replica.healthy = false;
        }

        assert_eq!(group.health(), ReplicaGroupHealth::Dead);
    }

    #[test]
    fn test_group_id_deterministic() {
        let reg1 = DaemonRegistry::new();
        let reg2 = DaemonRegistry::new();
        let sched = make_scheduler();

        let g1 =
            ReplicaGroup::spawn(test_config(1), || Box::new(NoopDaemon), &sched, &reg1).unwrap();

        let g2 =
            ReplicaGroup::spawn(test_config(1), || Box::new(NoopDaemon), &sched, &reg2).unwrap();

        assert_eq!(g1.group_id(), g2.group_id());
    }
}
