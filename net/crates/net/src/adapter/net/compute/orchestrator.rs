//! Migration orchestrator — coordinates all 6 phases of daemon migration.
//!
//! The orchestrator runs on the controller node (which may be the source,
//! target, or a third-party coordinator). It sequences phase transitions,
//! routes migration messages, and handles failures/timeouts.

use std::sync::Arc;
use std::time::Instant;

use bytes::Buf;
use dashmap::DashMap;
use parking_lot::Mutex;

use super::migration::{MigrationError, MigrationFailureReason, MigrationPhase, MigrationState};
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
        chunk_index: u32,
        /// Total number of chunks for this snapshot.
        total_chunks: u32,
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
        /// Structured reason code — source dispatches on this to
        /// decide whether the migration is retriable. See
        /// [`MigrationFailureReason`].
        reason: MigrationFailureReason,
    },

    /// Buffered events from source for replay on target.
    BufferedEvents {
        /// Origin hash of daemon being migrated.
        daemon_origin: u32,
        /// Events to replay, in causal order.
        events: Vec<CausalEvent>,
    },

    /// Phase 5→6: Source has cleaned up; target should now go live.
    ///
    /// Emitted by the orchestrator once it observes `CleanupComplete`. The
    /// target calls `MigrationTargetHandler::activate` in response and
    /// replies with `ActivateAck`.
    ActivateTarget {
        /// Origin hash of daemon whose target should activate.
        daemon_origin: u32,
    },

    /// Phase 6: Target has activated and is now authoritative.
    ActivateAck {
        /// Origin hash of daemon whose migration is complete.
        daemon_origin: u32,
        /// Sequence number the target is authoritative through.
        replayed_seq: u64,
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
    /// Wire type: orchestrator tells target to activate.
    pub const MSG_ACTIVATE_TARGET: u8 = 8;
    /// Wire type: target acknowledges activation.
    pub const MSG_ACTIVATE_ACK: u8 = 9;

    /// Encode a migration message to bytes.
    ///
    /// Returns `MigrationError::StateFailed` when a length-prefixed field
    /// would not fit in its on-wire width. Length prefixes are `u32` for
    /// payloads and counts and `u16` for the failure reason string; silently
    /// truncating to fit would corrupt the stream and confuse the decoder.
    pub fn encode(msg: &MigrationMessage) -> Result<Vec<u8>, MigrationError> {
        // Helper: convert a usize length to u32 with an error on overflow.
        fn len_u32(field: &str, n: usize) -> Result<u32, MigrationError> {
            u32::try_from(n).map_err(|_| {
                MigrationError::StateFailed(format!("{} length {} exceeds u32::MAX", field, n))
            })
        }

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
                let payload_len = len_u32("snapshot_bytes", snapshot_bytes.len())?;
                buf.put_u8(MSG_SNAPSHOT_READY);
                buf.put_u32_le(*daemon_origin);
                buf.put_u64_le(*seq_through);
                buf.put_u32_le(*chunk_index);
                buf.put_u32_le(*total_chunks);
                buf.put_u32_le(payload_len);
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
                // Wire layout:
                //   code:  u16 le
                //   then variant-specific payload (0 bytes for
                //   zero-payload variants; `u16 le + bytes` for
                //   string-bearing variants; `u8` for NotReadyTimeout).
                buf.put_u16_le(reason.code());
                match reason {
                    MigrationFailureReason::NotReady
                    | MigrationFailureReason::FactoryNotFound
                    | MigrationFailureReason::ComputeNotSupported
                    | MigrationFailureReason::AlreadyMigrating => {}
                    MigrationFailureReason::StateFailed(msg)
                    | MigrationFailureReason::IdentityTransportFailed(msg) => {
                        let len = u16::try_from(msg.len()).map_err(|_| {
                            MigrationError::StateFailed(format!(
                                "failure reason message length {} exceeds u16::MAX",
                                msg.len()
                            ))
                        })?;
                        buf.put_u16_le(len);
                        buf.extend_from_slice(msg.as_bytes());
                    }
                    MigrationFailureReason::NotReadyTimeout { attempts } => {
                        buf.put_u8(*attempts);
                    }
                }
            }
            MigrationMessage::BufferedEvents {
                daemon_origin,
                events,
            } => {
                let event_count = len_u32("events", events.len())?;
                buf.put_u8(MSG_BUFFERED_EVENTS);
                buf.put_u32_le(*daemon_origin);
                buf.put_u32_le(event_count);
                for event in events {
                    let payload_len = len_u32("event payload", event.payload.len())?;
                    let link_bytes = event.link.to_bytes();
                    buf.extend_from_slice(&link_bytes);
                    buf.put_u32_le(payload_len);
                    buf.extend_from_slice(&event.payload);
                    buf.put_u64_le(event.received_at);
                }
            }
            MigrationMessage::ActivateTarget { daemon_origin } => {
                buf.put_u8(MSG_ACTIVATE_TARGET);
                buf.put_u32_le(*daemon_origin);
            }
            MigrationMessage::ActivateAck {
                daemon_origin,
                replayed_seq,
            } => {
                buf.put_u8(MSG_ACTIVATE_ACK);
                buf.put_u32_le(*daemon_origin);
                buf.put_u64_le(*replayed_seq);
            }
        }

        Ok(buf)
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
                // daemon_origin(4) + seq_through(8) + chunk_index(4) + total_chunks(4) + len(4) = 24
                if cur.remaining() < 24 {
                    return Err(MigrationError::StateFailed(
                        "truncated SnapshotReady".into(),
                    ));
                }
                let daemon_origin = cur.get_u32_le();
                let seq_through = cur.get_u64_le();
                let chunk_index = cur.get_u32_le();
                let total_chunks = cur.get_u32_le();
                let len = cur.get_u32_le() as usize;
                // Reject structurally invalid chunks at the wire boundary so
                // malformed messages never even reach the reassembler. The
                // reassembler enforces the same invariants defensively.
                if total_chunks == 0 {
                    return Err(MigrationError::StateFailed(
                        "SnapshotReady: total_chunks must be >= 1".into(),
                    ));
                }
                if total_chunks > MAX_TOTAL_CHUNKS {
                    return Err(MigrationError::StateFailed(format!(
                        "SnapshotReady: total_chunks {} exceeds MAX_TOTAL_CHUNKS ({})",
                        total_chunks, MAX_TOTAL_CHUNKS
                    )));
                }
                if chunk_index >= total_chunks {
                    return Err(MigrationError::StateFailed(format!(
                        "SnapshotReady: chunk_index {} out of range for total_chunks {}",
                        chunk_index, total_chunks
                    )));
                }
                if len > MAX_SNAPSHOT_CHUNK_SIZE {
                    return Err(MigrationError::StateFailed(format!(
                        "SnapshotReady: chunk len {} exceeds MAX_SNAPSHOT_CHUNK_SIZE ({})",
                        len, MAX_SNAPSHOT_CHUNK_SIZE
                    )));
                }
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
                if cur.remaining() < 4 + 2 {
                    return Err(MigrationError::StateFailed(
                        "truncated MigrationFailed header".into(),
                    ));
                }
                let daemon_origin = cur.get_u32_le();
                let code = cur.get_u16_le();
                let reason = decode_failure_reason(&mut cur, code)?;
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
                // Bound `count` against the remaining wire bytes before
                // allocating. Each event on the wire is at least
                // CAUSAL_LINK_SIZE(24) + u32 payload_len(4) + u64
                // received_at(8) = 36 bytes (empty payload). Without this
                // check, a malformed packet could claim `count = u32::MAX`
                // and force the decoder to allocate ~4G Vec slots before
                // the per-event bound checks fire — a cheap DoS against
                // the migration subprotocol.
                use crate::adapter::net::state::causal::CAUSAL_LINK_SIZE;
                const MIN_EVENT_WIRE_SIZE: usize = CAUSAL_LINK_SIZE + 4 + 8;
                // Hard cap as defense-in-depth. Well above any realistic
                // buffered-event batch (the orchestrator sends one
                // per-daemon batch at restore-complete; millions of
                // events per daemon per migration is already pathological).
                const MAX_BUFFERED_EVENTS: usize = 1_000_000;
                let max_possible = cur.remaining() / MIN_EVENT_WIRE_SIZE;
                if count > max_possible || count > MAX_BUFFERED_EVENTS {
                    return Err(MigrationError::StateFailed(format!(
                        "BufferedEvents: count {} exceeds bound (remaining={}, \
                         min_event_size={}, max_possible={}, hard_cap={})",
                        count,
                        cur.remaining(),
                        MIN_EVENT_WIRE_SIZE,
                        max_possible,
                        MAX_BUFFERED_EVENTS,
                    )));
                }
                let mut events = Vec::with_capacity(count);
                for _ in 0..count {
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
            MSG_ACTIVATE_TARGET => {
                if cur.remaining() < 4 {
                    return Err(MigrationError::StateFailed(
                        "truncated ActivateTarget".into(),
                    ));
                }
                Ok(MigrationMessage::ActivateTarget {
                    daemon_origin: cur.get_u32_le(),
                })
            }
            MSG_ACTIVATE_ACK => {
                if cur.remaining() < 12 {
                    return Err(MigrationError::StateFailed("truncated ActivateAck".into()));
                }
                Ok(MigrationMessage::ActivateAck {
                    daemon_origin: cur.get_u32_le(),
                    replayed_seq: cur.get_u64_le(),
                })
            }
            _ => Err(MigrationError::StateFailed(format!(
                "unknown message type: {}",
                msg_type
            ))),
        }
    }
}

/// Decode a `MigrationFailureReason` from the `MSG_FAILED` variant
/// payload. The 16-bit tag already consumed by the caller selects
/// the variant; unknown tags are rejected so forward-compat is
/// explicit rather than silent-ignore.
fn decode_failure_reason(
    cur: &mut std::io::Cursor<&[u8]>,
    code: u16,
) -> Result<MigrationFailureReason, MigrationError> {
    match code {
        0 => Ok(MigrationFailureReason::NotReady),
        1 => Ok(MigrationFailureReason::FactoryNotFound),
        2 => Ok(MigrationFailureReason::ComputeNotSupported),
        3 => {
            let msg = read_u16_string(cur, "StateFailed message")?;
            Ok(MigrationFailureReason::StateFailed(msg))
        }
        4 => Ok(MigrationFailureReason::AlreadyMigrating),
        5 => {
            let msg = read_u16_string(cur, "IdentityTransportFailed message")?;
            Ok(MigrationFailureReason::IdentityTransportFailed(msg))
        }
        6 => {
            if cur.remaining() < 1 {
                return Err(MigrationError::StateFailed(
                    "truncated NotReadyTimeout attempts field".into(),
                ));
            }
            Ok(MigrationFailureReason::NotReadyTimeout {
                attempts: cur.get_u8(),
            })
        }
        other => Err(MigrationError::StateFailed(format!(
            "unknown MigrationFailureReason code {other}",
        ))),
    }
}

fn read_u16_string(cur: &mut std::io::Cursor<&[u8]>, ctx: &str) -> Result<String, MigrationError> {
    if cur.remaining() < 2 {
        return Err(MigrationError::StateFailed(format!(
            "truncated {ctx} length prefix",
        )));
    }
    let len = cur.get_u16_le() as usize;
    if cur.remaining() < len {
        return Err(MigrationError::StateFailed(format!("truncated {ctx} body")));
    }
    let mut bytes = vec![0u8; len];
    cur.copy_to_slice(&mut bytes);
    String::from_utf8(bytes)
        .map_err(|e| MigrationError::StateFailed(format!("{ctx} is not valid UTF-8: {e}")))
}

// ── Snapshot chunking ────────────────────────────────────────────────────────

/// Maximum snapshot chunk size. Sized to fit within `MAX_PAYLOAD_SIZE` after
/// accounting for the SnapshotReady wire header overhead
/// (msg_type + daemon_origin + seq_through + chunk_index + total_chunks + len = 25 bytes)
/// and leaving headroom for the outer transport framing.
pub const MAX_SNAPSHOT_CHUNK_SIZE: usize = 7000;

/// Maximum transferable snapshot size: `u32::MAX` chunks * 7,000 bytes per chunk.
///
/// This is ~28 TB — effectively unlimited for daemon state. The `StateSnapshot`
/// wire format itself caps at ~4 GB (`state_len: u32`), so in practice the
/// snapshot serialization limit is reached first.
pub const MAX_SNAPSHOT_SIZE: usize = u32::MAX as usize * MAX_SNAPSHOT_CHUNK_SIZE;

/// Maximum `total_chunks` the reassembler will accept per reassembly.
///
/// `StateSnapshot` wire format caps payload at ~4 GB (`state_len: u32`), so
/// `ceil(u32::MAX / MAX_SNAPSHOT_CHUNK_SIZE)` ≈ 613,566 chunks is the largest
/// legitimate value. We cap above that with headroom; anything higher is an
/// attacker declaring a fake `total_chunks` to either flood us with
/// BTreeMap insertions or stall the reassembler forever waiting for chunks
/// that will never arrive.
pub const MAX_TOTAL_CHUNKS: u32 = 700_000;

/// Split a snapshot into chunked `SnapshotReady` messages.
///
/// Small snapshots (<= `MAX_SNAPSHOT_CHUNK_SIZE`) produce a single message
/// with `chunk_index = 0, total_chunks = 1`. Larger snapshots are split
/// into multiple messages that the receiver must reassemble.
///
/// Returns `MigrationError::SnapshotTooLarge` if the snapshot exceeds
/// `MAX_SNAPSHOT_SIZE` (~28 TB).
pub fn chunk_snapshot(
    daemon_origin: u32,
    snapshot_bytes: Vec<u8>,
    seq_through: u64,
) -> Result<Vec<MigrationMessage>, MigrationError> {
    if snapshot_bytes.len() <= MAX_SNAPSHOT_CHUNK_SIZE {
        return Ok(vec![MigrationMessage::SnapshotReady {
            daemon_origin,
            snapshot_bytes,
            seq_through,
            chunk_index: 0,
            total_chunks: 1,
        }]);
    }

    let total_chunks = snapshot_bytes.len().div_ceil(MAX_SNAPSHOT_CHUNK_SIZE);
    let total_chunks =
        u32::try_from(total_chunks).map_err(|_| MigrationError::SnapshotTooLarge {
            size: snapshot_bytes.len(),
            max: MAX_SNAPSHOT_SIZE,
        })?;

    Ok(snapshot_bytes
        .chunks(MAX_SNAPSHOT_CHUNK_SIZE)
        .enumerate()
        .map(|(i, chunk)| MigrationMessage::SnapshotReady {
            daemon_origin,
            snapshot_bytes: chunk.to_vec(),
            seq_through,
            chunk_index: i as u32,
            total_chunks,
        })
        .collect())
}

/// Why a chunk was rejected by [`SnapshotReassembler::feed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReassemblyError {
    /// `total_chunks == 0` — a well-formed message always declares at least one.
    ZeroTotalChunks,
    /// `chunk_index >= total_chunks` — attacker trying to smuggle out-of-range
    /// indices past the "all chunks received" count check.
    ChunkIndexOutOfRange {
        /// The chunk index declared by the peer.
        chunk_index: u32,
        /// The `total_chunks` declared by the peer.
        total_chunks: u32,
    },
    /// `total_chunks > MAX_TOTAL_CHUNKS` — peer declared more chunks than any
    /// legitimate snapshot could produce.
    TotalChunksTooLarge {
        /// The `total_chunks` declared by the peer.
        total_chunks: u32,
    },
    /// An individual chunk exceeds `MAX_SNAPSHOT_CHUNK_SIZE`.
    ChunkTooLarge {
        /// The chunk length observed.
        len: usize,
    },
    /// A later chunk declared a different `total_chunks` than the first chunk
    /// for the same `(daemon_origin, seq_through)`. Peer is either buggy or
    /// trying to resize an in-flight reassembly to force extra allocations.
    TotalChunksMismatch {
        /// The value declared by the current chunk.
        got: u32,
        /// The value locked in by the first chunk.
        expected: u32,
    },
    /// Peer sent a chunk for an older `seq_through` after we already
    /// accepted a newer one for the same daemon.
    StaleSeqThrough {
        /// The `seq_through` on the incoming chunk.
        got: u64,
        /// The newest `seq_through` we've accepted for this daemon.
        latest: u64,
    },
}

impl std::fmt::Display for ReassemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroTotalChunks => write!(f, "total_chunks == 0"),
            Self::ChunkIndexOutOfRange {
                chunk_index,
                total_chunks,
            } => write!(
                f,
                "chunk_index {} out of range for total_chunks {}",
                chunk_index, total_chunks
            ),
            Self::TotalChunksTooLarge { total_chunks } => write!(
                f,
                "total_chunks {} exceeds MAX_TOTAL_CHUNKS ({})",
                total_chunks, MAX_TOTAL_CHUNKS
            ),
            Self::ChunkTooLarge { len } => write!(
                f,
                "chunk length {} exceeds MAX_SNAPSHOT_CHUNK_SIZE ({})",
                len, MAX_SNAPSHOT_CHUNK_SIZE
            ),
            Self::TotalChunksMismatch { got, expected } => write!(
                f,
                "total_chunks {} does not match first chunk's declared {}",
                got, expected
            ),
            Self::StaleSeqThrough { got, latest } => write!(
                f,
                "seq_through {} is older than latest accepted {} for this daemon",
                got, latest
            ),
        }
    }
}

impl std::error::Error for ReassemblyError {}

/// Reassembles chunked `SnapshotReady` messages into a complete snapshot.
///
/// Keyed by `(daemon_origin, seq_through)` so chunks from different snapshot
/// generations cannot be mixed. At most one in-flight reassembly is kept
/// per daemon — a chunk for a newer `seq_through` evicts any older pending
/// state for that daemon, and chunks for older `seq_through` are rejected.
pub struct SnapshotReassembler {
    /// Pending reassemblies: (daemon_origin, seq_through) → chunks.
    pending: std::collections::HashMap<(u32, u64), ReassemblyState>,
    /// Latest `seq_through` accepted per daemon, for stale-chunk rejection
    /// even after a reassembly completes and is evicted from `pending`.
    latest_seq: std::collections::HashMap<u32, u64>,
}

struct ReassemblyState {
    total_chunks: u32,
    chunks: std::collections::BTreeMap<u32, Vec<u8>>,
}

impl SnapshotReassembler {
    /// Create a new reassembler.
    pub fn new() -> Self {
        Self {
            pending: std::collections::HashMap::new(),
            latest_seq: std::collections::HashMap::new(),
        }
    }

    /// Feed a snapshot chunk.
    ///
    /// Returns `Ok(Some(bytes))` when all chunks for the current
    /// `(daemon_origin, seq_through)` have been received, `Ok(None)` while
    /// still waiting, and `Err(ReassemblyError)` if the chunk is malformed or
    /// part of an attacker-shaped sequence. Rejected chunks never mutate
    /// in-flight state.
    pub fn feed(
        &mut self,
        daemon_origin: u32,
        snapshot_bytes: Vec<u8>,
        seq_through: u64,
        chunk_index: u32,
        total_chunks: u32,
    ) -> Result<Option<Vec<u8>>, ReassemblyError> {
        // ---- Per-chunk validation (no mutation until we've passed these) ----
        if total_chunks == 0 {
            return Err(ReassemblyError::ZeroTotalChunks);
        }
        if total_chunks > MAX_TOTAL_CHUNKS {
            return Err(ReassemblyError::TotalChunksTooLarge { total_chunks });
        }
        if chunk_index >= total_chunks {
            return Err(ReassemblyError::ChunkIndexOutOfRange {
                chunk_index,
                total_chunks,
            });
        }
        if snapshot_bytes.len() > MAX_SNAPSHOT_CHUNK_SIZE {
            return Err(ReassemblyError::ChunkTooLarge {
                len: snapshot_bytes.len(),
            });
        }
        if let Some(&latest) = self.latest_seq.get(&daemon_origin) {
            if seq_through < latest {
                return Err(ReassemblyError::StaleSeqThrough {
                    got: seq_through,
                    latest,
                });
            }
        }

        // A newer seq_through for the same daemon evicts older in-flight state.
        // This is what the public docstring always claimed; without it, the
        // `pending` map grew unbounded across seq_through values.
        if self
            .latest_seq
            .get(&daemon_origin)
            .is_none_or(|&latest| seq_through > latest)
        {
            self.pending
                .retain(|&(origin, seq), _| origin != daemon_origin || seq == seq_through);
            self.latest_seq.insert(daemon_origin, seq_through);
        }

        // Single-chunk fast path: no state to keep.
        if total_chunks == 1 {
            self.pending.remove(&(daemon_origin, seq_through));
            return Ok(Some(snapshot_bytes));
        }

        let key = (daemon_origin, seq_through);
        let state = self.pending.entry(key).or_insert_with(|| ReassemblyState {
            total_chunks,
            chunks: std::collections::BTreeMap::new(),
        });

        // The first chunk fixes total_chunks; later chunks must agree.
        if state.total_chunks != total_chunks {
            return Err(ReassemblyError::TotalChunksMismatch {
                got: total_chunks,
                expected: state.total_chunks,
            });
        }

        state.chunks.insert(chunk_index, snapshot_bytes);

        // With `chunk_index < total_chunks` enforced above, the BTreeMap's
        // keys are all in 0..total_chunks. Reaching total_chunks entries
        // therefore means we have every distinct index exactly once.
        if state.chunks.len() == state.total_chunks as usize {
            let state = self.pending.remove(&key).unwrap();
            let mut full = Vec::with_capacity(state.chunks.values().map(|c| c.len()).sum());
            for (_idx, chunk) in state.chunks {
                full.extend_from_slice(&chunk);
            }
            Ok(Some(full))
        } else {
            Ok(None)
        }
    }

    /// Cancel reassembly for a daemon (e.g., on migration abort).
    ///
    /// Removes all pending reassemblies for this daemon regardless of
    /// `seq_through`. Does **not** reset `latest_seq`, so a subsequent
    /// replay of old chunks is still rejected.
    pub fn cancel(&mut self, daemon_origin: u32) {
        self.pending
            .retain(|&(origin, _), _| origin != daemon_origin);
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

    /// Shared handle to the daemon registry this orchestrator was
    /// built against. Exposed so the migration subprotocol
    /// dispatcher can reach the registry without an extra `Arc`
    /// plumbed alongside.
    pub fn daemon_registry(&self) -> &Arc<DaemonRegistry> {
        &self.daemon_registry
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
        // Atomic check-and-insert via entry() to prevent TOCTOU races
        let entry = match self.migrations.entry(daemon_origin) {
            dashmap::mapref::entry::Entry::Occupied(_) => {
                return Err(MigrationError::AlreadyMigrating(daemon_origin));
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => entry,
        };

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

            entry.insert(Mutex::new(MigrationRecord {
                state,
                superposition,
                started_at: Instant::now(),
            }));

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

            entry.insert(Mutex::new(MigrationRecord {
                state,
                superposition,
                started_at: Instant::now(),
            }));

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
        chunk_index: u32,
        total_chunks: u32,
    ) -> Result<MigrationMessage, MigrationError> {
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;

        let mut record = entry.lock();

        // Only validate and advance phase on the first chunk
        if chunk_index == 0 && total_chunks == 1 {
            // Single-chunk: validate immediately and set snapshot
            let snapshot = StateSnapshot::from_bytes(&snapshot_bytes).ok_or_else(|| {
                MigrationError::StateFailed("failed to parse snapshot bytes".into())
            })?;

            if record.state.phase() == MigrationPhase::Snapshot {
                record.state.set_snapshot(snapshot)?;
            }
        } else if chunk_index == 0 {
            // Multi-chunk: can't validate until target reassembles all chunks.
            // Advance phase past Snapshot so buffering and subsequent phases work.
            // The target will validate the full snapshot after reassembly.
            if record.state.phase() == MigrationPhase::Snapshot {
                record.state.force_phase(MigrationPhase::Transfer);
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

    /// Handle cleanup complete from source (phase 5→6).
    ///
    /// The source has stopped accepting writes and freed its local daemon
    /// state. Advances Cutover→Complete on the orchestrator — the source's
    /// local `on_cutover_acknowledged` call is a no-op when the orchestrator
    /// lives on a different node (it operates on the source's local
    /// orchestrator, which has no record), so `CleanupComplete` is the
    /// authoritative signal on the orchestrator side. The record is kept in
    /// place until the target acknowledges activation via `on_activate_ack`,
    /// so the subprotocol handler still has somewhere to look up
    /// `target_node` when it needs to route `ActivateTarget`.
    pub fn on_cleanup_complete(
        &self,
        daemon_origin: u32,
    ) -> Result<MigrationMessage, MigrationError> {
        let entry = self
            .migrations
            .get(&daemon_origin)
            .ok_or(MigrationError::DaemonNotFound(daemon_origin))?;
        let mut record = entry.lock();
        if record.state.phase() == MigrationPhase::Cutover {
            record.state.cutover_complete()?;
        }
        Ok(MigrationMessage::ActivateTarget { daemon_origin })
    }

    /// Handle activation acknowledgement from target (phase 6 terminus).
    ///
    /// The target has drained remaining events and is now the authoritative
    /// copy. This is the true end of the migration lifecycle; the record is
    /// removed here, not in `on_cleanup_complete`.
    pub fn on_activate_ack(
        &self,
        daemon_origin: u32,
        _replayed_seq: u64,
    ) -> Result<(), MigrationError> {
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
    /// `reason` is wrapped in [`MigrationFailureReason::StateFailed`]
    /// since a generic abort doesn't fit any of the more specific
    /// variants.
    pub fn abort_migration(
        &self,
        daemon_origin: u32,
        reason: String,
    ) -> Result<MigrationMessage, MigrationError> {
        self.abort_migration_with_reason(daemon_origin, MigrationFailureReason::StateFailed(reason))
    }

    /// Abort a migration with a caller-supplied structured reason.
    pub fn abort_migration_with_reason(
        &self,
        daemon_origin: u32,
        reason: MigrationFailureReason,
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

    /// Get the source node for an in-flight migration.
    pub fn source_node(&self, daemon_origin: u32) -> Option<u64> {
        self.migrations
            .get(&daemon_origin)
            .map(|entry| entry.lock().state.source_node())
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
    use bytes::{BufMut, Bytes};

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
                // `abort_migration` wraps its string in `StateFailed`.
                match reason {
                    MigrationFailureReason::StateFailed(msg) => {
                        assert_eq!(msg, "test abort")
                    }
                    other => panic!("expected StateFailed, got {other:?}"),
                }
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
        let encoded = wire::encode(&msg).unwrap();
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
        let encoded = wire::encode(&msg).unwrap();
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
        let chunks = chunk_snapshot(0xAAAA, vec![1, 2, 3], 10).unwrap();
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
                assert_eq!(snapshot_bytes, &[1, 2, 3]);
            }
            _ => panic!("expected SnapshotReady"),
        }
    }

    #[test]
    fn test_chunk_snapshot_large() {
        // Create a snapshot larger than MAX_SNAPSHOT_CHUNK_SIZE
        let big = vec![0xABu8; MAX_SNAPSHOT_CHUNK_SIZE * 3 + 100];
        let total_len = big.len();
        let chunks = chunk_snapshot(0xBBBB, big, 42).unwrap();

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
                    assert_eq!(*chunk_index, i as u32);
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
                let result = reassembler
                    .feed(
                        daemon_origin,
                        snapshot_bytes,
                        seq_through,
                        chunk_index,
                        total_chunks,
                    )
                    .expect("legitimate chunks must not be rejected");
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
        reassembler.feed(0xAAAA, vec![1, 2], 10, 0, 3).unwrap();
        assert_eq!(reassembler.pending_count(), 1);
        reassembler.cancel(0xAAAA);
        assert_eq!(reassembler.pending_count(), 0);
    }

    // ---- Regression tests: SnapshotReassembler DoS / forgery holes ----

    #[test]
    fn test_regression_reassembler_rejects_chunk_index_out_of_range() {
        // Regression: feed() never checked that `chunk_index < total_chunks`,
        // so an attacker could declare total_chunks=3 and feed indices
        // {0, 5, 7}. The BTreeMap happily stored them, `chunks.len() == 3 ==
        // total_chunks` fired "complete", and the reassembler concatenated
        // three non-contiguous chunks as if they were chunks 0,1,2 —
        // silently forging a snapshot from attacker-chosen partial content.
        //
        // Fix: feed() rejects any chunk with `chunk_index >= total_chunks`
        // before touching state.
        let mut reassembler = SnapshotReassembler::new();

        let r0 = reassembler.feed(0xAAAA, vec![1; 10], 1, 0, 3);
        assert!(r0.is_ok(), "in-range chunk must be accepted: {:?}", r0);

        let forged = reassembler.feed(0xAAAA, vec![2; 10], 1, 5, 3);
        assert!(
            matches!(
                forged,
                Err(ReassemblyError::ChunkIndexOutOfRange {
                    chunk_index: 5,
                    total_chunks: 3,
                })
            ),
            "chunk_index=5 with total_chunks=3 must be rejected, got {:?}",
            forged
        );

        // The reassembly must not have "completed" from the forged chunk —
        // still waiting for real chunks 1 and 2.
        assert_eq!(
            reassembler.pending_count(),
            1,
            "state must stay in-flight after rejected chunk"
        );
    }

    #[test]
    fn test_regression_reassembler_rejects_zero_total_chunks() {
        // Regression: total_chunks == 0 created a ReassemblyState that
        // could never complete (len check 0 == 0 never true after the
        // first insert), leaking memory. Fix: reject at the entry point.
        let mut reassembler = SnapshotReassembler::new();
        let result = reassembler.feed(0xAAAA, vec![1; 10], 1, 0, 0);
        assert!(matches!(result, Err(ReassemblyError::ZeroTotalChunks)));
        assert_eq!(reassembler.pending_count(), 0);
    }

    #[test]
    fn test_regression_reassembler_caps_total_chunks() {
        // Regression: an attacker could declare total_chunks = u32::MAX
        // and flood the BTreeMap with up to ~4B insertions before any
        // completion check would fire. Fix: cap total_chunks at
        // MAX_TOTAL_CHUNKS (well above any legitimate snapshot).
        let mut reassembler = SnapshotReassembler::new();
        let result = reassembler.feed(0xAAAA, vec![1; 10], 1, 0, u32::MAX);
        assert!(matches!(
            result,
            Err(ReassemblyError::TotalChunksTooLarge {
                total_chunks: u32::MAX
            })
        ));
        assert_eq!(reassembler.pending_count(), 0);
    }

    #[test]
    fn test_regression_reassembler_rejects_oversized_chunk() {
        // Defense in depth: even if the transport framing lets a larger
        // payload through, the reassembler refuses a single chunk bigger
        // than MAX_SNAPSHOT_CHUNK_SIZE.
        let mut reassembler = SnapshotReassembler::new();
        let oversized = vec![0u8; MAX_SNAPSHOT_CHUNK_SIZE + 1];
        let result = reassembler.feed(0xAAAA, oversized, 1, 0, 3);
        assert!(
            matches!(result, Err(ReassemblyError::ChunkTooLarge { .. })),
            "got {:?}",
            result
        );
    }

    #[test]
    fn test_regression_reassembler_rejects_total_chunks_mismatch() {
        // Regression: an attacker who opened a reassembly with
        // total_chunks=3 could send a later chunk declaring total_chunks=100
        // and the code would just keep inserting. Fix: the first chunk's
        // total_chunks is locked in; later chunks must agree.
        let mut reassembler = SnapshotReassembler::new();
        reassembler.feed(0xAAAA, vec![1; 10], 1, 0, 3).unwrap();
        let result = reassembler.feed(0xAAAA, vec![2; 10], 1, 1, 100);
        assert!(
            matches!(
                result,
                Err(ReassemblyError::TotalChunksMismatch {
                    got: 100,
                    expected: 3,
                })
            ),
            "got {:?}",
            result
        );
        assert_eq!(reassembler.pending_count(), 1);
    }

    #[test]
    fn test_regression_reassembler_evicts_older_seq_per_daemon() {
        // Regression: `pending` was keyed by (daemon_origin, seq_through)
        // and a fresh seq_through did NOT evict older pending reassemblies
        // for the same daemon. A peer could open unbounded in-flight
        // entries by incrementing seq_through forever.
        //
        // Fix: at most one in-flight reassembly per daemon. A newer
        // seq_through evicts older ones; older seq_through values are
        // rejected as stale.
        let mut reassembler = SnapshotReassembler::new();

        reassembler.feed(0xAAAA, vec![1; 10], 10, 0, 3).unwrap();
        reassembler.feed(0xAAAA, vec![1; 10], 11, 0, 3).unwrap();
        reassembler.feed(0xAAAA, vec![1; 10], 12, 0, 3).unwrap();

        assert_eq!(
            reassembler.pending_count(),
            1,
            "only the newest seq_through for a daemon should remain in flight"
        );

        // A stale seq_through is rejected — not silently dropped on the floor.
        let stale = reassembler.feed(0xAAAA, vec![1; 10], 5, 0, 3);
        assert!(
            matches!(
                stale,
                Err(ReassemblyError::StaleSeqThrough { got: 5, latest: 12 })
            ),
            "stale seq_through must be rejected, got {:?}",
            stale
        );
        assert_eq!(reassembler.pending_count(), 1);
    }

    #[test]
    fn test_regression_reassembler_distinct_daemons_coexist() {
        // Eviction is per-daemon, not global — parallel migrations of
        // different daemons must be able to share the reassembler.
        let mut reassembler = SnapshotReassembler::new();
        reassembler.feed(0x1111, vec![1; 10], 1, 0, 3).unwrap();
        reassembler.feed(0x2222, vec![2; 10], 7, 0, 3).unwrap();
        reassembler.feed(0x3333, vec![3; 10], 9, 0, 3).unwrap();
        assert_eq!(reassembler.pending_count(), 3);
    }

    #[test]
    fn test_regression_wire_decode_rejects_zero_total_chunks() {
        // Regression: the wire decoder accepted any u32 for total_chunks
        // and chunk_index, including nonsense like total_chunks=0. A
        // defensive validation at the wire boundary stops malformed
        // messages from ever reaching the reassembler.
        use bytes::BufMut;
        let mut buf = Vec::new();
        buf.put_u8(wire::MSG_SNAPSHOT_READY);
        buf.put_u32_le(0xAAAA); // daemon_origin
        buf.put_u64_le(1); // seq_through
        buf.put_u32_le(0); // chunk_index
        buf.put_u32_le(0); // total_chunks — invalid
        buf.put_u32_le(0); // len
        let err = wire::decode(&buf).expect_err("total_chunks=0 must be rejected");
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("total_chunks"),
            "error must mention total_chunks, got {:?}",
            err_msg
        );
    }

    #[test]
    fn test_regression_wire_decode_rejects_chunk_index_out_of_range() {
        use bytes::BufMut;
        let mut buf = Vec::new();
        buf.put_u8(wire::MSG_SNAPSHOT_READY);
        buf.put_u32_le(0xAAAA);
        buf.put_u64_le(1);
        buf.put_u32_le(5); // chunk_index
        buf.put_u32_le(3); // total_chunks — index out of range
        buf.put_u32_le(0);
        let err = wire::decode(&buf).expect_err("chunk_index >= total_chunks must be rejected");
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("chunk_index"),
            "error must mention chunk_index, got {:?}",
            err_msg
        );
    }

    #[test]
    fn test_regression_wire_decode_rejects_total_chunks_overflow() {
        use bytes::BufMut;
        let mut buf = Vec::new();
        buf.put_u8(wire::MSG_SNAPSHOT_READY);
        buf.put_u32_le(0xAAAA);
        buf.put_u64_le(1);
        buf.put_u32_le(0);
        buf.put_u32_le(u32::MAX); // total_chunks — exceeds MAX_TOTAL_CHUNKS
        buf.put_u32_le(0);
        let err = wire::decode(&buf).expect_err("total_chunks > MAX_TOTAL_CHUNKS must be rejected");
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("MAX_TOTAL_CHUNKS"),
            "error must mention MAX_TOTAL_CHUNKS, got {:?}",
            err_msg
        );
    }

    #[test]
    fn test_regression_reassembler_end_to_end_forged_chunk_cannot_complete() {
        // Integration: simulate an attacker who learns total_chunks=4 from a
        // legitimate first chunk and then tries to race ahead with forged
        // content at indices beyond the range. Even if indices {0,5,7} each
        // carry attacker-chosen bytes, the reassembler must never "complete"
        // a snapshot without receiving every real index in 0..total_chunks.
        let mut reassembler = SnapshotReassembler::new();

        // Real chunk 0 — opens the reassembly at total_chunks=4.
        let r0 = reassembler.feed(0xDEAD, vec![0xA0; 10], 1, 0, 4).unwrap();
        assert!(r0.is_none());

        // Forged out-of-range chunks — all rejected, none completes.
        for bad_idx in [4, 5, 7, 999] {
            let r = reassembler.feed(0xDEAD, vec![0xFF; 10], 1, bad_idx, 4);
            assert!(
                matches!(r, Err(ReassemblyError::ChunkIndexOutOfRange { .. })),
                "index {} must be rejected, got {:?}",
                bad_idx,
                r
            );
        }

        // A snapshot-like "complete" signal can only come from filling
        // real indices 1, 2, 3.
        assert!(reassembler
            .feed(0xDEAD, vec![0xA1; 10], 1, 1, 4)
            .unwrap()
            .is_none());
        assert!(reassembler
            .feed(0xDEAD, vec![0xA2; 10], 1, 2, 4)
            .unwrap()
            .is_none());
        let full = reassembler
            .feed(0xDEAD, vec![0xA3; 10], 1, 3, 4)
            .unwrap()
            .expect("all four real chunks received — reassembly must complete");
        // Concatenation order is by chunk_index ascending, so the payload
        // is exactly the legitimate chunks 0,1,2,3 — not a forgery.
        assert_eq!(full.len(), 40);
        assert!(full[..10].iter().all(|&b| b == 0xA0));
        assert!(full[10..20].iter().all(|&b| b == 0xA1));
        assert!(full[20..30].iter().all(|&b| b == 0xA2));
        assert!(full[30..].iter().all(|&b| b == 0xA3));
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
        let encoded = wire::encode(&msg).unwrap();
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
        // Round-trip every variant of MigrationFailureReason to
        // pin the wire-layout contract. A future bump that drops
        // or adds a variant without updating the match will trip
        // the exhaustive match below.
        for reason in [
            MigrationFailureReason::NotReady,
            MigrationFailureReason::FactoryNotFound,
            MigrationFailureReason::ComputeNotSupported,
            MigrationFailureReason::StateFailed("something broke".into()),
            MigrationFailureReason::AlreadyMigrating,
            MigrationFailureReason::IdentityTransportFailed("seal failed".into()),
            MigrationFailureReason::NotReadyTimeout { attempts: 5 },
        ] {
            let msg = MigrationMessage::MigrationFailed {
                daemon_origin: 0xCCCC,
                reason: reason.clone(),
            };
            let encoded = wire::encode(&msg).unwrap();
            let decoded = wire::decode(&encoded).unwrap();
            match decoded {
                MigrationMessage::MigrationFailed {
                    daemon_origin,
                    reason: r,
                } => {
                    assert_eq!(daemon_origin, 0xCCCC);
                    assert_eq!(r, reason);
                }
                _ => panic!("expected MigrationFailed"),
            }
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
        let encoded = wire::encode(&msg).unwrap();
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
    fn test_wire_encode_rejects_oversized_failure_reason() {
        // Regression: `reason.len() as u16` previously truncated silently when
        // the reason exceeded u16::MAX, producing a stream the decoder
        // misparses. Encoding must now return an error.
        let oversized = "x".repeat(u16::MAX as usize + 1);
        let msg = MigrationMessage::MigrationFailed {
            daemon_origin: 0xDEAD,
            reason: MigrationFailureReason::StateFailed(oversized),
        };
        let result = wire::encode(&msg);
        assert!(
            matches!(result, Err(MigrationError::StateFailed(_))),
            "encode of oversized reason must error, got {:?}",
            result
        );
    }

    #[test]
    fn test_wire_rejects_unknown_failure_code() {
        // Manually-crafted `MSG_FAILED` with code 0xFFFF (unknown
        // variant). Decoder must refuse rather than mis-parse.
        let mut buf = Vec::new();
        buf.put_u8(wire::MSG_FAILED);
        buf.put_u32_le(0xBEEF);
        buf.put_u16_le(0xFFFF); // unknown code
        let err = wire::decode(&buf).expect_err("unknown code must reject");
        match err {
            MigrationError::StateFailed(msg) => {
                assert!(msg.contains("unknown MigrationFailureReason code"));
            }
            other => panic!("expected StateFailed, got {other:?}"),
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
