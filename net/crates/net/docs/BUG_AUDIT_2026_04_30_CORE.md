# Bug Audit (Core Modules) — 2026-04-30

Follow-up audit focused on the core bus/event/adapter/consumer/FFI/shard surfaces of `net/crates/net/src/`. Continues the numbering of [BUG_AUDIT_2026_04_30.md](./BUG_AUDIT_2026_04_30.md) starting at #55.

Original scope (passes 1–N): `bus.rs`, `event.rs`, `config.rs`, `timestamp.rs`, `error.rs`, `lib.rs`, `adapter/{mod.rs, jetstream.rs, redis.rs, noop.rs}`, `consumer/{filter.rs, merge.rs, mod.rs}`, `ffi/{mod.rs, cortex.rs, mesh.rs}`, `shard/{mod.rs, mapper.rs, batch.rs, ring_buffer.rs}`.

Extended scope (#80 onward): a follow-up multi-agent sweep added point-checks across the previously-deferred `adapter/net/` UDP transport stack and the `sdk/` surface. The new findings are spot-checks, not a systematic re-audit of those subtrees — additional defects may remain.
- `sdk/src/{net.rs, ...}`
- `adapter/net/redex/{disk.rs, file.rs}`
- `adapter/net/{mesh.rs, session.rs, router.rs, linux.rs}`
- `adapter/net/subnet/gateway.rs`
- `adapter/net/contested/correlation.rs`

Extended scope (#97 onward): a third multi-agent sweep covered the remaining UDP transport, behavior, compute, and continuity/state subtrees the prior pass left out of scope.
- `adapter/net/{mod.rs, session.rs, pool.rs, reliability.rs, failure.rs, crypto.rs, protocol.rs}`
- `adapter/net/{swarm.rs, route.rs, reroute.rs, proxy.rs, router.rs, traversal/classify.rs}`
- `adapter/net/behavior/{safety.rs, loadbalance.rs, capability.rs, proximity.rs, api.rs, rules.rs}`
- `adapter/net/compute/{orchestrator.rs, standby_group.rs, migration_target.rs, migration_source.rs}`
- `adapter/net/subprotocol/migration_handler.rs`
- `adapter/net/continuity/{chain.rs, superposition.rs, discontinuity.rs}`
- `adapter/net/cortex/memories/fold.rs`
- `adapter/net/traversal/portmap/natpmp.rs`

Extended scope (#121 onward): a fourth multi-agent sweep covered the subtrees explicitly called out as "still not re-audited" in the prior pass.
- `adapter/net/state/{causal.rs, horizon.rs, log.rs, snapshot.rs}`
- `adapter/net/cortex/{adapter.rs, config.rs, envelope.rs, error.rs, meta.rs, mod.rs, tasks/*, memories/*}` (excluding `memories/fold.rs`, already audited)
- `adapter/net/netdb/{db.rs, error.rs, mod.rs}`
- `adapter/net/identity/{entity.rs, envelope.rs, origin.rs, token.rs}`
- `adapter/net/subprotocol/{descriptor.rs, negotiation.rs, registry.rs, stream_window.rs}`
- `adapter/net/behavior/{context.rs, metadata.rs, diff.rs}`

## Status (running tally)

**Outstanding:** #55, #56, #57, #58, #59, #60, #61, #62, #63, #64, #65, #66, #67, #68, #69, #70, #71, #72, #73, #74, #75, #76, #77, #78, #79, #98, #100, #104, #105, #106, #107, #108, #109, #110, #111, #112, #113, #114, #115, #116, #117, #118, #119, #120, #121, #122, #123, #124, #125, #126, #127, #128, #129, #130, #131, #132, #133, #134, #135, #136, #137, #138, #139, #140, #141, #142, #143, #144, #145, #146, #147, #148, #149, #150, #151, #152, #153, #154, #155

**Fixed on 2026-04-30 (with regression tests where reasonable):**
- **#80** — `Net::shutdown` now routes through `EventBus::shutdown_via_ref(&self)`, an idempotent reference-based shutdown that runs regardless of outstanding `Arc<EventBus>` clones. Tests: `sdk/tests/shutdown_regression.rs::{shutdown_runs_even_with_outstanding_event_stream, shutdown_via_ref_is_idempotent}`.
- **#81** — implicitly fixed by #80. With `shutdown_via_ref` no longer gating on `Arc::try_unwrap`, `EventStream`'s perpetuated `Arc<EventBus>` clones are now benign — shutdown still runs, in-flight poll futures observe the `shutdown_completed` flag on their next operation, and the inner bus drops when the last clone is released. Same tests as #80.
- **#82** — `manual_scale_down` now drives the full lifecycle (scale_down → poll for empty → finalize_draining → remove_shard_internal) rather than only marking shards `Draining`. Now async. Test: `bus::tests::manual_scale_down_finalizes_and_removes_drained_shards`.
- **#83** — `ShardManager::remove_shard` now calls `mapper.remove_stopped_shards()` after the manager-level removal, so the mapper's `shards: RwLock<Vec<MappedShard>>` no longer accumulates `Stopped` entries indefinitely across scale-up/down cycles. Implicitly covered by `manual_scale_down` regression test (which would underflow `num_shards` if mapper-side cleanup was missing).
- **#85** — Mesh dispatch now invokes the new `verify_heartbeat_aead` helper (which mirrors the legacy adapter's AEAD verification) before touching `failure_detector` or `session.touch()`. Tests: `mesh::heartbeat_aead_tests::{aead_authenticated_heartbeat_passes_verification, unauthenticated_heartbeat_fails_verification, replay_of_authenticated_heartbeat_fails_verification_on_second_try}`.
- **#88** — Subnet gateway now treats `hop_ttl == 0` as expired (drop) rather than "unlimited". The TTL check is now `if hop_ttl == 0 || hop_count >= hop_ttl`. Existing tests using a zero TTL were updated to use a non-zero value where TTL wasn't the focus. Test: `subnet::gateway::tests::ttl_zero_is_treated_as_expired`.
- **#90** — `BatchedTransport::recv_batch` and `recv_batch_blocking` now return `io::ErrorKind::Unsupported` when called on an instance constructed via `new_send_only`, replacing the index-out-of-bounds panic. Test: `linux::tests::recv_batch_returns_unsupported_for_send_only_transport`.
- **#92** — `sweep_retention` now renormalizes `state.index` offsets to be segment-relative and rebases `segment.base_offset` to 0 after a successful `compact_to`. Test: `redex::file::tests::sweep_retention_post_sweep_appends_survive_restart`.
- **#94** — `append_entry_inner` and `append_entries_inner` now wrap the second/third `metadata()` calls in an explicit `match` that runs the dat (and dat+idx) rollback before returning, instead of relying on `?` which short-circuits the rollback paths. Tests: `redex::file::tests::{append_rolls_back_dat_on_idx_metadata_failure, append_rolls_back_dat_and_idx_on_ts_metadata_failure}`.
- **#95** — `sweep_retention` now performs `disk.compact_to` BEFORE mutating in-memory state. Test: `redex::file::tests::sweep_retention_keeps_in_memory_state_when_disk_compact_fails`.
- **#97** (third-pass entry) — both heartbeat senders (`mesh.rs:3220` and the legacy `mod.rs:841`) now acquire from the session's shared `packet_pool()` instead of constructing fresh `PacketBuilder::new(&[0u8; 32], ...)` on every tick. The all-zero key meant the AEAD tag never matched the receiver's RX cipher; the fresh `tx_counter` per-builder meant successive heartbeats reused counter=0 (replay-rejected post-#85). The session pool fixes both — same key, persistent counter. Tests: targeted unit test `mesh::heartbeat_aead_tests::pooled_heartbeat_builds_succeed_in_sequence_and_verify` (acquires two heartbeats from the same session pool and verifies both decode against the peer's RX cipher in order — pins both the key and the counter dimensions); end-to-end coverage from `failure_detector_matrix::*` (drives real Mesh handshakes and depends on legitimate heartbeats keeping unaffected peers `Healthy` — these tests broke without this fix once #85 verification went live).
- **#84** — resolved as **docstring fix**, not a behavioral change. Re-investigation surfaced that the receive-time auto-grant is the documented v2 design (see `mesh.rs:3110-3135`: "Accounting runs at receive time (not drain time); this closes the v1 gap where a single serial sender ran `Transport(io::Error)` into a full kernel buffer"). Per-stream kernel-buffer protection comes from the round-trip grant loop; per-application throttling is provided by per-shard queue-depth limits, not this counter. The `RxCreditState` rustdoc previously described a threshold-emit pattern that didn't match the implementation — that discrepancy was the actual bug. The docstring has been rewritten to describe the receive-time-accounting design accurately. Tests: `session::tests::{test_rx_credit_emits_authoritative_total_consumed, rx_credit_outstanding_stays_at_window_under_receive_time_accounting (new)}`. (An earlier round of this audit applied a behavioral threshold-emit fix; that fix broke `three_node_integration::test_v2_serial_sender_sees_backpressure_on_slow_receiver` which depends on the receive-time auto-grant. The behavioral change has been reverted in favour of the docstring fix.)
- **#86** — direct-handshake initiator now registers an oneshot in a new `pending_direct_initiators: DashMap<SocketAddr, oneshot::Sender<Bytes>>` BEFORE sending msg1, then awaits the oneshot when `started == true`. The dispatcher's direct-handshake branch (`mesh.rs:2440`) looks up the source addr in the registry and forwards the parsed payload bytes through. Pre-`start()` the initiator falls back to the original `recv_from` path (no race exists pre-start). Concurrent direct connects to distinct peers no longer race for the same socket on the initiator side. Tests: `tests/connect_post_start.rs::{initiator_connect_after_start_completes_handshake, second_connect_after_first_uses_registry_path}`. Note: the responder side of the same race (`try_handshake_responder` polling `recv_from` post-`start()`) is NOT addressed by this fix; the documented contract is "`accept` must be called before `start`," and a fully symmetric responder-side registry would require a different design (the responder doesn't know the peer's addr in advance). That broader fix is deferred.
- **#93** — `compact_to` now fsyncs the parent directory after the three-rename sequence via a new `fsync_dir(&Path)` helper. On POSIX, `rename` is not durable until the directory inode is fsynced — without this, a power loss between successful rename calls and a subsequent fsync could leave the directory pointing at the OLD inodes. On Windows the helper is a no-op (rename durability is governed by separate APIs that stdlib doesn't expose; durability is best-effort under the current implementation). The cross-file atomicity gap (a manifest-pointer scheme would be needed to fully close it) is a remaining limitation, called out in the inline comment at the rename site. Test: `disk::tests::fsync_dir_helper_succeeds_on_a_normal_directory`; existing redex compaction tests implicitly cover the integrated path.
- **#97 follow-up** (counter-pool conflict): the first round of #97 routed heartbeats through `session.packet_pool()` — that fixed the all-zero-key + per-builder-counter bugs but introduced a *new* counter conflict because `packet_pool` and `thread_local_pool` each own their own `tx_counter`, and the data path uses `thread_local_pool`. The receiver verifies all packets against a single `rx_cipher` with one replay window, so heartbeats and data must share the same sender-side counter or one will be replay-rejected. Fix: route heartbeats through `thread_local_pool` (same pool as data) so counters monotonically increase across BOTH heartbeats and data. Test: `mesh::heartbeat_aead_tests::packet_pool_and_thread_local_pool_have_independent_counters` pins the invariant that the two pools have independent counters, so any future change routing heartbeats back through `packet_pool` would re-introduce the conflict and this test would catch it. This regression also restores the `tests/integration_net.rs` suite (`test_net_send_receive_fire_and_forget` and friends), which was failing on `e531b61` because of this counter conflict.
- **#87** — post-handshake state inserted at `mesh.rs:2524-2534` is now protected by a `PeerRegistrationGuard` whose `Drop` impl runs the rollback. The guard moves the rollback off the spawned future's success arm and onto its `Drop`, which fires synchronously whenever the future is dropped (cancellation, panic, runtime shutdown). The success path calls `guard.commit()` (`mem::forget`-equivalent) to skip the rollback. Tests: three unit tests in `mesh::heartbeat_aead_tests::peer_registration_guard_*` covering rollback-on-drop, no-op-on-commit, and concurrent-overwrite preservation.
- **#89** — `RoutingTable::record_in/out/drop` now gate insertions on a soft cap of `MAX_STREAM_STATS = 65_536` (set at `route.rs`). Existing entries always continue to record; novel `stream_id`s are only admitted while the map is below the cap. `cleanup_idle_streams` reclaims slots for legitimate streams as they idle out, after which new IDs may be admitted again. Tests: `route::tests::{record_in_stops_admitting_new_streams_at_cap, cap_admits_new_streams_after_cleanup_reclaims_slots}`.
- **#91** — `analyze_subnet_correlation` now sorts `subnet_counts` entries by `(depth desc, subnet_id asc)` before scanning for the threshold-meeting subnet. Ties at the same depth resolve to the lower `SubnetId` (derived `Ord` on the inner `u32`) deterministically across process invocations — pre-fix, hash iteration order picked an arbitrary winner. Test: `correlation::tests::ties_resolve_deterministically_across_runs` (32-iteration loop with two equally-deep subnets at threshold).
- **#81** — implicitly fixed by #80 (entry already updated above to reflect this).
- **#99** — `SuperpositionState::continuity_proof` previously used `head.parent_hash` as the proof's hash but anchored `from_seq`/`to_seq` at `head.sequence`. Since `parent_hash` is the forward hash of the *previous* event (event at `sequence - 1`), the verifier (`compute_parent_hash` of event AT from_seq) would never match. The fix anchors at `head.sequence - 1` so the seq aligns with the hash bytes. Pre-fix also mixed identities when `target.seq < source.seq` (`from_seq` was target's but `from_hash` was source's parent_hash); the new lo/hi-by-seq pattern picks the head matching each anchor. Tests: `superposition::tests::test_continuity_proof` (updated assertions), `superposition::tests::continuity_proof_round_trips_through_entity_log` (new — builds a real `EntityLog` via `CausalChainBuilder`, derives a SuperpositionState from it, and verifies the resulting proof against the log; pre-fix this fails with `HashMismatch`).
- **#101** — `EndpointState::is_circuit_open` is now a pure predicate (no side effects). The half-open probe slot is claimed lazily at selection time via the new `try_claim_half_open_probe` method, called only on the endpoint actually chosen by the selector. If `try_record_request` then fails (max-conn cap, race), the new `release_half_open_probe` reverts the claim so it doesn't strand. Pre-fix, `is_circuit_open`'s CAS-claim ran during the filter scan over all endpoints — a multi-endpoint outage past its recovery window claimed the probe slot on every candidate, while only one was selected; the N-1 others were stranded forever. Tests: `loadbalance::tests::circuit_breaker_does_not_leak_probe_slot_on_multi_endpoint_scan` (3-endpoint outage; recovery elapses; one select call must claim the probe slot on EXACTLY one endpoint, not all three). All 23 existing loadbalance tests still pass.
- **#102** — `SafetyEnforcer::release` now uses `fetch_update` + `saturating_sub` for `concurrent` and `memory_mb`, mirroring the tokens/cost paths. Pre-fix `release` ran raw `fetch_sub`; combined with `Disabled`-mode `acquire` (which short-circuits without incrementing those counters), a release in `Disabled` mode would underflow `u32` to ~4 billion. Hot-swapping to `Enforce` then made every subsequent acquire fail with `ResourceLimitExceeded` until process restart. Test: `safety::tests::release_does_not_underflow_concurrent_or_memory_in_disabled_mode` (acquire+drop in Disabled, hot-swap to Enforce, acquire again — must succeed).
- **#103** — `StandbyGroup::promote` now searches for `best_standby` BEFORE mutating the old active's role/health. Pre-fix it marked the old active `Unhealthy`/`Standby` first, then searched; on `NoHealthyMember` it returned `Err` but left the group with a demoted, unhealthy "active" pointer. `on_node_recovery` only restores health, not the `Active` role, so the group was silently demoted forever. Test: `standby_group::tests::promote_does_not_half_mutate_on_no_healthy_member` (3-member group, mark all standbys unhealthy, promote → asserts `NoHealthyMember` AND that the active's role/health/index are unchanged).

**Refuted on verification:** #96 (`read_timestamps` torn-tail alignment — alignment is preserved by construction, see entry).

**Verified (read end-to-end on 2026-04-30):** #80, #81, #82, #83, #85, #86, #87, #88, #89, #90, #91, #92, #93, #94, #95. #84 was found to be **mis-located** — the cited code is correct; the bug is at the upstream caller `mesh.rs:3000-3008` (see entry).

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

### 81. `Net::subscribe` perpetuates `Arc<EventBus>` clones, making #80 the default outcome (verified)
**File:** `sdk/src/net.rs:191-193`

`EventStream::new(self.bus.clone(), opts)` increments the strong count of `Arc<EventBus>`. Even after the user drops the `EventStream`, any in-flight poll future the stream spawned can still hold the clone, so `Net::shutdown`'s `Arc::try_unwrap` fails (#80). The SDK provides no escape hatch — there is no `shutdown_async`-after-flush, no synchronous drain primitive — for the documented "subscribe → done streaming → shutdown" pattern. Fix is paired with #80: once shutdown is signal-based, surviving clones become benign.

**Verification (2026-04-30):** confirmed by reading `sdk/src/net.rs:191-193` (subscribe) and `:198-203` (subscribe_typed) — both call `self.bus.clone()` into the stream constructor.

**Status: implicitly fixed by #80** (2026-04-30). The #80 fix changed `Net::shutdown` to call `bus.shutdown_via_ref(&self)` directly — no more `Arc::try_unwrap` gate. Perpetuated `Arc<EventBus>` clones from `EventStream` and `TypedEventStream` are now benign: shutdown's CAS + drain runs regardless of strong-ref count, in-flight poll futures see the bus's `shutdown_completed` flag flip on their next operation, and the inner `EventBus` drops when the last clone is released. The two regression tests for #80 (`shutdown_runs_even_with_outstanding_event_stream` and `shutdown_via_ref_is_idempotent`) cover this path: the first explicitly holds an `EventStream` (which clones the Arc) across the shutdown call and asserts shutdown still succeeds.

### 84. `RxCreditState::on_bytes_consumed` is *itself* correct — but the dispatcher calls it on every accepted packet, refunding credit before the application has actually consumed (refined location)
**File:** `adapter/net/mesh.rs:3000-3008` (caller); `adapter/net/session.rs:781-788` (callee, correct per docstring)

The original entry pinned the bug at `RxCreditState::on_bytes_consumed` itself. **Verification (2026-04-30) showed the function is correct per its docstring:** it is documented as "called when bytes are consumed by the application" and emits an authoritative grant covering whatever has been consumed so far (`session.rs:773-780`). The bug is upstream — `mesh.rs:3000-3008`:

```rust
if accepted {
    stream.update_rx_seq(parsed.header.sequence);
    stream.on_bytes_consumed(payload_bytes)
} else {
    None
}
```

The dispatcher invokes `on_bytes_consumed` immediately on packet *acceptance* (delivery into the reliability layer's in-order buffer), not on the application actually draining bytes off the receive queue. As a result, every accepted byte triggers an authoritative grant for that same byte count. The window opens just-in-time for whatever the network delivered, and the receiver never applies backpressure regardless of how slowly the application reads — the only effective "backpressure" is whatever the reliability layer's NACK/SACK behaviour buys.

**Trigger:** open a stream with `window_bytes = 65_536`; have the dispatcher accept 100 KB of inbound traffic without the application reading; sender's credit replenishes 1:1 with what arrives, never blocks.

Fix: move the `on_bytes_consumed` call to the application-side read path (whatever drains payloads off the per-stream queue), or rename it `on_bytes_delivered` and add a separate `on_bytes_consumed` that the application drives. The audit's previously-suggested "threshold check inside `RxCreditState`" would not actually fix this — the function is doing what it's told.

### 85. Mesh dispatch path skips AEAD verification on heartbeat packets (verified)
**File:** `adapter/net/mesh.rs:2367-2371` (compare with `adapter/net/mod.rs:642-663` and `adapter/net/pool.rs:237-249`)

```rust
if parsed.header.flags.is_heartbeat() {
    failure_detector.heartbeat(peer_node_id, source);
    session.touch();
    return;
}
```

The mesh dispatch loop fast-paths `is_heartbeat()` packets — touching the failure detector and session timestamp — without invoking AEAD verification. The legacy single-peer adapter (`mod.rs:642-663`) explicitly verifies the tag for the same packet shape (calls `rx_cipher.decrypt(counter, &aad, &parsed.payload)` at `mod.rs:656`), and the comment on `pool.rs:237-249` claims heartbeats are now AEAD-authenticated specifically to prevent off-path spoofing. An off-path attacker with the cleartext `session_id` (visible on every prior data packet) and the source UDP address can spoof heartbeat-flagged 64-byte headers from `peer_addr`, indefinitely defeating session-idle timeout and triggering false `failure_detector.heartbeat(...)` notifications.

**Verification (2026-04-30):** the mesh path matches a session by `session_id` (`mesh.rs:2348-2364`), then calls `failure_detector.heartbeat()` and `session.touch()` with no `rx_cipher.decrypt(...)` call before the early `return`. The legacy `mod.rs:642-663` path correctly calls `rx_cipher.decrypt` at line 656.

Either route heartbeats through the same AEAD-verify path as data packets, or document this as a known design limitation.

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

### 93. Redex `compact_to` non-atomic three-rename sequence with no parent-dir fsync (verified)
**File:** `adapter/net/redex/disk.rs:1086-1089`

The atomic-rewrite pattern uses three sequential `std::fs::rename` calls (idx → dat → ts) without bracketing them in a single dirent flip and without fsyncing the parent directory afterward. A power loss between the first and second rename leaves the new (renormalized) idx alongside the old dat + old ts; on reopen, recovery's checksum verification (`disk.rs:322-348`) fails for every entry because the new idx's offsets index into the wrong dat bytes, and all entries are dropped. On POSIX, even a successful series of renames is not durable until the directory inode is fsynced. Combined with #92, a crash during retention sweep can corrupt the entire segment with no recovery path.

**Verification (2026-04-30):** read `compact_to` in full. No `File::open(&dir).sync_all()` exists in the function; the three renames at `:1087`, `:1088`, `:1089` are unbracketed; placeholder cleanup (`:1119-1121`) is best-effort. Fix: either move to a single rename of a packed manifest, or fence the three renames inside an explicit dir-fsync.

### 94. Redex `metadata()?` early-return after a prior file write leaves orphaned bytes without rollback (verified, lines refined)
**File:** `adapter/net/redex/disk.rs:638, 663` (`append_entry_inner`); `:802, 820` (`append_entries_inner`)

In `append_entry_inner` the order is: dat write (`:607`) → idx metadata (`:638`) → idx write (`:639`) → ts metadata (`:663`) → ts write (`:664`). The bug is at the **second and third** metadata calls: by the time `idx.metadata()?` runs at `:638`, the dat write at `:607` has already committed bytes to disk; by the time `ts.metadata()?` runs at `:663`, both dat and idx have committed. Each `?` early-returns via `RedexError::io(...)` without entering the explicit `if let Err(e) = file.write_all(...)` rollback block, which is the only place that issues `set_len` truncations. Result: the on-disk state ends up with orphaned dat (or dat+idx) bytes; the caller is told the append failed and rolls back `next_seq` in memory. The same pattern holds for the batch path: dat write (`:787`) → idx metadata (`:802`) → idx write (`:803`) → ts metadata (`:820`).

**Verification (2026-04-30):** read both functions end-to-end. The first metadata call in each (`:606`, `:786`) is fine because no writes have happened yet. The bug is real for `:638`, `:663`, `:802`, `:820`. On restart with orphaned dat (idx-metadata failure case), the torn-tail recovery walk at `disk.rs:245-269` will trim dat to `retained_dat_end`, so the orphan dat alone is harmless. But for orphaned dat+idx (ts-metadata failure case), the surplus idx record references a payload offset whose bytes still exist in dat → the entry is "recovered" without a matching ts entry, so `read_timestamps` returns None for the length mismatch and all recovered entries get `now()` as their timestamp; age-based retention silently breaks for the affected window. Fix: replace each `?` with explicit error handling that triggers the existing rollback path before returning.

### 95. Redex `sweep_retention` commits in-memory eviction even when `compact_to` fails (verified)
**File:** `adapter/net/redex/file.rs:919-1003` (failure point at `:991-998`)

`sweep_retention` mutates in-memory state (drains `index`, `timestamps`, calls `evict_prefix_to`) at `:946-957` before invoking `disk.compact_to` at `:991`. If `compact_to` fails, lines `:992-997` log a warning whose message literally reads "in-memory eviction succeeded but on-disk files retain evicted entries" — the comment is an explicit acknowledgment. The function returns implicitly with `()` (no `Result`); there is no rollback (no re-prepending to `state.index`, no restoration of segment base). On the next reopen, recovery replays the full on-disk state, resurrecting the entries that were just evicted in memory. Combined with #92 it becomes a corruption vector (post-failure appends interleave with resurrected entries on disk).

**Verification (2026-04-30):** read `sweep_retention` end-to-end. Fix: either roll back in-memory eviction on `compact_to` failure, or perform the disk compaction first and only mutate in-memory state on success.

### 97. Heartbeat senders build with an all-zero key + fresh per-tick counter — every heartbeat would fail AEAD verify (verified, fixed)
**File:** `adapter/net/mod.rs:841` (legacy NetAdapter); also independently present at `adapter/net/mesh.rs:3220` (Mesh heartbeat timer)

**Original claim:** `build_heartbeat` (`pool.rs:251`) AEAD-encrypts an empty payload with the builder's cipher. The builder was constructed with `&[0u8; 32]` so the Poly1305 tag was computed under key=0; the receiver verifies with `session.rx_cipher()` (the real session key) and would always fail.

**Verification (2026-04-30):** the bug had a *second* compounding dimension that surfaced once #85 was fixed (mesh receiver started AEAD-verifying heartbeats). Each fresh `PacketBuilder::new(...)` owns its own `tx_counter: Arc<AtomicU64>` starting at 0 (`crypto.rs:523`), so even with the right key, successive heartbeats would all encrypt under counter=0, and the receiver's replay window would accept the first and reject every subsequent one. The legacy `mod.rs:1742` regression test happened to test only one heartbeat, so this dimension was invisible until end-to-end Mesh tests started failing (`failure_detector_matrix::*`) once AEAD verify was wired up.

The same bug pattern existed at `mesh.rs:3220` — also with an all-zero key, also building a fresh PacketBuilder per heartbeat tick. Caught while validating the #85 fix end-to-end.

**Fix:** both senders now acquire from `session.packet_pool().get()`. The pool's builders are constructed once per session with the right key and a single shared `tx_counter: Arc<AtomicU64>` (`pool.rs:341`), so:
- Every heartbeat uses the session's actual TX key → AEAD verify succeeds against the receiver's RX cipher.
- The shared counter increments atomically per packet → no replay-window collisions.

**Regression coverage:** the three end-to-end `failure_detector_matrix::*` tests (`partition_of_one_peer_does_not_mark_unrelated_peers_failed`, `partition_heal_recovers_peer_to_healthy_status`, `peer_failure_clears_capability_index_via_harness`) drive real Mesh handshakes and depend on legitimate heartbeats keeping unaffected peers `Healthy`. Without this fix (after #85 lands), all three fail. With it, all five tests in that file pass. The pre-existing legacy-adapter unit test `mod.rs::heartbeat_is_aead_authenticated` still passes (it builds the heartbeat directly with the right key, exercising the receiver-side verify only).

### 98. `ContinuityProof::verify_against` only checks the two endpoint events, never validates the chain in between
**File:** `adapter/net/continuity/chain.rs:103-137`

`verify_against` recomputes the parent hashes for the events at `from_seq` and `to_seq` and asserts they match the proof's `from_hash` / `to_hash`. It never iterates the events between those two anchors and never verifies that consecutive `parent_hash` chains link up. A peer can claim a continuity proof spanning `from=0, to=1000` while having lost or fabricated events 1..999, and verification still passes as long as the two endpoints hash correctly. There is also no check that `from_seq <= to_seq` — reversed bounds are accepted. **Failure scenario:** node A produces a 1000-event chain. A malicious intermediary holding only events 0 and 999 builds a proof with the correct two endpoint hashes; B's `verify_against` accepts it and propagates "continuous chain from 0 to 999" even though the middle is missing. This is the primary continuity-bypass vector — exactly what the proof was supposed to prevent. Walk the event range, verify each `parent_hash` chains to the previous, and bound the iteration to a sane maximum.

### 99. `SuperpositionState::continuity_proof` constructs a proof with backward-pointing parent hashes (verify will always fail)
**File:** `adapter/net/continuity/superposition.rs:133-141`

```rust
ContinuityProof {
    origin_hash: self.origin_hash,
    from_seq: self.source_head.sequence.min(self.target_head.sequence),
    to_seq:   self.source_head.sequence.max(self.target_head.sequence),
    from_hash: self.source_head.parent_hash,
    to_hash:   self.target_head.parent_hash,
}
```

`CausalLink::parent_hash` is the **backward**-pointing predecessor hash. But `ContinuityProof::verify_against` (`chain.rs:110`) computes `compute_parent_hash(&event.link, &event.payload)` for the event at `from_seq` — the **forward**-pointing self hash. These are different bytes. The proof can never verify correctly against any log built from the same chain. Compounding: when `target_head.sequence < source_head.sequence` (the common case during Replay), `from_seq` is target's seq but `from_hash` is source's predecessor hash — mixing identities. **Failure scenario:** every migration that enters Replay phase advertises a continuity proof; every peer that runs `verify_against` rejects it; meshes treat the migration as `Forked` / `Unverifiable` and either refuse routing or trigger spurious re-bootstrapping. Use `compute_parent_hash(&head.link, &head.payload)` for both endpoints, and clamp the from/to ordering to a single direction.

### 100. `LocalGraph::on_pingwave` lets unverified peers poison node addresses and flood the node DashMap
**File:** `adapter/net/swarm.rs:489-531`

A pingwave's `addr` field is taken from the forwarder's `from` socket address and stored unconditionally as `LocalGraph.nodes[origin_id].addr` (line 513). Any peer forwarding a pingwave for `origin_id=Y` overwrites Y's recorded address with the forwarder's address. A malicious peer can also flood pingwaves with arbitrary `origin_id` values (8 random bytes per packet), growing `LocalGraph.nodes` (line 517) and `seen_pingwaves` (line 502) at line-rate; cleanup runs on a 30s/10s timer, so per-window growth is bounded only by link bandwidth. `mesh.rs` route-install gates on `addr_to_node` (rule 4 at `mesh.rs:2181`), but `LocalGraph` itself has no such gate, and is exported as a public API in `mod.rs:146`. **Adverse outcome:** route-address poisoning + memory exhaustion from any peer that completes the cheap mesh-handshake gate. AEAD-verify pingwave origin / forwarder identity before insert, and cap `nodes`/`seen_pingwaves` size with an LRU policy.

### 101. Half-open probe slot leaks via `is_circuit_open`'s filter-time side effect — the breaker becomes permanently stuck
**File:** `adapter/net/behavior/loadbalance.rs:365-381` (claim) + `682-688, 720` (consumers)

`is_circuit_open` is *both* a predicate AND has a side effect: when the recovery window has elapsed it CAS's `half_open_probe` from `false→true`, claiming the probe slot for whoever asked, and only `record_completion` ever clears it. But `is_circuit_open` is invoked from `get_available_endpoints` (line 720) — i.e. every endpoint the load balancer is filtering — so a single `select()` call with N circuit-open endpoints past their recovery window claims the probe slot on **all N** of them, while only one (or zero) endpoint will actually be selected. The N-1 others have `half_open_probe=true` with no in-flight request, no timer, and no completion path; the slot leaks and every subsequent `is_circuit_open` returns true forever for those endpoints. A second leak: even on the chosen endpoint, if `try_record_request` (line 684) legitimately fails (max-conn cap, race), the retry loop continues without clearing the slot. **Adverse outcome:** any cluster with >1 endpoint on long-running load — the first post-recovery `select()` after a multi-endpoint outage permanently strands every endpoint except the one chosen, and the chosen endpoint also strands itself if `try_record_request` happens to fail. The breaker recovery flow is practically un-recoverable until process restart. Either separate "test if open" from "claim probe", or guarantee the slot is released along every path that claims it.

### 102. `SafetyEnforcer::release` underflows resource counters in `EnforcementMode::Disabled`
**File:** `adapter/net/behavior/safety.rs:997-1003, 1245-1251`

`acquire()` short-circuits in `Disabled` mode and returns a `ResourceGuard` **without** incrementing any usage counter (lines 997-1003). When the guard drops, `release()` unconditionally calls raw `fetch_sub` on the `concurrent` (AtomicU32) and `memory_mb` (AtomicU32) counters at lines 1247-1251. From a zero counter this wraps to ~`u32::MAX`. The matching tokens/cost code (lines 1254-1264) uses `fetch_update` with `saturating_sub` precisely because the comment at `check_resource_limits` (line 1417) acknowledges this hazard — but the same hardening was not applied to `concurrent` or `memory_mb`. **Failure scenario:** an operator runs in `Disabled` mode for warm-up / dry-run, then flips back to `Enforce` (envelope is hot-swappable via `update_envelope`). The first enforce-mode `acquire()` reads the wrapped counter, decides `current.saturating_add(claim) > max_concurrent`, and returns `ResourceLimitExceeded`. Every subsequent request is rejected forever until process restart. Use `fetch_update` with `saturating_sub` for `concurrent` and `memory_mb` like the tokens/cost paths already do.

### 103. `StandbyGroup::promote` half-mutates state when no standby is healthy
**File:** `adapter/net/compute/standby_group.rs:267-281`

The function applies `mark_unhealthy(old_active)` and sets `members[old_active].role = Standby` *before* searching for `best_standby`. If the search returns `NoHealthyMember`, the function exits with `Err` but leaves `self.active_index` still pointing at `old_active` — whose role is now `Standby` and whose health is now `Unhealthy`. **Failure scenario:** a node fails such that the active and the only viable standby both go down (split-network). `on_node_failure` calls `promote()`, promotion errors out, and the group is now in a state where `active_origin()` returns a `Standby`/unhealthy member. A subsequent `on_node_recovery` for the old active doesn't re-promote — it only marks healthy, leaving `role = Standby`. The group is silently demoted forever. Move the role/health mutations after the standby search succeeds, or roll them back in the `Err` arm.

### 104. Local-source migration silently mutates source daemon state after snapshot is sent
**File:** `adapter/net/compute/orchestrator.rs:911-946`

When `source_node == self.local_node_id`, `start_migration` calls `daemon_registry.snapshot()` directly and never invokes `MigrationSourceHandler::start_snapshot`. As a result, `source_handler.is_migrating(origin)` returns `false`, no events are buffered on the source side, and the source daemon stays registered — continuing to accept `deliver()` calls and mutating its in-memory state *after* the snapshot has been sent to the target. **Failure scenario:** daemon at origin O is migrating from local node. Caller invokes `start_migration(O, local, target)`; orchestrator captures snapshot at `seq=100`. While the target restores, more events arrive at the source via `DaemonRegistry::deliver()` and advance the daemon to `seq=120`. Nothing buffers them (orchestrator's own `buffer_event` is a separate code path the caller may not invoke). At cutover, source is unregistered with seq=120 of unsaved state; target activates at seq=100. Events 101-120 are lost. Compare to the dispatcher's `TakeSnapshot` path (`migration_handler.rs:310-312`) which DOES call `source_handler.start_snapshot` and then routes events through `buffer_event`. Mirror that path for local migrations.

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

### 82. `manual_scale_down` strands events on drained shards (verified)
**File:** `bus.rs:815-824` (compare scaling-monitor finalize at `bus.rs:935-952`)

`manual_scale_down` calls `mapper.scale_down(count)` to mark shards `Draining` and returns the drained ids. Unlike the scaling-monitor path, it never calls `mapper.finalize_draining()` or `bus.remove_shard_internal(...)`. Events still queued in those shards' ring buffers (and any pushes that arrived between the read-locked early budget check and the write-locked state transition) sit unread until the scaling monitor catches up — which only fires if a monitor is running and reaches its next tick before `bus.shutdown()`. Bus configs without an active monitor lose those events on shutdown.

**Verification (2026-04-30):** confirmed by reading `bus.rs:815-824`. The function is six lines: it acquires the mapper, calls `mapper.scale_down(count)`, and returns the drained ids. There is no finalize call, no `remove_shard_internal`, no integration with the drain-worker shutdown path. Fix: either drive the finalize loop synchronously inside `manual_scale_down`, or document the API as "requires `start_scaling_monitor` before use" and assert it at call time.

### 83. `ShardManager::remove_shard` never drops the entry from `ShardMapper.shards` — unbounded growth across scale-up/down cycles (verified)
**File:** `shard/mod.rs:819-872`

`remove_shard` drains the ring buffer (`:833-839`), rebuilds the routing table (`:842-864`), and decrements `num_shards` (`:866-868`), but never asks the mapper to drop the corresponding `MappedShard` record. The function's only mapper interaction is `let _mapper = self.mapper.as_ref().ok_or(...)` at `:823` — a guard against calling on a non-scaling configuration; no method is invoked on the bound. The mapper's `shards: RwLock<Vec<MappedShard>>` keeps growing — every scale-up appends an entry, and `remove_stopped_shards` is the only API that deletes them, but no production caller invokes it (only tests reference it). Long-running services with frequent scaling activity accumulate `Stopped` entries indefinitely; `evaluate_scaling` filters by state but still iterates the full list, so per-tick cost grows with cumulative scaling history.

**Verification (2026-04-30):** read `shard/mod.rs:819-872` end-to-end. Fix: wire `remove_shard` to call `mapper.remove_stopped_shards()` after the manager-level removal completes, or remove the specific id directly via a new mapper method.

### 86. Direct handshake `recv_from` on `Arc<UdpSocket>` races the dispatch receive loop (verified)
**File:** `adapter/net/mesh.rs:6093-6118, 6155-6176` (dispatch loop spawn at `:2032`)

`try_handshake_initiator` and `try_handshake_responder` poll `socket_arc.recv_from` directly. Once `start()` has spawned `spawn_receive_loop` (`mesh.rs:2032-2092`, which consumes from the same `self.socket.socket_arc()` at `:2033` via `PacketReceiver`), both consumers race for each datagram on the same `Arc<UdpSocket>`; tokio dispatches a UDP datagram to exactly one waiter. The handshake response can be swallowed by the dispatch loop, which then drops it because `dispatch_packet` at `:2344-2346` returns when `is_handshake()` is true on the direct path — there is no handshake channel forwarding unmatched-session datagrams. Concurrent direct connects on the same node also steal each other's responses.

**Verification (2026-04-30):** confirmed there is no documented "must call before `start()`" invariant on `connect()` (`mesh.rs:1619-1627`) or `accept()` (`:1678-1679`); both are public API. Fix: bridge handshakes through an in-memory channel populated by the dispatch loop (forward msg2/msg3 datagrams to a per-pending-handshake oneshot keyed by `session_id`), or synchronize handshake initiation to suspend dispatch dequeue for the matching session.

### 87. Mesh post-handshake `tokio::spawn` is fire-and-forget and can wedge peer/session/route on cancellation (verified)
**File:** `adapter/net/mesh.rs:2553-2569` (state insert at `:2524-2534`)

After completing the responder handshake, the code inserts session, peer entry, routing entry, and `peer_addrs` (`:2524-2534`), then `tokio::spawn`s a fire-and-forget `socket.send_to(&payload, next_hop).await` whose only rollback fires from inside the spawned future on socket-send error (`:2563-2567`). The spawn is a bare `tokio::spawn(async move { ... })` with no `JoinHandle` retained anywhere. If the runtime is shutting down or the spawned task is cancelled before the send completes, the rollback at `:2563-2567` never runs but the peer/session/route state at `:2524-2534` is already wedged. There is no idle-session sweeper that reaps unsendable peer entries — `cleanup_idle_streams` (`route.rs:624-635`) only cleans `stream_stats`, not peer/route entries.

**Verification (2026-04-30):** confirmed no `JoinHandle` capture. Fix: either await the send synchronously on the handshake task, or track the JoinHandle alongside the rollback closure so cancellation triggers cleanup.

### 88. Subnet gateway interprets `hop_ttl == 0` as unlimited rather than expired (verified)
**File:** `adapter/net/subnet/gateway.rs:112` (header constructor at `adapter/net/protocol.rs:206-224`; AAD definition at `:319-344`)

```rust
if hop_ttl > 0 && hop_count >= hop_ttl {
    // drop
}
```

`NetHeader::new` defaults `hop_ttl` to 0 (`protocol.rs:212`), so any packet with default headers forwards regardless of `hop_count`. `aad()` at `protocol.rs:319-344` includes `hop_ttl` (`:326`) but explicitly excludes `hop_count` (`:327`: "aad[6] = 0: hop_count excluded from AAD"). Any sender that uses default `NetHeader::new` and crosses a gateway will have `hop_ttl == 0`, which short-circuits the TTL check entirely — packets forward forever. Routing-layer TTL still bounds end-to-end loops for routed packets, but pure subnet-gateway forwarding paths (no routing header) lack any cap.

**Verification (2026-04-30):** confirmed by reading `gateway.rs:112` and `NetHeader` constructor + `aad()` in `protocol.rs`. Fix: treat `hop_ttl == 0` as expired (drop), set a sensible non-zero default in `NetHeader::new`, or both.

### 89. Router `stream_stats` keyed by AEAD-unverified bytes — DashMap flood (verified)
**File:** `adapter/net/router.rs:475-481` (record_in at `route.rs:567-571`, DashMap declaration at `route.rs:391, 406`)

```rust
let stream_id = if data.len() >= ROUTING_HEADER_SIZE + HEADER_SIZE {
    let net_header = &data[ROUTING_HEADER_SIZE..ROUTING_HEADER_SIZE + HEADER_SIZE];
    u64::from_le_bytes(net_header[32..40].try_into().unwrap_or([0; 8]))
} else {
    0
};
```

`route_packet` parses `RoutingHeader` only (no AEAD verify possible — keys are per-session, router is per-node), extracts `stream_id` from raw bytes at `router.rs:481`, then calls `self.routing_table.record_in(stream_id, len)` at `:487`. `record_in` (`route.rs:567-571`) does `self.stream_stats.entry(stream_id).or_default()` — unbounded insert into a `DashMap<u64, SchedulerStreamStats>` with no size cap; `cleanup_idle_streams` (`route.rs:624-635`) is interval-driven. An attacker can pick 2^64 random `stream_id` values and exhaust router memory between cleanup ticks.

**Verification (2026-04-30):** confirmed via direct trace through `route_packet` → `record_in` → DashMap insert; no AEAD gate exists at the router layer because the router is upstream of session keys. Fix: gate `stream_stats` insertion on a successful AEAD verify (will require restructuring), cap the DashMap with an LRU/size-bounded policy, or restrict accounting to known/registered `stream_id`s.

### 90. `BatchedTransport::recv_batch` indexes empty `recv_buffers` constructed via `new_send_only` (verified)
**File:** `adapter/net/linux.rs:215, 285` (constructor at `:48-50`, `:52-60`)

`BatchedTransport::new_send_only` (`:48-50`) intentionally skips the `recv_buffers` allocation — `new_inner(_, false)` initializes `recv_buffers = Vec::new()` (`:59`). Both `recv_batch` (`:207-234`) and `recv_batch_blocking` then unconditionally do `self.recv_buffers[i].resize(MAX_PACKET_SIZE, 0)` for `i in 0..count` (`:215`), which panics with index-out-of-bounds for any send-only-constructed instance. The doc comment on `new_send_only` (`:44-47`) only states the contract verbally; there is no runtime guard.

**Verification (2026-04-30):** confirmed by reading `linux.rs:40-60, 200-235`. Fix: either return an `InvalidOperation` error from the recv methods when `recv_buffers` is empty, or split the type so send-only construction returns a different type that lacks the recv methods at compile time.

### 91. `analyze_subnet_correlation` returns a non-deterministic subnet on tied depth (verified)
**File:** `adapter/net/contested/correlation.rs:249-257` (HashMap declaration at `:221`; downstream consumer at `:262-265`)

```rust
for (&subnet, &count) in &subnet_counts {
    if count >= threshold && subnet.depth() >= best_depth {
        best_subnet = Some(subnet);
        best_depth = subnet.depth();
    }
}
```

`subnet_counts` is declared as `let mut subnet_counts: HashMap<SubnetId, usize> = HashMap::new();` at `:221` — std `HashMap` with randomized iteration order. The predicate uses `>=` on depth (`:253`), so on tied `best_depth` the last subnet visited in iteration order wins. With `subnet_counts` populated by walking parent chains (`:228-237`), ties at the same depth are absolutely possible (e.g. two sibling subnets each with the same number of failures both rolled up to a shared parent depth). The downstream `FailureCause::SubnetFailure { subnet, ... }` at `:262-265` carries that subnet to the recovery path — non-determinism propagates.

**Verification (2026-04-30):** confirmed both the iteration order risk and the propagation. Fix: switch to a deterministic tiebreaker (e.g. lowest subnet id at equal depth) by iterating a sorted view, or use strictly `>` with a documented "first seen wins" semantic over an ordered iterator (e.g. `BTreeMap`).

### 96. Redex `read_timestamps` accepts ts file with stale per-position semantics after torn-tail recovery — **REFUTED**
**File:** `adapter/net/redex/disk.rs:298-310` (rewrite branch at `:384-399`)

**Original claim:** `read_timestamps` accepts ts as long as length matches; after torn-tail truncation of idx without rewriting ts, surviving entries' timestamps are misaligned.

**Verification (2026-04-30):** the claim is wrong about the recovery flow. `read_timestamps` is called at `disk.rs:305`, which is *after* the torn-tail walk at `:247-269` has already truncated the on-disk idx and the in-memory `index` vec. Crucially, ts is append-only and the surviving N idx entries are the **first N** of the original idx — torn-tail truncation only chops the tail, so per-position alignment between idx and ts is preserved by construction. The first N timestamps in ts correspond exactly to the surviving entries. The rewrite branch at `:384-399` (which only fires for mid-file checksum drops) handles the only case where positions could shift; pure torn-tail does not produce that case. Age-based retention reads timestamps that ARE correctly aligned. Removed from outstanding tally.

### 105. `recently_closed` quarantine map grows unbounded under stream open/close churn
**File:** `adapter/net/session.rs:67-68, 464-487`

`close_stream` and `evict_idle_streams` insert into `self.recently_closed`. The only GC site is `is_grant_quarantined` (line 475-487) — it removes a stream's entry when the entry is queried *and* its window has elapsed. `is_grant_quarantined` is only called from `mesh.rs:2770`, when an inbound `StreamWindow` grant arrives for that exact `stream_id`. **Failure scenario:** a long-lived peer that opens/closes many distinct stream IDs (e.g., one short-lived stream per RPC) and doesn't receive a grant for each closed stream after `GRANT_QUARANTINE_WINDOW` (2s) accumulates a `recently_closed` entry per closed stream forever. With N streams/sec churn, after T seconds the map holds ~N*T entries, bounded only by total distinct stream IDs ever closed. Add a periodic sweep that drops entries past `GRANT_QUARANTINE_WINDOW`, or piggyback on `evict_idle_streams`.

### 106. `NetSession` constructs `tx_cipher`, `packet_pool`, and `thread_local_pool` with the same TX key but **independent** counters
**File:** `adapter/net/session.rs:96-101`

```rust
let tx_cipher = PacketCipher::new(&keys.tx_key, keys.session_id);                       // counter A
let packet_pool = super::pool::shared_pool(pool_size, &keys.tx_key, keys.session_id);   // counter B
let thread_local_pool = super::pool::shared_local_pool(pool_size, &keys.tx_key, ...);   // counter C
```

All three share the same key. `PacketPool` and `ThreadLocalPool` each correctly serialize their internal counters within the pool (regression tests at `pool.rs:952, 992` confirm) — but the three constructions have independent counters that all start at 0. Currently dormant: `tx_cipher` (line 153) and `packet_pool` (line 562) getters are exposed, but no caller in the tree uses them — only `thread_local_pool` is wired through. The moment any caller obtains `session.packet_pool()` or `session.tx_cipher()` and encrypts a packet, ChaCha20-Poly1305 nonce reuse against the corresponding counter slot in `thread_local_pool` is guaranteed (same key + same nonce), giving an attacker XOR access to the plaintext. The pool-internal regression tests prevent this **within** a pool; the construction here defeats it across pools. Either share one `Arc<AtomicU64>` counter across all three, or remove the unused getters.

### 107. `ClassifyFsm::classify` cannot recognize `Open` for nodes bound to wildcard addresses
**File:** `adapter/net/traversal/classify.rs:280-294`

The "Open" predicate is `reflex.port() == bind_addr.port() && reflex.ip() == bind_addr.ip()` (line 291). When the daemon binds to `0.0.0.0:9001` (the common default), peer reflex observations like `192.0.2.1:9001` will never compare equal — even though the ports match and the node is, in fact, directly reachable. The FSM classifies as `Cone` (or `Symmetric`) and `pair_action` triggers an unnecessary `SinglePunch`. Capability tags advertise `nat:cone` instead of `nat:open`, biasing peer-side decisions. The docstring at line 277 acknowledges callers should pre-resolve `bind_addr` to an interface address but provides no runtime guard — the API silently mis-classifies. Either resolve wildcard binds against the node's interface table before classification, or treat IP comparison as a wildcard match when `bind_addr.ip().is_unspecified()`.

### 108. `NodeInfo::update_from_pingwave` and `from_pingwave` use raw `pw.hop_count + 1` (panics in debug, wraps in release)
**File:** `adapter/net/behavior/proximity.rs:285, 301-303`

`forward()` was hardened to `saturating_add` for the same field (regression test at line 1226), but the ingest sites that compute `pw.hop_count + 1` to set/compare `self.hops` were not. In debug builds this panics on `hop_count == 255`; in release it wraps to 0, taking the `pw.hop_count + 1 < self.hops` branch and recording the node as **0 hops away** (i.e., "self"). `from_bytes` accepts any u8 hop_count from the wire, so a single bit-flip in transit or a malicious peer can trick `find_best`/`routing_score` into selecting a 255-hop-away node as the lowest-cost route. Apply `saturating_add(1)` (or `checked_add(1)?` with reject-on-overflow) at both sites.

### 109. `SchemaType::validate` recurses without bound on attacker-controlled schema → stack overflow DoS
**File:** `adapter/net/behavior/api.rs:297, 466-471, 498-503, 528-537`

`SchemaType` is `#[derive(Deserialize)]` (line 62) and contains recursive variants (`Array { items: Box<SchemaType> }`, `Object { properties: HashMap<_, SchemaType> }`, `AnyOf { schemas: Vec<SchemaType> }`). `validate` calls itself recursively on every branch with no depth cap. An attacker who can ship a schema (API announcements broadcast over the mesh, or any caller that parses untrusted JSON into `SchemaType`) submits a deeply nested `AnyOf`/`Array` chain and crashes the validator — and the whole process — via stack overflow when a request gets validated against it. Add a recursion-depth ceiling (or convert to iterative) on both deserialize and validate paths.

### 110. Capability index admits expired announcements with a fresh local TTL
**File:** `adapter/net/mesh.rs:3682-3877` (handler) + `adapter/net/behavior/capability.rs:1525-1565` (index) + `1953-1971` (gc)

`handle_capability_announcement` never calls `ann.is_expired()` before forwarding + indexing. Even though the announcement carries `timestamp_ns` (in the signed envelope), `CapabilityIndex::index()` discards it and stores `indexed_at: Instant::now()` plus `ttl: Duration::from_secs(ann.ttl_secs as u64)` — meaning the entry is alive for `ttl_secs` *from local indexing time*, not from origin time. Combined with dedup keyed only on `(node_id, version)`, an attacker who saved a 9-month-old signed announcement (still cryptographically valid) and replays it to a peer that never saw that exact `(node_id, version)` (a fresh node, or a peer where the entry was GC'd while the source was offline) gets the stale capabilities reinstated with a fresh 5-minute lease. Useful for re-introducing a model/tag/scope an operator deliberately removed, or an old `reflex_addr` to misdirect NAT traversal. Reject announcements where `is_expired()`, and use `min(now+ttl, origin_timestamp+ttl_secs)` for the index lifetime.

### 111. `MigrationFailed` dispatch doesn't clean up the snapshot reassembler — partial-snapshot leak
**File:** `adapter/net/subprotocol/migration_handler.rs:629-651`

When a `MigrationFailed` arrives, the dispatcher aborts source/target/orchestrator state but never calls `self.reassemblers.remove(&daemon_origin)`. Any partially-received snapshot chunks for that daemon stay pinned in the `DashMap` indefinitely. Compare to `fail_migration_with_reason` (line 1037) which correctly removes the reassembler — only the inbound-failure path forgets. **Failure scenario:** source begins a 400-chunk snapshot to target. After 200 chunks arrive, source aborts (e.g., `NotReady` retry exhausted) and broadcasts `MigrationFailed`. Target's dispatcher cleans `target_handler` + `orchestrator` but the 200 chunks (~1.4 MB per migration with default 7 KB chunks) stay in `reassemblers`. Future migrations with a *higher* `seq_through` for the same origin evict via the reassembler's own logic, but if the same origin never migrates again the memory is held until process exit. With many ephemeral daemons this is an unbounded leak.

### 112. Remote-orchestrator `on_cleanup_complete` never resolves `SuperpositionState`
**File:** `adapter/net/compute/orchestrator.rs:1152-1165`

`on_cleanup_complete` advances the migration phase Cutover→Complete but does NOT call `record.superposition.advance(MigrationPhase::Complete)` or `record.superposition.resolve()`. Compare to `on_cutover_acknowledged` (line 1123-1139) which does both. When the orchestrator runs on a third party node, `on_cutover_acknowledged` is a no-op (the comment at line 1144-1148 confirms), so the remote path is the only authoritative one — and it leaves `SuperpositionState` stuck mid-collapse. **Failure scenario:** cross-node migration via a remote orchestrator. After `CleanupComplete`, the migration is functionally finished, but `superposition_phase()` continues to report a pre-resolution phase forever (until `on_activate_ack` removes the record entirely). Operator dashboards / readiness probes / SDK handles keyed on superposition state never observe resolution. Mirror `on_cutover_acknowledged` and call `advance` + `resolve`.

### 113. `NatPmpMapper::install` with `ttl == Duration::ZERO` silently REMOVES the mapping instead of installing it
**File:** `adapter/net/traversal/portmap/natpmp.rs:462`

```rust
let lifetime = ttl.as_secs().min(u32::MAX as u64) as u32;
```

If `ttl` is zero, `lifetime = 0`, which by RFC 6886 §3.3 is the "remove this mapping" signal — the same wire format `remove()` sends. The gateway acks (mapping removed); `install` returns `Ok(PortMapping { ttl: ZERO, .. })` which the caller treats as freshly installed. Compounds with the renewal loop (`mod.rs:611-647`): if a misbehaving gateway grants `lifetime=0` once, `mapping.ttl = ZERO`, and the next renewal calls `self.install(mapping.internal_port, mapping.ttl)` (`natpmp.rs:520`) which sends another remove. Renewals keep "succeeding" with lifetime=0 while the router has nothing mapped. **Failure scenario:** gateway responds to install with `granted_lifetime=0` (some BSD / legacy IGD setups do this on policy refusal). Mesh records `ttl=ZERO`. On the next renewal tick, the sequencer self-removes. Peers can't reach the node. Reject `ttl == ZERO` before sending the install request, or renew with a sane minimum (e.g. `max(ttl, 60s)`).

### 114. `assess_continuity` reports `Continuous` for pruned logs missing genesis (no snapshot detection)
**File:** `adapter/net/continuity/chain.rs:174-219`

The function walks `log.range(0, u64::MAX)` and only validates consecutive pairs. After a `prune_through(N)`, the log only contains events with `seq > N`. The function never checks that the first event has `sequence == 0` or that the gap from `0..N` is accounted for by a snapshot — it just validates pair-wise linkage and returns `Continuous { head_seq, .. }`. A log containing only events 100..200 produces `Continuous` with `head_seq=200` even though events 0..99 may be entirely missing. **Failure scenario:** a node restarts with a corrupt/missing snapshot but a partial log carrying events 100..200. `assess_continuity` reports the chain is intact; the node propagates that belief, and downstream peers see "everything's fine" when in fact the entity has lost its first 100 events with no recoverable lineage. Take an optional snapshot reference and require `first_event.sequence == 0` OR `first_event.sequence == snapshot.seq + 1`; otherwise return `Unverifiable { gap_start: 0 }`.

### 115. `MemoriesFold` `DISPATCH_MEMORY_STORED` resets `pinned` and `created_ns` on re-store
**File:** `adapter/net/cortex/memories/fold.rs:46-61`

When a `STORED` event lands for an existing memory id, the fold unconditionally constructs a new `Memory { ..., pinned: false, created_ns: p.now_ns, ... }`, replacing the existing entry. **Failure scenario:** user pins memory id=42, then later calls `memories.store(42, "updated content", ...)`. The pin flag silently resets to false; queries with `where_pinned(true)` no longer return id=42. The original `created_ns` is also overwritten, breaking any downstream "created_after" filter relying on it. Operator has no observable signal that the pin was dropped. Either preserve `pinned` and `created_ns` on re-store of an existing id (treating STORED as content-update, not full-replace), or split STORED into separate "create" and "update-content" verbs.

### 116. `NetProxy.hop_stats` DashMap grows without bound and is not cleared by `remove_route`
**File:** `adapter/net/proxy.rs:192, 234-236, 385-394`

`record_hop_forward` and `record_hop_drop` call `hop_stats.entry(dest_id).or_default()` for every routed packet. There is no eviction logic anywhere — `remove_route(dest_id)` deletes the next_hop entry but leaves `hop_stats[dest_id]` in place. A peer churning through many destinations (or sending zero-route packets that hit `record_hop_drop`) grows the map indefinitely. Memory growth is proportional to total-distinct-dest-ids-ever-seen, not active dest count. Wire `remove_route` to also drop `hop_stats[dest_id]`, or apply a periodic LRU sweep.

### 117. `ReroutePolicy::on_recovery` cannot match saved routes after peer NAT rebind, leaking `saved_routes`
**File:** `adapter/net/reroute.rs:222-243`

`on_recovery` resolves `recovered_addr = peer_addrs.get(&recovered_node_id)`, then filters `saved_routes` by `entry.next_hop == recovered_addr` (line 231). When a peer reconnects from a different `SocketAddr` (NAT rebind, reconnect on different port, mobile network change), `peer_addrs` reflects the new address but `saved_routes` was keyed on the old `next_hop`. The filter returns empty, no routes are restored, and the `saved_routes` entry persists indefinitely (DashMap entries are only dropped on successful match in line 242). **Adverse outcome:** routes stay pinned to alternate paths after the peer has actually recovered, causing avoidable extra-hop traffic; `saved_routes` grows without bound across mobile / NAT-changing peers. Index `saved_routes` by `node_id` rather than `next_hop`, and rewrite it on `peer_addrs` updates.

### 121. `PermissionToken::issue` and `delegate` panic on a public-only signer keypair across the FFI boundary
**File:** `adapter/net/identity/token.rs:172` (`issue`); `adapter/net/identity/token.rs:313` (`delegate`)

`issue` calls `issuer_keypair.sign(&payload)` and `delegate` calls `signer.sign(&payload)` directly — neither uses `try_sign`. `EntityKeypair::sign` panics with `"public-only keypair"` when the signing half is absent (`identity/entity.rs:263-266`). The same module exposes `EntityKeypair::public_only` (`entity.rs:215`) and `zeroize` (`entity.rs:319`); the migration-source path explicitly invokes `zeroize` after `ActivateAck` (`entity.rs:309-318`) — so a daemon that finished migrating its identity holds exactly such a keypair.

The FFI bindings `net_identity_issue_token` (`ffi/mesh.rs:1938`) and `net_identity_delegate_token` (`ffi/mesh.rs:2149`) read a user-supplied handle's keypair and feed it to `issue` / `delegate`. After a daemon migrates and the source zeroizes its key, any subsequent FFI caller asking that source to mint or delegate a token panics inside Rust and unwinds across `extern "C"` — undefined behaviour, identical in shape to #58 / #61. Fix: switch both functions to `try_sign`, surface a `TokenError::ReadOnly` (or `NotAuthorized`) variant, and return `Result` from `issue` (signature change).

### 122. `SnapshotStore::store` allows older snapshots to overwrite newer ones (no monotonicity check)
**File:** `adapter/net/state/snapshot.rs:451-454`

```rust
pub fn store(&self, snapshot: StateSnapshot) {
    let key = *snapshot.entity_id.as_bytes();
    self.snapshots.insert(key, snapshot);
}
```

There is no comparison against the existing entry's `through_seq` or `created_at`. A delayed/reordered snapshot delivery (or migration message arriving from a stale node) installs a snapshot at sequence N, replacing one at sequence N+M already present — subsequent restore reads the older state. Two threads concurrently storing snapshots race; whichever DashMap insert lands last wins regardless of which is fresher. There is no AEAD on the storage path, so an attacker who replays a captured archived snapshot rewinds state. Fix: gate the insert on `existing.through_seq < snapshot.through_seq` (e.g. via `DashMap::entry`/`alter`) or return a "stale snapshot ignored" signal.

### 123. `MetadataStore::upsert` non-atomic 5-step update produces permanent index drift under concurrent updates
**File:** `adapter/net/behavior/metadata.rs:826-849`

`upsert` is a 5-step sequence with no overarching lock: (1) capacity check, (2) `nodes.get(&node_id)` to read old, (3) `remove_from_indexes(&old)`, (4) `add_to_indexes(&metadata)`, (5) `nodes.insert`. Two threads upserting the same node concurrently both read the same `old` at step 2, both remove its index entries at step 3 (second is a no-op), and both add to indexes at step 4 — in two *different* index buckets if the metadata differs. Whichever `nodes.insert` lands second wins, but the loser's index entries are never removed.

Concrete failure: thread A sets node X to `Online` with tag `t1`; thread B sets the same node to `Degraded` with tag `t2`. Final `nodes` has the second write's metadata, but `by_status[Online]` *and* `by_status[Degraded]` both contain X, and both `by_tag[t1]` and `by_tag[t2]` retain it. Stats over-count, queries return X under wrong filters, and the drift persists forever — no rebuild path exists. Fix: serialize the entire upsert via `DashMap::entry` on `nodes` (so step 2-5 hold the per-shard write lock), or version-check + retry, or use a coarse per-node mutex.

### 124. `NodeMetadata` deserialize is unbounded — peer-supplied DoS via giant tags / custom map
**File:** `adapter/net/behavior/metadata.rs:382-411` (and `add_to_indexes` at `:1062-1083`)

`NodeMetadata` derives `Deserialize` over `HashMap<String, String> custom`, `HashSet<String> tags`, `HashSet<String> roles`, `Vec<NodeId> preferred_peers`, `HashMap<String, u8> hop_distances`, `Vec<IpAddr> public_addresses`, plus several `String`s. None of these have size limits in the deserialize path. A malicious peer ships `NodeMetadata` with millions of unique tags or a multi-megabyte custom map; `serde_json::from_slice` allocates it; `MetadataStore::upsert` then stores it (capacity is on count, not bytes). `add_to_indexes` faithfully inserts each tag into `by_tag.entry(tag.clone()).or_default().insert(node_id)` — turning one peer's announcement into N DashMap entries with no upper bound.

Combined with `with_capacity` defaulting to `None`, an attacker registers a single node with 1M unique tags and creates 1M `by_tag` entries. Fix: validate after deserialize — cap name/description length, tag/role counts, custom-map size, preferred_peers length — and reject in `upsert` (or before).

### 125. `DiffEngine::apply` declares `VersionMismatch` but never checks the version, accepting stale diffs against fresher state
**File:** `adapter/net/behavior/diff.rs:518-530` (variant defined at `:226-233`)

```rust
pub fn apply(base: &CapabilitySet, diff: &CapabilityDiff, strict: bool) -> Result<CapabilitySet, DiffError> {
    let mut result = base.clone();
    for op in &diff.ops {
        Self::apply_op(&mut result, op, strict)?;
    }
    Ok(result)
}
```

There is no `base.version == diff.base_version` check, despite `DiffError::VersionMismatch` being a documented error variant. A receiver at v5 (state with X, Y, Z added at v3-v5) accepts an old diff `base_version=2 → new_version=3` containing `RemoveModel("Y")`. The receiver removes Y and silently bumps its tracked version to v3 — diverging from peers that already applied v3-v5. Subsequent diffs against v3 are then accepted, snowballing the divergence.

Fix: require the caller to thread the live version into `apply`, or store it on `CapabilitySet` and check at the top of `apply`. At minimum, document the contract loudly so callers don't trust the empty-promise variant.

### 126. `Tasks/MemoriesAdapter::ingest_typed` advances `app_seq` BEFORE the inner ingest succeeds — phantom seq on failure
**File:** `adapter/net/cortex/tasks/adapter.rs:312-327`, `adapter/net/cortex/memories/adapter.rs:305-320`

Both adapters call `self.app_seq.fetch_add(1, ...)` to allocate `seq_or_ts`, then call `inner.ingest(payload, meta)`. If `inner.ingest` fails (closed adapter, RedEX append error, fold error under `Stop` policy), the local counter is permanently advanced past a `seq_or_ts` that was never written to the log. After restore, the snapshot's persisted `app_seq` reflects the higher counter — a future ingest picks up at the higher value, leaving a permanent gap. A second adapter sharing the same `origin_hash` (a documented configuration in the cortex layer) and recovering via on-disk scan rather than snapshot disagrees on `app_seq`, producing duplicate `seq_or_ts` collisions when both come back online. Fix: only commit `app_seq.fetch_add` after `inner.ingest` returns Ok — load + CAS retry, or roll back on Err.

### 127. `IdentityEnvelope::open` accepts any attacker-chosen `signer_pub`; AEAD AAD is empty
**File:** `adapter/net/identity/envelope.rs:261-334` (specifically lines 269-276, 296-299)

The envelope-open primitive verifies that `signer_pub`'s signature over `target_static_pub || chain_link` is valid (line 270-276) and that the decrypted seed reconstructs to the same `signer_pub` (line 329). Crucially it does NOT take an `expected_signer_pub` (or `expected_origin_hash`) parameter from the caller — any well-formed envelope from any keypair passes. The AEAD payload uses `aad: &[]` (line 298), so the chain_link is bound only to the signature, not the ciphertext.

Failure scenario: a malicious peer in the migration-source's path injects a substituted envelope built from the attacker's keypair, with `target_static_pub` set correctly to the actual target. The target `open`s it, reconstructs the attacker's keypair, and (if the migration handler doesn't cross-check) registers it as the migrated daemon's identity — then signs subsequent capability announcements / tokens under the attacker's identity. The doc-comment at lines 105-106 acknowledges "the primitive returns the keypair and the caller cross-checks" — but the primitive itself is unsafe by default. Fix: add `expected_signer_pub: &EntityId` (or `expected_origin_hash: u32`) as a parameter and reject early; pass `chain_link.to_bytes()` as AAD on encrypt/decrypt so a tampered link breaks both the signature *and* AEAD.

### 128. `StateSnapshot::to_bytes` panics in release on >4 GiB state or bindings
**File:** `adapter/net/state/snapshot.rs:227, 232`

```rust
let state_len = u32::try_from(self.state.len()).expect("state snapshot exceeds 4 GiB");
...
let bindings_len = u32::try_from(self.bindings_bytes.len())
    .expect("bindings_bytes exceeds 4 GiB — this is almost certainly a bug");
```

`state` is opaque daemon state passed in from any caller (compute orchestrator, FFI clients), and `bindings_bytes` is opaque externally-controlled migration metadata. An adversarial or buggy producer with >4 GiB content makes serialization panic — and `to_bytes` is on the migration / snapshot-send path, where a panic crashes the dispatch task without releasing locks. Compare against `write_causal_events` in `causal.rs`, which gracefully *skips* oversized events and returns `events_skipped`. Fix: change `to_bytes` to return `Result<Vec<u8>, SnapshotError>` and bail with an error variant instead of `expect`-panicking.

### 129. `EntityLog::prune_through` desyncs `snapshot_seq` from `base_link.sequence` on an already-empty log
**File:** `adapter/net/state/log.rs:163-194`

When `prune_through(seq)` is called on an empty log, `events.iter().rev().find(...)` returns `None`. The `events.is_empty()` branch fires but the inner `if let Some(...)` does nothing — yet `snapshot_seq` is unconditionally bumped to `seq` if `seq > self.snapshot_seq`. Result: `snapshot_seq` advances to an arbitrary `seq` while `base_link.sequence` stays at its previous value (e.g., 0 for a fresh genesis log).

Failure scenario: caller restored a fresh log via `from_snapshot(_, snapshot_seq=0, head_link=genesis, ..)`, then took an externally-coordinated snapshot at sequence 1000 and called `prune_through(1000)`. `head_seq()` reports 0 (base_link.sequence), but `snapshot_seq()` returns 1000. Code that prefers `head_seq().max(snapshot_seq())` to decide where the next event must start gets contradictory answers; the next `append` will only accept sequence == 1, not 1001 — silently dropping legitimate events from peers that observed the actual snapshot point. Fix: clamp `snapshot_seq = snapshot_seq.max(seq).min(head_seq())`, or only bump when `last_pruned.is_some()`.

### 130. `HorizonEncoder::might_contain` saturates after ~8 origins, collapsing causal-concurrency detection to "always concurrent"
**File:** `adapter/net/state/horizon.rs:91-141`

The bloom filter is 16 bits with two 4-bit hash positions per origin (`h & 0xF`, `(h >> 4) & 0xF`). Each insert sets two of 16 bits — by the birthday bound, after only ~6-8 inserted origins the bloom is fully saturated and `might_contain` returns true for *every* `origin_hash`. At that point `potentially_concurrent` always returns false, meaning the system claims "every event has observed every other event" — defeating causal concurrency detection on any non-tiny mesh. A node observing 8+ peer entities encodes a saturated bloom into every outgoing `CausalLink::horizon_encoded`. Receivers using `potentially_concurrent` to schedule conflict resolution / merging see all events as causally-ordered and skip the resolution they should be running. Fix: widen the bloom space (`horizon_encoded` is u32 but only the high 16 bits are bloom — the low 16 carry seq), or document a hard cardinality ceiling of ~6 origins, or use a counting/sliding sketch sized for realistic entity counts.

### 131. Subprotocol manifest exposes forwarding-only entries as if they were locally handled
**File:** `adapter/net/subprotocol/negotiation.rs:39-50, 55-70`

`SubprotocolManifest::from_registry` calls `registry.list()` which returns *every* descriptor regardless of `handler_present`. The 6-byte wire format (id, version, min_compatible) has no flag for `handler_present`, and `to_bytes` forces every entry to deserialize back as `handler_present: true` (line 65). After `negotiate()` produces the `compatible` set, the receiving peer believes the sender has a local handler for every advertised id — including ones registered via `.forwarding_only()`.

Failure scenario: Node B registers subprotocol 0x1000 forwarding-only because it lacks the daemon but participates in routing it. B's manifest still advertises 0x1000. Node A negotiates → marks 0x1000 compatible → schedules a 0x1000-bound RPC to B → B has no handler → silent drop. The `capability_tags()` pathway (negotiation.rs:119) correctly filters forwarding-only entries; this direct-manifest path does not. Two parallel discovery channels disagree. Fix: filter `from_registry` to only emit `handler_present` entries, mirroring `capability_tags`, OR extend the wire format with a flag byte (bumping `MANIFEST_ENTRY_SIZE` to 7).

### 132. `read_manifest_entry` accepts `min_compatible > version`, enabling phantom-incompatibility DoS
**File:** `adapter/net/subprotocol/descriptor.rs:133-143`, `subprotocol/negotiation.rs:99-110`

Neither `read_manifest_entry` nor `SubprotocolManifest::from_bytes` validates the wire-format invariant that `min_compatible <= version`. A peer that advertises `version = 1.0, min_compatible = 255.255` passes parsing. In `negotiate()`, every honest peer's `local_entry.version.satisfies(remote_entry.min_compatible)` returns false, so the subprotocol is added to `incompatible` rather than `compatible`. The attacker thereby unilaterally evicts any subprotocol from negotiation between the victim and its peers — without ever actually being a peer that handles it.

`with_min_compatible` in `SubprotocolDescriptor::new` is also `pub`, so any local builder can produce malformed descriptors that violate `is_compatible_with`'s contract. Fix: in both `read_manifest_entry` and `with_min_compatible`, reject when `min_compatible > version`.

### 133. `NetDb::close` early-returns on first adapter close failure, leaking subsequent fold tasks
**File:** `adapter/net/netdb/db.rs:95-103`

```rust
pub fn close(&self) -> Result<(), NetDbError> {
    if let Some(t) = &self.tasks {
        t.close()?;          // ?-short-circuits
    }
    if let Some(m) = &self.memories {
        m.close()?;
    }
    Ok(())
}
```

When `tasks.close()` errors, `?` short-circuits and `memories.close()` is never invoked. The memories adapter retains its fold task and keeps consuming events, even though the caller has been told `close` failed and likely treats the whole NetDb as torn down. Combined with the "fold task outlives builder" hazard the build path explicitly guards against (lines 175-187), this leaks fold tasks per-NetDb-close-failure. Fix: attempt both closes regardless, then surface the first error.

### 134. `CortexAdapter::open` accepts arbitrary `initial_state` with `FromSeq(n>0)` / `LiveOnly`, falsely advancing `wait_for_seq`
**File:** `adapter/net/cortex/adapter.rs:181-263` (watermark init at lines 207-211)

`open` takes `initial_state: State` and `start_seq` is derived from `adapter_config.start`. With `StartPosition::FromSeq(n)` (n > 0) or `LiveOnly`, the adapter starts folding at `n` and never reads events `[0, n-1]`. `initial_watermark` is set to `start_seq - 1`, so `wait_for_seq(k)` for any `k <= start_seq-1` returns immediately — the adapter claims those seqs are "applied" while state has never seen them. A consumer using `FromSeq(n)` to skip an old prefix gets a state that pretends those events were applied, producing silently wrong query results until live events overwrite the keys.

Doc on `LiveOnly` says "use when `State` is rehydrated from an external snapshot", but `open` doesn't enforce that — only `open_from_snapshot` does. Fix: restrict `FromSeq` / `LiveOnly` to `open_from_snapshot` (drop them from raw `open`), or require a snapshot-source proof on `open`.

### 135. `EventMeta::compute_checksum` truncates xxh3 64→32 bits, defeating the documented tamper-detection property
**File:** `adapter/net/cortex/meta.rs:111-114` (used at `cortex/tasks/fold.rs:36-43`, `cortex/memories/fold.rs:37-43`)

`compute_checksum` does `xxh3_64(tail) as u32`, throwing away 32 bits. The fold doctstrings claim this catches "tampered on-disk files". A 32-bit checksum has ~1-in-2^16 birthday collision probability across the file's lifetime; even worse, it's an unkeyed hash, so an attacker who can write to the on-disk redex file can compute the matching checksum trivially. As an accidental-corruption detector for stray bit flips, 32 bits is marginal; as a tamper detector (per the docstring), it's nearly meaningless. Fix: either downgrade the docstring claim to "corruption detector" or use a keyed MAC stamped at append.

### 136. `ContextStore::create_context` capacity check is non-atomic; sustained load grows the map past the cap
**File:** `adapter/net/behavior/context.rs:822-829, 871-878`

When the store is at capacity, `cleanup_expired` is called inline (synchronous, scans entire `DashMap`). Two threads inserting at exactly capacity will both call `cleanup_expired` in parallel, then both pass the recheck (line 825), and both insert (line 842). Worse, between the recheck and the insert, a third thread can insert. There is no atomic "insert-if-under-cap" — under sustained load the map grows unbounded past `max_traces`. Combined with the W3C `from_traceparent` resetting `hop_count: 1` and `max_hops: None` (line 654), a peer storming with synthetic traces via `continue_context` defeats both the trace-loop guard and the cap. Fix: serialize via `DashMap::entry` with a coordinated counter, or use a single coarse insert-lock when at the cap.

### 137. `Sampler::should_sample` `RateLimited` over-samples by `num_threads-1` per second under contention
**File:** `adapter/net/behavior/context.rs:710-722`

In the `RateLimited` arm: each thread reads `count.load`, compares to `max_per_second`, then `fetch_add`s. The check + fetch_add is not atomic — N concurrent threads can all observe `current < max_per_second` and all increment, producing `max + N - 1` samples in a window. Not catastrophic, but the documented invariant is violated, and a downstream consumer relying on the rate-limit to bound sampler-driven write traffic over the wire (e.g., trace-emit telemetry) can see the rate burst by `num_threads × max_per_second`. Fix: use a `compare_exchange` loop or `fetch_update` to atomically gate on the cap.



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

### 118. `current_timestamp` truncates `as_nanos()` (u128) → u64 silently
`adapter/net/mod.rs:176-181` — practical wrap doesn't happen until ~year 2554, but: (a) on a system whose clock is misconfigured to a far-future date the timestamp wraps to a small number, immediately tripping `is_timed_out` everywhere; (b) `unwrap_or_default()` on `duration_since(UNIX_EPOCH)` returns `Duration::ZERO` when the clock is set **before** epoch, producing identical timestamps until correction. Use checked conversion (`u64::try_from(...).unwrap_or(u64::MAX)`) or move idle-timeout bookkeeping to monotonic `Instant`.

### 119. `RoutingHeader::forward` decrements TTL even when the packet is for a local destination
`adapter/net/router.rs:489-512`, `adapter/net/proxy.rs:268-293` — `route_packet` correctly delivers locally even at TTL=0 (line 490), but for non-local destinations the order is: TTL check → lookup → `forward()` (decrements TTL). The bug: `forward()` returns `false` if TTL is now 0 but the return value is discarded (line 512). The packet is still queued (line 525) and sent. The next hop receives a TTL=0 packet and drops it. Wastes one forward + bandwidth + queue slot whenever a packet reaches its last hop. Check `forward()`'s return value and drop locally if it's false.

### 120. `LocalGraph::on_pingwave` rejects restart-induced sequence regressions, leaving stale node info
`adapter/net/swarm.rs:510-515` — `and_modify` only updates a node's `addr`/`hops`/`last_seq`/`last_seen` if `pw.seq > n.last_seq || hops < n.hops`. When a peer restarts, `next_seq` resets to 1; the local node's `n.last_seq` is still the old high-water-mark (e.g. 10000). Incoming pingwaves with seq=1, 2, ... are dropped from updating, so neither `hops` nor `last_seen` advance. The node enters `is_stale` after 30s and gets removed by cleanup, only to be re-inserted as new — in the gap, capability lookups against the stale entry return outdated capabilities. Accept seq regression when the new value is much smaller than the recorded one (indicating a restart), or fall back to wall-time-based staleness independent of seq monotonicity.

### 138. `CapabilityDiff::from_bytes` swallows all deserialize errors and accepts unbounded ops
`adapter/net/behavior/diff.rs:215-217` — returns `Option<Self>`, dropping error context. There is no input-size cap, no `ops.len()` cap, and `serde_json::from_slice` will faithfully expand a peer-supplied 1M-`SetField` diff. `apply` then iterates all 1M ops (each currently a no-op for `SetField`/`UnsetField`, see #151) — CPU/RAM burned for no useful effect. Cap `data.len()` before parsing, cap `diff.ops.len()` after, and return `Result<Self, DeserializeError>` so callers can distinguish malformed from absent.

### 139. `MetadataStore::clear` races with concurrent `upsert`, leaving nodes without index entries
`adapter/net/behavior/metadata.rs:1035-1043` — `clear()` calls `nodes.clear()` then six other `clear()`s sequentially, with no global lock. A concurrent `upsert` between `nodes.clear()` and `by_status.clear()` reads `nodes.get(&id)` → `None`, skips `remove_from_indexes`, calls `add_to_indexes` (writes to `by_status`/`by_tier`/etc), then `nodes.insert` succeeds. `clear()` then wipes the indexes, leaving a node in `nodes` with NO index entries — invisible to any indexed query (status, continent, tier, tag, role, owner). Only the full-scan branch (`query` line 922) finds it. Fix: drain `nodes` first, then iterate the drained set calling `remove_from_indexes`, then clear the indexes — making intermediate states consistent.

### 140. `ObservedHorizon::observe` uses plain `+= 1` while `merge` uses `saturating_add`, debug-panicking on overflow inconsistently
`adapter/net/state/horizon.rs:28-34, 64-76` — `observe` does `self.logical_time += 1` (debug-panics on overflow); `merge` does `saturating_add(1)`. The comment at lines 71-74 acknowledges the convention. Adversarial high-cardinality observes panic in debug builds while the same overflow saturates in release; merge-driven overflow saturates in both. Make `observe` use `saturating_add(1)` for consistency.

### 141. `Tasks/MemoriesFold` `Stop` policy lets a single corrupt tail wedge fold task forever
`adapter/net/cortex/tasks/fold.rs:46-79`, `adapter/net/cortex/memories/fold.rs:47-87` — under `OnFoldError::Stop` (the default), a postcard decode failure halts the fold task permanently. The 32-bit checksum (#135) catches most disk corruption first, but it's not a strong tamper detector — an attacker who can craft a tail with matching truncated checksum (or the 1-in-2^32 collision case) DoSes a multi-tenant cortex instance via a single bad event. Classify recoverable decode errors (bad postcard) separately from unrecoverable storage errors so the default policy can skip-and-continue without halting; or strengthen the checksum.

### 142. Filter `created_after`/`created_before`/`updated_after`/`updated_before` are strict, dropping events at the cutoff
`adapter/net/cortex/tasks/query.rs:56-74`, `adapter/net/cortex/memories/query.rs:92-110` (docs at `cortex/tasks/filter.rs:24-31`, `cortex/memories/filter.rs:30-37`) — comparators are `>` and `<` exclusive. An event with `created_ns == cutoff` is dropped by both `created_after(cutoff)` and `created_before(cutoff)` filters — falls through holes between paginations using "last sync ns" as cutoff. Worse, two events written in the same nanosecond (achievable on Windows where wall-clock granularity is ~15ms) produce identical timestamps; one of them is elided in any window using either bound. Use inclusive comparators or expose explicit `_inclusive` variants.

### 143. `*Watcher::stream` `skip_while` against captured initial deadlocks subscribers under oscillating predicate
`adapter/net/cortex/tasks/watch.rs:154-177`, `adapter/net/cortex/memories/watch.rs:177-200` — the watcher is fed from a `tokio::sync::watch` (single-slot). On a sequence (A → B → A) collapsed by watch into final A, `skip_while` (sticky: only stops skipping once it sees a non-equal item) silently skips the surviving A, then no further state changes ever produce a stream item, and the consumer is permanently starved of legitimate state changes. Use a one-shot dedup (e.g. `skip(1)`) rather than a sticky-data equality check.

### 144. `CortexAdapter::changes` silently drops `BroadcastStream::Lagged` errors, hiding subscriber data-loss
`adapter/net/cortex/adapter.rs:154-157` — `filter_map(|r| async move { r.ok() })` discards `Lagged(n)` signals from `BroadcastStream`. Watchers downstream see fewer-than-expected seqs without any signal. The eventual emission still reflects latest state (so the visible symptom is a delay, not corruption), but a watcher that wants to surface "you missed N changes" for telemetry / back-pressure cannot retrieve the count. Expose a separate `changes_with_lag()` or convert lag errors to a re-emit-latest signal.

### 145. `MetadataStore::find_nearby` / `find_best_for_routing` sort comparators are non-deterministic on NaN
`adapter/net/behavior/metadata.rs:964, 986` — `a.1.partial_cmp(&b.1).unwrap_or(Equal)` on NaN produces a non-total order; `sort_by` then permutes arbitrarily and `truncate(limit)` drops random items. `LocationInfo::distance_to` (lines 124-139) computes `(...).asin()`; for *near*-antipodal points FP rounding can push `a` to slightly > 1.0, producing NaN. Filter NaN distances out before sorting, or use `total_cmp` on a sentinel-replaced score.

### 146. `TokenCache` slot map is unbounded; signed-token flooding fills memory linearly
`adapter/net/identity/token.rs:405-457` — `DashMap<([u8;32], u16), Vec<PermissionToken>>` has no size cap. Distinct `(subject, channel_hash)` tuples create distinct slots; within a slot, distinct scope bitfields stack. `evict_expired` only reclaims past-deadline entries — a long-TTL flood survives. Any caller that processes peer-supplied tokens through `insert` (or any service that issues tokens at attacker request) can grow the cache linearly with `(subject × channel × scope)` cardinality. Add an LRU/size cap or per-issuer rate limit.

### 147. Mesh stream-window dispatch consumes only the first event of a batched grant packet
`adapter/net/mesh.rs:2925-2955` interacting with `subprotocol/stream_window.rs:69-84` — the handler does `events.into_iter().next()`. A peer that batches grants for several streams into one event-frame packet (the codec supports multiple events per frame, and `StreamWindow::decode` is fixed at 16 bytes per grant) sees only the first stream credited; the rest stall until the sender retransmits. `apply_authoritative_grant` is monotonic so retransmits eventually catch up — efficiency loss, not data loss. Either iterate the full event vector, or document that grant frames must contain exactly one event and reject violators.

### 148. `Tasks/MemoriesAdapter::open_from_snapshot` redundantly re-reads events the inner fold task already folded
`adapter/net/cortex/tasks/adapter.rs:286-301`, `adapter/net/cortex/memories/adapter.rs:281-296` — after `CortexAdapter::open_from_snapshot` spawns the fold-tail task, this code does `read_range(replay_start, replay_end)` on the same payloads just to extract `EventMeta::seq_or_ts` for the local origin. Doubles startup IO and CPU on large logs; payloads are read twice. Piggyback `app_seq` discovery onto the fold (read `seq_or_ts` from EventMeta inside the fold callback), or compute it from `TasksState`/`MemoriesState` once caught up.

### 149. Subprotocol manifest serialization order is non-deterministic (DashMap iteration)
`adapter/net/subprotocol/negotiation.rs:39-50` calling `registry.list()` at `registry.rs:96-98` — `DashMap` iteration is non-deterministic across runs and builds, so `from_registry` produces different byte sequences on identical content. Today the manifest is unsigned and not used in any digest dedup, so this is dormant. But the architecture comment at `negotiation.rs:18-19` describes the manifest as "exchanged during session setup" — a surface that typically *does* end up signed once a security model is added. The non-determinism would silently break signature verification on retransmits and any "same manifest? skip re-negotiation" optimisation. Sort by `id` in `from_registry` before emitting.

### 150. RNG `expect` panics across the FFI boundary in identity-layer ops
`adapter/net/identity/envelope.rs:200`, `adapter/net/identity/entity.rs:177`, `adapter/net/identity/token.rs:156, 298` — `getrandom::fill(...).expect("getrandom failed")`. Same hazard pattern as #58 / #61: under FD pressure / sandbox restrictions / kernel hang, `getrandom` returns an error and the unwind crosses `extern "C"` (these helpers are reachable from the FFI bindings under `ffi/mesh.rs`). Surface dedicated error variants instead of panicking.

### 151. `DiffOp::SetField` / `UnsetField` are silent no-ops despite documented as operations
`adapter/net/behavior/diff.rs:634-641` — `apply_op` for these variants does nothing, even under `strict=true` (no error). A peer ships a `SetField{path: "tags", value: [...]}` expecting a state mutation; receiver state is unchanged but `apply` reports `Ok`. Sender's view diverges from receiver's silently, and a `validate_chain` over a sequence of such diffs will not catch the divergence. Either return `DiffError::NotApplicable` for the unimplemented variants, or remove the variants entirely.

### 152. `validate_chain` accepts diffs where `new_version <= base_version` (forward roll-back)
`adapter/net/behavior/diff.rs:649-675` — chains `prev.new_version == curr.base_version` and `prev.timestamp_ns <= curr.timestamp_ns`, but does not check `curr.new_version > curr.base_version` within a single diff. A peer can ship `base_version=5, new_version=3` (a "rollback while applying ops"), and validation accepts it; combined with #125 (no version check in `apply`), a receiver advances state forward but its tracked version goes backward. Add the within-diff `new_version > base_version` assertion.

### 153. `remove_shard_internal` flushes stranded events with `sequence_start = 0`, colliding with the original BatchWorker's first batch under JetStream dedup
**File:** `bus.rs:401-411`

```rust
let batch = crate::event::Batch::new(shard_id, stranded, 0);
let dispatched = dispatch_batch(&*self.adapter, batch, shard_id, ...).await;
```

The hardcoded `0` is the new batch's `sequence_start`. Every `BatchWorker` for a given `shard_id` also starts its sequence at 0 (`shard/batch.rs:163`). The JetStream adapter builds `Nats-Msg-Id = {nonce}:{shard_id}:{sequence_start}:{i}` (`adapter/jetstream.rs:281`). The shard's *original* first batch wrote msg-ids `{nonce}:{sid}:0:{i}`; this stranded-flush batch writes the same ids. JetStream's default 2-min dedup window silently discards them — the events that the post-#47 fix went out of its way to recover are then thrown away by the adapter. Severity: **high** — extension of #9 / #17 / #56 to the `remove_shard_internal` path, which post-#47 fix is the *primary* place stranded events meet the adapter. Fix: thread the BatchWorker's last `next_sequence` through `remove_shard_internal` (it already drops the worker handle at line 395 — capture the sequence before that), or use a sentinel sequence (e.g. `u64::MAX`) reserved for stranded-flush.

### 154. `finalize_draining` declares emptiness from ring-buffer fill alone, ignoring the per-shard mpsc channel and the BatchWorker's pending batch
**File:** `shard/mapper.rs:912-944`

The predicate is `fill_ratio == 0.0 && pushes_since_drain_start == 0`. Both probes look only at the ring buffer / producer-side counter. The drain worker pumps from the ring buffer into a per-shard mpsc channel (cap 1024, `bus.rs:315`); the BatchWorker assembles `current_batch` from that channel. Neither the channel depth nor `BatchWorker.current_batch.len()` enters the predicate. So a Draining shard can finalize → `on_shard_removed` fires → `remove_shard_internal` runs while the BatchWorker still has events in flight. Combined with #153, those in-flight events race the stranded-flush batch through dedup — and since `remove_shard_internal` drops the BatchWorker `JoinHandle` (line 395) without awaiting it, the worker can also be cut short mid-`on_batch`. Severity: **medium**. Fix: track per-shard `pending_in_channel` (incremented on `tx.send`, decremented on `rx.recv` in the batch worker) and require it == 0 alongside the existing checks; alternatively, await the `JoinHandle` in `remove_shard_internal` before invoking the stranded-flush dispatch.

### 155. `events_unrouted` is double-counted as `events_dropped` in `ingest_raw_batch` bus stats
**File:** `bus.rs:548-567` (interaction with `shard/mod.rs:617-620`)

```rust
let total = events.len();
let success = self.shard_manager.ingest_raw_batch(events);
let dropped = total.saturating_sub(success);
if dropped > 0 { self.stats.events_dropped.fetch_add(dropped as u64, ...); }
```

Inside `ingest_raw_batch`, routing-miss events (concurrent scale-down removed the chosen shard) increment `events_unrouted` on the manager — #44 introduced exactly this distinction so callers can tell routing failures from buffer-fullness drops. But the bus then reads `total - success`, sees the unrouted events as missing-from-success, and bumps `events_dropped` for them too. The same event is now counted once in `events_unrouted` and once in `events_dropped` in the public stats surfaced through `bus.stats()`; SDK consumers reading `Stats::events_dropped` over-report backpressure-class drops by exactly the unrouted count. Severity: **low** (cosmetic stats divergence; no event-loss impact). Fix: change `ShardManager::ingest_raw_batch` to return both counts (`success`, `unrouted`) so the bus subtracts `unrouted` from `dropped` before publishing.

---

## Notably clean

`event.rs`, `timestamp.rs`, `error.rs`, `lib.rs`, `consumer/filter.rs`, `shard/batch.rs`. Many would-be bugs in these modules — zero-divisor configs, non-deterministic merge sort tiebreaking (#52), `Filter::And` empty pass-through, sequence-number saturation on `u64::MAX` — already have regression tests pinning the fixes from prior audit passes. (Removed `shard/ring_buffer.rs` from this list — see #77 and #78.)

## Top priorities to fix first

1. **#80** — `Net::shutdown` silently no-ops with outstanding Arc clones (silent data loss on the documented graceful-shutdown path; trivially reproducible via `subscribe`)
2. **#92** — Redex `compact_to` in-memory vs on-disk offset divergence (every event after retention sweep silently lost on restart — directly breaks the "redex-disk" merge's stated goal)
3. **#98** — `ContinuityProof::verify_against` only checks endpoints, never validates the chain in between (primary continuity-bypass vector)
4. **#99** — `SuperpositionState::continuity_proof` uses backward-pointing parent hashes (every migration's continuity assertion fails to verify)
5. **#100** — `LocalGraph::on_pingwave` accepts attacker-poisoned addresses + grows `nodes` map at line-rate (DoS + route hijack)
6. **#101** — Loadbalance probe slot leaks via filter-time side effect (circuit breaker permanently stuck on any multi-endpoint cluster)
7. **#97** — Legacy `NetAdapter` heartbeat sender uses zero key (idle session keep-alive silently dead on the legacy single-peer path)
8. **#55** — JetStream `direct_get` retention-rollover stall (consumer DoS after MAXLEN trim)
9. **#57** — Redis MULTI/EXEC timeout duplicates (silent stream corruption)
10. **#58** — `net_free_bytes` panic-across-FFI on adversarial `len`
11. **#84** — `RxCreditState` auto-grants every consumed byte, defeating receive-side backpressure entirely
12. **#103** — `StandbyGroup::promote` half-mutates state when no standby is healthy (group silently demoted forever)
13. **#104** — Local-source migration silently mutates source daemon state after snapshot is sent (event loss across cutover)
14. **#102** — `SafetyEnforcer::release` underflows `concurrent` / `memory_mb` in Disabled mode (Disabled → Enforce flip permanently rejects every request)
15. **#59** — Bus shutdown timeout strands events despite the "no stranding" contract
16. **#56** — JetStream cross-process retry duplicates (inverse trade-off of #9's fix)
17. **#93** — Redex `compact_to` non-atomic three-rename + missing dir fsync (compounds #92 into segment corruption on crash)
18. **#85** — Mesh dispatch fast-paths heartbeats without AEAD verify (off-path heartbeat spoofing defeats idle timeout)
19. **#109** — `SchemaType::validate` unbounded recursion (stack-overflow DoS via attacker-shipped schema)
20. **#110** — Capability index admits expired announcements with a fresh local TTL (replay vector for revoked capabilities)
21. **#63** — NaN thresholds silently disable auto-scaling
22. **#64** — `scale_up_provisioning` + `activate` over-allocates past `max_shards`
23. **#66** — `update_from_events` cursor regression on unsorted input (re-delivery)
24. **#75** — `add_shard_internal` permanent worker leak on activate failure
25. **#76** — `flush()` phase-2 barrier collapses to one window (re-introduces #16-class loss on many-shard configs)
26. **#127** — `IdentityEnvelope::open` accepts attacker-chosen `signer_pub`, AAD is empty (identity-substitution hole on the migration path; the migration handler's cross-check is the only line of defense)
27. **#121** — `PermissionToken::issue` / `delegate` panic across FFI on a public-only keypair (any post-migration daemon's FFI caller crashes)
28. **#125** — `DiffEngine::apply` declares `VersionMismatch` but never checks the version (silent state divergence under stale diff replay)
29. **#124** — `NodeMetadata` deserialize is unbounded; peer-supplied tags / custom-map flood
30. **#123** — `MetadataStore::upsert` non-atomic 5-step update produces permanent index drift (queries return wrong / duplicate nodes after concurrent writes)
31. **#122** — `SnapshotStore::store` allows older snapshots to overwrite newer (state rewind on replay/race)
32. **#126** — `Tasks/MemoriesAdapter::ingest_typed` advances `app_seq` before successful append (phantom seq on failure, snowballs into duplicate keys on cross-handle restore)
33. **#131** — Subprotocol manifest exposes forwarding-only entries as locally-handled (silent RPC drop)
34. **#132** — `read_manifest_entry` accepts `min_compatible > version` (peer can blacklist subprotocols at will)
35. **#153** — stranded-flush batch from `remove_shard_internal` collides with original first-batch msg-ids under JetStream dedup (the recovery path #47 was added for now silently throws those events away)
36. **#154** — `finalize_draining` ignores mpsc channel + BatchWorker pending; combined with #153, the BatchWorker can be cut short while events are still mid-flight

## Out of scope (deferred)

The `adapter/net/` UDP transport stack — `cortex/`, `swarm/`, `traversal/`, `state/`, `behavior/`, `compute/`, `continuity/` — was deferred in the second pass, spot-checked in the third pass (#97–#120), and the previously-deferred subtrees were systematically swept in the fourth pass (#121–#152). The earliest audit ([BUG_AUDIT_2026_04_30.md](./BUG_AUDIT_2026_04_30.md)) covers some of those subsystems through #54.

Audited across passes 2-4: `redex/`, `mesh.rs`, `session.rs`, `router.rs`, `linux.rs`, `subnet/gateway.rs`, `contested/correlation.rs`, `sdk/src/net.rs`, `adapter/net/{mod.rs, pool.rs, reliability.rs, failure.rs, crypto.rs, protocol.rs}`, `swarm.rs`, `route.rs`, `reroute.rs`, `proxy.rs`, `traversal/{classify.rs, portmap/natpmp.rs}`, all of `behavior/` (`{safety, loadbalance, capability, proximity, api, rules, context, metadata, diff}.rs`), `compute/{orchestrator, standby_group, migration_target, migration_source}.rs`, all of `subprotocol/` (`{descriptor, migration_handler, mod, negotiation, registry, stream_window}.rs`), `continuity/{chain, superposition, discontinuity}.rs`, all of `cortex/` (`{adapter, config, envelope, error, meta, mod}.rs` + `tasks/*` + `memories/*`), all of `state/` (`{causal, horizon, log, snapshot}.rs`), `netdb/`, and `identity/`.

Still **not re-audited** in any pass: `adapter/net/{batch.rs, config.rs, stream.rs, transport.rs}`, `adapter/net/channel/` (entire subtree: `config.rs, guard.rs, membership.rs, mod.rs, name.rs, publisher.rs, roster.rs`), `adapter/net/contested/{partition.rs, reconcile.rs}` (only `correlation.rs` was covered), most of `adapter/net/redex/` (only `disk.rs` and `file.rs` were spot-checked — `entry.rs, event.rs, fold.rs, index.rs, manager.rs, ordered.rs, retention.rs, segment.rs, typed.rs` remain), `adapter/net/subnet/{assignment.rs, id.rs}` (only `gateway.rs` was covered), `adapter/net/traversal/{config.rs, reflex.rs, rendezvous.rs}` and `traversal/portmap/{gateway.rs, sequential.rs, upnp.rs}` (only `classify.rs` and `portmap/natpmp.rs` were covered). Pass 4 was a systematic sweep of the explicitly-listed deferred subtrees but is not a continuous re-audit — code added or modified after this date may contain new defects.
