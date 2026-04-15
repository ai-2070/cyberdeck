//! Superposition during migration — entity on two nodes simultaneously.
//!
//! Wraps `MigrationState` with observational semantics. During migration,
//! an entity exists in superposition until routing "collapses" it to the
//! target node.

use crate::adapter::nltp::compute::MigrationPhase;
use crate::adapter::nltp::state::causal::CausalLink;

use super::chain::ContinuityProof;

/// Observational phase of an entity during migration.
///
/// Maps to `MigrationPhase` but with physical semantics:
/// the entity's "wavefunction" spreads, superposes, and collapses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuperpositionPhase {
    /// Entity exists only on source (pre-snapshot).
    Localized,
    /// Snapshot taken, target restoring. Entity still authoritative on source.
    Spreading,
    /// Both nodes may hold the entity. Superposed state.
    Superposed,
    /// Target caught up. Ready to collapse to single location.
    ReadyToCollapse,
    /// Routing switched. Target is canonical. Source draining.
    Collapsed,
    /// Source cleaned up. Back to single location on target.
    Resolved,
}

impl SuperpositionPhase {
    /// Map from MigrationPhase to SuperpositionPhase.
    pub fn from_migration(phase: MigrationPhase) -> Self {
        match phase {
            MigrationPhase::Snapshot => Self::Localized,
            MigrationPhase::Transfer => Self::Spreading,
            MigrationPhase::Restore => Self::Spreading,
            MigrationPhase::Replay => Self::Superposed,
            MigrationPhase::Cutover => Self::Collapsed,
            MigrationPhase::Complete => Self::Resolved,
        }
    }

    /// Whether the entity currently exists on multiple nodes.
    pub fn is_superposed(self) -> bool {
        matches!(self, Self::Superposed | Self::ReadyToCollapse)
    }

    /// Whether routing has collapsed to a single node.
    pub fn is_collapsed(self) -> bool {
        matches!(self, Self::Collapsed | Self::Resolved)
    }
}

/// Tracks an entity's superposition state during migration.
///
/// Provides observational semantics on top of the mechanical
/// `MigrationState` from L5.
pub struct SuperpositionState {
    /// Entity being migrated.
    origin_hash: u32,
    /// Source node's chain head at snapshot time.
    source_head: CausalLink,
    /// Target node's chain head (advances during replay).
    target_head: CausalLink,
    /// Current superposition phase.
    phase: SuperpositionPhase,
    /// Events observed by source since snapshot.
    source_observed_since: u64,
    /// Events replayed on target.
    target_replayed_through: u64,
}

impl SuperpositionState {
    /// Create a new superposition state when migration begins.
    pub fn new(origin_hash: u32, source_head: CausalLink) -> Self {
        Self {
            origin_hash,
            source_head,
            target_head: source_head, // target starts from same point
            phase: SuperpositionPhase::Localized,
            source_observed_since: 0,
            target_replayed_through: source_head.sequence,
        }
    }

    /// Advance the phase based on migration progress.
    pub fn advance(&mut self, migration_phase: MigrationPhase) {
        self.phase = SuperpositionPhase::from_migration(migration_phase);
    }

    /// Record that source has processed more events since snapshot.
    pub fn source_advanced(&mut self, new_head: CausalLink) {
        self.source_head = new_head;
        self.source_observed_since = new_head
            .sequence
            .saturating_sub(self.target_replayed_through);
    }

    /// Record that target has replayed events.
    pub fn target_replayed(&mut self, new_head: CausalLink) {
        self.target_head = new_head;
        self.target_replayed_through = new_head.sequence;

        // Check if target has caught up to source
        if self.target_replayed_through >= self.source_head.sequence
            && self.phase == SuperpositionPhase::Superposed
        {
            self.phase = SuperpositionPhase::ReadyToCollapse;
        }
    }

    /// Whether the target has caught up to the source.
    pub fn target_caught_up(&self) -> bool {
        self.target_replayed_through >= self.source_head.sequence
    }

    /// Collapse the superposition (routing switches to target).
    pub fn collapse(&mut self) {
        self.phase = SuperpositionPhase::Collapsed;
    }

    /// Mark migration as fully resolved.
    pub fn resolve(&mut self) {
        self.phase = SuperpositionPhase::Resolved;
    }

    /// Generate a continuity proof spanning the migration.
    ///
    /// Proves that the chain is intact from the source's snapshot point
    /// through the target's current head.
    pub fn continuity_proof(&self) -> ContinuityProof {
        ContinuityProof {
            origin_hash: self.origin_hash,
            from_seq: self.source_head.sequence.min(self.target_head.sequence),
            to_seq: self.source_head.sequence.max(self.target_head.sequence),
            from_hash: self.source_head.parent_hash,
            to_hash: self.target_head.parent_hash,
        }
    }

    /// Get the current phase.
    #[inline]
    pub fn phase(&self) -> SuperpositionPhase {
        self.phase
    }

    /// Get the origin hash.
    #[inline]
    pub fn origin_hash(&self) -> u32 {
        self.origin_hash
    }

    /// Get the source head.
    #[inline]
    pub fn source_head(&self) -> &CausalLink {
        &self.source_head
    }

    /// Get the target head.
    #[inline]
    pub fn target_head(&self) -> &CausalLink {
        &self.target_head
    }

    /// Events the target still needs to replay.
    pub fn replay_gap(&self) -> u64 {
        self.source_head
            .sequence
            .saturating_sub(self.target_replayed_through)
    }
}

impl std::fmt::Debug for SuperpositionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SuperpositionState")
            .field("origin", &format!("{:#x}", self.origin_hash))
            .field("phase", &self.phase)
            .field("source_seq", &self.source_head.sequence)
            .field("target_seq", &self.target_head.sequence)
            .field("replay_gap", &self.replay_gap())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_link(origin: u32, seq: u64) -> CausalLink {
        CausalLink {
            origin_hash: origin,
            horizon_encoded: 0,
            sequence: seq,
            parent_hash: seq * 1000, // deterministic for testing
        }
    }

    #[test]
    fn test_lifecycle() {
        let source_head = make_link(0xAAAA, 100);
        let mut state = SuperpositionState::new(0xAAAA, source_head);

        assert_eq!(state.phase(), SuperpositionPhase::Localized);
        assert!(!state.phase().is_superposed());

        // Migration starts
        state.advance(MigrationPhase::Transfer);
        assert_eq!(state.phase(), SuperpositionPhase::Spreading);

        state.advance(MigrationPhase::Replay);
        assert_eq!(state.phase(), SuperpositionPhase::Superposed);
        assert!(state.phase().is_superposed());

        // Source advances while target replays
        state.source_advanced(make_link(0xAAAA, 105));
        assert_eq!(state.replay_gap(), 5);
        assert!(!state.target_caught_up());

        // Target catches up
        state.target_replayed(make_link(0xAAAA, 105));
        assert!(state.target_caught_up());
        assert_eq!(state.phase(), SuperpositionPhase::ReadyToCollapse);

        // Collapse
        state.collapse();
        assert!(state.phase().is_collapsed());

        state.resolve();
        assert_eq!(state.phase(), SuperpositionPhase::Resolved);
    }

    #[test]
    fn test_continuity_proof() {
        let source_head = make_link(0xAAAA, 100);
        let mut state = SuperpositionState::new(0xAAAA, source_head);

        state.source_advanced(make_link(0xAAAA, 110));
        state.target_replayed(make_link(0xAAAA, 105));

        let proof = state.continuity_proof();
        assert_eq!(proof.origin_hash, 0xAAAA);
        assert_eq!(proof.from_seq, 105);
        assert_eq!(proof.to_seq, 110);
    }

    #[test]
    fn test_auto_ready_to_collapse() {
        let source_head = make_link(0xAAAA, 50);
        let mut state = SuperpositionState::new(0xAAAA, source_head);

        state.advance(MigrationPhase::Replay);
        assert_eq!(state.phase(), SuperpositionPhase::Superposed);

        // Target catches up while in Superposed phase
        state.target_replayed(make_link(0xAAAA, 50));
        assert_eq!(state.phase(), SuperpositionPhase::ReadyToCollapse);
    }

    #[test]
    fn test_replay_gap() {
        let source_head = make_link(0xAAAA, 100);
        let mut state = SuperpositionState::new(0xAAAA, source_head);

        assert_eq!(state.replay_gap(), 0); // target starts at same point

        state.source_advanced(make_link(0xAAAA, 120));
        assert_eq!(state.replay_gap(), 20);

        state.target_replayed(make_link(0xAAAA, 110));
        assert_eq!(state.replay_gap(), 10);
    }
}
