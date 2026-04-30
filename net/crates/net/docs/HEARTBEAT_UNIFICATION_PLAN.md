# Heartbeat Unification Plan — 2026-04-30 (revised)

The same primitive — "AEAD-tagged keep-alive packet for a session" — used to have three independent implementations in the tree, with the same wire shape but inconsistent verification logic. Audit findings #11, #85, and #97 were all symptoms.

**Update (post-implementation):** the security-critical correctness bugs have been fixed. Heartbeats are now built and verified consistently on the wire. The remaining unification work is cleanup — removing duplicated code paths and locking the API door so the bugs cannot recur. This document tracks what changed vs. the original plan and what's still open.

## Current state (post-fix)

Send side, both call sites identical:

```rust
let mut pooled = session.thread_local_pool().get();
let packet = pooled.build_heartbeat();
```

- Legacy: `adapter/net/mod.rs:888-889` (in `spawn_heartbeat`).
- Mesh: `adapter/net/mesh.rs:3380-3381` (in `spawn_heartbeat_loop`).

Both now use the session's `thread_local_pool` — the same pool the data path uses, so heartbeats and data share a single TX counter. This closes both the wrong-key half of #97 and the counter-conflict variant: a fresh `PacketBuilder::new(&[0u8;32], ...)` builder owned its own counter starting at 0, so successive heartbeats reused counter=0 and the receiver replay-rejected every heartbeat after the first. Pinned by `aead_authenticated_heartbeat_passes_verification` (`mesh.rs:7407`) and the `process_packet` heartbeat suite (`mod.rs:1786`).

Receive side, two implementations of the same logic:

| Path | Location | Shape |
|---|---|---|
| Legacy `NetAdapter` | `mod.rs:659-680` (22 lines, inline) | full AEAD verify: source check → counter validity → decrypt → counter commit → touch |
| Mesh `MeshNode` | `mesh.rs:2500-2522` calls free fn `verify_heartbeat_aead(&parsed, &session)` at `mesh.rs:760-771` | same logic minus the source check (handled upstream by session lookup) and minus the touch (caller does it explicitly so `failure_detector.heartbeat` runs first) |

Both pass identical regression tests: `unauthenticated_heartbeat_fails_verification` (`mesh.rs:7424`) and the `process_packet` heartbeat suite. **The bugs are closed.** What remains is two implementations that should be one.

`Session` API surface (`adapter/net/session.rs`) post-fix:
- `rx_cipher()` exposed at line 201.
- `thread_local_pool()` exposed at line 622.
- `tx_cipher()` **removed** — no longer in the public API.
- `packet_pool()` getter **removed** — there is now exactly one pool reachable per session.

The two latent footguns behind #106 (parallel pools, parallel ciphers) are gone. The remaining heartbeat unification is purely cosmetic: collapse the two-line build pattern into one method and the duplicated verify into one call.

## Outstanding work

The four steps below are *cleanup* — they do not fix new bugs, but they remove ~30 lines of duplication and prevent regression.

### Step 1 — Move `verify_heartbeat_aead` from `mesh.rs` to `session.rs` (or to a shared module) and rewrite the legacy receive path to call it

`verify_heartbeat_aead` (`mesh.rs:760-771`) is a free function that operates on `(&ParsedPacket, &NetSession)`. It belongs alongside the session it verifies against, not inside the mesh module. Moving it (and exposing it `pub(crate)`) lets the legacy path replace 22 lines (`mod.rs:659-680`) with:

```rust
if parsed.header.flags.is_heartbeat() {
    if source != session.peer_addr() {
        return;
    }
    if !verify_heartbeat_aead(&parsed, &session) {
        return;
    }
    session.touch();
    return;
}
```

Why keep the source check at the call site, not inside the helper: mesh dispatch resolves the session by `session_id` first (so it doesn't have a single canonical "expected source"), while the legacy adapter has exactly one peer per session. Centralizing the source check would force the helper to either accept `Option<SocketAddr>` or duplicate again. The cheapest fix is to leave the source-check at the legacy call site and let the helper own the AEAD-only portion that's actually shared.

### Step 2 — Wrap the build-side two-liner in `NetSession::build_heartbeat()`

Both call sites do:
```rust
let mut pooled = session.thread_local_pool().get();
let packet = pooled.build_heartbeat();
```

Replace with:
```rust
impl NetSession {
    pub fn build_heartbeat(&self) -> Bytes {
        self.thread_local_pool().get().build_heartbeat()
    }
}
```

Each call site becomes `let packet = session.build_heartbeat();`. Two lines saved per site, but more importantly the next person who needs to build a heartbeat doesn't have to know about `thread_local_pool` — and won't accidentally reach for `PacketBuilder::new(&[0u8; 32], ...)` again.

### Step 3 — Demote `PacketBuilder::new` from `pub` to `pub(crate)`

After Steps 1–2, `PacketBuilder::new` has no remaining heartbeat callers in production. The remaining `PacketBuilder::new(&[0u8; 32], 0)` call sites are all handshake builders (e.g. `mod.rs:484, 596`; `mesh.rs:2637`) — handshakes don't have a session key yet, so the zero-key construction is correct there. Those handshake call sites are all inside `adapter/net/`, so demoting `pub` → `pub(crate)` does not break them.

This is the structural guarantee that #97-shape bugs cannot recur from outside the crate. Verify nothing in `bindings/`, `sdk/`, `sdk-py/`, `sdk-ts/`, or `cli/` calls `PacketBuilder::new` directly before flipping the modifier.

### Step 4 — Static check

Add a CI grep (or a `compile_fail` test) asserting that no `*.rs` file outside `adapter/net/` constructs a `PacketBuilder::new` and that no file outside `session.rs` and `pool.rs` calls `build_heartbeat()` on anything other than the session helper. Catches future drift cheaply.

## What's already done

- ✅ Send-side bug (#97): both call sites route through `session.thread_local_pool()`. Heartbeat counter is shared with the data path.
- ✅ Receive-side bug (#85): mesh dispatch verifies AEAD before touching `failure_detector` or session state.
- ✅ Dual-pool / dual-cipher footgun (#106 surface): `Session::tx_cipher()` and `Session::packet_pool()` removed.
- ✅ Regression tests: `aead_authenticated_heartbeat_passes_verification`, `unauthenticated_heartbeat_fails_verification` (mesh side); the `process_packet` heartbeat suite at `mod.rs:1786` covering legitimate / no-tag / garbage-tag / `session.touch()` invariants.

## What's deliberately not done

- **`Session::verify_inbound_heartbeat` (the original Step 1 of this plan).** The receive paths diverge enough — mesh's session lookup precedes verify; legacy's source check precedes verify; only mesh wants `failure_detector.heartbeat` after verify — that an opinionated "verify and touch" wrapper either needs flags (failure detector? source check?) or duplicates state-mutation logic in the caller anyway. The simpler `verify_heartbeat_aead(&parsed, &session) -> bool` predicate (currently in `mesh.rs`) covers the actually-shared portion and lets each caller compose the rest. Step 1 above moves that helper to its proper home rather than escalating it into a method.
- **#56** (JetStream cross-process nonce) — different layer.
- **Subnet-gateway heartbeat handling** — gateways don't process heartbeats today.

## Risks

- **`pub` → `pub(crate)` on `PacketBuilder::new`.** Any external caller breaks. The constructor is not in the documented SDK surface, but the bindings (`bindings/`, `sdk*/`) need a quick grep before flipping.
- **Moving `verify_heartbeat_aead` out of `mesh.rs`.** Tests at `mesh.rs:7407, 7424, 7676, 7682, 7698, 7702` reference the symbol via `super::*`. They keep working as long as the symbol is re-exported `pub(crate)` from its new location and the mesh module imports it. Verify the test imports compile after the move.

## Acceptance criteria for the remaining work

- Legacy receive path (`mod.rs:659-680`) calls a single shared `verify_heartbeat_aead`-style helper rather than re-implementing AEAD verify.
- Both build sites read `let packet = session.build_heartbeat();` (single line each).
- `PacketBuilder::new` is `pub(crate)`.
- Existing regression tests still pass; no new tests required for the cleanup itself (the helper is exercised by the same tests that already pin #85 and #97).

## Status

| Step | State |
|---|---|
| Send-side fix (#97) — route through `thread_local_pool` | ✅ done |
| Mesh receive-side fix (#85) — `verify_heartbeat_aead` helper | ✅ done |
| Drop `Session::tx_cipher` / `Session::packet_pool` getters | ✅ done |
| Regression tests for #85 and #97 | ✅ done |
| 1 — Move `verify_heartbeat_aead` to `session.rs`, port legacy receive to call it | outstanding |
| 2 — `NetSession::build_heartbeat()` wrapper, port both build sites | outstanding |
| 3 — `PacketBuilder::new` → `pub(crate)` | outstanding |
| 4 — Static check / CI grep | outstanding |

Closed by the implementation: **#11**, **#85**, **#97**, surface area for **#106**. Outstanding steps reduce duplication only; they do not fix new bugs.
