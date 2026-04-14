//! Causal cones — what events could have influenced a given event.
//!
//! A `CausalCone` answers: "given event E, which other entities' events
//! could have causally preceded E?" Uses the compressed horizon from
//! `CausalLink::horizon_encoded` for approximate O(1) queries, or the
//! full `ObservedHorizon` for exact answers when available.

use crate::adapter::bltp::state::causal::CausalLink;
use crate::adapter::bltp::state::horizon::{HorizonEncoder, ObservedHorizon};

/// The causal relationship between two events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Causality {
    /// Event A definitely preceded event B (exact horizon match).
    Definite,
    /// Event A possibly preceded event B (bloom filter match, may be false positive).
    Possible,
    /// Event A definitely did NOT precede event B.
    No,
    /// Cannot determine (insufficient information).
    Unknown,
}

/// The causal cone of an event: the set of observations that preceded it.
///
/// Constructed from a `CausalLink`'s horizon data. Local node has exact
/// cone (full `ObservedHorizon`), remote observers only see the approximate
/// 4-byte compressed horizon.
#[derive(Debug, Clone)]
pub struct CausalCone {
    /// Origin of the event this cone belongs to.
    origin_hash: u32,
    /// Sequence of the event.
    sequence: u64,
    /// Full horizon at this point (exact, if available locally).
    horizon: Option<ObservedHorizon>,
    /// Compressed horizon from the wire (always available).
    horizon_encoded: u32,
}

impl CausalCone {
    /// Construct from wire data only (compressed, approximate).
    pub fn from_causal_link(link: &CausalLink) -> Self {
        Self {
            origin_hash: link.origin_hash,
            sequence: link.sequence,
            horizon: None,
            horizon_encoded: link.horizon_encoded,
        }
    }

    /// Construct from full data (exact).
    pub fn from_link_with_horizon(link: &CausalLink, horizon: &ObservedHorizon) -> Self {
        Self {
            origin_hash: link.origin_hash,
            sequence: link.sequence,
            horizon: Some(horizon.clone()),
            horizon_encoded: link.horizon_encoded,
        }
    }

    /// Could event from `other_origin` at `other_seq` have influenced this event?
    pub fn could_have_influenced(&self, other_origin: u32, other_seq: u64) -> Causality {
        // Same entity: strictly ordered by sequence
        if other_origin == self.origin_hash {
            return if other_seq < self.sequence {
                Causality::Definite
            } else {
                Causality::No
            };
        }

        // Check full horizon first (exact answer)
        if let Some(ref horizon) = self.horizon {
            return if horizon.has_observed(other_origin, other_seq) {
                Causality::Definite
            } else {
                Causality::No
            };
        }

        // Fall back to compressed horizon (approximate)
        if HorizonEncoder::might_contain(self.horizon_encoded, other_origin) {
            Causality::Possible
        } else {
            Causality::No
        }
    }

    /// Check if this event is concurrent with another (neither causally preceded the other).
    pub fn is_concurrent_with(&self, other: &CausalCone) -> bool {
        HorizonEncoder::potentially_concurrent(
            self.horizon_encoded,
            other.origin_hash,
            other.horizon_encoded,
            self.origin_hash,
        )
    }

    /// Merge multiple cones (for daemons with multiple inputs).
    ///
    /// The merged cone's horizon is the union of all input horizons.
    pub fn merge(cones: &[CausalCone]) -> Option<CausalCone> {
        if cones.is_empty() {
            return None;
        }

        let first = &cones[0];
        let mut merged_horizon = first.horizon.clone().unwrap_or_default();
        let mut merged_encoded = first.horizon_encoded;

        for cone in &cones[1..] {
            if let Some(ref h) = cone.horizon {
                merged_horizon.merge(h);
            }
            // OR the bloom bits together for approximate merge
            merged_encoded |= cone.horizon_encoded;
        }

        Some(CausalCone {
            origin_hash: first.origin_hash,
            sequence: first.sequence,
            horizon: Some(merged_horizon),
            horizon_encoded: merged_encoded,
        })
    }

    /// Get the origin hash.
    #[inline]
    pub fn origin_hash(&self) -> u32 {
        self.origin_hash
    }

    /// Get the sequence number.
    #[inline]
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Get the compressed horizon.
    #[inline]
    pub fn horizon_encoded(&self) -> u32 {
        self.horizon_encoded
    }

    /// Whether this cone has exact (full horizon) data.
    #[inline]
    pub fn is_exact(&self) -> bool {
        self.horizon.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_link(origin: u32, seq: u64, horizon: u32) -> CausalLink {
        CausalLink {
            origin_hash: origin,
            horizon_encoded: horizon,
            sequence: seq,
            parent_hash: 0,
        }
    }

    #[test]
    fn test_same_entity_ordering() {
        let link = make_link(0xAAAA, 10, 0);
        let cone = CausalCone::from_causal_link(&link);

        // Earlier event from same entity = definite influence
        assert_eq!(cone.could_have_influenced(0xAAAA, 5), Causality::Definite);
        // Later event = no influence
        assert_eq!(cone.could_have_influenced(0xAAAA, 15), Causality::No);
        // Same sequence = no (not strictly before)
        assert_eq!(cone.could_have_influenced(0xAAAA, 10), Causality::No);
    }

    #[test]
    fn test_exact_horizon() {
        let mut horizon = ObservedHorizon::new();
        horizon.observe(0xBBBB, 20);

        let link = make_link(0xAAAA, 10, horizon.encode());
        let cone = CausalCone::from_link_with_horizon(&link, &horizon);

        assert!(cone.is_exact());
        assert_eq!(cone.could_have_influenced(0xBBBB, 15), Causality::Definite);
        assert_eq!(cone.could_have_influenced(0xBBBB, 25), Causality::No);
        assert_eq!(cone.could_have_influenced(0xCCCC, 5), Causality::No);
    }

    #[test]
    fn test_approximate_horizon() {
        let mut horizon = ObservedHorizon::new();
        horizon.observe(0xBBBB, 20);
        let encoded = horizon.encode();

        let link = make_link(0xAAAA, 10, encoded);
        let cone = CausalCone::from_causal_link(&link); // no full horizon

        assert!(!cone.is_exact());
        // Bloom filter should report Possible for observed origin
        let result = cone.could_have_influenced(0xBBBB, 15);
        assert!(result == Causality::Possible || result == Causality::No);
    }

    #[test]
    fn test_concurrent_events() {
        // A hasn't observed B, B hasn't observed A
        let mut h_a = ObservedHorizon::new();
        h_a.observe(0xAAAA, 10);
        let mut h_b = ObservedHorizon::new();
        h_b.observe(0xBBBB, 10);

        let link_a = make_link(0xAAAA, 10, h_a.encode());
        let link_b = make_link(0xBBBB, 10, h_b.encode());

        let cone_a = CausalCone::from_causal_link(&link_a);
        let cone_b = CausalCone::from_causal_link(&link_b);

        assert!(cone_a.is_concurrent_with(&cone_b));
    }

    #[test]
    fn test_merge_cones() {
        let mut h1 = ObservedHorizon::new();
        h1.observe(0xBBBB, 5);
        let mut h2 = ObservedHorizon::new();
        h2.observe(0xCCCC, 10);

        let link1 = make_link(0xAAAA, 1, h1.encode());
        let link2 = make_link(0xAAAA, 2, h2.encode());

        let cone1 = CausalCone::from_link_with_horizon(&link1, &h1);
        let cone2 = CausalCone::from_link_with_horizon(&link2, &h2);

        let merged = CausalCone::merge(&[cone1, cone2]).unwrap();
        assert!(merged.is_exact());

        // Merged cone should know about both entities
        assert_eq!(merged.could_have_influenced(0xBBBB, 3), Causality::Definite);
        assert_eq!(merged.could_have_influenced(0xCCCC, 8), Causality::Definite);
    }

    #[test]
    fn test_merge_empty() {
        assert!(CausalCone::merge(&[]).is_none());
    }
}
