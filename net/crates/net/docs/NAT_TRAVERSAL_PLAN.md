# NAT Traversal Plan

Give nodes behind NAT a path to direct peer connectivity — reflexive-address discovery, NAT-type classification, hole-punch rendezvous, opportunistic port mapping — while keeping the existing mesh-relay fallback as the safety net for the cases direct connectivity can't solve. Mesh-native throughout: no external STUN / TURN servers, no WebRTC ICE, no third-party signalling.

## Context

The mesh already has two load-bearing pieces that a NAT-traversal design can lean on:

- **Routed handshakes.** A node with no direct UDP path to its peer completes a full Noise NKpsk0 handshake through already-connected relays (`mesh.rs::handle_routed_handshake`). This is the equivalent of TURN-through-the-mesh for the unreachable case, and it already works multi-hop. We never need to "fall back to TURN" because we're always implicitly on it.
- **Relay forwarding without decryption.** Forwarders route encrypted bytes end-to-end. Adding more relay-assisted primitives (rendezvous, reflex echo) doesn't erode the security posture — relays still see only routing headers.

The mesh also has dormant infrastructure for NAT awareness:

- `adapter::net::behavior::metadata::NatType` — five-way enum (`None | FullCone | RestrictedCone | PortRestrictedCone | Symmetric | Unknown`) with a `difficulty()` score and a `can_connect_direct()` helper. Exists; is not currently populated.
- `NodeMetadata.nat_type` field — storage exists; no writer ever updates it.

Nothing in the current codebase actively *detects* NAT type, discovers reflexive addresses, or attempts hole punching. Every connection between NATed peers today rides the routed-handshake fallback for its entire lifetime. That works, but:

1. Every packet traverses a relay, adding 1+ RTTs to every request.
2. Relays pay the forwarding cost for traffic that, with a punched path, wouldn't touch them.
3. Bandwidth-heavy daemons (inference streams, file transfer) concentrate load on whichever relay is topologically between the endpoints.

Direct paths when they're available would cut both latency and relay load, with relayed fallback staying as the catch-all.

## Goals

- Discover each node's own reflexive (public) address and classify its NAT type on first startup + periodically thereafter.
- Announce NAT type so peers can decide whether to attempt direct connection vs. skip to routed-handshake fallback.
- Hole-punch between two NATed nodes via a mutually-connected relay acting as rendezvous coordinator.
- Opportunistically request port mappings via UPnP-IGD / NAT-PMP / PCP when the gateway supports it, lifting the node to `NatType::None`.
- Route preference for relay-capable peers when no direct path exists, without mandating that any given node relay (autonomy stays intact).
- Parity across all four bindings (core, Node, Python, Go) at the SDK-visible layer (observable `nat_type`, explicit `attempt_direct_connect(peer)` API, stats for punch attempts / successes).

## Non-goals

- **External STUN / TURN servers.** No third-party dependencies for address discovery or relay fallback. The mesh is the STUN server and the TURN server.
- **WebRTC ICE.** No Candidate Pair tables, no priority algebra, no SDP. The proximity graph already answers "which path works"; we just need to *discover* that a direct path exists, and the graph does the rest.
- **Mandatory relaying.** Every node decides whether to serve as a relay through the same autonomy envelope that gates rate limits. A `relay-capable` capability tag is opt-in.
- **IPv6-only networks as a special case.** IPv6 without NAT is already `NatType::None`; the plan applies as-is. Dual-stack nodes probe both families.
- **Punching symmetric × symmetric.** Two symmetric NATs can't hole-punch reliably. These pairs stay on the routed-handshake path forever — we don't waste coordination budget trying.
- **Mobile carrier NAT churn compensation.** CGNAT + mobile networks rebind ports aggressively. The plan doesn't attempt to track rebinds mid-flow; a re-probe after connection loss is sufficient.

---

## Design decisions

### 1. Mesh-native reflex discovery, not STUN

The classic STUN protocol is a simple request/response that echoes the observer's source address. We implement the same semantics as a mesh subprotocol — any peer can answer a reflex probe with "I saw you at `ip:port`." Two+ probes to different peers are enough to detect symmetric NAT (the observed source port differs per destination).

**Decision:** `SUBPROTOCOL_REFLEX` (pick a free id in the 0x0D00 block — see *Subprotocol ID assignment* below). Request: empty body (source comes from the UDP header). Response: 18 bytes (`family: u8` + `addr: [u8; 16]` (IPv4 zero-padded into the low bytes) + `port: u16`).

**Alternative considered:** piggyback reflex on the existing heartbeat / pingwave. Rejected — heartbeats are peer-to-peer on an already-established session, so the source address they'd report is the session's cached one, not a fresh observation from another vantage point. Reflex needs unsolicited probes from the node's perspective.

### 2. NAT type classification via two-probe comparison

Classic STUN-style NAT classification (the NAT Behavior Discovery RFC 5780 version) needs two public IP addresses on the server side to detect each cone type. We don't have that — all our peers have single addresses from our perspective. What we *can* cheaply determine is symmetric vs. non-symmetric, which is the classification that actually matters for punching decisions:

- Probe peer A → observe reflexive address R_A.
- Probe peer B → observe reflexive address R_B.
- If R_A.port == R_B.port → non-symmetric (some cone variant, punching is likely).
- If R_A.port != R_B.port → symmetric (punching is unlikely; skip to relay).

**Decision:** collapse the classification to `{Open, Cone, Symmetric, Unknown}` at the wire level. `NatType`'s five-way enum stays as-is internally for richer reasoning later, but wire announcement uses the collapsed form.

**Alternative considered:** full RFC-5780 behavior-discovery with a dedicated test server. Rejected — the classification cost (multiple peers, multiple probes) isn't justified by the extra granularity for our use case.

### 3. Piggyback NAT type on the capability broadcast

`CapabilitySet` already has a tag set (`add_tag("gpu")`, `add_tag("prod")`, etc.). NAT type fits naturally as reserved tags: `nat:open`, `nat:cone`, `nat:symmetric`, `nat:unknown`. Placement and subnet derivation don't need NAT type; peer selection for hole-punch initiation does.

**Decision:** reserved tags on `CapabilitySet`. Nodes set exactly one `nat:*` tag; the tag is overwritten on re-classification. Tag syntax avoids carving out a dedicated subprotocol for a single 2-bit value.

**Alternative considered:** dedicated `SUBPROTOCOL_NAT_META`. Rejected — the propagation path is identical to capability broadcast (same TTL, same dedup, same signing), so duplicating the subprotocol infrastructure just to avoid tag reservation adds surface area for no gain.

**Tradeoff:** hitches NAT-type propagation to the capability broadcast cadence (default 10 s min-interval). For a NAT type that changes (mobile network reassignment), peers see the new tag on the next cap announcement. Acceptable because hole-punch decisions cache NAT type for longer than 10 s anyway.

### 4. Rendezvous-coordinated simultaneous open, not cold punching

Hole punching requires both endpoints to send a packet outbound "at the same time" so each NAT's connection-tracking table has an entry for the peer's address before the peer's packet arrives. A coordinator (any mutually-connected peer) exchanges addresses and nominates a target wall-clock instant.

**Decision:** `SUBPROTOCOL_RENDEZVOUS`. Three-message dance:

1. A → R: `PunchRequest { target: B_node_id }`.
2. R → B: `PunchIntroduce { peer: A_node_id, peer_reflex: A_reflex, fire_at: ts }`.
   R → A: `PunchIntroduce { peer: B_node_id, peer_reflex: B_reflex, fire_at: ts }`.
3. At `fire_at`, A and B send 3 keep-alive packets to each other's reflexive address (spaced 100 / 250 / 500 ms to cover clock skew).
4. Whichever side sees inbound first sends back a `PunchAck`; on ack, the Noise handshake continues over the punched path.
5. If no `PunchAck` within `fire_at + 5 s`, both sides give up and fall back to routed-handshake.

Clock skew tolerance: `fire_at` is ~500 ms in the future; NTP-level clock accuracy is sufficient. The 3-packet keep-alive train covers up to ~1 s of skew.

**Alternative considered:** Nostr-style relay-initiated open (R sends both sides the address and lets them each reach out at their own pace). Rejected — without synchronized firing, the first packet typically hits before the other side has opened a hole, and each side's retry loop has to discover the right retransmission cadence. The synchronized approach converges in one RTT if it converges at all.

### 5. Port mapping upcalls are best-effort side quests

UPnP-IGD, NAT-PMP, and PCP let a host ask its gateway to install a port forward. When it works, the host effectively becomes `NatType::None` for the duration of the mapping. When it doesn't (disabled on the router, no gateway on the path, firewall in the way), we fall back to stage 1–3 behavior.

**Decision:** opt-in at mesh-build time via `MeshBuilder::try_port_mapping(true)` (default off). On startup the node probes NAT-PMP first (short timeout, 1 s), then UPnP-IGD (2 s), skipping if the first succeeds. Mapping TTL renewal runs as a background task on a 60 s interval.

**Alternative considered:** on by default. Rejected — port mapping is a noticeable side effect (modifies external state on the user's router), and some environments actively forbid it (corporate LAN, public Wi-Fi). Opt-in matches the "every node enforces its own rules" invariant.

### 6. Relay-preference routing, not relay-forced routing

When A can't reach B directly and the existing `RoutingTable::lookup` returns multiple candidates, prefer ones that advertise `nat:open` or the `relay-capable` tag. This is a soft preference — if no relay-capable path exists, routing falls through to whatever the proximity graph says.

**Decision:** add an optional `prefer_relay_tag: Option<&str>` to `RoutingTable::lookup` paths that matter for unreachable pairs. Default behavior unchanged; hole-punch-failure handlers opt in.

**Alternative considered:** hard routing rule that forces all unreachable-pair traffic through declared relays. Rejected — violates node autonomy (a relay can be overloaded, withdraw its `relay-capable` tag, or simply refuse to forward) and would require mesh-wide coordination on which relays to use.

---

## Stage 0 — Scaffolding

New module: `adapter::net::traversal`. Single parent for all NAT-traversal surface so future growth (IPv6-specific heuristics, WebRTC-DataChannel bridge, etc.) has a clear home.

```
adapter/net/traversal/
├── mod.rs          — pub use; SUBPROTOCOL_* constants
├── reflex.rs       — reflex probe sub-protocol handler
├── classify.rs     — NatType classification state machine
├── rendezvous.rs   — hole-punch coordinator
├── portmap.rs      — UPnP / NAT-PMP / PCP client (behind feature)
└── config.rs       — TraversalConfig (probe cadence, timeouts, ...)
```

Subprotocol ID assignment (next free block is `0x0D00`):

| ID       | Name                      |
|----------|---------------------------|
| `0x0D00` | `SUBPROTOCOL_REFLEX`      |
| `0x0D01` | `SUBPROTOCOL_RENDEZVOUS`  |
| `0x0D02` | `SUBPROTOCOL_PORTMAP_META` *(optional, stage 4 only if we decide to announce mapping TTL over the wire)* |

Feature flags added to `net/crates/net/Cargo.toml`:

```toml
[features]
nat-traversal = ["net"]           # reflex + classify + rendezvous (stages 1–3)
port-mapping = ["nat-traversal", "dep:igd-next", "dep:rust-natpmp"]  # stage 4
```

Keeps the core `net` feature minimal — consumers who don't need NAT traversal (LAN-only testbeds, fully-public nodes) don't pay compile / link cost.

---

## Stage 1 — Reflex probe subprotocol

### Wire format

```rust
// SUBPROTOCOL_REFLEX = 0x0D00
// Request body: empty (the source address is the echo target).
// Response body: 18 bytes.

#[repr(C, packed)]
pub struct ReflexResponse {
    pub family: u8,      // 4 or 6
    pub addr: [u8; 16],  // IPv4 zero-padded into the low 4 bytes
    pub port: u16,       // network-byte-order
}
```

Requests ride as regular mesh packets; the response is unicast back to the requester using the source address of the request (not the requester's claimed `node_id` → address mapping, which would defeat the purpose).

### Handler

`ReflexHandler::process_inbound(src_sockaddr, payload)` — one-liner: marshal the src socket address into a `ReflexResponse` and send it back. Stateless.

### Client

`MeshNode::probe_reflex(peer_node_id: u64) -> impl Future<Output = Result<SocketAddr, TraversalError>>` — sends one reflex request to the peer, awaits the response (timeout 3 s), returns the observed address.

### Exit criteria

- `probe_reflex(peer)` returns the actual public `ip:port` of the requesting node when run through a real NAT, and `ip:port` == bind address when no NAT is present.
- Two-node test: node A runs behind a deterministic NAT simulator (or on a different interface), probes node B, gets back its NAT-rewritten address.
- The new subprotocol is idle in steady state — only fires on explicit `probe_reflex` calls.

---

## Stage 2 — NAT type classification

### State machine

```rust
pub enum NatType {
    Open,       // reflexive == bind (no NAT) OR port-mapping installed
    Cone,       // reflexive.port consistent across different destinations
    Symmetric,  // reflexive.port varies per destination
    Unknown,    // classification inconclusive / not run
}

pub struct ClassifyFsm {
    probes: Vec<(NodeId, SocketAddr)>,  // at most 3 probes held
    result: Option<NatType>,
}

impl ClassifyFsm {
    fn observe(&mut self, peer: NodeId, reflex: SocketAddr);
    fn classify(&self) -> NatType;
}
```

Classification rule:

- If `bind_addr == reflex_addr` for any probe → `Open`.
- Else if all observed ports match → `Cone`.
- Else → `Symmetric`.
- If fewer than 2 probes → `Unknown`.

### Trigger

Classification runs on `MeshNode::start()`:

1. Pick 2 random peers from the handshake table (skip if < 2).
2. Fire `probe_reflex` to each in parallel; wait up to 5 s total.
3. Feed results into `ClassifyFsm`, write result to `NodeMetadata.nat_type`.
4. Add one of `nat:open` / `nat:cone` / `nat:symmetric` / `nat:unknown` to the local `CapabilitySet`.
5. Trigger a fresh `announce_capabilities` so peers see the new tag.

Re-classification triggers:

- Capability re-announce cadence (default every 10 s min-interval); if the cached reflex address from either anchor peer differs from the last observation, re-classify.
- Explicit `mesh.reclassify_nat()` on demand (useful after mobile network transitions).

### Exit criteria

- `mesh.node_metadata().nat_type` populates within 5 s of `start()` when ≥ 2 peers are handshaken.
- Capability announcements after start carry exactly one `nat:*` tag.
- `find_peers(CapabilityFilter::new().require_tag("nat:open"))` returns relay-capable candidates.

---

## Stage 3 — Hole-punch rendezvous

### Wire format

```rust
// SUBPROTOCOL_RENDEZVOUS = 0x0D01
// Request types (discriminator byte + body):

#[repr(u8)]
pub enum RendezvousMsg {
    PunchRequest(PunchRequest)     = 0x01,
    PunchIntroduce(PunchIntroduce) = 0x02,
    PunchAck(PunchAck)             = 0x03,
}

pub struct PunchRequest {
    pub target: u64,          // peer the requester wants to punch to
    pub self_reflex: SocketAddr,  // requester's currently-believed reflexive address
}

pub struct PunchIntroduce {
    pub peer: u64,
    pub peer_reflex: SocketAddr,
    pub fire_at_unix_millis: u64,
}

pub struct PunchAck {
    pub peer: u64,
    pub punch_id: u32,  // request correlation; echoed from `PunchRequest`
}
```

### Coordinator (R's role)

On `PunchRequest { target, self_reflex }` from A:

1. Look up target's reflexive address (must be in R's observations — either from capability announcements carrying a `reflex:<addr>` hint, or from R's own past `PunchIntroduce` to this pair).
2. If reflex unknown, reject with a typed error; caller falls back to routed-handshake.
3. Pick `fire_at = now() + 500 ms`.
4. Send `PunchIntroduce` to both A and B with the other's reflex and the shared `fire_at`.

### Endpoints (A's and B's role)

On receipt of `PunchIntroduce { peer, peer_reflex, fire_at }`:

1. Schedule 3 keep-alive sends to `peer_reflex` at `fire_at`, `fire_at + 100 ms`, `fire_at + 250 ms`.
2. Arm a 5 s timer: if no inbound packet from `peer_reflex` arrives, declare punch failed.
3. On first inbound packet from `peer_reflex`, send `PunchAck` via routed path (not the punched one — we don't know yet if it's reliable) and begin Noise handshake on the punched path.

### SDK surface

```rust
impl MeshNode {
    /// Attempt a direct connection via rendezvous punch. Falls back
    /// to routed-handshake on failure.
    pub async fn connect_direct(&self, peer: NodeId) -> Result<(), MeshError>;
}
```

Default behavior for an ordinary `connect()`: attempt punch if *both* sides advertise `nat:cone` or `nat:open`; skip punch entirely if either side is `nat:symmetric` (go straight to routed-handshake).

### Exit criteria

- Three-node integration test: A behind NAT1, B behind NAT2, R reachable by both. `A.connect_direct(B)` completes. Inspect the resulting session: `peer_addr()` is the punched socket, not the relay's.
- Symmetric×symmetric test: `connect_direct` short-circuits to routed-handshake without attempting a punch.
- Punch-failure test: simulate a dropped keep-alive; assert the 5 s timer fires and routed-handshake takes over.

---

## Stage 4 — Port mapping (UPnP / NAT-PMP / PCP)

Gated behind the `port-mapping` cargo feature. Adds two dependencies:

- `igd-next` — UPnP-IGD control point.
- `rust-natpmp` — NAT-PMP + PCP (they share a wire format).

### Behavior

`PortMapper` is a tokio task spawned by `MeshNode::start()` when `MeshBuilder::try_port_mapping(true)` is set:

1. Read the default gateway address from the routing table.
2. Fire NAT-PMP probe (1 s timeout). On success: request a mapping for the mesh's bind port, TTL 3600 s.
3. On NAT-PMP failure: fire UPnP SSDP probe (2 s timeout). On success: issue `AddPortMapping` for the mesh's bind port.
4. On success: record the external `ip:port`; call `mesh.set_reflex_override(external)` which forces `NatType::Open` and caches the mapping for reflex responses.
5. Background renewal task: every 30 min, re-issue the mapping (both protocols renew on re-request).
6. On mesh shutdown: `DeletePortMapping` if the mapping is still alive.

### Config

```rust
impl MeshBuilder {
    /// Enable UPnP-IGD / NAT-PMP port mapping at startup. Off by
    /// default because it modifies external state (the user's
    /// router) and some environments forbid it.
    pub fn try_port_mapping(self, enabled: bool) -> Self;

    /// Optional hard override — skip auto-probe, use this mapping.
    /// Useful for port-forwarded servers where the mapping is
    /// manually configured.
    pub fn reflex_override(self, external: SocketAddr) -> Self;
}
```

### Exit criteria

- On a LAN with a UPnP-enabled router, startup completes with `NatType::Open` and the mesh's `reflex_addr()` matches the router's external IP.
- On a LAN with UPnP disabled, startup continues (doesn't hang) and falls through to stage 2 classification.
- Graceful shutdown removes the mapping from the router's table.

---

## Stage 5 — SDK + binding surface

Symmetric across Rust, TS, Python, Go. Each binding exposes:

- `mesh.nat_type() -> "open" | "cone" | "symmetric" | "unknown"` (getter).
- `mesh.reflex_addr() -> Option<String>` (the observed / mapped public address).
- `mesh.probe_reflex(peer_node_id) -> Promise<string>` (test/debugging).
- `mesh.connect_direct(peer_node_id)` (non-default; ordinary `connect()` picks punch vs. routed automatically).
- `mesh.traversal_stats() -> { punches_attempted, punches_succeeded, punches_failed, relay_fallbacks, port_mapping_active }`.

### Error surface

New typed error variants on the mesh-error family:

```rust
pub enum TraversalError {
    ReflexTimeout,
    RendezvousNoRelay,
    RendezvousRejected,
    PunchFailed { reason: PunchFailureReason },
    PortMapUnavailable,
    Unsupported,  // peer doesn't advertise nat-traversal capability
}
```

Kind vocabulary for cross-binding parity (TS / Python / Go map to classes with a `kind` discriminator, same pattern as `MigrationError` and `GroupError`):

| Kind                     | Meaning                                                      |
|--------------------------|--------------------------------------------------------------|
| `reflex-timeout`         | reflex probe didn't complete in time                         |
| `rendezvous-no-relay`    | no mutually-connected relay found                            |
| `rendezvous-rejected`    | relay refused to coordinate (rate-limit / unknown target)    |
| `punch-failed`           | keep-alive train didn't establish a path                     |
| `port-map-unavailable`   | UPnP/NAT-PMP/PCP all failed                                  |
| `unsupported`            | peer doesn't advertise traversal capability                  |

### Exit criteria

- `nat_type()` returns a stable classification on all four bindings within 5 s of a handshaken mesh.
- `connect_direct(peer)` resolves the punched path where possible and falls back cleanly otherwise.
- Stats reflect real attempts — a test that forces a cone↔cone punch sees `punches_succeeded > 0`; a test behind symmetric NAT sees `relay_fallbacks > 0` and `punches_attempted == 0`.

---

## Critical files

### Stages 1–3 (core Rust)

- `adapter/net/traversal/{reflex,classify,rendezvous,config}.rs` — new module.
- `adapter/net/subprotocol/mod.rs` — register `SUBPROTOCOL_REFLEX` / `SUBPROTOCOL_RENDEZVOUS` dispatchers.
- `adapter/net/mesh.rs` — plumb classification trigger into `start()`; extend `connect()` to consider direct-punch first.
- `adapter/net/behavior/capability.rs` — reserve the `nat:*` tag namespace; document it.
- `adapter/net/behavior/metadata.rs` — wire up the `NatType` writer path (currently an orphaned field).

### Stage 4 (port mapping)

- `adapter/net/traversal/portmap.rs` — UPnP / NAT-PMP / PCP client.
- `Cargo.toml` — `port-mapping` feature + deps (`igd-next`, `rust-natpmp`).
- `adapter/net/mesh.rs` — `MeshBuilder::try_port_mapping`, `MeshBuilder::reflex_override`, spawn `PortMapper` task.

### Stage 5 (SDK + bindings)

- `sdk/src/mesh.rs` — `Mesh::nat_type()` / `reflex_addr()` / `connect_direct()` / `traversal_stats()` / `probe_reflex()`.
- `sdk/src/error.rs` — `TraversalError` variants.
- `bindings/node/src/lib.rs` — NAPI exports + typed `TraversalError` class.
- `bindings/python/src/lib.rs` — PyO3 exports + `TraversalError` exception class.
- `bindings/go/net/traversal.go` + `bindings/go/compute-ffi` or a new `bindings/go/traversal-ffi` crate — extern "C" surface + Go wrappers.
- `sdk-ts/src/mesh.ts` / `sdk-py/` wrappers as appropriate.

---

## Open questions

### Rendezvous relay selection

How does A pick which peer to use as rendezvous R? The first mutually-connected peer is the simplest rule, but may bias toward a single overloaded node. A random-two-choices pick across mutually-connected peers advertising `relay-capable` is probably the right default. **Open for decision:** if no `relay-capable` peer is mutually connected, do we (a) fail, (b) use any mutually-connected peer, or (c) fall through to routed-handshake immediately?

### Reflex address in capability announcements

Should peers advertise their observed reflex address so a fresh joiner can pre-populate its routing table without needing to probe? **Tradeoff:** reduces probe traffic + speeds up initial classification; leaks slightly more address info (though a peer already knows its own reflex from the handshake). Tentatively yes — add `reflex: SocketAddr` to `CapabilityAnnouncement`, signed with the identity keypair.

### Symmetric × cone

Two cone NATs punch reliably. A symmetric × cone pair *can* sometimes punch if the cone side opens first and the symmetric side's outbound port happens to match — but it's flaky. **Decision needed:** attempt with retries, or skip to relay? Recommend: attempt twice (since the cone side's outbound port is deterministic), skip after.

### Punch packet-loss resilience

3 keep-alives spaced 100/250/500 ms. On a lossy link, all 3 might drop. **Decision needed:** retry the punch once before giving up, or let the caller retry? Recommend single-shot — upper layers already handle connection-establishment retry.

### IPv6 hole punching

IPv6 usually doesn't need punching (most IPv6 stacks are `Open`), but some CGNAT deployments do apply NAT64 / 464XLAT. Stage 2 classification handles this correctly via the usual probe logic; stage 3 rendezvous works the same over IPv6. Worth an explicit dual-stack test in stage 3 exit criteria.

### Port-mapping lease on crash

If a node crashes after installing a UPnP mapping, the mapping leaks until its TTL expires (we request 3600 s). **Decision needed:** aggressive TTL (60 s) with frequent renewal, or let leaks decay naturally? Recommend 3600 s — renewal traffic is load on the router's control plane.

### Relay autonomy vs. rendezvous responsibility

A node advertises `relay-capable` to volunteer as a data relay. Is the same opt-in sufficient to volunteer as a rendezvous coordinator, or should rendezvous get its own tag (`rendezvous-capable`)? Data relaying is a per-packet ongoing cost; rendezvous is a handful of control packets per introduction. **Tentatively:** one tag covers both — the relay's autonomy envelope can rate-limit rendezvous separately.

### Classification churn on mobile networks

A node on a mobile carrier may shift NAT type mid-session. The capability broadcast cadence (10 s min-interval) will eventually propagate the new tag, but active hole-punch coordinators may have stale info. **Worth deciding:** auto-reclassify on every handshake failure, or only on explicit trigger? Recommend explicit — avoid classification thrash.

---

## Rough estimates

| Stage | Surface        | Complexity     | Estimate |
|-------|----------------|----------------|----------|
| 1     | Reflex probe   | Small          | ~1 day   |
| 2     | Classification | Small          | ~1 day   |
| 3     | Rendezvous + punch | Medium–large (state machine, integration test harness) | ~3–4 days |
| 4     | Port mapping   | Medium (two external protocols + renewal task) | ~2 days  |
| 5     | SDK + 4 bindings | Medium (same typed-error pattern across bindings) | ~2–3 days |

Total: ~9–11 days serial. Parallelizable across people for stages 3/4/5.

---

## Dependencies

- `igd-next` (≈150 KB, MIT licensed, actively maintained) — UPnP-IGD.
- `rust-natpmp` (≈50 KB, MIT, sparse maintenance but stable wire format) — NAT-PMP + PCP.

Both feature-gated under `port-mapping`. Stages 1–3 have no new deps.

---

## Out of scope (for this plan)

- TURN-over-TLS / TURN-over-TCP fallback for DPI'd networks (the mesh's own UDP transport is assumed).
- Browser-compatible WebRTC bridge (needs separate ICE / DTLS / SRTP plumbing — different problem).
- Persistent hole-punch state across mesh restarts. The node re-probes and re-classifies on every startup; no disk state.
- Reflex-address signing. The reflex response carries the observer's signature on nothing today; an attacker in the forwarding path could rewrite it. A future extension can add a signed observation if we find a threat model that demands it — for now, the mesh's own identity authentication means a lie only lasts until the punch fails.
