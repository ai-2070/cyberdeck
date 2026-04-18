//! Target-side migration handler.
//!
//! Manages the target node's role in migration: restoring from snapshot,
//! replaying buffered events in causal order, and activating the daemon
//! as the authoritative copy after cutover.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use parking_lot::Mutex;

use super::daemon::{DaemonHostConfig, MeshDaemon};
use super::daemon_factory::DaemonFactoryRegistry;
use super::host::DaemonHost;
use super::migration::{MigrationError, MigrationPhase};
use super::registry::DaemonRegistry;
use crate::adapter::net::identity::EntityKeypair;
use crate::adapter::net::state::causal::{CausalEvent, CausalLink};
use crate::adapter::net::state::snapshot::StateSnapshot;

/// Per-daemon target-side migration state.
#[allow(dead_code)]
struct TargetMigrationState {
    daemon_origin: u32,
    source_node: u64,
    /// Node that initiated the migration. Replies
    /// (RestoreComplete / ReplayComplete / ActivateAck) are routed here.
    orchestrator_node: u64,
    phase: MigrationPhase,
    /// Sequence number through which events have been replayed.
    replayed_through: u64,
    /// Events pending replay, keyed by sequence for ordered replay.
    pending_events: BTreeMap<u64, CausalEvent>,
    /// Causal chain head on target after restore.
    target_head: CausalLink,
    started_at: Instant,
}

/// Target-side state retained after a successful migration completes,
/// so that retried `ActivateTarget` packets (after a lost `ActivateAck`)
/// can be handled idempotently by replaying the original result.
#[derive(Debug, Clone, Copy)]
struct CompletedTargetState {
    orchestrator_node: u64,
    replayed_through: u64,
    #[allow(dead_code)]
    completed_at: Instant,
}

/// Handles the target node's role in daemon migration.
///
/// The target handler:
/// 1. Restores a daemon from a snapshot (phase 2)
/// 2. Replays buffered events in strict sequence order (phase 3)
/// 3. Activates as the authoritative copy after cutover (phase 4)
pub struct MigrationTargetHandler {
    /// Target node's daemon registry.
    daemon_registry: Arc<DaemonRegistry>,
    /// Active migrations on this node as target: daemon_origin → state.
    migrations: DashMap<u32, Mutex<TargetMigrationState>>,
    /// Factories for constructing daemon instances during restore.
    ///
    /// Consulted by the subprotocol handler when it has a reassembled
    /// snapshot but needs a fresh daemon instance + keypair + config to
    /// pass to [`MigrationTargetHandler::restore_snapshot`]. Empty when
    /// the handler is created via `new()`.
    factories: Arc<DaemonFactoryRegistry>,
    /// Completed migrations retained for ActivateAck idempotency. A
    /// retried `ActivateTarget` after a lost `ActivateAck` looks up the
    /// stored `(orchestrator_node, replayed_through)` and re-sends the
    /// same ack instead of failing with `DaemonNotFound`.
    completed: DashMap<u32, CompletedTargetState>,
}

impl MigrationTargetHandler {
    /// Create a new target handler with no daemon factories registered.
    ///
    /// Use this on nodes that are source-only, or in unit tests that call
    /// `restore_snapshot` directly with an inline factory closure. For a
    /// node that the subprotocol handler should auto-restore onto, use
    /// [`MigrationTargetHandler::new_with_factories`] instead.
    pub fn new(daemon_registry: Arc<DaemonRegistry>) -> Self {
        Self::new_with_factories(daemon_registry, DaemonFactoryRegistry::empty())
    }

    /// Create a target handler backed by a shared factory registry.
    ///
    /// The subprotocol handler resolves restore inputs through this
    /// registry; if a migration arrives for an origin that hasn't been
    /// registered, the handler fails the migration instead of silently
    /// ignoring it.
    pub fn new_with_factories(
        daemon_registry: Arc<DaemonRegistry>,
        factories: Arc<DaemonFactoryRegistry>,
    ) -> Self {
        Self {
            daemon_registry,
            migrations: DashMap::new(),
            factories,
            completed: DashMap::new(),
        }
    }

    /// Access the factory registry (for the subprotocol handler).
    pub fn factories(&self) -> &Arc<DaemonFactoryRegistry> {
        &self.factories
    }

    /// Phase 2: Restore a daemon from a snapshot.
    ///
    /// Creates a new `DaemonHost` from the snapshot and registers it in the
    /// local daemon registry. The daemon is not yet authoritative — events
    /// will be replayed before cutover.
    ///
    /// The `daemon_factory` closure creates the daemon implementation that
    /// will be restored from the snapshot. The caller must provide the correct
    /// daemon type matching the origin hash. `orchestrator_node` is the
    /// node that initiated this migration; reply messages route here, not
    /// to the immediate wire hop.
    pub fn restore_snapshot<F>(
        &self,
        daemon_origin: u32,
        snapshot: &StateSnapshot,
        source_node: u64,
        orchestrator_node: u64,
        keypair: EntityKeypair,
        daemon_factory: F,
        config: DaemonHostConfig,
    ) -> Result<(), MigrationError>
    where
        F: FnOnce() -> Box<dyn MeshDaemon>,
    {
        if self.migrations.contains_key(&daemon_origin) {
            return Err(MigrationError::AlreadyMigrating(daemon_origin));
        }

        // Validate snapshot matches the daemon
        if snapshot.entity_id.origin_hash() != daemon_origin {
            return Err(MigrationError::StateFailed(format!(
                "snapshot origin {:#x} does not match daemon {:#x}",
                snapshot.entity_id.origin_hash(),
                daemon_origin,
            )));
        }

        // Create daemon from snapshot
        let daemon = daemon_factory();
        let host = DaemonHost::from_snapshot(daemon, keypair, snapshot, config)
            .map_err(|e| MigrationError::StateFailed(e.to_string()))?;

        let target_head = snapshot.chain_link;
        let replayed_through = snapshot.through_seq;

        // Register in local daemon registry
        self.daemon_registry
            .register(host)
            .map_err(|e| MigrationError::StateFailed(e.to_string()))?;

        // Track migration state
        self.migrations.insert(
            daemon_origin,
            Mutex::new(TargetMigrationState {
                daemon_origin,
                source_node,
                orchestrator_node,
                phase: MigrationPhase::Restore,
                replayed_through,
                pending_events: BTreeMap::new(),
                target_head,
                started_at: Instant::now(),
            }),
        );

        Ok(())
    }

    /// Recorded orchestrator for an active or recently-completed
    /// target-side migration.
    pub fn orchestrator_node(&self, daemon_origin: u32) -> Option<u64> {
        if let Some(e) = self.migrations.get(&daemon_origin) {
            return Some(e.lock().orchestrator_node);
        }
        self.completed
            .get(&daemon_origin)
            .map(|e| e.orchestrator_node)
    }

    /// Phase 3: Replay buffered events from the source.
    ///
    /// Events are inserted into a BTreeMap keyed by sequence and replayed
    /// in strict order. Returns the sequence number replayed through.
    pub fn replay_events(
        &self,
        daemon_origin: u32,
        events: Vec<CausalEvent>,
    ) -> Result<u64, MigrationError> {
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        let mut state = entry.lock();
        state.phase = MigrationPhase::Replay;

        // Insert into BTreeMap for ordered replay
        for event in events {
            state.pending_events.insert(event.link.sequence, event);
        }

        // Replay in order
        self.drain_pending(&mut state)?;

        Ok(state.replayed_through)
    }

    /// Buffer an event arriving during migration (before cutover).
    ///
    /// Events that arrive out-of-order are buffered in the BTreeMap and
    /// will be replayed in sequence order.
    pub fn buffer_event(
        &self,
        daemon_origin: u32,
        event: CausalEvent,
    ) -> Result<bool, MigrationError> {
        let entry = match self.migrations.get(&daemon_origin) {
            Some(entry) => entry,
            None => return Ok(false),
        };

        let mut state = entry.lock();
        state.pending_events.insert(event.link.sequence, event);

        // Try to drain any contiguous events
        self.drain_pending(&mut state)?;

        Ok(true)
    }

    /// Phase 4: Activate — daemon goes live on this node.
    ///
    /// Drains any remaining pending events and marks the daemon as
    /// the authoritative copy. **Idempotent:** if the migration has
    /// already been completed (e.g. a retried `ActivateTarget` after a
    /// lost `ActivateAck`), returns the stored `replayed_through` so the
    /// subprotocol handler can re-emit the same ack instead of failing.
    pub fn activate(&self, daemon_origin: u32) -> Result<u64, MigrationError> {
        if let Some(done) = self.completed.get(&daemon_origin) {
            return Ok(done.replayed_through);
        }
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        let mut state = entry.lock();
        state.phase = MigrationPhase::Cutover;

        // Drain remaining events
        self.drain_pending(&mut state)?;

        Ok(state.replayed_through)
    }

    /// Mark migration as complete and move tracking state into the
    /// `completed` index so that a retried `ActivateTarget` after a lost
    /// `ActivateAck` can be handled idempotently.
    ///
    /// The daemon remains registered in the daemon registry — it's now
    /// the authoritative copy. Also removes the factory entry, since the
    /// target won't need to re-restore from an orchestrator retry once
    /// the migration has successfully completed.
    pub fn complete(&self, daemon_origin: u32) -> Result<(), MigrationError> {
        if self.completed.contains_key(&daemon_origin) {
            return Ok(());
        }
        let (_, entry) = self
            .migrations
            .remove(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;
        let state = entry.into_inner();
        self.completed.insert(
            daemon_origin,
            CompletedTargetState {
                orchestrator_node: state.orchestrator_node,
                replayed_through: state.replayed_through,
                completed_at: Instant::now(),
            },
        );
        // The factory is single-shot on a successful migration: keeping it
        // registered would let a stale or replayed SnapshotReady re-trigger
        // restore against what is now the authoritative copy.
        self.factories.remove(daemon_origin);
        Ok(())
    }

    /// Forget a completed migration's retry-idempotency record. Safe to
    /// call at any time; a subsequent retried `ActivateTarget` would
    /// then fail normally with `DaemonNotFound`.
    pub fn forget_completed(&self, daemon_origin: u32) -> bool {
        self.completed.remove(&daemon_origin).is_some()
    }

    /// Abort migration — unregister daemon and clean up.
    pub fn abort(&self, daemon_origin: u32) -> Result<(), MigrationError> {
        if self.migrations.remove(&daemon_origin).is_some() {
            // Unregister daemon (it's not authoritative, source still has it)
            let _ = self.daemon_registry.unregister(daemon_origin);
        }
        Ok(())
    }

    /// Check if a daemon is being migrated to this node.
    pub fn is_migrating(&self, daemon_origin: u32) -> bool {
        self.migrations.contains_key(&daemon_origin)
    }

    /// Get the current phase of a target-side migration.
    pub fn phase(&self, daemon_origin: u32) -> Option<MigrationPhase> {
        self.migrations
            .get(&daemon_origin)
            .map(|entry| entry.lock().phase)
    }

    /// Get the sequence number replayed through.
    pub fn replayed_through(&self, daemon_origin: u32) -> Option<u64> {
        self.migrations
            .get(&daemon_origin)
            .map(|entry| entry.lock().replayed_through)
    }

    /// Number of active target-side migrations.
    pub fn active_count(&self) -> usize {
        self.migrations.len()
    }

    /// Drain pending events in sequence order, delivering to the daemon.
    fn drain_pending(&self, state: &mut TargetMigrationState) -> Result<(), MigrationError> {
        // Collect events to replay (contiguous from replayed_through + 1)
        let mut to_replay = Vec::new();
        let mut next_seq = state.replayed_through + 1;

        while let Some(event) = state.pending_events.remove(&next_seq) {
            to_replay.push(event);
            next_seq += 1;
        }

        // Also drain any events with sequence <= replayed_through (duplicates)
        let stale: Vec<u64> = state
            .pending_events
            .keys()
            .take_while(|&&seq| seq <= state.replayed_through)
            .cloned()
            .collect();
        for seq in stale {
            state.pending_events.remove(&seq);
        }

        // Deliver events to daemon via registry
        for event in &to_replay {
            self.daemon_registry
                .deliver(state.daemon_origin, event)
                .map_err(|e| MigrationError::StateFailed(e.to_string()))?;
        }

        if let Some(last) = to_replay.last() {
            state.replayed_through = last.link.sequence;
            state.target_head = last.link;
        }

        Ok(())
    }
}

impl std::fmt::Debug for MigrationTargetHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationTargetHandler")
            .field("active_migrations", &self.migrations.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::behavior::capability::CapabilityFilter;
    use crate::adapter::net::compute::{DaemonError, MeshDaemon};
    use crate::adapter::net::identity::EntityKeypair;
    use crate::adapter::net::state::causal::CausalChainBuilder;
    use crate::adapter::net::state::horizon::ObservedHorizon;
    use bytes::Bytes;

    struct AccumDaemon {
        total: u64,
    }

    impl MeshDaemon for AccumDaemon {
        fn name(&self) -> &str {
            "accum"
        }
        fn requirements(&self) -> CapabilityFilter {
            CapabilityFilter::default()
        }
        fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
            self.total += 1;
            Ok(vec![])
        }
        fn snapshot(&self) -> Option<Bytes> {
            Some(Bytes::from(self.total.to_le_bytes().to_vec()))
        }
        fn restore(&mut self, state: Bytes) -> Result<(), DaemonError> {
            if state.len() != 8 {
                return Err(DaemonError::RestoreFailed("bad size".into()));
            }
            self.total = u64::from_le_bytes(state[..8].try_into().unwrap());
            Ok(())
        }
    }

    fn make_event(origin: u32, seq: u64) -> CausalEvent {
        CausalEvent {
            link: CausalLink {
                origin_hash: origin,
                horizon_encoded: 0,
                sequence: seq,
                parent_hash: 0,
            },
            payload: Bytes::from_static(b"data"),
            received_at: 0,
        }
    }

    fn make_snapshot(kp: &EntityKeypair, through_seq: u64, value: u64) -> StateSnapshot {
        let mut chain = CausalChainBuilder::new(kp.origin_hash());
        // Advance the chain to the desired sequence so through_seq is correct
        for _ in 0..through_seq {
            chain.append(Bytes::from_static(b"x"), 0).unwrap();
        }
        StateSnapshot::new(
            kp.entity_id().clone(),
            *chain.head(),
            Bytes::from(value.to_le_bytes().to_vec()),
            ObservedHorizon::new(),
        )
    }

    #[test]
    fn test_restore_and_replay() {
        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationTargetHandler::new(reg.clone());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();

        let snapshot = make_snapshot(&kp, 10, 42);

        handler
            .restore_snapshot(
                origin,
                &snapshot,
                0x1111,
                0x2222,
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();

        assert!(handler.is_migrating(origin));
        assert!(reg.contains(origin));

        // Replay events starting after snapshot's through_seq (10)
        let events = vec![
            make_event(0xBBBB, 11),
            make_event(0xBBBB, 12),
            make_event(0xBBBB, 13),
        ];
        let replayed = handler.replay_events(origin, events).unwrap();
        assert_eq!(replayed, 13);
    }

    #[test]
    fn test_restore_wrong_origin_rejected() {
        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationTargetHandler::new(reg);
        let kp = EntityKeypair::generate();

        let snapshot = make_snapshot(&kp, 10, 42);

        // Use a different origin hash
        let err = handler
            .restore_snapshot(
                0xDEAD,
                &snapshot,
                0x1111,
                0x2222,
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap_err();
        assert!(err.to_string().contains("does not match"));
    }

    #[test]
    fn test_out_of_order_buffering() {
        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationTargetHandler::new(reg.clone());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();

        let snapshot = make_snapshot(&kp, 0, 0);

        handler
            .restore_snapshot(
                origin,
                &snapshot,
                0x1111,
                0x2222,
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();

        // Buffer events out of order
        handler.buffer_event(origin, make_event(0xBBBB, 3)).unwrap();
        handler.buffer_event(origin, make_event(0xBBBB, 1)).unwrap();
        handler.buffer_event(origin, make_event(0xBBBB, 2)).unwrap();

        // After buffering 1, 2, 3 should all be replayed in order
        assert_eq!(handler.replayed_through(origin), Some(3));
    }

    #[test]
    fn test_activate_and_complete() {
        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationTargetHandler::new(reg.clone());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();

        let snapshot = make_snapshot(&kp, 0, 0);

        handler
            .restore_snapshot(
                origin,
                &snapshot,
                0x1111,
                0x2222,
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();

        handler.activate(origin).unwrap();
        assert_eq!(handler.phase(origin), Some(MigrationPhase::Cutover));

        handler.complete(origin).unwrap();
        assert!(!handler.is_migrating(origin));
        assert!(reg.contains(origin)); // daemon still registered (authoritative)
    }

    #[test]
    fn test_abort() {
        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationTargetHandler::new(reg.clone());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();

        let snapshot = make_snapshot(&kp, 0, 0);

        handler
            .restore_snapshot(
                origin,
                &snapshot,
                0x1111,
                0x2222,
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();

        handler.abort(origin).unwrap();
        assert!(!handler.is_migrating(origin));
        assert!(!reg.contains(origin)); // daemon unregistered on abort
    }
}
