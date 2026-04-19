//! MeshDaemon trait and supporting types.
//!
//! A daemon is a stateful or stateless event processor that runs on the mesh.
//! It consumes causal events and produces output events. The runtime handles
//! chain building, horizon tracking, and snapshot packaging.

use bytes::Bytes;

use crate::adapter::net::behavior::capability::CapabilityFilter;
use crate::adapter::net::state::causal::CausalEvent;

/// A daemon that runs on the mesh.
///
/// Daemons consume inbound causal events via `process()` and return zero or
/// more output payloads. The runtime wraps outputs in `CausalLink`s
/// automatically — the daemon only produces raw payloads.
///
/// # Performance
///
/// `process()` must complete in microseconds. Heavy work should be deferred
/// to a background task and emitted as a later event.
///
/// # WASM compatibility
///
/// All methods are synchronous — no async. Input/output are `Bytes` — maps
/// cleanly to WASM linear memory. No generics or associated types.
pub trait MeshDaemon: Send + Sync {
    /// Human-readable name (for logging, placement ads).
    fn name(&self) -> &str;

    /// Capability requirements for placement.
    ///
    /// The scheduler uses this to find nodes whose `CapabilitySet` matches.
    /// Return `CapabilityFilter::default()` to run anywhere.
    fn requirements(&self) -> CapabilityFilter;

    /// Process one inbound causal event, returning zero or more output payloads.
    ///
    /// The output `Bytes` values become payloads in the daemon's own causal
    /// chain (the runtime wraps them in CausalLinks automatically).
    fn process(&mut self, event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError>;

    /// Serialize current state for migration/checkpoint.
    ///
    /// Returns `None` for stateless daemons. Stateful daemons must return
    /// opaque bytes that `restore()` can accept.
    fn snapshot(&self) -> Option<Bytes> {
        None
    }

    /// Restore from a previous snapshot.
    ///
    /// Called before any `process()` calls after migration.
    /// The default implementation accepts any state (for stateless daemons).
    fn restore(&mut self, _state: Bytes) -> Result<(), DaemonError> {
        Ok(())
    }
}

/// Errors from daemon operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonError {
    /// Daemon processing logic failed.
    ProcessFailed(String),
    /// Snapshot serialization failed.
    SnapshotFailed(String),
    /// Restore from snapshot failed.
    RestoreFailed(String),
    /// Daemon not found in registry.
    NotFound(u32),
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessFailed(msg) => write!(f, "daemon process failed: {}", msg),
            Self::SnapshotFailed(msg) => write!(f, "snapshot failed: {}", msg),
            Self::RestoreFailed(msg) => write!(f, "restore failed: {}", msg),
            Self::NotFound(id) => write!(f, "daemon not found: {:#x}", id),
        }
    }
}

impl std::error::Error for DaemonError {}

/// Configuration for a daemon host.
#[derive(Debug, Clone)]
pub struct DaemonHostConfig {
    /// How often to auto-snapshot (in events processed). 0 = manual only.
    pub auto_snapshot_interval: u64,
    /// Maximum events to buffer before forcing a snapshot.
    pub max_log_entries: u32,
}

impl Default for DaemonHostConfig {
    fn default() -> Self {
        Self {
            auto_snapshot_interval: 0,
            max_log_entries: 10_000,
        }
    }
}

/// Runtime statistics for a daemon.
#[derive(Debug, Clone, Default)]
pub struct DaemonStats {
    /// Total events processed.
    pub events_processed: u64,
    /// Total output events emitted.
    pub events_emitted: u64,
    /// Total processing errors.
    pub errors: u64,
    /// Number of snapshots taken.
    pub snapshots_taken: u64,
}

/// Per-daemon consumption snapshot for metered-compute accounting.
///
/// `DaemonStats` tracks what happened; `ResourceUsage` is framed for the
/// settlement layer — wall-clock time plus the event counters the dashboard
/// bills against. OS-level CPU/RSS are out of scope today.
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Total events processed (same counter as `DaemonStats::events_processed`).
    pub events_processed: u64,
    /// Total output events emitted (same counter as `DaemonStats::events_emitted`).
    pub events_emitted: u64,
    /// Total processing errors (same counter as `DaemonStats::errors`).
    pub errors: u64,
    /// Seconds since the host was constructed.
    pub uptime_secs: u64,
    /// Cumulative wall-clock nanoseconds spent inside `MeshDaemon::process`.
    pub cumulative_process_ns: u64,
}
