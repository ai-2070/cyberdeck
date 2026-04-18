# Automatic Rerouting

## Problem

When a peer dies, the `MeshNode` currently requires manual intervention to update the routing table. The test calls `router().remove_route()` then `router().add_route()` with the new next hop. In a real mesh, this should happen automatically: the failure detector marks the peer as failed, and the routing table updates itself to route through an alternate peer.

The pieces exist:

- `FailureDetector` tracks per-node heartbeats and transitions nodes through Healthy → Suspected → Failed → (recovered → Healthy)
- `FailureDetector` has `on_failure` and `on_recovery` callbacks
- `RoutingTable` has `add_route`, `remove_route`, `lookup`
- `ProximityGraph` has `path_to`, `find_best`, `find_k_best`, `all_nodes`

The missing piece: a component that watches the failure detector and updates the routing table.

## Design

```
FailureDetector
    │
    ├─ on_failure(node_id) ──→ ReroutePolicy
    │                              │
    │                              ├─ find alternate via ProximityGraph
    │                              ├─ update RoutingTable
    │                              └─ log reroute event
    │
    └─ on_recovery(node_id) ──→ ReroutePolicy
                                   │
                                   ├─ restore original route if better
                                   └─ log recovery event
```

### `ReroutePolicy`

```rust
pub struct ReroutePolicy {
    /// Routing table to update
    routing_table: Arc<RoutingTable>,
    /// Proximity graph for finding alternates
    proximity_graph: Arc<ProximityGraph>,
    /// Peer sessions (for resolving node_id → addr)
    peers: Arc<DashMap<SocketAddr, PeerInfo>>,
    /// Original routes (saved before reroute, for recovery)
    original_routes: DashMap<u64, RouteEntry>,
    /// Reroute history (for observability)
    reroute_count: AtomicU64,
}
```

### Failure callback

When `FailureDetector::on_failure(failed_node_id)` fires:

1. **Find all routes through the failed node.** Iterate the routing table. Any route whose `next_hop` resolves to the failed peer's address is affected.

2. **For each affected route, find an alternate.** Query the `ProximityGraph` for the destination node. `path_to(dest_id)` returns an alternate path. The first hop of that path is the new `next_hop`. If no path exists, try `find_best` with a capability filter matching the destination's requirements.

3. **Update the routing table.** Save the original route in `original_routes`, then call `add_route(dest_id, alternate_next_hop)`. The routing table overwrites atomically — the next packet uses the new route.

4. **Log.** Emit a `tracing::info!` with the failed node, affected routes, and chosen alternates.

### Recovery callback

When `FailureDetector::on_recovery(recovered_node_id)` fires:

1. **Check original routes.** If the recovered node was the original next hop for any route, compare its metric against the current alternate.

2. **Restore if better.** The original direct route typically has a lower metric (fewer hops) than the rerouted path. Restore it by calling `add_route` with the original entry.

3. **Clean up.** Remove from `original_routes`.

### Decision: restore original or keep alternate?

The simplest correct policy: always restore the original route on recovery. The original was chosen because it was the best path. The alternate was a fallback. When the original becomes available again, it's still the best path.

Edge case: the recovered node might be degraded (partially functional). The failure detector transitions to Healthy only after receiving a heartbeat, so by the time `on_recovery` fires, the node is confirmed alive. If it's flaky, the failure detector will trip again and reroute again. The circuit breaker prevents rapid oscillation.

## Integration into MeshNode

### Where it lives

```
src/adapter/net/
  reroute.rs    ← ReroutePolicy struct and logic
  mesh.rs       ← wires it into the failure detector callbacks
```

### How it's wired

In `MeshNode::new()`, after creating the failure detector:

```rust
let reroute_policy = Arc::new(ReroutePolicy::new(
    router.routing_table().clone(),
    proximity_graph.clone(),
    peers.clone(),
));

let rp = reroute_policy.clone();
let failure_detector = FailureDetector::with_config(config)
    .on_failure(move |node_id| rp.on_failure(node_id))
    .on_recovery(move |node_id| rp.on_recovery(node_id));
```

The callbacks are synchronous (`Fn(u64)` — no async). `ReroutePolicy::on_failure` and `on_recovery` only touch `DashMap` and `RoutingTable` — all lock-free operations. No allocations on the reroute path.

### ProximityGraph requirement

Currently, `MeshNode` does not have a `ProximityGraph`. It needs one. The graph is populated from:

1. **Direct peers.** When `connect()` or `accept()` completes, add the peer as a 1-hop node in the graph.

2. **Pingwave propagation.** The heartbeat loop should also broadcast `EnhancedPingwave` to all peers. When a peer receives a pingwave, it adds/updates the originating node in its graph and re-broadcasts (if TTL allows). This builds the multi-hop topology.

Without pingwave propagation, the proximity graph only knows about direct peers, so the only alternates are other direct peers. That's sufficient for a 3-node triangle but not for larger meshes. Pingwave integration is a separate feature that can be added after the basic reroute works.

## What changes, what doesn't

| Component | Changes? | Why |
|-----------|----------|-----|
| `FailureDetector` | No | Already has `on_failure`/`on_recovery` callbacks |
| `RoutingTable` | No | Already has `add_route`/`remove_route` |
| `ProximityGraph` | No | Already has `path_to`/`find_best` |
| `MeshNode` | Yes | Adds `ProximityGraph`, wires `ReroutePolicy` into failure detector |
| `ReroutePolicy` | **New** | ~100 lines: on_failure finds alternate, on_recovery restores |

## Implementation steps

### Step 1: `ReroutePolicy` (~100 lines)

- `on_failure(node_id)`: iterate routing table, find affected routes, find alternates from peers list (direct peers only — no proximity graph yet), update routes
- `on_recovery(node_id)`: restore original routes
- Unit tests: verify reroute and recovery with a mock routing table

### Step 2: Wire into `MeshNode` (~20 lines)

- Add `ReroutePolicy` field to `MeshNode`
- Wire callbacks in constructor
- Pass `peers` map so the policy can resolve node_id → addr

### Step 3: Three-node test

- `test_mesh_node_auto_reroute`: A sends to C via B. B dies. Wait for failure detection. A's reroute policy updates the route to C directly. Send more events — C receives them without any manual `add_route` call.
- `test_mesh_node_auto_reroute_recovery`: Same as above, but B comes back. Route restores to B (original path, lower metric).

### Step 4: Add `ProximityGraph` to `MeshNode` (future)

- Populate graph from `connect()`/`accept()` results
- Broadcast `EnhancedPingwave` in heartbeat loop
- Process incoming pingwaves in receive loop
- `ReroutePolicy` uses graph for multi-hop alternate selection

## Risk

**Low.** The reroute callback is synchronous, lock-free, and only touches existing data structures. The failure detector callbacks are already proven by the three-node tests. The routing table's `add_route` is an atomic `DashMap` insert. The worst case: a bad alternate is chosen and traffic drops. The next heartbeat cycle re-evaluates — the system self-corrects.

## Scope

Step 1-3 is the minimum: ~120 lines of new code, one new file, one modified file, two new tests. The reroute works with direct peers only (no proximity graph). Step 4 adds multi-hop awareness and is independent.
