//! Benchmarks for event ingestion throughput.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;

use net::event::InternalEvent;
use net::shard::RingBuffer;
use net::timestamp::TimestampGenerator;

/// Benchmark ring buffer push/pop operations.
fn bench_ring_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer");

    for size in [1024, 8192, 65536, 1048576].iter() {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(BenchmarkId::new("push", size), size, |b, &size| {
            let buffer: RingBuffer<u64> = RingBuffer::new(size);
            let mut i = 0u64;
            b.iter(|| {
                // Push until full, then pop to make room
                if buffer.try_push(i).is_err() {
                    let _ = buffer.try_pop();
                    let _ = buffer.try_push(i);
                }
                i = i.wrapping_add(1);
            });
        });

        group.bench_with_input(BenchmarkId::new("push_pop", size), size, |b, &size| {
            let buffer: RingBuffer<u64> = RingBuffer::new(size);
            let mut i = 0u64;
            b.iter(|| {
                let _ = buffer.try_push(i);
                let _ = buffer.try_pop();
                i = i.wrapping_add(1);
            });
        });
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
fn bench_batch_pop(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch");

    for batch_size in [100, 1000, 10000].iter() {
        let buffer: RingBuffer<u64> = RingBuffer::new(1 << 20);

        // Pre-fill buffer
        for i in 0..(*batch_size * 10) {
            let _ = buffer.try_push(i as u64);
        }

        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("pop_batch", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let batch = buffer.pop_batch(size);
                    // Refill what we popped
                    for i in 0..batch.len() {
                        let _ = buffer.try_push(i as u64);
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
    bench_ring_buffer,
    bench_timestamp,
    bench_event_creation,
    bench_batch_pop,
);

criterion_main!(benches);
