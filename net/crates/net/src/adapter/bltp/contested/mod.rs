//! Layer 8 (Partial): Contested Environments for BLTP.
//!
//! Correlated failure detection and partition healing. Detects mass
//! failures, classifies them by subnet correlation, tracks partition
//! state, and reconciles divergent event logs after healing.

pub mod correlation;
pub mod partition;
pub mod reconcile;

/// Subprotocol ID for partition coordination messages.
pub const SUBPROTOCOL_PARTITION: u16 = 0x0800;

/// Subprotocol ID for log reconciliation messages.
pub const SUBPROTOCOL_RECONCILE: u16 = 0x0801;

pub use correlation::{
    CorrelatedFailureConfig, CorrelatedFailureDetector, CorrelationVerdict, FailureCause,
};
pub use partition::{PartitionDetector, PartitionPhase, PartitionRecord};
pub use reconcile::{
    reconcile_entity, verify_remote_chain, ConflictResolution, ReconcileOutcome, Side,
};
