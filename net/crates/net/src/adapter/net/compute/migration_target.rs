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

/// Scalar inputs for [`MigrationTargetHandler::restore_snapshot`]. Bundled
/// into a struct to keep the call site under the "too many arguments"
/// clippy limit; none of these fields are optional.
#[derive(Debug, Clone, Copy)]
pub struct RestoreContext<'a> {
    /// `origin_hash` of the daemon being migrated. Must match
    /// `snapshot.entity_id.origin_hash()`.
    pub daemon_origin: u32,
    /// The snapshot to restore from.
    pub snapshot: &'a StateSnapshot,
    /// Node the daemon is migrating from.
    pub source_node: u64,
    /// Node that initiated this migration. Reply messages
    /// (RestoreComplete / ReplayComplete / ActivateAck) route here
    /// instead of to the immediate wire hop.
    pub orchestrator_node: u64,
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
        ctx: RestoreContext<'_>,
        keypair: EntityKeypair,
        daemon_factory: F,
        config: DaemonHostConfig,
    ) -> Result<(), MigrationError>
    where
        F: FnOnce() -> Box<dyn MeshDaemon>,
    {
        let RestoreContext {
            daemon_origin,
            snapshot,
            source_node,
            orchestrator_node,
        } = ctx;

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
    /// the authoritative copy. **Idempotent** for a retried
    /// `ActivateTarget` after a lost `ActivateAck`: if no active
    /// migration exists but a completed record does, returns the stored
    /// `replayed_through` so the subprotocol handler can re-emit the
    /// same ack.
    ///
    /// An active migration in `self.migrations` always takes precedence
    /// over a completed record for the same origin: a new migration for
    /// the same daemon (e.g., migrated back to us later) must not be
    /// skipped just because we still remember the previous completion.
    pub fn activate(&self, daemon_origin: u32) -> Result<u64, MigrationError> {
        if let Some(entry) = self.migrations.get(&daemon_origin) {
            let mut state = entry.lock();
            state.phase = MigrationPhase::Cutover;
            self.drain_pending(&mut state)?;
            return Ok(state.replayed_through);
        }
        if let Some(done) = self.completed.get(&daemon_origin) {
            return Ok(done.replayed_through);
        }
        Err(MigrationError::DaemonNotFound(daemon_origin))
    }

    /// Mark migration as complete and move tracking state into the
    /// `completed` index so that a retried `ActivateTarget` after a lost
    /// `ActivateAck` can be handled idempotently.
    ///
    /// The daemon remains registered in the daemon registry — it's now
    /// the authoritative copy. Also removes the factory entry, since the
    /// target won't need to re-restore from an orchestrator retry once
    /// the migration has successfully completed.
    ///
    /// Atomicity vs `activate()` and `abort()`: the `migrations` write
    /// entry is held across the entire operation. That guard serializes
    /// us against:
    ///
    /// - a retried `activate()`, which calls `migrations.get()` and
    ///   blocks on the shard write lock; once we drop the entry the
    ///   migration is gone but `completed` already has the idempotency
    ///   record, so the retry resolves through the `completed` lookup;
    /// - a concurrent `abort()`, which would otherwise observe an empty
    ///   `migrations` after a remove-first, insert-second ordering and
    ///   `daemon_registry.unregister()` a daemon we just promoted to
    ///   authoritative. Holding the entry forces abort to wait, and
    ///   it then finds nothing and no-ops — which matches the legacy
    ///   semantics where a successful complete makes a racing abort a
    ///   no-op.
    ///
    /// `completed.insert` happens while the entry is held, so a third
    /// thread observing both maps still sees the migration in at least
    /// one of them at every instant — closing the original
    /// `DaemonNotFound` gap on `activate()` retries.
    pub fn complete(&self, daemon_origin: u32) -> Result<(), MigrationError> {
        use dashmap::mapref::entry::Entry;
        match self.migrations.entry(daemon_origin) {
            Entry::Occupied(occ) => {
                let completion = {
                    let state = occ.get().lock();
                    CompletedTargetState {
                        orchestrator_node: state.orchestrator_node,
                        replayed_through: state.replayed_through,
                        completed_at: Instant::now(),
                    }
                };
                // Insert into `completed` before dropping the entry
                // guard so a concurrent `activate()` cannot observe
                // both maps empty.
                self.completed.insert(daemon_origin, completion);
                // Removes from `migrations` and drops the entry guard.
                occ.remove();
            }
            Entry::Vacant(_) => {
                // Vacant + completed-record-present is the lost-ack
                // retry path. Vacant + no completed record is a stale
                // origin we never knew about.
                if self.completed.contains_key(&daemon_origin) {
                    return Ok(());
                }
                return Err(MigrationError::DaemonNotFound(daemon_origin));
            }
        }
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
    ///
    /// On a mid-batch delivery failure, advances `replayed_through` past
    /// the events that *did* land and re-inserts the undelivered tail back
    /// into `pending_events` so a subsequent `drain_pending` (triggered
    /// by the next `replay_events` / `buffer_event` / `activate`) resumes
    /// at the failure point.
    ///
    /// BUG #1: pre-fix, this returned `?` on the first delivery error
    /// without updating `replayed_through` and without restoring the
    /// remaining events. Every event in `to_replay` had already been
    /// removed from `pending_events` upstream, so on retry the
    /// undelivered tail was simply gone — silent, permanent desync
    /// between source and target for any non-empty replay batch where
    /// one delivery errored mid-loop.
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

        // Deliver events to daemon via registry. Track how many actually
        // landed; on failure we persist the prefix and restore the tail.
        let mut delivered = 0usize;
        let mut failure: Option<MigrationError> = None;
        for event in &to_replay {
            match self.daemon_registry.deliver(state.daemon_origin, event) {
                Ok(_) => delivered += 1,
                Err(e) => {
                    failure = Some(MigrationError::StateFailed(e.to_string()));
                    break;
                }
            }
        }

        if delivered > 0 {
            let last = &to_replay[delivered - 1];
            state.replayed_through = last.link.sequence;
            state.target_head = last.link;
        }

        if let Some(err) = failure {
            // Restore the undelivered tail so the next drain_pending
            // call replays from the failure point. Without this, any
            // event with sequence > the failed one — already removed
            // upstream — would be lost forever, since the source has
            // moved on and won't re-send it.
            for event in to_replay.into_iter().skip(delivered) {
                state.pending_events.insert(event.link.sequence, event);
            }
            return Err(err);
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
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snapshot,
                    source_node: 0x1111,
                    orchestrator_node: 0x2222,
                },
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
                RestoreContext {
                    daemon_origin: 0xDEAD,
                    snapshot: &snapshot,
                    source_node: 0x1111,
                    orchestrator_node: 0x2222,
                },
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
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snapshot,
                    source_node: 0x1111,
                    orchestrator_node: 0x2222,
                },
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
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snapshot,
                    source_node: 0x1111,
                    orchestrator_node: 0x2222,
                },
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
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snapshot,
                    source_node: 0x1111,
                    orchestrator_node: 0x2222,
                },
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();

        handler.abort(origin).unwrap();
        assert!(!handler.is_migrating(origin));
        assert!(!reg.contains(origin)); // daemon unregistered on abort
    }

    #[test]
    fn test_regression_activate_target_idempotent_after_ack_loss() {
        // Regression: `complete()` used to remove state eagerly and the
        // target ack was sent BEFORE the idempotency record existed. A
        // retried `ActivateTarget` after a lost `ActivateAck` would hit
        // `DaemonNotFound` on `activate()`, wedging the orchestrator.
        //
        // Fix: `complete()` moves state into a `completed` index; a
        // retried `activate()` looks the completion up and returns the
        // same `replayed_through`.
        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationTargetHandler::new(reg);
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let snapshot = make_snapshot(&kp, 0, 0);

        handler
            .restore_snapshot(
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snapshot,
                    source_node: 0x1111,
                    orchestrator_node: 0x2222,
                },
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();

        let first_seq = handler.activate(origin).unwrap();
        handler.complete(origin).unwrap();

        // Simulate the orchestrator's retry: ActivateTarget arrives again.
        // Must return the same replayed_through, and complete() must be
        // a no-op — not a DaemonNotFound error.
        let retry_seq = handler.activate(origin).unwrap();
        assert_eq!(
            retry_seq, first_seq,
            "retried activate() must return the originally-activated seq"
        );
        handler
            .complete(origin)
            .expect("repeated complete() must no-op, not error");

        // Recorded orchestrator is still queryable via the completed record.
        assert_eq!(handler.orchestrator_node(origin), Some(0x2222));
    }

    #[test]
    fn test_regression_activate_prefers_active_over_completed() {
        // Regression: `activate()` used to consult the `completed` index
        // first and returned stale `replayed_through` from a prior
        // migration. A new active migration for the same daemon_origin
        // (e.g., migrated back later) would skip cutover and report a
        // wrong sequence number.
        //
        // Fix: active migrations always take precedence over completed
        // records for the same origin.
        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationTargetHandler::new(reg.clone());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();

        // First migration: through_seq = 10.
        let snap1 = make_snapshot(&kp, 10, 42);
        handler
            .restore_snapshot(
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snap1,
                    source_node: 0x1111,
                    orchestrator_node: 0x2222,
                },
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();
        handler.activate(origin).unwrap();
        handler.complete(origin).unwrap();

        // Simulate the daemon migrating away: unregister from the local
        // DaemonRegistry so the restore of the second migration can
        // re-register it (mirrors what `complete()` would do on the
        // source side in production).
        reg.unregister(origin).unwrap();

        // Second migration for the SAME origin (e.g., migrated away then
        // back), with a later through_seq = 100.
        let snap2 = make_snapshot(&kp, 100, 42);
        handler
            .restore_snapshot(
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snap2,
                    source_node: 0x3333,
                    orchestrator_node: 0x4444,
                },
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();

        let seq = handler.activate(origin).unwrap();
        assert_eq!(
            seq, 100,
            "activate() must reflect the NEW active migration, not the old completed one"
        );
        assert_eq!(
            handler.phase(origin),
            Some(MigrationPhase::Cutover),
            "new active migration must transition to Cutover"
        );
    }

    #[test]
    fn test_regression_complete_prefers_active_over_completed() {
        // Regression: `complete()` returned `Ok(())` early when a
        // completed record already existed, even if an active migration
        // for the same origin was in-flight. That left the new migration
        // stuck in its pre-cutover phase and its state unmoved to the
        // idempotency index.
        //
        // Fix: complete() finalizes the active migration if one exists;
        // only no-ops when NO active migration is present.
        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationTargetHandler::new(reg.clone());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();

        let snap1 = make_snapshot(&kp, 10, 42);
        handler
            .restore_snapshot(
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snap1,
                    source_node: 0x1111,
                    orchestrator_node: 0x2222,
                },
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();
        handler.activate(origin).unwrap();
        handler.complete(origin).unwrap();

        // Simulate migrate-away before the second migration arrives.
        reg.unregister(origin).unwrap();

        // Second migration for the same origin.
        let snap2 = make_snapshot(&kp, 100, 42);
        handler
            .restore_snapshot(
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snap2,
                    source_node: 0x3333,
                    orchestrator_node: 0x4444,
                },
                kp.clone(),
                || Box::new(AccumDaemon { total: 0 }),
                DaemonHostConfig::default(),
            )
            .unwrap();
        assert!(handler.is_migrating(origin));

        handler.activate(origin).unwrap();
        // complete() must actually finalize the new migration, not
        // short-circuit because a completed record from the prior one
        // exists.
        handler.complete(origin).unwrap();
        assert!(
            !handler.is_migrating(origin),
            "complete() must move the new migration to the completed index"
        );
        assert_eq!(
            handler.orchestrator_node(origin),
            Some(0x4444),
            "completed record must reflect the second (new) orchestrator"
        );
    }

    #[test]
    fn test_regression_complete_activate_no_intermediate_gap() {
        // Regression: `complete()` removed from `migrations` BEFORE
        // inserting into `completed`. A concurrent `activate()` retry
        // landing in the gap would observe neither map and return
        // `DaemonNotFound`, breaking the idempotency contract that
        // `activate()` documents.
        //
        // The race window is sub-microsecond, so the test is structured
        // as continuous mutual stress: a long-lived observer thread
        // spin-loops calling `activate()` while the main thread cycles
        // through many `restore → activate → complete` rounds. Tight
        // atomic-flag handshakes (rather than a `Barrier`) keep the
        // threads aligned closely enough to land observer probes inside
        // the gap. With the bug, this hits `DaemonNotFound` reliably.
        // With the fix, it never does.
        use std::collections::HashSet;
        use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
        use std::thread;

        const TRIALS: u32 = 2_000;
        const STOP: u32 = u32::MAX;

        let reg = Arc::new(DaemonRegistry::new());
        let handler = Arc::new(MigrationTargetHandler::new(reg.clone()));

        // Observer is signaled by writing the current trial's origin
        // into `current_origin`; `STOP` ends the observer. The observer
        // reports any DaemonNotFound it sees via `gap_seen`.
        let current_origin = Arc::new(AtomicU32::new(0));
        let gap_seen = Arc::new(AtomicU64::new(0));

        let h_observer = handler.clone();
        let origin_observer = current_origin.clone();
        let gap_observer = gap_seen.clone();
        let observer = thread::spawn(move || loop {
            let origin = origin_observer.load(Ordering::Acquire);
            if origin == STOP {
                return;
            }
            if origin == 0 {
                std::hint::spin_loop();
                continue;
            }
            match h_observer.activate(origin) {
                Ok(_) => {}
                Err(MigrationError::DaemonNotFound(o)) => {
                    gap_observer.store(o as u64, Ordering::Release);
                    return;
                }
                Err(_) => {}
            }
        });

        // Track origins we've already used so a 32-bit `origin_hash`
        // collision doesn't trip `restore_snapshot` (the daemon
        // registry rejects re-registration of an already-registered
        // origin since we deliberately do NOT unregister between
        // trials). The birthday probability of a collision over
        // 2_000 trials of a 32-bit space is ~5e-4, so without this
        // guard the test would hang on the unwrap a small fraction
        // of the time.
        let mut seen_origins: HashSet<u32> = HashSet::with_capacity(TRIALS as usize);

        for _ in 0..TRIALS {
            if gap_seen.load(Ordering::Acquire) != 0 {
                break;
            }
            let kp = EntityKeypair::generate();
            let origin = kp.origin_hash();
            // Skip the reserved sentinels and any origin already used
            // by a prior trial.
            if origin == 0 || origin == STOP || !seen_origins.insert(origin) {
                continue;
            }
            let snapshot = make_snapshot(&kp, 5, 0);
            handler
                .restore_snapshot(
                    RestoreContext {
                        daemon_origin: origin,
                        snapshot: &snapshot,
                        source_node: 0x1111,
                        orchestrator_node: 0x2222,
                    },
                    kp.clone(),
                    || Box::new(AccumDaemon { total: 0 }),
                    DaemonHostConfig::default(),
                )
                .unwrap();
            handler.activate(origin).unwrap();

            // Hand the origin to the observer, then race complete()
            // against its activate() spin-loop. We deliberately do NOT
            // call `forget_completed` / `unregister` between trials:
            // doing so would let the observer race and read a forgotten
            // origin (test artifact, not a production bug). Each trial
            // uses a fresh origin so accumulation is bounded by TRIALS.
            current_origin.store(origin, Ordering::Release);
            handler.complete(origin).unwrap();
            current_origin.store(0, Ordering::Release);
        }

        current_origin.store(STOP, Ordering::Release);
        observer.join().unwrap();

        let gap = gap_seen.load(Ordering::Acquire);
        assert_eq!(
            gap, 0,
            "concurrent activate() observed a DaemonNotFound gap during complete() \
             for origin {gap:#x} — the migration was unobservable in both \
             `migrations` and `completed`"
        );
    }

    #[test]
    fn test_regression_complete_abort_no_inconsistent_state() {
        // Regression: an earlier version of the `complete()` ordering
        // fix did `completed.insert(...)` followed by
        // `migrations.remove(...)` outside any shared guard. A
        // concurrent `abort()` racing in between would observe
        // `migrations` still occupied (from before the insert step
        // released the shard) and unregister the daemon — leaving
        // `completed` with an idempotency record for an origin that
        // is no longer present in the registry. A subsequent
        // `activate()` retry would then resolve happily through the
        // completed record while routing pointed at a daemon that
        // had been silently torn down.
        //
        // The fix takes a write entry on `migrations` and holds it
        // across both `completed.insert` and the migrations remove.
        // With the entry held, `abort()`'s `migrations.remove()`
        // serializes after us and finds nothing, so it never reaches
        // its `unregister` branch. This test stresses the race and
        // asserts the invariant: `completed.contains(origin)` implies
        // `daemon_registry.contains(origin)`.
        use std::collections::HashSet;
        use std::sync::Barrier;
        use std::thread;

        const TRIALS: u32 = 1_000;

        let reg = Arc::new(DaemonRegistry::new());
        let handler = Arc::new(MigrationTargetHandler::new(reg.clone()));
        let mut seen_origins: HashSet<u32> = HashSet::with_capacity(TRIALS as usize);

        for _ in 0..TRIALS {
            let kp = EntityKeypair::generate();
            let origin = kp.origin_hash();
            if origin == 0 || !seen_origins.insert(origin) {
                continue;
            }
            let snapshot = make_snapshot(&kp, 5, 0);
            handler
                .restore_snapshot(
                    RestoreContext {
                        daemon_origin: origin,
                        snapshot: &snapshot,
                        source_node: 0x1111,
                        orchestrator_node: 0x2222,
                    },
                    kp.clone(),
                    || Box::new(AccumDaemon { total: 0 }),
                    DaemonHostConfig::default(),
                )
                .unwrap();
            handler.activate(origin).unwrap();

            let barrier = Arc::new(Barrier::new(2));

            let h_complete = handler.clone();
            let b_complete = barrier.clone();
            let completer = thread::spawn(move || {
                b_complete.wait();
                let _ = h_complete.complete(origin);
            });

            let h_abort = handler.clone();
            let b_abort = barrier.clone();
            let aborter = thread::spawn(move || {
                b_abort.wait();
                let _ = h_abort.abort(origin);
            });

            completer.join().unwrap();
            aborter.join().unwrap();

            // Invariant: if a completed record exists for this origin,
            // the daemon must still be registered. Bug allowed the
            // opposite — completed.insert wins, abort.unregister wins,
            // resulting in a "completed" record for an unregistered
            // daemon.
            if handler.orchestrator_node(origin).is_some() {
                assert!(
                    reg.contains(origin),
                    "complete() promoted origin {origin:#x} to authoritative \
                     while a concurrent abort() unregistered it — \
                     completed-record-without-registered-daemon is the bug \
                     this test is pinning"
                );
            }
        }
    }

    /// BUG #1: a daemon whose `process()` fails on event N of M during
    /// `replay_events` would, pre-fix, lose every event with seq > N
    /// permanently — `drain_pending` removed all M events from
    /// `pending_events` upstream of the delivery loop, and the `?`
    /// early-return on the failure left the undelivered tail in the
    /// local `to_replay` Vec which then dropped on function exit.
    /// `replayed_through` was also left at its pre-batch value, so a
    /// retry replayed the prefix again (re-incrementing the daemon's
    /// counters) but never reached the failed event again.
    ///
    /// Post-fix: a mid-batch failure advances `replayed_through` past
    /// the events that did land, restores the undelivered tail
    /// (including the failed event itself) into `pending_events`, and
    /// returns the error. A subsequent `replay_events` /
    /// `buffer_event` / `activate` triggers another `drain_pending`
    /// that picks up exactly where the previous one stopped.
    #[test]
    fn drain_pending_restores_undelivered_tail_on_mid_batch_failure() {
        use std::sync::atomic::{AtomicU64, Ordering};

        // Daemon that fails on the Nth `process` call (1-indexed).
        struct FailOnNth {
            count: Arc<AtomicU64>,
            fail_at: u64,
            state: u64,
        }
        impl MeshDaemon for FailOnNth {
            fn name(&self) -> &str { "fail-on-nth" }
            fn requirements(&self) -> CapabilityFilter { CapabilityFilter::default() }
            fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
                let n = self.count.fetch_add(1, Ordering::SeqCst) + 1;
                if n == self.fail_at {
                    return Err(DaemonError::ProcessFailed("simulated".into()));
                }
                self.state += 1;
                Ok(vec![])
            }
            fn snapshot(&self) -> Option<Bytes> {
                Some(Bytes::from(self.state.to_le_bytes().to_vec()))
            }
            fn restore(&mut self, state: Bytes) -> Result<(), DaemonError> {
                if state.len() != 8 {
                    return Err(DaemonError::RestoreFailed("bad size".into()));
                }
                self.state = u64::from_le_bytes(state[..8].try_into().unwrap());
                Ok(())
            }
        }

        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationTargetHandler::new(reg.clone());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let snapshot = make_snapshot(&kp, 0, 0);
        let count = Arc::new(AtomicU64::new(0));

        // Fail on the 3rd process() call — events at seq 1, 2 should
        // land; seq 3 fails; seq 4, 5 should be retained for retry.
        let count_for_factory = count.clone();
        handler
            .restore_snapshot(
                RestoreContext {
                    daemon_origin: origin,
                    snapshot: &snapshot,
                    source_node: 0x1111,
                    orchestrator_node: 0x2222,
                },
                kp.clone(),
                move || Box::new(FailOnNth {
                    count: count_for_factory.clone(),
                    fail_at: 3,
                    state: 0,
                }),
                DaemonHostConfig::default(),
            )
            .unwrap();

        let events = vec![
            make_event(0xBBBB, 1),
            make_event(0xBBBB, 2),
            make_event(0xBBBB, 3),
            make_event(0xBBBB, 4),
            make_event(0xBBBB, 5),
        ];

        // First replay: should fail mid-batch on event 3.
        let err = handler.replay_events(origin, events).unwrap_err();
        assert!(
            err.to_string().contains("simulated"),
            "expected the simulated process() failure, got: {err}"
        );

        // Pre-fix this would be 0 (no advance) and events 4, 5 would
        // be gone. Post-fix: advanced past 1, 2 — the prefix that
        // actually landed.
        assert_eq!(
            handler.replayed_through(origin),
            Some(2),
            "replayed_through must advance past the events that did \
             land before the failure (BUG #1)"
        );

        // Confirm 3 process() calls happened (1 OK, 2 OK, 3 fail).
        assert_eq!(count.load(Ordering::SeqCst), 3);

        // Second drain — issue an empty `replay_events` to retrigger
        // drain_pending. Pre-fix this would be a no-op because
        // pending_events was empty; post-fix events 3, 4, 5 are
        // still there and replay resumes from seq 3.
        //
        // Reset the failure counter so the daemon now succeeds on
        // every call. (This simulates the operator clearing whatever
        // transient condition caused the original failure.)
        count.store(100, Ordering::SeqCst); // > fail_at, never matches

        let replayed = handler.replay_events(origin, vec![]).unwrap();
        assert_eq!(
            replayed, 5,
            "second drain must replay seq 3, 4, 5 — pre-fix these were \
             permanently lost when the first batch errored (BUG #1)"
        );
        assert_eq!(handler.replayed_through(origin), Some(5));
    }
}
