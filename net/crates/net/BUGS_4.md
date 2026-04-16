# Known Bugs — Sweep 4

## Critical — Data loss

### 1. Net adapter `poll_shard` permanently loses events that don't pass cursor filter
- **File**: `src/adapter/net/mod.rs:800-809`
- Events are popped from the `SegQueue` (destructive read), then checked against `from_id`. Events at or before the cursor are silently dropped — removed from the queue but never returned. This causes permanent data loss during cursor-based pagination.
- **Fix**: Collect into a temp buffer and re-enqueue events that don't match, or use a non-destructive index-based approach.

## High — Incorrect ordering

### 2. Net adapter `poll_shard` uses lexicographic comparison on numeric event IDs
- **File**: `src/adapter/net/mod.rs:803`
- Event IDs are formatted as `"{seq}:{idx}"` (line 531). String comparison is lexicographic: `"9:0" > "10:0"` is `true` because `'9' > '1'`. This causes events to be incorrectly included or excluded during pagination.
- **Fix**: Parse the sequence number numerically for comparison, or use zero-padded formatting.

## Medium — Silent data corruption

### 3. `StoredEvent::Serialize` silently replaces invalid raw bytes with `null`
- **File**: `src/event.rs:382`
- `serde_json::from_slice(&self.raw).unwrap_or(JsonValue::Null)` silently substitutes `null` if raw bytes are not valid JSON. Same class of bug as the `raw_str()` fix (BUGS_3 #4) but in the `Serialize` impl path.
- **Fix**: Return a serialization error, or serialize raw bytes as a string with an error indicator.

### 4. `net_poll` FFI silently replaces unparseable events with `null`
- **File**: `src/ffi/mod.rs:557`
- `e.parse().unwrap_or(serde_json::Value::Null)` in the poll response means corrupted events become `null` with no indication to the caller.
- **Fix**: Filter out unparseable events and log a warning, or embed the raw bytes with an error flag.

### 5. `DropOldest` double-counts the new event as both dropped and ingested
- **File**: `src/shard/mod.rs:350-361`
- When the initial `try_push_raw` fails, `Shard` increments `events_dropped`. The retry then succeeds and increments `events_ingested`. The new event is counted as both dropped and ingested. The oldest event that was actually discarded is never counted. `events_ingested + events_dropped` exceeds actual submissions.
- **Fix**: Decrement `events_dropped` after a successful retry, or bypass the stats-incrementing push for the retry.

## Low — Edge cases

### 6. `AdaptiveBatcher::total_events` can overflow `usize` on 32-bit targets
- **File**: `src/shard/batch.rs:51`
- `total_events: usize` is incremented on every `record_events()` call and never reset (only velocity samples are pruned). On 32-bit targets, wraps after ~4B events, corrupting velocity calculations.
- **Fix**: Use `u64` instead of `usize`, or use `wrapping_add` with wrap-aware velocity math.

### 7. `current_timestamp()` truncates `u128` nanoseconds to `u64`
- **File**: `src/adapter/net/session.rs:354-359`
- `Duration::as_nanos()` returns `u128`, cast to `u64`. Safe until ~year 2554 for wall-clock time, but `is_timed_out()` also casts `timeout.as_nanos() as u64` which truncates for extreme timeout values.
- **Fix**: Use `u64::try_from(...).unwrap_or(u64::MAX)` for the timeout cast.
