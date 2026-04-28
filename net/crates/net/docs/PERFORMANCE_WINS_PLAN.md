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

### 14. Shard mutex serializes producers around the lock-free ring buffer

**File:** `src/shard/mod.rs:241, 128, 461, 480, 527`

```rust
// mod.rs:241 — routing table holds shards behind a mutex
shards: Vec<Arc<parking_lot::Mutex<Shard>>>,

// mod.rs:128 — try_push_raw takes &mut self, which forces the lock
pub fn try_push_raw(&mut self, raw: Bytes) -> Result<u64, IngestionError> { ... }

// mod.rs:461 — every single-event ingest path acquires the mutex
let mut shard = shard_lock.lock();
self.push_with_backpressure(&mut shard, shard_id, raw)
```

The `RingBuffer::try_push` underneath is `&self` and lock-free, with
cache-padded head/tail and a documented SPSC contract. The wrapping
`parking_lot::Mutex<Shard>` makes the producer side single-threaded
*via the mutex* — correct in the sense that it preserves SPSC, but it
throws away the point of the lock-free ring. Every multi-producer
ingest into a single shard now pays a mutex acquisition.

Nothing the producer touches actually needs `&mut self`:
`RingBuffer::try_push` is already `&self`; `counters` are atomics;
`metrics_collector` is an `Arc`; `timestamp_gen.next()` is internally
CAS-based.

**Fix.** Two viable options:

1. **Switch to a true MPSC lock-free queue.** `crossbeam-queue::ArrayQueue`
   (already pulled in under the `net` feature) is a bounded MPMC
   lock-free ring. Producer side goes lockless under contention;
   consumer side too. Drop `Mutex<Shard>` entirely — the batch worker
   becomes the only consumer post-#4 and can use `try_pop` directly.
2. **Keep the SPSC ring, flip `Shard::try_push_raw` to `&self`.** More
   delicate: SPSC requires effectively single-producer, which the
   mutex preserves but a bare `Arc<Shard>` does not. Only viable if
   you can guarantee per-shard producer affinity (pinned executor /
   thread-local). Doing this naively reintroduces the UB call out at
   `ring_buffer.rs:62-67`.

Option 1 is the safer default; the dep already exists.

Interactions with other items:

- #4 (Notify drain worker) eliminates *consumer*-side empty-poll
  mutex acquisitions; #14 eliminates the producer-side ones. Land
  both for the full picture.
- #5's third sub-point ("verify `push_with_backpressure` is a pure
  fast path") becomes moot once the lock is gone.
- #9 (timestamp CAS): a hot CAS on a per-shard `AtomicU64` becomes
  load-bearing once producers go truly concurrent. Keep the CAS;
  re-evaluate the spin hint under real contention.

**Estimated impact:** uncontended (one producer per shard, e.g. one
tokio worker pinned per shard), ~20-40 ns saved per ingest. Under
realistic multi-tokio-worker load where traffic lands in the same
shard, expect **2-10× publish throughput** at the boundary. Single
biggest perf change available in the crate.

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

### 15. `RingBuffer::pop_batch` allocates a fresh `Vec<T>` every call

**File:** `src/shard/ring_buffer.rs:199-242`

```rust
pub fn pop_batch(&self, max: usize) -> Vec<T> {
    ...
    let mut result = Vec::with_capacity(count);
    for i in 0..count { result.push(...); }
    ...
    result
}
```

Called by the drain worker on every cycle (`bus.rs:793, 806`). The
returned `Vec` is sent over an mpsc, consumed, and dropped — so each
cycle allocates a buffer the consumer immediately frees. Under load
this is 100k+ allocs/sec/shard.

**Fix.** Add `pop_batch_into(&self, dst: &mut Vec<T>, max: usize) -> usize`
that drains into a caller-owned buffer; keep the allocating variant
for ergonomics. The drain/batch worker keeps one reused `Vec`.

If #4 collapses drain+batch into one task (which it should), this
becomes "drain directly into the worker's `current_batch`" — no
intermediate allocation at all. Land #4 first, then this becomes a
trivial follow-on.

**Estimated impact:** one heap alloc + free removed per drain cycle
per shard. Modest but free.

---

### 16. `ShardTable::shard_index` uses SipHash; a dense `Vec` is strictly better

**File:** `src/shard/mod.rs:246, 393-402`

```rust
shard_index: std::collections::HashMap<u16, usize>,
...
fn resolve_idx(&self, table: &ShardTable, shard_id: u16) -> Option<usize> {
    if self.mapper.is_none() {
        Some(shard_id as usize)              // static-mode fast path
    } else {
        table.shard_index.get(&shard_id).copied()  // dynamic mode
    }
}
```

The static-mode path bypasses the map, so this only matters for
dynamic scaling. But there it runs once per event in the hot path,
through `HashMap::get(&u16)` with the default SipHash — chosen for
HashDoS resistance, irrelevant for u16 ids the bus assigns itself.

**Fix.** Either swap to `rustc_hash::FxHashMap` (drop-in, ~2-3× faster
hash) or — since shard ids are bounded and small — use
`Vec<Option<usize>>` indexed by `shard_id as usize`. The dense Vec is
strictly better: bounds check + load, no hashing. With shard counts
that can grow during scaling, also consider a sparse `Vec` keyed by
`shard_id` allocated lazily up to the current max id.

**Estimated impact:** ~10-20 ns/lookup → ~2 ns. Per-event in dynamic
scaling mode; zero in static mode. Pairs with #5's hoisting of
`resolve_idx`.

---

### 17. Cursor-advance check compares two `HashMap`s for full equality every poll

**File:** `src/consumer/merge.rs:402`

```rust
let cursor_advanced = final_cursor.positions != cursor.positions;
```

Sets a single boolean by structurally comparing two
`HashMap<u16, Arc<str>>` — every shard's `Arc<str>` content compared
end-to-end. The mutations that actually change the cursor are
explicit and trackable: line 290 (`nc.set(...)` from adapter result)
and line 398 (`final_cursor.set(event.shard_id, event.id.clone())`).

**Fix.** Maintain `cursor_advanced: bool` directly: set it `true` at
those two mutation sites, drop the equality comparison. The
equivalent flag is both faster and easier to reason about than the
structural compare.

**Estimated impact:** O(shards) Arc-content compares saved per poll —
~200-400 ns at 16 shards. Adds up at high consumer poll rates.

---

### 18. `EventBus::ingest_batch` allocates an intermediate `Vec<RawEvent>` for type conversion

**File:** `src/bus.rs:441-442`

```rust
let raw: Vec<RawEvent> = events.into_iter().map(|e| e.into_raw()).collect();
self.ingest_raw_batch(raw)
```

Pure type-conversion alloc: the `Vec<RawEvent>` exists only to be
consumed once by `ingest_raw_batch`.

**Fix.** Make `ingest_raw_batch` (here and `ShardManager::ingest_raw_batch`
at `mod.rs:492`) accept `impl IntoIterator<Item = RawEvent>`. The
`into_raw()` map then fuses with the bucket-grouping pass — no
intermediate `Vec`. Combines naturally with #5.

**Estimated impact:** one heap alloc removed per `ingest_batch` call.
Trivial diff.

---

### 19. `PollMerger::poll` allocates a shards `Vec` on every all-shards poll

**File:** `src/consumer/merge.rs:225-228`

```rust
let shards: Vec<u16> = request
    .shards
    .clone()
    .unwrap_or_else(|| (0..self.num_shards).collect());
```

When the caller didn't specify any shards (the common case), this
allocates a fresh `Vec<u16>` of `0..N` just to iterate it once at
line 247.

**Fix.** Split the two paths: borrow `request.shards.as_deref()`
when present, iterate `0..self.num_shards` directly otherwise. No
allocation on the common path.

**Estimated impact:** one ~128-byte allocation removed per poll at
64 shards. Low per-call but high call frequency on consumer-heavy
deployments.

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

1. **#1 — JetStream pipelining.** Biggest single adapter gain; local to
   `on_batch`.
2. **#14 + #4 together — drop the shard mutex, switch the drain worker
   to `Notify`.** Eliminates producer-side and consumer-side mutex
   traffic on the same shard at once; biggest publish-side throughput
   win in the crate. Land as a paired change so the SPSC/MPSC contract
   never goes stale between commits.
3. **#3, #2 — Kill per-event String allocations in adapters.**
   Straightforward, measurable.
4. **#5 + #16 + #18 — Batch ingest cleanup.** Hoist `resolve_idx`,
   replace `shard_index` HashMap with a dense Vec, take
   `IntoIterator<Item = RawEvent>`. Touches the same code; do in one
   pass.
5. **#15 — `pop_batch_into`.** Trivial follow-on once #4 is in place
   (or "drain straight into the worker buffer" if drain+batch get
   collapsed).
6. **#17 — cursor-advance flag.** Pairs with #11/#12 in the merge
   path; one-file diff.
7. **#6 — `Acquire` → `Relaxed`.** 15-minute change; measure to confirm.

Each item should land with a benchmark in `benches/` or a numbers
update in `BENCHMARKS.md` so regressions are caught.
