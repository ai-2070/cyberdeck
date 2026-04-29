//! Benchmarks for event ingestion throughput.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;

use net::config::BackpressureMode;
use net::event::{InternalEvent, RawEvent};
use net::shard::ShardManager;
use net::timestamp::TimestampGenerator;

/// Benchmark shard ingest/drain through the public API.
///
/// Replaces a previous bench against the raw `RingBuffer` type. That
/// type is now `pub(crate)` (BUG_REPORT.md #5), so the next-cleanest
/// proxy is `ShardManager`, which is what real ingestion paths use.
/// The numbers therefore include the per-shard atomic counter
/// updates and the `Mutex<Shard>` acquire/release — i.e. the actual
/// hot-path overhead, not just the lock-free ring atomics.
fn bench_shard(c: &mut Criterion) {
    let mut group = c.benchmark_group("shard");

    // Single shard so the hash routing is deterministic and the
    // bench measures the push/pop hot path rather than hashing.
    for capacity in [1024, 8192, 65536, 1_048_576].iter() {
        group.throughput(Throughput::Elements(1));

        // Pre-built `RawEvent` so each iteration measures only ingest,
        // not JSON construction.
        let raw_template = RawEvent::from_str(r#"{"i":0}"#);

        group.bench_with_input(
            BenchmarkId::new("ingest_raw", capacity),
            capacity,
            |b, &cap| {
                let manager = ShardManager::new(1, cap, BackpressureMode::DropOldest);
                b.iter(|| {
                    let _ = manager.ingest_raw(raw_template.clone());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("ingest_raw_pop", capacity),
            capacity,
            |b, &cap| {
                let manager = ShardManager::new(1, cap, BackpressureMode::DropNewest);
                b.iter(|| {
                    let _ = manager.ingest_raw(raw_template.clone());
                    // Pop one to make room for the next push.
                    let _ = manager.with_shard(0, |s| s.try_pop());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark timestamp generation.
fn bench_timestamp(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamp");
    group.throughput(Throughput::Elements(1));

    let ts_gen = TimestampGenerator::new();

    group.bench_function("next", |b| {
        b.iter(|| ts_gen.next());
    });

    group.bench_function("now_raw", |b| {
        b.iter(|| ts_gen.now_raw());
    });

    group.finish();
}

/// Benchmark event creation and serialization.
fn bench_event_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("event");
    group.throughput(Throughput::Elements(1));

    let ts_gen = TimestampGenerator::new();

    group.bench_function("internal_event_new", |b| {
        b.iter(|| {
            InternalEvent::from_value(json!({"token": "hello", "index": 42}), ts_gen.next(), 0)
        });
    });

    group.bench_function("json_creation", |b| {
        b.iter(|| json!({"token": "hello", "index": 42}));
    });

    group.finish();
}

/// Benchmark batch operations.
///
/// Steady-state pop-after-refill: every iteration pops `size`
/// elements then immediately re-pushes the same count to keep the
/// buffer at its target depth. The number we report is therefore
/// *not* a pure pop cost — it includes the refill and the
/// partial-pop branch. That's intentional for tracking real
/// workloads (consumers that drain into the same ring), but call
/// it out so future readers don't compare it against an
/// isolated-pop benchmark.
fn bench_batch_pop(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch");

    let raw_template = RawEvent::from_value(json!({"i": 0}));

    for batch_size in [100, 1000, 10000].iter() {
        let manager = ShardManager::new(1, 1 << 20, BackpressureMode::DropNewest);

        // Pre-fill the shard once so the first iteration starts in
        // steady state.
        for _ in 0..(*batch_size * 10) {
            let _ = manager.ingest_raw(raw_template.clone());
        }

        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("pop_batch_steady_state", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let batch = manager
                        .with_shard(0, |s| s.pop_batch(size))
                        .unwrap_or_default();
                    // Refill what we popped to maintain depth.
                    let popped = batch.len();
                    for _ in 0..popped {
                        let _ = manager.ingest_raw(raw_template.clone());
                    }
                    batch
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_shard,
    bench_timestamp,
    bench_event_creation,
    bench_batch_pop,
);

criterion_main!(benches);
