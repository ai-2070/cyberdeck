//! Wire-speed authorization guard for BLTP packets.
//!
//! The `AuthGuard` uses a bloom filter to authorize packets in under 10ns.
//! Authorized `(origin_hash, channel_hash)` pairs are inserted at subscription
//! time (slow path). The per-packet fast path probes the bloom filter with
//! no crypto, no heap allocation, and no pointer chasing.

use std::sync::atomic::{AtomicU8, Ordering};

use dashmap::DashMap;
use xxhash_rust::xxh3::xxh3_64;

/// Result of a fast-path authorization check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthVerdict {
    /// Packet is authorized (bloom hit + verified cache hit).
    Allowed,
    /// Packet is definitely not authorized (bloom miss).
    Denied,
    /// Bloom filter hit but not in verified cache — needs full check.
    NeedsFullCheck,
}

/// Wire-speed authorization guard.
///
/// Contains a bloom filter for O(1) per-packet checks and a verified-positive
/// cache to avoid repeated full token verification on bloom hits.
///
/// # Performance
///
/// `check_fast()` does:
/// - 2 hash computations (xxh3, ~1ns each)
/// - 2 array lookups (bloom filter bits)
/// - 1 DashMap probe (verified cache, ~5ns)
///
/// Total: <10ns for the Allowed/Denied paths.
pub struct AuthGuard {
    /// Bloom filter bits using atomics for safe concurrent access.
    /// Size is `1 << (BLOOM_BITS - 3)` bytes. Fits in L1 cache.
    bloom: Vec<AtomicU8>,
    /// Number of bits in the bloom filter (power of 2).
    bloom_mask: u64,
    /// Verified-positive cache: (origin_hash, channel_hash) -> authorized.
    verified: DashMap<(u32, u16), bool>,
}

/// Bloom filter size: 2^15 bits = 4KB. Fits in L1 cache.
const BLOOM_BITS: u32 = 15;

impl AuthGuard {
    /// Create a new authorization guard.
    pub fn new() -> Self {
        let num_bytes = 1usize << (BLOOM_BITS - 3);
        let bloom = (0..num_bytes).map(|_| AtomicU8::new(0)).collect();
        Self {
            bloom,
            bloom_mask: (1u64 << BLOOM_BITS) - 1,
            verified: DashMap::new(),
        }
    }

    /// Fast-path authorization check.
    ///
    /// Called on every packet by forwarding nodes. Must complete in <10ns.
    #[inline]
    pub fn check_fast(&self, origin_hash: u32, channel_hash: u16) -> AuthVerdict {
        let key = bloom_key(origin_hash, channel_hash);

        // Probe bloom filter with two hash functions
        let h1 = (key & self.bloom_mask) as usize;
        let h2 = ((key >> BLOOM_BITS) & self.bloom_mask) as usize;

        let bit1 = (self.bloom[h1 >> 3].load(Ordering::Relaxed) >> (h1 & 7)) & 1;
        let bit2 = (self.bloom[h2 >> 3].load(Ordering::Relaxed) >> (h2 & 7)) & 1;

        if bit1 == 0 || bit2 == 0 {
            return AuthVerdict::Denied;
        }

        // Bloom hit — check verified cache
        if self.verified.contains_key(&(origin_hash, channel_hash)) {
            AuthVerdict::Allowed
        } else {
            AuthVerdict::NeedsFullCheck
        }
    }

    /// Authorize an (origin_hash, channel_hash) pair.
    ///
    /// Called at subscription time (slow path). Inserts into both the
    /// bloom filter and the verified cache.
    pub fn authorize(&self, origin_hash: u32, channel_hash: u16) {
        let key = bloom_key(origin_hash, channel_hash);

        // Insert into bloom filter
        let h1 = (key & self.bloom_mask) as usize;
        let h2 = ((key >> BLOOM_BITS) & self.bloom_mask) as usize;

        // Safety: bloom is large enough and indices are masked
        self.bloom_set(h1);
        self.bloom_set(h2);

        // Insert into verified cache
        self.verified.insert((origin_hash, channel_hash), true);
    }

    /// Revoke authorization for an (origin_hash, channel_hash) pair.
    ///
    /// Removes from verified cache. The bloom filter is not cleared
    /// (bloom filters don't support deletion), but the verified cache
    /// miss will cause `NeedsFullCheck` which will then fail.
    pub fn revoke(&self, origin_hash: u32, channel_hash: u16) {
        self.verified.remove(&(origin_hash, channel_hash));
    }

    /// Check if a pair is authorized (verified cache only, no bloom).
    pub fn is_authorized(&self, origin_hash: u32, channel_hash: u16) -> bool {
        self.verified.contains_key(&(origin_hash, channel_hash))
    }

    /// Number of authorized pairs in the verified cache.
    pub fn authorized_count(&self) -> usize {
        self.verified.len()
    }

    /// Rebuild the bloom filter from the verified cache.
    ///
    /// Call this after many revocations to clear stale bloom bits.
    /// Requires `&mut self` to prevent concurrent reads during the
    /// clear-then-reinsert window, which would incorrectly deny
    /// authorized traffic.
    pub fn rebuild_bloom(&mut self) {
        // Clear all bits
        for byte in &self.bloom {
            byte.store(0, Ordering::Relaxed);
        }

        // Re-insert all verified pairs
        for entry in self.verified.iter() {
            let (origin_hash, channel_hash) = *entry.key();
            let key = bloom_key(origin_hash, channel_hash);
            let h1 = (key & self.bloom_mask) as usize;
            let h2 = ((key >> BLOOM_BITS) & self.bloom_mask) as usize;
            self.bloom[h1 >> 3].fetch_or(1 << (h1 & 7), Ordering::Relaxed);
            self.bloom[h2 >> 3].fetch_or(1 << (h2 & 7), Ordering::Relaxed);
        }
    }

    /// Set a bit in the bloom filter using atomic fetch_or.
    #[inline]
    fn bloom_set(&self, bit_index: usize) {
        let byte_index = bit_index >> 3;
        let bit_offset = bit_index & 7;
        self.bloom[byte_index].fetch_or(1 << bit_offset, Ordering::Relaxed);
    }
}

impl Default for AuthGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AuthGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthGuard")
            .field("bloom_size_bytes", &self.bloom.len())
            .field("authorized_pairs", &self.verified.len())
            .finish()
    }
}

/// Compute bloom filter key from (origin_hash, channel_hash).
#[inline]
fn bloom_key(origin_hash: u32, channel_hash: u16) -> u64 {
    let mut buf = [0u8; 6];
    buf[0..4].copy_from_slice(&origin_hash.to_le_bytes());
    buf[4..6].copy_from_slice(&channel_hash.to_le_bytes());
    xxh3_64(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_guard_denies() {
        let guard = AuthGuard::new();
        assert_eq!(guard.check_fast(0x1234, 0xABCD), AuthVerdict::Denied);
    }

    #[test]
    fn test_authorize_then_allow() {
        let guard = AuthGuard::new();
        guard.authorize(0x1234, 0xABCD);

        assert_eq!(guard.check_fast(0x1234, 0xABCD), AuthVerdict::Allowed);
    }

    #[test]
    fn test_different_pair_denied() {
        let guard = AuthGuard::new();
        guard.authorize(0x1234, 0xABCD);

        // Different origin
        assert_ne!(guard.check_fast(0x5678, 0xABCD), AuthVerdict::Allowed);
        // Different channel
        assert_ne!(guard.check_fast(0x1234, 0x1111), AuthVerdict::Allowed);
    }

    #[test]
    fn test_revoke() {
        let guard = AuthGuard::new();
        guard.authorize(0x1234, 0xABCD);
        assert_eq!(guard.check_fast(0x1234, 0xABCD), AuthVerdict::Allowed);

        guard.revoke(0x1234, 0xABCD);
        // After revoke, bloom still has the bits but verified cache is empty.
        // Result should be NeedsFullCheck (bloom hit, cache miss).
        assert_eq!(
            guard.check_fast(0x1234, 0xABCD),
            AuthVerdict::NeedsFullCheck
        );
    }

    #[test]
    fn test_rebuild_bloom_after_revoke() {
        let mut guard = AuthGuard::new();
        guard.authorize(0x1234, 0xABCD);
        guard.authorize(0x5678, 0xBEEF);

        guard.revoke(0x1234, 0xABCD);
        guard.rebuild_bloom();

        // Revoked pair should now be Denied (bloom cleared)
        assert_eq!(guard.check_fast(0x1234, 0xABCD), AuthVerdict::Denied);
        // Other pair should still be Allowed
        assert_eq!(guard.check_fast(0x5678, 0xBEEF), AuthVerdict::Allowed);
    }

    #[test]
    fn test_multiple_authorizations() {
        let guard = AuthGuard::new();

        for i in 0..100u32 {
            guard.authorize(i, (i * 7) as u16);
        }

        assert_eq!(guard.authorized_count(), 100);

        for i in 0..100u32 {
            assert_eq!(
                guard.check_fast(i, (i * 7) as u16),
                AuthVerdict::Allowed,
                "pair ({}, {}) should be allowed",
                i,
                i * 7
            );
        }
    }

    #[test]
    fn test_is_authorized() {
        let guard = AuthGuard::new();
        assert!(!guard.is_authorized(0x1234, 0xABCD));

        guard.authorize(0x1234, 0xABCD);
        assert!(guard.is_authorized(0x1234, 0xABCD));

        guard.revoke(0x1234, 0xABCD);
        assert!(!guard.is_authorized(0x1234, 0xABCD));
    }

    #[test]
    fn test_bloom_false_positive_rate() {
        // Insert 1000 pairs, check 10000 random pairs that weren't inserted.
        // False positive rate should be well under 1% for a 4KB filter.
        let guard = AuthGuard::new();

        for i in 0..1000u32 {
            guard.authorize(i, i as u16);
        }

        let mut false_positives = 0;
        for i in 10000..20000u32 {
            let verdict = guard.check_fast(i, i as u16);
            if verdict != AuthVerdict::Denied {
                false_positives += 1;
            }
        }

        let fp_rate = false_positives as f64 / 10000.0;
        assert!(fp_rate < 0.01, "false positive rate {} exceeds 1%", fp_rate);
    }

    // ---- Regression tests for Cubic AI findings ----

    #[test]
    fn test_regression_concurrent_authorize_and_check() {
        // Regression: bloom filter used unsafe raw pointer mutation through
        // &self, causing UB under concurrent access. Now uses AtomicU8.
        use std::sync::Arc;
        use std::thread;

        let guard = Arc::new(AuthGuard::new());

        // Spawn writers
        let mut handles = Vec::new();
        for t in 0..4u32 {
            let g = Arc::clone(&guard);
            handles.push(thread::spawn(move || {
                for i in 0..250u32 {
                    g.authorize(t * 1000 + i, (t * 1000 + i) as u16);
                }
            }));
        }

        // Spawn concurrent readers
        for _ in 0..4 {
            let g = Arc::clone(&guard);
            handles.push(thread::spawn(move || {
                for i in 0..1000u32 {
                    let _ = g.check_fast(i, i as u16);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // All authorized pairs should be findable
        assert_eq!(guard.authorized_count(), 1000);
        for t in 0..4u32 {
            for i in 0..250u32 {
                assert!(
                    guard.is_authorized(t * 1000 + i, (t * 1000 + i) as u16),
                    "pair ({}, {}) should be authorized after concurrent insertion",
                    t * 1000 + i,
                    t * 1000 + i
                );
            }
        }
    }
}
