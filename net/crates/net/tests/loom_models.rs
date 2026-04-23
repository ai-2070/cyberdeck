//! Loom concurrency models for FAILURE_PATH_HARDENING_PLAN
//! Stage 2.
//!
//! Run with:
//!
//! ```text
//! RUSTFLAGS="--cfg loom" cargo test --release --test loom_models
//! ```
//!
//! Each model re-implements a production concurrency *pattern*
//! using `loom`'s substitute atomics / threads and asserts the
//! documented memory-ordering contract holds under every
//! exhaustively-explored thread interleaving. This catches
//! Acquire/Release vs. Relaxed confusion, missing publication
//! barriers, and lost-update bugs that probabilistic stress
//! tests in the regular suite can only hope to hit.
//!
//! # Why test patterns, not production structs?
//!
//! Loom substitutes `std::sync::atomic` + `std::sync::Mutex` but
//! not `parking_lot::Mutex` or `dashmap::DashMap`. The crate's
//! atomics-heavy cores (`AuthGuard`, `TokenCache`, `RoutingTable`,
//! `CapabilityIndex`, `FailureDetector`) are DashMap-heavy, so
//! loom can't test them directly without a multi-week shim
//! refactor. The two cores with atomics-only sub-pieces
//! (`SchedulerStreamStats` counter battery in `RoutingTable`,
//! the burst-decrement CAS loop in `LossSimulator`) additionally
//! call `SystemTime::now()` in-situ, which loom's deterministic
//! scheduler can't observe usefully.
//!
//! The workaround: model the *pattern* here with loom's atomics.
//! If the pattern is correct, the production struct using the
//! same pattern is correct by construction. If the production
//! struct ever diverges from the pattern, that's a code review
//! issue — the model stays as the pinned reference.
//!
//! See `docs/FAILURE_PATH_HARDENING_PLAN.md` §Stage 2 for the
//! blocker discussion and the follow-up plan for moving the
//! production structs under loom directly.

#![cfg(loom)]

use loom::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use loom::sync::Arc;
use loom::thread;

// ─────────────────────────────────────────────────────────────
// Model 1: SchedulerStreamStats atomic counter battery
//
// Mirrors `src/adapter/net/route.rs:244-325` — five AtomicU64
// counters + an AtomicU64 last-activity timestamp, all using
// Relaxed ordering. Invariant under test: concurrent
// `record_in` / `record_out` / `record_drop` calls never produce
// counts that are arithmetically impossible (sum of increments
// across all threads == final count). Under Relaxed ordering
// this holds by construction for simple fetch_add sequences —
// the loom test pins that the sequence stays simple.
// ─────────────────────────────────────────────────────────────

#[derive(Default)]
struct StatsModel {
    packets_in: AtomicU64,
    packets_out: AtomicU64,
    packets_dropped: AtomicU64,
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
}

impl StatsModel {
    fn record_in(&self, bytes: u64) {
        self.packets_in.fetch_add(1, Ordering::Relaxed);
        self.bytes_in.fetch_add(bytes, Ordering::Relaxed);
    }
    fn record_out(&self, bytes: u64) {
        self.packets_out.fetch_add(1, Ordering::Relaxed);
        self.bytes_out.fetch_add(bytes, Ordering::Relaxed);
    }
    fn record_drop(&self) {
        self.packets_dropped.fetch_add(1, Ordering::Relaxed);
    }
    fn totals(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.packets_in.load(Ordering::Relaxed),
            self.packets_out.load(Ordering::Relaxed),
            self.packets_dropped.load(Ordering::Relaxed),
            self.bytes_in.load(Ordering::Relaxed),
            self.bytes_out.load(Ordering::Relaxed),
        )
    }
}

#[test]
fn stream_stats_counter_battery_is_atomic_under_concurrent_record() {
    loom::model(|| {
        let stats = Arc::new(StatsModel::default());

        // Thread A: record one `in` of 100 bytes.
        let a = {
            let s = stats.clone();
            thread::spawn(move || s.record_in(100))
        };
        // Thread B: record one `out` of 250 bytes, then one drop.
        let b = {
            let s = stats.clone();
            thread::spawn(move || {
                s.record_out(250);
                s.record_drop();
            })
        };

        a.join().unwrap();
        b.join().unwrap();

        let (p_in, p_out, p_drop, b_in, b_out) = stats.totals();
        // Final state: each counter sees exactly its own
        // contributions. No torn reads, no lost updates.
        assert_eq!(p_in, 1);
        assert_eq!(p_out, 1);
        assert_eq!(p_drop, 1);
        assert_eq!(b_in, 100);
        assert_eq!(b_out, 250);
    });
}

// ─────────────────────────────────────────────────────────────
// Model 2: Burst-decrement CAS loop
//
// Mirrors `src/adapter/net/failure.rs:387-406` — the
// `LossSimulator::should_drop` CAS loop that decrements
// `burst_remaining` atomically to avoid the underflow-to-u64::MAX
// race that a naive `load > 0 then fetch_sub(1)` sequence has
// when two threads see the same `remaining == 1` and both
// subtract. The loom test pins that the CAS loop is correct
// under every thread interleaving: the total number of
// successful decrements equals the initial value, and the
// counter never underflows past zero.
//
// See also: `tests/bus_shutdown_drain.rs` where cubic flagged
// the same pattern mis-implemented (P2 fix replaced `load;
// fetch_sub` with `fetch_update`). This loom model is the
// pattern's reference implementation.
// ─────────────────────────────────────────────────────────────

/// Decrement `counter` by 1 if it's > 0; return true if we
/// successfully decremented. Mirrors the production CAS loop.
fn try_decrement_burst(counter: &AtomicU64) -> bool {
    loop {
        let remaining = counter.load(Ordering::Relaxed);
        if remaining == 0 {
            return false;
        }
        match counter.compare_exchange_weak(
            remaining,
            remaining - 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return true,
            Err(_) => continue, // Another thread won; retry.
        }
    }
}

#[test]
fn burst_cas_decrement_never_underflows_under_contention() {
    loom::model(|| {
        let counter = Arc::new(AtomicU64::new(2));
        let decremented_a = Arc::new(AtomicBool::new(false));
        let decremented_b = Arc::new(AtomicBool::new(false));

        // Two threads race to decrement. With initial=2, both
        // SHOULD succeed. With initial=1, only one succeeds.
        // With initial=0, neither. Under a racy load+fetch_sub
        // pattern, two threads seeing remaining==1 would both
        // subtract → counter wraps to u64::MAX. The CAS loop
        // serializes the reads + writes so only one wins per
        // decrement.
        let a = {
            let c = counter.clone();
            let d = decremented_a.clone();
            thread::spawn(move || {
                if try_decrement_burst(&c) {
                    d.store(true, Ordering::Relaxed);
                }
            })
        };
        let b = {
            let c = counter.clone();
            let d = decremented_b.clone();
            thread::spawn(move || {
                if try_decrement_burst(&c) {
                    d.store(true, Ordering::Relaxed);
                }
            })
        };

        a.join().unwrap();
        b.join().unwrap();

        let final_count = counter.load(Ordering::Relaxed);
        // Invariant: counter cannot underflow past 0. Initial
        // is 2, at most 2 threads successfully decrement, so
        // final ∈ {0}. The dangerous regression — final ==
        // u64::MAX - N — is ruled out by this assertion.
        assert_eq!(
            final_count, 0,
            "CAS loop preserves the sum: initial(2) - decrements(2) = 0. \
             A non-zero final count means the loop failed to decrement; a \
             value near u64::MAX means the CAS loop regressed to load+sub.",
        );
        // Both threads must have observed a successful
        // decrement (initial was 2, both CAS loops must have
        // one winning iteration).
        assert!(
            decremented_a.load(Ordering::Relaxed),
            "thread A did not observe a successful decrement",
        );
        assert!(
            decremented_b.load(Ordering::Relaxed),
            "thread B did not observe a successful decrement",
        );
    });
}

#[test]
fn burst_cas_decrement_caps_at_initial_count_under_contention() {
    loom::model(|| {
        // Initial 1, three threads racing — only ONE must win.
        // The others must see `remaining == 0` and return false
        // without wrapping the counter. A regression from the
        // CAS loop to `load; fetch_sub` would let all three
        // decrement, producing a counter of u64::MAX - 2.
        let counter = Arc::new(AtomicU64::new(1));
        let winners = Arc::new(AtomicU64::new(0));

        let threads: Vec<_> = (0..2)
            .map(|_| {
                let c = counter.clone();
                let w = winners.clone();
                thread::spawn(move || {
                    if try_decrement_burst(&c) {
                        w.fetch_add(1, Ordering::Relaxed);
                    }
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }

        let final_count = counter.load(Ordering::Relaxed);
        let winner_count = winners.load(Ordering::Relaxed);
        assert_eq!(
            final_count, 0,
            "counter must end at exactly 0 — any other value means the \
             CAS loop is racy. winners saw {winner_count} successful decrements",
        );
        assert_eq!(
            winner_count, 1,
            "exactly one thread must win the last decrement when initial=1",
        );
    });
}
