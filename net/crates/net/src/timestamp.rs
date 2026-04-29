//! High-precision timestamp generation with zero syscall overhead.
//!
//! This module provides monotonically increasing timestamps using the CPU's
//! Time Stamp Counter (TSC) on x86_64, avoiding syscalls in the hot path.
//!
//! # Design
//!
//! - Uses `quanta` crate which calibrates against the system clock once at startup
//! - Subsequent reads use RDTSC instruction directly (no syscall)
//! - Each shard has its own `TimestampGenerator` to eliminate contention
//! - Monotonicity is guaranteed via atomic CAS operations

use std::sync::atomic::{AtomicU64, Ordering};

/// High-precision timestamp generator using TSC.
///
/// This generator provides strictly monotonic timestamps with sub-nanosecond
/// resolution and zero syscall overhead after initialization.
///
/// # Single-owner invariant
///
/// **Each producer should own a dedicated `TimestampGenerator`.** The
/// type is `Send + Sync` and `next()` *is* safe to call concurrently —
/// monotonicity is preserved by `compare_exchange_weak` — but the CAS
/// loop degenerates into a spin under sustained contention. The whole
/// design rests on the loop almost never iterating: that's only true
/// when one writer at a time accesses the generator.
///
/// The codebase enforces this structurally rather than at runtime:
///
/// - `Shard` owns its `TimestampGenerator` by value (not behind `Arc`).
/// - `TimestampGenerator` is **not** `Clone`, so duplicating one is a
///   deliberate `mem::replace` / `Default::default()` away — visible
///   in code review.
/// - The shard's surrounding `Mutex<Shard>` serializes producers, so
///   `next()` is invoked by exactly one caller at a time per shard.
///
/// If you find yourself reaching for `Arc<TimestampGenerator>`, stop
/// — give each producer its own instance instead. Every additional
/// concurrent caller is one more thread potentially CAS-spinning on
/// `last`.
pub struct TimestampGenerator {
    /// quanta clock (TSC-based after calibration).
    clock: quanta::Clock,
    /// Last generated timestamp (for monotonicity).
    last: AtomicU64,
}

impl TimestampGenerator {
    /// Create a new timestamp generator.
    ///
    /// This performs a one-time calibration against the system clock.
    /// Subsequent timestamp reads use TSC directly.
    pub fn new() -> Self {
        Self {
            clock: quanta::Clock::new(),
            last: AtomicU64::new(0),
        }
    }

    /// Generate the next timestamp.
    ///
    /// Returns a strictly monotonically increasing value (nanoseconds).
    /// This operation is lock-free and does not invoke any syscalls.
    ///
    /// # Performance
    ///
    /// - Single-threaded: ~5-10ns per call
    /// - Under contention: may loop due to CAS, but still lock-free
    #[inline(always)]
    pub fn next(&self) -> u64 {
        // Read TSC (no syscall)
        let now = self.clock.raw();

        // Ensure strict monotonicity via CAS loop
        loop {
            let last = self.last.load(Ordering::Acquire);

            // Guard against u64::MAX exhaustion: saturating_add(1) at MAX
            // would return MAX again, breaking strict monotonicity.
            let next = match last.checked_add(1) {
                Some(inc) => inc,
                None => panic!("TimestampGenerator: timestamp space exhausted (u64::MAX)"),
            };
            let ts = now.max(next);

            match self
                .last
                .compare_exchange_weak(last, ts, Ordering::Release, Ordering::Relaxed)
            {
                Ok(_) => return ts,
                Err(_) => {
                    // Another thread updated; retry
                    std::hint::spin_loop();
                }
            }
        }
    }

    /// Get the current raw timestamp without incrementing.
    ///
    /// This does NOT guarantee monotonicity and is only useful for
    /// measuring elapsed time or debugging.
    #[inline(always)]
    pub fn now_raw(&self) -> u64 {
        self.clock.raw()
    }

    /// Convert a raw timestamp to approximate nanoseconds since epoch.
    ///
    /// Note: This is approximate and should only be used for debugging
    /// or logging, not for ordering guarantees.
    #[inline]
    pub fn raw_to_nanos(&self, raw: u64) -> u64 {
        self.clock.delta_as_nanos(0, raw)
    }

    /// Get the last generated timestamp.
    #[inline]
    pub fn last(&self) -> u64 {
        self.last.load(Ordering::Acquire)
    }
}

impl Default for TimestampGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_monotonicity() {
        let ts_gen = TimestampGenerator::new();
        let mut prev = 0u64;

        for _ in 0..10_000 {
            let ts = ts_gen.next();
            assert!(ts > prev, "timestamps must be strictly increasing");
            prev = ts;
        }
    }

    #[test]
    fn test_monotonicity_concurrent() {
        let ts_gen = std::sync::Arc::new(TimestampGenerator::new());
        let mut handles = vec![];

        for _ in 0..4 {
            let ts_gen_clone = ts_gen.clone();
            handles.push(thread::spawn(move || {
                let mut timestamps = Vec::with_capacity(1000);
                for _ in 0..1000 {
                    timestamps.push(ts_gen_clone.next());
                }
                timestamps
            }));
        }

        let mut all_timestamps: Vec<u64> = handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect();

        // All timestamps should be unique (strictly monotonic)
        all_timestamps.sort();
        let unique_count = all_timestamps.windows(2).filter(|w| w[0] != w[1]).count() + 1;
        assert_eq!(
            unique_count,
            all_timestamps.len(),
            "all timestamps must be unique"
        );
    }

    #[test]
    fn test_no_syscall_performance() {
        let ts_gen = TimestampGenerator::new();

        // Warm up
        for _ in 0..1000 {
            let _ = ts_gen.next();
        }

        // Measure
        let start = std::time::Instant::now();
        let iterations = 100_000;

        for _ in 0..iterations {
            let _ = ts_gen.next();
        }

        let elapsed = start.elapsed();
        let per_call = elapsed.as_nanos() / iterations as u128;

        // Typically 5-20ns on bare metal, but CI runners can be slower.
        assert!(
            per_call < 500,
            "timestamp generation too slow: {}ns per call",
            per_call
        );
    }

    #[test]
    fn test_timestamp_generator_new() {
        let ts_gen = TimestampGenerator::new();
        // Initial last should be 0
        assert_eq!(ts_gen.last(), 0);
    }

    #[test]
    fn test_timestamp_generator_default() {
        let ts_gen = TimestampGenerator::default();
        assert_eq!(ts_gen.last(), 0);
    }

    #[test]
    fn test_now_raw() {
        let ts_gen = TimestampGenerator::new();
        let raw1 = ts_gen.now_raw();
        let raw2 = ts_gen.now_raw();
        // Raw timestamps should be increasing (or at least not decreasing significantly)
        assert!(raw2 >= raw1 || raw1 - raw2 < 1000); // Allow for some jitter
    }

    #[test]
    fn test_raw_to_nanos() {
        let ts_gen = TimestampGenerator::new();
        let raw = ts_gen.now_raw();
        let nanos = ts_gen.raw_to_nanos(raw);
        // Nanos should be a reasonable value (not zero for a non-zero raw)
        assert!(nanos > 0);
    }

    #[test]
    fn test_raw_to_nanos_zero() {
        let ts_gen = TimestampGenerator::new();
        let nanos = ts_gen.raw_to_nanos(0);
        assert_eq!(nanos, 0);
    }

    #[test]
    fn test_last_after_next() {
        let ts_gen = TimestampGenerator::new();
        let ts1 = ts_gen.next();
        assert_eq!(ts_gen.last(), ts1);

        let ts2 = ts_gen.next();
        assert_eq!(ts_gen.last(), ts2);
        assert!(ts2 > ts1);
    }

    #[test]
    fn test_next_returns_increasing_values() {
        let ts_gen = TimestampGenerator::new();
        let mut prev = ts_gen.next();

        for _ in 0..100 {
            let current = ts_gen.next();
            assert!(current > prev);
            prev = current;
        }
    }

    #[test]
    fn test_multiple_generators_independent() {
        let ts_gen1 = TimestampGenerator::new();
        let ts_gen2 = TimestampGenerator::new();

        let ts1 = ts_gen1.next();
        let ts2 = ts_gen2.next();

        // Both should have advanced
        assert!(ts1 > 0);
        assert!(ts2 > 0);

        // They are independent, so last values are different
        assert_eq!(ts_gen1.last(), ts1);
        assert_eq!(ts_gen2.last(), ts2);
    }

    #[test]
    fn test_now_raw_does_not_affect_last() {
        let ts_gen = TimestampGenerator::new();
        let initial_last = ts_gen.last();

        // Call now_raw multiple times
        let _ = ts_gen.now_raw();
        let _ = ts_gen.now_raw();
        let _ = ts_gen.now_raw();

        // last should not have changed
        assert_eq!(ts_gen.last(), initial_last);
    }

    #[test]
    fn test_rapid_calls() {
        let ts_gen = TimestampGenerator::new();
        let mut timestamps = Vec::with_capacity(10000);

        for _ in 0..10000 {
            timestamps.push(ts_gen.next());
        }

        // All should be unique and strictly increasing
        for window in timestamps.windows(2) {
            assert!(window[1] > window[0]);
        }
    }

    // Regression: saturating_add(1) at u64::MAX used to silently return
    // the same timestamp twice, breaking strict monotonicity (BUGS_3 #6).
    #[test]
    #[should_panic(expected = "timestamp space exhausted")]
    fn test_next_panics_at_u64_max() {
        let ts_gen = TimestampGenerator::new();
        // Force last to u64::MAX
        ts_gen.last.store(u64::MAX, Ordering::Release);
        let _ = ts_gen.next();
    }

    #[test]
    fn test_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TimestampGenerator>();
    }
}
