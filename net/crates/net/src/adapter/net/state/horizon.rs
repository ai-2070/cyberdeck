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
    ///
    /// BUG #140: pre-fix this used `self.logical_time += 1`,
    /// which debug-panics on overflow (u64::MAX → wrap) and
    /// silently wraps to 0 in release. The `merge` path at
    /// line 75 already uses `saturating_add(1)` and the comment
    /// there acknowledges the convention; observe was the
    /// outlier. Adversarial high-cardinality observe streams
    /// could panic the receive loop in debug builds.
    /// `saturating_add(1)` makes both paths consistent.
    pub fn observe(&mut self, origin_hash: u32, sequence: u64) {
        let entry = self.entries.entry(origin_hash).or_insert(0);
        if sequence > *entry {
            *entry = sequence;
            self.logical_time = self.logical_time.saturating_add(1);
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

    /// Iterate over all (origin_hash, sequence) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&u32, &u64)> {
        self.entries.iter()
    }

    /// Merge another horizon into this one (take max of each entry).
    pub fn merge(&mut self, other: &ObservedHorizon) {
        for (&origin, &seq) in &other.entries {
            let entry = self.entries.entry(origin).or_insert(0);
            if seq > *entry {
                *entry = seq;
            }
        }
        // `saturating_add` so a pathological u64::MAX logical clock
        // on either side doesn't panic in debug or wrap in release.
        // Unreachable in practice but consistent with `observe()`'s
        // counter-hygiene convention.
        self.logical_time = self.logical_time.max(other.logical_time).saturating_add(1);
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

        // Encode max_seq as log-scale u16 to preserve ordering across full u64 range.
        // Uses the position of the highest set bit (0-63) in the high 6 bits,
        // and the next 10 significant bits as mantissa. This gives ~0.1% precision
        // for large values and exact representation for values < 1024.
        let seq_encoded = encode_seq_log(max_seq);
        ((bloom as u32) << 16) | (seq_encoded as u32)
    }

    /// Decode the log-scale sequence from an encoded horizon.
    pub fn decode_seq(encoded: u32) -> u64 {
        decode_seq_log((encoded & 0xFFFF) as u16)
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

/// Encode a u64 sequence as a log-scale u16.
///
/// Format: [exponent: 6 bits][mantissa: 10 bits]
/// Values < 1024 are encoded exactly. Larger values lose low bits
/// but preserve ordering across the full u64 range.
fn encode_seq_log(seq: u64) -> u16 {
    if seq < 1024 {
        return seq as u16;
    }
    let bits = 64 - seq.leading_zeros() as u16; // position of highest bit (1-64)
    let exponent = bits - 10; // shift needed to get top 10 bits
    let mantissa = (seq >> exponent) as u16 & 0x3FF;
    (exponent << 10) | mantissa
}

/// Decode a log-scale u16 back to approximate u64 sequence.
fn decode_seq_log(encoded: u16) -> u64 {
    if encoded < 1024 {
        return encoded as u64;
    }
    let exponent = (encoded >> 10) as u64;
    let mantissa = (encoded & 0x3FF) as u64;
    mantissa << exponent
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

    /// Regression for BUG_AUDIT_2026_04_30_CORE.md #140:
    /// `observe` previously used `+= 1`, which debug-panics on
    /// overflow at `u64::MAX`. Adversarial high-cardinality
    /// observe streams (or a long-running process that just
    /// happens to hit the wraparound) crashed the receive loop
    /// in debug builds while the same overflow saturated in
    /// release — inconsistent and panic-prone. Post-fix
    /// `observe` uses `saturating_add(1)` matching `merge`.
    #[test]
    fn observe_saturates_logical_time_at_u64_max() {
        let mut h = ObservedHorizon::new();
        // Force logical_time to u64::MAX directly — the public
        // path can't reach it in test time, but the saturating
        // semantics must be tested.
        h.logical_time = u64::MAX;

        // Pre-fix: this would `+= 1` and panic in debug builds
        // (the wrap-to-0 in release was equally a bug).
        h.observe(0xAAAA, 1);

        // Post-fix: saturated, no panic.
        assert_eq!(
            h.logical_time(),
            u64::MAX,
            "saturating_add must clamp at u64::MAX, not wrap to 0"
        );
        // Sanity: the observation was still recorded.
        assert_eq!(h.get(0xAAAA), Some(1));
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

    // ---- Regression tests for Cubic AI findings ----

    #[test]
    fn test_regression_seq_encoding_preserves_ordering() {
        // Regression: old encoding used (max_seq >> 48) as u16 with a fallback
        // to low bits, which broke ordering for values in [0x10000, 0xFFFF_FFFF_FFFF].
        // Sequence 65536 encoded as 0, sequence 65537 encoded as 1.
        // Log-scale encoding now preserves monotonic ordering.
        let test_values: Vec<u64> = vec![
            0,
            1,
            100,
            1023,
            1024, // exact range
            65535,
            65536,
            65537, // old breakpoint
            1_000_000,
            1_000_000_000, // mid-range
            u64::MAX / 2,
            u64::MAX, // high range
        ];

        for i in 0..test_values.len() - 1 {
            let a = test_values[i];
            let b = test_values[i + 1];
            let enc_a = encode_seq_log(a);
            let enc_b = encode_seq_log(b);
            assert!(
                enc_a <= enc_b,
                "encoding must preserve ordering: encode({}) = {} > encode({}) = {}",
                a,
                enc_a,
                b,
                enc_b
            );
        }
    }

    #[test]
    fn test_regression_seq_roundtrip_exact_for_small() {
        // Values < 1024 must roundtrip exactly.
        for v in 0..1024u64 {
            let encoded = encode_seq_log(v);
            let decoded = decode_seq_log(encoded);
            assert_eq!(decoded, v, "small value {} must roundtrip exactly", v);
        }
    }

    #[test]
    fn test_seq_encoding_boundary_at_1024() {
        // 1023 is exact, 1024 is the first log-scale value.
        // Both must encode without panic and preserve ordering.
        let enc_1023 = encode_seq_log(1023);
        let enc_1024 = encode_seq_log(1024);
        assert!(enc_1024 > enc_1023, "1024 must encode higher than 1023");

        // 1024 should roundtrip exactly (it's a power of 2, mantissa is clean)
        let dec_1024 = decode_seq_log(enc_1024);
        assert_eq!(dec_1024, 1024, "1024 (power of 2) should roundtrip exactly");
    }

    #[test]
    fn test_seq_encoding_does_not_overflow_u16() {
        // The encoded value must fit in u16 for all u64 inputs.
        // The most extreme input is u64::MAX.
        let enc = encode_seq_log(u64::MAX);
        // enc is a u16 — if the encoding overflowed, this would have
        // panicked in debug or wrapped in release. We verify it's valid.
        assert!(enc > 0, "u64::MAX should encode to a nonzero u16");

        // Decode should produce a large value (approximate, not exact)
        let dec = decode_seq_log(enc);
        assert!(
            dec > u64::MAX / 2,
            "decoded u64::MAX should be in the right ballpark, got {}",
            dec
        );
    }

    #[test]
    fn test_seq_encoding_decode_preserves_magnitude() {
        // For large values, decode(encode(v)) should be within ~0.1% of v.
        let test_values = [
            1_000_000u64,
            1_000_000_000,
            1_000_000_000_000,
            u64::MAX / 1024,
        ];

        for &v in &test_values {
            let encoded = encode_seq_log(v);
            let decoded = decode_seq_log(encoded);
            let ratio = decoded as f64 / v as f64;
            assert!(
                (0.99..=1.01).contains(&ratio),
                "decode(encode({})) = {} — ratio {} is outside 1% tolerance",
                v,
                decoded,
                ratio
            );
        }
    }

    #[test]
    fn test_horizon_encode_decode_seq_consistency() {
        // Full round trip through ObservedHorizon → encode → decode_seq.
        let mut h = ObservedHorizon::new();
        h.observe(0xAAAA, 999_999);

        let encoded = h.encode();
        let decoded_seq = HorizonEncoder::decode_seq(encoded);

        // The decoded max-seq should approximate the input
        let ratio = decoded_seq as f64 / 999_999.0;
        assert!(
            (0.99..=1.01).contains(&ratio),
            "horizon seq should roundtrip within 1%, got ratio {}",
            ratio
        );
    }
}
