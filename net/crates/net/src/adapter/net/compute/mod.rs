//! Layer 5: Compute Runtime for Net.
//!
//! Defines the `MeshDaemon` trait for event processors, `DaemonHost` for
//! runtime management, `DaemonRegistry` for local daemon tracking,
//! `Scheduler` for capability-based placement, and `MigrationState`
//! for snapshot-based daemon migration.

mod daemon;
mod host;
mod migration;
pub mod migration_source;
pub mod migration_target;
pub mod orchestrator;
mod registry;
mod scheduler;

pub use daemon::{DaemonError, DaemonHostConfig, DaemonStats, MeshDaemon};
pub use host::DaemonHost;
pub use migration::{MigrationError, MigrationPhase, MigrationState, SUBPROTOCOL_MIGRATION};
pub use migration_source::MigrationSourceHandler;
pub use migration_target::MigrationTargetHandler;
pub use orchestrator::{
    chunk_snapshot, MigrationMessage, MigrationOrchestrator, SnapshotReassembler,
    MAX_SNAPSHOT_CHUNK_SIZE, MAX_SNAPSHOT_SIZE,
};
pub use registry::DaemonRegistry;
pub use scheduler::{PlacementDecision, PlacementReason, Scheduler, SchedulerError};
