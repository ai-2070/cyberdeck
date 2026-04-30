//! Regression coverage for `bus::remove_shard_internal`'s stranded-
//! ring-buffer flush — BUG #153 (sequence_start collision under
//! JetStream dedup) and BUG #154 (premature finalize-vs-pending-events
//! race).
//!
//! Both bugs land at the same code path: when a shard is being torn
//! down (via `manual_scale_down` or the scaling-monitor's auto path),
//! `remove_shard_internal` constructs a `Batch::new(shard_id,
//! stranded, sequence_start)` to flush whatever events were left in
//! the ring buffer at unmap time.
//!
//! - **#153.** Pre-fix `sequence_start` was hardcoded to `0`. Every
//!   `BatchWorker` for the same shard ALSO starts at sequence 0, so
//!   the stranded batch's per-event msg-ids
//!   (`{nonce}:{shard_id}:0:{i}`) collided with the worker's very
//!   first batch's msg-ids — JetStream's 2 min dedup window dropped
//!   the duplicates, silently losing the stranded events. The fix
//!   reads the worker's final `next_sequence` from a shared atomic
//!   after awaiting its `JoinHandle` and uses that as the
//!   `sequence_start`.
//!
//! - **#154.** Pre-fix `remove_shard_internal` dropped the
//!   `BatchWorker`'s `JoinHandle` without await, so a finalize
//!   triggered while the worker still had `current_batch` events or
//!   in-flight mpsc-channel events would race the worker's own
//!   dispatch against this function's stranded-flush. The fix awaits
//!   the worker first; by the time the stranded batch dispatches,
//!   the worker has flushed everything it had pending under proper
//!   sequencing.
//!
//! Bespoke `RecordingAdapter` records every batch's `(shard_id,
//! sequence_start, len, msg_id_prefix)` so the assertions can
//! distinguish "batch was delivered" from "batch was deduped" and
//! pin both fixes directly.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use net::adapter::{Adapter, ShardPollResult};
use net::bus::EventBus;
use net::config::{EventBusConfig, ScalingPolicy};
use net::error::AdapterError;
use net::event::{Batch, Event};
use parking_lot::Mutex;
use serde_json::json;

/// One observation of an `on_batch` call.
#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchObservation {
    shard_id: u16,
    sequence_start: u64,
    len: usize,
}

/// Adapter that records every batch verbatim and emits a
/// `(shard_id, sequence_start, i)` "msg-id" tuple per event so the
/// test can detect collisions the same way JetStream's dedup window
/// would.
#[derive(Clone)]
struct RecordingAdapter {
    batches: Arc<Mutex<Vec<BatchObservation>>>,
    msg_ids: Arc<Mutex<Vec<(u16, u64, usize)>>>,
}

impl RecordingAdapter {
    fn new() -> (
        Self,
        Arc<Mutex<Vec<BatchObservation>>>,
        Arc<Mutex<Vec<(u16, u64, usize)>>>,
    ) {
        let batches = Arc::new(Mutex::new(Vec::new()));
        let msg_ids = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                batches: batches.clone(),
                msg_ids: msg_ids.clone(),
            },
            batches,
            msg_ids,
        )
    }
}

#[async_trait]
impl Adapter for RecordingAdapter {
    async fn init(&mut self) -> Result<(), AdapterError> {
        Ok(())
    }
    async fn on_batch(&self, batch: Batch) -> Result<(), AdapterError> {
        let shard_id = batch.shard_id;
        let sequence_start = batch.sequence_start;
        let len = batch.len();

        // Mirror the JetStream msg-id construction
        // (`adapter/jetstream.rs:281`): every event becomes a
        // `(shard_id, sequence_start, i)` tuple. If two batches'
        // tuples collide, JetStream's dedup would drop the second.
        // We don't drop here — we record, and assert no duplicates
        // in the test.
        {
            let mut ids = self.msg_ids.lock();
            for i in 0..len {
                ids.push((shard_id, sequence_start, i));
            }
        }
        self.batches.lock().push(BatchObservation {
            shard_id,
            sequence_start,
            len,
        });
        Ok(())
    }
    async fn flush(&self) -> Result<(), AdapterError> {
        Ok(())
    }
    async fn shutdown(&self) -> Result<(), AdapterError> {
        Ok(())
    }
    async fn poll_shard(
        &self,
        _shard_id: u16,
        _from_id: Option<&str>,
        _limit: usize,
    ) -> Result<ShardPollResult, AdapterError> {
        Ok(ShardPollResult::empty())
    }
    fn name(&self) -> &'static str {
        "recording"
    }
}

fn config(num_shards: u16) -> EventBusConfig {
    let policy = ScalingPolicy {
        min_shards: 1,
        max_shards: 16,
        cooldown: Duration::from_nanos(1),
        ..Default::default()
    };
    EventBusConfig::builder()
        .num_shards(num_shards)
        .ring_buffer_capacity(1024)
        .scaling(policy)
        .build()
        .unwrap()
}

/// Pin BUG #153: the stranded-flush batch's `sequence_start` MUST
/// be strictly past every `(shard_id, sequence_start, i)` msg-id
/// the worker emitted. Pre-fix it was hardcoded to 0, colliding
/// with the worker's very first batch.
#[tokio::test]
async fn stranded_flush_does_not_collide_with_worker_msg_ids() {
    let (adapter, batches, msg_ids) = RecordingAdapter::new();
    let bus = EventBus::new_with_adapter(config(2), Box::new(adapter))
        .await
        .unwrap();

    // Scale up to 4, then back down to 2. The 2 removed shards
    // each produce a stranded-flush via `remove_shard_internal`.
    let added = bus.manual_scale_up(2).await.unwrap();
    assert_eq!(added.len(), 2);

    // Push enough events to force several flushes per shard so the
    // worker's `next_sequence` is well past 0 by the time scale-down
    // runs.
    for i in 0..2_000u64 {
        let _ = bus.ingest(Event::new(json!({"i": i})));
    }
    bus.flush().await.unwrap();

    let removed = bus.manual_scale_down(2).await.unwrap();
    assert_eq!(removed.len(), 2);
    bus.shutdown().await.unwrap();

    // Assertion: every `(shard_id, sequence_start, i)` tuple is
    // unique. A pre-fix run would have at least one shard where the
    // stranded batch's msg-ids collide with the worker's first
    // batch's msg-ids — under JetStream dedup those events would be
    // silently dropped.
    let ids = msg_ids.lock().clone();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        ids.len(),
        "duplicate (shard, sequence_start, i) msg-id tuples observed — \
         stranded-flush collided with worker batch (BUG #153). \
         Total batches: {}, total msg-ids: {}, unique: {}",
        batches.lock().len(),
        ids.len(),
        sorted.len(),
    );
}

/// Pin the wire-level invariant directly on the recorded batches:
/// for every shard there's at most one batch with `sequence_start
/// == 0`, even though both the worker's first batch AND the
/// stranded-flush batch *could* both want that slot.
#[tokio::test]
async fn at_most_one_batch_per_shard_uses_sequence_start_zero() {
    let (adapter, batches, _) = RecordingAdapter::new();
    let bus = EventBus::new_with_adapter(config(2), Box::new(adapter))
        .await
        .unwrap();

    let _ = bus.manual_scale_up(2).await.unwrap();
    for i in 0..1_000u64 {
        let _ = bus.ingest(Event::new(json!({"i": i})));
    }
    bus.flush().await.unwrap();

    let _ = bus.manual_scale_down(2).await.unwrap();
    bus.shutdown().await.unwrap();

    let observations = batches.lock().clone();
    // For each shard_id, count batches with sequence_start == 0.
    use std::collections::HashMap;
    let mut zero_starts: HashMap<u16, usize> = HashMap::new();
    for o in &observations {
        if o.sequence_start == 0 {
            *zero_starts.entry(o.shard_id).or_default() += 1;
        }
    }
    for (shard_id, count) in &zero_starts {
        assert!(
            *count <= 1,
            "shard {} produced {} batches with sequence_start=0 — \
             stranded-flush re-used the worker's first-batch sequence, \
             colliding under JetStream dedup (BUG #153). \
             Recorded batches: {:?}",
            shard_id,
            count,
            observations,
        );
    }
}

/// Pin BUG #154: events still in the BatchWorker's mpsc channel or
/// `current_batch` at the moment `remove_shard_internal` is invoked
/// must reach the adapter — they must NOT be silently dropped or
/// race the stranded-flush.
///
/// We force an eager finalize by pushing a small number of events
/// (less than `min_batch_size`), letting the drain worker pump them
/// into the BatchWorker's channel, then triggering scale-down before
/// the BatchWorker's flush timeout fires. Pre-fix the `JoinHandle`
/// would be dropped without await, and the worker's pending
/// `current_batch` would race with the stranded-flush dispatch.
#[tokio::test]
async fn events_in_flight_at_finalize_reach_adapter() {
    let (adapter, _batches, msg_ids) = RecordingAdapter::new();
    let bus = EventBus::new_with_adapter(config(2), Box::new(adapter))
        .await
        .unwrap();

    let added = bus.manual_scale_up(2).await.unwrap();
    assert_eq!(added.len(), 2);

    // Push 100 events but DON'T call flush. The drain worker will
    // pump them into the BatchWorker's channel; the BatchWorker may
    // or may not have flushed (depends on `min_batch_size` /
    // `max_delay`). Either way, scale-down must not lose them.
    const N: u64 = 100;
    for i in 0..N {
        let _ = bus.ingest(Event::new(json!({"i": i})));
    }

    // Tight scale-down — no `flush()` first. With pre-fix code, any
    // events still in the BatchWorker's `current_batch` or in the
    // mpsc channel could be dispatched concurrently with the
    // stranded-flush, racing through dedup.
    let _ = bus.manual_scale_down(2).await.unwrap();
    bus.shutdown().await.unwrap();

    // Every event must show up in the adapter exactly once.
    let total_seen: usize = msg_ids.lock().len();
    assert_eq!(
        total_seen, N as usize,
        "expected exactly {N} events delivered to adapter; got {total_seen}. \
         Events lost between BatchWorker pending state and stranded-flush \
         (BUG #154 race window)",
    );

    // No duplicates either — the same msg-id collision logic from
    // BUG #153 applies if the worker's pending batch and the
    // stranded batch raced through dedup.
    let ids = msg_ids.lock().clone();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        ids.len(),
        "duplicate msg-id tuples observed during in-flight finalize — \
         BatchWorker's pending batch raced with stranded-flush (BUG #153 + #154)",
    );
}

/// Pin the cross-cutting guarantee: a sustained ingest -> repeated
/// scale_up/scale_down cycle delivers every event the producer
/// pushed to the adapter, with no msg-id collisions across cycles.
/// This is the "production-shape" stress for both fixes together.
#[tokio::test]
async fn repeated_scale_cycles_preserve_every_event_with_unique_msg_ids() {
    let (adapter, _batches, msg_ids) = RecordingAdapter::new();
    let bus = EventBus::new_with_adapter(config(2), Box::new(adapter))
        .await
        .unwrap();

    let mut total_ingested = 0u64;
    for cycle in 0..3 {
        let _ = bus.manual_scale_up(1).await.unwrap();
        for i in 0..200u64 {
            if bus
                .ingest(Event::new(json!({"cycle": cycle, "i": i})))
                .is_ok()
            {
                total_ingested += 1;
            }
        }
        bus.flush().await.unwrap();
        let _ = bus.manual_scale_down(1).await.unwrap();
    }
    bus.shutdown().await.unwrap();

    let ids = msg_ids.lock().clone();
    assert_eq!(
        ids.len() as u64,
        total_ingested,
        "{} ingested events; adapter saw {}",
        total_ingested,
        ids.len(),
    );

    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        ids.len(),
        "duplicate msg-id tuples observed across scale cycles",
    );
}
