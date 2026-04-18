# Multi-hop routing discovery

## Status

`RoutingTable` entries only exist for directly-connected peers today. They're installed by `MeshNode::connect` and `::accept` as a side effect of session setup — the routing table *caches* what the peers map already knows. There's no mechanism for a node to learn "I can reach X via next-hop Y" when X is not a direct peer.

This blocks spontaneous multi-hop for everything that uses `RoutingTable::lookup`: `send_routed`, the routed-packet forwarding path in `dispatch_packet`, and the rewritten `connect_via` from the handshake plan.

Pingwaves already propagate origin identity + hop count across the mesh. The `ProximityGraph` already records the resulting topology. The missing piece is a tiny glue layer that installs/refreshes routing-table entries as pingwaves arrive from non-direct origins.

## Goal

Given the current infrastructure (pingwaves that carry `origin_id` + `hop_count`, `ProximityGraph::on_pingwave` that records origin nodes and their `from: SocketAddr`), make `RoutingTable::lookup(origin)` return a working next-hop for any origin the local node has heard a pingwave from — not just direct peers.

Explicit non-goal: path optimality, routing convergence guarantees, or replacement of `ProximityGraph::path_to`. This plan is "learn *a* route to every node we've heard about," not "compute the optimal shortest-path tree." Path optimization is a later layer.

## Design

### Distance-vector-style install-on-receive

`ProximityGraph::on_pingwave(pw, from)` already does the work of classifying pingwaves (dedup, origin tracking, edge inference). Extend it with one callback: when it observes a new-or-updated origin that is NOT this node's own id AND NOT a direct peer, install a routing-table entry `origin_id → from.addr` with metric = `hop_count + 1`.

- `from` is the UDP source addr of the pingwave — the immediate upstream peer that forwarded it. That peer IS a direct session partner (we only receive pingwaves from direct peers), so `from.addr` is a valid next-hop.
- `hop_count` in the pingwave is "hops from origin to here", so it's a natural metric.
- `add_route_with_metric` already exists; use it so updated pingwaves with better metrics win.

### Direct peers take precedence

Direct peers install a route at metric `0` in `connect`/`accept`. The distance-vector path always sees a non-zero `hop_count`, so direct routes never get overwritten by indirect observations.

### Route expiry

Pingwaves are periodic. A route learned via pingwave should age out if no fresh pingwave refreshes it — otherwise a dead origin stays in the routing table forever, and packets to it get forwarded into a black hole.

The simplest scheme: give each `RouteEntry` a `last_refreshed: Instant` and treat a route as valid for `N × heartbeat_interval` after its last refresh. Direct peers are refreshed by the session's own heartbeat loop (the current code already tolerates this implicitly — the session stays registered).

For this plan: `RouteEntry` gains `last_refreshed`; the pingwave path updates it; `RoutingTable::lookup` returns `None` for stale entries; a periodic sweep (piggybacked on the existing heartbeat timer) removes routes whose `last_refreshed` is older than `3 × session_timeout`.

### Interaction with ReroutePolicy

`ReroutePolicy` today saves the original next-hop when a direct peer fails and swaps in an alternate. With pingwave-learned routes in the table, the same machinery works: when a direct peer dies, its learned multi-hop alternate (if any pingwave has seeded one) is already sitting in the table under a different origin and can be promoted. Mostly orthogonal to this plan; just don't break it.

## What gets added or changed

### 1. `RouteEntry::last_refreshed: Instant` (`route.rs`, ~5 lines)

Add the field. `add_route` / `add_route_with_metric` set it to `Instant::now()`. `RoutingTable` gets:

```rust
pub fn sweep_stale(&self, max_age: Duration) -> usize
```

Returns the number of entries removed. Called from the heartbeat loop.

### 2. `RoutingTable::lookup` staleness check (`route.rs`, ~10 lines)

Before returning a next-hop, check `last_refreshed`. Return `None` if older than the configured `max_route_age`. Configurable via `RouterConfig`; default `3 × session_timeout`.

### 3. `ProximityGraph` → `RoutingTable` bridge (`proximity.rs` or `mesh.rs`, ~30 lines)

`ProximityGraph::on_pingwave` currently returns `Option<EnhancedPingwave>` (for rebroadcast). Extend to also call back a closure/trait when it accepts a new pingwave from a non-self origin:

```rust
pub fn set_route_installer(&self, installer: impl Fn(u64 /*origin*/, SocketAddr /*from*/, u8 /*hop_count*/) + Send + Sync + 'static)
```

`MeshNode::new` wires this callback at setup:

```rust
let routing_table = self.router.routing_table().clone();
proximity_graph.set_route_installer(move |origin, from, hop_count| {
    routing_table.add_route_with_metric(origin, from, hop_count as u16 + 1);
});
```

Alternative placement: put the callback directly inside `MeshNode`'s pingwave handling (the mesh already sees every pingwave via `dispatch_packet`). Either works; prefer whichever keeps `ProximityGraph` agnostic of `RoutingTable`.

### 4. Periodic sweep in the heartbeat loop (`mesh.rs`, ~5 lines)

`spawn_heartbeat_loop` already fires on a timer. Add:

```rust
router.routing_table().sweep_stale(max_route_age);
```

per tick. Cheap — a scan of a DashMap that's normally small.

### 5. Don't overwrite direct-peer routes (`proximity.rs`, ~3 lines)

When the bridge callback fires, check `routing_table.lookup(origin)` first. If a direct route exists (metric 0), don't update — direct routes always win.

Actually cleaner: let `add_route_with_metric` do the check. If `existing.metric == 0 && new_metric > 0`, skip.

## Interaction with handshake

With this plan landed, `MeshNode::connect_routed(peer_pubkey, peer_node_id)` becomes useful: the routing table will answer `lookup(peer_node_id)` with the next-hop to the first relay, the handshake packet gets forwarded through the mesh, and the handshake completes without any manual route configuration — provided pingwaves have reached the initiator from the responder's direction (or vice versa) before the call. Typical propagation is one heartbeat interval per hop.

## Security considerations

Pingwaves are **not authenticated** today — they're raw UDP blobs with no PSK or signature. A malicious node on the mesh can forge a pingwave claiming to be origin X, and the route installer will happily point traffic for X at the attacker.

This is an existing problem independent of this plan (`ProximityGraph` already trusts pingwaves unconditionally), but widening the consumer surface to include the routing table makes the consequences worse: bogus pingwaves now influence where actual data packets go.

Mitigations out of scope for this plan, but worth naming:

1. **PSK-authenticate pingwaves.** Add a MAC over `(origin_id, seq, hop_count)` computed with the shared PSK. Foils passive forgery.
2. **Signed pingwaves.** Each node signs its pingwave with its static keypair; intermediates can't modify fields and forward. More expensive per packet.
3. **Prefer ProximityGraph's trust signals.** `ProximityGraph` already tracks `ProximityNode::last_seen` and health; the route installer could refuse routes for origins in an unknown or suspicious state.

Of these, (1) is the natural match: we already have PSK for session setup; reusing it for pingwave authenticity is one line of crypto and removes the passive-forgery attack.

## Tests

- Unit: `RoutingTable::add_route_with_metric` preserves a metric-0 (direct) entry when a metric>0 update arrives.
- Unit: `RoutingTable::sweep_stale` removes entries whose `last_refreshed` is older than the cutoff; leaves fresh ones alone.
- Integration (4-node): A↔B↔C↔D chain (only adjacent pairs have direct sessions). After pingwaves propagate, `a.router().routing_table().lookup(nid_d)` returns `addr_b` (or wherever pingwaves from D arrived at A from).
- Integration (same 4-node): after seeding routes via pingwave, `a.connect_routed(&pub_d, nid_d)` succeeds without any manual `add_route` calls. This is the test that demonstrates the handshake plan + this plan compose end-to-end.
- Integration: kill D; wait for routes to age out; confirm `lookup(nid_d)` returns `None`. (Alternative: confirm D's route is replaced by a fresh pingwave through a different path once the topology updates — but that's `ReroutePolicy`'s job and already tested.)

## Scope

~60 lines of code across `route.rs`, `proximity.rs`, and `mesh.rs`. No wire format changes — pingwaves already carry everything needed. Test additions ~80 lines. Total roughly 140 lines including coverage.

## Sequencing

This plan is **independent of** the handshake rewrite and can ship first, second, or in parallel. The handshake rewrite works for single-relay even without this plan; this plan delivers value on its own for `send_routed` reaching non-direct peers. They compose to deliver spontaneous multi-hop handshake establishment, but neither is blocked on the other.
