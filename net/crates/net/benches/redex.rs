//! Microbenchmarks for the RedEX storage primitive.
//!
//! Run with: cargo bench --features redex --bench redex
//!            cargo bench --features "redex redex-disk" --bench redex
//!
//! These answer "is the local log the bottleneck?" independently of
//! CortEX / NetDB. The end-to-end story (ingest → fold → query) lives
//! in `benches/cortex.rs`.
//!
//! Measures:
//! - **Append** throughput at inline (≤8 B) and heap (32 / 256 / 1024 B) sizes,
//!   with and without disk durability (the latter gated on `redex-disk`)
//! - **Batch append** throughput, on heap and on disk
//! - **Disk policies** — single and batch appends across every
//!   `FsyncPolicy` variant (`Never`, `EveryN`, `Interval`,
//!   `IntervalOrBytes`). Confirms the appender doesn't pay
//!   fsync cost when the worker absorbs it (Phases 3 + 4).
//! - **Tail latency** — append → subscriber observes the new seq

use std::sync::Arc;

use bytes::Bytes;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::StreamExt;
use net::adapter::net::channel::ChannelName;
#[cfg(feature = "redex-disk")]
use net::adapter::net::redex::FsyncPolicy;
use net::adapter::net::redex::{Redex, RedexFileConfig};
use tokio::runtime::Runtime;

fn rt() -> Arc<Runtime> {
    Arc::new(Runtime::new().expect("tokio runtime"))
}

fn cn(s: &str) -> ChannelName {
    ChannelName::new(s).unwrap()
}

// ============================================================================
// Append — inline vs heap.
//
// Inline is the zero-alloc fast path for ≤8 B payloads (sensor ticks,
// counter bumps). Heap pays one memcpy into the append-only segment
// and proportional bytes through the checksum, so we sweep payload
// size to measure where that cost starts to bite.
// ============================================================================

fn bench_append_inline(c: &mut Criterion) {
    let mut group = c.benchmark_group("redex_append_inline");
    group.throughput(Throughput::Elements(1));

    group.bench_function("heap_file", |b| {
        let r = Redex::new();
        let f = r
            .open_file(&cn("bench/inline/heap"), RedexFileConfig::default())
            .unwrap();
        let payload: [u8; 8] = [0xAB; 8];
        b.iter(|| f.append_inline(&payload).unwrap());
    });

    group.finish();
}

fn bench_append_heap(c: &mut Criterion) {
    let mut group = c.benchmark_group("redex_append_heap");

    for &size in &[32usize, 256, 1024] {
        group.throughput(Throughput::Bytes(size as u64));
        let payload = vec![0xCDu8; size];

        group.bench_with_input(BenchmarkId::new("heap_file", size), &size, |b, &s| {
            let r = Redex::new();
            let f = r
                .open_file(
                    &cn(&format!("bench/heap/mem/{}", s)),
                    RedexFileConfig::default(),
                )
                .unwrap();
            b.iter(|| f.append(&payload).unwrap());
        });
    }

    group.finish();
}

// ============================================================================
// Append: no-watcher fast path vs. with-tail path.
//
// `append` skips the `Bytes::copy_from_slice` event materialization
// when nobody is tailing — most production traffic. The "with_tail"
// variant pre-subscribes (and drains) a tail stream so the copy +
// `notify_watchers` path is exercised. The delta between the two is
// the cost paid solely for live delivery.
// ============================================================================

fn bench_append_watcher_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("redex_append_watcher_paths");
    let runtime = rt();
    let payload = vec![0xCDu8; 256];
    group.throughput(Throughput::Bytes(payload.len() as u64));

    group.bench_function("no_watchers", |b| {
        let r = Redex::new();
        let f = r
            .open_file(&cn("bench/watcher/none"), RedexFileConfig::default())
            .unwrap();
        b.iter(|| f.append(&payload).unwrap());
    });

    group.bench_function("with_tail", |b| {
        let _enter = runtime.enter();
        let r = Redex::new();
        let f = r
            .open_file(&cn("bench/watcher/with"), RedexFileConfig::default())
            .unwrap();
        let mut stream = Box::pin(f.tail(0));
        b.iter(|| {
            f.append(&payload).unwrap();
            // Drain so the bounded buffer never saturates and the
            // benchmark doesn't measure disconnect handling.
            runtime.block_on(async {
                let _ = stream.next().await.unwrap();
            });
        });
    });

    group.finish();
}

// ============================================================================
// Batch append.
//
// Amortizes the seq-allocation and lock overhead across N payloads in
// one call. We benchmark a batch of 64 small heap payloads.
// ============================================================================

fn bench_append_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("redex_append_batch");
    const BATCH: usize = 64;
    group.throughput(Throughput::Elements(BATCH as u64));

    group.bench_function(format!("batch_{}_x_64B", BATCH), |b| {
        let r = Redex::new();
        let f = r
            .open_file(&cn("bench/batch/heap"), RedexFileConfig::default())
            .unwrap();
        let payloads: Vec<Bytes> = (0..BATCH).map(|_| Bytes::from(vec![0xEE; 64])).collect();
        b.iter(|| f.append_batch(&payloads).unwrap());
    });

    group.finish();
}

// ============================================================================
// Disk durability (feature `redex-disk`).
//
// Measures the cost of the disk segment write on the append path.
// Compare these numbers against `redex_append_heap::heap_file` at the
// same payload size to see the overhead of durability.
// ============================================================================

#[cfg(feature = "redex-disk")]
fn bench_append_disk(c: &mut Criterion) {
    let mut group = c.benchmark_group("redex_append_disk");
    let tmp = tempdir_prefix("redex_bench_disk");

    for &size in &[32usize, 256, 1024] {
        group.throughput(Throughput::Bytes(size as u64));
        let payload = vec![0xABu8; size];

        group.bench_with_input(BenchmarkId::new("disk_file", size), &size, |b, &s| {
            let r = Redex::new().with_persistent_dir(&tmp);
            let cfg = RedexFileConfig::default().with_persistent(true);
            // Each bench gets its own channel so appends don't
            // recover stale state between runs.
            let name = cn(&format!("bench/disk/{}/{}", s, rand_suffix()));
            let f = r.open_file(&name, cfg).unwrap();
            b.iter(|| f.append(&payload).unwrap());
        });
    }

    group.finish();
}

// Disk batch append. Targets the buffered-write path in
// `DiskSegment::append_entries_inner`: a batch of N entries should
// emit at most 3 syscalls (one each to dat / idx / ts), not 3·N.
#[cfg(feature = "redex-disk")]
fn bench_append_batch_disk(c: &mut Criterion) {
    let mut group = c.benchmark_group("redex_append_batch_disk");
    let tmp = tempdir_prefix("redex_bench_batch_disk");
    const BATCH: usize = 64;
    group.throughput(Throughput::Elements(BATCH as u64));

    for &size in &[64usize, 1024] {
        group.bench_with_input(
            BenchmarkId::new(format!("batch_{}_x", BATCH), size),
            &size,
            |b, &s| {
                let r = Redex::new().with_persistent_dir(&tmp);
                let cfg = RedexFileConfig::default().with_persistent(true);
                let name = cn(&format!("bench/disk_batch/{}/{}", s, rand_suffix()));
                let f = r.open_file(&name, cfg).unwrap();
                let payloads: Vec<Bytes> = (0..BATCH).map(|_| Bytes::from(vec![0xEE; s])).collect();
                b.iter(|| f.append_batch(&payloads).unwrap());
            },
        );
    }

    group.finish();
}

// Single-append cost across every `FsyncPolicy` variant. The
// non-`Never` policies should track close to `Never` because the
// fsync runs on a background worker (Phases 3 and 4); the
// appender pays only for the page-cache write. A regression that
// re-introduced synchronous fsync on the appender would surface
// here as a 10x–100x latency jump on `every_n_1` / `every_n_64`
// vs `never`.
#[cfg(feature = "redex-disk")]
fn bench_append_disk_policies(c: &mut Criterion) {
    let mut group = c.benchmark_group("redex_append_disk_policies");
    let tmp = tempdir_prefix("redex_bench_policies");
    let runtime = rt();
    let payload = vec![0xABu8; 256];
    group.throughput(Throughput::Bytes(payload.len() as u64));

    let policies: &[(&str, FsyncPolicy)] = &[
        ("never", FsyncPolicy::Never),
        ("every_n_1", FsyncPolicy::EveryN(1)),
        ("every_n_64", FsyncPolicy::EveryN(64)),
        (
            "interval_50ms",
            FsyncPolicy::Interval(std::time::Duration::from_millis(50)),
        ),
        (
            "interval_or_bytes",
            FsyncPolicy::IntervalOrBytes {
                period: std::time::Duration::from_millis(50),
                max_bytes: 1024 * 1024,
            },
        ),
    ];

    for (name, policy) in policies {
        group.bench_with_input(BenchmarkId::new("disk_file_256B", name), policy, |b, p| {
            // Workers spawn at file-open time, so we need a
            // tokio runtime in scope. The bench loop itself is
            // synchronous — appends don't await.
            let _enter = runtime.enter();
            let r = Redex::new().with_persistent_dir(&tmp);
            let cfg = RedexFileConfig::default()
                .with_persistent(true)
                .with_fsync_policy(*p);
            let chan = cn(&format!("bench/policies/{}/{}", name, rand_suffix()));
            let f = r.open_file(&chan, cfg).unwrap();
            b.iter(|| f.append(&payload).unwrap());
        });
    }

    group.finish();
}

// Batch-append cost across every `FsyncPolicy` variant. Combines
// Phase 1 (syscall coalescing) with Phase 3 / 4 (worker offload).
// `BATCH=64` × 64 B at the policy that fires fastest (`every_n_1`)
// is the most adversarial case — each batch crosses the cadence
// AND notifies the worker. Should still track `never` closely.
#[cfg(feature = "redex-disk")]
fn bench_append_batch_disk_policies(c: &mut Criterion) {
    let mut group = c.benchmark_group("redex_append_batch_disk_policies");
    let tmp = tempdir_prefix("redex_bench_batch_policies");
    let runtime = rt();
    const BATCH: usize = 64;
    group.throughput(Throughput::Elements(BATCH as u64));

    let policies: &[(&str, FsyncPolicy)] = &[
        ("never", FsyncPolicy::Never),
        ("every_n_1", FsyncPolicy::EveryN(1)),
        (
            "interval_or_bytes_small",
            // Small max_bytes so the byte arm fires every batch.
            FsyncPolicy::IntervalOrBytes {
                period: std::time::Duration::from_secs(60),
                max_bytes: 1024,
            },
        ),
    ];

    for (name, policy) in policies {
        group.bench_with_input(BenchmarkId::new("batch_64_x_64B", name), policy, |b, p| {
            let _enter = runtime.enter();
            let r = Redex::new().with_persistent_dir(&tmp);
            let cfg = RedexFileConfig::default()
                .with_persistent(true)
                .with_fsync_policy(*p);
            let chan = cn(&format!("bench/batch_policies/{}/{}", name, rand_suffix()));
            let f = r.open_file(&chan, cfg).unwrap();
            let payloads: Vec<Bytes> = (0..BATCH).map(|_| Bytes::from(vec![0xEE; 64])).collect();
            b.iter(|| f.append_batch(&payloads).unwrap());
        });
    }

    group.finish();
}

#[cfg(feature = "redex-disk")]
fn tempdir_prefix(prefix: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("{}_{}", prefix, rand_suffix()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(feature = "redex-disk")]
fn rand_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

// ============================================================================
// Tail latency — append → subscriber observes the new event.
//
// Pre-subscribes a tail stream, measures the time from `append()`
// producing a new seq to `stream.next().await` returning it. This is
// the read-after-write path without the CortEX fold on top.
// ============================================================================

fn bench_tail_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("redex_tail");
    group.throughput(Throughput::Elements(1));
    let runtime = rt();

    group.bench_function("append_to_next", |b| {
        let _enter = runtime.enter();
        let r = Redex::new();
        let f = r
            .open_file(&cn("bench/tail/latency"), RedexFileConfig::default())
            .unwrap();
        let mut stream = Box::pin(f.tail(0));
        let payload = vec![0u8; 32];

        b.iter(|| {
            f.append(&payload).unwrap();
            runtime.block_on(async {
                let _ = stream.next().await.unwrap();
            });
        });
    });

    group.finish();
}

#[cfg(feature = "redex-disk")]
criterion_group!(
    benches,
    bench_append_inline,
    bench_append_heap,
    bench_append_watcher_paths,
    bench_append_batch,
    bench_append_disk,
    bench_append_batch_disk,
    bench_append_disk_policies,
    bench_append_batch_disk_policies,
    bench_tail_latency,
);

#[cfg(not(feature = "redex-disk"))]
criterion_group!(
    benches,
    bench_append_inline,
    bench_append_heap,
    bench_append_watcher_paths,
    bench_append_batch,
    bench_tail_latency,
);

criterion_main!(benches);
