# Multi-hop Handshake Relay (source-routed, clean subprotocol)

## Status

Single-hop relay ships in `handshake_relay.rs`; multi-hop forwarding via the data-plane routing table was plumbed recently (the `Forward` branch falls back to `router.routing_table().lookup(to_node)`), but it only works when intermediate nodes have *manually* seeded routing-table entries for the final destination. End-to-end multi-hop initiation without that pre-seeding is not yet supported.

## Problem

A `connect_via(relay_addr, dest_pubkey, dest_node_id)` call today works only if:

- The relay has a direct session with `dest` (single hop), OR
- Every intermediate node on the chain has a routing-table entry for `dest` pointing at the correct next hop.

The second condition is satisfied in our tests by calling `node.router().add_route(dest_nid, next_hop_addr)` on each relay. In production, the routing table only populates entries for *direct* peers (via `connect` / `accept`). Nothing teaches intermediate nodes the route to an arbitrary far destination. Result: multi-hop handshakes don't initiate spontaneously.

A second, smaller problem: the current relay code couples the handshake subprotocol to the data-plane routing table. Handshake relay and data forwarding are semantically different — one is a setup operation between trust principals, the other is bulk transport — and mixing them invites subtle bugs (e.g., a reroute that's correct for data can be wrong for a handshake mid-flight).

## Goal

Two properties:

1. **Multi-hop works without pre-seeded routing-table entries.** The initiator should be able to say "connect me to D" and the protocol should figure out the chain.
2. **The subprotocol stays clean and single-relay-shaped at the wire level.** Each relay does one trivial, stateless thing — pop and forward. No coordination state between relays, no fan-out, no dependency on the data-plane routing table.

## Design — source routing

Extend the wire envelope with an **initiator-specified hop list**. The initiator chooses the path (using its local topology view from the proximity graph) and writes it into the envelope. Each relay pops its own entry off the front and forwards to the next.

### Wire format v2

```
[ttl: u8][hop_count: u8][hops: u64 × hop_count][dest: u64 LE][src: u64 LE][noise: variable]
```

- `hop_count = 0` → no intermediates; first-hop recipient *is* the responder. This is the degenerate direct case and matches today's single-relay envelope when the envelope's recipient is directly the responder (rare in practice — usually the first hop is a relay).
- `hop_count > 0` → the list `hops[0..hop_count]` is the ordered chain of intermediate relays. Relay `k` expects `hops[0] == self`, pops, and forwards to `hops[1]` (or to `dest` if `hops` is now empty).

### Noise prologue v2

Current prologue binds `(src, dest)`. Extend to `(src, dest, hop_count, hops[..])` so a malicious relay can't rewrite the chain mid-flight — a tampered hop list produces a prologue mismatch on the responder and the handshake is rejected end-to-end (same MAC-failure mechanism that catches `src`/`dest` tampering today).

### Per-hop rules

On receiving a `SUBPROTOCOL_HANDSHAKE` envelope:

1. If `dest == self.local_node_id`: process as responder (same as today — Noise `read_message`, emit `RegisterResponderPeer`).
2. Else: forward.
   - TTL: decrement; drop at zero (unchanged).
   - If `hop_count > 0`:
     - Check `hops[0] == self.local_node_id`. If not, this envelope was misrouted (buggy initiator or bit-flip). Drop with a warning.
     - Pop `hops[0]`. Decrement `hop_count`.
     - Next hop is the new `hops[0]` if any, else `dest`.
   - If `hop_count == 0`: next hop is `dest` directly. Requires a direct session with `dest` — if none, drop.
   - Resolve the next hop's session via `peers.get(&next_hop_nid)`. Re-encrypt and send.

The relay never consults the data-plane routing table. The handshake chain is entirely self-describing.

### msg2 return path

Responder derives msg2, emits `RegisterResponderPeer` with `response_payload`. The `response_payload` carries a **reversed hop list** (so msg2 retraces the chain back to the initiator). The responder constructs this from the parsed envelope — trivial since the chain is already decoded.

### Initiator API

Two levels:

- Explicit: `connect_via_chain(hops: Vec<u64>, first_hop_addr: SocketAddr, dest_pubkey, dest_node_id)` — caller supplies the chain.
- Convenience: `connect_via_topology(dest_pubkey, dest_node_id)` — internally calls `proximity_graph.path_to(dest) → Vec<NodeId>`, strips self, derives `first_hop_addr` from the peers map, calls `connect_via_chain`.

`connect_via(relay_addr, dest_pubkey, dest_node_id)` stays for the single-relay case and is implemented as `connect_via_chain(vec![relay_nid], relay_addr, ...)`.

## Security

The prologue change is the whole defense. A relay that rewrites its own entry in the hop list (e.g., to route through a different next hop) makes its outgoing envelope's hop list inconsistent with the one the initiator sealed into the Noise transcript; the responder's prologue won't match, and `read_message` fails. A relay that *drops* the envelope is just a denial-of-service and is handled by the caller's handshake timeout.

A relay can still see the authenticated Noise bytes (no change from today); NKpsk0 authentication and the PSK guarantee the relay can't forge them or derive session keys.

DoS surface: a malicious initiator can ask any peer for an arbitrary-length forward chain. Mitigated by:

- TTL budget (8 hops, default).
- `hop_count` is a `u8`: maximum 255 hops. A stricter cap (e.g., 8) can be enforced at decode time.
- Relay-side policy is a natural place to hang "only forward for peers in my subnet" rules; out of scope for this plan but the subprotocol doesn't preclude it.

## Implementation

### Step 1: envelope v2 (`handshake_relay.rs`, ~60 lines)

- `encode_payload(ttl, hops: &[u64], dest, src, noise) -> Bytes`.
- `HandshakePayload { ttl, hops: Vec<u64>, dest_node_id, src_node_id, noise_bytes }`.
- `PAYLOAD_HEADER_LEN` becomes `1 + 1 + 8*hop_count + 8 + 8` — variable. Decode: read ttl+hop_count, check bounds, read `hop_count` u64s, read dest+src, the rest is noise.
- Max `hop_count` enforced at decode: reject > `MAX_RELAY_HOPS` (8).

### Step 2: prologue v2 (`handshake_relay.rs`, ~15 lines)

- `relay_prologue(initiator, responder, hops) -> Vec<u8>` (was `[u8; 32]`).
- Domain tag `b"net-relay-v2\0\0\0\0"` so a v1 relay and a v2 relay can't accidentally interop (they'd produce different prologues and fail auth, which is the right outcome during rollout).
- Bytes: `[tag: 16][initiator: u64][responder: u64][hop_count: u8][hops: u64 × hop_count]`.

### Step 3: `handle_message` dispatch changes (`handshake_relay.rs`, ~40 lines)

- Forward branch: consume `hops[0]` if present; verify it's `self`; compute next-hop nid; emit `HandshakeAction::Forward { to_node: next_hop_nid, payload: re-encoded_with_shorter_hops }`.
- Responder branch: unchanged except that it constructs `response_payload` with the *reversed* incoming hop chain, so msg2 retraces back to the initiator.

### Step 4: `MeshNode::execute_handshake_action` simplification (`mesh.rs`, ~20 lines)

- `HandshakeAction::Forward { to_node, payload }` stays the same shape.
- The executor now looks up `to_node` **only** in `peers` (direct session). The routing-table fallback goes away — source routing means each hop is a direct peer by construction.

### Step 5: initiator API surface (`handshake_relay.rs` + `mesh.rs`, ~40 lines)

- `HandshakeHandler::begin_initiator_via_chain(dest, dest_pubkey, hops: Vec<u64>) -> ...`.
- `MeshNode::connect_via_chain(hops: Vec<u64>, first_hop_addr, dest_pubkey, dest_node_id) -> Result<u64, AdapterError>`.
- `MeshNode::connect_via_topology(dest_pubkey, dest_node_id) -> Result<u64, AdapterError>` — queries `proximity_graph.path_to(dest)`, strips self, derives `first_hop_addr` from `peer_addrs`, calls `connect_via_chain`. Returns `AdapterError::Connection("no path to {dest}")` if the graph has nothing.
- `connect_via(relay_addr, dest_pubkey, dest_node_id)` delegates to `connect_via_chain(vec![relay_nid], relay_addr, ...)` after resolving `relay_nid` from `addr_to_node`.

### Step 6: tests (~150 lines)

- Unit: wire-format roundtrip for v2 envelopes with 0, 1, and N hops; `hop_count > MAX_RELAY_HOPS` rejected at decode.
- Unit: relay pops `hops[0]` and decrements `hop_count` on `Forward`.
- Unit: tampering with the hop list (rewrite, insert, remove entries) is caught by the prologue — responder emits zero actions.
- Integration (existing 4-node test updated): `a.connect_via_chain([nid_b, nid_c], addr_b, &pub_d, nid_d)` succeeds without any manual `router().add_route` calls on B or C.
- Integration: `connect_via_topology` succeeds once pingwaves have populated the proximity graph enough to produce a path (test waits with a bounded timeout).

## Non-goals

- Multi-relay coordination (e.g., fan-out or redundant-path handshakes). Out of scope; this plan deliberately keeps each relay stateless.
- Automatic path re-selection during a handshake. If the chosen chain goes down mid-handshake, the initiator times out and retries — same as the current failure mode.
- Replacement of `proximity_graph.path_to` as the path source. That function is already implemented and good enough; if it returns `None`, that's a discovery problem, not a relay problem.
- Rollout / wire compatibility with v1. v2 is a breaking change to the envelope; plan is a coordinated upgrade. If gradual rollout is needed, that's a separate negotiation subprotocol job.

## Files touched

| File | Purpose |
|---|---|
| `src/adapter/net/handshake_relay.rs` | v2 envelope, prologue, handler dispatch, new initiator entry points |
| `src/adapter/net/mesh.rs` | `connect_via_chain`, `connect_via_topology`; `execute_handshake_action` simplification |
| `src/adapter/net/crypto.rs` | Prologue API already takes `&[u8]`; no change |
| `docs/HANDSHAKE_RELAY_PLAN.md` | Add a note that single-hop is a special case of the v2 source-routed design |
| `docs/SUBPROTOCOLS.md` / `docs/TRANSPORT.md` | Mention the v2 envelope shape |
| `tests/three_node_integration.rs` | 4-node chain test using `connect_via_chain`; topology-driven variant |

## Scope

~280 lines across src + tests. One breaking wire change gated behind the `v2` prologue tag so it can't silently interop with the v1 format.
