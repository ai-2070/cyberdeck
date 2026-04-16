//! Migration orchestrator — coordinates all 6 phases of daemon migration.
//!
//! The orchestrator runs on the controller node (which may be the source,
//! target, or a third-party coordinator). It sequences phase transitions,
//! routes migration messages, and handles failures/timeouts.

use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use parking_lot::Mutex;

use super::migration::{MigrationError, MigrationPhase, MigrationState};
use super::registry::DaemonRegistry;
use crate::adapter::net::continuity::superposition::SuperpositionState;
use crate::adapter::net::state::causal::{CausalEvent, CausalLink};
use crate::adapter::net::state::snapshot::StateSnapshot;

// ── Migration message protocol ──────────────────────────────────────────────

/// Wire message types for the migration subprotocol (0x0500).
#[derive(Debug, Clone)]
pub enum MigrationMessage {
    /// Phase 0→1: Request snapshot on source.
    TakeSnapshot {
        /// Origin hash of daemon to migrate.
        daemon_origin: u32,
        /// Destination node ID.
        target_node: u64,
    },

    /// Phase 1→2: Snapshot taken, payload included.
    ///
    /// Large snapshots are chunked across multiple `SnapshotReady` messages.
    /// The receiver must reassemble all chunks (0..total_chunks) before
    /// deserializing the snapshot. Single-chunk snapshots have
    /// `chunk_index = 0, total_chunks = 1`.
    SnapshotReady {
        /// Origin hash of daemon being migrated.
        daemon_origin: u32,
        /// Serialized `StateSnapshot` bytes (or chunk thereof).
        snapshot_bytes: Vec<u8>,
        /// Sequence number the snapshot covers through.
        seq_through: u64,
        /// Zero-based index of this chunk.
        chunk_index: u16,
        /// Total number of chunks for this snapshot.
        total_chunks: u16,
    },

    /// Phase 2→3: Target restored daemon from snapshot.
    RestoreComplete {
        /// Origin hash of daemon being migrated.
        daemon_origin: u32,
        /// Sequence number restored through.
        restored_seq: u64,
    },

    /// Phase 3→4: Target finished replaying buffered events.
    ReplayComplete {
        /// Origin hash of daemon being migrated.
        daemon_origin: u32,
        /// Sequence number replayed through.
        replayed_seq: u64,
    },

    /// Phase 4: Source stops accepting writes, routing switches.
    CutoverNotify {
        /// Origin hash of daemon being migrated.
        daemon_origin: u32,
        /// Target node that is now authoritative.
        target_node: u64,
    },

    /// Phase 5: Source cleaned up.
    CleanupComplete {
        /// Origin hash of daemon whose migration is complete.
        daemon_origin: u32,
    },

    /// Any phase: Abort migration.
    MigrationFailed {
        /// Origin hash of daemon whose migration failed.
        daemon_origin: u32,
        /// Human-readable reason for failure.
        reason: String,
    },

    /// Buffered events from source for replay on target.
    BufferedEvents {
        /// Origin hash of daemon being migrated.
        daemon_origin: u32,
        /// Events to replay, in causal order.
        events: Vec<CausalEvent>,
    },
}

// ── Wire format ─────────────────────────────────────────────────────────────

/// Wire format encode/decode for migration messages.
pub mod wire {
    use super::*;
    use bytes::{Buf, BufMut};

    /// Wire type: request snapshot on source.
    pub const MSG_TAKE_SNAPSHOT: u8 = 0;
    /// Wire type: snapshot taken, payload included.
    pub const MSG_SNAPSHOT_READY: u8 = 1;
    /// Wire type: target restored daemon from snapshot.
    pub const MSG_RESTORE_COMPLETE: u8 = 2;
    /// Wire type: target finished replaying buffered events.
    pub const MSG_REPLAY_COMPLETE: u8 = 3;
    /// Wire type: source stops writes, routing switches.
    pub const MSG_CUTOVER_NOTIFY: u8 = 4;
    /// Wire type: source cleaned up.
    pub const MSG_CLEANUP_COMPLETE: u8 = 5;
    /// Wire type: migration failed/aborted.
    pub const MSG_FAILED: u8 = 6;
    /// Wire type: buffered events for replay.
    pub const MSG_BUFFERED_EVENTS: u8 = 7;

    /// Encode a migration message to bytes.
    pub fn encode(msg: &MigrationMessage) -> Vec<u8> {
        let mut buf = Vec::with_capacity(128);

        match msg {
            MigrationMessage::TakeSnapshot {
                daemon_origin,
                target_node,
            } => {
                buf.put_u8(MSG_TAKE_SNAPSHOT);
                buf.put_u32_le(*daemon_origin);
                buf.put_u64_le(*target_node);
            }
            MigrationMessage::SnapshotReady {
                daemon_origin,
                snapshot_bytes,
                seq_through,
                chunk_index,
                total_chunks,
            } => {
                buf.put_u8(MSG_SNAPSHOT_READY);
                buf.put_u32_le(*daemon_origin);
                buf.put_u64_le(*seq_through);
                buf.put_u16_le(*chunk_index);
                buf.put_u16_le(*total_chunks);
                buf.put_u32_le(snapshot_bytes.len() as u32);
                buf.extend_from_slice(snapshot_bytes);
            }
            MigrationMessage::RestoreComplete {
                daemon_origin,
                restored_seq,
            } => {
                buf.put_u8(MSG_RESTORE_COMPLETE);
                buf.put_u32_le(*daemon_origin);
                buf.put_u64_le(*restored_seq);
            }
            MigrationMessage::ReplayComplete {
                daemon_origin,
                replayed_seq,
            } => {
                buf.put_u8(MSG_REPLAY_COMPLETE);
                buf.put_u32_le(*daemon_origin);
                buf.put_u64_le(*replayed_seq);
            }
            MigrationMessage::CutoverNotify {
                daemon_origin,
                target_node,
            } => {
                buf.put_u8(MSG_CUTOVER_NOTIFY);
                buf.put_u32_le(*daemon_origin);
                buf.put_u64_le(*target_node);
            }
            MigrationMessage::CleanupComplete { daemon_origin } => {
                buf.put_u8(MSG_CLEANUP_COMPLETE);
                buf.put_u32_le(*daemon_origin);
            }
            MigrationMessage::MigrationFailed {
                daemon_origin,
                reason,
            } => {
                buf.put_u8(MSG_FAILED);
                buf.put_u32_le(*daemon_origin);
                buf.put_u16_le(reason.len() as u16);
                buf.extend_from_slice(reason.as_bytes());
            }
            MigrationMessage::BufferedEvents {
                daemon_origin,
                events,
            } => {
                buf.put_u8(MSG_BUFFERED_EVENTS);
                buf.put_u32_le(*daemon_origin);
                buf.put_u32_le(events.len() as u32);
                for event in events {
                    let link_bytes = event.link.to_bytes();
                    buf.extend_from_slice(&link_bytes);
                    buf.put_u32_le(event.payload.len() as u32);
                    buf.extend_from_slice(&event.payload);
                    buf.put_u64_le(event.received_at);
                }
            }
        }

        buf
    }

    /// Decode a migration message from bytes.
    pub fn decode(data: &[u8]) -> Result<MigrationMessage, MigrationError> {
        if data.is_empty() {
            return Err(MigrationError::StateFailed("empty message".into()));
        }

        let mut cur = std::io::Cursor::new(data);

        let msg_type = cur.get_u8();

        match msg_type {
            MSG_TAKE_SNAPSHOT => {
                if cur.remaining() < 12 {
                    return Err(MigrationError::StateFailed("truncated TakeSnapshot".into()));
                }
                Ok(MigrationMessage::TakeSnapshot {
                    daemon_origin: cur.get_u32_le(),
                    target_node: cur.get_u64_le(),
                })
            }
            MSG_SNAPSHOT_READY => {
                // daemon_origin(4) + seq_through(8) + chunk_index(2) + total_chunks(2) + len(4) = 20
                if cur.remaining() < 20 {
                    return Err(MigrationError::StateFailed(
                        "truncated SnapshotReady".into(),
                    ));
                }
                let daemon_origin = cur.get_u32_le();
                let seq_through = cur.get_u64_le();
                let chunk_index = cur.get_u16_le();
                let total_chunks = cur.get_u16_le();
                let len = cur.get_u32_le() as usize;
                if cur.remaining() < len {
                    return Err(MigrationError::StateFailed(
                        "truncated snapshot payload".into(),
                    ));
                }
                let mut snapshot_bytes = vec![0u8; len];
                cur.copy_to_slice(&mut snapshot_bytes);
                Ok(MigrationMessage::SnapshotReady {
                    daemon_origin,
                    snapshot_bytes,
                    seq_through,
                    chunk_index,
                    total_chunks,
                })
            }
            MSG_RESTORE_COMPLETE => {
                if cur.remaining() < 12 {
                    return Err(MigrationError::StateFailed(
                        "truncated RestoreComplete".into(),
                    ));
                }
                Ok(MigrationMessage::RestoreComplete {
                    daemon_origin: cur.get_u32_le(),
                    restored_seq: cur.get_u64_le(),
                })
            }
            MSG_REPLAY_COMPLETE => {
                if cur.remaining() < 12 {
                    return Err(MigrationError::StateFailed(
                        "truncated ReplayComplete".into(),
                    ));
                }
                Ok(MigrationMessage::ReplayComplete {
                    daemon_origin: cur.get_u32_le(),
                    replayed_seq: cur.get_u64_le(),
                })
            }
            MSG_CUTOVER_NOTIFY => {
                if cur.remaining() < 12 {
                    return Err(MigrationError::StateFailed(
                        "truncated CutoverNotify".into(),
                    ));
                }
                Ok(MigrationMessage::CutoverNotify {
                    daemon_origin: cur.get_u32_le(),
                    target_node: cur.get_u64_le(),
                })
            }
            MSG_CLEANUP_COMPLETE => {
                if cur.remaining() < 4 {
                    return Err(MigrationError::StateFailed(
                        "truncated CleanupComplete".into(),
                    ));
                }
                Ok(MigrationMessage::CleanupComplete {
                    daemon_origin: cur.get_u32_le(),
                })
            }
            MSG_FAILED => {
                if cur.remaining() < 6 {
                    return Err(MigrationError::StateFailed(
                        "truncated MigrationFailed".into(),
                    ));
                }
                let daemon_origin = cur.get_u32_le();
                let reason_len = cur.get_u16_le() as usize;
                if cur.remaining() < reason_len {
                    return Err(MigrationError::StateFailed("truncated reason".into()));
                }
                let mut reason_bytes = vec![0u8; reason_len];
                cur.copy_to_slice(&mut reason_bytes);
                let reason =
                    String::from_utf8(reason_bytes).unwrap_or_else(|_| "invalid utf8".into());
                Ok(MigrationMessage::MigrationFailed {
                    daemon_origin,
                    reason,
                })
            }
            MSG_BUFFERED_EVENTS => {
                if cur.remaining() < 8 {
                    return Err(MigrationError::StateFailed(
                        "truncated BufferedEvents".into(),
                    ));
                }
                let daemon_origin = cur.get_u32_le();
                let count = cur.get_u32_le() as usize;
                let mut events = Vec::with_capacity(count);
                for _ in 0..count {
                    use crate::adapter::net::state::causal::CAUSAL_LINK_SIZE;
                    if cur.remaining() < CAUSAL_LINK_SIZE + 4 {
                        return Err(MigrationError::StateFailed(
                            "truncated buffered event".into(),
                        ));
                    }
                    let mut link_bytes = [0u8; CAUSAL_LINK_SIZE];
                    cur.copy_to_slice(&mut link_bytes);
                    let link = CausalLink::from_bytes(&link_bytes)
                        .ok_or_else(|| MigrationError::StateFailed("invalid causal link".into()))?;
                    let payload_len = cur.get_u32_le() as usize;
                    if cur.remaining() < payload_len + 8 {
                        return Err(MigrationError::StateFailed(
                            "truncated event payload".into(),
                        ));
                    }
                    let mut payload = vec![0u8; payload_len];
                    cur.copy_to_slice(&mut payload);
                    let received_at = cur.get_u64_le();
                    events.push(CausalEvent {
                        link,
                        payload: bytes::Bytes::from(payload),
                        received_at,
                    });
                }
                Ok(MigrationMessage::BufferedEvents {
                    daemon_origin,
                    events,
                })
            }
            _ => Err(MigrationError::StateFailed(format!(
                "unknown message type: {}",
                msg_type
            ))),
        }
    }
}

// ── Snapshot chunking ────────────────────────────────────────────────────────

/// Maximum snapshot chunk size. Sized to fit within `MAX_PAYLOAD_SIZE` after
/// accounting for the SnapshotReady wire header overhead
/// (msg_type + daemon_origin + seq_through + chunk_index + total_chunks + len = 21 bytes)
/// and leaving headroom for the outer transport framing.
pub const MAX_SNAPSHOT_CHUNK_SIZE: usize = 7000;

/// Split a snapshot into chunked `SnapshotReady` messages.
///
/// Small snapshots (<= `MAX_SNAPSHOT_CHUNK_SIZE`) produce a single message
/// with `chunk_index = 0, total_chunks = 1`. Larger snapshots are split
/// into multiple messages that the receiver must reassemble.
pub fn chunk_snapshot(
    daemon_origin: u32,
    snapshot_bytes: Vec<u8>,
    seq_through: u64,
) -> Vec<MigrationMessage> {
    if snapshot_bytes.len() <= MAX_SNAPSHOT_CHUNK_SIZE {
        return vec![MigrationMessage::SnapshotReady {
            daemon_origin,
            snapshot_bytes,
            seq_through,
            chunk_index: 0,
            total_chunks: 1,
        }];
    }

    let total_chunks = snapshot_bytes.len().div_ceil(MAX_SNAPSHOT_CHUNK_SIZE);
    let total_chunks = total_chunks as u16;

    snapshot_bytes
        .chunks(MAX_SNAPSHOT_CHUNK_SIZE)
        .enumerate()
        .map(|(i, chunk)| MigrationMessage::SnapshotReady {
            daemon_origin,
            snapshot_bytes: chunk.to_vec(),
            seq_through,
            chunk_index: i as u16,
            total_chunks,
        })
        .collect()
}

/// Reassembles chunked `SnapshotReady` messages into a complete snapshot.
///
/// Collects chunks keyed by `(daemon_origin, seq_through)` and returns the
/// full snapshot bytes once all chunks have arrived.
pub struct SnapshotReassembler {
    /// Pending reassemblies: daemon_origin → chunks.
    pending: std::collections::HashMap<u32, ReassemblyState>,
}

#[allow(dead_code)]
struct ReassemblyState {
    total_chunks: u16,
    seq_through: u64,
    chunks: std::collections::BTreeMap<u16, Vec<u8>>,
}

impl SnapshotReassembler {
    /// Create a new reassembler.
    pub fn new() -> Self {
        Self {
            pending: std::collections::HashMap::new(),
        }
    }

    /// Feed a snapshot chunk. Returns the complete snapshot bytes when all
    /// chunks have been received, or `None` if still waiting for more.
    pub fn feed(
        &mut self,
        daemon_origin: u32,
        snapshot_bytes: Vec<u8>,
        seq_through: u64,
        chunk_index: u16,
        total_chunks: u16,
    ) -> Option<Vec<u8>> {
        // Single-chunk fast path
        if total_chunks == 1 {
            self.pending.remove(&daemon_origin);
            return Some(snapshot_bytes);
        }

        let state = self.pending.entry(daemon_origin).or_insert_with(|| {
            ReassemblyState {
                total_chunks,
                seq_through,
                chunks: std::collections::BTreeMap::new(),
            }
        });

        state.chunks.insert(chunk_index, snapshot_bytes);

        if state.chunks.len() == state.total_chunks as usize {
            // All chunks received — reassemble in order
            let state = self.pending.remove(&daemon_origin).unwrap();
            let mut full = Vec::new();
            for (_idx, chunk) in state.chunks {
                full.extend_from_slice(&chunk);
            }
            Some(full)
        } else {
            None
        }
    }

    /// Cancel reassembly for a daemon (e.g., on migration abort).
    pub fn cancel(&mut self, daemon_origin: u32) {
        self.pending.remove(&daemon_origin);
    }

    /// Number of pending reassemblies.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for SnapshotReassembler {
    fn default() -> Self {
        Self::new()
    }
}

// ── Orchestrator ─────────────────────────────────────────────────────────────

/// Tracks an in-flight migration with its superposition state.
struct MigrationRecord {
    state: MigrationState,
    superposition: SuperpositionState,
    started_at: Instant,
}

/// Coordinates all 6 phases of daemon migration.
///
/// The orchestrator manages the lifecycle of migrations: initiating snapshots,
/// forwarding snapshot data to targets, coordinating replay, executing cutover,
/// and cleaning up the source.
pub struct MigrationOrchestrator {
    /// In-flight migrations: daemon_origin → record.
    migrations: DashMap<u32, Mutex<MigrationRecord>>,
    /// Local daemon registry (for taking snapshots on local daemons).
    daemon_registry: Arc<DaemonRegistry>,
    /// Local node ID.
    local_node_id: u64,
}

impl MigrationOrchestrator {
    /// Create a new orchestrator.
    pub fn new(daemon_registry: Arc<DaemonRegistry>, local_node_id: u64) -> Self {
        Self {
            migrations: DashMap::new(),
            daemon_registry,
            local_node_id,
        }
    }

    /// Initiate a migration (phase 0: Snapshot).
    ///
    /// If the source is the local node, takes the snapshot immediately and
    /// returns `SnapshotReady`. Otherwise, returns `TakeSnapshot` for the
    /// caller to send to the source node.
    pub fn start_migration(
        &self,
        daemon_origin: u32,
        source_node: u64,
        target_node: u64,
    ) -> Result<MigrationMessage, MigrationError> {
        // Reject if already migrating
        if self.migrations.contains_key(&daemon_origin) {
            return Err(MigrationError::AlreadyMigrating(daemon_origin));
        }

        let mut state = MigrationState::new(daemon_origin, source_node, target_node);

        // If we are the source, take snapshot locally
        if source_node == self.local_node_id {
            let snapshot = self
                .daemon_registry
                .snapshot(daemon_origin)
                .map_err(|e| MigrationError::StateFailed(e.to_string()))?
                .ok_or_else(|| {
                    MigrationError::StateFailed("daemon is stateless or snapshot failed".into())
                })?;

            let snapshot_bytes = snapshot.to_bytes();
            let seq_through = snapshot.through_seq;

            state.set_snapshot(snapshot)?;

            let source_head = state
                .snapshot()
                .map(|s| s.chain_link)
                .unwrap_or_else(|| CausalLink::genesis(daemon_origin, 0));

            let superposition = SuperpositionState::new(daemon_origin, source_head);

            self.migrations.insert(
                daemon_origin,
                Mutex::new(MigrationRecord {
                    state,
                    superposition,
                    started_at: Instant::now(),
                }),
            );

            Ok(MigrationMessage::SnapshotReady {
                daemon_origin,
                snapshot_bytes,
                seq_through,
                chunk_index: 0,
                total_chunks: 1,
            })
        } else {
            let source_head = CausalLink::genesis(daemon_origin, 0);
            let superposition = SuperpositionState::new(daemon_origin, source_head);

            self.migrations.insert(
                daemon_origin,
                Mutex::new(MigrationRecord {
                    state,
                    superposition,
                    started_at: Instant::now(),
                }),
            );

            Ok(MigrationMessage::TakeSnapshot {
                daemon_origin,
                target_node,
            })
        }
    }

    /// Initiate a migration with automatic target selection.
    ///
    /// Uses the scheduler to find the best migration-capable target node
    /// based on the daemon's capability requirements. The scheduler queries
    /// the `CapabilityIndex` for nodes advertising `subprotocol:0x0500`.
    ///
    /// Returns the target node ID and the first migration message.
    pub fn start_migration_auto(
        &self,
        daemon_origin: u32,
        source_node: u64,
        scheduler: &super::Scheduler,
        daemon_filter: &crate::adapter::net::behavior::capability::CapabilityFilter,
    ) -> Result<(u64, MigrationMessage), MigrationError> {
        let placement = scheduler
            .place_migration(daemon_filter, source_node)
            .map_err(|_| MigrationError::TargetUnavailable(0))?;

        let target_node = placement.node_id;
        let msg = self.start_migration(daemon_origin, source_node, target_node)?;
        Ok((target_node, msg))
    }

    /// Handle snapshot taken on source (phase 1→2).
    ///
    /// Validates and stores the snapshot, advances to Transfer phase.
    /// Returns the message to forward to the target node. For chunked
    /// snapshots, only the first chunk (index 0) triggers validation
    /// and phase advancement — subsequent chunks are forwarded as-is.
    pub fn on_snapshot_ready(
        &self,
        daemon_origin: u32,
        snapshot_bytes: Vec<u8>,
        seq_through: u64,
        chunk_index: u16,
        total_chunks: u16,
    ) -> Result<MigrationMessage, MigrationError> {
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        let mut record = entry.lock();

        // Only validate and advance phase on the first chunk
        if chunk_index == 0 && total_chunks == 1 {
            let snapshot = StateSnapshot::from_bytes(&snapshot_bytes)
                .ok_or_else(|| {
                    MigrationError::StateFailed("failed to parse snapshot bytes".into())
                })?;

            if record.state.phase() == MigrationPhase::Snapshot {
                record.state.set_snapshot(snapshot)?;
            }
        } else if chunk_index == 0 {
            // First chunk of a multi-chunk snapshot — advance phase
            // but can't validate until all chunks arrive
            if record.state.phase() == MigrationPhase::Snapshot {
                // We don't have the full snapshot yet, just advance phase
                // The target will reassemble and validate
            }
        }

        // Update superposition on first chunk
        if chunk_index == 0 {
            record.superposition.advance(MigrationPhase::Transfer);
        }

        // Forward to target
        Ok(MigrationMessage::SnapshotReady {
            daemon_origin,
            snapshot_bytes,
            seq_through,
            chunk_index,
            total_chunks,
        })
    }

    /// Handle restore complete on target (phase 2→3).
    ///
    /// Advances to Replay phase. Returns buffered events message if there
    /// are any, or None if no events were buffered.
    pub fn on_restore_complete(
        &self,
        daemon_origin: u32,
        _restored_seq: u64,
    ) -> Result<Option<MigrationMessage>, MigrationError> {
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        let mut record = entry.lock();

        // Advance: Transfer → Restore → Replay
        if record.state.phase() == MigrationPhase::Transfer {
            record.state.transfer_complete()?;
        }
        if record.state.phase() == MigrationPhase::Restore {
            record.state.restore_complete()?;
        }

        record.superposition.advance(MigrationPhase::Replay);

        // Drain buffered events for replay
        let events = record.state.take_buffered_events();
        if events.is_empty() {
            Ok(None)
        } else {
            Ok(Some(MigrationMessage::BufferedEvents {
                daemon_origin,
                events,
            }))
        }
    }

    /// Handle replay complete on target (phase 3→4).
    ///
    /// Returns `CutoverNotify` to send to the source node.
    pub fn on_replay_complete(
        &self,
        daemon_origin: u32,
        replayed_seq: u64,
    ) -> Result<MigrationMessage, MigrationError> {
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        let mut record = entry.lock();

        // Update target head in superposition
        let target_head = CausalLink {
            origin_hash: daemon_origin,
            horizon_encoded: 0,
            sequence: replayed_seq,
            parent_hash: 0, // we don't have the exact hash here
        };
        record.superposition.target_replayed(target_head);

        // Advance to Cutover
        if record.state.phase() == MigrationPhase::Replay {
            record.state.replay_complete()?;
        }

        record.superposition.advance(MigrationPhase::Cutover);
        record.superposition.collapse();

        let target_node = record.state.target_node();

        Ok(MigrationMessage::CutoverNotify {
            daemon_origin,
            target_node,
        })
    }

    /// Handle cutover acknowledged by source.
    ///
    /// Source has stopped accepting writes. Advances to Complete.
    pub fn on_cutover_acknowledged(&self, daemon_origin: u32) -> Result<(), MigrationError> {
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        let mut record = entry.lock();

        if record.state.phase() == MigrationPhase::Cutover {
            record.state.cutover_complete()?;
        }

        record.superposition.advance(MigrationPhase::Complete);
        record.superposition.resolve();

        Ok(())
    }

    /// Handle cleanup complete from source.
    ///
    /// Removes the migration record entirely.
    pub fn on_cleanup_complete(&self, daemon_origin: u32) -> Result<(), MigrationError> {
        self.migrations
            .remove(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;
        Ok(())
    }

    /// Buffer an event for a daemon that is currently migrating.
    ///
    /// Returns `true` if the event was buffered (migration in progress),
    /// `false` if no migration is active for this daemon.
    pub fn buffer_event(&self, daemon_origin: u32, event: CausalEvent) -> bool {
        if let Some(entry) = self.migrations.get(&daemon_origin) {
            let mut record = entry.lock();
            let phase = record.state.phase();
            // Buffer during Snapshot through Replay phases
            if phase != MigrationPhase::Cutover && phase != MigrationPhase::Complete {
                record.state.buffer_event(event);
                return true;
            }
        }
        false
    }

    /// Abort a migration at any phase.
    ///
    /// Returns the abort message to broadcast to involved nodes.
    pub fn abort_migration(
        &self,
        daemon_origin: u32,
        reason: String,
    ) -> Result<MigrationMessage, MigrationError> {
        self.migrations
            .remove(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        Ok(MigrationMessage::MigrationFailed {
            daemon_origin,
            reason,
        })
    }

    /// Check if a daemon is currently being migrated.
    pub fn is_migrating(&self, daemon_origin: u32) -> bool {
        self.migrations.contains_key(&daemon_origin)
    }

    /// Get migration status for a daemon.
    pub fn status(&self, daemon_origin: u32) -> Option<MigrationPhase> {
        self.migrations
            .get(&daemon_origin)
            .map(|entry| entry.lock().state.phase())
    }

    /// Get the target node for an in-flight migration.
    pub fn target_node(&self, daemon_origin: u32) -> Option<u64> {
        self.migrations
            .get(&daemon_origin)
            .map(|entry| entry.lock().state.target_node())
    }

    /// Get superposition phase for a daemon.
    pub fn superposition_phase(
        &self,
        daemon_origin: u32,
    ) -> Option<crate::adapter::net::continuity::superposition::SuperpositionPhase> {
        self.migrations
            .get(&daemon_origin)
            .map(|entry| entry.lock().superposition.phase())
    }

    /// List all in-flight migrations: (daemon_origin, phase, elapsed_ms).
    pub fn list_migrations(&self) -> Vec<(u32, MigrationPhase, u64)> {
        self.migrations
            .iter()
            .map(|entry| {
                let record = entry.lock();
                let elapsed = record.started_at.elapsed().as_millis() as u64;
                (*entry.key(), record.state.phase(), elapsed)
            })
            .collect()
    }

    /// Number of active migrations.
    pub fn active_count(&self) -> usize {
        self.migrations.len()
    }
}

impl std::fmt::Debug for MigrationOrchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationOrchestrator")
            .field("active_migrations", &self.migrations.len())
            .field("local_node_id", &format!("{:#x}", self.local_node_id))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::behavior::capability::CapabilityFilter;
    use crate::adapter::net::compute::{
        DaemonError, DaemonHost, DaemonHostConfig, DaemonRegistry, MeshDaemon,
    };
    use crate::adapter::net::identity::EntityKeypair;
    use bytes::Bytes;

    struct CounterDaemon {
        count: u64,
    }

    impl CounterDaemon {
        fn new() -> Self {
            Self { count: 0 }
        }
    }

    impl MeshDaemon for CounterDaemon {
        fn name(&self) -> &str {
            "counter"
        }
        fn requirements(&self) -> CapabilityFilter {
            CapabilityFilter::default()
        }
        fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
            self.count += 1;
            Ok(vec![Bytes::from(self.count.to_le_bytes().to_vec())])
        }
        fn snapshot(&self) -> Option<Bytes> {
            Some(Bytes::from(self.count.to_le_bytes().to_vec()))
        }
        fn restore(&mut self, state: Bytes) -> Result<(), DaemonError> {
            if state.len() != 8 {
                return Err(DaemonError::RestoreFailed("bad state size".into()));
            }
            self.count = u64::from_le_bytes(state[..8].try_into().unwrap());
            Ok(())
        }
    }

    fn setup_registry() -> (Arc<DaemonRegistry>, u32) {
        let reg = Arc::new(DaemonRegistry::new());
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();
        let host = DaemonHost::new(
            Box::new(CounterDaemon::new()),
            kp,
            DaemonHostConfig::default(),
        );
        reg.register(host).unwrap();
        (reg, origin)
    }

    #[test]
    fn test_start_migration_local_source() {
        let (reg, origin) = setup_registry();
        let orch = MigrationOrchestrator::new(reg, 0x1111);

        let msg = orch.start_migration(origin, 0x1111, 0x2222).unwrap();
        match msg {
            MigrationMessage::SnapshotReady { daemon_origin, .. } => {
                assert_eq!(daemon_origin, origin);
            }
            _ => panic!("expected SnapshotReady"),
        }

        assert!(orch.is_migrating(origin));
        assert_eq!(orch.status(origin), Some(MigrationPhase::Transfer));
    }

    #[test]
    fn test_start_migration_remote_source() {
        let (reg, origin) = setup_registry();
        let orch = MigrationOrchestrator::new(reg, 0x3333);

        let msg = orch.start_migration(origin, 0x1111, 0x2222).unwrap();
        match msg {
            MigrationMessage::TakeSnapshot {
                daemon_origin,
                target_node,
            } => {
                assert_eq!(daemon_origin, origin);
                assert_eq!(target_node, 0x2222);
            }
            _ => panic!("expected TakeSnapshot"),
        }

        assert_eq!(orch.status(origin), Some(MigrationPhase::Snapshot));
    }

    #[test]
    fn test_duplicate_migration_rejected() {
        let (reg, origin) = setup_registry();
        let orch = MigrationOrchestrator::new(reg, 0x1111);

        orch.start_migration(origin, 0x1111, 0x2222).unwrap();
        let err = orch.start_migration(origin, 0x1111, 0x3333).unwrap_err();
        assert_eq!(err, MigrationError::AlreadyMigrating(origin));
    }

    #[test]
    fn test_abort_migration() {
        let (reg, origin) = setup_registry();
        let orch = MigrationOrchestrator::new(reg, 0x1111);

        orch.start_migration(origin, 0x1111, 0x2222).unwrap();
        assert!(orch.is_migrating(origin));

        let msg = orch.abort_migration(origin, "test abort".into()).unwrap();
        match msg {
            MigrationMessage::MigrationFailed { reason, .. } => {
                assert_eq!(reason, "test abort");
            }
            _ => panic!("expected MigrationFailed"),
        }

        assert!(!orch.is_migrating(origin));
    }

    #[test]
    fn test_event_buffering() {
        let (reg, origin) = setup_registry();
        let orch = MigrationOrchestrator::new(reg, 0x3333);

        orch.start_migration(origin, 0x1111, 0x2222).unwrap();

        let event = CausalEvent {
            link: CausalLink::genesis(origin, 0),
            payload: Bytes::from_static(b"test"),
            received_at: 0,
        };

        assert!(orch.buffer_event(origin, event));
        assert!(!orch.buffer_event(
            0xDEAD,
            CausalEvent {
                link: CausalLink::genesis(0xDEAD, 0),
                payload: Bytes::from_static(b"nope"),
                received_at: 0,
            }
        ));
    }

    #[test]
    fn test_wire_roundtrip_take_snapshot() {
        let msg = MigrationMessage::TakeSnapshot {
            daemon_origin: 0xAAAA,
            target_node: 0x2222,
        };
        let encoded = wire::encode(&msg);
        let decoded = wire::decode(&encoded).unwrap();
        match decoded {
            MigrationMessage::TakeSnapshot {
                daemon_origin,
                target_node,
            } => {
                assert_eq!(daemon_origin, 0xAAAA);
                assert_eq!(target_node, 0x2222);
            }
            _ => panic!("expected TakeSnapshot"),
        }
    }

    #[test]
    fn test_wire_roundtrip_snapshot_ready() {
        let msg = MigrationMessage::SnapshotReady {
            daemon_origin: 0xBBBB,
            snapshot_bytes: vec![1, 2, 3, 4, 5],
            seq_through: 42,
            chunk_index: 0,
            total_chunks: 1,
        };
        let encoded = wire::encode(&msg);
        let decoded = wire::decode(&encoded).unwrap();
        match decoded {
            MigrationMessage::SnapshotReady {
                daemon_origin,
                snapshot_bytes,
                seq_through,
                chunk_index,
                total_chunks,
            } => {
                assert_eq!(daemon_origin, 0xBBBB);
                assert_eq!(snapshot_bytes, vec![1, 2, 3, 4, 5]);
                assert_eq!(seq_through, 42);
                assert_eq!(chunk_index, 0);
                assert_eq!(total_chunks, 1);
            }
            _ => panic!("expected SnapshotReady"),
        }
    }

    #[test]
    fn test_chunk_snapshot_small() {
        let chunks = chunk_snapshot(0xAAAA, vec![1, 2, 3], 10);
        assert_eq!(chunks.len(), 1);
        match &chunks[0] {
            MigrationMessage::SnapshotReady {
                chunk_index,
                total_chunks,
                snapshot_bytes,
                ..
            } => {
                assert_eq!(*chunk_index, 0);
                assert_eq!(*total_chunks, 1);
                assert_eq!(snapshot_bytes, &vec![1, 2, 3]);
            }
            _ => panic!("expected SnapshotReady"),
        }
    }

    #[test]
    fn test_chunk_snapshot_large() {
        // Create a snapshot larger than MAX_SNAPSHOT_CHUNK_SIZE
        let big = vec![0xABu8; MAX_SNAPSHOT_CHUNK_SIZE * 3 + 100];
        let total_len = big.len();
        let chunks = chunk_snapshot(0xBBBB, big, 42);

        assert_eq!(chunks.len(), 4); // 3 full + 1 partial

        // Verify chunk metadata
        for (i, chunk) in chunks.iter().enumerate() {
            match chunk {
                MigrationMessage::SnapshotReady {
                    chunk_index,
                    total_chunks,
                    daemon_origin,
                    seq_through,
                    ..
                } => {
                    assert_eq!(*chunk_index, i as u16);
                    assert_eq!(*total_chunks, 4);
                    assert_eq!(*daemon_origin, 0xBBBB);
                    assert_eq!(*seq_through, 42);
                }
                _ => panic!("expected SnapshotReady"),
            }
        }

        // Verify reassembly
        let mut reassembler = SnapshotReassembler::new();
        for chunk in chunks {
            if let MigrationMessage::SnapshotReady {
                daemon_origin,
                snapshot_bytes,
                seq_through,
                chunk_index,
                total_chunks,
            } = chunk
            {
                let result = reassembler.feed(
                    daemon_origin,
                    snapshot_bytes,
                    seq_through,
                    chunk_index,
                    total_chunks,
                );
                if chunk_index < total_chunks - 1 {
                    assert!(result.is_none());
                } else {
                    let full = result.expect("last chunk should complete reassembly");
                    assert_eq!(full.len(), total_len);
                    assert!(full.iter().all(|&b| b == 0xAB));
                }
            }
        }
    }

    #[test]
    fn test_reassembler_cancel() {
        let mut reassembler = SnapshotReassembler::new();
        reassembler.feed(0xAAAA, vec![1, 2], 10, 0, 3);
        assert_eq!(reassembler.pending_count(), 1);
        reassembler.cancel(0xAAAA);
        assert_eq!(reassembler.pending_count(), 0);
    }

    #[test]
    fn test_wire_roundtrip_chunked_snapshot() {
        let msg = MigrationMessage::SnapshotReady {
            daemon_origin: 0xCCCC,
            snapshot_bytes: vec![42; 100],
            seq_through: 99,
            chunk_index: 2,
            total_chunks: 5,
        };
        let encoded = wire::encode(&msg);
        let decoded = wire::decode(&encoded).unwrap();
        match decoded {
            MigrationMessage::SnapshotReady {
                chunk_index,
                total_chunks,
                ..
            } => {
                assert_eq!(chunk_index, 2);
                assert_eq!(total_chunks, 5);
            }
            _ => panic!("expected SnapshotReady"),
        }
    }

    #[test]
    fn test_wire_roundtrip_failed() {
        let msg = MigrationMessage::MigrationFailed {
            daemon_origin: 0xCCCC,
            reason: "something broke".into(),
        };
        let encoded = wire::encode(&msg);
        let decoded = wire::decode(&encoded).unwrap();
        match decoded {
            MigrationMessage::MigrationFailed {
                daemon_origin,
                reason,
            } => {
                assert_eq!(daemon_origin, 0xCCCC);
                assert_eq!(reason, "something broke");
            }
            _ => panic!("expected MigrationFailed"),
        }
    }

    #[test]
    fn test_wire_roundtrip_buffered_events() {
        let events = vec![
            CausalEvent {
                link: CausalLink::genesis(0xAAAA, 0),
                payload: Bytes::from_static(b"event1"),
                received_at: 100,
            },
            CausalEvent {
                link: CausalLink {
                    origin_hash: 0xAAAA,
                    horizon_encoded: 1,
                    sequence: 1,
                    parent_hash: 12345,
                },
                payload: Bytes::from_static(b"event2"),
                received_at: 200,
            },
        ];
        let msg = MigrationMessage::BufferedEvents {
            daemon_origin: 0xAAAA,
            events,
        };
        let encoded = wire::encode(&msg);
        let decoded = wire::decode(&encoded).unwrap();
        match decoded {
            MigrationMessage::BufferedEvents {
                daemon_origin,
                events,
            } => {
                assert_eq!(daemon_origin, 0xAAAA);
                assert_eq!(events.len(), 2);
                assert_eq!(events[0].payload, Bytes::from_static(b"event1"));
                assert_eq!(events[0].received_at, 100);
                assert_eq!(events[1].link.sequence, 1);
                assert_eq!(events[1].link.parent_hash, 12345);
                assert_eq!(events[1].payload, Bytes::from_static(b"event2"));
                assert_eq!(events[1].received_at, 200);
            }
            _ => panic!("expected BufferedEvents"),
        }
    }

    #[test]
    fn test_list_migrations() {
        let (reg, origin) = setup_registry();
        let orch = MigrationOrchestrator::new(reg, 0x1111);

        assert!(orch.list_migrations().is_empty());

        orch.start_migration(origin, 0x1111, 0x2222).unwrap();

        let list = orch.list_migrations();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, origin);
    }
}
