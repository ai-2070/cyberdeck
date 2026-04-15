//! Stateful daemon migration.
//!
//! Migration uses L4 `StateSnapshot` to move a daemon between nodes while
//! preserving causal chain continuity. The process is a 6-phase state machine.

use crate::adapter::nltp::state::causal::CausalEvent;
use crate::adapter::nltp::state::snapshot::StateSnapshot;

/// Subprotocol ID for migration control messages.
pub const SUBPROTOCOL_MIGRATION: u16 = 0x0500;

/// Phases of daemon migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationPhase {
    /// Take snapshot on source node.
    Snapshot,
    /// Transfer snapshot to target node.
    Transfer,
    /// Restore daemon on target, start buffering events.
    Restore,
    /// Replay buffered events on target.
    Replay,
    /// Atomic routing cutover: new events go to target.
    Cutover,
    /// Cleanup source.
    Complete,
}

/// State of an in-progress migration.
pub struct MigrationState {
    /// Origin hash of the daemon being migrated.
    daemon_origin: u32,
    /// Source node ID.
    source_node: u64,
    /// Target node ID.
    target_node: u64,
    /// Current phase (only mutable through transition methods).
    phase: MigrationPhase,
    /// Snapshot taken from source (set in Snapshot phase).
    snapshot: Option<StateSnapshot>,
    /// Events buffered between snapshot and cutover.
    buffered_events: Vec<CausalEvent>,
    /// Timestamp when migration started (nanos).
    started_at: u64,
}

impl MigrationState {
    /// Create a new migration.
    pub fn new(daemon_origin: u32, source_node: u64, target_node: u64) -> Self {
        Self {
            daemon_origin,
            source_node,
            target_node,
            phase: MigrationPhase::Snapshot,
            snapshot: None,
            buffered_events: Vec::new(),
            started_at: current_timestamp(),
        }
    }

    /// Buffer an event that arrived during migration.
    pub fn buffer_event(&mut self, event: CausalEvent) {
        self.buffered_events.push(event);
    }

    /// Set the snapshot and advance to Transfer phase.
    pub fn set_snapshot(&mut self, snapshot: StateSnapshot) -> Result<(), MigrationError> {
        if self.phase != MigrationPhase::Snapshot {
            return Err(MigrationError::WrongPhase {
                expected: MigrationPhase::Snapshot,
                got: self.phase,
            });
        }
        // Validate snapshot belongs to the daemon being migrated
        if snapshot.entity_id.origin_hash() != self.daemon_origin {
            return Err(MigrationError::StateFailed(format!(
                "snapshot origin {:#x} does not match daemon {:#x}",
                snapshot.entity_id.origin_hash(),
                self.daemon_origin,
            )));
        }
        self.snapshot = Some(snapshot);
        self.phase = MigrationPhase::Transfer;
        Ok(())
    }

    /// Mark transfer complete, advance to Restore.
    pub fn transfer_complete(&mut self) -> Result<(), MigrationError> {
        if self.phase != MigrationPhase::Transfer {
            return Err(MigrationError::WrongPhase {
                expected: MigrationPhase::Transfer,
                got: self.phase,
            });
        }
        self.phase = MigrationPhase::Restore;
        Ok(())
    }

    /// Mark restore complete, advance to Replay.
    pub fn restore_complete(&mut self) -> Result<(), MigrationError> {
        if self.phase != MigrationPhase::Restore {
            return Err(MigrationError::WrongPhase {
                expected: MigrationPhase::Restore,
                got: self.phase,
            });
        }
        self.phase = MigrationPhase::Replay;
        Ok(())
    }

    /// Take buffered events for replay (drains the buffer).
    pub fn take_buffered_events(&mut self) -> Vec<CausalEvent> {
        std::mem::take(&mut self.buffered_events)
    }

    /// Mark replay complete, advance to Cutover.
    pub fn replay_complete(&mut self) -> Result<(), MigrationError> {
        if self.phase != MigrationPhase::Replay {
            return Err(MigrationError::WrongPhase {
                expected: MigrationPhase::Replay,
                got: self.phase,
            });
        }
        self.phase = MigrationPhase::Cutover;
        Ok(())
    }

    /// Mark cutover complete, advance to Complete.
    pub fn cutover_complete(&mut self) -> Result<(), MigrationError> {
        if self.phase != MigrationPhase::Cutover {
            return Err(MigrationError::WrongPhase {
                expected: MigrationPhase::Cutover,
                got: self.phase,
            });
        }
        self.phase = MigrationPhase::Complete;
        Ok(())
    }

    /// Check if migration is finished.
    pub fn is_complete(&self) -> bool {
        self.phase == MigrationPhase::Complete
    }

    /// Elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        (current_timestamp().saturating_sub(self.started_at)) / 1_000_000
    }

    /// Get the daemon origin hash.
    #[inline]
    pub fn daemon_origin(&self) -> u32 {
        self.daemon_origin
    }

    /// Get the source node ID.
    #[inline]
    pub fn source_node(&self) -> u64 {
        self.source_node
    }

    /// Get the target node ID.
    #[inline]
    pub fn target_node(&self) -> u64 {
        self.target_node
    }

    /// Get the current phase.
    #[inline]
    pub fn phase(&self) -> MigrationPhase {
        self.phase
    }

    /// Get the snapshot (if taken).
    #[inline]
    pub fn snapshot(&self) -> Option<&StateSnapshot> {
        self.snapshot.as_ref()
    }
}

impl std::fmt::Debug for MigrationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationState")
            .field("daemon", &format!("{:#x}", self.daemon_origin))
            .field("source", &format!("{:#x}", self.source_node))
            .field("target", &format!("{:#x}", self.target_node))
            .field("phase", &self.phase)
            .field("buffered", &self.buffered_events.len())
            .field("has_snapshot", &self.snapshot.is_some())
            .finish()
    }
}

/// Errors from migration operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// Daemon not registered locally.
    DaemonNotFound(u32),
    /// Target node unreachable or refused.
    TargetUnavailable(u64),
    /// Snapshot/restore failure.
    StateFailed(String),
    /// Migration already in progress for this daemon.
    AlreadyMigrating(u32),
    /// Attempted to advance from wrong phase.
    WrongPhase {
        /// Expected phase.
        expected: MigrationPhase,
        /// Actual phase.
        got: MigrationPhase,
    },
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DaemonNotFound(id) => write!(f, "daemon {:#x} not found", id),
            Self::TargetUnavailable(id) => write!(f, "target node {:#x} unavailable", id),
            Self::StateFailed(msg) => write!(f, "state operation failed: {}", msg),
            Self::AlreadyMigrating(id) => write!(f, "daemon {:#x} already migrating", id),
            Self::WrongPhase { expected, got } => {
                write!(
                    f,
                    "wrong migration phase: expected {:?}, got {:?}",
                    expected, got
                )
            }
        }
    }
}

impl std::error::Error for MigrationError {}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::nltp::state::causal::CausalLink;
    use bytes::Bytes;

    fn make_event(seq: u64) -> CausalEvent {
        CausalEvent {
            link: CausalLink {
                origin_hash: 0xAAAA,
                horizon_encoded: 0,
                sequence: seq,
                parent_hash: 0,
            },
            payload: Bytes::from_static(b"data"),
            received_at: 0,
        }
    }

    #[test]
    fn test_migration_phase_progression() {
        let kp = crate::adapter::nltp::identity::EntityKeypair::generate();
        let origin = kp.origin_hash();
        let mut state = MigrationState::new(origin, 0x1111, 0x2222);
        assert_eq!(state.phase(), MigrationPhase::Snapshot);

        // Can't skip phases
        assert!(state.transfer_complete().is_err());

        // Normal progression
        let snapshot = StateSnapshot::new(
            kp.entity_id().clone(),
            CausalLink::genesis(origin, 0),
            Bytes::from_static(b"state"),
            crate::adapter::nltp::state::horizon::ObservedHorizon::new(),
        );

        state.set_snapshot(snapshot).unwrap();
        assert_eq!(state.phase(), MigrationPhase::Transfer);

        state.transfer_complete().unwrap();
        assert_eq!(state.phase(), MigrationPhase::Restore);

        state.restore_complete().unwrap();
        assert_eq!(state.phase(), MigrationPhase::Replay);

        state.replay_complete().unwrap();
        assert_eq!(state.phase(), MigrationPhase::Cutover);

        state.cutover_complete().unwrap();
        assert_eq!(state.phase(), MigrationPhase::Complete);
        assert!(state.is_complete());
    }

    #[test]
    fn test_event_buffering() {
        let mut state = MigrationState::new(0xAAAA, 0x1111, 0x2222);

        state.buffer_event(make_event(1));
        state.buffer_event(make_event(2));
        state.buffer_event(make_event(3));

        let events = state.take_buffered_events();
        assert_eq!(events.len(), 3);
        assert!(state.buffered_events.is_empty());
    }

    #[test]
    fn test_wrong_phase_error() {
        let mut state = MigrationState::new(0xAAAA, 0x1111, 0x2222);

        let err = state.restore_complete().unwrap_err();
        assert_eq!(
            err,
            MigrationError::WrongPhase {
                expected: MigrationPhase::Restore,
                got: MigrationPhase::Snapshot,
            }
        );
    }

    // ---- Regression tests for Cubic AI findings ----

    #[test]
    fn test_regression_set_snapshot_rejects_wrong_origin() {
        // Regression: set_snapshot accepted snapshots from any entity,
        // allowing migration to bind state from the wrong daemon.
        let kp = crate::adapter::nltp::identity::EntityKeypair::generate();
        let wrong_origin = kp.origin_hash();

        // Migration is for daemon 0xBBBB, but snapshot is for kp's origin
        let mut state = MigrationState::new(0xBBBB, 0x1111, 0x2222);

        let snapshot = StateSnapshot::new(
            kp.entity_id().clone(),
            CausalLink::genesis(wrong_origin, 0),
            Bytes::from_static(b"state"),
            crate::adapter::nltp::state::horizon::ObservedHorizon::new(),
        );

        assert!(
            state.set_snapshot(snapshot).is_err(),
            "set_snapshot must reject snapshot from a different daemon"
        );
    }
}
