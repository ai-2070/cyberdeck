# MeshNode: Composing the Protocol Layers

> **Status:** Implemented. `MeshNode` lives in `src/adapter/net/mesh.rs` and is covered by `three_node_integration`'s 49 tests over real encrypted UDP. The gaps listed in the "Problem" section below (relay without decryption, auto-reroute, migration over transport, subnet-gateway enforcement on real packets, handshake relay) are now closed. This document is preserved for design rationale.

## Problem

`NetAdapter` is point-to-point (one peer, one encrypted session). `NetRouter` is a standalone packet forwarder. `FailureDetector`, `SubprotocolHandler`, `SubnetGateway`, and `CapabilityIndex` are independent components. There is no runtime that composes them into a multi-peer mesh node.

This means:
- A node can't relay packets between two peers
- A node can't reroute traffic when a peer dies
- Migration messages can't flow over the actual transport
- Subnet gateway enforcement doesn't happen on real packets

## What exists

```
NetAdapter           1:1 encrypted UDP session (Noise NKpsk0)
  ├─ PacketBuilder   zero-alloc packet construction
  ├─ PacketCipher    ChaCha20-Poly1305 encrypt/decrypt
  ├─ NetSession      session state, heartbeats, streams
  └─ process_packet  recv → parse → decrypt → queue events

NetRouter            routing table + fair scheduler
  ├─ RoutingTable    dest_id → next_hop lookup
  ├─ FairScheduler   per-stream fair queuing
  └─ route_packet    recv → parse routing header → forward or local

FailureDetector      heartbeat tracking → Healthy/Suspected/Failed
SubnetGateway        visibility enforcement on header fields
SubprotocolHandler   migration message dispatch
CapabilityIndex      capability-based target discovery
ProximityGraph       RTT-weighted neighbor awareness
```

Each component works correctly in isolation (proven by 881 tests). The missing piece is the **receive loop** that dispatches incoming packets to the right component.

## Design: MeshNode

```rust
pub struct MeshNode {
    /// This node's identity
    identity: EntityKeypair,
    node_id: u64,

    /// Single UDP socket (all peers share one socket)
    socket: Arc<Socket>,

    /// Per-peer encrypted sessions: peer_addr → NetSession
    sessions: DashMap<SocketAddr, Arc<NetSession>>,

    /// Router for multi-hop forwarding
    router: Arc<NetRouter>,

    /// Failure detector for all peers
    failure_detector: Arc<FailureDetector>,

    /// Inbound event queues (for poll_shard)
    inbound: InboundQueues,

    /// Subprotocol handler (migration, etc.)
    subprotocol_handler: Option<Arc<SubprotocolHandler>>,

    /// Subnet gateway (if this node is a gateway)
    gateway: Option<Arc<SubnetGateway>>,
}
```

### Key architectural decisions

**One socket, many peers.** The current `NetAdapter` binds a new socket per peer. `MeshNode` uses one socket for all peers. This is how real mesh nodes work — you listen on one port and multiplex by source address. The `sessions` map dispatches to the right `NetSession` based on the packet's source.

**Receive loop replaces `process_packet`.** Instead of `NetAdapter::process_packet` which assumes a single session, `MeshNode` has a receive loop that:

```
recv(packet, source_addr)
  │
  ├─ if handshake packet → handshake state machine
  │
  ├─ if heartbeat packet → failure_detector.heartbeat(node_id, addr)
  │
  ├─ lookup session:
  │   ├─ direct packets: by source_addr
  │   └─ routed packets: by inner session_id (source is the relay, not the sender)
  │
  ├─ check routing header
  │   ├─ if dest_id == self.node_id → LOCAL
  │   │   ├─ decrypt payload
  │   │   ├─ check subprotocol_id
  │   │   │   ├─ 0x0000: queue in inbound (events)
  │   │   │   ├─ 0x0500: subprotocol_handler (migration)
  │   │   │   └─ other: opaque forwarding
  │   │   └─ queue events for poll_shard
  │   │
  │   └─ if dest_id != self.node_id → FORWARD
  │       ├─ check subnet gateway (if configured)
  │       ├─ check TTL
  │       ├─ DO NOT decrypt (header-only routing)
  │       ├─ update hop_count, TTL
  │       └─ router.enqueue(packet, next_hop)
  │
  └─ router send loop → dequeue → socket.send_to(dest)
```

**Handshake on demand.** When a `MeshNode` wants to connect to a new peer, it initiates a Noise handshake over the shared socket. The handshake uses the existing `perform_handshake` logic but routes through the shared socket. After handshake, the `NetSession` is added to the `sessions` map.

**Adapter trait.** `MeshNode` implements the `Adapter` trait so it can back an `EventBus`. The `on_batch` method picks the right session based on routing (not a hardcoded peer), and `poll_shard` reads from the shared `inbound` queue.

## Implementation plan

### Phase 1: Shared socket + multi-session (smallest useful delta)

1. **`MeshNode::new(identity, bind_addr)`** — binds one socket, creates router and failure detector.

2. **`MeshNode::connect(peer_addr, peer_pubkey)`** — performs Noise handshake over the shared socket, stores the session in the `sessions` map. Can be called multiple times for different peers.

3. **`MeshNode::spawn_receiver()`** — single receive loop that:
   - Looks up session by source address
   - For local packets: decrypts and queues events
   - For forwarded packets: passes to router (no decryption)
   - For heartbeats: updates failure detector

4. **Adapter impl** — `on_batch` sends through the session whose routing table entry matches the destination. `poll_shard` reads from the shared inbound queue.

**Tests unlocked:** Relay through middle node (A→B→C with B forwarding). Relay tamper detection. TTL enforcement over encrypted packets.

### Phase 2: Failure detection + rerouting

5. **Integrate `FailureDetector` into heartbeat path.** Each heartbeat updates the detector. Periodic `check_all()` identifies failed peers.

6. **Reroute on failure.** When a peer is marked Failed, the routing table is updated to route through an alternate peer. The `ProximityGraph` guides the alternate selection.

**Tests unlocked:** Reroute on node failure. Circuit breaker over real sockets. Failure detection timing.

### Phase 3: Subprotocol dispatch

7. **Wire `SubprotocolHandler` into the receive loop.** Packets with `subprotocol_id == 0x0500` (migration) are dispatched to the migration handler. All other subprotocol IDs (causal `0x0400`, snapshots `0x0401`, unregistered, etc.) fall through to the standard event path or opaque forwarding.

8. **Migration message flow.** The `MigrationSubprotocolHandler` receives snapshot chunks, buffered events, cutover notifications over real UDP.

**Tests unlocked:** Migration over wire. Snapshot chunking over wire. Migration abort cleanup.

### Phase 4: Subnet gateway

9. **Add gateway check on forward path.** Before forwarding a non-local packet, consult the `SubnetGateway` for a forward/drop decision.

**Tests unlocked:** Gateway enforcement on real packets.

## File placement

```
src/adapter/net/
  mesh.rs        ← MeshNode struct, receive loop, Adapter impl
  mod.rs         ← add `pub mod mesh; pub use mesh::MeshNode;`
```

`MeshNode` composes existing components — it doesn't rewrite them. `NetAdapter` continues to exist for simple point-to-point use cases (SDKs, benchmarks, single-peer connections).

## What changes, what doesn't

| Component | Changes? | Why |
|-----------|----------|-----|
| `NetSession` | No | Already handles encrypt/decrypt per-session |
| `PacketBuilder` / `PacketPool` | No | Already handle zero-alloc packet construction |
| `NetRouter` | No | Already handles routing decisions |
| `FailureDetector` | No | Already handles heartbeat tracking |
| `SubnetGateway` | No | Already handles visibility decisions |
| `process_packet` | Extracted | Same logic, but dispatches by session instead of assuming one |
| `perform_handshake` | Adapted | Same Noise logic, but uses shared socket |
| `spawn_receiver` | Replaced | New multi-session receive loop |

## Risk

**Low.** The components are proven individually. `MeshNode` is composition, not rewrite. The receive loop is the only new code — everything else is calling existing methods on existing structs. The existing `NetAdapter` and all 881 tests are untouched.

## Scope

Phase 1 is the minimum viable mesh node. It's ~300 lines of new code: struct definition, `connect()`, receive loop, `Adapter` impl. It unlocks the relay/forwarding tests and proves the architecture. Phases 2-4 are incremental additions to the receive loop (~50-100 lines each).
