//! PyO3 wrapper for the consumer-side Redis Streams dedup helper.
//!
//! Thin wrapper around `net_sdk::RedisStreamDedup`. See that
//! module's docs for the dedup contract — the Redis adapter writes
//! a `dedup_id` field on every XADD entry; this helper filters
//! duplicates by maintaining an LRU-bounded set of seen ids.
//!
//! Wire shape from Python:
//!
//! ```python
//! from net import RedisStreamDedup
//!
//! # Default capacity (4096).
//! dedup = RedisStreamDedup()
//!
//! # Explicit capacity for higher-throughput consumers.
//! dedup = RedisStreamDedup(capacity=65536)
//!
//! # Read entries from your Redis client of choice; pull the
//! # `dedup_id` field from each entry.
//! for entry in stream:
//!     if not dedup.is_duplicate(entry["dedup_id"]):
//!         await process(entry)
//! ```
//!
//! `RedisStreamDedup` is NOT thread-safe across Python threads.
//! Python users typically read a stream from a single async
//! context; if you need cross-thread dedup, instantiate one
//! helper per thread.

use pyo3::prelude::*;
use std::sync::Mutex;

/// Consumer-side dedup helper for the Redis Streams adapter.
///
/// See `net::adapter::redis` module docs for the producer-side
/// contract and BUG #57 background.
#[pyclass(name = "RedisStreamDedup")]
pub struct PyRedisStreamDedup {
    // The inner LRU is `!Sync` (it owns mutable state behind a
    // `&mut self` receiver). PyO3 exposes `&self` methods for
    // the Python-visible API, so we wrap in a `Mutex` to
    // serialize — which also matches the documented one-helper-
    // per-thread shape (no contention in the common case).
    inner: Mutex<net_sdk::RedisStreamDedup>,
}

#[pymethods]
impl PyRedisStreamDedup {
    /// Create a helper. `capacity` defaults to 4096 if omitted;
    /// `0` is clamped to `1`.
    ///
    /// Sizing: a consumer at ~10k events/sec with a 1 min
    /// dedup window should pick ~600,000.
    #[new]
    #[pyo3(signature = (capacity=None))]
    fn new(capacity: Option<usize>) -> Self {
        let inner = match capacity {
            Some(c) => net_sdk::RedisStreamDedup::with_capacity(c),
            None => net_sdk::RedisStreamDedup::new(),
        };
        Self {
            inner: Mutex::new(inner),
        }
    }

    /// Test-and-insert: returns `True` if the caller should treat
    /// the entry as a DUPLICATE (skip it), `False` if it's the
    /// first time we've seen this `dedup_id`.
    ///
    /// Maps to the Rust `is_duplicate(&mut self, &str) -> bool`.
    fn is_duplicate(&self, dedup_id: &str) -> bool {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.is_duplicate(dedup_id)
    }

    /// Number of distinct ids currently tracked.
    #[getter]
    fn len(&self) -> usize {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.len()
    }

    /// Configured maximum capacity.
    #[getter]
    fn capacity(&self) -> usize {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.capacity()
    }

    /// True if no ids are tracked yet.
    #[getter]
    fn is_empty(&self) -> bool {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.is_empty()
    }

    /// Clear all tracked ids. Use after a consumer-group
    /// rebalance to reset the dedup window without losing the
    /// helper instance.
    fn clear(&self) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.clear();
    }

    fn __repr__(&self) -> String {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        format!(
            "RedisStreamDedup(len={}, capacity={})",
            guard.len(),
            guard.capacity(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test: PyO3 surface answers the same way as the Rust
    /// helper for the canonical BUG #57 producer-retry scenario.
    #[test]
    fn pyo3_helper_filters_duplicates() {
        let dedup = PyRedisStreamDedup::new(Some(64));

        // First pass: every id is new.
        for i in 0..3 {
            let id = format!("deadbeef:0:0:{i}");
            assert!(!dedup.is_duplicate(&id));
        }
        assert_eq!(dedup.len(), 3);
        assert!(!dedup.is_empty());

        // Retry path: every id reappears with the same dedup_id.
        for i in 0..3 {
            let id = format!("deadbeef:0:0:{i}");
            assert!(dedup.is_duplicate(&id));
        }
        assert_eq!(dedup.len(), 3); // length unchanged on duplicate hits

        dedup.clear();
        assert_eq!(dedup.len(), 0);
        assert!(dedup.is_empty());
    }

    #[test]
    fn pyo3_helper_default_capacity() {
        let dedup = PyRedisStreamDedup::new(None);
        assert_eq!(dedup.capacity(), 4096);
    }

    #[test]
    fn pyo3_helper_explicit_capacity() {
        let dedup = PyRedisStreamDedup::new(Some(8192));
        assert_eq!(dedup.capacity(), 8192);
    }

    #[test]
    fn pyo3_helper_capacity_zero_is_clamped() {
        let dedup = PyRedisStreamDedup::new(Some(0));
        assert_eq!(dedup.capacity(), 1);
    }
}
