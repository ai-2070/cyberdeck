//! Compressed observed horizon for causal ordering.
//!
//! Replaces full vector clocks with a 4-byte bloom sketch that fits in
//! `CausalLink::horizon_encoded`. Provides O(1) approximate causality
//! detection without O(N) scaling.

use std::collections::HashMap;
use xxhash_rust::xxh3::xxh3_64;

/// Full vector clock — exchanged out-of-band, not per-event.
///
/// Tracks the highest sequence number observed from each entity.
#[derive(Debug, Clone, Default)]
pub struct ObservedHorizon {
    /// origin_hash -> highest sequence observed from that entity.
    entries: HashMap<u32, u64>,
    /// Logical time (incremented on each observation).
    logical_time: u64,
}

impl ObservedHorizon {
    /// Create an empty horizon.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an observation of an entity at a given sequence.
    pub fn observe(&mut self, origin_hash: u32, sequence: u64) {
        let entry = self.entries.entry(origin_hash).or_insert(0);
        if sequence > *entry {
            *entry = sequence;
            self.logical_time += 1;
        }
    }

    /// Get the highest observed sequence for an entity.
    pub fn get(&self, origin_hash: u32) -> Option<u64> {
        self.entries.get(&origin_hash).copied()
    }

    /// Check if this horizon has observed a specific (origin, sequence) pair.
    pub fn has_observed(&self, origin_hash: u32, sequence: u64) -> bool {
        self.entries
            .get(&origin_hash)
            .is_some_and(|&seq| seq >= sequence)
    }

    /// Number of entities in the horizon.
    pub fn entity_count(&self) -> usize {
        self.entries.len()
    }

    /// Logical time of this horizon.
    pub fn logical_time(&self) -> u64 {
        self.logical_time
    }

    /// Merge another horizon into this one (take max of each entry).
    pub fn merge(&mut self, other: &ObservedHorizon) {
        for (&origin, &seq) in &other.entries {
            let entry = self.entries.entry(origin).or_insert(0);
            if seq > *entry {
                *entry = seq;
            }
        }
        self.logical_time = self.logical_time.max(other.logical_time) + 1;
    }

    /// Encode to a 4-byte compressed horizon for CausalLink.
    ///
    /// Format:
    /// - bits 31-16: bloom filter of recently-observed origin_hashes (16 bits)
    /// - bits 15-0: truncated max observed sequence across all entities (16 bits)
    pub fn encode(&self) -> u32 {
        HorizonEncoder::encode(self)
    }
}

/// Encoder/decoder for the 4-byte compressed horizon.
pub struct HorizonEncoder;

impl HorizonEncoder {
    /// Encode a full horizon into 4 bytes.
    pub fn encode(horizon: &ObservedHorizon) -> u32 {
        let mut bloom: u16 = 0;
        let mut max_seq: u64 = 0;

        for (&origin_hash, &seq) in &horizon.entries {
            // Set bloom bits for this origin (2 hash functions)
            let h = xxh3_64(&origin_hash.to_le_bytes());
            bloom |= 1 << (h & 0xF);
            bloom |= 1 << ((h >> 4) & 0xF);

            if seq > max_seq {
                max_seq = seq;
            }
        }

        // Truncate max_seq to high 16 bits
        let seq_compressed = (max_seq >> 48) as u16;
        if seq_compressed == 0 && max_seq > 0 {
            // For small sequences, use low bits instead
            let seq_low = (max_seq & 0xFFFF) as u16;
            ((bloom as u32) << 16) | (seq_low as u32)
        } else {
            ((bloom as u32) << 16) | (seq_compressed as u32)
        }
    }

    /// Check if an origin_hash is possibly in a compressed horizon.
    ///
    /// Returns false = definitely not observed, true = possibly observed.
    pub fn might_contain(encoded: u32, origin_hash: u32) -> bool {
        let bloom = (encoded >> 16) as u16;
        let h = xxh3_64(&origin_hash.to_le_bytes());
        let bit1 = 1u16 << (h & 0xF);
        let bit2 = 1u16 << ((h >> 4) & 0xF);
        (bloom & bit1) != 0 && (bloom & bit2) != 0
    }

    /// Check if two events are potentially concurrent (neither observed the other).
    pub fn potentially_concurrent(
        horizon_a: u32,
        origin_b: u32,
        horizon_b: u32,
        origin_a: u32,
    ) -> bool {
        !Self::might_contain(horizon_a, origin_b) && !Self::might_contain(horizon_b, origin_a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_horizon() {
        let h = ObservedHorizon::new();
        assert_eq!(h.entity_count(), 0);
        assert_eq!(h.encode(), 0);
    }

    #[test]
    fn test_observe() {
        let mut h = ObservedHorizon::new();
        h.observe(0xAAAA, 10);
        h.observe(0xBBBB, 20);

        assert_eq!(h.get(0xAAAA), Some(10));
        assert_eq!(h.get(0xBBBB), Some(20));
        assert_eq!(h.get(0xCCCC), None);
        assert!(h.has_observed(0xAAAA, 5));
        assert!(h.has_observed(0xAAAA, 10));
        assert!(!h.has_observed(0xAAAA, 11));
    }

    #[test]
    fn test_observe_max_only() {
        let mut h = ObservedHorizon::new();
        h.observe(0xAAAA, 10);
        h.observe(0xAAAA, 5); // lower — should not change
        assert_eq!(h.get(0xAAAA), Some(10));

        h.observe(0xAAAA, 15); // higher — should update
        assert_eq!(h.get(0xAAAA), Some(15));
    }

    #[test]
    fn test_merge() {
        let mut h1 = ObservedHorizon::new();
        h1.observe(0xAAAA, 10);
        h1.observe(0xBBBB, 5);

        let mut h2 = ObservedHorizon::new();
        h2.observe(0xAAAA, 8);
        h2.observe(0xCCCC, 20);

        h1.merge(&h2);
        assert_eq!(h1.get(0xAAAA), Some(10)); // max(10, 8)
        assert_eq!(h1.get(0xBBBB), Some(5));
        assert_eq!(h1.get(0xCCCC), Some(20));
    }

    #[test]
    fn test_encode_nonzero() {
        let mut h = ObservedHorizon::new();
        h.observe(0xAAAA, 42);
        let encoded = h.encode();
        assert_ne!(encoded, 0);
    }

    #[test]
    fn test_might_contain() {
        let mut h = ObservedHorizon::new();
        h.observe(0xAAAA, 10);
        h.observe(0xBBBB, 20);
        let encoded = h.encode();

        // Should detect observed origins
        assert!(HorizonEncoder::might_contain(encoded, 0xAAAA));
        assert!(HorizonEncoder::might_contain(encoded, 0xBBBB));

        // Empty horizon should not contain anything
        assert!(!HorizonEncoder::might_contain(0, 0xAAAA));
    }

    #[test]
    fn test_potentially_concurrent() {
        let mut h_a = ObservedHorizon::new();
        h_a.observe(0xAAAA, 10); // A has observed itself
        let enc_a = h_a.encode();

        let mut h_b = ObservedHorizon::new();
        h_b.observe(0xBBBB, 10); // B has observed itself
        let enc_b = h_b.encode();

        // A hasn't observed B and B hasn't observed A — concurrent
        assert!(HorizonEncoder::potentially_concurrent(
            enc_a, 0xBBBB, enc_b, 0xAAAA
        ));

        // Now A observes B
        h_a.observe(0xBBBB, 10);
        let enc_a2 = h_a.encode();

        // A has observed B — no longer concurrent from A's perspective
        assert!(!HorizonEncoder::potentially_concurrent(
            enc_a2, 0xBBBB, enc_b, 0xAAAA
        ));
    }
}
