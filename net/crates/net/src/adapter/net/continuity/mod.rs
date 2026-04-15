//! Layer 7: Observational Continuity for Net.
//!
//! Each node's truth is what it can observe. The causal chain IS identity.
//! This module provides the query/reasoning layer on top of L4's causal
//! infrastructure: causal cones, propagation modeling, continuity proofs,
//! honest discontinuity handling, and superposition during migration.

pub mod chain;
pub mod cone;
pub mod discontinuity;
pub mod observation;
pub mod propagation;
pub mod superposition;

/// Subprotocol ID for continuity messages.
pub const SUBPROTOCOL_CONTINUITY: u16 = 0x0700;

/// Subprotocol ID for fork announcements.
pub const SUBPROTOCOL_FORK_ANNOUNCE: u16 = 0x0701;

/// Subprotocol ID for continuity proof exchange.
pub const SUBPROTOCOL_CONTINUITY_PROOF: u16 = 0x0702;

pub use chain::{
    assess_continuity, ContinuityProof, ContinuityStatus, ProofError, CONTINUITY_PROOF_SIZE,
};
pub use cone::{CausalCone, Causality};
pub use discontinuity::{
    fork_entity, fork_sentinel, Discontinuity, DiscontinuityReason, ForkRecord,
};
pub use observation::{HorizonDivergence, ObservationWindow};
pub use propagation::{crossing_depth, PropagationModel};
pub use superposition::{SuperpositionPhase, SuperpositionState};
