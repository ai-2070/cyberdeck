//! Source-side migration handler.
//!
//! Manages the source node's role in migration: taking a snapshot, buffering
//! events during transfer/replay, executing cutover (stop writes), and
//! cleaning up the daemon after the target is live.

use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use parking_lot::Mutex;

use super::migration::{MigrationError, MigrationPhase};
use super::registry::DaemonRegistry;
use crate::adapter::net::state::causal::CausalEvent;
use crate::adapter::net::state::snapshot::StateSnapshot;

/// Per-daemon source-side migration state.
#[allow(dead_code)]
struct SourceMigrationState {
    daemon_origin: u32,
    target_node: u64,
    phase: MigrationPhase,
    snapshot: Option<StateSnapshot>,
    /// Events buffered between snapshot and cutover, in sequence order.
    buffered_events: Vec<CausalEvent>,
    /// Last buffered event sequence number.
    last_buffered_seq: u64,
    started_at: Instant,
}

/// Handles the source node's role in daemon migration.
///
/// The source handler:
/// 1. Takes a snapshot of the daemon (phase 0)
/// 2. Buffers events arriving for the daemon during migration (phases 0-3)
/// 3. Stops accepting writes at cutover (phase 4)
/// 4. Unregisters the daemon and cleans up (phase 5)
pub struct MigrationSourceHandler {
    /// Local daemon registry.
    daemon_registry: Arc<DaemonRegistry>,
    /// Active migrations on this node as source: daemon_origin → state.
    migrations: DashMap<u32, Mutex<SourceMigrationState>>,
}

impl MigrationSourceHandler {
    /// Create a new source handler.
    pub fn new(daemon_registry: Arc<DaemonRegistry>) -> Self {
        Self {
            daemon_registry,
            migrations: DashMap::new(),
        }
    }

    /// Phase 0: Take snapshot of a local daemon.
    ///
    /// Registers the migration and returns the snapshot for transfer.
    pub fn start_snapshot(
        &self,
        daemon_origin: u32,
        target_node: u64,
    ) -> Result<StateSnapshot, MigrationError> {
        // Atomic check-and-reserve via entry() to prevent TOCTOU races
        let entry = match self.migrations.entry(daemon_origin) {
            dashmap::mapref::entry::Entry::Occupied(_) => {
                return Err(MigrationError::AlreadyMigrating(daemon_origin));
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => entry,
        };

        if !self.daemon_registry.contains(daemon_origin) {
            return Err(MigrationError::DaemonNotFound(daemon_origin));
        }

        let snapshot = self
            .daemon_registry
            .snapshot(daemon_origin)
            .map_err(|e| MigrationError::StateFailed(e.to_string()))?
            .ok_or_else(|| {
                MigrationError::StateFailed("daemon is stateless or snapshot failed".into())
            })?;

        entry.insert(Mutex::new(SourceMigrationState {
            daemon_origin,
            target_node,
            phase: MigrationPhase::Snapshot,
            snapshot: Some(snapshot.clone()),
            buffered_events: Vec::new(),
            last_buffered_seq: snapshot.through_seq,
            started_at: Instant::now(),
        }));

        Ok(snapshot)
    }

    /// Buffer an event arriving for a daemon during migration.
    ///
    /// Events are buffered during Snapshot through Replay phases.
    /// Returns `Ok(true)` if buffered, `Ok(false)` if no migration active
    /// or past cutover. Returns `Err` if the daemon was cut over (writes rejected).
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
        match state.phase {
            MigrationPhase::Snapshot
            | MigrationPhase::Transfer
            | MigrationPhase::Restore
            | MigrationPhase::Replay => {
                state.last_buffered_seq = event.link.sequence;
                state.buffered_events.push(event);
                Ok(true)
            }
            MigrationPhase::Cutover | MigrationPhase::Complete => {
                // After cutover, source rejects writes
                Err(MigrationError::StateFailed(format!(
                    "daemon {:#x} has been cut over, writes rejected",
                    daemon_origin,
                )))
            }
        }
    }

    /// Check if a daemon is being migrated from this node.
    pub fn is_migrating(&self, daemon_origin: u32) -> bool {
        self.migrations.contains_key(&daemon_origin)
    }

    /// Get buffered events for transfer to the target (during replay phase).
    ///
    /// Drains the buffer — events are moved, not copied.
    pub fn take_buffered_events(
        &self,
        daemon_origin: u32,
    ) -> Result<Vec<CausalEvent>, MigrationError> {
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        let mut state = entry.lock();
        Ok(std::mem::take(&mut state.buffered_events))
    }

    /// Phase 4: Cutover — stop accepting writes for this daemon.
    pub fn on_cutover(&self, daemon_origin: u32) -> Result<Vec<CausalEvent>, MigrationError> {
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        let mut state = entry.lock();
        state.phase = MigrationPhase::Cutover;

        // Return any remaining buffered events for final sync
        Ok(std::mem::take(&mut state.buffered_events))
    }

    /// Phase 5: Cleanup — unregister daemon from this node.
    ///
    /// Removes the daemon from the local registry and clears migration state.
    pub fn cleanup(&self, daemon_origin: u32) -> Result<(), MigrationError> {
        // Unregister daemon from local registry
        let _ = self.daemon_registry.unregister(daemon_origin);

        // Remove migration state
        self.migrations.remove(&daemon_origin);

        Ok(())
    }

    /// Abort a migration — return to normal operation.
    ///
    /// Clears migration state. The daemon remains registered locally.
    pub fn abort(&self, daemon_origin: u32) -> Result<(), MigrationError> {
        self.migrations.remove(&daemon_origin);
        Ok(())
    }

    /// Get the current phase of a migration on this source.
    pub fn phase(&self, daemon_origin: u32) -> Option<MigrationPhase> {
        self.migrations
            .get(&daemon_origin)
            .map(|entry| entry.lock().phase)
    }

    /// Number of active source-side migrations.
    pub fn active_count(&self) -> usize {
        self.migrations.len()
    }
}

impl std::fmt::Debug for MigrationSourceHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationSourceHandler")
            .field("active_migrations", &self.migrations.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::behavior::capability::CapabilityFilter;
    use crate::adapter::net::compute::{DaemonError, DaemonHost, DaemonHostConfig, MeshDaemon};
    use crate::adapter::net::identity::EntityKeypair;
    use crate::adapter::net::state::causal::CausalLink;
    use bytes::Bytes;

    struct StatefulDaemon {
        value: u64,
    }

    impl MeshDaemon for StatefulDaemon {
        fn name(&self) -> &str {
            "stateful"
        }
        fn requirements(&self) -> CapabilityFilter {
            CapabilityFilter::default()
        }
        fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
            self.value += 1;
            Ok(vec![])
        }
        fn snapshot(&self) -> Option<Bytes> {
            Some(Bytes::from(self.value.to_le_bytes().to_vec()))
        }
        fn restore(&mut self, state: Bytes) -> Result<(), DaemonError> {
            self.value = u64::from_le_bytes(state[..8].try_into().unwrap());
            Ok(())
        }
    }

    fn setup() -> (Arc<DaemonRegistry>, u32) {
        let reg = Arc::new(DaemonRegistry::new());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let host = DaemonHost::new(
            Box::new(StatefulDaemon { value: 42 }),
            kp,
            DaemonHostConfig::default(),
        );
        reg.register(host).unwrap();
        (reg, origin)
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

    #[test]
    fn test_start_snapshot() {
        let (reg, origin) = setup();
        let handler = MigrationSourceHandler::new(reg);

        let snapshot = handler.start_snapshot(origin, 0x2222).unwrap();
        assert_eq!(snapshot.entity_id.origin_hash(), origin);
        assert!(handler.is_migrating(origin));
    }

    #[test]
    fn test_start_snapshot_not_found() {
        let reg = Arc::new(DaemonRegistry::new());
        let handler = MigrationSourceHandler::new(reg);
        assert!(handler.start_snapshot(0xDEAD, 0x2222).is_err());
    }

    #[test]
    fn test_duplicate_snapshot_rejected() {
        let (reg, origin) = setup();
        let handler = MigrationSourceHandler::new(reg);

        handler.start_snapshot(origin, 0x2222).unwrap();
        let err = handler.start_snapshot(origin, 0x3333).unwrap_err();
        assert_eq!(err, MigrationError::AlreadyMigrating(origin));
    }

    #[test]
    fn test_buffer_events() {
        let (reg, origin) = setup();
        let handler = MigrationSourceHandler::new(reg);

        handler.start_snapshot(origin, 0x2222).unwrap();

        assert!(handler.buffer_event(origin, make_event(origin, 1)).unwrap());
        assert!(handler.buffer_event(origin, make_event(origin, 2)).unwrap());
        assert!(handler.buffer_event(origin, make_event(origin, 3)).unwrap());

        let events = handler.take_buffered_events(origin).unwrap();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_buffer_event_no_migration() {
        let (reg, _origin) = setup();
        let handler = MigrationSourceHandler::new(reg);

        let result = handler.buffer_event(0xDEAD, make_event(0xDEAD, 1)).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_cutover_rejects_writes() {
        let (reg, origin) = setup();
        let handler = MigrationSourceHandler::new(reg);

        handler.start_snapshot(origin, 0x2222).unwrap();
        handler.buffer_event(origin, make_event(origin, 1)).unwrap();

        // Cutover
        let remaining = handler.on_cutover(origin).unwrap();
        assert_eq!(remaining.len(), 1);

        // After cutover, buffer_event should reject
        let err = handler
            .buffer_event(origin, make_event(origin, 2))
            .unwrap_err();
        assert!(err.to_string().contains("cut over"));
    }

    #[test]
    fn test_cleanup() {
        let (reg, origin) = setup();
        let handler = MigrationSourceHandler::new(reg.clone());

        handler.start_snapshot(origin, 0x2222).unwrap();
        handler.on_cutover(origin).unwrap();
        handler.cleanup(origin).unwrap();

        assert!(!handler.is_migrating(origin));
        assert!(!reg.contains(origin)); // daemon unregistered
    }

    #[test]
    fn test_abort() {
        let (reg, origin) = setup();
        let handler = MigrationSourceHandler::new(reg.clone());

        handler.start_snapshot(origin, 0x2222).unwrap();
        handler.abort(origin).unwrap();

        assert!(!handler.is_migrating(origin));
        assert!(reg.contains(origin)); // daemon still registered
    }
}
