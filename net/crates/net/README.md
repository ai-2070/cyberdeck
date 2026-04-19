# Net

High-performance encrypted mesh runtime.

For the design philosophy, architecture rationale, and benchmarks, see the [project README](../../README.md).

## Contents

- [Key Concepts](#key-concepts)
- [Stack](#stack)
- [Architecture](#architecture)
- [Net Header](#net-header-64-bytes-cache-line-aligned)
- [Performance](#performance)
- [Capabilities](#capabilities)
- [Proximity & Discovery](#proximity--discovery)
- [Subnets](#subnets)
- [Channels](#channels)
- [Daemons](#daemons)
- [Safety & Autonomy](#safety--autonomy)
- [Module Map](#module-map)
- [Adapters](#adapters)
- [SDKs](#sdks)
- [Features](#features)
- [Building](#building)
- [Tests](#tests)
- [Benchmarks](#benchmarks)
- [Test Architecture](#test-architecture)
- [Subprotocol ID Space](#subprotocol-id-space)
- [License](#license)

## Key Concepts

**Identity is cryptographic.** Every node has an ed25519 keypair. The public key IS the identity. `origin_hash` (truncated BLAKE2s) is stamped on every outgoing packet. Permission tokens are ed25519-signed, delegatable, and expirable.

**Channels are named and policy-bearing.** Hierarchical names like `sensors/lidar/front`. Access control via capability filters (does this node have a GPU? the right tool? the right tag?) combined with permission tokens. Authorization cached in a bloom filter for <10ns per-packet checks.

**Behavior is declarative.** Nodes announce hardware/software capabilities, expose API schemas, and publish metadata. A rule engine enforces device autonomy policies. Load balancing, proximity-aware routing, and safety envelopes operate on this semantic layer. Distributed context propagation enables cross-node tracing.

**Subnets are hierarchical.** 4-level encoding (region/fleet/vehicle/subsystem) in 4 bytes. Gateways enforce channel visibility at subnet boundaries. Label-based assignment from capability tags.

**State is causal.** Every event carries a 24-byte `CausalLink`: origin, sequence, parent hash, compressed horizon. The chain IS the entity's identity. If the chain breaks, a new entity forks with documented lineage.

**Compute migrates.** The `MeshDaemon` trait defines event processors. The runtime handles causal chain production, horizon tracking, and snapshot packaging. Migration is a 6-phase state machine preserving chain continuity across nodes.

**Compute replicates.** A `ReplicaGroup` manages N copies of a daemon as a logical unit. Each replica has its own identity (derived deterministically from a group seed) and its own causal chain. The group load-balances events across replicas, tracks group-level health, spreads placement across failure domains, and auto-replaces failed replicas without migration — stateless re-spawn with the same deterministic identity.

**Subprotocols are extensible.** `subprotocol_id: u16` in every header. Formal registry with version negotiation. Unknown subprotocols are forwarded opaquely. Vendor protocols get IDs in `0x1000..0xEFFF`.

**Observation is local.** Each node's truth is what it can observe. Causal cones answer "what could have influenced this event?" Propagation modeling estimates latency by subnet distance. Continuity proofs (36 bytes) verify chain integrity without the full log.

**Partitions heal honestly.** Correlated failure detection classifies mass failures by subnet correlation. When partitions heal, divergent entity logs are reconciled: longest chain wins, deterministic tiebreak, losing chains fork with documented lineage.

**The event bus is non-localized.** Unlike broker-based systems (Kafka, Pulsar) or single-process ring buffers (LMAX Disruptor), the event bus has no fixed location. Local ring buffers are speed buffers; the logical bus spans the mesh. No broker to provision or fail over. No plaintext at relay nodes. No partition-leader bottleneck — ordering is per-entity via causal chains, not per-partition via a single leader. Events exist in transit; storage is a choice via adapters, not an architectural requirement.

**Event consumption is location-transparent.** A `MeshDaemon` receives events through the same `process(&CausalEvent)` interface regardless of whether the event originated locally, one hop away, or across the mesh. The mesh handles routing, decryption, and chain validation before the daemon sees the event. Code written for a single-node prototype runs unmodified on a multi-hop deployment. The topology is a runtime decision, not a code change.

**Capability announcements drive routing.** Every node advertises what it can do — hardware (GPU model, VRAM, CPU cores), software (loaded models, tools, supported subprotocols), and capacity (available slots, current load). The `CapabilityIndex` indexes these announcements for sub-microsecond queries. Routing decisions use capability tags: a request for inference routes to the nearest node with a matching GPU, not to a fixed endpoint. `CapabilityDiff` propagates incremental updates — a node that loads a new model announces only the delta.

**The proximity graph is the topology.** Each node maintains a `ProximityGraph` of its neighborhood built from direct observation and `EnhancedPingwave` broadcasts. Edges carry measured latency. The graph answers "who is nearby and how fast can I reach them?" without a global directory. Combined with capability announcements, it answers "who nearby can do what I need?" Routing follows the graph — traffic flows toward nodes that are close and capable.

**Subnets partition the mesh hierarchically.** A `SubnetId` encodes 4 levels (region/fleet/vehicle/subsystem) in 4 bytes. Subnets constrain observation — a node observes its peers at its level and derives the rest through gateways. `SubnetGateway` nodes aggregate health, compress capability summaries, and enforce channel visibility at boundaries. `SubnetPolicy` assigns nodes to subnets from capability labels. This keeps observation cost bounded as the mesh grows.

**Channels are the pub/sub layer.** `ChannelName` uses hierarchical hashing (`sensors/lidar/front`) with wildcard support. `ChannelConfig` sets per-channel policies: visibility (public, subnet-local, private), required capabilities, and retention. `AuthGuard` enforces access control at the channel boundary using a bloom filter — <10ns per-packet authorization checks. Channels are how applications structure communication without coupling to node identity.

**Daemons are the compute unit.** The `MeshDaemon` trait defines a stateful event processor: receive a `CausalEvent`, produce output, maintain a causal chain. `DaemonHost` manages the lifecycle — initialization, event dispatch, chain production, horizon tracking, snapshot packaging. `DaemonRegistry` maps daemon types to constructors. The `PlacementScheduler` decides where to run daemons based on capability requirements. When a node fails, the migration state machine moves the daemon's state (snapshot + chain) to a new host in 6 phases, preserving continuity.

**Safety envelopes enforce autonomy.** Every node runs a `SafetyEnforcer` that defines resource limits, rate caps, and kill-switch conditions via `ResourceEnvelope`. A `RuleEngine` evaluates device autonomy policies — declarative rules that determine what a node will accept, reject, or redirect. No external authority can override a node's safety envelope. The mesh routes around nodes that refuse work, it doesn't force them.

## Stack

| Layer | What it does | Docs |
|-------|--------------|------|
| **Transport** | Encrypted UDP, 64-byte cache-line-aligned header, zero-alloc packet pools, multi-hop forwarding, adaptive batching, fair scheduling, failure detection, pingwave swarm discovery | [TRANSPORT.md](docs/TRANSPORT.md) |
| **Trust & Identity** | ed25519 entity identity, origin binding on every packet, permission tokens with delegation chains | [IDENTITY.md](docs/IDENTITY.md) |
| **Channels & Authorization** | Named hierarchical channels, capability-based access control, bloom filter authorization at <10ns per packet | [CHANNELS.md](docs/CHANNELS.md) |
| **Behavior Plane** | Capability announcements & indexing, capability diffs, node metadata, API schema registry, device autonomy rules, context fabric (distributed tracing), load balancing, proximity graph, safety envelope enforcement | [BEHAVIOR.md](docs/BEHAVIOR.md) |
| **Subnets & Hierarchy** | 4-level subnet hierarchy (8/8/8/8 encoding), label-based assignment, gateway visibility enforcement | [SUBNETS.md](docs/SUBNETS.md) |
| **Distributed State** | 24-byte causal links, compressed observed horizons, append-only entity logs with chain validation, state snapshots for migration | [STATE.md](docs/STATE.md) |
| **Compute Runtime** | MeshDaemon trait, daemon hosting, capability-based placement, 6-phase migration with snapshot chunking, replica groups, fork groups with verifiable lineage, active-passive standby groups, shared group coordination | [COMPUTE.md](docs/COMPUTE.md) |
| **Subprotocols** | Formal protocol registry, version negotiation, capability-aware routing via tags, opaque forwarding guarantee, migration message dispatch | [SUBPROTOCOLS.md](docs/SUBPROTOCOLS.md) |
| **Observational Continuity** | Causal cones, propagation modeling, continuity proofs, honest discontinuity with deterministic forking, superposition during migration | [CONTINUITY.md](docs/CONTINUITY.md) |
| **Contested Environments** | Correlated failure detection, subnet-aware partition classification, partition healing with log reconciliation | [CONTESTED.md](docs/CONTESTED.md) |
| **RedEX (local log)** | 20-byte append-only event records, inline + heap payload hybrid, `ChannelName`-bound files, atomic backfill-then-live tail, count + size retention, optional disk durability via `redex-disk` (torn-write truncation on reopen) | [REDEX_PLAN.md](docs/REDEX_PLAN.md) |
| **CortEX adapter** | Seam between Net events and RedEX storage: 20-byte `EventMeta` prefix projection, fold-driver spawning on a tokio task, `changes()` broadcast primitive for reactive queries, `Arc<RwLock<State>>` as the NetDB read surface, start-position + fold-error policies | [CORTEX_ADAPTER_PLAN.md](docs/CORTEX_ADAPTER_PLAN.md) |
| **CortEX models** | Concrete fold implementations: tasks (CRUD on `Task`) and memories (content + tags + pin, with single/any/all tag predicates). Each ships a Prisma-style query builder and a reactive watcher (initial + deduplicated emissions). Dispatches partitioned under `0x00..0x7F`. | [CORTEX_ADAPTER_PLAN.md](docs/CORTEX_ADAPTER_PLAN.md) |
| **NetDB (query façade)** | Unified `NetDb` handle bundling `TasksAdapter` + `MemoriesAdapter` under one object. Prisma-ish `find_unique` / `find_many(&filter)` / `count_where` / `exists_where` on per-model state. Whole-db snapshot/restore. Cross-language: Rust, Node, Python. | [NETDB_PLAN.md](docs/NETDB_PLAN.md) |

## Architecture

```
                    ┌──────────────────────────────────┐
                    │            EventBus              │
                    │  (sharded ring buffers, < 1us)   │
                    └──────────┬───────────────────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
        ┌─────┴─────┐   ┌─────┴─────┐   ┌──────┴──────┐
        │   Redis    │   │ JetStream │   │    Net     │
        │  Streams   │   │   (NATS)  │   │ (encrypted  │
        └───────────┘   └───────────┘   │  UDP mesh)  │
                                         └──────┬──────┘
                                                │
┌───────────────────────────────────────────────────────────────────┐
│                        Net Mesh Layers                          │
├──────────┬──────────┬──────────┬──────────┬──────────┬───────────┤
│ Identity │ Channels │ Behavior │  State   │ Compute  │ Contested │
│ ed25519  │ AuthGuard│ CAP-ANN  │ Causal   │ Daemon   │ Partition │
│ tokens   │ bloom    │ API-REG  │ chains   │ host     │ healing   │
│ origin   │ caps     │ rules    │ horizons │ scheduler│ reconcile │
├──────────┴──────────┴──────────┴──────────┴──────────┴───────────┤
│        Subnets (4-level hierarchy, gateway enforcement)          │
├──────────────────────────────────────────────────────────────────┤
│           Subprotocols + Observational Continuity                │
│        version negotiation, causal cones, fork records           │
├──────────────────────────────────────────────────────────────────┤
│                       Transport (Net)                           │
│     64B header, ChaCha20-Poly1305, Noise NK, zero-alloc pools   │
│     routing, swarm, failure detection, proximity graph           │
└──────────────────────────────────────────────────────────────────┘
```

Every field is used by at least one layer. Forwarding nodes read one cache line, make a routing decision, and forward without decrypting the payload.

## Net Header (64 bytes, cache-line aligned)

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         MAGIC (0x4E45)        |     VER       |     FLAGS     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   PRIORITY    |    HOP_TTL    |   HOP_COUNT   |  FRAG_FLAGS   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       SUBPROTOCOL_ID          |        CHANNEL_HASH           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         NONCE (12 bytes)                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       SESSION_ID (8 bytes)                    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       STREAM_ID (8 bytes)                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       SEQUENCE (8 bytes)                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|      SUBNET_ID (4 bytes)      |     ORIGIN_HASH (4 bytes)     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       FRAGMENT_ID             |        FRAGMENT_OFFSET        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       PAYLOAD_LEN             |        EVENT_COUNT            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

## Performance

Benchmarked on Apple M1 Max, macOS.

| Layer | Operation | Latency | Throughput |
|-------|-----------|---------|------------|
| **Core** | Event ingestion | < 1 us p99 | 10M+ events/sec sustained |
| **Net** | Header serialize | 2.05 ns | 487M ops/sec |
| **Net** | Packet build (50 events) | 10.8 us | -- |
| **Net** | Encryption (ChaCha20) | 743 ns (64B) | -- |
| **Routing** | Header roundtrip | 0.98 ns | 1.02G ops/sec |
| **Routing** | Lookup hit | 20.2 ns | 49.5M ops/sec |
| **Routing** | Decision pipeline | 18.0 ns | 55.7M ops/sec |
| **Forwarding** | Per-hop (64B) | 30.4 ns | -- |
| **Forwarding** | 5-hop chain | 291 ns | 3.4M ops/sec |
| **Swarm** | Pingwave roundtrip | 0.98 ns | 1.02G ops/sec |
| **Swarm** | Graph (5,000 nodes) | 125 us | 39.9M/sec |
| **Failure** | Heartbeat | 32.4 ns | 30.9M ops/sec |
| **Failure** | Full recovery cycle | 362 ns | 2.8M ops/sec |
| **Capability** | Filter (single tag) | 10.5 ns | 95.5M ops/sec |
| **Capability** | GPU check | 0.33 ns | 3.07G ops/sec |
| **Auth** | Bloom filter check | < 10 ns | -- |
| **SDK** | Go raw ingest | 377 ns | 2.65M/sec |
| **SDK** | Python batch ingest | 0.36 us | 2.78M/sec |
| **SDK** | Node.js push batch | 0.35 us | 2.89M/sec |
| **SDK** | Bun batch ingest | 0.30 us | 3.37M/sec |

Benchmarks accurate as of April 15, 2026.

Thread-local packet pools scale to **23x contention advantage** over shared pools at 32 threads. All SDKs exceed **2M events/sec** with optimal ingestion patterns.

1,208 tests. ~840 KB deployed binary.

## Capabilities

Every node advertises what it can do. `HardwareCapabilities` describes the machine — GPU model, VRAM, CPU cores, available memory. The `CapabilityIndex` indexes all known nodes' capabilities for sub-microsecond queries.

```
Node A announces:
  gpu: RTX 4090, vram: 24GB
  models: [gemma-21b, llama-7b]
  tags: [inference, cuda]
  capacity: 8 slots available

Node B queries:
  CapabilityIndex::query(require_gpu(24GB) & tag("inference"))
  → returns [Node A] in ~10ns
```

Capabilities are not static. When a node loads a new model, drops a tool, or runs out of capacity, it publishes a `CapabilityDiff` — an incremental update, not a full re-announcement. The `DiffEngine` computes minimal diffs. Neighbors propagate diffs through the proximity graph, so the mesh converges without flooding.

Routing follows capabilities. A request tagged `subprotocol:0x1000` routes to the nearest node that advertises support for that subprotocol. An inference request routes to the nearest node with enough VRAM. The mesh doesn't have fixed endpoints — it has a capability graph, and traffic flows toward capability.

The `CapabilityAd` struct is what travels on the wire: compact, versioned, and signed with the node's identity. A node that claims capabilities it doesn't have will be routed around when its behavior diverges from its advertisement — the proximity graph measures actual latency, not claimed latency.

## Proximity & Discovery

Nodes find each other through `Pingwave` — periodic broadcasts that propagate outward within a configurable hop radius. A pingwave carries the node's identity, capabilities summary, and a timestamp. If you can hear a node's pingwave, you know it exists, how far away it is, and what it can do.

The `ProximityGraph` is built from direct observation. Each node maintains a local view of its neighborhood — not a global directory. Edges carry measured RTT latency. The graph is continuously updated from pingwave observations and direct communication.

```
ProximityGraph for Node A:
  Node B — 0.3ms (direct neighbor)
  Node C — 0.7ms (via B)
  Node D — 1.2ms (via B → C)
  Gateway G — 2.1ms (subnet boundary)
```

`EnhancedPingwave` extends the basic pingwave with capability summaries and load indicators, so routing decisions can be made from the proximity graph alone without querying the full `CapabilityIndex`.

**Pingwaves install routes.** On receipt of a pingwave for origin Y forwarded by direct peer Z, node X calls `RoutingTable::add_route_with_metric(Y, next_hop=Z, metric=hop_count+2)` and inserts the `Z → Y` edge into `ProximityGraph::edges`. The `+2` metric keeps direct routes (metric 1) strictly better than any pingwave-installed route. Four loop-avoidance rules sit at the dispatch boundary: origin self-check (drop pingwaves with `origin == self_id`), `MAX_HOPS = 16` receive-time cap, split horizon (don't advertise a route back on the link used to reach it), and unregistered-source rejection (only registered direct peers can inject routing state). Latency EWMA per `(origin, next_hop)` edge provides an equal-hop tie-breaker for future multi-alternate ranking. See [`ROUTING_DV_PLAN.md`](docs/ROUTING_DV_PLAN.md).

Discovery is emergent. There are no bootstrap servers, no DNS, no service registry. After first contact (manual address, LAN broadcast, QR code, cached peers), pingwaves propagate and the proximity graph builds itself. Nodes that go silent are pruned. Nodes that appear are integrated. The graph is always a reflection of current reality.

## Subnets

The mesh is logically flat but scales via hierarchical partitioning. A `SubnetId` packs 4 levels into 4 bytes:

```
SubnetId: [region: u8] [fleet: u8] [vehicle: u8] [subsystem: u8]

Example: 10.3.7.2
  region=10 (EU-West)
  fleet=3   (Factory Floor A)
  vehicle=7 (Robot Arm #7)
  subsystem=2 (Gripper Controller)
```

`SubnetGateway` nodes sit at subnet boundaries. They aggregate health from their subnet, compress capability summaries for external consumption, and enforce channel visibility — a channel marked `subnet-local` doesn't leak through the gateway. Gateways are protocol-equal nodes that happen to be reachable from both sides of a boundary.

`SubnetPolicy` assigns nodes to subnets automatically from capability labels. A node tagged `fleet:factory-a` and `role:robot-arm` gets assigned to the matching subnet without manual configuration.

Subnets bound observation cost. A node observes its peers at its level. For everything beyond, it observes the gateway and derives the rest. A node doesn't need heartbeats from 10,000 peers — it needs heartbeats from its neighbors and health summaries from gateways. Observation scales with the depth of the hierarchy, not the size of the mesh.

## Channels

Channels are how applications structure communication. `ChannelName` uses hierarchical hashing with path components:

```
sensors/lidar/front     → ChannelId(0xa3f1)
sensors/lidar/rear      → ChannelId(0xb7c2)
sensors                 → prefix match on hierarchical names
alerts/temperature      → ChannelId(0x1e09)
```

`ChannelConfig` defines per-channel policy:
- **Visibility**: public (mesh-wide), subnet-local (stays within subnet), private (explicit peer list)
- **Required capabilities**: only nodes with matching capabilities can subscribe
- **Retention**: how long events persist in adapters

`AuthGuard` enforces authorization at the channel boundary. It combines capability filters with permission tokens. A node needs both the right capabilities (hardware, tags) and a valid token (ed25519-signed, delegatable, expirable) to access a channel. Authorization results are cached in a bloom filter — <10ns per-packet checks.

Channels decouple applications from node identity. A producer emits to `sensors/temperature`. A consumer subscribes to `sensors/temperature`. Neither knows or cares which node the other is. The mesh connects them through the channel, the proximity graph finds the shortest path, and the auth guard ensures both sides are authorized.

## Daemons

A `MeshDaemon` is a stateful event processor — the compute unit of the mesh.

```rust
trait MeshDaemon: Send + Sync {
    fn process(&mut self, event: &CausalEvent) -> DaemonOutput;
    fn snapshot(&self) -> StateSnapshot;
    fn restore(&mut self, snapshot: StateSnapshot);
}
```

`DaemonHost` manages the runtime lifecycle: initialization, event dispatch, causal chain production, horizon tracking, and snapshot packaging. Every event a daemon produces is automatically linked into a causal chain with a 24-byte `CausalLink` (origin, sequence, parent hash, compressed horizon).

`DaemonRegistry` maps daemon types to constructors. The `PlacementScheduler` decides where to run each daemon based on capability requirements — a daemon that needs a GPU is placed on a GPU node. If the best candidate is already loaded, the scheduler considers the next-best via the proximity graph.

When a node fails or needs load balancing, migration preserves continuity in 6 phases:

1. **Snapshot** — source captures daemon state, chain head, and horizon
2. **Transfer** — snapshot sent to target (auto-chunked for large state)
3. **Restore** — target reassembles chunks and rebuilds the daemon from the snapshot using a factory + keypair + config resolved from its local `DaemonFactoryRegistry`
4. **Replay** — buffered events (arrived during transfer) replayed in causal order
5. **Cutover** — source stops writes and cleans up; source daemon unregistered
6. **Complete** — orchestrator emits `ActivateTarget`; target drains remaining events, activates, replies with `ActivateAck`; migration record removed

The full chain runs autonomously over `SUBPROTOCOL_MIGRATION` (0x0500); no manual orchestration is required once `start_migration()` is called. The `MigrationOrchestrator` coordinates across three nodes (source, target, controller). `MigrationSourceHandler` manages the source side (snapshot, buffer, quiesce, cleanup). `MigrationTargetHandler`, constructed via `new_with_factories(registry, factories)`, manages the target side (reassemble, restore, ordered replay via `BTreeMap`, activate). Auto-target selection queries the `CapabilityIndex` for nodes advertising `subprotocol:0x0500`.

The daemon's causal chain continues unbroken on the new host. During migration, a `SuperpositionState` tracks which phase the daemon is in — it exists on both nodes briefly, then collapses to the new host.

For daemons that need horizontal scale rather than mobility, `ReplicaGroup` manages N copies as a logical unit. Each replica gets a deterministic identity derived from `group_seed + index` — the same index always produces the same keypair, making replacement idempotent. The group load-balances inbound events across replicas (round-robin, least-connections, consistent hash — any `LoadBalancer` strategy), tracks group-level health (alive as long as one replica is healthy), and spreads placement across nodes for failure-domain isolation. When a node fails, the group re-spawns the affected replicas on new nodes with the same deterministic identity — no migration protocol needed for stateless daemons. Scaling is `scale_to(n)`: scale up appends new replicas, scale down removes the highest-index ones deterministically.

For daemons that need to diverge rather than replicate, `ForkGroup` creates N independent entities forked from a common parent. Each fork has a `ForkRecord` with a cryptographically verifiable sentinel hash linking back to the parent's causal chain at the fork point. Unlike replicas (interchangeable, deterministic per-index identities), forks are independent entities with documented lineage — any node can verify the fork by recomputing the sentinel. Fork keypairs are stored for recovery on failure, preserving identity across re-spawns.

For stateful daemons that need fault tolerance without duplicate compute, `StandbyGroup` implements active-passive replication. One member processes all events. The others hold readiness to promote — they consume memory for their snapshot but do zero event processing. Periodic `sync_standbys()` captures the active's state. On failure, the standby with the most recent snapshot promotes and replays the buffered events since that snapshot — the same replay mechanism MIKOSHI uses for migration. Persistence of snapshot bytes to disk is an application concern; the protocol provides the bytes and the bookkeeping.

All three group types share coordination logic via `GroupCoordinator` — health tracking, member management, and placement work identically. Any member of any group is a normal daemon in the `DaemonRegistry`, so MIKOSHI can migrate it without knowing it belongs to a group.

## Safety & Autonomy

Every node enforces its own rules. The `SafetyEnforcer` evaluates a `ResourceEnvelope` that defines:

- **Rate limits**: max events/sec ingested, max events/sec forwarded
- **Memory limits**: max ring buffer usage, max snapshot size
- **Compute limits**: max concurrent daemons, max CPU time per event
- **Kill switch**: conditions under which the node drops all traffic and goes silent

The `RuleEngine` evaluates declarative `RuleSet` policies:

```
Rule: if load > 90% then reject(priority < 5)
Rule: if peer.subnet != self.subnet then require(token.scope = "cross-subnet")
Rule: if event.size > 64KB then drop
```

Rules are local decisions, not network policy. No external authority can override a node's safety envelope. A node that refuses work is routed around — the proximity graph reflects this within a heartbeat interval. The mesh adapts to the node's boundaries rather than forcing the node to adapt to the mesh.

This is device autonomy in practice. A $5 sensor node sets tight limits — low rate, small buffer, no daemons. A $1500 GPU node sets generous limits — high rate, large buffer, many daemons. Both are equal participants on the mesh. The protocol treats them identically. Their capabilities and autonomy rules determine what they actually do.

## Module Map

```
src/adapter/net/
├── mod.rs                 # NetAdapter, routing utilities
├── mesh.rs                # MeshNode — multi-peer mesh runtime (single socket, forwarding, subprotocol dispatch)
├── config.rs              # NetAdapterConfig
├── crypto.rs              # Noise NKpsk0 handshake, ChaCha20-Poly1305 AEAD
├── handshake_relay.rs     # Handshake relay (SUBPROTOCOL_HANDSHAKE 0x0601), connect_via
├── protocol.rs            # 64-byte wire header, EventFrame, NackPayload
├── transport.rs           # UDP socket abstraction, batched I/O
├── session.rs             # Session state, stream multiplexing, thread-local pools
├── router.rs              # FairScheduler, stream routing, priority bypass
├── route.rs               # RoutingTable, multi-hop headers, stream stats
├── proxy.rs               # Zero-copy multi-hop forwarding, TTL enforcement
├── pool.rs                # Zero-alloc PacketPool, PacketBuilder, ThreadLocalPool
├── batch.rs               # AdaptiveBatcher, latency-aware sizing
├── reliability.rs         # FireAndForget / ReliableStream, selective NACKs
├── failure.rs             # FailureDetector, RecoveryManager, CircuitBreaker
├── swarm.rs               # Pingwave discovery, CapabilityAd, LocalGraph
├── linux.rs               # recvmmsg batch reads (Linux-only)
│
├── identity/              # Layer 1 — Trust & Identity
│   ├── entity.rs          #   EntityId, EntityKeypair (ed25519)
│   ├── origin.rs          #   OriginStamp binding
│   └── token.rs           #   PermissionToken, TokenScope, TokenCache
│
├── channel/               # Layer 2 — Channels & Authorization
│   ├── config.rs          #   ChannelConfig, Visibility, ChannelConfigRegistry
│   ├── guard.rs           #   AuthGuard, AuthVerdict, bloom filter
│   └── name.rs            #   ChannelId, ChannelName (hierarchical hashing)
│
├── behavior/              # Behavior Plane — Semantic Layer
│   ├── capability.rs      #   HardwareCapabilities, CapabilityIndex, GpuInfo
│   ├── diff.rs            #   CapabilityDiff, DiffEngine
│   ├── metadata.rs        #   NodeMetadata, MetadataStore, TopologyHints
│   ├── api.rs             #   ApiRegistry, ApiSchema, version validation
│   ├── rules.rs           #   RuleEngine, RuleSet, device autonomy policies
│   ├── context.rs         #   Context, ContextStore, Span, distributed tracing
│   ├── loadbalance.rs     #   LoadBalancer, Strategy, health-aware selection
│   ├── proximity.rs       #   ProximityGraph, EnhancedPingwave, latency edges
│   └── safety.rs          #   SafetyEnforcer, ResourceEnvelope, rate limits, kill switch
│
├── subnet/                # Layer 3 — Subnets & Hierarchy
│   ├── id.rs              #   SubnetId (4 x 8-bit levels)
│   ├── assignment.rs      #   SubnetPolicy, label-based rules
│   └── gateway.rs         #   SubnetGateway, visibility enforcement
│
├── state/                 # Layer 4 — Distributed State
│   ├── causal.rs          #   CausalChainBuilder, CausalEvent, CausalLink (24B)
│   ├── horizon.rs         #   HorizonEncoder, ObservedHorizon (compressed)
│   ├── log.rs             #   EntityLog, append-only chain validation
│   └── snapshot.rs        #   StateSnapshot, SnapshotStore
│
├── compute/               # Layer 5 — Compute Runtime
│   ├── daemon.rs          #   MeshDaemon trait
│   ├── daemon_factory.rs  #   DaemonFactoryRegistry (origin_hash → factory + keypair + config) for target-side restore
│   ├── host.rs            #   DaemonHost runtime, from_snapshot(), from_fork()
│   ├── migration.rs       #   MigrationState, MigrationPhase, 6-phase state machine
│   ├── orchestrator.rs    #   MigrationOrchestrator, wire protocol, snapshot chunking, ActivateTarget/ActivateAck
│   ├── migration_source.rs #  Source-side: snapshot, buffer, cutover, cleanup
│   ├── migration_target.rs #  Target-side: restore, replay, activate
│   ├── group_coord.rs     #   GroupCoordinator, shared LB/health/routing
│   ├── replica_group.rs   #   ReplicaGroup, N-way replication, deterministic identity
│   ├── fork_group.rs      #   ForkGroup, N-way forking, verifiable lineage
│   ├── standby_group.rs   #   StandbyGroup, active-passive stateful replication
│   ├── registry.rs        #   DaemonRegistry
│   └── scheduler.rs       #   Capability-based placement, migration target discovery
│
├── subprotocol/           # Layer 6 — Subprotocol Registry
│   ├── descriptor.rs      #   SubprotocolDescriptor, versioning
│   ├── migration_handler.rs #  Migration message dispatch (0x0500)
│   ├── negotiation.rs     #   Version negotiation, SubprotocolManifest
│   └── registry.rs        #   SubprotocolRegistry, capability enrichment
│
├── continuity/            # Layer 7 — Observational Continuity
│   ├── chain.rs           #   ContinuityProof (36B), ContinuityStatus
│   ├── cone.rs            #   CausalCone, Causality analysis
│   ├── discontinuity.rs   #   ForkRecord, DiscontinuityReason, fork_entity()
│   ├── observation.rs     #   ObservationWindow, HorizonDivergence
│   ├── propagation.rs     #   PropagationModel, subnet-distance latency
│   └── superposition.rs   #   SuperpositionState, migration phase tracking
│
├── contested/             # Layer 8 (Partial) — Contested Environments
│   ├── correlation.rs     #   CorrelatedFailureDetector, subnet correlation
│   ├── partition.rs       #   PartitionDetector, PartitionPhase, healing
│   └── reconcile.rs       #   Log reconciliation, longest-chain-wins, ForkRecord
│
├── redex/                 # RedEX v1 — local append-only event log (feature `redex`)
│   ├── mod.rs             #   Re-exports: Redex, RedexFile, RedexEvent, RedexError, ...
│   ├── entry.rs           #   20-byte RedexEntry codec, RedexFlags, payload_checksum
│   ├── config.rs          #   RedexFileConfig (persistent, retention, sync_interval)
│   ├── event.rs           #   RedexEvent { entry, payload }
│   ├── error.rs           #   RedexError (thiserror-derived)
│   ├── segment.rs         #   HeapSegment (append-only Vec<u8>, evict_prefix_to)
│   ├── retention.rs       #   compute_eviction_count (count + size policy)
│   ├── fold.rs            #   RedexFold<State> trait (CortEX / NetDB integration hook)
│   ├── file.rs            #   RedexFile (append / tail / read_range / close)
│   ├── manager.rs         #   Redex manager (open_file / get_file / with_persistent_dir)
│   └── disk.rs            #   DiskSegment (feature `redex-disk`): idx + dat append-only files, torn-write recovery
│
├── cortex/                # CortEX adapter — NetDB fold driver (feature `cortex`)
│   ├── mod.rs             #   Re-exports: CortexAdapter, EventMeta, EventEnvelope, ...
│   ├── meta.rs            #   20-byte EventMeta prefix codec + dispatch/flag constants
│   ├── envelope.rs        #   EventEnvelope + IntoRedexPayload trait
│   ├── config.rs          #   CortexAdapterConfig, StartPosition, FoldErrorPolicy
│   ├── error.rs           #   CortexAdapterError
│   ├── adapter.rs         #   CortexAdapter<State>: fold task, wait_for_seq, changes() broadcast
│   │
│   ├── tasks/             # First CortEX model — mutate-by-id CRUD (feature `cortex`)
│   │   ├── types.rs       #     Task, TaskStatus, TaskId + serde payload structs
│   │   ├── dispatch.rs    #     DISPATCH_TASK_* (0x01..0x04), TASKS_CHANNEL
│   │   ├── state.rs       #     TasksState + basic accessors
│   │   ├── fold.rs        #     TasksFold (decodes EventMeta, routes by dispatch)
│   │   ├── query.rs       #     TasksQuery fluent builder + TasksFilterSpec + OrderBy
│   │   ├── watch.rs       #     TasksWatcher reactive stream (initial + dedup)
│   │   └── adapter.rs     #     TasksAdapter wrapper (typed ingest + watch)
│   │
│   └── memories/          # Second CortEX model — content + tags + pin (feature `cortex`)
│       ├── types.rs       #     Memory, MemoryId + serde payload structs
│       ├── dispatch.rs    #     DISPATCH_MEMORY_* (0x10..0x14), MEMORIES_CHANNEL
│       ├── state.rs       #     MemoriesState + pinned/unpinned splits
│       ├── fold.rs        #     MemoriesFold
│       ├── query.rs       #     MemoriesQuery with single/any/all tag predicates
│       ├── watch.rs       #     MemoriesWatcher
│       └── adapter.rs     #     MemoriesAdapter wrapper
│
└── netdb/                 # NetDB — unified query façade over CortEX state (feature `netdb`)
    ├── mod.rs             #   Re-exports: NetDb, NetDbBuilder, NetDbSnapshot, NetDbError + re-exports of TasksFilter / MemoriesFilter
    ├── db.rs              #   NetDb (bundles TasksAdapter + MemoriesAdapter) + NetDbBuilder + whole-db snapshot/restore
    └── error.rs           #   NetDbError (wraps CortexAdapterError + missing-model errors)
```

## Adapters

### In-Memory (default)

```rust
use net::{EventBus, EventBusConfig};

let bus = EventBus::new(EventBusConfig::default()).await?;
bus.ingest(Event::from_str(r#"{"token": "hello"}"#)?)?;
```

### Redis

```toml
net = { path = ".", features = ["redis"] }
```

### JetStream

```toml
net = { path = ".", features = ["jetstream"] }
```

### Net

```toml
net = { path = ".", features = ["net"] }
```

## SDKs

All SDKs wrap the same Rust core. Every language gets the same performance.

| SDK | Package | Docs | Highlights |
|-----|---------|------|------------|
| **Rust** | [`net-sdk`](sdk/) | [README](sdk/README.md) | Builder pattern, async streams, typed subscriptions |
| **TypeScript** | [`@ai2070/net-sdk`](sdk-ts/) | [README](sdk-ts/README.md) | AsyncIterator, typed channels, Zod support |
| **Python** | [`net-sdk`](sdk-py/) | [README](sdk-py/README.md) | Generators, dataclass/Pydantic, context manager |
| **Go** | [`net`](bindings/go/) | [README](bindings/go/README.md) | CGO bindings, zero allocations on raw ingest |
| **C** | [`net.h`](include/net.h) | [README](include/README.md) | One header, structured types, zero JSON overhead |

### Rust

```rust
use net_sdk::{Net, Backpressure};
use futures::StreamExt;

let node = Net::builder()
    .shards(4)
    .backpressure(Backpressure::DropOldest)
    .memory()
    .build()
    .await?;

// Emit
node.emit(&serde_json::json!({"token": "hello"}))?;

// Stream
let mut stream = node.subscribe(Default::default());
while let Some(event) = stream.next().await {
    println!("{}", event?.raw_str());
}

node.shutdown().await?;
```

### TypeScript

```typescript
import { NetNode } from '@ai2070/net-sdk';

const node = await NetNode.create({ shards: 4 });

// Emit
node.emit({ token: 'hello', index: 0 });

// Stream
for await (const event of node.subscribe({ limit: 100 })) {
  console.log(event.raw);
}

// Typed channels
const temps = node.channel<{ celsius: number }>('sensors/temperature');
temps.publish({ celsius: 22.5 });

await node.shutdown();
```

### Python

```python
from net_sdk import NetNode

node = NetNode(shards=4)

# Emit
node.emit({'token': 'hello', 'index': 0})

# Stream (generator)
for event in node.subscribe(limit=100):
    print(event.raw)

# Typed channels with Pydantic
temps = node.channel('sensors/temperature', TemperatureReading)
temps.publish(TemperatureReading(sensor_id='A1', celsius=22.5))

node.shutdown()
```

### Go

```go
node, _ := net.New(&net.Config{NumShards: 4})
defer node.Shutdown()

// Ingest
node.IngestRaw(`{"token": "hello"}`)

// Batch (zero allocations on raw path)
jsons := []string{`{"a":1}`, `{"a":2}`, `{"a":3}`}
count := node.IngestRawBatch(jsons)

// Poll
response, _ := node.Poll(100, "")
for _, event := range response.Events {
    fmt.Println(string(event))
}
```

### C

```c
#include "net.h"

net_handle_t node = net_init("{\"num_shards\": 4}");

// Ingest with receipt
const char* event = "{\"token\": \"hello\"}";
net_receipt_t receipt;
net_ingest_raw_ex(node, event, strlen(event), &receipt);

// Poll (structured, no JSON parsing)
net_poll_result_t result;
net_poll_ex(node, 100, NULL, &result);
for (size_t i = 0; i < result.count; i++) {
    printf("%.*s\n", (int)result.events[i].raw_len, result.events[i].raw);
}
net_free_poll_result(&result);

net_shutdown(node);
```

## Features

| Feature | Flag | Dependencies |
|---------|------|--------------|
| Redis Streams | `redis` | `redis` crate |
| NATS JetStream | `jetstream` | `async-nats` |
| Net transport | `net` | `chacha20poly1305`, `snow`, `blake2`, `dashmap`, `socket2`, `ed25519-dalek` |
| Regex filters | `regex` | `regex` crate |
| C FFI | `ffi` | -- |
| RedEX (local append-only log) | `redex` | `net`, `tokio-stream`, `bincode` |
| RedEX disk durability | `redex-disk` | `redex` |
| CortEX (adapter core + tasks + memories) | `cortex` | `redex` |
| NetDB (unified query façade) | `netdb` | `cortex` |

Default feature is `redis`.

## Building

```bash
# Default (Redis adapter)
cargo build --release

# Net only (831 KB binary)
cargo build --release --no-default-features --features net

# Everything
cargo build --release --all-features
```

## Tests

```bash
# Unit tests (944 with every cortex/redex/netdb feature on)
cargo test --lib --features "net redex redex-disk cortex netdb"

# Migration & group integration tests (53 tests)
cargo test --test migration_integration --features net

# Three-node mesh integration tests (65 tests)
cargo test --test three_node_integration --features net

# Two-node transport integration (13 tests)
cargo test --test integration_net --features net

# RedEX integration tests (22 tests: heap + persistent + age retention + ordered appender + typed wrappers)
cargo test --test integration_redex --features "redex redex-disk"

# CortEX adapter core (8 tests)
cargo test --test integration_cortex_adapter --features cortex

# CortEX tasks model (17 tests: CRUD + query + watch + replay + snapshot)
cargo test --test integration_cortex_tasks --features cortex

# CortEX memories model (14 tests: CRUD + tag queries + watch + coexistence + snapshot)
cargo test --test integration_cortex_memories --features cortex

# NetDB unified façade (9 tests: build, CRUD, filters, whole-db snapshot/restore)
cargo test --test integration_netdb --features netdb

# Rust SDK smoke tests (2 async + 3 doctests)
cargo test --features net -p net-sdk

# Node SDK smoke tests (36 tests — CortEX tasks + memories over napi, incl. watch/AsyncIterator, disk durability, snapshot/restore round-trip, NetDb handle, and classified CortexError/NetDbError from @ai2070/net/errors)
cd bindings/node && npx napi build --platform --no-default-features -F cortex && npx vitest run

# Python SDK smoke tests (33 tests — CortEX tasks + memories via PyO3, incl. sync watch iterators, disk durability, snapshot/restore round-trip, NetDb handle, and typed CortexError / NetDbError from net._net)
cd bindings/python && uv venv .venv && source .venv/bin/activate && \
    uv pip install -e '.[test]' maturin && \
    maturin develop --no-default-features --features cortex && \
    python -m pytest tests/test_cortex.py tests/test_netdb.py

# Backend adapters (requires running services)
cargo test --test integration_redis --features redis
cargo test --test integration_jetstream --features jetstream
```

**1,146 tests total across the Rust stack** — lib (944) + migration (53) + three_node (65) + integration_net (13) + integration_redex (22) + integration_cortex_{adapter,tasks,memories} (8+17+14) + integration_netdb (9) + SDK doctest (1). Plus 36 Node SDK smoke tests (vitest) and 33 Python SDK smoke tests (pytest), both covering CRUD, filtered queries, reactive watchers, multi-model coexistence, disk-durability round-trips, whole-db `NetDb` snapshot/restore, per-adapter `open_from_snapshot`, and classified `CortexError` / `NetDbError` via the `@ai2070/net/errors` subpath (Node) / `net._net` module (Python) — all via the `cortex` / `netdb` features (which pull in `redex-disk`).

### Test Architecture

Unit tests live in `#[cfg(test)]` modules alongside the code they test. Each migration module (orchestrator, source handler, target handler, subprotocol handler) has isolated tests covering happy paths, error paths, and edge cases.

Integration tests in `tests/migration_integration.rs` exercise the full migration system across module boundaries:

| Category | What it validates |
|----------|-------------------|
| **Phase chain** | All 6 phases sequenced end-to-end through the orchestrator, with and without buffered events |
| **End-to-end** | Source handler → orchestrator → target handler composing correctly: snapshot, buffer, restore, replay, cutover, cleanup. Verifies daemon moves between registries. |
| **Auto-target** | Scheduler-driven target selection via `CapabilityIndex` queries for `subprotocol:0x0500` |
| **Handler dispatch** | Each `MigrationMessage` variant dispatched through `MigrationSubprotocolHandler`, verifying correct outbound message types |
| **Handler routing** | Outbound `dest_node` assertions — CutoverNotify reaches source, SnapshotReady reaches target, CleanupComplete reaches orchestrator |
| **Snapshot chunking** | Small (single-chunk), large (multi-chunk), out-of-order reassembly, duplicate chunks, chunk count boundaries |
| **Event flow** | Events buffered on source during migration → drained → replayed on target → daemon stats verify processing |
| **Concurrency** | Two daemons migrating simultaneously without interference |
| **Abort** | Clean abort at every phase (Snapshot, Transfer, Replay, Cutover) |
| **Capability discovery** | `enrich_capabilities()` → `CapabilityAnnouncement` → `CapabilityIndex` → `Scheduler.find_migration_targets()` |
| **Wire format** | Encode/decode roundtrip for all 10 message variants including chunked SnapshotReady, ActivateTarget, ActivateAck |
| **Full lifecycle auto-chaining** | TakeSnapshot through ActivateAck runs end-to-end through the subprotocol handler with a mock message pump — single-chunk and multi-chunk. Failure paths verified: missing `DaemonFactoryRegistry` entry, corrupt snapshot bytes, `ActivateTarget` without prior restore. |

Three-node mesh tests in `tests/three_node_integration.rs` exercise the `MeshNode` runtime over real encrypted UDP:

| Category | What it validates |
|----------|-------------------|
| **Mesh formation** | 3-way handshake, health isolation after node death |
| **Data flow** | Point-to-point, bidirectional, stream isolation, full ring traffic, sustained throughput |
| **Relay** | A→B→C forwarding without decryption, payload integrity over 100 events, **tamper detection** (AEAD rejects corrupted relay) |
| **Rerouting** | Manual route update after failure, **automatic reroute** via ReroutePolicy + failure detector, auto-recovery when peer returns. Resolution order: `RoutingTable::lookup_alternate` → `ProximityGraph::path_to` → any direct peer. |
| **Router** | Forward/local/TTL/hop-count decisions over real UDP, multi-hop with 2 routers |
| **Full stack** | EventBus→NetAdapter→encrypted UDP→poll, bidirectional EventBus, backpressure flood |
| **Subnet gateway** | SubnetLocal blocked, Global forwarded, Exported selective, ParentVisible ancestor-only |
| **Failure detection** | Heartbeat→suspect→fail→recover lifecycle, correlated failure classification |
| **Migration over wire** | Full 6-phase lifecycle (TakeSnapshot → SnapshotReady → Restore → Replay → Cutover → Cleanup → Activate) runs autonomously over encrypted UDP. Three-node test asserts daemon ends up on target, absent from source, orchestrator record cleared. Acks route to the recorded orchestrator, not the wire hop. |
| **Handshake relay** | `connect_via(relay_addr, …)` establishes a Noise NKpsk0 session with a peer that has no direct UDP path. Handshake rides as a routed Net packet (`HANDSHAKE` flag) over existing relay sessions; post-handshake data flows A↔C through B via `send_routed`. |
| **DV routing** | Pingwave-driven route install populates both `RoutingTable` and `ProximityGraph::edges`. 3-hop chain A→B→C→D: A learns the route to D via B; `path_to(D)` returns the full 3-hop path. Regression: `path_to` used to always return `None` because edges were never populated. |
| **Stream multiplexing** | Multiple independent streams per peer, per-stream reliability + fairness weight, epoch-guarded handles reject sends after close+reopen, idle eviction + LRU cap |
| **Stream back-pressure (v1)** | Concurrent callers on a window-sized stream: exactly one admission per slot; others get `StreamError::Backpressure`. `send_with_retry` absorbs the pressure and eventually succeeds. |
| **Channel fan-out** | `ChannelPublisher` + `SubscriberRoster` over `SUBPROTOCOL_CHANNEL_MEMBERSHIP` — subscribe, publish fan-out reaches every subscriber, unsubscribe + peer-fail eviction from the roster |
| **Partition** | Detection via filter, healing with data flow recovery, asymmetric 3-node partition |

Regression tests are prefixed `test_regression_` and tied to specific bugs found during review. Each documents the original bug in its doc comment and would fail if the fix were reverted.

## Benchmarks

```bash
cargo bench --features net --bench net
cargo bench --bench ingestion
cargo bench --bench parallel
```

## Subprotocol ID Space

| Range | Purpose |
|-------|---------|
| `0x0000` | Plain events (no subprotocol) |
| `0x0001..0x03FF` | Reserved for core |
| `0x0400` | Causal events |
| `0x0401` | State snapshots |
| `0x0500` | Daemon migration (Mikoshi) |
| `0x0600` | Subprotocol negotiation |
| `0x0700..0x0702` | Continuity / fork announce / continuity proof |
| `0x0800..0x0801` | Partition / reconciliation |
| `0x0900` | Replica group coordination (reserved) |
| `0x0A00` | Channel membership (subscribe / unsubscribe / ack) |
| `0x0B00` | Stream credit window (v2 backpressure, reserved — see [`STREAM_BACKPRESSURE_PLAN_V2.md`](docs/STREAM_BACKPRESSURE_PLAN_V2.md)) |
| `0x1000..0xEFFF` | Vendor / third-party |
| `0xF000..0xFFFF` | Experimental / ephemeral |

Note: handshake relay no longer consumes a subprotocol ID — it rides as a routed Net packet with the `HANDSHAKE` flag in the routing header, sharing the forwarding path with data packets.

## License

Apache-2.0
