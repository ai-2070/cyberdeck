//! Wire-speed authorization guard for Net packets.
//!
//! The `AuthGuard` uses a bloom filter to authorize packets in under 10ns.
//! Authorized `(origin_hash, channel_hash)` pairs are inserted at subscription
//! time (slow path). The per-packet fast path probes the bloom filter with
//! no crypto, no heap allocation, and no pointer chasing.
//!
//! # Two-tier authorization
//!
//! The guard keeps two parallel ACLs:
//!
//! - **Fast path** (`check_fast`, `authorize`, `is_authorized`): keyed
//!   on the 16-bit `channel_hash` that rides the Net header. Used by
//!   the packet data plane. Collisions are tolerable here because
//!   AEAD still enforces payload integrity end-to-end — a bloom false
//!   positive at most costs a full check further up the stack.
//! - **Exact path** (`allow_channel`, `revoke_channel`,
//!   `is_authorized_full`): keyed on the **canonical `ChannelName`**
//!   itself, not any hash. Used by control-plane and storage
//!   decisions (e.g. `Redex::open_file`) where a hash collision
//!   would let one channel's ACL authorize access to a different
//!   channel's file. `xxh3_64` is non-cryptographic (~2^32 ops to
//!   birthday-collide, feasible offline), so a hash-keyed ACL would
//!   be crackable by an attacker who can influence the name passed
//!   to `open_file`. Keying on the canonical string eliminates the
//!   hash layer entirely — two distinct names can never alias.
//!
//! `allow_channel` populates both tiers so a caller granted storage
//! access can also continue sending packets on that channel.

use std::sync::atomic::{AtomicU8, Ordering};

use dashmap::DashMap;
use xxhash_rust::xxh3::xxh3_64;

use super::ChannelName;

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
    ///
    /// `origin_hash` is a 64-bit subscriber projection — typically the
    /// full `node_id` — rather than a 32-bit truncation. A 32-bit key
    /// births-collides at ~65 k peers (√2^32), inside the practical
    /// reach of a medium-sized mesh; 64 bits pushes the collision
    /// floor to ~4 billion peers, which is no longer a plausible
    /// operating point.
    verified: DashMap<(u64, u16), bool>,
    /// Exact-identity ACL: `(origin_hash, canonical ChannelName) ->
    /// authorized`. Keys on the name string (not a hash) so that no
    /// two distinct channels can alias through a hash collision —
    /// this is the control-plane / storage authorization path.
    /// `ChannelName` already implements `Hash + Eq` against its
    /// underlying validated `String`, so DashMap keys on the exact
    /// name comparison.
    exact: DashMap<(u64, ChannelName), ()>,
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
            exact: DashMap::new(),
        }
    }

    /// Fast-path authorization check.
    ///
    /// Called on every packet by forwarding nodes. Must complete in <10ns.
    #[inline]
    pub fn check_fast(&self, origin_hash: u64, channel_hash: u16) -> AuthVerdict {
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
    pub fn authorize(&self, origin_hash: u64, channel_hash: u16) {
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
    pub fn revoke(&self, origin_hash: u64, channel_hash: u16) {
        self.verified.remove(&(origin_hash, channel_hash));
    }

    /// Check if a pair is authorized (verified cache only, no bloom).
    ///
    /// This is the fast-path check used by the packet data plane.
    /// For control-plane / storage decisions, use
    /// [`Self::is_authorized_full`] — the 16-bit `channel_hash` alone
    /// is collision-prone at mesh scale and must not be trusted as
    /// an ACL for non-data-plane decisions.
    pub fn is_authorized(&self, origin_hash: u64, channel_hash: u16) -> bool {
        self.verified.contains_key(&(origin_hash, channel_hash))
    }

    /// Grant `origin_hash` full (control-plane) access to `name`.
    ///
    /// Populates both ACL tiers:
    /// - the exact canonical-name ACL that control-plane / storage
    ///   callers must consult via [`Self::is_authorized_full`];
    /// - the fast-path bloom + verified cache, so the same origin
    ///   can continue sending packets on that channel via
    ///   [`Self::check_fast`] / [`Self::is_authorized`].
    pub fn allow_channel(&self, origin_hash: u64, name: &ChannelName) {
        self.exact.insert((origin_hash, name.clone()), ());
        self.authorize(origin_hash, name.hash());
    }

    /// Revoke `origin_hash`'s full access to `name`.
    ///
    /// Removes from both the exact ACL and the fast-path verified
    /// cache. Bloom bits are not cleared (bloom filters don't support
    /// deletion), so the fast path may transition to
    /// [`AuthVerdict::NeedsFullCheck`] for this pair — the exact-map
    /// miss then fails the full check.
    pub fn revoke_channel(&self, origin_hash: u64, name: &ChannelName) {
        self.exact.remove(&(origin_hash, name.clone()));
        self.revoke(origin_hash, name.hash());
    }

    /// Exact authorization check keyed on the canonical `ChannelName`
    /// string. Used by control-plane / storage decisions
    /// (e.g. `Redex::open_file`). Unlike [`Self::is_authorized`],
    /// this cannot be bypassed by a hash collision between two
    /// different channel names — two distinct canonical names can
    /// never alias.
    pub fn is_authorized_full(&self, origin_hash: u64, name: &ChannelName) -> bool {
        self.exact.contains_key(&(origin_hash, name.clone()))
    }

    /// Number of authorized pairs in the verified cache.
    pub fn authorized_count(&self) -> usize {
        self.verified.len()
    }

    /// Number of (origin, channel) pairs with exact (control-plane)
    /// authorization.
    pub fn exact_authorized_count(&self) -> usize {
        self.exact.len()
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
            .field("exact_authorized_pairs", &self.exact.len())
            .finish()
    }
}

/// Compute bloom filter key from (origin_hash, channel_hash).
#[inline]
fn bloom_key(origin_hash: u64, channel_hash: u16) -> u64 {
    let mut buf = [0u8; 10];
    buf[0..8].copy_from_slice(&origin_hash.to_le_bytes());
    buf[8..10].copy_from_slice(&channel_hash.to_le_bytes());
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

        for i in 0..100u64 {
            guard.authorize(i, (i * 7) as u16);
        }

        assert_eq!(guard.authorized_count(), 100);

        for i in 0..100u64 {
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

        for i in 0..1000u64 {
            guard.authorize(i, i as u16);
        }

        let mut false_positives = 0;
        for i in 10000..20000u64 {
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
    fn test_regression_u64_origin_hash_defeats_32bit_collision() {
        // Regression: before this fix the guard keyed on `u32`, so two
        // distinct `node_id`s sharing the low 32 bits were
        // indistinguishable — the first subscriber's grant admitted the
        // second's packets. Birthday collision is plausible at ~65 k
        // peers. Widening to `u64` pushes the floor out of reach.
        let guard = AuthGuard::new();

        let name = ChannelName::new("regression-u64-origin").unwrap();
        let legit: u64 = 0x0000_ABCD_1234_5678;
        let forged: u64 = 0xFFFF_FFFF_1234_5678; // same low 32, different high
        assert_eq!(legit as u32, forged as u32);
        assert_ne!(legit, forged);

        guard.allow_channel(legit, &name);

        // Legit subscriber is admitted.
        assert_eq!(
            guard.check_fast(legit, name.hash()),
            AuthVerdict::Allowed,
            "legit subscriber must be admitted"
        );
        assert!(guard.is_authorized_full(legit, &name));

        // Forged subscriber (sharing only the low 32 bits) is rejected.
        assert_ne!(
            guard.check_fast(forged, name.hash()),
            AuthVerdict::Allowed,
            "forged subscriber must not ride the legit grant"
        );
        assert!(!guard.is_authorized_full(forged, &name));
    }

    #[test]
    fn test_regression_channel_hash_collision_distinguishable_by_exact_name() {
        // Regression: `check_fast` alone is keyed on the 16-bit
        // channel_hash, which collides often enough at mesh scale to
        // let one channel's subscription authorize another. The exact
        // ACL on the canonical `ChannelName` is the intended backstop —
        // this test asserts two colliding names never alias there.
        let guard = AuthGuard::new();

        // Construct two distinct names whose `ChannelName::hash()`
        // happens to collide. We brute-force because the hash is
        // xxh3_64 truncated to 16 bits — collisions are cheap to find.
        let base = "regression/coll-";
        let mut name_a: Option<ChannelName> = None;
        let mut name_b: Option<ChannelName> = None;
        'outer: for i in 0..200_000u32 {
            let cand = ChannelName::new(&format!("{base}{i}")).unwrap();
            if name_a.is_none() {
                name_a = Some(cand);
                continue;
            }
            if cand.hash() == name_a.as_ref().unwrap().hash()
                && cand.as_str() != name_a.as_ref().unwrap().as_str()
            {
                name_b = Some(cand);
                break 'outer;
            }
        }
        let name_a = name_a.expect("seeded name");
        let name_b = name_b
            .expect("two distinct ChannelNames with the same 16-bit hash — widen the search range");
        assert_eq!(name_a.hash(), name_b.hash());
        assert_ne!(name_a.as_str(), name_b.as_str());

        let origin: u64 = 0xDEAD_BEEF_CAFE_F00D;
        guard.allow_channel(origin, &name_a);

        // Fast-path collision: check_fast says Allowed for B because
        // it only sees the 16-bit hash.
        assert_eq!(
            guard.check_fast(origin, name_b.hash()),
            AuthVerdict::Allowed
        );

        // Exact check distinguishes them — this is what callers must
        // consult before trusting the fast-path verdict for any
        // authorization decision that survives past the AEAD backstop.
        assert!(guard.is_authorized_full(origin, &name_a));
        assert!(!guard.is_authorized_full(origin, &name_b));
    }

    #[test]
    fn test_regression_concurrent_authorize_and_check() {
        // Regression: bloom filter used unsafe raw pointer mutation through
        // &self, causing UB under concurrent access. Now uses AtomicU8.
        use std::sync::Arc;
        use std::thread;

        let guard = Arc::new(AuthGuard::new());

        // Spawn writers
        let mut handles = Vec::new();
        for t in 0..4u64 {
            let g = Arc::clone(&guard);
            handles.push(thread::spawn(move || {
                for i in 0..250u64 {
                    g.authorize(t * 1000 + i, (t * 1000 + i) as u16);
                }
            }));
        }

        // Spawn concurrent readers
        for _ in 0..4 {
            let g = Arc::clone(&guard);
            handles.push(thread::spawn(move || {
                for i in 0..1000u64 {
                    let _ = g.check_fast(i, i as u16);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // All authorized pairs should be findable
        assert_eq!(guard.authorized_count(), 1000);
        for t in 0..4u64 {
            for i in 0..250u64 {
                assert!(
                    guard.is_authorized(t * 1000 + i, (t * 1000 + i) as u16),
                    "pair ({}, {}) should be authorized after concurrent insertion",
                    t * 1000 + i,
                    t * 1000 + i
                );
            }
        }
    }

    // ========================================================================
    // TEST_COVERAGE_PLAN §P2-8 — concurrent authorize + revoke on the
    // same (origin, channel) key.
    //
    // The existing regression above stresses writer + reader races on
    // disjoint keys. Subscribe / unsubscribe on the SAME pair is the
    // harder case: authorize sets the bloom bits + inserts the
    // verified entry, revoke removes the verified entry. Bloom bits
    // never clear, so the only read-observable state is the verified
    // map. A torn interleaving could leave the map in either state
    // depending on last-writer-wins, but it must not panic, must not
    // leak a half-inserted entry, and `is_authorized` / `check_fast`
    // must never observe a half-committed state.
    // ========================================================================

    /// Authorize + revoke on the SAME key racing across N threads.
    /// The final `is_authorized` state depends on the
    /// last-writer-wins interleaving, but the map must end in a
    /// coherent state (either entry present or absent, never
    /// corrupted) and no panic along the way.
    #[test]
    fn concurrent_authorize_and_revoke_on_same_key_is_panic_free() {
        use std::sync::Arc;
        use std::thread;

        let guard = Arc::new(AuthGuard::new());
        let origin = 0x1234_5678_9ABC_DEF0u64;
        let channel = 0x4242u16;
        let iters = 1_000u32;

        let authorizer = {
            let guard = guard.clone();
            thread::spawn(move || {
                for _ in 0..iters {
                    guard.authorize(origin, channel);
                }
            })
        };
        let revoker = {
            let guard = guard.clone();
            thread::spawn(move || {
                for _ in 0..iters {
                    guard.revoke(origin, channel);
                }
            })
        };
        // Observer: constantly check the auth state. Every
        // observation must return a bool, never panic, and the
        // internal DashMap state must remain self-consistent
        // (covered by the other assertions after join).
        let observer = {
            let guard = guard.clone();
            thread::spawn(move || {
                for _ in 0..iters {
                    let _ = guard.is_authorized(origin, channel);
                    let _ = guard.check_fast(origin, channel);
                }
            })
        };

        authorizer.join().expect("authorizer panicked");
        revoker.join().expect("revoker panicked");
        observer.join().expect("observer panicked");

        // Final state is SOME boolean — either "last op was
        // authorize → entry present" or "last op was revoke →
        // entry absent". Both are legitimate; asserting that
        // the state is NOT torn means the two calls round-trip.
        let final_state = guard.is_authorized(origin, channel);
        // Double-query to ensure read stability.
        assert_eq!(
            final_state,
            guard.is_authorized(origin, channel),
            "two sequential is_authorized calls must agree — \
             torn read would indicate DashMap corruption",
        );
        // And authorized_count must equal exactly 0 or 1 — no
        // phantom entries, no duplicates.
        let count = guard.authorized_count();
        assert!(
            count == 0 || count == 1,
            "authorized_count should be 0 or 1 after the race; got {count}",
        );
    }

    /// Control-plane variant: `allow_channel` + `revoke_channel`
    /// race on the same `(origin, ChannelName)` entry. Same
    /// invariants — panic-free, coherent final state — applied
    /// to the exact-match ACL that storage / control-plane
    /// paths consult via `is_authorized_full`.
    #[test]
    fn concurrent_allow_and_revoke_channel_on_same_key_is_panic_free() {
        use std::sync::Arc;
        use std::thread;

        let guard = Arc::new(AuthGuard::new());
        let origin = 0xDEAD_BEEF_FEED_CAFEu64;
        let name = ChannelName::new("auth/contended").expect("channel name");
        let iters = 1_000u32;

        let allower = {
            let guard = guard.clone();
            let name = name.clone();
            thread::spawn(move || {
                for _ in 0..iters {
                    guard.allow_channel(origin, &name);
                }
            })
        };
        let revoker = {
            let guard = guard.clone();
            let name = name.clone();
            thread::spawn(move || {
                for _ in 0..iters {
                    guard.revoke_channel(origin, &name);
                }
            })
        };
        let observer = {
            let guard = guard.clone();
            let name = name.clone();
            thread::spawn(move || {
                for _ in 0..iters {
                    let _ = guard.is_authorized_full(origin, &name);
                }
            })
        };

        allower.join().expect("allower panicked");
        revoker.join().expect("revoker panicked");
        observer.join().expect("observer panicked");

        // Coherent terminal state — true or false, never torn.
        let final_state = guard.is_authorized_full(origin, &name);
        assert_eq!(
            final_state,
            guard.is_authorized_full(origin, &name),
            "sequential is_authorized_full reads must agree",
        );
    }
}
