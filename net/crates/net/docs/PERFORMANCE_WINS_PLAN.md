# Performance Wins Plan

Concrete, verified perf opportunities in `crates/net/src/`. Findings are
ranked by impact tier and ordered roughly by ROI. Line numbers are pinned
to the state of the tree at the time of audit and may drift; the quoted
snippets are the source of truth.

## HIGH

### 1. JetStream `on_batch` awaits each ack serially

**File:** `src/adapter/jetstream.rs:228-244`

```rust
for (i, data) in serialized.into_iter().enumerate() {
    let msg_id = format!("{}:{}:{}", batch.shard_id, batch.sequence_start, i);
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("Nats-Msg-Id", msg_id.as_str());

    js.publish_with_headers(subject.clone(), headers, data.into())
        .await
        .map_err(|e| {
            if is_transient_error(&e) {
                AdapterError::Transient(e.to_string())
            } else {
                AdapterError::Fatal(e.to_string())
            }
        })?
        .await
        .map_err(|e| AdapterError::Transient(e.to_string()))?;
}
```

The double `.await` is structurally correct (first awaits enqueue, second
awaits the ack future), but doing it inside the `for` loop means every
event blocks on the prior event's full network round-trip. For a
1k-event batch over a 1ms-RTT link this is roughly 1s/batch instead of
the few ms it should be.

**Fix.** Issue all publishes first, then collect acks via
`futures::future::try_join_all` (or a `FuturesUnordered` if you want
early failure). Pipelining ack futures is the JetStream-native idiom.

```rust
let mut acks = Vec::with_capacity(serialized.len());
for (i, data) in serialized.into_iter().enumerate() {
    // build headers, publish; PUSH the ack future without awaiting
    acks.push(js.publish_with_headers(subject.clone(), headers, data.into()).await?);
}
futures::future::try_join_all(acks).await?;
```

**Estimated impact:** 10-100x adapter throughput depending on RTT.

---

### 2. JetStream `msg_id` String allocation per event

**File:** `src/adapter/jetstream.rs:229`

`format!("{}:{}:{}", …)` allocates a fresh `String` for every event in
the publish loop, purely to populate the `Nats-Msg-Id` dedup header.
After fix #1 lands the publish loop becomes the hot path and this
allocation matters.

**Fix.** Reuse a single `String` with `.clear()` + `write!`, or build
into a stack buffer (`itoa::Buffer` + small stack concat) and pass
`&str` to `headers.insert`.

**Estimated impact:** small constant factor on top of #1; only worth
doing once #1 is in.

---

### 3. Net adapter allocates an event_id String per event

**File:** `src/adapter/net/mod.rs:580-585`

```rust
for (i, event_data) in events.into_iter().enumerate() {
    use std::fmt::Write;
    let mut event_id = String::with_capacity(24);
    let _ = write!(event_id, "{}:{}", seq, i);
    queue.push(StoredEvent::new(event_id, event_data, seq, shard_id));
}
```

At the documented ~40M evt/s this is the dominant allocator pressure on
the adapter side.

**Fix.** If `StoredEvent` can hold the `(seq, i)` pair directly as
`(u64, u32)`, store the tuple and only render to `String` on demand. If
the field has to be `String`, build into an `itoa::Buffer` plus a small
stack/`SmallVec` buffer and only `String::from` on the final push.

**Estimated impact:** 5-10% throughput on the Net adapter.

---

### 4. Drain worker uses a fixed 100µs `tokio::time::sleep` poll

**File:** `src/bus.rs:805-822`

```rust
let events = shard_manager.with_shard(shard_id, |shard| shard.pop_batch(1_000));

match events {
    Some(events) if !events.is_empty() => {
        if sender.send(events).await.is_err() {
            break;
        }
    }
    Some(_) => {
        // No events — yield briefly. The 100μs sleep is deliberate:
        // this is a latency-first system where the drain loop is the
        // hot path. Longer backoff would add milliseconds of latency
        // to the first event after a quiet period, violating the
        // sub-microsecond design target. The CPU cost of 100μs polling
        // is acceptable for a system that processes 10M+ events/sec.
        tokio::time::sleep(Duration::from_micros(100)).await;
    }
    None => break,
}
```

The comment defends the choice, but two real costs remain: (a) every
empty poll still acquires the per-shard mutex inside `with_shard`, and
(b) the timer-wheel sleep imposes a hard 100µs floor on first-event
latency after any idle period.

**Fix.** Replace the fixed sleep with a `tokio::sync::Notify`-based
wakeup. `try_push_raw` (and any other producer entry point) calls
`notify.notify_one()` only on the empty→non-empty transition. The drain
worker awaits:

```rust
tokio::select! {
    _ = notify.notified() => { /* drain */ }
    _ = tokio::time::sleep(max_idle) => { /* periodic safety wakeup */ }
}
```

This eliminates the per-event latency floor *and* the wasted shard-lock
acquisitions while idle.

**Estimated impact:** removes the 100µs first-event latency floor;
reduces idle CPU cost.

---

### 5. `ingest_raw_batch` double-iterates and over-allocates per shard

**File:** `src/shard/mod.rs:497-538`

```rust
let mut groups: Vec<Vec<Bytes>> = (0..table.shards.len()).map(|_| Vec::new()).collect();
let mut group_ids: Vec<u16> = vec![0; groups.len()];

for event in events {
    let shard_id = self.select_shard_by_hash(event.hash());
    let Some(idx) = self.resolve_idx(&table, shard_id) else { continue; };
    if let Some(g) = groups.get_mut(idx) {
        if g.is_empty() { group_ids[idx] = shard_id; }
        g.push(event.bytes());
    }
}

for (idx, group) in groups.into_iter().enumerate() {
    if group.is_empty() { continue; }
    let shard_id = group_ids[idx];
    let Some(shard_lock) = table.shards.get(idx) else { continue; };
    let mut shard = shard_lock.lock();
    for bytes in group {
        if self.push_with_backpressure(&mut shard, shard_id, bytes).is_ok() {
            success += 1;
        }
    }
}
```

Three wins:

1. **Hoist `resolve_idx` out of the loop.** It hits a HashMap when
   dynamic scaling is enabled. Cache the table once and have
   `select_shard_by_hash` produce the index directly when no remap
   table is present (the static common case).
2. **Stop pre-allocating an empty `Vec<Bytes>` per shard.** With
   `num_shards=64` and 100-event batches you allocate ~64 vecs to use
   ~4. Use a `SmallVec<[(u16, Bytes); 16]>` of `(idx, bytes)` and
   bucket lazily, or do a quick first pass to size groups.
3. **Verify `push_with_backpressure` is a pure `try_push` fast path.**
   If it does any blocking/yield-style backpressure work while the
   shard lock is held, it serializes everything to that shard.

**Estimated impact:** measurable on batch-heavy workloads with high
shard counts.

---

## MEDIUM

### 6. Shutdown flag uses `Acquire` on every ingest

**File:** `src/bus.rs:375, 406`

```rust
if self.shutdown.load(AtomicOrdering::Acquire) {
    return Err(IngestionError::ShuttingDown);
}
```

Both `ingest` and `ingest_raw` are called per event. `Relaxed` is
sufficient: shutdown is a one-way latch, no memory the producer needs to
observe is gated by it. Pair with `Release` in `shutdown()`. Pure win
on ARM/POWER; on x86 `Acquire` loads are essentially free, so verify on
your target before celebrating.

---

### 7. Redis stream-key cache acquires a write lock on miss

**File:** `src/adapter/redis.rs:67-82`

First call for a new shard formats `"{prefix}:shard:{id}"`, takes the
`RwLock` for write, inserts. With `num_shards` known at adapter init,
just precompute `Box<[Arc<str>]>` indexed by shard id. Removes both the
lock and the allocation entirely from steady state.

Low practical gain unless dynamic scaling adds shards frequently.

---

### 8. Filter recursion isn't flattened or specialized

**File:** `src/consumer/filter.rs:96-109`

```rust
#[inline]
pub fn matches(&self, event: &JsonValue) -> bool {
    match self {
        Self::And { filters } => !filters.is_empty() && filters.iter().all(|f| f.matches(event)),
        Self::Or  { filters } => filters.iter().any(|f| f.matches(event)),
        ...
    }
}
```

`And { filters: [single] }` still iterates a Vec. Flatten
`And[And[a,b],c]` to `And[a,b,c]` once at deserialize, and add a
single-element fast path. Only matters for filter-heavy workloads.

---

### 9. `TimestampGenerator` is a CAS spin loop on a shared `AtomicU64`

**File:** `src/timestamp.rs`

Fine if every shard owns its generator (the docs say so), but nothing
in the type system enforces it. If any caller shares one across shards
this becomes a hot-spot under load. Either enforce per-shard ownership
in the API or add a debug assertion.

---

### 10. `dispatch_batch` clones the entire `Batch` per retry attempt

**File:** `src/bus.rs:651-671`

```rust
async fn dispatch_batch(
    adapter: &dyn Adapter,
    batch: Batch,
    shard_id: u16,
    timeout: Duration,
    retries: u32,
) -> bool {
    // Retry attempts clone the batch; the final attempt moves it, saving
    // one clone per dispatch (the common path is retries == 0).
    for attempt in 0..retries {
        match tokio::time::timeout(timeout, adapter.on_batch(batch.clone())).await {
            ...
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    match tokio::time::timeout(timeout, adapter.on_batch(batch)).await { ... }
}
```

The existing comment acknowledges the trade-off: common path moves on
the final attempt, but with `adapter_batch_retries=3` (default) a
flapping adapter clones the full `Vec<Event>` up to 3 times per
dispatch. Cost is zero when retries don't fire and meaningful when
they do.

**Fix.** Change `Adapter::on_batch` (currently `adapter/mod.rs:104`) to
take `Arc<Batch>` and call `Arc::clone` (refcount bump) on each retry.
Adapter implementations that mutate the batch internally — none
currently do — would need `Arc::try_unwrap` on the final attempt or a
`&Batch` signature instead.

**Estimated impact:** zero on the happy path; eliminates O(retries)
batch-sized allocations during adapter degradation. Trait signature
change touches every adapter (`noop`, `redis`, `jetstream`, `net`) but
each call site is one line.

---

### 11. Poll cursor finalization clones every event id as `String`

**File:** `src/consumer/merge.rs:391-400`

```rust
// Only the last returned event per shard matters for the cursor, so
// iterate in reverse and skip shards already seen. This reduces id
// clones from O(all_events.len()) to O(shards.len()).
let mut seen_shards: std::collections::HashSet<u16> =
    std::collections::HashSet::with_capacity(shards.len());
for event in all_events.iter().rev() {
    if seen_shards.insert(event.shard_id) {
        final_cursor.set(event.shard_id, event.id.clone());
    }
}
```

Reverse iteration already pinned the clone count to O(shards) — but
each clone is still a heap-allocating `String::clone`. `CursorPos`
internals already use `Arc<str>` for stored positions; the event-side
type doesn't yet, so every poll merge re-allocates strings the cursor
will turn around and refcount.

**Fix.** Make `event.id` (and the matching cursor setter) carry
`Arc<str>` end-to-end. `.clone()` becomes a refcount bump and the
allocation moves once, to event construction.

**Estimated impact:** poll-rate × shard-count fewer string allocations.
Real on consumer-heavy workloads, modest otherwise.

---

### 12. `seen_shards` HashSet allocated per poll merge

**File:** `src/consumer/merge.rs:394-395`

`HashSet<u16>` with capacity = `shards.len()` is allocated once per
poll merge purely as a "have I seen this shard yet" flag. With typical
shard counts in the 4-64 range a `u64` / `u128` bitset is faster, has
no allocation, and stays in a register.

**Fix.** Replace with a stack-allocated bitset for `shards.len() <=
128`; fall back to the existing `HashSet` only above that.

**Estimated impact:** one allocation removed per poll; micro-win, but
trivially small diff and complements #11.

---

### 13. Public `select_shard()` re-serializes the JSON value

**File:** `src/shard/mod.rs:358-376`

```rust
pub fn select_shard(&self, event: &JsonValue) -> u16 {
    let bytes = serde_json::to_vec(event).expect("Value serialization is infallible");
    let hash = xxhash_rust::xxh3::xxh3_64(&bytes);
    ...
}
```

The internal hot path (`ingest`, `ingest_raw`, `ingest_raw_batch`) all
correctly use `select_shard_by_hash` and serialize exactly once, so
this is **not** a hot-path issue inside the crate. But the method is
`pub`, so any SDK / FFI consumer that calls it as part of its own
ingestion routing pays a second `serde_json::to_vec` for the same
event the bus is about to serialize again.

**Fix.** Either deprecate `select_shard(&JsonValue)` in favor of
`RawEvent::from_value(...).hash()` + `select_shard_by_hash`, or make
the public method take `&RawEvent` directly. Document the hash-then-
select pattern as the intended public API.

**Estimated impact:** zero on internal benchmarks; up to 2× alloc
savings for external producers that don't follow the `RawEvent`
pattern.

---

## LOW / already in good shape

- `Cargo.toml` release profile: LTO=fat, `codegen-units=1`, `panic=abort`,
  `opt-level=3`. Already tuned.
- Tokio features minimal (`rt-multi-thread, sync, time, macros, net`),
  no `full`.
- Ring buffer uses `crossbeam_utils::CachePadded` on head/tail —
  false-sharing handled.
- `xxhash-rust` for hashing — already optimal.
- `thiserror`-based errors with `String` payloads — no `Box<dyn Error>`
  per-event.

---

## Suggested order of attack

1. **#1 — JetStream pipelining.** Single biggest gain in the crate; the
   change is local to `on_batch`.
2. **#4 — Notify-based drain worker.** Biggest latency improvement;
   modest effort.
3. **#3, #2 — Kill per-event String allocations in adapters.**
   Straightforward, measurable.
4. **#5 — Batch ingest cleanup.** Touches a hot path and simplifies the
   locking story.
5. **#6 — `Acquire` → `Relaxed`.** 15-minute change; measure to confirm.

Each item should land with a benchmark in `benches/` or a numbers
update in `BENCHMARKS.md` so regressions are caught.
