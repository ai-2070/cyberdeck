//! NAPI wrapper for the consumer-side Redis Streams dedup helper.
//!
//! Thin wrapper around `net_sdk::RedisStreamDedup`. See that
//! module's docs for the dedup contract — briefly: the Redis
//! adapter writes a `dedup_id` field on every XADD entry; this
//! helper filters duplicates by maintaining an LRU-bounded set of
//! seen ids.
//!
//! Wire shape from the JS side:
//!
//! ```js
//! const { RedisStreamDedup } = require('@ai2070/net-sdk');
//!
//! // Default capacity (4096).
//! const dedup = new RedisStreamDedup();
//!
//! // Explicit capacity for higher-throughput consumers.
//! const dedup = new RedisStreamDedup(65_536);
//!
//! // Read entries from your Redis client of choice; pull the
//! // dedup_id field from each entry.
//! for (const entry of stream) {
//!   if (!dedup.isDuplicate(entry.fields.dedup_id)) {
//!     await process(entry);
//!   }
//! }
//! ```
//!
//! `RedisStreamDedup` is NOT thread-safe across NAPI worker
//! threads. JS users typically read a stream from a single async
//! context; if you need cross-worker dedup, instantiate one
//! helper per worker (each has its own LRU) or share state via
//! a Redis-side data structure.

#![allow(dead_code)]

use napi_derive::napi;
use std::sync::Mutex;

/// Consumer-side dedup helper for the Redis Streams adapter.
///
/// See `net::adapter::redis` module docs for the producer-side
/// contract and BUG #57 background.
#[napi]
pub struct RedisStreamDedup {
    // The inner LRU is `!Sync` (it owns mutable state behind a
    // `&mut self` receiver). NAPI exposes `&self` methods for
    // the JS-visible API, so we wrap in a `Mutex` to serialize
    // — which also matches the documented one-helper-per-worker
    // shape (no contention in the common case).
    inner: Mutex<net_sdk::RedisStreamDedup>,
}

#[napi]
impl RedisStreamDedup {
    /// Create a helper with the given LRU capacity. Defaults to
    /// 4096 if omitted. `0` is clamped to 1.
    ///
    /// Sizing: a consumer at ~10k events/sec with a 1 min
    /// dedup window should pick ~600k.
    #[napi(constructor)]
    pub fn new(capacity: Option<u32>) -> Self {
        let inner = match capacity {
            Some(c) => net_sdk::RedisStreamDedup::with_capacity(c as usize),
            None => net_sdk::RedisStreamDedup::new(),
        };
        Self {
            inner: Mutex::new(inner),
        }
    }

    /// Test-and-insert: returns `true` if the caller should treat
    /// the entry as a DUPLICATE (skip it), `false` if it's the
    /// first time we've seen this `dedupId`.
    ///
    /// Matches the Rust `is_duplicate(&mut self, &str) -> bool`.
    #[napi]
    pub fn is_duplicate(&self, dedup_id: String) -> bool {
        // Mutex poisoning would only happen if a previous thread
        // panicked while holding the lock; in that case the
        // helper's state is unknown but the LRU semantics are
        // still safe to continue from. Recover the inner.
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.is_duplicate(&dedup_id)
    }

    /// Number of distinct ids currently tracked.
    #[napi(getter)]
    pub fn len(&self) -> u32 {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // `as u32` is fine: capacity is bounded by the constructor
        // argument which we accept as `u32`, so `len()` can never
        // exceed it.
        guard.len() as u32
    }

    /// Configured maximum capacity.
    #[napi(getter)]
    pub fn capacity(&self) -> u32 {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.capacity() as u32
    }

    /// True if no ids are tracked yet.
    #[napi(getter)]
    pub fn is_empty(&self) -> bool {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.is_empty()
    }

    /// Clear all tracked ids. Use after a consumer-group
    /// rebalance to reset the dedup window without losing the
    /// helper instance.
    #[napi]
    pub fn clear(&self) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test: NAPI surface answers the same way as the Rust
    /// helper for the canonical BUG #57 producer-retry scenario.
    /// Pre-fix consumers had no way to filter — the duplicate
    /// XADDs reached the application; the helper makes the filter
    /// trivial.
    #[test]
    fn napi_helper_filters_duplicates() {
        let dedup = RedisStreamDedup::new(Some(64));

        // First pass: every id is new.
        for i in 0..3 {
            let id = format!("deadbeef:0:0:{i}");
            assert!(!dedup.is_duplicate(id));
        }
        assert_eq!(dedup.len(), 3);
        assert!(!dedup.is_empty());

        // Retry path: every id reappears with the same dedup_id.
        for i in 0..3 {
            let id = format!("deadbeef:0:0:{i}");
            assert!(dedup.is_duplicate(id));
        }
        assert_eq!(dedup.len(), 3); // length unchanged on duplicate hits

        dedup.clear();
        assert_eq!(dedup.len(), 0);
        assert!(dedup.is_empty());
    }

    #[test]
    fn napi_helper_default_capacity() {
        let dedup = RedisStreamDedup::new(None);
        assert_eq!(dedup.capacity(), 4096);
    }

    #[test]
    fn napi_helper_explicit_capacity() {
        let dedup = RedisStreamDedup::new(Some(8192));
        assert_eq!(dedup.capacity(), 8192);
    }

    #[test]
    fn napi_helper_capacity_zero_is_clamped() {
        let dedup = RedisStreamDedup::new(Some(0));
        assert_eq!(dedup.capacity(), 1);
    }
}
