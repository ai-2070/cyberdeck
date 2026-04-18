# Pingwave-driven distance-vector routing — fill in the gaps

## Status

**Partial.** The seed of pingwave-driven route installation is already in `mesh.rs` dispatch: when node X receives a pingwave originated by Y, forwarded via direct peer Z, X calls `RoutingTable::add_route_with_metric(Y, next_hop=Z, metric=pw.hop_count + 2)`. The metric policy in `add_route_with_metric` preserves the lower-metric entry, so direct routes (metric 1) always beat pingwave-installed routes. Routes age out via `sweep_stale` on the heartbeat-loop tick. What's missing is the **topology graph side**, **loop / stability policy**, and **invalidation semantics** that turn this from "flooded hop-count advertisements" into a disciplined distance-vector installation.

This plan closes those gaps without rebuilding the pingwave wire format — everything ships as additional bookkeeping and policy over the existing 72-byte `EnhancedPingwave`.

## What already works

- **Pingwave emission** (`mesh.rs:spawn_heartbeat_loop`) — every `heartbeat_interval`, each node emits one `EnhancedPingwave` (72 bytes, unencrypted) to every connected peer.
- **Pingwave forwarding** (`mesh.rs:743-757`) — receivers re-broadcast to all peers except the sender; TTL decremented, `hop_count` saturating-incremented.
- **Route install on receipt** (`mesh.rs:735-741`) — `(origin, next_hop=source, metric=hop_count+2)` goes into the routing table; metric policy keeps the best.
- **Age-out** — `RoutingTable::sweep_stale` removes entries older than `max_route_age` (configured as `3 × session_timeout`). Called from the heartbeat loop.
- **Metric policy** — `add_route_with_metric` preserves a strictly-better existing metric; equal-or-better replaces. Direct route (`add_route` → metric 1) always wins.
- **Reactive reroute** — `ReroutePolicy::find_graph_alternate_for` queries `ProximityGraph::path_to()` on failure to pick an alternate per-destination.

## Gaps this plan fills

1. **`ProximityGraph::edges` is never populated.** The struct holds an `edges: DashMap<NodeId, Vec<NodeId>>` that `path_to()` BFS-walks, but nothing ever inserts into it. `path_to()` therefore returns `None` for every multi-hop destination today, and `ReroutePolicy::find_graph_alternate_for` falls through to the "any direct peer" fallback. The routing-table installer works; the graph-based lookup does not.

2. **No split horizon / loop prevention beyond TTL.** Pingwave re-broadcast fans out to every peer except the sender. In a partitioned-then-healed mesh, a pingwave can loop back through a convergent path and re-install a longer route than the one the node already has. The metric policy absorbs most of this (it won't replace a better route), but it doesn't stop a node from advertising a route *back toward the origin* — classic count-to-infinity kindling.

3. **No route-validity signal.** A pingwave that stops arriving is the only "route is gone" signal. If the intermediate hop (Z) is alive but the path Z→Y has silently broken (partition past Z), X keeps using the stale `(Y, via=Z)` route until `sweep_stale` fires `~3 × session_timeout` later. Peer-failure reroute doesn't help because Z itself is healthy.

4. **Metric is hop count only.** Pingwaves already carry `origin_timestamp_us`; we never use it to estimate path latency, so "fewer hops" always wins even if that path is slower.

5. **No integration story.** `ReroutePolicy` still calls `path_to()` reactively on failure. The DV-installed routes in `RoutingTable` and the graph-based lookup in `ProximityGraph` don't share a source of truth. Either the table is authoritative (and the graph is a side-channel), or the graph is authoritative (and the table is a cache) — the code implies the former but the reroute code acts as if it's the latter.

## Goals

- **Populate `ProximityGraph::edges`** from pingwave hops so `path_to()` returns real multi-hop paths.
- **Split-horizon** the re-broadcast: never forward a pingwave on the same link it arrived on *and* never advertise a route back to a node that would use us as next-hop for the same destination.
- **Detect stale indirect routes** with an explicit "origin still fresh?" check tied to the origin's own pingwave arrivals (not just next-hop peer liveness).
- **Latency-aware metric** (optional): combine hop count with measured one-way latency from `origin_timestamp_us` so a 2-hop 5 ms path beats a 1-hop 200 ms satellite link.
- **Make `RoutingTable` authoritative.** Graph is the topology derivation; table is what the fast-path routes off.

## Non-goals

- **Not OSPF / BGP.** No link-state flooding, no sequence-number-per-link routing DB, no path-vector attribute lists. DV + poison reverse is sufficient for the mesh sizes Net targets.
- **No wire-format changes.** Pingwave stays 72 bytes. Destination advertisements are *implicit* in pingwave propagation — the origin advertises itself; intermediate hops relay-install routes to the origin as a side effect. This is the existing shape, just made safer.
- **Not a secure routing protocol.** Pingwaves are unencrypted and unsigned today; a malicious peer can forge origin ids and inflate/deflate hop counts. That's a separate protocol concern (`PINGWAVE_AUTH_PLAN.md` if we ever need it). This plan assumes mutually-trusted participants or a PSK-gated mesh.
- **No geographic / latency-tier routing.** Metric is `hop_count` (+ optional latency estimate), not a multi-dimensional policy. Teams that need "prefer WiFi over cellular" build that on top.

## Design

### 1. Populate edges on pingwave receipt

When X receives a pingwave for origin Y via direct peer Z, X knows one concrete topology fact: **Z has a route to Y** (otherwise Z wouldn't be forwarding the pingwave). Insert the directed edge `Z → Y` into `ProximityGraph::edges` on every pingwave receipt.

X itself always has the edge `X → Z` (Z is a direct peer). Combined, X's local graph now has enough to BFS a 2-hop path `X → Z → Y`. For deeper chains, each intermediate's own pingwave contributes its outbound edge to the nodes that see it.

Edges are timestamped and aged out via a periodic sweep (same cadence as `sweep_stale` on the routing table).

### 2. Split horizon on re-broadcast

The current re-broadcast skips the sender. Tighten it with a per-origin "poison reverse" rule: if X installed `(Y, via=Z)`, X does **not** re-broadcast pingwaves for Y to Z. This prevents Z from learning "X can reach Y in N+1 hops" and installing a backward loop.

Implementation: during pingwave re-broadcast, consult `RoutingTable.lookup(origin)` — if the next-hop for `origin` equals a given peer's address, skip that peer.

### 3. Origin-freshness invalidation

Today's route `(Y, via=Z, metric=N)` survives as long as either (a) Z's direct session is alive, or (b) a pingwave for Y arrives within `max_route_age`. The second is sufficient; the first is misleading because Z being alive tells us nothing about the Z→Y path.

Simplify: **drop the peer-liveness leg**. A pingwave-installed route is alive iff it was refreshed by a pingwave for its origin within `max_route_age`. The failure detector still triggers `ReroutePolicy::on_failure` when a peer dies, because the routing-table entries whose `next_hop` points at that peer get rerouted — that path covers "Z died" independently of "the route through Z went stale."

Concrete: in `RoutingTable`, tag entries with a `RouteSource::{Direct, Pingwave}` enum. Direct routes age via session liveness (refreshed on every heartbeat pingwave we emit); pingwave routes age via origin pingwave arrivals. `sweep_stale` already checks `updated_at`; update-point is the only thing that changes per source.

### 4. Latency-aware metric (optional, v1.1)

`origin_timestamp_us` in the pingwave lets a receiver estimate path latency: `now_us − origin_timestamp_us`. This measurement is noisy (asymmetric clocks, queueing), so we smooth it with EWMA over successive pingwaves for the same origin and use `metric = hop_count × 100 + latency_bucket`, where `latency_bucket` is the EWMA latency in 1 ms increments, capped. Hop count dominates; latency breaks ties.

Optional in v1 — if the clock-sync assumption is too shaky, keep hop-only and revisit.

### 5. Routing table is authoritative; graph is a topology cache

`ReroutePolicy::find_graph_alternate_for(dest)` currently queries the graph. After this plan, the graph's `path_to()` works correctly, so this call returns a real path. But a reroute-time BFS over the graph is slower than a direct `RoutingTable::lookup_excluding(dest, failed_addr)` check — which is what we should do first, falling back to the graph only if no installed alternate exists.

Add `RoutingTable::lookup_alternate(dest, exclude_next_hop) -> Option<SocketAddr>` that returns the best route whose `next_hop != exclude_next_hop`. `ReroutePolicy::on_failure` consults this first; if None, fall back to the graph-based `path_to()`.

## Implementation steps

1. **`ProximityGraph::edges` insert + sweep.** On `on_pingwave`, insert `(source_peer_node_id → origin_node_id)` with a timestamp. Periodic sweep drops edges older than `max_route_age`. `path_to()` already reads `edges` — no algorithmic change.
2. **Split horizon in re-broadcast.** In `mesh.rs:743-757`, before emitting `fwd_bytes` to a peer, check `routing_table.lookup(origin_nid)`: if the next-hop matches the peer's address, skip it.
3. **`RouteSource` tag on `RouteEntry`.** Introduce the enum, thread through `add_route` (→ `Direct`) and `add_route_with_metric` (→ `Pingwave`). Direct heartbeats refresh only `Direct` entries; pingwave receipts refresh only `Pingwave` entries for the origin they carry. `sweep_stale` unchanged.
4. **Latency EWMA (deferred to v1.1).** Add `latency_ewma_us: AtomicU32` per origin in `ProximityGraph::nodes`; update on pingwave receipt. Plumb through to metric calculation in `mesh.rs:737`. Hop-only for v1 is fine.
5. **`RoutingTable::lookup_alternate(dest, exclude)`.** New accessor; prefers lowest-metric entry not equal to the excluded next-hop.
6. **`ReroutePolicy::on_failure` rework.** Try `lookup_alternate(dest, failed_addr)` first; fall back to `find_graph_alternate_for` only if the table has no alternate.
7. **Docs.** Extend `TRANSPORT.md` with a "Routing" subsection summarizing the DV story + the "RoutingTable authoritative" principle.
8. **Tests.**

## Tests

- **Unit (`route.rs`)** — `lookup_alternate` picks the lowest-metric alternate excluding a given next_hop; returns None if only the excluded route exists; regression: doesn't return a stale entry.
- **Unit (`proximity.rs`)** — edge insert on pingwave; edge sweep on age-out; `path_to` returns a real 3-hop path after enough pingwave traversal.
- **Integration** — 4-node chain A-B-C-D with pingwaves propagating. After convergence:
  - A's routing table has `(D, via=B, metric≥3)`, `(C, via=B, metric=2)`.
  - A's `path_to(D)` returns `[A, B, C, D]`.
  - Split horizon: A does NOT re-broadcast D's pingwave back to B.
  - Killing B: A's routes for C and D reroute through an alternate if one exists; if not, both entries expire via `sweep_stale`.
  - Origin staleness: if D stops emitting pingwaves but B is still alive, A's route to D expires; A's route to B does not.
- **Regression** — the old `ReroutePolicy::find_graph_alternate_for` "any direct peer" fallback path is still exercised for topology gaps (e.g., a node that's a direct peer but hasn't been heard from via pingwave).

## Risks and open questions

- **Count-to-infinity.** Split horizon alone doesn't fully prevent it — poison reverse would, but requires advertising a metric-∞ entry for the *opposite* direction, which needs wire-format space we don't want to spend. For v1, TTL + split horizon + metric age-out are the circuit breakers. If a partition stabilizes, routes eventually converge; if it oscillates fast, some queries hit stale routes until `sweep_stale`. Documented expected behavior.
- **Unbounded fan-out.** Pingwave flooding is O(peers²) per cycle in a fully-connected mesh. Current heartbeat is 5 s; scaling this down would be expensive. For larger meshes we'd want selective re-broadcast (already allowed by the `on_pingwave` returning `Option<EnhancedPingwave>` — the graph can decide to drop). Out of scope here; flagged.
- **Clock skew for latency metric.** `origin_timestamp_us` assumes rough clock sync. In a heterogeneous mesh with no NTP it's unreliable. Keep hop-only for v1; address in v1.1 with an NTP-fallback or one-way delay estimation.
- **Trust model.** Malicious peers can inflate/deflate hop counts or forge origin ids to hijack routes. Pingwave auth is a separate protocol. For v1, assume mutually-trusted participants (the PSK gates mesh entry). Flag as known pre-condition.
- **Interaction with subnets.** Pingwave propagation across subnet boundaries respects `ChannelConfig::visibility` — wait, pingwaves aren't channel traffic. Confirm the subnet gateway doesn't block them; if it does, intentional or accidental is a design question.

## Summary

This plan doesn't invent a DV protocol — it finishes the one that's already half-present. `add_route_with_metric` + pingwave-driven install were the hard bits; the finish work is populating the graph's edges, adding split horizon, tagging route source for proper invalidation, and letting the routing table be the authoritative path source rather than delegating to on-demand graph BFS. ~250 LoC of changes + ~150 LoC of tests. v1.1 folds in latency-aware metric once we're confident in the hop-only shape.
