# Bug Report: `net/crates/net/`

Audit of the Rust event-bus crate at `crates/net/src/`. Findings are based on the source tree at branch `performance-simple` (commits through `62fe0ef "Fix issues"`). Severity-ordered within each section. Highest-severity claims were spot-verified against the source; speculative findings that did not survive verification are listed in the "Dropped after verification" section for transparency.

---

## Summary

| # | Severity | File | Issue |
|---|---|---|---|
| 1 | HIGH | `consumer/merge.rs:326-339` | Lazy filter drops matching events from un-inspected shards |
| 2 | HIGH | `config.rs` | Multiple unvalidated zero-divisors in `validate()` |
| 3 | MEDIUM | `bus.rs:565-599` | Shutdown vs. concurrent `ingest` race strands events |
| 4 | MEDIUM | `shard/mapper.rs:564` | Shard ID reuse after `remove_stopped_shards` |
| 5 | MEDIUM | `shard/mod.rs:438-456` | `DropOldest` retry violates SPSC contract |
| 6 | MEDIUM | `shard/mapper.rs:688-712` | `finalize_draining` reads a destructively-reset counter |
| 7 | MEDIUM | `adapter/jetstream.rs` | Silent failures and bad transient classification |
| 8 | MEDIUM | `adapter/redis.rs:198-228` | Pipeline timeout-then-retry produces duplicates |
| 9 | MEDIUM | `adapter/redis.rs:46-82` | `stream_key` cache uses `Vec` keyed by sparse shard_id |
| 10 | MEDIUM | `timestamp.rs:63` | `next()` returns raw TSC ticks; doc says nanoseconds |
| 11 | MEDIUM | `ffi/mesh.rs:1092-1105` | `collect_payloads` lacks per-entry null check |
| 12 | MEDIUM | `ffi/mod.rs:866-875` | `net_shutdown` spin-waits unboundedly on in-flight ops |
| 13 | LOW | `error.rs:21-22, 45-46` | `Serialization(String)` discards underlying serde error |

---

## 1. HIGH — `consumer/merge.rs:326-339` — Lazy filter drops matching events from un-inspected shards

**What's wrong.** The `Ordering::None` filter loop breaks once `kept.len() >= limit + 1`:

```rust
for event in all_events.drain(..) {
    if kept.len() >= target {
        break;
    }
    if event_matches_filter(&event, filter) {
        kept.push(event);
    }
}
```

Because `all_events` is built shard-by-shard via `extend()`, hitting the target inside (say) shard 0 means shard 1/2/3 events are silently discarded by `Drain::drop` without ever being filtered.

**Failure scenario.** The cursor at line 382 starts from `new_cursor` (which already advanced past every fetched event for every polled shard) and is only overridden for shards present in the *returned* set. Shards that were never inspected keep the advanced position, so any matching events in their fetched range are lost forever. The infinite-loop regression test at line 1336 only happens to pass because its shard-1 events don't match the filter — flip the test so shard 1 also has `type:"token"` events and matching events are silently dropped.

**Fix.** In the `Ordering::None` filter path, drop the early `break` (use `retain` like the sort path), or only advance the cursor for shards whose events were fully drained.

---

## 2. HIGH — `config.rs` — Multiple unvalidated zero-divisors in `validate()`

**What's wrong.** Several config knobs that act as divisors are accepted as 0 by `EventBusConfig::validate`:

- `BackpressureMode::Sample { rate: 0 }` is accepted (`config.rs:220-223`); downstream sampling typically uses `counter % rate` → div-by-zero panic.
- `BatchConfig.velocity_window: Duration::ZERO` with `adaptive: true` (`config.rs:262-277`) div-by-zeros the throughput calculator.
- `ScalingPolicy.cooldown` / `metrics_window` / `scale_down_delay` zero values aren't rejected (`config.rs:636-656`). `metrics_window=0` panics in rate math; `cooldown=0` thrashes the scaler; `scale_down_delay=0` scales down on the first underutilized sample.
- `EventBusConfig.adapter_timeout: Duration::ZERO` is accepted (`config.rs:38, 69-90`); every adapter call then times out instantly.
- `RedisAdapterConfig` (`config.rs:326-355`) and `JetStreamAdapterConfig` have no `validate()` at all and aren't recursed into from `EventBusConfig::validate`. `pipeline_size: 0` ships through.

**Failure scenario.** A user constructs e.g. `RedisAdapterConfig { pipeline_size: 0, .. }`, the bus accepts it, then the Redis adapter spins forever or panics on first batch. Similar for the other configs.

**Fix.** Add the missing zero/non-zero checks in `validate()`, prefer `NonZeroU32` / `NonZeroDuration` types where possible, and recurse into adapter configs.

---

## 3. MEDIUM — `bus.rs:565-599` — Shutdown vs. concurrent `ingest` race strands events

**What's wrong.** `ingest` does `shutdown.load(Relaxed)` and the drain worker uses `Relaxed` reads too. There is no happens-before relationship between "producer's ring-buffer push completes" and "drain worker observed empty + shutdown".

**Failure scenario.** A producer reads `shutdown==false`, is preempted, then pushes after the drain worker's last `pop_batch_into` returned 0 and exited. The ring buffer is later dropped wholesale with the event still in it. The shutdown comment at lines 559-564 claims the inverse ("every event the producer has handed to the bus is now in the per-shard mpsc channel").

**Fix.** Introduce a quiescence step — count in-flight ingests, and have the drain worker re-check the ring buffer after observing `shutdown=true && in_flight==0`. Alternatively, gate `ingest` behind an `Acquire` load and ensure the drain worker performs an `Acquire` of the same atomic before its final sweep.

---

## 4. MEDIUM — `shard/mapper.rs:564` — Shard ID reuse after `remove_stopped_shards`

**What's wrong.**

```rust
let max_id = shards.iter().map(|s| s.id).max().unwrap_or(0);
```

`max_id` is computed against the *current* shards vector, which excludes any shards already drained-and-removed. After removing the highest-ID shard, the next `scale_up` re-allocates the same ID.

**Failure scenario.** Scale up to 10, drain+remove shard 9, scale up by 1 → new shard also gets ID 9. `ShardManager.shard_index` (`mod.rs:661`) silently overwrites the stale entry, and any external metric/checkpoint that keys on shard ID will merge two unrelated shard lifetimes into one.

**Fix.** Maintain a monotonic `next_id: AtomicU16` on `ShardMapper` and `fetch_add` it for new shards.

---

## 5. MEDIUM — `shard/mod.rs:438-456` — `DropOldest` retry violates SPSC contract

**What's wrong.** `push_with_backpressure` calls `shard.try_pop()` from the producer's call stack to evict the oldest event when the buffer is full. The ring buffer is documented SPSC (`ring_buffer.rs:62-67`), and the test build asserts a single consumer thread ID — the legitimate consumer is the batch worker.

**Failure scenario.** In test builds, mixing producer-side `try_pop` with the batch worker's `try_pop` panics on the consumer-thread tracking assertion. In release, mutex-serialization happens to keep the atomics race-free today, but any future SPSC optimization (relaxing tail-side memory ordering on the assumption that only one thread touches it) would silently corrupt data.

**Fix.** Add a producer-side `evict_oldest()` on the ring buffer that bypasses consumer-thread tracking, or document that `DropOldest` requires the producer thread also to be the SPSC consumer (currently false, since the batch worker is the consumer).

---

## 6. MEDIUM — `shard/mapper.rs:688-712` — `finalize_draining` reads a destructively-reset counter

**What's wrong.** The "is the shard quiescent?" check uses `events_in_window`, but `collect_and_reset()` swaps that field to 0 on every metrics tick.

**Failure scenario.** A draining shard whose buffer transiently empties between two metrics ticks is finalized while a producer with a cached `shard_id` is still pushing. The 100ms grace window reduces but does not eliminate the race.

**Fix.** Track a separate "events since drain start" counter that is *not* reset by `collect_and_reset`. Use the ring buffer's actual `len()` plus the `draining` flag age as the empty signal.

---

## 7. MEDIUM — `adapter/jetstream.rs` — Silent failures and bad transient classification

**Three independent issues:**

1. **`shutdown` swallows `drain` error** (`L307-323`). `let _ = client.drain().await;` discards the error and reports `Ok(())`. Trait contract says shutdown should flush.
2. **`is_transient_error` is over-broad** (`L433-440`). Treats *every* error other than `WrongLastSequence` as transient — including auth, quota-exceeded, and stream-config errors. Combined with a naive caller retry loop, this becomes an infinite retry storm.
3. **Stream-creation race** (`L110-165`). Two cold-start `on_batch` calls for the same shard both fire `get_stream → create_stream` and both insert into the cache. Idempotent today, but extra RPCs on cold start, and a hazard if `create_stream` configs ever diverge between callers.

**Fix.** Surface the `drain` error as `AdapterError::Transient`; enumerate fatal error kinds in `is_transient_error`; gate cold-start with a per-shard `OnceCell` or per-shard mutex.

---

## 8. MEDIUM — `adapter/redis.rs:198-228` — Pipeline timeout-then-retry produces duplicates

**What's wrong.** `tokio::time::timeout` over a `MULTI/EXEC` does not roll back bytes already on the wire. Redis can execute the EXEC after the future is dropped; the subsequent retry then double-publishes via XADD with auto-generated `*` IDs and no dedup.

**Failure scenario.** At-least-once degrades to *more*-than-once with no dedup. The "Improve Redis throughput" comment claiming "either all XADDs succeed or none do" is true within one `MULTI` but not across cancel-then-retry.

**Fix.** Include a per-event dedup token (e.g. `{shard_id}:{seq_start}:{i}`) and gate XADD via a Lua script that checks `SADD` of recent tokens. Otherwise, document at-least-once-with-duplicates explicitly and rely on consumer-side idempotency.

---

## 9. MEDIUM — `adapter/redis.rs:46-82` — `stream_key` cache uses a `Vec` keyed by shard_id

**What's wrong.**

```rust
while cache.len() <= idx { cache.push(...) }
```

If the first access is `shard_id = 65535`, this allocates 65536 placeholder entries.

**Failure scenario.** Sparse / hashed shard IDs cause O(max_shard_id) memory blowup on cold access.

**Fix.** Switch the cache to `HashMap<u16, Arc<str>>`.

---

## 10. MEDIUM — `timestamp.rs:63` — `next()` returns raw TSC ticks; doc says nanoseconds

**What's wrong.** `quanta::Clock::raw()` returns the raw TSC counter, not nanoseconds. The docstring at line 63 says "Returns a strictly monotonically increasing value (nanoseconds)". `insertion_ts` is plumbed into `StoredEvent`, serialized externally, and used cross-shard for sorting in `consumer/merge.rs:351`.

**Failure scenario.** Sorting works (TSC is invariant on modern x86 across cores), but external consumers reading the JSON `insertion_ts` field receive TSC ticks — about 3.5× larger than wall-clock-ns on a 3.5GHz core — breaking interop with anything expecting ns-since-epoch or correlating against ns from other sources. The `raw_to_nanos` helper exists but is never applied before storage.

**Fix.** Convert via `clock.delta_as_nanos(0, raw)` inside `next()` (preserving monotonicity by storing nanos), or rename the field/doc to `insertion_tsc` and document the unit honestly.

---

## 11. MEDIUM — `ffi/mesh.rs:1092-1105` — `collect_payloads` lacks per-entry null check

**What's wrong.** After verifying that the outer `payloads` / `lens` arrays are non-null, the function dereferences each `*payloads.add(i)` and feeds the result straight to `slice::from_raw_parts(ptr, lens[i])`. Per-entry pointers are not null-checked.

**Failure scenario.** Any C caller that passes an array containing a null entry — easy to do when batching optional payloads — produces instant UB on `from_raw_parts(null, len)`. The C contract gives the caller no way to convey "skip this entry," so a defensive null check is cheap and correct.

**Fix.** Null-check each per-entry pointer before constructing the slice; return an error code on nulls.

---

## 12. MEDIUM — `ffi/mod.rs:866-875` — `net_shutdown` spin-waits unboundedly

**What's wrong.** The atomic `FfiOpGuard` correctly prevents `Box::from_raw` while ops are in flight, but if a concurrent op blocks (e.g. `net_flush` against a hung adapter), `net_shutdown` busy-waits forever with no progress and no timeout.

**Failure scenario.** A hung adapter pins one CPU at 100% inside `net_shutdown` and the C client has no recovery path.

**Fix.** Bounded wait with a deadline; on timeout, return a `Busy` / `Timeout` error code. Or convert to `tokio::sync::Notify` so the wakeup is event-driven.

---

## 13. LOW — `error.rs:21-22, 45-46` — `Serialization(String)` discards underlying serde error

**What's wrong.** `Serialization(String)` instead of `Serialization(#[from] serde_json::Error)` loses category, line, column, and breaks the `source()` chain.

**Fix.** Change to `Serialization(#[from] serde_json::Error)`.

---

## Dropped after verification

These were flagged by the audit but did not survive a check of the source. Documented here so future audits don't waste cycles on them.

- **`ring_buffer.rs:319-322` `len()` torn read.** Claimed that head/tail load ordering could underflow `wrapping_sub`. False under the SPSC contract: head only increases (producer), tail only increases up to head (consumer), so `head ≥ tail` always holds and `wrapping_sub` returns the correct small positive value.
- **`ring_buffer.rs:230-241` `pop_batch` panic safety.** Claimed that a panic between `assume_init_read` and the `tail.store` would leave moved-out slots reachable. False: the only operations between the read and the store are `Vec::push` calls, and the vec was pre-`reserve`d, so push is allocation-free and panic-free.
- **FFI panics across the C ABI are UB on every entry point.** False for shipped builds: `[profile.release]` in `Cargo.toml:225` sets `panic = "abort"`, so unwinding cannot cross the boundary in release artifacts. Test/debug builds still unwind across the boundary, but that is a test-only concern and the FFI is not exercised from C in those configurations.

---

## Recommended order of fixes

1. **`merge.rs:326-339`** — silent data loss; drop the early `break` or restrict cursor advance to fully-drained shards.
2. **`config.rs` zero-divisor validation** — turn panics on misconfiguration into config errors.
3. **`mapper.rs:564` shard ID monotonicity** — replace `max+1` with `next_id: AtomicU16`.
4. **`bus.rs::shutdown` quiescence step** — close the concurrent-ingest stranding window.
5. **`ffi/mesh.rs::collect_payloads` per-entry null check** + **`net_shutdown` bounded wait**.
6. **`jetstream.rs` `is_transient_error` enumeration** + **stop swallowing `drain()`**.
7. Remaining MEDIUM items as time permits; LOW item is a clean refactor.
