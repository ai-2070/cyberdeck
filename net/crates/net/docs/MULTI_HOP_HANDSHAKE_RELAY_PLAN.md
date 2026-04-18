# Multi-hop Handshake: handshakes are just routed packets

## Status

A dedicated single-relay handshake subprotocol (`SUBPROTOCOL_HANDSHAKE = 0x0601`) ships in `handshake_relay.rs`. It works for chains up to one intermediate relay, and the `Forward` path falls back to the routing table for additional hops when routes are pre-seeded manually. That design conflates two concerns — handshake packaging and multi-hop forwarding — in a way that duplicates routing metadata and invents relay-specific state for no real gain.

This plan collapses handshake relay into the generic routing path. After these changes, there is no handshake subprotocol: handshake packets are ordinary Net packets with the `HANDSHAKE` flag set and a routing header prepended. Multi-hop works the same way it does for data and heartbeats — through the routing table.

## Problem

Two coupled problems currently live in `handshake_relay.rs`:

1. **Relay envelope duplicates routing metadata.** The wire format `[ttl][hop][dest][src][noise]` replicates the `(dest_id, src_id, ttl, hop_count)` fields already present in the `RoutingHeader`. Two sources of truth for "where is this going".
2. **Per-hop session re-encryption is load-bearing.** Each relay decrypts the incoming subprotocol packet (under its session with the previous hop), reads the Noise bytes, re-encrypts (under its session with the next hop). This is defense-in-depth on top of Noise NKpsk0, which already authenticates the handshake end-to-end. The cost is a bespoke relay loop that has to track sessions, preserve payload integrity across re-encryption, and handle a separate set of failure modes.

Neither is necessary. Noise provides confidentiality and authenticity of the handshake bytes regardless of whether each hop re-encrypts. The routing header already carries destination and TTL. The correct factoring is: handshake is just one more packet type that flows through the standard routing plane.

## Design

### Handshake packets ARE routed packets

Handshake is identified by the existing `HANDSHAKE` flag in the Net header, **not** by a subprotocol ID. Relays don't need to know a packet is a handshake — they only see "some Net packet destined for `dest_id = C`" and forward it.

### Stack layering (on the wire, multi-hop)

```
[RoutingHeader: src=A, dest=C, ttl, hop_count, ...]
[NetHeader:     HANDSHAKE=1, session_id=0, ...]
[payload:       Noise msgN bytes]
```

**At a relay**:
- Reads `RoutingHeader`.
- Decrements TTL / updates `hop_count`.
- Forwards as-is to next hop based on `RoutingTable::lookup(dest_id)`.
- Does not touch the `NetHeader` or the payload.

**At the responder**:
- Strips `RoutingHeader`.
- Sees `HANDSHAKE=1` in the `NetHeader`.
- Hands the payload to the Noise responder state, keyed by `(src_id, dest_id)` taken from the routing header that just arrived.

**At the initiator (for the return msg2)**:
- Receives a routed packet with `dest=self`, `HANDSHAKE=1`, `session_id=0`.
- Strips routing header, processes msg2, derives session keys.

No "inner envelope". No per-hop crypto beyond what's already in play at the link layer (see below).

### What lives where

| Concern | Where it's solved |
|---|---|
| Next-hop selection | `RoutingTable::lookup(dest_id)` — same as data |
| TTL / loop prevention | `RoutingHeader.ttl` — same as data |
| Payload confidentiality on the wire | Link-layer concern; not guaranteed for handshakes, and doesn't need to be |
| Payload confidentiality end-to-end | Noise NKpsk0: msg1 is encrypted to responder's static pubkey, msg2 to the ephemeral ee+es+psk combination |
| Payload authenticity end-to-end | Noise NKpsk0 + PSK |
| (dest, src) tamper resistance | Noise prologue binds `(routing_header.dest_id, routing_header.src_id)` — a relay that rewrites either fails the responder's MAC check |
| Multi-hop | Emerges from the routing table learning non-direct routes (separate plan) |

### Security detail: prologue binds routing-header src/dest

The routing header is unauthenticated metadata. A malicious relay could rewrite `src_id` before forwarding; the responder would still decrypt msg1 correctly (Noise doesn't depend on that field) but would register the derived session under whatever `src_id` the relay wrote. Cheap peer-ID spoofing.

Fix: both sides bind the routing header's `(src_id, dest_id)` into the Noise prologue via a domain-tagged construction:

```
prologue = "net-handshake-v1" || src_id_LE(8) || dest_id_LE(8)
// total: 16 + 8 + 8 = 32 bytes, matching crypto::handshake_prologue().
```

- **Initiator** uses `(src = self.node_id, dest = peer_node_id)`.
- **Responder** uses `(src = routing_header.src_id, dest = self.node_id)` — taken from the routing header that just arrived.
- Endpoints use this prologue for all Noise messages in the handshake.

Consequences:
- If a relay rewrites `src_id`: responder's prologue differs from initiator's → MAC check fails on msg1 → no session keys derived → handshake rejected.
- If a relay rewrites `dest_id`: two overlapping defenses. (a) Noise NKpsk0 encrypts msg1 to the targeted responder's static pubkey, so a redirected msg1 fails to decrypt at the wrong responder. (b) If somehow the same pubkey is reachable under a different `dest_id`, the prologue still mismatches. Belt-and-suspenders.

That's all we need. No hop lists, no extra state, no per-handshake tracking of the path.

### End-to-end vs link-layer confidentiality

We rely on Noise NKpsk0 for end-to-end confidentiality and authenticity of handshake messages — a passive observer between hops without the PSK cannot read them; a compromised relay with the PSK could (but also could forge msg1 for any origin it wants to impersonate, so having the PSK already breaks the trust boundary). If operators want hop-by-hop encryption as a separate defense, that lives at the link/transport layer (e.g., wrapping the routed packet in a per-hop Net session) and is explicitly **not** part of the handshake protocol.

## What gets deleted

| File / symbol | Fate |
|---|---|
| `src/adapter/net/handshake_relay.rs` | Deleted |
| `SUBPROTOCOL_HANDSHAKE = 0x0601` | Removed from everywhere (mesh.rs, subprotocol-id table, docs) |
| `HandshakeHandler`, `HandshakePayload`, `encode_payload`, `relay_prologue` | Deleted — replaced by a prologue helper in `crypto.rs` and routed-packet dispatch in `mesh.rs` |
| Bespoke per-hop re-encryption in `execute_handshake_action` | Deleted |

Net deletion ≈ 500 lines. Replaced by ~80 lines of changes in `mesh.rs`.

## What gets added or changed

### 1. `mesh.rs::dispatch_packet` — routed handshakes no longer dropped at destination

Current code has:

```rust
if parsed.header.flags.is_handshake() || parsed.header.flags.is_heartbeat() {
    return;
}
```

inside the `is_routed && dest_id == local_node_id` branch. Split that: heartbeats stay dropped (they're link-local by definition); handshakes are forwarded into the responder-accept path.

### 2. `MeshNode::connect_via(relay_addr, dest_pubkey, dest_node_id)` — rewrite

Replace the current `SUBPROTOCOL_HANDSHAKE` machinery with:

- Build msg1 using `NoiseHandshake::initiator_with_prologue` where `prologue = handshake_prologue(src=self.node_id, dest=dest_node_id)`.
- Wrap the resulting handshake packet in a routing header with `dest=dest_node_id, src=self.node_id, ttl=DEFAULT_HANDSHAKE_TTL`.
- Send to `relay_addr` (the first hop) via the raw socket — no session encryption needed for handshake.
- Wait for a routed handshake packet to arrive with `dest=self` and `session_id=0`, process msg2, derive keys, register peer.

The **wait** is integrated with the existing receive loop. Two equivalent implementations:
- A small `pending_initiator: DashMap<u64 peer_node_id, oneshot::Sender<…>>` on `MeshNode`, populated at the start of `connect_via` and consumed by `dispatch_packet` when a matching routed-handshake msg2 arrives. This is what today's `handshake_relay::HandshakeHandler` already does — reuse the same shape.
- Or a per-call oneshot + a future-registered wake hook. Either works; the DashMap is simpler and matches the rest of the codebase's style.

There's no global `HandshakeHandler` state beyond that one map; no module-level subprotocol machinery.

### 3. Responder-accept path

The existing `handshake_responder` loop already pulls handshake packets off the socket. Extend it to:

- Recognize **routed-arrival** handshakes: if the packet is routed and `dest == self`, strip the routing header and feed the inner handshake packet into Noise responder state.
- Always compute the prologue from `(src_id, dest_id)`:
  - For **direct** handshakes: `(src = peer_node_id from accept API, dest = self.node_id)`.
  - For **routed** handshakes: `(src = routing_header.src_id, dest = self.node_id)`.
- For msg2 on the routed path:
  - Wrap the reply in a routing header with `src = self.node_id`, `dest = routing_header.src_id` (from the msg1 envelope).
  - Send via the routing table's next-hop lookup, **falling back to the immediate upstream `source`** (the peer that delivered msg1) if no route to the initiator is known yet. At msg2 time the responder has not yet registered the initiator as a peer, so a pure routing-table lookup can dead-end; the upstream peer is guaranteed reachable by construction and has the symmetric return path.
- Register the new session keyed by the resolved `peer_node_id`. For routed handshakes, `peer_node_id = routing_header.src_id`; for direct, the existing resolution stands.

### 4. Prologue helper in `crypto.rs`

One small helper:

```rust
pub fn handshake_prologue(src_node_id: u64, dest_node_id: u64) -> [u8; 16 + 16] {
    let mut buf = [0u8; 32];
    buf[0..16].copy_from_slice(b"net-handshake-v1");
    buf[16..24].copy_from_slice(&src_node_id.to_le_bytes());
    buf[24..32].copy_from_slice(&dest_node_id.to_le_bytes());
    buf
}
```

(Replaces `relay_prologue` from the deleted module. Same construction, different domain tag.)

### 5. Direct-connect path (unchanged semantics, minor refactor)

`MeshNode::connect(peer_addr, peer_pubkey, peer_node_id)` for a *direct* UDP peer should also bind `(self.node_id, peer_node_id)` into the prologue, so direct and relayed handshakes share one prologue convention. This is a mechanical change: flip `NoiseHandshake::initiator` → `NoiseHandshake::initiator_with_prologue` on the direct path and mirror on the responder side. No wire change.

## API surface

- `MeshNode::connect(peer_addr, peer_pubkey, peer_node_id)` — direct UDP. Unchanged signature.
- `MeshNode::connect_via(relay_addr, peer_pubkey, peer_node_id)` — same signature, new internals. Still the common entry point for "I have a first-hop and want a session with peer X."
- `MeshNode::connect_routed(peer_pubkey, peer_node_id)` — new convenience: looks up `peer_node_id` in the routing table, picks the next-hop addr, delegates to `connect_via`. Depends on the routing table being populated (see separate plan).

The previous `connect_via_chain` / `connect_via_topology` idea disappears — source routing isn't needed.

## Dependencies — and what this plan is NOT

This plan assumes the routing plane can deliver packets to `dest_node_id` across multiple hops. **Today it cannot**: `RoutingTable` only knows about direct peers. Multi-hop routing discovery — learning and maintaining routes to non-direct nodes via `ProximityGraph` + pingwaves — is a **separate** plan and will be implemented outside the handshake logic. See [MULTI_HOP_ROUTING_PLAN.md](MULTI_HOP_ROUTING_PLAN.md).

The split is deliberate: if a future reader is tempted to re-solve routing inside the handshake protocol, the answer is no. Routing is a routing problem.

This plan stands alone for the single-relay case: `connect_via(addr_of_B, pubkey_of_C, nid_of_C)` works end-to-end after implementation, because the first-hop addr is supplied by the caller. `connect_routed` becomes useful in topologies larger than one relay only after the routing-discovery plan lands.

## Tests

- Unit: handshake packet with a routing header round-trips through a mock `route_packet` that forwards by dest_id. Destination receives and completes Noise.
- Unit: tampered `src_id` in routing header → responder's Noise `read_message` fails (prologue mismatch).
- Integration (3-node): `connect_via(addr_b, &pub_c, nid_c)` on a 3-node chain A-B-C, B has direct sessions with both. Replaces current `test_mesh_handshake_via_relay` with the new plumbing but asserts the same externally-visible outcome.
- Integration (4-node, requires multi-hop routing): `connect_routed(&pub_d, nid_d)` on A-B-C-D with routes seeded on A (D→B) and B (D→C). Identical to today's `test_regression_handshake_relay_multi_hop_via_routing_table` but using the new path.
- All existing direct-connect tests pass unchanged.

## Scope

~80 lines added (mesh.rs dispatch + connect_via + prologue helper), ~500 lines deleted (handshake_relay.rs + its tests). Net reduction. One coordinated change: the `HANDSHAKE`-flag packet is now sometimes routed, so all nodes must be on the new dispatch code at rollout time. No wire format churn on the handshake payload itself — the Noise bytes and Net header are unchanged; only the outer routing wrapper is new for the relayed case.
