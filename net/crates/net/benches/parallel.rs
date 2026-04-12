//! Benchmarks for parallel ingestion throughput.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;
use std::sync::Arc;
use std::thread;

use blackstream::config::BackpressureMode;
use blackstream::event::RawEvent;
use blackstream::shard::ShardManager;

/// Benchmark parallel ingestion throughput.
fn bench_parallel_ingestion(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel");

    // Measure total throughput with multiple producer threads
    for num_threads in [1, 2, 4, 8].iter() {
        let manager = Arc::new(ShardManager::new(16, 1 << 20, BackpressureMode::DropOldest));
        let ops_per_iter = 10_000u64;

        group.throughput(Throughput::Elements(ops_per_iter * (*num_threads as u64)));
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let manager = Arc::clone(&manager);
                            thread::spawn(move || {
                                for i in 0..ops_per_iter {
                                    let _ = manager.ingest(json!({"i": i}));
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark shard manager ingestion with different shard counts.
fn bench_shard_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("shard_manager");
    group.throughput(Throughput::Elements(1));

    for num_shards in [1, 4, 8, 16].iter() {
        let manager = ShardManager::new(*num_shards, 1 << 20, BackpressureMode::DropOldest);
        group.bench_with_input(
            BenchmarkId::new("ingest_json", num_shards),
            num_shards,
            |b, _| {
                b.iter(|| {
                    let _ = manager.ingest(json!({"token": "hello", "index": 42}));
                });
            },
        );
    }

    // Benchmark raw event ingestion (pre-serialized)
    for num_shards in [1, 4, 8, 16].iter() {
        let manager = ShardManager::new(*num_shards, 1 << 20, BackpressureMode::DropOldest);
        // Pre-serialize the event once (simulating data arriving as bytes)
        let raw_event = RawEvent::from_str(r#"{"token": "hello", "index": 42}"#);
        group.bench_with_input(
            BenchmarkId::new("ingest_raw", num_shards),
            num_shards,
            |b, _| {
                b.iter(|| {
                    let _ = manager.ingest_raw(raw_event.clone());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark ingestion with varying event sizes - comparing JSON vs Raw paths.
fn bench_event_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_size");
    group.throughput(Throughput::Elements(1));

    let manager = ShardManager::new(4, 1 << 20, BackpressureMode::DropOldest);

    // Small event (~50 bytes) - JSON path
    group.bench_function("small_50b_json", |b| {
        let event = json!({"t": "x"});
        b.iter(|| {
            let _ = manager.ingest(event.clone());
        });
    });

    // Small event (~50 bytes) - Raw path
    group.bench_function("small_50b_raw", |b| {
        let event = RawEvent::from_str(r#"{"t": "x"}"#);
        b.iter(|| {
            let _ = manager.ingest_raw(event.clone());
        });
    });

    // Medium event (~200 bytes) - JSON path
    group.bench_function("medium_200b_json", |b| {
        let event = json!({
            "token": "hello_world",
            "index": 42,
            "session_id": "abc123def456",
            "timestamp": 1234567890,
            "metadata": {"key": "value"}
        });
        b.iter(|| {
            let _ = manager.ingest(event.clone());
        });
    });

    // Medium event (~200 bytes) - Raw path
    group.bench_function("medium_200b_raw", |b| {
        let event = RawEvent::from_str(r#"{"token":"hello_world","index":42,"session_id":"abc123def456","timestamp":1234567890,"metadata":{"key":"value"}}"#);
        b.iter(|| {
            let _ = manager.ingest_raw(event.clone());
        });
    });

    // Large event (~1KB) - JSON path
    let large_json = json!({
        "token": "hello_world_this_is_a_longer_token",
        "index": 42,
        "session_id": "abc123def456ghi789jkl012mno345",
        "timestamp": 1234567890,
        "user_id": "user_12345678901234567890",
        "request_id": "req_abcdefghijklmnopqrstuvwxyz",
        "metadata": {
            "key1": "value1_with_some_extra_data",
            "key2": "value2_with_more_extra_data",
            "key3": "value3_with_even_more_data",
            "nested": {
                "a": 1, "b": 2, "c": 3, "d": 4, "e": 5
            }
        },
        "tags": ["tag1", "tag2", "tag3", "tag4", "tag5"],
        "content": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris."
    });
    let large_raw = RawEvent::from_value(large_json.clone());

    group.bench_function("large_1kb_json", |b| {
        b.iter(|| {
            let _ = manager.ingest(large_json.clone());
        });
    });

    group.bench_function("large_1kb_raw", |b| {
        b.iter(|| {
            let _ = manager.ingest_raw(large_raw.clone());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_shard_manager,
    bench_event_sizes,
    bench_parallel_ingestion,
);

criterion_main!(benches);
