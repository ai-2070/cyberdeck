# Known Bugs (Fixed)

All bugs below have been fixed and regression tests added.

## Critical — Panics in library code (FIXED)

### 1. Unwrap in ingestion hot path
- **File**: `src/shard/mod.rs` — `ShardManager::ingest()`
- `.unwrap()` on `serde_json::to_vec()`. Now returns `IngestionError::Serialization`.

### 2. Unwrap in from_value methods
- **Files**: `src/event.rs` — `RawEvent::from_value()`, `InternalEvent::from_value()`, `StoredEvent::from_value()`
- `.unwrap()` replaced with `expect("Value serialization is infallible")`. These take `serde_json::Value` which cannot fail to serialize, so `Result` was unnecessary — `expect` documents the invariant without polluting every call site.

### 3. Unwrap in ShardManager initialization
- **File**: `src/shard/mod.rs` — `ShardManager::with_mapper()` and `add_shard()`
- `.unwrap()` on `metrics_collector()`. Now returns `ScalingError::InvalidPolicy`.

## High — Logic bugs (FIXED)

### 4. Redundant dead logic in ring buffer full check
- **File**: `src/shard/ring_buffer.rs` — `try_push()`
- Removed the redundant first fullness check. Single clean check remains.

## Medium — Defensive gaps (unchanged, documented)

### 5. Unvalidated shard index in static mode
- **File**: `src/shard/mod.rs:326-327`
- In static mode, `shard_id as usize` is used as a direct index. Guarded by `.get(idx)` on line 336 so it won't panic, but a hash collision or misconfiguration could silently route to the wrong shard.

### 6. Stale index after swap_remove in remove_shard
- **File**: `src/shard/mod.rs:516-524`
- `remove_shard()` holds both write locks so it's not a data race, but the `swap_remove` + index update pattern means any concurrent reader that cached an old index before the write lock could get stale data after the lock releases.
