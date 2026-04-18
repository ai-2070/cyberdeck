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

An initiator that wants to establish a session with `C` (which it has no direct UDP path to) emits a normal Net packet:

```
[RoutingHeader: dest=C, src=A, ttl, hop_count][NetHeader: HANDSHAKE flag, session_id=0][noise_bytes]
```

- The inner Net packet is exactly what today's direct-connect handshake produces via `PacketBuilder::build_handshake` — header with `HANDSHAKE` flag and `session_id = 0`, payload is the raw Noise message.
- The outer `RoutingHeader` is what today's `send_routed` prepends for data packets.
- Every intermediate hop forwards by routing header only. The inner packet is never inspected or modified.
- At the destination, the routing header is stripped and the Net packet is dispatched like any other handshake packet — fed to the responder half of the Noise state machine, which writes `msg2`, and the response goes back through the routing plane the same way.

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

Fix (same mechanism we already ship for the single-relay envelope): both sides bind the routing header's `(src_id, dest_id)` into the Noise prologue via a domain-tagged construction:

```
prologue = "net-handshake-v1\0\0\0\0" || src_id_LE(8) || dest_id_LE(8)
```

Initiator's view: `src = self, dest = C`. Responder's view after parsing the routing header it received: `src = routing_header.src_id, dest = self`. If a relay rewrote either, the prologues differ, `read_message` fails the MAC check on msg1, and the handshake is rejected end-to-end.

Dest tampering is also inherently caught by Noise NKpsk0 — msg1 is encrypted to the responder's static pubkey, so a relay redirecting to a different node produces a decryption failure at the wrong-responder. The prologue is belt-and-suspenders for dest and load-bearing for src.

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
- Send to `relay_addr` (the first hop). `send_to` on the raw socket — no session encryption needed for handshake.
- Wait for a routed handshake packet to arrive with `dest=self` and `session_id=0`, process msg2, derive keys, register peer.

There's no longer any notion of a handshake-specific initiator map. The `pending_initiator` state becomes per-call local (stored in the `connect_via` future's stack frame).

### 3. Responder-accept path

The existing `handshake_responder` loop already pulls handshake packets off the socket. Extend it to also recognize routed-arrival handshakes: if the packet is routed *and* `dest == self`, strip the routing header and process the inner handshake packet. The responder then sends msg2 back by reversing the routing (dest := incoming src, src := self) via the routing table.

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

## Dependencies

**Blocker for spontaneous multi-hop:** the routing table has to know routes to non-direct peers. That's [MULTI_HOP_ROUTING_PLAN.md](MULTI_HOP_ROUTING_PLAN.md) and is required before `connect_routed` becomes useful in topologies larger than one relay.

This plan stands alone for the single-relay case: `connect_via(addr_of_B, pubkey_of_C, nid_of_C)` works end-to-end after implementation.

## Tests

- Unit: handshake packet with a routing header round-trips through a mock `route_packet` that forwards by dest_id. Destination receives and completes Noise.
- Unit: tampered `src_id` in routing header → responder's Noise `read_message` fails (prologue mismatch).
- Integration (3-node): `connect_via(addr_b, &pub_c, nid_c)` on a 3-node chain A-B-C, B has direct sessions with both. Replaces current `test_mesh_handshake_via_relay` with the new plumbing but asserts the same externally-visible outcome.
- Integration (4-node, requires multi-hop routing): `connect_routed(&pub_d, nid_d)` on A-B-C-D with routes seeded on A (D→B) and B (D→C). Identical to today's `test_regression_handshake_relay_multi_hop_via_routing_table` but using the new path.
- All existing direct-connect tests pass unchanged.

## Scope

~80 lines added (mesh.rs dispatch + connect_via + prologue helper), ~500 lines deleted (handshake_relay.rs + its tests). Net reduction. One coordinated change: the `HANDSHAKE`-flag packet is now sometimes routed, so all nodes must be on the new dispatch code at rollout time. No wire format churn on the handshake payload itself — the Noise bytes and Net header are unchanged; only the outer routing wrapper is new for the relayed case.
