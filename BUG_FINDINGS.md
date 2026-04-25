# Bug findings

Combined audit notes. Severity is the reporter's best estimate; race-window bugs are real but narrow.

## Confirmed bugs

### 1. `MigrationTargetHandler::complete()` breaks its own idempotency contract (low)
**File:** `net/crates/net/src/adapter/net/compute/migration_target.rs:297-324`

`complete()` removes from `self.migrations` (line 302) *before* inserting into `self.completed` (line 312). A concurrent retried `activate()` from the orchestrator (e.g. after a lost `ActivateAck`) that lands in that gap walks both maps:

- `migrations.get()` → None (we just removed it)
- `completed.get()` → None (we haven't inserted yet)

…and returns `DaemonNotFound` (line 286), even though the doc-comment at `activate()` line 266-270 explicitly promises idempotent behavior for exactly this retry. Window is microseconds, gated on a network retry — but the failure mode is "subprotocol returns an error for a successfully-completed migration."

**Fix:** swap the order in `complete()` — insert into `completed` first, then remove from `migrations`. Re-draining a Cutover-phase state in `activate()` is idempotent (drain_pending advances `replayed_through` monotonically and removes drained events from the BTreeMap).

### 2. `CircuitBreaker::allow()` can silently undo a `reset()` (low)
**File:** `net/crates/net/src/adapter/net/failure.rs:522-539`

`allow()` reads `state` under the read lock, sees `Open` with elapsed timeout, drops the lock (line 530), then calls `transition_to(HalfOpen)` (line 531). If `reset()` (line 615 — public API) fires between the read-lock release and the write-lock acquisition, it transitions to Closed and `allow()`'s subsequent `transition_to(HalfOpen)` clobbers Closed back down to HalfOpen. The reset is silently undone.

`record_success` and `record_failure` (lines 542-602) explicitly hold the write lock across the read-decide-transition path *and the comments say they do this specifically to avoid TOCTOU* — `allow()` is the lone exception that didn't get the same fix.

**Fix:** acquire the write lock once in `allow()`, re-check `*state == Open` and the elapsed time inside it, then transition; or pass the expected `from` state to a new `transition_locked_if(from, to)` that no-ops on mismatch.

### 3. Corrupt events silently dropped from filtered queries (medium)
**File:** `net/crates/net/src/consumer/merge.rs:304,310`

```rust
event.parse().map(|v| filter.matches(&v)).unwrap_or(false)
```

A `StoredEvent` with malformed JSON in `raw` is treated as "doesn't match" rather than surfacing an error. A consumer with any filter will silently lose corrupted events instead of seeing them or learning that storage is corrupt. The unfiltered path returns the event raw, so behavior is inconsistent.

**Fix:** either propagate the error, or at minimum log/metric on parse failure so corruption isn't invisible.

### 4. Node `Stats` silently saturates at `i64::MAX` (medium)
**File:** `net/crates/net/bindings/node/src/lib.rs:574-582`

```rust
events_ingested: stats.events_ingested.load(...).min(i64::MAX as u64) as i64,
```

napi-rs supports `BigInt`; clamping `u64` to a signed JS number both loses precision past 2^53 and silently caps at `i64::MAX`. Long-running nodes will report wrong totals with no signal.

**Fix:** change `Stats` fields to `BigInt`.

### 5. `tags.clone()` inside `b.iter()` (medium, perf-bench validity)
**File:** `net/crates/net/benches/cortex.rs:65`

```rust
b.iter(|| { ...; memories.store(id, "content", tags.clone(), "source", 0).unwrap() });
```

Vec allocation/copy is timed as part of `store`, inflating the result.

**Fix:** pass `&tags` or move the clone outside.

### 6. `ShardManager` shared across thread-count variants (medium, perf-bench validity)
**File:** `net/crates/net/benches/parallel.rs:18`

```rust
for num_threads in [1, 2, 4, 8].iter() {
    let manager = Arc::new(ShardManager::new(16, 1 << 20, BackpressureMode::DropOldest));
```

The same manager carries pending state from the 1-thread run into the 2/4/8-thread runs (and across `b.iter()` invocations within each), so the comparison isn't apples-to-apples and depends on prior state.

**Fix:** construct inside `bench_with_input`.

### 7. `pop_batch` bench depends on running steady-state (low-medium)
**File:** `net/crates/net/benches/ingestion.rs:87-104`

Buffer is pre-filled once, then each iteration pops `size` and refills `batch.len()`. Throughput measured here mixes pop + push + branch on partial pops.

**Fix:** either bench pop in isolation (refill outside the timed body) or document this is a steady-state bench.

### 8. Python SDK: `TypedChannel.subscribe()` mutates caller's `SubscribeOpts`
**File:** `net/crates/net/sdk-py/src/net_sdk/channel.py:74-103`

```python
def subscribe(self, opts: Optional[SubscribeOpts] = None) -> TypedEventStream[T]:
    merged = opts or SubscribeOpts()
    if merged.filter is None:
        merged.filter = self._filter   # mutates caller's opts
    ...

def subscribe_raw(self, opts: Optional[SubscribeOpts] = None) -> EventStream:
    merged = opts or SubscribeOpts()
    if merged.filter is None:
        merged.filter = self._filter   # same bug
```

`SubscribeOpts` is a `@dataclass` (mutable), and `merged = opts or ...` aliases the user's object. Trigger:

```python
opts = SubscribeOpts(limit=50)
temps.subscribe(opts)        # opts.filter is now temps's channel filter
humidity.subscribe(opts)     # second call sees opts.filter != None, keeps temps's filter
                             # → "humidity" subscription silently delivers temps events
```

Note: the equivalent TS code at `net/crates/net/sdk-ts/src/channel.ts:82-86` correctly uses `{ ...opts, filter: ... }` to copy. The Python side regressed.

**Fix:** copy first, e.g. `merged = replace(opts) if opts else SubscribeOpts()`, or just `merged = SubscribeOpts(**asdict(opts)) if opts else SubscribeOpts()`.

### 9. Go zero-copy batch ingest — likely violates cgo pointer rules (high)
**File:** `net/crates/net/bindings/go/net/net.go:261-278` (`IngestRawBatch`)

```go
ptrs := make([]*C.char, len(jsons))
for i, j := range jsons {
    if len(j) > 0 {
        ptrs[i] = (*C.char)(unsafe.Pointer(unsafe.StringData(j)))
    } else {
        ptrs[i] = (*C.char)(unsafe.Pointer(&emptyByte[0]))
    }
    ...
}
result := C.net_ingest_raw_batch(
    bs.handle,
    (**C.char)(unsafe.Pointer(&ptrs[0])),
    ...
)
```

`unsafe.StringData(j)` returns a Go-managed pointer (into the string's Go-heap backing store). Storing those into `ptrs[]` makes `ptrs` "Go memory containing Go pointers." Passing `&ptrs[0]` to C is exactly the case the cgo rules forbid: *"Go code may pass a Go pointer to C provided that the Go memory to which it points does not contain any Go pointers."* Single-string `IngestRaw` (line 237) is fine — that's a flat `*C.char` to bytes — but the batch variant adds an indirection that crosses the line. May silently work today (the `unsafe.Pointer` cast can hide it from cgo's runtime checker depending on Go version), but it's officially UB. Worth running the integration test under `GODEBUG=cgocheck=2`.

### 10. Capability fast-path skips per-dimension re-check on stale index (medium)
**File:** `net/crates/net/src/adapter/net/behavior/capability.rs:1563-1568`

```rust
if !filter.needs_full_check() {
    return candidates
        .into_iter()
        .filter(|&node_id| self.nodes.contains_key(&node_id))
        .collect();
}
```

Index updates in `index()` (lines 1315-1332) do `remove_from_indexes(old)` → `add_to_indexes(new)` → `nodes.insert(new)` non-atomically. During an update where `caps.tags` changes from `["foo"]` to `["bar"]`, a concurrent `query(filter{tag:"foo"})` can hit a window where the tag-"foo" index still lists the node (or has just been removed), and the fast path returns it via `contains_key` without verifying the current capability set. The slow path's `filter.matches()` would have caught this. Old code always took the slow path. Race window is brief and the comment acknowledges eviction (but not capability mutation), so medium not critical.

### 11. Go `Poll` retry loop missing `KeepAlive(buffer)` (low)
**File:** `net/crates/net/bindings/go/net/net.go:340-358`

`buffer := make([]byte, bufferSize)` is taken by `&buffer[0]` and handed to `C.net_poll`. There's `KeepAlive(cRequestBuf)` after the call but no `KeepAlive(buffer)`. In practice Go's escape analysis keeps `buffer` alive because it's used after (`buffer[:result]`), and Go GC is non-moving, so this works today. Flagging it because the PR added explicit `KeepAlive(cRequestBuf)` for the request — the matching response-buffer KeepAlive was missed.

## Possible bugs

### `Redex::open_file` can leak a fsync task and dup file handles on concurrent first-open
**Files:** `net/crates/net/src/adapter/net/redex/manager.rs:106-126`, `net/crates/net/src/adapter/net/redex/file.rs:113,156`

`manager.rs:106-126` lets two threads both run `build_file()` (which calls `RedexFile::open_persistent` at `file.rs:113`) before either reaches `self.files.entry(name).or_insert(file)`. Both opens spawn an Interval fsync task in `file.rs:156` and both hold their own `Arc<DiskSegment>` with separate file handles to the same `idx`/`dat` files. The loser of the `or_insert` race is dropped without `close()` being called — so its `interval_shutdown` Notify is never fired, and its tokio task lives until the runtime ends, periodically syncing its own (leaked) file handles to the same disk file the winner uses.

Requires: a tokio runtime, `persistent: true`, `FsyncPolicy::Interval`, and exactly-concurrent first-open. Real but niche.

## Rejected after verification

For the record — claims that did not survive reading the code:

- **"Token expiration not validated before authorization"** (`mesh.rs:4348`) — `TokenCache::check()` at `token.rs:478,488` calls `is_valid().is_ok()` which validates time bounds. The `verify()`-only filter is intentional (signature gate before caching).
- **"FFI `free_events_array` mismatched allocator"** (`ffi/mod.rs:1160-1171`) — standard `Box<[u8]>` round-trip pattern through C FFI; `slice_from_raw_parts_mut(ptr, len)` rebuilds the original fat pointer.
- **"`PermissionToken::from_bytes` unwraps panic on truncation"** (`token.rs:353-360`) — unreachable; `data.len() != WIRE_SIZE` is checked first.
- **"`wait_for_seq` lost wakeup"** (`cortex/adapter.rs:103-122`) — `notified.as_mut().enable()` runs *before* the watermark load, which is the correct tokio Notify pattern.
- **"Replay-window poison recovery is unsafe"** (`crypto.rs:670-688`) — backwards; preserving the seen-counter set on poison is the safe choice.
- **"`session_prefix_from_id` XOR collision"** (`crypto.rs:511`) — by design, defense-in-depth atop the per-packet counter.
- **"NetDb cross-model snapshot isn't atomic"** (`db.rs:105-121`) — documented limitation.
- **"`net_identity_entity_id`/`sign` write fixed bytes without size param"** (`ffi/mesh.rs:1788, 1834`) — standard C FFI contract; size is fixed and documented in the header.
