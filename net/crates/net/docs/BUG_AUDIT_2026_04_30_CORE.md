# Bug Audit (Core Modules) — 2026-04-30

Follow-up audit focused on the core bus/event/adapter/consumer/FFI/shard surfaces of `net/crates/net/src/`. Continues the numbering of [BUG_AUDIT_2026_04_30.md](./BUG_AUDIT_2026_04_30.md) starting at #55.

Original scope (passes 1–N): `bus.rs`, `event.rs`, `config.rs`, `timestamp.rs`, `error.rs`, `lib.rs`, `adapter/{mod.rs, jetstream.rs, redis.rs, noop.rs}`, `consumer/{filter.rs, merge.rs, mod.rs}`, `ffi/{mod.rs, cortex.rs, mesh.rs}`, `shard/{mod.rs, mapper.rs, batch.rs, ring_buffer.rs}`.

Extended scope (#80 onward): a follow-up multi-agent sweep added point-checks across the previously-deferred `adapter/net/` UDP transport stack and the `sdk/` surface. The new findings are spot-checks, not a systematic re-audit of those subtrees — additional defects may remain.
- `sdk/src/{net.rs, ...}`
- `adapter/net/redex/{disk.rs, file.rs}`
- `adapter/net/{mesh.rs, session.rs, router.rs, linux.rs}`
- `adapter/net/subnet/gateway.rs`
- `adapter/net/contested/correlation.rs`

## Status (running tally)

**Outstanding:** #55, #56, #57, #58, #59, #60, #61, #62, #63, #64, #65, #66, #67, #68, #69, #70, #71, #72, #73, #74, #75, #76, #77, #78, #79, #80, #81, #82, #83, #84, #85, #86, #87, #88, #89, #90, #91, #92, #93, #94, #95, #96

## High

### 55. JetStream `direct_get` walks deleted sequence range one RTT at a time
**File:** `adapter/jetstream.rs:428-433`

After a long retention rollover (e.g. MAXLEN trimmed the first 10M sequences), `poll_shard(from_id=None)` resumes at `start_seq=1`. `direct_get(seq)` returns `NotFound` for every deleted seq; the loop simply increments by one and tries again. Result: 10M sequential network RTTs before a single event is returned. The consumer hangs for minutes — until the request timeout fires, at which point the next poll resumes from where it left off, never making progress. Should query `info().state.first_sequence` and bump `current_seq` to that on the first `NotFound`, or use `direct_get_next_for_subject` / a bounded fetch.

### 56. JetStream cross-process retry duplicates due to per-process nonce (inverse of #9)
**File:** `adapter/jetstream.rs:285-320` (with `process_nonce` at `event.rs:304`)

Issue #9's fix prepends a per-process nonce to `Nats-Msg-Id` so legitimate same-batch retransmits dedup correctly within one process lifetime. The trade-off: a producer that crashes mid-batch (after the server already accepted some events) and restarts gets a fresh `process_nonce` on retry, so JetStream sees the post-crash msg-ids as new and persists the partial batch *plus* the full retry — duplicates in the stream under the same `(shard_id, sequence_start)` tuple. The mid-batch failure comment claiming "the dedup window discards the prior copies" only holds within one process. Same hazard for `try_join_all(acks)` short-circuiting on first error: the dropped ack futures may still complete server-side, leaving partial state that survives the retry. Mitigation requires either persisting `process_nonce` across restarts or moving to a server-side checkpoint scheme.

### 57. Redis `MULTI`/`EXEC` timeout cancellation produces duplicate XADDs
**File:** `adapter/redis.rs:298-319`

`tokio::time::timeout` cancels the future locally but does not roll back bytes already on the wire. `EXEC` may run server-side after the future is dropped; the caller-driven retry then runs *another* `EXEC`, producing duplicate XADDs (each with a distinct server-generated `*` stream id, so downstream consumers cannot dedupe on `r.id` since that is application-defined and not necessarily unique). Self-acknowledged in the inline comment, unmitigated. Either drop the timeout (let the connection-manager handle it) or implement an idempotency token consulted by the retry path.

### 58. `net_free_bytes` panics across the FFI boundary on adversarial `len`
**File:** `ffi/mesh.rs:1721`

`Layout::array::<u8>(len).expect("byte layout")` panics when `len > isize::MAX` — `Layout::array` rejects any size that would overflow `isize`. `net_free_bytes` is `extern "C"` with no `catch_unwind`, so a C/Go-cgo/NAPI/PyO3 caller that passes a corrupted `len` (or a `len` it derived from outside-controlled storage) gets a Rust panic unwinding through the FFI boundary — UB. The `len == 0` early-return doesn't screen large values. Either return `NetError::InvalidArgument` for `len > isize::MAX`, or wrap the body in `catch_unwind` and convert to an error code.

### 59. Bus shutdown timeout strands ingests, contradicting the documented "no stranding" contract
**File:** `bus.rs:725-743` (contract docs at `bus.rs:446-450`)

The in-flight wait deadline (5s, real-time `std::time::Instant`) breaks out with a warning and unconditionally stores `drain_finalize_ready=true`. A slow producer that has already incremented `in_flight_ingests` (and therefore observed `shutdown=false` immediately before pushing) will still complete its push *after* the drain worker has run its final sweep. The event lands in the ring buffer but is never read — directly contradicting the SeqCst handshake comment promising "every observed in-flight ingest completes before the final sweep." Either widen the deadline, abort stalled producer tasks before flipping the gate, or re-document this as a known data-loss path.

### 75. `add_shard_internal` leaks workers and routing state if `activate_shard` fails
**File:** `bus.rs:307-355`

The two-phase shard add introduced by #46's fix (`provision → spawn workers + register sender → activate`) has no rollback if step 3 errors. On `Err` from `activate_shard` (line 343-345) the function returns leaving:
- the new sender still in `batch_senders` (inserted at line 327),
- the batch + drain `JoinHandle`s still in `batch_workers` (inserted at line 337-339),
- the `Provisioning` `MappedShard` still in the mapper.

The drain worker for the orphaned id then loops indefinitely on an empty ring buffer — its `with_shard` call still finds the entry (it's mapped, just `Provisioning`), and `select_shard` skips Provisioning so producer pushes never reach the buffer — burning a 100µs sleep per cycle until process shutdown. The mapper's `next_shard_id` stays advanced, so a subsequent retry allocates a higher id while the dead one squats. The compounding hazard is repeated scale-ups: each failed `activate_shard` adds another zombie drain worker and another orphan provisioning entry. Mirror `remove_shard_internal`'s teardown: on `activate_shard` Err, drop the sender, abort both join handles, and call `remove_shard` to unmap the provisioning entry before returning.

### 80. `Net::shutdown` discards the `Arc<EventBus>` and skips `bus.shutdown()` when any clone is outstanding (verified)
**File:** `sdk/src/net.rs:236-246`

```rust
pub async fn shutdown(self) -> Result<()> {
    match Arc::try_unwrap(self.bus) {
        Ok(bus) => bus.shutdown().await?,
        Err(_) => Err(SdkError::Adapter("cannot shutdown: outstanding references exist".into())),
    }
}
```

The user does receive an `Err` (not a silent return), but the `Err(_)` arm **drops the `Arc` returned by `try_unwrap`** rather than retrying or signalling shutdown via the inner state — the only effect on the bus is decrementing the strong count by one. `bus.shutdown()` is never invoked, so the drain barrier doesn't run, the adapter's `flush()` / `shutdown()` never execute, background tasks keep running, and any pending events in ring buffers ride on the bus's normal `Drop` semantics whenever the last `Arc` clone happens to be released. There is no SDK escape hatch — no `shutdown_async`-after-flush, no synchronous drain primitive — so a caller that ever subscribed (which always perpetuates an Arc clone via `EventStream`, see #81) is stuck.

**Verification (2026-04-30):** read `sdk/src/net.rs:191-193` and `:236-246`. Confirmed `EventStream::new(self.bus.clone(), opts)` clones the Arc on every subscribe, and the `Err(_)` arm above takes ownership of the Arc back from `try_unwrap` only to drop it — no inner-flag signalling exists.

The mesh-FFI side has an explicit regression test (`net_mesh_shutdown_runs_even_with_outstanding_arc_refs`) for this exact pattern; the SDK still has the legacy gating. Mirror the FFI fix: signal shutdown via a flag on the inner `Arc` rather than gating on `try_unwrap`, and let outstanding-handle paths consume the signal as they finish their work.

### 81. `Net::subscribe` perpetuates `Arc<EventBus>` clones, making #80 the default outcome
**File:** `sdk/src/net.rs:191-193`

`EventStream::new(self.bus.clone(), opts)` increments the strong count of `Arc<EventBus>`. Even after the user drops the `EventStream`, any in-flight poll future the stream spawned can still hold the clone, so `Net::shutdown`'s `Arc::try_unwrap` fails (#80). The SDK provides no escape hatch — there is no `shutdown_async`-after-flush, no synchronous drain primitive — for the documented "subscribe → done streaming → shutdown" pattern. Fix is paired with #80: once shutdown is signal-based, surviving clones become benign.

### 84. `RxCreditState::on_bytes_consumed` issues a matching grant for every byte consumed, defeating receive-side backpressure
**File:** `adapter/net/session.rs:781-788`

```rust
let new_consumed = self.consumed.fetch_add(bytes, Ordering::AcqRel) + bytes;
self.granted.fetch_add(bytes, Ordering::AcqRel);
```

Each consumed byte is paired with an immediate matching grant of the same size, so `granted - consumed` never falls below the initial window regardless of how slowly the application drains the inbound queue — the receiver never applies backpressure on a stream. The "round-trip credit window" docstring promises threshold-based grants (issue a window when outstanding ≤ `window_bytes / 2`); no such threshold is implemented, every byte auto-grants. The two `fetch_add`s are also not atomic together, so a concurrent reader observes momentary `consumed > granted` and `outstanding()` saturates to 0. The sender's credit therefore drops only when the receiver stops calling `on_bytes_consumed`, not when the receiver buffer is bounded. Replace the immediate auto-grant with a threshold check, or make the two updates an atomic CAS pair.

### 85. Mesh dispatch path skips AEAD verification on heartbeat packets
**File:** `adapter/net/mesh.rs:2367-2371` (compare with `adapter/net/mod.rs:642-663` and `adapter/net/pool.rs:237-249`)

```rust
if parsed.header.flags.is_heartbeat() {
    failure_detector.heartbeat(peer_node_id, source);
    session.touch();
    return;
}
```

The mesh dispatch loop fast-paths `is_heartbeat()` packets — touching the failure detector and session timestamp — without invoking AEAD verification. The legacy single-peer adapter (`mod.rs:642-663`) explicitly verifies the tag for the same packet shape, and the comment on `pool.rs:237-249` claims heartbeats are now AEAD-authenticated specifically to prevent off-path spoofing. An off-path attacker with the cleartext `session_id` (visible on every prior data packet) and the source UDP address can spoof heartbeat-flagged 64-byte headers from `peer_addr`, indefinitely defeating session-idle timeout and triggering false `failure_detector.heartbeat(...)` notifications. Either route heartbeats through the same AEAD-verify path as data packets, or document this as a known design limitation. (Worth verifying whether mesh has its own gate elsewhere before AAD-skip is concluded.)

### 92. Redex `compact_to` keeps in-memory index offsets absolute while on-disk offsets become segment-relative — appends after retention sweep silently lost on restart (verified)
**File:** `adapter/net/redex/disk.rs:917-1010` (offset rewrite at `:977-979`); caller `adapter/net/redex/file.rs:919-1003` (sweep) and `:368-414` (append); recovery walk `adapter/net/redex/disk.rs:245-269`

`compact_to` rewrites surviving on-disk idx records with offsets *relative* to the new dat (`e.payload_offset = (entry.payload_offset as u64).saturating_sub(dat_base) as u32` at `disk.rs:977-979`). The local `e` is a copy; the corresponding in-memory `state.index` entries are NOT renormalized — their `payload_offset` fields remain absolute in the segment's pre-compaction logical space. The next append computes `offset = state.segment.base_offset() + current_live` (`file.rs:384-388`), which is also absolute (because `evict_prefix_to(new_base)` advanced `base_offset` to the surviving entry's old absolute position), and writes that value verbatim to disk via `entry.to_bytes()` in `append_entry_inner` (`disk.rs:639`). The on-disk idx ends up with mixed semantics: pre-compaction records have small relative offsets, post-compaction records have large absolute offsets that index past the end of the new dat. On reopen, the torn-tail recovery walk (`disk.rs:245-269`) detects every post-compaction record's `(offset+len) > dat_len` and truncates the tail.

**Verification (2026-04-30):** read disk.rs and file.rs end-to-end and traced concretely:

- Setup: `retention_max_events=2`, append 5×100-byte heap entries (seq 0–4). Pre-sweep state: `segment.base_offset=0`, `live_bytes=500`, in-mem index offsets `[0,100,200,300,400]`, on-disk idx mirrors.
- `sweep_retention` (`file.rs:919-1003`): `state.index.drain(..3)` leaves `[(seq=3,off=300),(seq=4,off=400)]` with **absolute** offsets retained; `state.segment.evict_prefix_to(300)` advances `base_offset` to 300; `dat_base = state.segment.base_offset() = 300` (`file.rs:989`); `compact_to(clone, ts, 300)` writes new on-disk idx with offsets `[0, 100]` (relative) and 200-byte new dat — but `state.index` is unchanged.
- Next append (seq=5, 100 bytes): `offset = 300 + 200 = 500` (`file.rs:387`); `disk.append_entry_at` writes `(seq=5, off=500, len=100)` verbatim. On-disk dat now 300 bytes; idx now `[(off=0,len=100),(off=100,len=100),(off=500,len=100)]`.
- Reopen (`disk.rs:245-269`): walking backward, seq=5 has `end=500+100=600 > dat_len=300` → torn → `truncate_at = 2`. seq=4 has `end=200 ≤ 300` → break. `index.truncate(2)` — seq=5 silently dropped.

The existing regression test `sweep_retention_persists_eviction_to_disk` (`file.rs:2373`) appends 5 → sweeps → closes → reopens, but does **not** append between sweep and close, so it does not exercise this path.

Fix options:
1. Renormalize `state.index` entries during `sweep_retention` (subtract the same `dat_base` from each surviving entry's `payload_offset` before releasing the lock), so subsequent appends land on a 0-based segment.
2. Or change `compact_to` to leave on-disk offsets **absolute** as well (skip the `saturating_sub(dat_base)`) and store `dat_base` in a per-segment header that the recovery walk consults — this avoids touching in-memory state but requires a header format change.

Option 1 is the smaller delta. Option 2 keeps the format consistent with the in-memory representation and avoids any future drift.

**Decision:** go with option 1 — renormalize `state.index` offsets by `dat_base` inside `sweep_retention`, and ensure `segment.base_offset` is reset consistently so that subsequent appends compute offsets against a 0-based segment. Then add a regression test that:

- appends → `sweep_retention` → appends again → `close` → reopen,
- and asserts the post-sweep append survives restart (e.g. seq numbers `[3, 4, 5, 6]` — surviving pair plus two post-sweep appends — are all present after reopen).

### 93. Redex `compact_to` non-atomic three-rename sequence with no parent-dir fsync
**File:** `adapter/net/redex/disk.rs:1086-1089`

The atomic-rewrite pattern uses three sequential `std::fs::rename` calls (idx → dat → ts) without bracketing them in a single dirent flip and without fsyncing the parent directory afterward. A power loss between the first and second rename leaves the new (renormalized) idx alongside the old dat + old ts; on reopen, recovery's checksum verification fails for every entry because the new idx's offsets index into the wrong dat bytes, and all entries are dropped. On POSIX, even a successful series of renames is not durable until the directory inode is fsynced. Combined with #92, a crash during retention sweep can corrupt the entire segment with no recovery path. Either move to a single rename of a packed manifest, or fence the three renames inside an explicit dir-fsync.

### 94. Redex `metadata()?` early-return mid-batch leaves orphaned idx/dat bytes without rollback
**File:** `adapter/net/redex/disk.rs:638, 663, 802, 820`

In both `append_entry_inner` and `append_entries_inner`, the `pre_*_len = file.metadata()?` lines use `?` to early-return. By that point, dat (and possibly idx) has already been written successfully. Only the rollback inside the explicit `if let Err(e) = file.write_all(...)` block performs file rollback — a `metadata()` error skips it entirely. The on-disk state ends up with orphaned idx/dat bytes; the caller is told the append failed and rolls back `next_seq` in memory. On restart, `read_timestamps` returns None for the length mismatch (idx longer than ts), so all recovered entries get `now()` as their timestamp and age-based retention silently breaks for the affected window. Replace each `?` with explicit error handling that triggers the existing rollback path before returning.

### 95. Redex `sweep_retention` commits in-memory eviction even when `compact_to` fails
**File:** `adapter/net/redex/file.rs:919-1003` (failure point ~line 991)

`sweep_retention` mutates in-memory state (drains `index`, `timestamps`, calls `evict_prefix_to`) before invoking `disk.compact_to`. If `compact_to` fails, the function logs a warning and returns `Ok(())` — but in-memory state has already evicted the head, while on-disk dat/idx/ts still contain everything. On the next reopen, recovery replays the full on-disk state, resurrecting the entries that were just evicted in memory. The inline comment on ~line 992 acknowledges the divergence but treats it as benign; combined with #92 it becomes a corruption vector (post-failure appends now interleave with resurrected entries on disk). Either roll back in-memory eviction on `compact_to` failure, or perform the disk compaction first and only mutate in-memory state on success.

## Medium

### 60. JetStream `poll_shard` `info()` race truncates concurrent writes
**File:** `adapter/jetstream.rs:392-398`

`max_seq = stream.info().await.last_sequence` is sampled once before the read loop. If a producer writes new messages while the loop is running, the `current_seq > max_seq` short-circuit fires early and `has_more=false` is returned even though the stream tail has more events. Concretely: limit=100, stream had 50 messages at info-time, producer writes 200 more during the read; consumer returns 50 with `has_more=false`, sleeps thinking the stream is drained, and only catches the new tail on the next poll cycle. Worse on a tailing/realtime consumer with a small fetch limit. Either re-read `info()` before declaring drain, or treat `max_seq` as a lower bound and let `direct_get` itself signal end-of-stream.

### 61. `runtime()` lazy initializer panics across the FFI boundary on builder failure
**File:** `ffi/cortex.rs:75`, `ffi/mesh.rs:154`

`tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("...")` panics when worker-thread spawning fails — `pthread_create` returning `EAGAIN` from `RLIMIT_NPROC`, container thread limits, or memory pressure. Every CortEX/Mesh FFI entry point lazily triggers this on first use. A daemon under thread-limit pressure that calls e.g. `net_redex_open_file` then sees the panic unwind into its C/Go/NAPI/PyO3 binding — UB. Replace `expect` with a recoverable error path that returns a `NetError` and leaves the cell unset for retry on the next call.

### 62. `net_init` early-return paths drop `Runtime` from a tokio worker thread
**File:** `ffi/mod.rs:286, 290, 463`

`Runtime::new()` succeeds, then `CStr::to_str` returns `Err` (line 286) or `parse_config_json` returns `None` (line 290) or `EventBus::new` errors (line 463) — and the local `runtime` drops on function return. Dropping a multi-thread `Runtime` from inside another tokio runtime's worker thread panics with "Cannot drop a runtime in a context where blocking is not allowed", crossing the `extern "C"` boundary = UB. A Python/Go/Node caller that already runs on tokio (e.g. via PyO3 `pyo3-asyncio`, NAPI workers, or any embedded server) reaches this from a worker thread on malformed config. Either build the runtime *after* validating inputs, or move runtime construction to a `OnceLock` so successful prior init survives parse errors.

### 63. `ScalingPolicy::validate()` accepts NaN thresholds and silently disables auto-scaling
**File:** `config.rs:731-740`

`fill_ratio_threshold` and `underutilized_threshold` are `f64`; `<=` and `>` against NaN both return `false`, so `f64::NaN` passes both validation arms. At runtime, `mapper.rs:560` does `m.fill_ratio > self.policy.fill_ratio_threshold`; the comparison is always `false` for NaN, so the scaler never scales up regardless of fill ratio (mirror hazard for scale-down). User configs deserialized from `0.0/0.0`-style arithmetic or fed through environment templating end up "valid" but inert. Add `is_finite()` checks in `validate()`.

### 64. `scale_up_provisioning` + `activate` race over-allocates past `max_shards`
**File:** `shard/mapper.rs:757-786` (budget gate at `597-612` and `629-664`)

The budget gate compares `active_count + count <= max_shards`, ignoring already-pending Provisioning shards. A caller that batches several `add_shard` calls before activating any of them slips multiple `scale_up_provisioning` calls past the gate. `activate()` then unconditionally `fetch_add(1, Release)` for each. Worked example: `max_shards=4`, `active_count=3`. `scale_up_provisioning(1)` passes (3+1≤4), `scale_up_provisioning(1)` passes again (still sees `active_count=3`), both `activate()` increments push `active_count` to 5. Subsequent `evaluate_scaling` budget arithmetic does `self.policy.max_shards - active_count` (`mapper.rs:578`), which underflows u16 — debug-build panic, release-build wraps to ~65530. Either gate `activate()` on `active_count < policy.max_shards`, or count Provisioning shards toward the budget.

### 65. `start_scaling_monitor` leaks the prior monitor task on a second call
**File:** `bus.rs:286`

`*self.scaling_monitor.lock() = Some(handle);` overwrites without aborting or awaiting the previous `JoinHandle`. The displaced task continues running detached, holding a `Weak<EventBus>`, only exiting when it next observes `shutdown` or fails to upgrade. Two concurrent monitors briefly run in parallel, doubling metrics-tick wakeups and (more importantly) competing for `evaluate_scaling`'s lock — adding contention without callers expecting it. Make the function idempotent: if the slot is `Some`, log and return without spawning.

### 66. `CompositeCursor::update_from_events` regresses cursor on unsorted input
**File:** `consumer/merge.rs:93-98`

`update_from_events` loops events and unconditionally inserts each into the per-shard position map; whichever event for a given `shard_id` appears *last* in the slice wins, regardless of whether its `id` is actually further along the stream than what is already stored. A caller who passes events sorted by `insertion_ts` (not `id`), or merged from multiple buffers in arbitrary order, can move the cursor *behind* a previously returned id, causing those events to be re-delivered on the next poll. The test at `consumer/merge.rs:511-520` actively pins the broken behavior — passes events `[100-0, 200-0, 150-0]` for shard 0 and asserts the final position is `"150-0"` (i.e. regressed from `"200-0"`). Either compare-and-set per shard (only update if new id is greater under the adapter's id ordering), or restrict the API contract to ascending-id-sorted input and assert it.

### 67. `alloc_bytes` `Layout::array` "cannot overflow" comment is wrong
**File:** `ffi/mesh.rs:1699`

`Layout::array::<u8>(len).expect("byte layout")` matches the same panic shape as #58 — `Layout::array` rejects sizes >`isize::MAX`. The inline comment claiming this "cannot overflow for any valid usize" is incorrect; the boundary is `isize::MAX`, not `usize::MAX`. Currently bounded by what `to_bytes()` produces on token-sized payloads, so unreachable today, but the load-bearing comment will mislead future maintainers reusing the helper. Same fix as #58.

### 76. `flush()` phase-2 early-break check is redundant with phase 1 — barrier collapses to one `max_delay`
**File:** `bus.rs:663-668`; helper `shard/mod.rs:670-673`

The phase-2 loop is meant to give "at least `max_delay × n_workers`" for in-flight batches sitting in the per-shard mpsc channels and the batch worker's pending-batch buffer to time out and dispatch (comment at lines 636-647 — explicitly added because #16's old single-window wait was too short). The early-break inside the loop calls `all_shards_empty()`, but that probes ring-buffer fill (`table.shards.iter().all(|s| s.lock().is_empty())`), which phase 1 already drained. With no concurrent ingest the predicate is constant-true after the first sleep window, so the early-break fires after exactly one `max_delay` regardless of `n_workers` — and the documented multi-worker budget is never observed. Phase 2 collapses back to the single-window behavior that #16 was supposed to replace; a flush-as-barrier caller on a many-shard config (default 8+) returning during a partial-batch dispatch sees the same pre-#16 silent loss. Either probe per-shard mpsc-channel depth directly (e.g. via a `pending_in_channel` counter incremented on `tx.send` and decremented on `rx.recv` in the batch worker), gate the break on a "no batches dispatched in last window" signal, or remove the early-break and pay the full budget.

### 82. `manual_scale_down` strands events on drained shards
**File:** `bus.rs:815-824` (compare scaling-monitor finalize at `bus.rs:935-952`)

`manual_scale_down` calls `mapper.scale_down(count)` to mark shards `Draining` and returns the drained ids. Unlike the scaling-monitor path, it never calls `mapper.finalize_draining()` or `bus.remove_shard_internal(...)`. Events still queued in those shards' ring buffers (and any pushes that arrived between the read-locked early budget check and the write-locked state transition) sit unread until the scaling monitor catches up — which only fires if a monitor is running and reaches its next tick before `bus.shutdown()`. Bus configs without an active monitor lose those events on shutdown. Either drive the finalize loop synchronously inside `manual_scale_down`, or document the API as "requires `start_scaling_monitor` before use" and assert it at call time.

### 83. `ShardManager::remove_shard` never drops the entry from `ShardMapper.shards` — unbounded growth across scale-up/down cycles
**File:** `shard/mod.rs:819-872`

`remove_shard` rebuilds the routing table (removing from `ShardTable`), drains the ring buffer, and decrements `num_shards`, but never asks the mapper to drop the corresponding `MappedShard` record. The mapper's `shards: RwLock<Vec<MappedShard>>` keeps growing — every scale-up appends an entry, and `remove_stopped_shards` is the only API that deletes them, but no production caller invokes it (only tests reference it). Long-running services with frequent scaling activity accumulate `Stopped` entries indefinitely; `evaluate_scaling` filters by state but still iterates the full list, so per-tick cost grows with cumulative scaling history. Wire `remove_shard` to call `mapper.remove_stopped_shards()` after the manager-level removal completes, or remove the specific id directly.

### 86. Direct handshake `recv_from` on `Arc<UdpSocket>` races the dispatch receive loop
**File:** `adapter/net/mesh.rs:6093-6118, 6155-6176` (dispatch loop spawn at `~2032`)

`try_handshake_initiator` and `try_handshake_responder` poll `socket_arc.recv_from` directly. Once `start()` has spawned `spawn_receive_loop`, both consumers race for each datagram on the same `Arc<UdpSocket>`; tokio dispatches a UDP datagram to exactly one waiter. The handshake response can be swallowed by the dispatch loop, which then drops it because no matching session exists yet (~lines 2349-2364), and the handshake times out. Concurrent direct connects on the same node also steal each other's responses. Either bridge handshakes through an in-memory channel populated by the dispatch loop (forward msg2/msg3 datagrams to a per-pending-handshake oneshot keyed by `session_id`), or synchronize handshake initiation to suspend dispatch dequeue for the matching session.

### 87. Mesh post-handshake `tokio::spawn` is fire-and-forget and can wedge peer/session/route on cancellation
**File:** `adapter/net/mesh.rs:2553-2569` (state insert at `2524-2534`)

After completing the responder handshake, the code inserts session, peer entry, routing entry, and `peer_addrs`, then `tokio::spawn`s a fire-and-forget `socket.send_to(&payload, next_hop).await` whose only rollback fires from inside the spawned future on socket-send error. If the runtime is shutting down or the spawned task is cancelled before the send completes, rollback never runs and the peer/session/route survive in an unsendable state — the responder holds session keys the initiator never received the matching msg2 for. There is no JoinHandle tracking. Either await the send synchronously on the handshake task, or track the JoinHandle alongside the rollback closure so cancellation triggers cleanup.

### 88. Subnet gateway interprets `hop_ttl == 0` as unlimited rather than expired
**File:** `adapter/net/subnet/gateway.rs:112` (AAD definition at `adapter/net/protocol.rs:319`)

```rust
if hop_ttl > 0 && hop_count >= hop_ttl {
    // drop
}
```

`NetHeader::new` defaults `hop_ttl` to 0, so any packet with default headers forwards regardless of `hop_count`. `hop_count` is excluded from AAD (per `protocol.rs:319`) and is mutable in transit, so an attacker or buggy peer can craft `hop_ttl=0` packets that loop through gateways with no Net-layer bound. Routing-layer TTL still bounds end-to-end loops for routed packets, but pure subnet-gateway forwarding paths (no routing header) lack any cap. Either treat `hop_ttl == 0` as expired (drop), set a sensible non-zero default in `NetHeader::new`, or pull `hop_ttl` into AAD so it cannot be downgraded mid-flight.

### 89. Router `stream_stats` keyed by AEAD-unverified bytes — DashMap flood
**File:** `adapter/net/router.rs:475-481`

```rust
let stream_id = if data.len() >= ROUTING_HEADER_SIZE + HEADER_SIZE {
    let net_header = &data[ROUTING_HEADER_SIZE..ROUTING_HEADER_SIZE + HEADER_SIZE];
    u64::from_le_bytes(net_header[32..40].try_into().unwrap_or([0; 8]))
} else {
    0
};
```

The router extracts the inner `stream_id` from raw packet bytes before AEAD verification and inserts an entry into the `stream_stats` DashMap keyed by that value. A malicious peer can spam routed packets with random `stream_id` values to exhaust router memory; `cleanup_idle_streams` runs on an interval, so growth is bounded only by the cleanup period. Either gate `stream_stats` insertion on a successful AEAD verify, cap the DashMap with an LRU/size-bounded policy, or restrict accounting to known/registered `stream_id`s.

### 90. `BatchedTransport::recv_batch` indexes empty `recv_buffers` constructed via `new_send_only`
**File:** `adapter/net/linux.rs:215, 285` (constructor at `48-50`)

`BatchedTransport::new_send_only` intentionally skips the `recv_buffers` allocation. Both `recv_batch` and `recv_batch_blocking` then unconditionally do `self.recv_buffers[i].resize(MAX_PACKET_SIZE, 0)` for `i in 0..count`, which panics with index-out-of-bounds for any send-only-constructed instance. The doc comment on `new_send_only` only states the contract verbally; there is no runtime guard. Either return an `InvalidOperation` error from the recv methods when `recv_buffers` is empty, or split the type so send-only construction returns a different type that lacks the recv methods at compile time.

### 91. `analyze_subnet_correlation` returns a non-deterministic subnet on tied depth
**File:** `adapter/net/contested/correlation.rs:249-257`

```rust
for (&subnet, &count) in &subnet_counts {
    if count >= threshold && subnet.depth() >= best_depth {
        best_subnet = Some(subnet);
        best_depth = subnet.depth();
    }
}
```

`HashMap` iteration is unordered, and the predicate uses `>=` on depth as the tie-breaker, so when two subnets at the same `best_depth` both meet the threshold, the chosen subnet depends on hash iteration order (randomized per process). Downstream `partition.rs::detect` uses the chosen subnet to brand the partition record, and recovery scope flips between runs given identical inputs. Switch to a deterministic tiebreaker (e.g. lowest subnet id at equal depth, or strictly `>` with a documented "first seen wins" semantic that uses an ordered iterator).

### 96. Redex `read_timestamps` accepts ts file with stale per-position semantics after torn-tail recovery
**File:** `adapter/net/redex/disk.rs:298-310` (rewrite branch at `384-399`)

`read_timestamps` accepts a ts file as long as `bytes.len() >= expected_entries * 8` and returns the first `expected_entries` u64s. This assumes the ts file's per-position semantics align with idx's, but if a previous run truncated idx via the torn-tail walk (`open` ~lines 247-266) without rewriting ts, the surviving entries' timestamps are misaligned (the first N timestamps refer to the *first* N entries the ts file ever held, not the surviving ones after truncation). The recovery code only rewrites ts when `bad_entries > 0`; torn-tail truncations skip the rewrite. Age-based retention then operates on wrong timestamps. Either rewrite ts whenever idx is truncated for any reason (capturing the surviving timestamps and rewriting in order), or store timestamps inside idx records.

## Low

### 68. `JetStreamAdapterConfig::max_messages` / `max_bytes` typed `i64`, not validated for negatives
`config.rs:499, 503, 549, 555` (validator at `:575-597`) — accepts `with_max_messages(-1)` etc. NATS rejects negatives at stream-create time, surfacing as a runtime adapter error instead of at startup `validate()` (which is the documented purpose). Switch to `Option<u64>` (or `Option<NonZeroU64>`).

### 69. Bus scaling monitor and drain worker read `shutdown` with `Relaxed` while writers use `SeqCst`
`bus.rs:906, 1137` — currently sound because the Acquire/Release handshake on `drain_finalize_ready` provides the needed happens-before, but the inconsistency is a footgun: any future code change adding a producer-side path that piggybacks on `shutdown`'s ordering would silently break. The drain worker's comment claiming "same rationale as ingest" is misleading — `try_enter_ingest` (`bus.rs:454`) uses SeqCst.

### 70. Bus shutdown awaits drain workers sequentially
`bus.rs:760-763` — `for (...) in workers { drain.await; }` serializes shutdown wall-clock as N×T instead of max(T). Default 1024 shards × per-shard final-drain time becomes painful. Use `futures::future::join_all`.

### 71. `JetStreamAdapter::init` / `RedisAdapter::init` are silently re-entrant
`adapter/jetstream.rs:197-219`, `adapter/redis.rs:233-258` — second `init` overwrites `client`/`conn`, dropping the prior client and any in-flight publishes. The trait says "Called once before any other methods" but doesn't enforce it. An orchestrator that calls `init` defensively after a perceived failure silently loses messages. Either no-op when already initialized or return an error.

### 72. `PollMerger` Step-2 cursor override re-fetches non-matching events
`consumer/merge.rs:430-441` (with new_cursor at `289-291`) — when a shard's filter matches don't get truncated, Step 1 doesn't roll back, but Step 2 unconditionally overrides `final_cursor[shard_id]` with the last *matched* (not last *fetched*) event id. Adapter's `next_id` (which pointed past the last fetched event) is overwritten with an earlier position. Subsequent polls re-fetch and re-evaluate the intervening non-matches against the same filter. Throughput penalty proportional to `over_fetch_factor` on low-match-rate streams. Events are re-evaluated, not lost — efficiency only.

### 73. `Ordering::InsertionTs` lex tiebreaker mis-orders unpadded numeric ids
`consumer/merge.rs:356-361` — sort tiebreaker is `a.id.cmp(&b.id)` (string compare). Backends emitting unpadded numeric ids (`"9-0"` vs `"10-0"`) get inverted ordering. Dormant for fixed-width ids (Redis Streams `ms-seq`, ULIDs, UUIDs, zero-padded sequences); surfaces only for adapters that emit unpadded numerics. Either document the id-format contract or parse-aware compare.

### 74. `net_shutdown` takes raw `&mut` to a field while `&NetHandle` borrow is in scope
`ffi/mod.rs:912, 966, 987-988` — `let bus = ManuallyDrop::take(&mut (*handle).bus);` while `handle_ref: &NetHandle` was acquired at line 912 and last used at line 966. NLL likely ends the immutable borrow before line 987, but the `&mut`-via-raw-pointer adjacent to a live `&` is fragile under stacked/tree borrows. The function's own doc comment hints at the soundness concern. Restructure to drop the `&NetHandle` binding explicitly before taking the field, or move the `ManuallyDrop::take` calls into a block scoped after `handle_ref` is no longer reachable.

### 77. RingBuffer SPSC thread guards are gated on `cfg(test)` despite docs claiming `debug_assertions`
`shard/ring_buffer.rs:89-97, 132-135, 146-163, 198-222, 244-261, 287-303` — the doc and SAFETY comments explicitly advertise *"active under `debug_assertions`, not just `cfg(test)`, so dev runs of the binary catch SPSC violations even outside of unit tests"* (lines 89-92, 198-203). The actual attribute on every `producer_thread`/`consumer_thread` field, initializer, and `assert_eq!` site is `#[cfg(test)]`. The runtime safety net the doc promises is therefore absent in any non-`cargo test` build — including the unoptimized debug binaries that consumers run during development — defeating the explicit goal of catching new SPSC-violating callers (the same threat-model #35 calls out) before release. Either swap every `#[cfg(test)]` site to `#[cfg(debug_assertions)]` (matching the contract) or correct the doc.

### 78. RingBuffer `head`/`tail` `usize` wraparound permanently wedges the buffer on 32-bit targets
`shard/ring_buffer.rs:165-184, 245-279` — `try_push` computes `len = head.wrapping_sub(tail)` and rejects if `len >= capacity - 1` (lines 169-172). Sound on 64-bit (~58 years to wrap at 10 G events/sec). On 32-bit (wasm32 is in the test matrix per `test_parse_poll_request_limit_overflows_usize_on_32bit`), `head` wraps after 2³² pushes — ~7 minutes per shard at 10 M events/sec, ~12 hours at 100 K events/sec. Once `head` laps `tail` and the wrapping distance exceeds `capacity-1`, `try_push` rejects forever and the buffer is permanently wedged; no compaction or counter recovery exists. Either widen the cursors to `u64` on 32-bit or modulo-reduce after each store so the wrap point coincides with capacity.

### 79. FFI returns `BufferTooSmall` for `c_int` overflow when the buffer was actually large enough
`ffi/mod.rs:789-792, 849-852` — after the response JSON is successfully copied into the caller's C buffer, `c_int::try_from(response_json.len())` is converted to indicate the written length. On overflow the current path returns `NetError::BufferTooSmall`, which tells the caller "resize and retry" — but the data was already written and the buffer was big enough; the caller can't make progress by resizing. `NetError::IntOverflow` is defined at line 220 specifically for this case; both call sites should use it. Trivial fix.

---

## Notably clean

`event.rs`, `timestamp.rs`, `error.rs`, `lib.rs`, `consumer/filter.rs`, `shard/batch.rs`. Many would-be bugs in these modules — zero-divisor configs, non-deterministic merge sort tiebreaking (#52), `Filter::And` empty pass-through, sequence-number saturation on `u64::MAX` — already have regression tests pinning the fixes from prior audit passes. (Removed `shard/ring_buffer.rs` from this list — see #77 and #78.)

## Top priorities to fix first

1. **#80** — `Net::shutdown` silently no-ops with outstanding Arc clones (silent data loss on the documented graceful-shutdown path; trivially reproducible via `subscribe`)
2. **#92** — Redex `compact_to` in-memory vs on-disk offset divergence (every event after retention sweep silently lost on restart — directly breaks the "redex-disk" merge's stated goal)
3. **#55** — JetStream `direct_get` retention-rollover stall (consumer DoS after MAXLEN trim)
4. **#57** — Redis MULTI/EXEC timeout duplicates (silent stream corruption)
5. **#58** — `net_free_bytes` panic-across-FFI on adversarial `len`
6. **#84** — `RxCreditState` auto-grants every consumed byte, defeating receive-side backpressure entirely
7. **#59** — Bus shutdown timeout strands events despite the "no stranding" contract
8. **#56** — JetStream cross-process retry duplicates (inverse trade-off of #9's fix)
9. **#93** — Redex `compact_to` non-atomic three-rename + missing dir fsync (compounds #92 into segment corruption on crash)
10. **#85** — Mesh dispatch fast-paths heartbeats without AEAD verify (off-path heartbeat spoofing defeats idle timeout)
11. **#63** — NaN thresholds silently disable auto-scaling
12. **#64** — `scale_up_provisioning` + `activate` over-allocates past `max_shards`
13. **#66** — `update_from_events` cursor regression on unsorted input (re-delivery)
14. **#75** — `add_shard_internal` permanent worker leak on activate failure
15. **#76** — `flush()` phase-2 barrier collapses to one window (re-introduces #16-class loss on many-shard configs)

## Out of scope (deferred)

The `adapter/net/` UDP transport stack — `cortex/`, `swarm/`, `traversal/`, `state/`, `behavior/`, `compute/`, `continuity/` — was not re-audited in this pass. The previous audit ([BUG_AUDIT_2026_04_30.md](./BUG_AUDIT_2026_04_30.md)) covers those subsystems through #54.

The follow-up sweep that produced #80–#96 did spot-check `redex/`, `mesh.rs`, `session.rs`, `router.rs`, `linux.rs`, `subnet/gateway.rs`, `contested/correlation.rs`, and `sdk/src/net.rs`, but those were targeted point-checks, not a systematic re-audit — additional defects in those files may remain.
