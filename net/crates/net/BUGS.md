# Known Bugs

## Critical — Panics in library code

### 1. Unwrap in ingestion hot path
- **File**: `src/shard/mod.rs:320`
- `.unwrap()` on `serde_json::to_vec()` in `ShardManager::ingest()`. If serialization fails (deeply nested input, custom serializer error), the whole process panics.

### 2. Unwrap in RawEvent::from_value
- **File**: `src/event.rs:147`
- Same `.unwrap()` on `serde_json::to_vec()`. Library code should never panic on user input.

### 3. Unwrap in ShardManager initialization
- **File**: `src/shard/mod.rs:249`
- `.unwrap()` on `metrics_collector()` during `ShardManager::with_mapper()`. Panics if the collector returns `None`.

## High — Logic bugs

### 4. Dropped events not counted in DropOldest mode
- **File**: `src/shard/mod.rs:347-351`
- In `DropOldest` backpressure mode, the popped event is silently discarded (`let _ = shard.try_pop()`) without incrementing any dropped-event counter. Stats will undercount drops.

### 5. Redundant dead logic in ring buffer full check
- **File**: `src/shard/ring_buffer.rs:94-112`
- The first fullness check (lines 99-106) is entirely dead code — the second check (lines 109-112) catches every case the first one does. Misleading dead logic that should be removed.

## Medium — Defensive gaps

### 6. Unvalidated shard index in static mode
- **File**: `src/shard/mod.rs:326-327`
- In static mode, `shard_id as usize` is used as a direct index. Guarded by `.get(idx)` on line 336 so it won't panic, but a hash collision or misconfiguration could silently route to the wrong shard.

### 7. Stale index after swap_remove in remove_shard
- **File**: `src/shard/mod.rs:516-524`
- `remove_shard()` holds both write locks so it's not a data race, but the `swap_remove` + index update pattern means any concurrent reader that cached an old index before the write lock could get stale data after the lock releases.
