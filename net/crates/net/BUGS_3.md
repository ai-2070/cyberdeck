# Known Bugs — Sweep 3

## Critical — FFI safety

### 1. `net_shutdown` drops handle without flushing — silent data loss
- **File**: `src/ffi/mod.rs:672-684`
- `net_shutdown` simply `drop()`s the `NetHandle` instead of calling `EventBus::shutdown()`. Pending events in ring buffers and batch workers are silently lost. The Tokio runtime is aborted, killing all spawned tasks (drain workers, batch workers, scaling monitor) without letting them complete.
- **Fix**: Call `handle.runtime.block_on(handle.bus.shutdown())` before dropping. Requires restructuring since `shutdown(self)` consumes the bus.

## Medium — Data corruption / resource leaks

### 2. `net_poll` / `net_stats` return value overflow
- **File**: `src/ffi/mod.rs:581, 634`
- `response_json.len() as c_int` silently wraps to a negative number if the JSON exceeds `i32::MAX` bytes. Callers interpret negative values as error codes.
- **Fix**: Bounds-check with `c_int::try_from(len)` and return `BufferTooSmall` on overflow.

### 3. `net_poll_ex` leaks memory on CString error path
- **File**: `src/ffi/mod.rs:912-916`
- If `CString::new` fails on the cursor string (null byte in cursor), the function returns early. The already-allocated `events_ptr` array and all `id`/`raw` Box allocations are leaked because `out` was never populated and the caller cannot call `net_free_poll_result`.
- **Fix**: Free the events array before the early return.

### 4. `StoredEvent::raw_str()` silently corrupts data for invalid UTF-8
- **File**: `src/event.rs:365-368`
- `std::str::from_utf8(&self.raw).unwrap_or("{}")` silently replaces invalid UTF-8 with an empty JSON object. Since `RawEvent::from_bytes` performs no validation, corrupt data can flow through the system and be silently replaced with `{}` on read.
- **Fix**: Return `Result` or validate UTF-8 at the `from_bytes` boundary.

### 5. JetStream gap search range can cause consumers to miss events
- **File**: `src/adapter/jetstream.rs:294`
- `max_seq = start_seq + (fetch_limit * 10)` is an arbitrary cap. If the stream has large gaps (deletions, compaction), the consumer hits the cap before finding enough events and may report `has_more = false` when more events exist.
- **Fix**: Use the stream's `info()` to get the actual last sequence number, or increase the multiplier and use `saturating_add`.

## Low — Edge cases

### 6. Timestamp monotonicity breaks at `u64::MAX`
- **File**: `src/timestamp.rs:60`
- `last.saturating_add(1)` at `u64::MAX` returns `u64::MAX`, so the CAS succeeds returning the same timestamp twice, breaking the documented "strictly monotonic" guarantee.
- **Fix**: Check for saturation and return an error or panic.

### 7. `ScalingPolicy::high_throughput()` can overflow `max_shards` on u16
- **File**: `src/config.rs:597`
- `cpus * 2` can overflow `u16` on machines with >32K logical CPUs. Panics in debug, wraps silently in release.
- **Fix**: Use `cpus.saturating_mul(2)`.

### 8. `num_cpus()` truncates `usize` to `u16`
- **File**: `src/config.rs:660-664`
- `n.get() as u16` silently truncates on systems with >65535 CPUs.
- **Fix**: Use `u16::try_from(n.get()).unwrap_or(u16::MAX)`.

### 9. `max_seq` integer overflow in JetStream adapter
- **File**: `src/adapter/jetstream.rs:294`
- `start_seq + (fetch_limit as u64 * 10)` wraps if `start_seq` is near `u64::MAX`, causing the loop to return zero events.
- **Fix**: Use `saturating_add`.

### 10. `Mutex::lock().unwrap()` in JetStream adapter cascades panics
- **File**: `src/adapter/jetstream.rs:114, 159, 265`
- Uses `std::sync::Mutex` which poisons on panic. A prior panic causes all subsequent operations to also panic. Rest of the codebase uses `parking_lot::Mutex` (no poisoning).
- **Fix**: Switch to `parking_lot::Mutex` for consistency.
