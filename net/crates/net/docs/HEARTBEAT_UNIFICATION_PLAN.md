# Heartbeat Unification Plan — 2026-04-30

The same primitive — "AEAD-tagged keep-alive packet for a session" — has three independent implementations in the tree. They have the same shape on the wire and overlapping (but inconsistent) verification logic. The audit findings #11, #85, and #97 are all symptoms of the duplication; #56 / #106 / #97 also rhyme. This plan replaces the three implementations with one helper on `NetSession` and reduces every call site to a single line.

## Current state

Three call sites for build, two for verify:

| Path | Build site | Verify site |
|---|---|---|
| Legacy `NetAdapter` | `adapter/net/mod.rs:841` — `PacketBuilder::new(&[0u8;32], session.session_id())` (zero key, **#97**) | `adapter/net/mod.rs:642-664` — full AEAD verify via `session.rx_cipher` (correct since **#11** fix) |
| Mesh `MeshNode` | `adapter/net/mesh.rs:3170` — same zero-key shape as legacy | `adapter/net/mesh.rs:2367-2371` — fast-path **skips** AEAD verify (**#85**), then touches `failure_detector` + session |
| Pool primitive | `adapter/net/pool.rs:251-281` — `PacketBuilder::build_heartbeat()` (correct given a correctly-keyed builder) | n/a |

What `NetSession` exposes today (`adapter/net/session.rs`): `tx_cipher()`, `rx_cipher()`, `touch()`, `thread_local_pool()`. **No** "build a heartbeat for this session" helper, **no** "verify this inbound heartbeat" helper. Every call site reassembles the primitive in-line; that is why two of the three reach the same wrong key.

## Goal

One blessed way to (a) build a heartbeat for a session, (b) verify-and-observe an inbound heartbeat. After this lands, every call site is a single function call and bugs #11, #85, #97 cannot be reintroduced without a structural change to `NetSession`.

## Proposed API

Add two methods to `NetSession` (`adapter/net/session.rs`):

```rust
impl NetSession {
    /// Build an authenticated heartbeat packet for this session.
    /// AEAD-encrypts an empty payload under the session's TX cipher and
    /// assigns the next TX counter. Routed through `thread_local_pool` so
    /// the counter is shared with the data-path builders (closes #106).
    pub fn build_heartbeat(&self) -> Result<Vec<u8>, BuildError>;

    /// Verify an inbound heartbeat. On success: validates source matches
    /// `peer_addr`, the counter is in the rx replay window, AEAD tag
    /// verifies, then commits the counter and refreshes `last_activity`.
    /// On failure: returns an error and does NOT mutate session state.
    pub fn verify_inbound_heartbeat(
        &self,
        parsed: &ParsedPacket,
        source: SocketAddr,
    ) -> Result<(), VerifyError>;
}
```

Design choices:
- `build_heartbeat` returns owned bytes; per-peer fan-out in mesh stays at the caller.
- `verify_inbound_heartbeat` performs the side effects internally (`update_rx_counter` + `touch`) so callers cannot get the order wrong.
- The function does **not** touch `failure_detector` — that observation is mesh-specific and stays at the call site, immediately after a successful verify.
- `BuildError` / `VerifyError` are local to `session.rs` and convert into the existing adapter-level error types.

## Migration

1. **Implement** `NetSession::build_heartbeat` and `verify_inbound_heartbeat` (`adapter/net/session.rs`). Both are thin wrappers — `build_heartbeat` calls `self.thread_local_pool.build_heartbeat(...)`; `verify_inbound_heartbeat` lifts the verify sequence verbatim from `mod.rs:642-664`.
2. **Legacy send.** Replace `adapter/net/mod.rs:841-842`:
   ```rust
   // before
   let mut builder = PacketBuilder::new(&[0u8; 32], session.session_id());
   let packet = builder.build_heartbeat();
   // after
   let packet = session.build_heartbeat()?;
   ```
3. **Legacy receive.** Replace the body of the heartbeat fast-path at `mod.rs:642-664` with:
   ```rust
   if parsed.header.flags.is_heartbeat() {
       if session.verify_inbound_heartbeat(&parsed, source).is_err() {
           return;
       }
       return;
   }
   ```
4. **Mesh send.** Replace `mesh.rs:3170-3172` with `let packet = session.build_heartbeat()?;`. The per-peer loop scaffolding (cadence, shutdown gate, partition filter) stays — only the primitive changes.
5. **Mesh receive.** Replace `mesh.rs:2367-2371` with:
   ```rust
   if parsed.header.flags.is_heartbeat() {
       if session.verify_inbound_heartbeat(&parsed, source).is_err() {
           return;
       }
       failure_detector.heartbeat(peer_node_id, source);
       return;
   }
   ```
   This is the only structural difference between the two receive paths after unification: mesh has one extra line for the failure detector.
6. **Lock the door.** Demote `PacketBuilder::new` from `pub` to `pub(crate)` (`adapter/net/pool.rs`). Heartbeats already go through `Session`; data-path builders go through the pool. No legitimate external caller needs raw key bytes. This is the structural guarantee that #97-shape bugs cannot recur.
7. **Drop the dead getter** `Session::packet_pool()` (`session.rs:562`) and the parallel `tx_cipher` if unused after step 1 — both are the latent surface area behind #106. If any caller surfaces, route it through the unified API instead of re-exposing the raw cipher.

## Test plan

- **Round-trip:** `Session::build_heartbeat` for an init session, `verify_inbound_heartbeat` on the responder session, assert `Ok(())` and `last_activity` advanced. Pin #97.
- **Tag tamper:** flip one byte of the built packet's tag, assert `verify_inbound_heartbeat` returns `Err` and `last_activity` is **unchanged**. Pin #85.
- **Source mismatch:** valid packet from the wrong `SocketAddr`, assert reject + no state change.
- **Replay:** verify, then re-submit the same bytes, assert the second verify rejects via the rx counter window.
- **Counter monotonicity:** call `build_heartbeat` N times, assert the TX counter monotonically increases and is consistent with `thread_local_pool`'s counter (closes the dormant variant of #106 on this path).
- **Static check:** a `compile_fail` test (or a CI `rg`) asserting that no `*.rs` outside `pool.rs` calls `PacketBuilder::new`. Catches future drift.

## Out of scope

- **#56** (cross-process JetStream nonce) — different layer.
- **#106** (full dual-pool counter consolidation) — this plan removes the failure mode for the heartbeat path; the broader consolidation is a separate cleanup.
- Subnet-gateway heartbeat handling — gateways don't process heartbeats today; if that changes, the new code must call `Session::verify_inbound_heartbeat` rather than re-rolling the verify sequence.

## Risks

- **Per-peer fan-out in mesh.** The mesh send loop iterates peers and calls `build_heartbeat` once per peer per tick. The session-internal `thread_local_pool` must remain per-session, not per-peer; a session is held by the mesh loop while iterating, so this is structurally fine, but worth confirming in the implementation review (the alternative — accidentally reaching for a per-peer pool — would re-introduce #106).
- **Fan-out cost.** If a session has 1000 peers and `Session::build_heartbeat` runs sequentially per peer, throughput is bounded by AEAD throughput × peer count per tick. The current code has the same cost; verify there is no regression.
- **`pub` → `pub(crate)` on `PacketBuilder::new`.** Any downstream crate that constructed `PacketBuilder` directly breaks. This crate is workspace-internal and `PacketBuilder::new` is not in the documented SDK surface, but check the bindings (`bindings/`, `sdk*/`) before flipping.

## Acceptance criteria

- All five call sites in `mod.rs` and `mesh.rs` reduced to one-line `Session::build_heartbeat` / `Session::verify_inbound_heartbeat` calls.
- `PacketBuilder::new` not callable from outside `adapter/net/`.
- New tests pinning #11, #85, #97 (round-trip, tag tamper, source mismatch, replay).
- No new findings introduced (re-run the relevant audits in `BUG_AUDIT_2026_04_30_CORE.md` against the changed files).

## Status

| Step | Outstanding |
|---|---|
| 1 — `Session::build_heartbeat` / `verify_inbound_heartbeat` | not started |
| 2 — legacy send rewrite | not started |
| 3 — legacy receive rewrite | not started |
| 4 — mesh send rewrite | not started |
| 5 — mesh receive rewrite | not started |
| 6 — `PacketBuilder::new` → `pub(crate)` | not started |
| 7 — drop unused `Session::packet_pool` getter | not started |
| Tests | not started |

Closes audit findings: **#11**, **#85**, **#97**. Reduces surface area for **#106**.
