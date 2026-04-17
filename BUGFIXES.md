# Net Bug Audit

Audit date: 2026-04-16
Status: **All 39 bugs fixed. 847 tests pass (0 failures).**

All paths relative to `net/crates/net/`.

---

## Critical

### 1. Use-after-free race in `net_shutdown` (FFI)

**File:** `src/ffi/mod.rs:738-743`

Between setting `shutting_down = true` and `Box::from_raw(handle)`, another thread can still be inside an FFI function that already passed the shutdown guard. The handle is deallocated while the other thread still holds a reference to it.

**Fix:** Use `Arc` for the handle so concurrent callers keep it alive, or add an `RwLock` so `net_shutdown` waits until all in-flight operations complete before deallocating.

---

## High

### 2. Node.js `shutdown()` permanently loses EventBus

**File:** `bindings/node/src/lib.rs:512-533`

`shutdown()` atomically swaps the bus out (`self.bus.swap(None)`), then tries `Arc::try_unwrap`. If `try_unwrap` fails (outstanding references from concurrent `poll`/`load_full` calls), the `Arc<EventBus>` is dropped in the `Err` arm without being put back. The bus is permanently gone and never shut down gracefully, leaking background tasks and connections.

**Fix:** In the `Err(arc)` branch, restore the bus with `self.bus.store(Some(arc))` before returning the error.

### 3. Cursor never advances for filtered-out shards

**File:** `src/consumer/merge.rs:250-276`

`PollMerger::poll` builds `final_cursor` from only the events that survive filtering. Shards whose events are entirely filtered out never have their cursor advanced, causing infinite re-fetching of the same events.

**Fix:** Use `new_cursor` (which tracks fetched positions) instead of building `final_cursor` from returned events only. Take the max of `final_cursor` and `new_cursor` per shard.

### 4. `flush()` uses hardcoded 50ms sleep instead of drain confirmation

**File:** `src/bus.rs:470-474`

`flush()` sleeps for a fixed 50ms hoping drain workers will have processed all events. Under high load or slow adapters, this is insufficient. There is no mechanism to signal that all ring buffer contents have been drained and all in-flight batches dispatched.

**Fix:** Implement a barrier or in-flight event counter that `flush` can wait on.

### 5. No `Drop` impl for `EventBus` — orphaned background tasks

**File:** `src/bus.rs:50-71`

If an `EventBus` is dropped without calling `shutdown()` (early return, `?` propagation, panic), background tasks (batch workers, drain workers, scaling monitor) loop forever. `JoinHandle`s are dropped (detaching tasks), but the tasks hold `Arc` references preventing cleanup. Leaks memory and connections.

**Fix:** Implement `Drop for EventBus` that sets the shutdown flag. Cannot await futures in `Drop`, but setting the atomic flag triggers eventual task termination.

---

## Medium

### 6. Thread-local builder pool shared across all `ThreadLocalPool` instances

**File:** `src/adapter/net/pool.rs:430-433`

`LOCAL_BUILDERS` is a single process-wide `RefCell<Vec<PacketBuilder>>`. Builders from different `ThreadLocalPool` instances (potentially with different encryption keys) can intermingle. If two pools share a session_id but use different keys, the session_id check passes and packets are silently encrypted with the wrong key.

**Fix:** Key the thread-local cache by a unique pool ID (e.g., `HashMap<u64, Vec<PacketBuilder>>`).

### 7. `place_with_spread` ignores exclusion set

**File:** `src/adapter/net/compute/group_coord.rs:246-253`

The `_exclude` parameter is never used. Fork/replica placement does not enforce node spread, defeating the purpose of spread logic in `ForkGroup::fork` and `scale_to`.

**Fix:** Filter candidate nodes against `_exclude` before selecting placement.

### 8. CircuitBreaker TOCTOU between state read and transition

**File:** `src/adapter/net/failure.rs:523-555`

`record_success()` and `record_failure()` read the state with a read lock, drop it, then make decisions and call `transition_to()` which acquires a write lock. Between dropping the read lock and acquiring the write lock, another thread can change the state. A circuit breaker can re-open after legitimate recovery.

**Fix:** Hold the write lock through the read-decide-transition path, or use a single `Mutex<CircuitState>` with counter checks inside the critical section.

### 9. `LossSimulator::burst_remaining` wraps to `u64::MAX`

**File:** `src/adapter/net/failure.rs:388-392`

`should_drop` loads `burst_remaining`, checks `> 0`, then does `fetch_sub(1)`. Under concurrent access with `remaining = 1`, multiple threads can each `fetch_sub(1)`, wrapping to `u64::MAX`. All subsequent packets are dropped.

**Fix:** Use `fetch_update` or a compare-and-swap loop to atomically check-and-decrement.

### 10. `FairScheduler` streams never cleaned up

**File:** `src/adapter/net/router.rs:122-277`

`cleanup_empty()` exists but is never called. Each unique `stream_id` creates a `DashMap` entry with an `ArrayQueue`. Over a long-running router, memory grows without bound. The `idle_timeout_ns` config field is never referenced.

**Fix:** Periodically call `cleanup_empty()` from the send loop, or integrate `idle_timeout_ns` to remove idle streams (requires tracking last-activity time per stream).

### 11. `scale_down` doesn't re-validate under lock

**File:** `src/shard/mapper.rs:573-618`

`to_drain` is computed from an `active_count` read before acquiring the write lock. Between reading and locking, another thread could `scale_up` or `scale_down`, making `to_drain` exceed actual active shards.

**Fix:** Re-read and re-validate `active_count` and `to_drain` after acquiring the write lock, matching the double-check pattern in `scale_up`.

### 12. `finalize_draining` destructively resets metrics

**File:** `src/shard/mapper.rs:634`

`finalize_draining()` calls `collect_and_reset()` to check if a draining shard is empty, which zeros all window counters as a side effect. If called frequently, `collect_metrics()` never sees accurate data for draining shards.

**Fix:** Use a read-only metrics snapshot method for the draining check. Only `collect_metrics()` should reset counters.

### 13. Non-atomic metric window reset

**File:** `src/shard/mapper.rs:177-210`

`collect_and_reset()` performs multiple independent atomic `swap` operations that are not collectively atomic. Between swapping `events_in_window` and `push_count`, a concurrent producer can lose a push's latency contribution while its count is captured in the next window. Causes systematic metric drift at high throughput.

**Fix:** Protect `collect_and_reset` with a lightweight mutex, or accept and document the small inaccuracy.

### 14. `push_with_hash` ignores BigInt sign/overflow (Node.js)

**File:** `bindings/node/src/lib.rs:259`

`hash.get_u64()` returns `(sign, value, lossless)` but the code ignores `sign` and `lossless`. A negative BigInt or a value exceeding u64 range produces a wrong hash, routing the event to the wrong shard.

**Fix:** Check `lossless == true` and `sign == false`, returning an error if the BigInt doesn't fit in a u64.

### 15. `net_poll` silently drops unparseable events (FFI)

**File:** `src/ffi/mod.rs:597-607`

Events that fail `parse()` are silently filtered out via `filter_map`. The caller receives fewer events than expected with no indication. The `count` field reflects parsed count, so pagination can skip events permanently.

**Fix:** Include an `errors` count in the response, or return unparseable events as raw strings.

### 16. `shutdown` doesn't drain NATS client (JetStream)

**File:** `src/adapter/jetstream.rs:260-272`

`shutdown()` sets `initialized = false` and clears the stream cache but never calls `client.drain()` or `client.close()`. The NATS client behind an `Arc` keeps TCP connections open indefinitely.

**Fix:** Call `self.client.take()` and drain/close the client in `shutdown()`.

---

## Low

### 17. `Connection` errors not marked retryable

**File:** `src/error.rs:53`

`is_retryable()` returns `false` for `AdapterError::Connection`. Connection errors are inherently transient and should generally be retryable.

**Fix:** Add `Self::Connection(_)` to the `is_retryable()` match arm.

### 18. Drain worker busy-spins with 100μs sleep

**File:** `src/bus.rs:723-726`

When idle, drain workers sleep for only 100μs before polling again — effectively a busy-wait. With 16 cores (16 shards), that's 16 threads busy-polling at 10,000 iterations/second each.

**Fix:** Use `tokio::sync::Notify` to wake drain workers when events are available, or use adaptive backoff.

### 19. `JoinHandle`s leaked on scale-down

**File:** `src/bus.rs:316-333`

`remove_shard_internal()` removes the batch sender but never removes the corresponding `JoinHandle`s from `self.batch_workers`. Stale handles accumulate over the lifetime of the EventBus.

**Fix:** Track worker handles per shard in a `HashMap<u16, Vec<JoinHandle<()>>>`.

### 20. Default scaling policy prevents scale-up

**File:** `src/config.rs:573-586`

`default_for_cpus(cpus)` sets `max_shards = cpus`. When used via the builder's default path, the scaling policy's `max_shards` equals `num_shards`, so dynamic scaling is enabled but can never actually scale up.

**Fix:** Set `max_shards` to `cpus * 2` or document that the default only allows scale-down.

### 21. Unnecessary `unsafe impl Send/Sync` for `TimestampGenerator`

**File:** `src/timestamp.rs:114-115`

`quanta::Clock` and `AtomicU64` already implement `Send + Sync`. The manual `unsafe impl` suppresses auto-trait checking — if fields change to include a non-Send/Sync type, the compiler won't catch the violation.

**Fix:** Remove both `unsafe impl` lines.

### 22. `EntityLog::prune_through` doesn't update `base_link`/`head_payload`

**File:** `src/adapter/net/state/log.rs:155-160`

After pruning all events, `base_link` and `head_payload` become inconsistent, causing subsequent appends to fail chain validation.

**Fix:** Update `base_link` to the last pruned entry's link and clear `head_payload` when all entries are removed.

### 23. First event skips parent-hash validation in `EntityLog`

**File:** `src/adapter/net/state/log.rs:85-86`

The `sequence == 1` special case accepts any `parent_hash`, allowing an invalid chain linkage for the first event.

**Fix:** Validate `parent_hash` is zero/sentinel for the first event.

### 24. `read_causal_events` silently drops malformed events

**File:** `src/adapter/net/state/causal.rs:248-281`

Returns partial results on parse failure with no error indication. Callers cannot distinguish "10 events received" from "7 parsed, 3 dropped."

**Fix:** Return `Result` with the count of dropped events, or include an error list alongside partial results.

### 25. `scale_to` doesn't track newly placed nodes in loop

**File:** `src/adapter/net/compute/fork_group.rs:185-220`

`used_nodes` is not updated inside the loop, so new forks in the same scale-up operation can be placed on the same node.

**Fix:** Add newly placed nodes to `used_nodes` after each `place_with_spread` call.

### 26. Unchecked array index in `on_node_failure`

**File:** `src/adapter/net/compute/fork_group.rs:267`

`self.forks[index as usize]` can panic if the index is out of bounds after scale-down.

**Fix:** Use `.get()` and handle the out-of-bounds case gracefully.

### 27. `CausalLink::next()` panics on sequence overflow

**File:** `src/adapter/net/state/causal.rs:60`

Calls `.expect()` on `checked_add(1)` which panics at `u64::MAX`.

**Fix:** Return a `Result` or `Option` instead of panicking.

### 28. Python `__repr__` panics on short public key

**File:** `bindings/python/src/lib.rs:175`

`&self.public_key[..8]` panics if the key string is shorter than 8 bytes.

**Fix:** Use `self.public_key.get(..8).unwrap_or(&self.public_key)`.

### 29. `member_count()`/`health()` silently truncate with `as u8`

**File:** `src/adapter/net/compute/group_coord.rs:219-225`

Wraps on >255 members, producing incorrect member counts and health indicators.

**Fix:** Use `u16` or `u32`, or cap at `u8::MAX` with saturation.

### 30. Stats truncation casts `u64 as i64` (Node.js)

**File:** `bindings/node/src/lib.rs:480-484`

`events_ingested` and `events_dropped` are `u64` atomics cast to `i64`. After ~9.2 quintillion events, values wrap to negative in JS.

**Fix:** Use napi `BigInt` type for these fields, or cap at `i64::MAX`.

### 31. `RawEvent::from_str` does not validate JSON

**File:** `src/event.rs:156-158`

Accepts any string without JSON validation, unlike `from_bytes_validated`. Invalid JSON enters the system and fails later during deserialization.

**Fix:** Add a doc comment warning no validation is performed, or validate and return `Result`.

### 32. `max_depth_within` returns 0 even when depth 0 exceeds budget

**File:** `src/adapter/net/continuity/propagation.rs:111-124`

With a zero or very small budget, the function claims depth 0 is reachable when it is not.

**Fix:** Check whether the cost of depth 0 itself exceeds the budget before returning.

### 33. Dead `write_buffer` field in Redis adapter

**File:** `src/adapter/redis.rs:42-43`

`write_buffer` is declared but never used — each `on_batch` creates a fresh `Vec` instead.

**Fix:** Remove the dead field or use it to avoid per-batch allocation.

### 34. JetStream `get_or_create_stream` TOCTOU race

**File:** `src/adapter/jetstream.rs:109-164`

Two concurrent calls for the same `shard_id` can both miss the cache check, both create the stream, and the second insert overwrites the first. Benign but causes unnecessary `create_stream` calls.

**Fix:** Accept the benign race, or use `tokio::sync::OnceCell` per shard.

### 35. `net_free_poll_result` doesn't free the struct itself (FFI)

**File:** `src/ffi/mod.rs:1050-1066`

Frees the internal events array and `next_id` but not the `NetPollResult` struct. API docs say "Free with `net_free_poll_result`" which is misleading.

**Fix:** Clarify documentation, or box-allocate the struct and free it fully.

### 36. `net_ingest_batch` wraps count at `c_int::MAX` (FFI)

**File:** `src/ffi/mod.rs:538`

`c_int::try_from(count).unwrap_or(c_int::MAX)` makes it impossible to distinguish an exact count of `c_int::MAX` from overflow.

**Fix:** Acceptable given extreme unlikelihood; document the behavior.

### 37. `LossSimulator` RNG is non-atomic under concurrency

**File:** `src/adapter/net/failure.rs:448-452`

LCG load-compute-store is non-atomic. Two threads can get identical "random" numbers, losing entropy and causing correlated drops.

**Fix:** Use `fetch_update` with CAS loop, or accept for a test-only utility.

### 38. SPSC ring buffer assertion fires under `Mutex`-protected multi-thread ingestion

**File:** `src/shard/mod.rs:324-375`

`ShardManager::ingest` acquires a `Mutex<Shard>` then calls `try_push_raw` on the SPSC ring buffer. The Mutex ensures correctness, but debug assertions tracking producer thread identity will panic.

**Fix:** Disable SPSC thread-identity assertions for externally-synchronized instances.

### 39. Duplicated `current_timestamp()` function

**File:** `src/adapter/net/state/causal.rs:335`, `snapshot.rs:187`, `observation.rs:183`, `migration.rs:257`

Four identical copies. Maintenance risk — changes to one won't propagate.

**Fix:** Extract to a shared utility.
