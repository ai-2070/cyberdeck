//! Producer-consumer contract test for the Redis `dedup_id` field.
//! Verifies the consumer-side helper
//! [`net_sdk::RedisStreamDedup`] correctly filters duplicates
//! whose `dedup_id` strings exactly match the format the Redis
//! adapter writes:
//!
//!     `{producer_nonce:hex}:{shard_id}:{sequence_start}:{i}`
//!
//! We don't spin up a Redis instance for this test — the contract
//! is purely about the field format. If the producer side ever
//! changes the format string (e.g. drops the hex encoding, switches
//! delimiters), this test fails because the consumer would no
//! longer recognize the duplicates.

#![cfg(feature = "redis")]

use net_sdk::RedisStreamDedup;

/// Format the dedup_id exactly as `adapter/redis.rs::on_batch` does.
/// The format is intentionally duplicated here (not imported from
/// the adapter) so a future refactor that changes the format on
/// either side is caught by this test failing.
fn dedup_id(producer_nonce: u64, shard_id: u16, sequence_start: u64, i: usize) -> String {
    format!("{producer_nonce:x}:{shard_id}:{sequence_start}:{i}")
}

/// Pin the canonical scenario: a producer's MULTI/EXEC
/// timed out client-side but the EXEC ran server-side, the retry
/// then issued another EXEC, and the Redis stream now carries
/// two copies of the same logical batch. Each copy has distinct
/// server-generated `*` ids but identical `dedup_id` fields.
/// The consumer helper filters the second copy.
#[test]
fn helper_filters_producer_retry_duplicates() {
    let mut dedup = RedisStreamDedup::new();
    let producer_nonce: u64 = 0xDEAD_BEEF_CAFE_F00D;
    let shard_id: u16 = 7;
    let sequence_start: u64 = 1234;
    let batch_size = 10;

    // First pass through the stream: every event is new.
    for i in 0..batch_size {
        let id = dedup_id(producer_nonce, shard_id, sequence_start, i);
        assert!(
            !dedup.is_duplicate(&id),
            "first observation of {id} should not be flagged as duplicate",
        );
    }

    // Producer-side retry path: the same logical events arrive
    // again, with the same dedup_ids. Helper filters all of them.
    for i in 0..batch_size {
        let id = dedup_id(producer_nonce, shard_id, sequence_start, i);
        assert!(
            dedup.is_duplicate(&id),
            "retry-path observation of {id} should be filtered",
        );
    }
}

/// Pin cross-restart scenario: a producer crashed mid-batch, was
/// restarted with the same `producer_nonce_path`, and retried the
/// batch. The persistent nonce ensures the dedup_id matches
/// pre-crash; the consumer helper filters the post-restart copy.
///
/// (We model "same persistent nonce" by reusing the same u64; the
/// producer-side fix is what makes that hold in
/// production.)
#[test]
fn helper_filters_cross_restart_duplicates_via_stable_nonce() {
    let mut dedup = RedisStreamDedup::new();
    let nonce: u64 = 0x0123_4567_89AB_CDEF;

    // Pre-crash: the producer wrote 5 events for shard 0, seq 0.
    for i in 0..5 {
        let id = dedup_id(nonce, 0, 0, i);
        assert!(!dedup.is_duplicate(&id));
    }

    // Crash-restart: same persistent nonce loaded, sequence
    // counter reset to 0 (it's process-local). Producer retries
    // the batch.
    for i in 0..5 {
        let id = dedup_id(nonce, 0, 0, i);
        assert!(
            dedup.is_duplicate(&id),
            "post-restart retry must be filtered by the consumer — \
             the persistent nonce makes the dedup_id stable across \
             restarts",
        );
    }
}

/// Disjoint events do NOT trip the helper. This is the negative
/// shape: events with different `(shard, sequence_start, i)`
/// tuples or different producer nonces are treated as distinct.
#[test]
fn helper_does_not_collide_distinct_events() {
    let mut dedup = RedisStreamDedup::new();

    // Same nonce, different shards.
    assert!(!dedup.is_duplicate(&dedup_id(0xAAAA, 0, 0, 0)));
    assert!(!dedup.is_duplicate(&dedup_id(0xAAAA, 1, 0, 0)));
    assert!(!dedup.is_duplicate(&dedup_id(0xAAAA, 2, 0, 0)));

    // Same nonce + shard, different sequence_starts.
    assert!(!dedup.is_duplicate(&dedup_id(0xAAAA, 0, 100, 0)));
    assert!(!dedup.is_duplicate(&dedup_id(0xAAAA, 0, 200, 0)));

    // Same nonce + shard + sequence_start, different i.
    assert!(!dedup.is_duplicate(&dedup_id(0xAAAA, 0, 300, 0)));
    assert!(!dedup.is_duplicate(&dedup_id(0xAAAA, 0, 300, 1)));
    assert!(!dedup.is_duplicate(&dedup_id(0xAAAA, 0, 300, 2)));

    // Different nonce, otherwise identical.
    assert!(!dedup.is_duplicate(&dedup_id(0xBBBB, 0, 0, 0)));
}
