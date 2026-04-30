# RedEX Disk — Throughput Plan

> Companion to [`REDEX_PLAN.md`](REDEX_PLAN.md). Targets the persistent-segment hot path under `net/crates/net/src/adapter/net/redex/disk.rs`. Out of scope: anything in [`REDEX_V2_PLAN.md`](REDEX_V2_PLAN.md) (warm tier, mmap, replication).

## Status

Design only. No code changes yet.

## Goals

1. **Batch appends** — group many serialized entries into a single `write()` per file (`dat`, `idx`, `ts`); amortize syscalls and let the kernel coalesce I/O. Plays well with `O_APPEND`.
2. **Strictly sequential writes** — keep `dat` append-only; keep `idx` append-only; defer compaction/merge to background tasks. No seeks on the hot path.
3. **Minimize fsyncs on the hot path** — use `FsyncPolicy` (`Never` / `Interval` for high-throughput, `EveryN` for tighter durability) and move fsyncs to a background task that flushes when **either** a time interval passes **or** buffered bytes exceed a threshold.

## Current state (baseline)

- `disk.rs::append_entries_inner` (line 583) holds each file's `Mutex<File>` and loops `write_all` per entry. A batch of N events emits **3 · N** syscalls (one per entry per file) instead of 3.
- `disk.rs::append_entry_inner` issues 3 sequential `write_all` calls per logical event (one per file).
- `dat`, `idx`, `ts` are already append-only on the hot path. The only `set_len` calls are rollback paths after a write error and the recovery-time tail-truncation in `DiskSegment::open`.
- `maybe_sync_after_append` (line 370) calls `self.sync()` **synchronously on the appender thread** when `EveryN` fires.
- `FsyncPolicy::Interval(d)` is driven by a per-file tokio task spawned in `file.rs` that wakes every `d` and calls `disk.sync()`. There is no byte-threshold trigger.
- `HeapSegment::append` (`segment.rs:66`) takes one payload at a time. `file.rs::append_batch` (`file.rs:441`) writes one batched syscall to disk via `append_entries_at`, then loops over the heap side per entry.

---

## Phase 1 — Coalesce batch syscalls into one `write_all` per file

**Touches:** `disk.rs::append_entries_inner` only.

Rewrite the per-entry loops to build one contiguous buffer per file in a single pass, then issue one `write_all` per file:

```rust
let total_payload: usize = entries_and_payloads.iter()
    .filter(|(e, _)| !e.is_inline())
    .map(|(_, p)| p.len()).sum();
let mut dat_buf = Vec::with_capacity(total_payload);
let mut idx_buf = Vec::with_capacity(entries_and_payloads.len() * REDEX_ENTRY_SIZE);
let mut ts_buf  = Vec::with_capacity(timestamps.len() * 8);
for ((e, p), t) in entries_and_payloads.iter().zip(timestamps) {
    if !e.is_inline() { dat_buf.extend_from_slice(p); }
    idx_buf.extend_from_slice(&e.to_bytes());
    ts_buf.extend_from_slice(&t.to_le_bytes());
}
// dat.write_all(&dat_buf), idx.write_all(&idx_buf), ts.write_all(&ts_buf)
```

Constraints to preserve:

- **Write order:** `dat → idx → ts`. The recovery torn-tail logic in `DiskSegment::open` (lines 142–186, 207–212) depends on dat being durable before idx/ts; do not reorder.
- **Rollback discipline:** capture `pre_len` per file, `set_len` back on error. Whether `write_all` is one syscall or many, the partial-write hazard is the same — a single `write_all` can still be partially flushed if the kernel returns short.
- **Per-batch lock acquisition order:** `dat → idx → ts` (matches today's pattern; do not introduce overlapping locks).

**Win:** `append_batch(64)` drops from 192 → 3 syscalls.
**Risk:** None functionally; pure refactor.
**Test/bench:**
- Existing `test_disk_append_and_recover`, `test_external_dat_truncation_*`, and `append_failure_after_dat_write_rolls_back_dat` cover correctness.
- Add `bench_append_batch_disk` to `net/crates/net/benches/redex.rs` (mirror `bench_append_batch` at line 131 with `feature = "redex-disk"`).

---

## Phase 2 — Heap-side `append_many`

**Touches:** `segment.rs` (new API), `file.rs::append_batch` (line 450), `file.rs::append_batch_ordered`.

After Phase 1, disk-side batching is one syscall per file. The heap-side loop in `file.rs:450–457` is now the bottleneck for large batches.

Add to `HeapSegment`:

```rust
pub fn append_many<'a>(
    &mut self,
    payloads: impl IntoIterator<Item = &'a [u8]>,
) -> Result<(), RedexError>
```

One `reserve` for the total bytes, one fused `extend` pass. `append` becomes a thin wrapper around `append_many(std::iter::once(payload))`.

Update `append_batch` and `append_batch_ordered` to call `append_many` once per batch instead of looping. Watcher notification (the second loop in `append_batch`) stays per-event — it's unrelated to the segment write.

**Risk:** Low; mechanical.
**Tests:** existing batch tests cover; add a microbench comparing `append_many(N)` vs `for _ in 0..N { append() }`.

---

## Phase 3 — Move `EveryN` fsync off the appender thread

**Touches:** `disk.rs::DiskSegment` (new field), `disk.rs::maybe_sync_after_append`, `disk.rs::DiskSegment::open` (spawn task), `disk.rs` close path, `config.rs` rustdoc on `FsyncPolicy::EveryN`.

Today an `EveryN(N)` policy blocks every Nth appender for the duration of three `fsync_all` syscalls — milliseconds on rotational disks, hundreds of µs on NVMe. The page-cache write is already done; the appender's caller doesn't gain anything from waiting on the fsync.

Plumbing:

- Add `fsync_signal: Option<Arc<Notify>>` and `fsync_shutdown: Option<Arc<Notify>>` to `DiskSegment`.
- `maybe_sync_after_append` does `signal.notify_one()` instead of calling `sync()` inline. Returns immediately.
- Background task spawned at `DiskSegment::open` time when `fsync_every_n > 0`:
  ```
  loop {
      tokio::select! {
          _ = signal.notified() => { let _ = self.sync(); }
          _ = shutdown.notified() => break,
      }
  }
  ```
  Single in-flight fsync; if multiple appenders signal during one fsync, the next iteration coalesces them — `Notify` is a permit, not a counter, which is exactly the semantics we want.
- `close()` fires `shutdown.notify_one()` and joins the task.

Document the timing change in `config.rs:29` (`FsyncPolicy::EveryN`):

> Fsync is enqueued for a background worker, not run synchronously on the appender. Worst-case loss bound stays at (N − 1) entries since the last sync **point**, plus the (small) window of an in-flight fsync that the crash interrupts.

**Risk:**
- `sync_count` (test-only at line 100) is incremented inside `sync()`, which the background task still calls — semantics preserved.
- A burst of appends followed by an immediate process kill could lose the trailing batch even at small N. The bound is `(N - 1) + in_flight_window` rather than `N - 1`. Document; do not pretend otherwise.

**Tests:**
- New `everyn_does_not_block_appender`: arm a slow-fsync mock, assert `append_entry` returns within microseconds at N=1.
- New `everyn_coalesces_under_burst`: hammer 10k appends at N=1, assert observed `sync_count` is bounded by what the worker could complete in wall time (not 10k).
- Existing tests that assert `sync_count == k` after a known number of appends need a `flush_pending_syncs()` test helper that drives the worker to quiescence first.

---

## Phase 4 — Byte-threshold trigger for `Interval`

**Touches:** `config.rs` (new variant), `disk.rs` (counter), `file.rs` (background task select).

Add `bytes_since_sync: AtomicU64` to `DiskSegment`:

- Bump by `idx_buf.len() + dat_buf.len() + ts_buf.len()` after a successful append (single or batch).
- Reset to 0 inside `sync()` after the fsyncs succeed.

Extend the policy additively (no breaking change to existing `Interval(d)` callers):

```rust
pub enum FsyncPolicy {
    Never,
    EveryN(u64),
    Interval(Duration),
    /// Fsync when **either** `period` elapses OR `max_bytes` of writes
    /// have accumulated since the last sync, whichever comes first.
    IntervalOrBytes { period: Duration, max_bytes: u64 },
}
```

In the Interval task in `file.rs` (currently a `sleep(period)` loop), replace with a `select!` over a periodic timer and a short polling sleep that checks `bytes_since_sync >= max_bytes` (relaxed load; cheap). Bytes-threshold accuracy is upper-bound, not exact — that's fine; the contract is "at most `max_bytes` of unsynced data."

**Risk:** None for existing `Interval(d)` callers (unchanged variant).
**Tests:**
- `interval_or_bytes_fires_on_byte_threshold`: tiny period + small max_bytes, hammer appends, observe sync count tracks bytes.
- `interval_or_bytes_fires_on_period`: large max_bytes, idle → tick, observe one sync per period.

---

## Phase 5 — Single-append coalescer (speculative, gated)

**Touches:** `file.rs::append`, `file.rs::append_inline`, `disk.rs` (writer task + queue), `config.rs` (new flag).

For workloads with many concurrent **single** appends (no batching at the API level), the disk-side mutex on `dat_file` / `idx_file` / `ts_file` serializes writers. A small per-`DiskSegment` queue can fold concurrent singles into one batched write:

- Each `append_entry_at` enqueues `(entry, payload, ts, oneshot::Sender<Result<()>>)`.
- A dedicated writer task drains up to `K` items (or up to a max-buffered-bytes cap), calls Phase-1-batched `append_entries_inner`, fans out the result to each waiter.

Behavioral changes that matter:

- **Per-call latency goes up** for the un-contended path (one channel hop + scheduler tick) — typically 1–10 µs added.
- The **failure-atomicity contract** in `file.rs:299–305` runs on the appender thread today; with coalescing the disk write happens elsewhere, and the seq-rollback path needs to handle a remote error. Either: (a) appender awaits the oneshot and rolls back seq itself on error, or (b) the writer task signals the file layer to roll back the contiguous seq range. Sketch in design before coding.
- Watcher notify is already off the disk path; unaffected.

Recommendation: build it **gated** behind `RedexFileConfig::coalesce_appends: bool` (default `false`). Ship Phase 1–4 first; flip Phase 5 on only after a bench shows it wins for the target workload.

**Tests:**
- Concurrency stress: 64 tasks × 10k single appends, assert all seqs land contiguously and durably.
- Error fan-out: arm a one-shot disk failure mid-batch, assert every waiter in that batch sees `Err` and seq rolls back exactly.

---

## Cross-cutting: invariants to assert in code review

These are constraints today's code already obeys; future patches must not regress:

1. **No seeks on the hot path.** All append handles use `OpenOptions::new().append(true)`. `set_len` only fires on rollback or recovery, both off the happy path.
2. **Write order is `dat → idx → ts`.** Recovery's torn-tail logic depends on this; reordering is a silent corruption risk.
3. **Lock acquisition order is `dat → idx → ts`.** Acquiring out of order risks deadlock when a future change holds two simultaneously.
4. **`close()` and explicit `RedexFile::sync()` always fsync, regardless of policy.** Phases 3–5 move *append-path* fsyncs around; the explicit barriers stay synchronous.

Add these to the module rustdoc in `disk.rs` so they are visible at the top of the file.

---

## Validation methodology

Add to `net/crates/net/benches/redex.rs`:

- `bench_append_single_disk` — already exists at line 157; keep as baseline.
- `bench_append_batch_disk` — Phase 1 target. 64 × 64 B and 64 × 1 KiB.
- `bench_append_single_disk_concurrent` — N tasks × M appends, varying N. Phase 5 target.
- `bench_fsync_p99_everyn_n1` — measures appender p99 at `EveryN(1)`. Phase 3 target.
- `bench_interval_or_bytes_under_burst` — Phase 4 target.

Capture before/after numbers per phase in the PR description. Expected:

- **Phase 1:** single-digit-multiplier win on `bench_append_batch_disk` (3× to 10× depending on payload size; smaller payloads benefit more from syscall amortization).
- **Phase 2:** modest win (1.2–2×) on the same bench; more important as a correctness/clarity improvement.
- **Phase 3:** p99 single-append latency drops by the fsync duration. p50 unchanged on already-fast disks; large drop on slow disks.
- **Phase 4:** no throughput change; durability bound becomes "either time or bytes," giving operators a knob for bursty workloads.
- **Phase 5:** wins only above some concurrency threshold (likely 8+ concurrent single-appenders); under-contended workloads should not regress because the flag defaults off.

Re-run the full disk recovery test suite after each phase:

```
cargo test -p net --features redex-disk --test redex_disk -- --nocapture
```

---

## Out of scope

- **Compaction off the state lock.** `sweep_retention` holds the file-state lock across `compact_to`. Real win to address, but it's a retention problem, not a hot-path-write problem. Track separately.
- **Replacing `parking_lot::Mutex<File>` with a lock-free ring.** Unnecessary; after Phase 1 the lock is held for one `write_all` per batch, not per entry.
- **Switching to `O_DIRECT` / `io_uring`.** Big architectural shift; revisit only if Phase 1–5 leave throughput on the table.
- **Persisting heap-side timestamps differently.** The `ts` sidecar already covers this; v2's mmap tier supersedes the question.

---

## Sequencing recommendation

1. Phase 1 (mechanical, isolated, high payoff) — single PR.
2. Phase 2 (mechanical, depends on Phase 1 to unlock batched-disk + batched-heap end-to-end) — single PR.
3. Phase 3 (semantics tweak, async task added) — single PR with the rustdoc update on `EveryN` and the new tests.
4. Phase 4 (additive variant + counter) — single PR.
5. Phase 5 (gated, opt-in) — separate design discussion before code; do not start until 1–4 are merged and benched.
