# Net v0.9 — "Killing Moon" Phase II

v0.9 is a hardening release. No new features, no new transports, no new SDK surfaces — every commit on this branch is a bug fix, a regression test, or a documentation tightening. The conviction we shipped under v0.8 ("Killing Moon") was that distributed compute should not be a control-plane problem. v0.9 is the version where we stand behind that conviction by walking it through audit after audit and tightening every seam we found.

The work was driven by four parallel-pass internal audits totalling 102 items across the bus, the shard manager, the RedEX append log and its CortEX fold, the JetStream and Redis adapters, the mesh transport, the FFI surface, and every binding.

**By the numbers**

- 110 commits on `bugfixes-8` over `master`
- 74 audit items: 51 code fixes, 13 doc-only, 4 rejected as non-bugs, 1 already correct, 5 categorized
- 23 review concerns: all addressed in code or doc
- 4 cubic-ai bug reports: all fixed
- 36 new regression tests
- Lib test count: 1391 → 1427 → 1435+ (passing)
- `cargo clippy --all-features --all-targets -- -D warnings`: clean
- `cargo doc --all-features`: clean

The diff is roughly +5 200 / -440 lines. Most of it is one-line fixes
plus the cluster of comments that explain *why* the line had to change
— a v0.9 commit that doesn't carry an explanatory comment is rare.

---

## Addressed in this release

### RedEX & CortEX (storage + folded state)

- **Lost events on partial replay failure** — `MigrationTargetHandler::drain_pending` returned on first delivery error without restoring the undelivered tail; everything past the failure was permanently lost. Fix preserves the tail for the next drain and a regression test pins both the resume and the prefix-not-redelivered invariant.
- **Silent eviction during tail backfill** — backfill could miss the `Lagged` signal under retention rollover and silently drop events. Now signals correctly during backfill.
- **Index task exits permanently after `Lagged`** — the tail task halted on `Lagged` and never recovered. Now clears the index, re-tails live-only with a 5/20/60/250 ms saturation backoff, and surfaces a `lag_resets()` counter so aggregating downstreams can detect lossy resets.
- **Snapshot-store retention drops high-water mark on `remove`** — a stale producer could re-stage older snapshots after a remove. Added a per-entity high-water table that survives `remove`. `forget()` is now `pub(crate)` so the anti-rewind invariant can't be defeated externally.
- **Observable seq rollback via `next_seq()`** — external readers could observe a temporarily-bumped `next_seq` mid-rollback. Now reads under the state lock.
- **`new_heap` accepts `RedexFlags::INLINE`** — the heap path silently accepted the inline flag, breaking invariants. Now rejected.
- **`append_batch` empty-input returns plausible-looking seq (breaking)** — returned `0` for both empty input and the legitimate seq-0 first write. Now `Result<Option<u64>, _>`. **See breaking-changes section.**
- **Age-retention off-by-one (breaking)** — boundary was `>` (entries at exact cutoff dropped); now `>=` (retained). **See breaking-changes section.**
- **`Stop` policy halts without final `changes_tx` notify** — subscribers got no signal on halt. Initial fix added `notify_waiters` + `changes_tx.send(seq)`; the broadcast was later refined to NOT emit the failing seq, since `changes_tx` is documented as carrying *successfully-folded* sequences.
- **Cortex `changes_tx` broadcasts failing seq on Stop+non-recoverable halt** — pre-fix subscribers could observe a phantom `Seq(failing_seq)`, mis-routing state. Now drops the broadcast on halt; subscribers poll `is_running()`.
- **`RedexFile::Debug` deadlock footgun** — `Debug` called `len()` and `next_seq()`, both of which take the state lock. Now reads only the lock-free atomics.
- **`RedexIndex::clear()` on Lagged is silent** — added the `lag_resets()` accessor as a public sentinel.
- **`RedexIndex` saturation-resume can hot-loop** — under sustained burst with an under-sized `tail_buffer_size` the loop emitted a warn per cycle. Now backed off and rate-limited.

### Bus, shards, and dispatch

- **Activation-failure abort drops drain-worker scratch buffer** / **Batch worker abort drops in-memory `current_batch`** — `.abort()` dropped events. Now graceful await + dispatch with bounded `tokio::time::timeout(2 × adapter_timeout)` so the rollback can't hang on a parked worker.
- **`num_shards` decremented on rollback that never incremented it** — activate-failure rollback over-decremented `num_shards` for never-activated shards. Decrement is now gated on the shard's mapper state. A targeted `remove_specific_stopped_shard` replaces the bulk `remove_stopped_shards()` so sequential `manual_scale_down` doesn't prune sibling state under itself.
- **`ShardManager::activate_shard` double-counts on idempotent calls** — repeated activates kept bumping `num_shards`. Now gated on the mapper's `transitioned` signal.
- **`activate()` budget gate** — load-then-store is safe today because the held write lock on `shards` serializes both the load and the mutation. The lock-held invariant is now documented as the correctness gate (CAS would be belt-and-braces, not strictly required).
- **Shutdown drain race past `in_flight_ingests`** — single zero-pass could miss late producers. Now requires two consecutive zero passes.
- **`shutdown()` returns `Ok(())` after timeout-with-drops** — lossy shutdown looked successful. Now surfaces via `events_dropped` + a dedicated `shutdown_was_lossy` flag.
- **`drain_finalize_ready`** — `Release` pairs only via implicit fence on the in-flight spin's SeqCst; promoted to SeqCst at the store site so the happens-before is explicit. Deadline-break path documented as the data-loss escape hatch.
- **`PollMerger` default shard list is wrong after dynamic scale-down** — polled from a stale `0..num_shards` range, missing live shards. Now uses the live shard id set, propagated through both add and remove paths.
- **`poll_merger` ArcSwap leaves polls operating on stale topology** — topology-snapshot semantics now documented on `poll()`.
- **`per_shard_limit` silently capped at 10 000** — caller had no signal. Surfaced via `truncated_at_per_shard_cap: bool` in `ConsumeResponse`.
- **`has_more=true` from a stalled adapter is silently suppressed** — stalled shards invisible to the caller. Now surfaced via `stalled_shards: Vec<u16>`.
- **`Cursor::encode` returns empty cursor on serialization failure** — empty cursor restarted polling from zero (silent rewind). Initial fix used `expect(...)`; later refined to return `Result<String, ConsumerError>` so an async `poll()` panic can't take down a runtime worker. **Minor breaking change for direct callers.**
- **`PER_SHARD_FETCH_CAP` made public** — exposed an internal tuning knob as API. Now `#[doc(hidden)]`. Read `truncated_at_per_shard_cap` instead.
- **`add_events(vec![])` flushes as a side effect** — load-bearing for the rollback path. Documented and pinned by `add_events_empty_can_flush_via_timeout`.
- **`flush()` baseline excludes events flushed via `remove_shard_internal`** — verified `events_dispatched` is bumped on stranded-flush; was already correct.
- **`dispatch_batch` final attempt collapses error reasons** — all retries were tagged with one collapsed error. Now structured per-attempt `reason`.
- **`dispatch_batch` retry sleep has no jitter / backoff** — synchronized retry storms across shards. Now jittered exponential via `retry_backoff(shard_id, attempt)`.
- **`drain_finalize_ready` ordering doc** — clarified that the SeqCst happens-before only covers the non-deadline exit; deadline-path stranded events are exactly the ones surfaced via `events_dropped` + `shutdown_was_lossy`.

### Atomics, timestamps, and counters

- **`pushes_since_drain_start` mismatched atomic ordering** — producer used Relaxed, drain side used Acquire. Now both Acquire.
- **`in_flight_ingests` is `AtomicU32` with no saturating semantics** — pathological producer counts could wrap. Widened to `AtomicU64`.
- **`TimestampGenerator` uses hard-coded baseline `0`** — TSC delta math wrong. Now captures baseline at construction.
- **`TimestampGenerator` monotonicity stalls before the documented panic** — stalled spin instead of advertised panic. Now panics preemptively at `u64::MAX`.
- **`velocity_samples` `VecDeque` bounded only by time, not count** — burst could grow unbounded. Now also count-capped.
- **Partition `next_id` reuses ID 0 on `u64::MAX` overflow** — wrap-around silently re-issued IDs. Now saturates.

### Adapters (JetStream / Redis / dedup)

- **JetStream `as u16` truncates `shard_id`** — values > 65 535 wrapped silently. Now rejected with `Fatal` (and `poll_shard` propagates the `Fatal` instead of log-and-skipping).
- **JetStream `unwrap_or_default()` on remote JSON** — malformed `r` field re-serialized as empty bytes. Now propagated as `Fatal`.
- **JetStream cold-stream poll walks `fetch_limit * 10` round-trips** — ~1010 RTTs per poll on cold streams. Now bails after `consecutive_not_found_cap`, gated on `first_seq == 0` so populated *sparse* streams (events at seq 1, 500, 1000) walk past arbitrary deletion gaps.
- **JetStream `from_id` cursor `seq + 1` overflows** — wrapped to 0 at `u64::MAX`, silent restart. Now `checked_add(1).unwrap_or(seq)`.
- **JetStream `Fatal` drops accumulated batch in `poll_shard`** — documented; acceptable since `Fatal` is non-retryable.
- **Redis `is_healthy` PING has no enforced timeout** — could hang indefinitely. Now wrapped in `command_timeout`.
- **Redis & JetStream `limit + 1` overflow on adversarial limits** — wrapped to 0, silent under-delivery. Now `saturating_add(1)`.
- **`RedisStreamDedup::new` accepts unbounded capacity** — clamped at `MAX_CAPACITY = 1<<24`.
- **`RedisStreamDedup` is FIFO eviction, not LRU as documented** — docs were wrong. Updated to describe FIFO accurately.
- **`dedup_state` silently swallows fsync failures** — `let _ = f.sync_all()` ignored disk-full errors. Propagated; cross-platform fixed via single writable handle (`File::open` returned read-only on Windows; `FlushFileBuffers` failed silently).
- **`dedup_state::create_new(true)` poison after crash** — a stale tempfile from a crashed prior run could break every subsequent save. Added `fs::remove_file(&tmp).ok()` before `create_new`.

### Security & permissions

- **`ttl_seconds = 0` token mints expired** — born-expired tokens with no diagnostic to the issuer. `try_issue` returns `TokenError::ZeroTtl`.
- **`Identity::issue_token` panic on `Duration::ZERO`** — first fix routed through `try_issue.expect(...)`, which still aborted the process with a misleading "ReadOnly" message. Now soft-clamps to 1 second, `debug_assert!`s in dev builds, and the wrapper's panic messages match each `try_issue` variant precisely.
- **`PermissionToken::issue` panic message misattributes ZeroTtl as ReadOnly** — fixed in tandem with the above.
- **Anti-replay window cleared on large legitimate jumps** — whole bitmap zeroed silently. Now emits a structured warn before zeroing.
- **`OriginStamp` has no per-packet binding** — threat model documented.
- **Untrusted-input panics in subnet config** — added `try_*` fallible constructors for SDK callers.
- **Channel decoder accepts trailing bytes on UNSUBSCRIBE/ACK** — decoder now requires `cur.remaining() == 0` after the channel name + token.

### Bindings (Node, Python, Go, C)

- **Node binding `u32 → u8` truncation on member index** — `as u8` silently truncated > 255. Switched to `try_into` with explicit `> 255` rejection.
- **Python bindings hold GIL across blocking compute ops** — `scale_to`, `on_node_failure`, `sync_standbys`, `promote` blocked the GIL during long ops. Now release via PyO3 0.28's `py.detach`.
- **Node-binding groups carry an unused `kind: String` field** — removed dead field.
- **`RedisStreamDedup` stripped from generated Node binding surface** — a regen-without-redis-feature dropped the class from `index.d.ts` and `index.js`. Re-ran NAPI generation with `--features redis,…`.
- **Python parity test for `append_batch([])` returns `None`** — added so future binding regenerations don't silently drop the contract.
- **`include_str!` of `go/net.h` escapes the crate root** — broke `cargo publish` and out-of-repo vendoring. Copied to in-crate `include/net.go.h` and updated the parity test.
- **C SDK README** — fixed stale references to a removed `bindings/go/net/net.h` path.
- **`Runtime::block_on` from `extern "C"` shims unwinds across FFI** — reentrancy hazard documented.

### Behavior rules & evaluators

- **Lossy `as_f64` for all numeric ordering in rules** — big i64/u64 values lost precision through f64. Now compares i64/u64 directly with sign-aware mixed-type fallback.
- **`compare_numbers` brittle with `serde_json/arbitrary_precision`** — a transitive dep enabling that feature would silently make rules fail closed. Added `debug_assert!` so the misuse is loud in dev.
- **Non-deterministic verdict ordering** — `window_failures` ordering depended on iteration order. Now sorts and dedups for determinism.
- **`record_execution` window-reset across rule reload** — counters mis-reset for non-rate-limited rules. Now skipped for those.
- **Stream tight-loop spin** — zero `poll_interval` spun the loop. Clamped to non-zero.
- **Stream backoff overflow on absurd `poll_interval`** — doubling overflowed. Now saturating.
- **`Rule::new` lossily casts `u128` millis to `u64`** — long uptimes truncated. Now uses saturating `u64::try_from`.

### Compute (daemons + migration)

- **Migration `next_seq` overflow** — `replayed_through + 1` could panic at `u64::MAX`. Now `saturating_add`.
- **DashMap entry guard held across registry I/O** — `start_snapshot` held the entry guard across user-supplied snapshot code, deadlock-prone. Drops the guard before I/O. Two racing starts produce two `MeshDaemon::snapshot()` calls — non-idempotent daemons must single-flight at their layer; documented.
- **`on_node_recovery` does not break after first matching partition** — documented as intentional for overlapping partitions.

### Mesh transport & packet codec

- **Silent `event_count` truncation in packet builder** — builder accepted oversized batches and truncated. Now rejects with explicit error.
- **`StreamWindow.decode` unbounded `total_consumed`** — consumer-side clamp was already enforced; documented.
- **Modulo bias in equal-weight candidate selection** — `hash % len` biased low for non-power-of-2. Now Lemire's `(hash * len) >> 64`.
- **`cpus.saturating_mul(2)` caps `max_shards: u16` at 65 535** — documented as intentional.
- **`mapper.rs` cooldown check + scale mutation atomicity** — RwLock-implicit serialization documented.

### SDK & error surface

- **`SdkError::Ingestion(String)` flattens structured `IngestionError`** — backpressure / sampled / unrouted all funnelled through one stringly-typed variant. Routed to structured `Sampled` / `Unrouted` / `Backpressure`. **Breaking — see breaking-changes section.**
- **`SdkError` enum is breaking and not `#[non_exhaustive]`** — added `#[non_exhaustive]` so future variant additions are minor-version changes.
- **`NetBuilder::identity()` silently overrides `entity_keypair`** — builder accepted both fields and silently dropped one; now rejects the conflict at build time.
- **`NetAdapterConfig::validate` accepts pathological values** — added upper bounds + heartbeat floor.
- **`Drop` releases shutdown gates synchronously while workers hold `Arc<Self>`** — no partial-destruction UB; documented.

### Test hygiene

- **`MigrationTargetHandler::drain_pending` regression test** — strengthened to also assert the prefix is NOT redelivered.
- **`add_events_empty_can_flush_via_timeout`** — pins that empty input flushes after `max_delay`. Load-bearing for the rollback path.
- **`retry_backoff` jitter test** — relaxed from `>= 8 / 16` to `>= 4 / 16` to stay robust against `DefaultHasher` distribution drift across toolchain versions.
- **`debug_does_not_acquire_state_lock`** — pins the lock-free `Debug` invariant by holding `state.lock()` across `format!("{:?}", file)`.
- **`stop_policy_does_not_broadcast_failing_seq`** — pins the cortex broadcast contract.
- **`cold_stream_bail_gate_only_fires_when_first_seq_is_zero`** — pins the JetStream sparse-stream gate.

---

## Breaking changes

### Rust core (`net` crate)

#### `RedexFile::append_batch` signature changed

`append_batch` and `append_batch_ordered` now return `Result<Option<u64>, RedexError>` instead of `Result<u64, RedexError>`.

**Why**: the prior shape returned `Ok(0)` for an empty batch, which collided with the legitimate "first event of a non-empty batch landed at seq 0" return — callers couldn't distinguish "I appended nothing" from "I appended one event at seq 0".

**Migrate**:
```rust
// Before
let first_seq: u64 = file.append_batch(&payloads)?;

// After
let first_seq: Option<u64> = file.append_batch(&payloads)?;
```

Same change cascaded through `OrderedAppender::append_batch` and `TypedRedexFile::append_batch`.

#### Retention boundary semantics

Age-based retention now uses `>=` instead of `>` for the cutoff. An entry whose timestamp equals the cutoff exactly is **retained** (was: evicted).

**Why**: the original `>` comparison was off-by-one — entries on the boundary lasted strictly less than the configured `retention_max_age`. Production deployments with tight age caps observed events expiring one tick early.

**Migrate**: no source change required, but tests that asserted exact-boundary entries were *evicted* will now fail. Update assertions to expect retention.

#### `Cursor::encode` returns `Result`

`CompositeCursor::encode` now returns `Result<String, ConsumerError>` instead of `String`. Affects callers using the type directly; `EventBus::poll()` already handles the new shape.

**Migrate**: append `.unwrap()` (in tests) or `?` (in production) to existing call sites.

#### `PollMerger::new` signature

`PollMerger::new` takes `Vec<u16>` of active shard IDs instead of `num_shards: u16`. This is an internal-leaning type but `pub`; downstream wrappers may need to update.

#### `ConsumeResponse` struct fields

Added `truncated_at_per_shard_cap: bool` and `stalled_shards: Vec<u16>`. Callers that construct `ConsumeResponse` directly need to populate the new fields. Pattern matches with `..` unaffected.

#### `PER_SHARD_FETCH_CAP` is `#[doc(hidden)]`

Still `pub const` (callable), but no longer documented as API. Callers checking truncation should read `ConsumeResponse::truncated_at_per_shard_cap` instead of comparing against the constant.

#### `SnapshotStore::forget` is `pub(crate)`

Was `pub`. The function defeats the high-water-mark anti-rewind invariant — exposing it publicly let any caller stage stale snapshots over fresh ones. No production callers existed; only test code referenced it.

### Rust SDK (`net-sdk`)

#### `SdkError` is `#[non_exhaustive]` + new variants

`SdkError` now carries the `#[non_exhaustive]` attribute. Two new variants moved out of the stringly-typed `Ingestion(String)` fallback:

- `Sampled` — event deliberately dropped by a sampling / decimation policy. Retry is pointless.
- `Unrouted` — no routable shard for the event (typically a topology-transient state). Retry once topology stabilizes.

`From<IngestionError>` now routes `IngestionError::Sampled` and `IngestionError::Unrouted` to these structured variants. Code that string-matched on the content of `Ingestion(String)` for those causes silently stops matching.

**Migrate**:
```rust
// Match arms now must include a wildcard
match err {
    SdkError::Backpressure => /* drop or retry */,
    SdkError::Sampled => /* accept the drop */,
    SdkError::Unrouted => /* retry after topology stabilizes */,
    SdkError::NotConnected => /* peer gone */,
    _ => /* future-proof catch-all */,
}
```

If you were substring-matching on `Ingestion(...)` for "sampled" or "no shard", switch to the structured variants.

#### `Identity::issue_token` no longer panics on `Duration::ZERO`

Previously the panicking convenience wrapper aborted with a misleading `"public-only keypair"` message when `ttl == Duration::ZERO`. It now soft-clamps to 1 second and `debug_assert!`s in dev builds, so the misuse surfaces in tests but doesn't take down the process in release.

`Identity::try_issue_token` (the explicit fallible surface) still rejects zero-TTL with `TokenError::ZeroTtl` — bindings route through it.

**Migrate**: nothing strictly required. Tests that exercised the panic with `#[should_panic(expected = "public-only keypair")]` need updating — the new debug-assert message contains `"Duration::ZERO"`.

### Bindings

| Binding | Change |
|---|---|
| **Node** | `appendBatch(...)` returns `bigint \| null` (was `bigint`). Empty input → `null`. |
| **Python** | `append_batch(...)` returns `int \| None` (was `int`). Empty input → `None`. |
| **Node** | `RedisStreamDedup` class is back on the binding surface (it had been stripped by an earlier feature-incomplete regen — not a breaking change for downstream npm consumers, just a regression repaired). |
| **Go** | `IssueToken{TTLSeconds: 0}` returns a non-nil `error` (was: same — surfaced from FFI's `try_issue` path). No source change. |

### Behavioral fixes that may surface as test breakage

These aren't strictly API-breaking, but if your test suite asserted the old behavior they will need updating:

- **`num_shards` rollback**: `add_shard` + failed `activate_shard` + rollback no longer over-decrements `num_shards`. Tests that expected the off-by-one will fail.
- **JetStream sparse-stream polling**: `poll_shard` no longer breaks early on 64 consecutive `NotFound`s when `info()` reported a populated stream (`first_seq > 0`). Tests on populated sparse streams that asserted early-bail behavior will see longer walks.
- **Cortex `changes_with_lag` halt path**: on `Stop` + non-recoverable error the failing seq is no longer broadcast on `changes_tx`. Subscribers need to poll `is_running()` to detect halt — pre-fix they could have observed (incorrectly) a `ChangeEvent::Seq(failing_seq)`.
- **`RedexFile::Debug`**: no longer acquires the state mutex; reads only the lock-free atomics. Output format changed (`next_seq_atomic` field name; `len` removed).
- **`SnapshotStore::store`**: equal-seq concurrent-store linearization is now documented to be on the snapshots-side entry guard, not on the high-water mark. Behavior unchanged; doc clarified.

---

## How to upgrade

1. Bump your `Cargo.toml` / `package.json` / `requirements.txt` / `go.mod` to the v0.9 line.
2. Recompile. The signature changes (`append_batch` → `Result<Option<u64>>`, `Cursor::encode` → `Result`, `SdkError` `#[non_exhaustive]`) will surface as compile errors at the exact call sites that need updating — follow the **Migrate** snippets above.
3. If you have tests that assert pre-fix behavior on the items in *Behavioral fixes that may surface as test breakage*, update those assertions.
4. Bindings consumers (Node / Python): no source change is *required* — the type-stub updates are forward-compatible — but treat the new `null` / `None` empty-input returns as the canonical "I appended nothing" signal in your call sites.
5. Re-run your full suite. The lib + binding suites run green; if your suite covers integration paths not exercised by the audit, this is the right release to catch any drift.

---

Released 2026-05-02.

## License

See [LICENSE](../../../LICENSE).
