//! Layer 5: Compute Runtime for BLTP.
//!
//! Defines the `MeshDaemon` trait for event processors, `DaemonHost` for
//! runtime management, `DaemonRegistry` for local daemon tracking,
//! `Scheduler` for capability-based placement, and `MigrationState`
//! for snapshot-based daemon migration.

mod daemon;
mod host;
mod migration;
mod registry;
mod scheduler;

pub use daemon::{DaemonError, DaemonHostConfig, DaemonStats, MeshDaemon};
pub use host::DaemonHost;
pub use migration::{MigrationError, MigrationPhase, MigrationState, SUBPROTOCOL_MIGRATION};
pub use registry::DaemonRegistry;
pub use scheduler::{PlacementDecision, PlacementReason, Scheduler, SchedulerError};
