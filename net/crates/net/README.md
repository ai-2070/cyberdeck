# Net

High-performance encrypted mesh runtime.

## Key Concepts

**Identity is cryptographic.** Every node has an ed25519 keypair. The public key IS the identity. `origin_hash` (truncated BLAKE2s) is stamped on every outgoing packet. Permission tokens are ed25519-signed, delegatable, and expirable.

**Channels are named and policy-bearing.** Hierarchical names like `sensors/lidar/front`. Access control via capability filters (does this node have a GPU? the right tool? the right tag?) combined with permission tokens. Authorization cached in a bloom filter for <10ns per-packet checks.

**Behavior is declarative.** Nodes announce hardware/software capabilities, expose API schemas, and publish metadata. A rule engine enforces device autonomy policies. Load balancing, proximity-aware routing, and safety envelopes operate on this semantic layer. Distributed context propagation enables cross-node tracing.

**Subnets are hierarchical.** 4-level encoding (region/fleet/vehicle/subsystem) in 4 bytes. Gateways enforce channel visibility at subnet boundaries. Label-based assignment from capability tags.

**State is causal.** Every event carries a 24-byte `CausalLink`: origin, sequence, parent hash, compressed horizon. The chain IS the entity's identity. If the chain breaks, a new entity forks with documented lineage.

**Compute migrates.** The `MeshDaemon` trait defines event processors. The runtime handles causal chain production, horizon tracking, and snapshot packaging. Migration is a 6-phase state machine preserving chain continuity across nodes.

**Subprotocols are extensible.** `subprotocol_id: u16` in every header. Formal registry with version negotiation. Unknown subprotocols are forwarded opaquely. Vendor protocols get IDs in `0x1000..0xEFFF`.

**Observation is local.** Each node's truth is what it can observe. Causal cones answer "what could have influenced this event?" Propagation modeling estimates latency by subnet distance. Continuity proofs (36 bytes) verify chain integrity without the full log.

**Partitions heal honestly.** Correlated failure detection classifies mass failures by subnet correlation. When partitions heal, divergent entity logs are reconciled: longest chain wins, deterministic tiebreak, losing chains fork with documented lineage.

## Stack

| Layer | What it does |
|-------|--------------|
| **Transport** | Encrypted UDP, 64-byte cache-line-aligned header, zero-alloc packet pools, multi-hop forwarding, adaptive batching, fair scheduling, failure detection, pingwave swarm discovery |
| **Trust & Identity** | ed25519 entity identity, origin binding on every packet, permission tokens with delegation chains |
| **Channels & Authorization** | Named hierarchical channels, capability-based access control, bloom filter authorization at <10ns per packet |
| **Behavior Plane** | Capability announcements & indexing, capability diffs, node metadata, API schema registry, device autonomy rules, context fabric (distributed tracing), load balancing, proximity graph, safety envelope enforcement |
| **Subnets & Hierarchy** | 4-level subnet hierarchy (8/8/8/8 encoding), label-based assignment, gateway visibility enforcement |
| **Distributed State** | 24-byte causal links, compressed observed horizons, append-only entity logs with chain validation, state snapshots for migration |
| **Compute Runtime** | MeshDaemon trait, daemon hosting with causal chain production, capability-based placement, 6-phase migration state machine |
| **Subprotocols** | Formal protocol registry, version negotiation, capability-aware routing via tags, opaque forwarding guarantee |
| **Observational Continuity** | Causal cones, propagation modeling, continuity proofs, honest discontinuity with deterministic forking, superposition during migration |
| **Contested Environments** | Correlated failure detection, subnet-aware partition classification, partition healing with log reconciliation |

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
        │   Redis    │   │ JetStream │   │    BLTP     │
        │  Streams   │   │   (NATS)  │   │ (encrypted  │
        └───────────┘   └───────────┘   │  UDP mesh)  │
                                         └──────┬──────┘
                                                │
┌───────────────────────────────────────────────────────────────────┐
│                        BLTP Mesh Layers                          │
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
│                       Transport (BLTP)                           │
│     64B header, ChaCha20-Poly1305, Noise NK, zero-alloc pools   │
│     routing, swarm, failure detection, proximity graph           │
└──────────────────────────────────────────────────────────────────┘
```

Every field is used by at least one layer. Forwarding nodes read one cache line, make a routing decision, and forward without decrypting the payload.

## BLTP Header (64 bytes, cache-line aligned)

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         MAGIC (0x424C)        |     VER       |     FLAGS     |
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
| **BLTP** | Header serialize | 2.05 ns | 487M ops/sec |
| **BLTP** | Packet build (50 events) | 10.8 us | -- |
| **BLTP** | Encryption (ChaCha20) | 743 ns (64B) | -- |
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

Thread-local packet pools scale to **23x contention advantage** over shared pools at 32 threads. All SDKs exceed **2M events/sec** with optimal ingestion patterns.

605 tests. 831 KB deployed binary.

## Module Map

```
src/adapter/bltp/
├── mod.rs                 # BltpAdapter, routing utilities
├── config.rs              # BltpAdapterConfig
├── crypto.rs              # Noise NKpsk0 handshake, ChaCha20-Poly1305 AEAD
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
│   ├── host.rs            #   DaemonHost runtime
│   ├── migration.rs       #   6-phase snapshot migration
│   ├── registry.rs        #   DaemonRegistry
│   └── scheduler.rs       #   Capability-based placement, PlacementDecision
│
├── subprotocol/           # Layer 6 — Subprotocol Registry
│   ├── descriptor.rs      #   SubprotocolDescriptor, versioning
│   ├── negotiation.rs     #   Version negotiation, SubprotocolManifest
│   └── registry.rs        #   SubprotocolRegistry, ID → handler mapping
│
├── continuity/            # Layer 7 — Observational Continuity
│   ├── chain.rs           #   ContinuityProof (36B), ContinuityStatus
│   ├── cone.rs            #   CausalCone, Causality analysis
│   ├── discontinuity.rs   #   ForkRecord, DiscontinuityReason, fork_entity()
│   ├── observation.rs     #   ObservationWindow, HorizonDivergence
│   ├── propagation.rs     #   PropagationModel, subnet-distance latency
│   └── superposition.rs   #   SuperpositionState, migration phase tracking
│
└── contested/             # Layer 8 (Partial) — Contested Environments
    ├── correlation.rs     #   CorrelatedFailureDetector, subnet correlation
    ├── partition.rs       #   PartitionDetector, PartitionPhase, healing
    └── reconcile.rs       #   Log reconciliation, longest-chain-wins, ForkRecord
```

## Adapters

### In-Memory (default)

```rust
use blackstream::{EventBus, EventBusConfig};

let bus = EventBus::new(EventBusConfig::default()).await?;
bus.ingest(Event::from_str(r#"{"token": "hello"}"#)?)?;
```

### Redis

```toml
blackstream = { path = ".", features = ["redis"] }
```

### JetStream

```toml
blackstream = { path = ".", features = ["jetstream"] }
```

### BLTP

```toml
blackstream = { path = ".", features = ["bltp"] }
```

## Language Bindings

All bindings wrap the same Rust core.

### Node.js / Bun

```js
const { Blackstream } = require("@the/net");
const bus = await Blackstream.create({ numShards: 4 });
bus.push(Buffer.from('{"token": "hello"}'));
```

### Python

```python
from blackstream import Blackstream
bus = Blackstream(num_shards=4)
bus.ingest_raw('{"token": "hello"}')
```

### Go

```go
bus, _ := blackstream.New(&blackstream.Config{NumShards: 4})
bus.IngestRaw(`{"token": "hello"}`)
```

## Features

| Feature | Flag | Dependencies |
|---------|------|--------------|
| Redis Streams | `redis` | `redis` crate |
| NATS JetStream | `jetstream` | `async-nats` |
| BLTP transport | `bltp` | `chacha20poly1305`, `snow`, `blake2`, `dashmap`, `socket2`, `ed25519-dalek` |
| Regex filters | `regex` | `regex` crate |
| C FFI | `ffi` | -- |

Default feature is `redis`.

## Building

```bash
# Default (Redis adapter)
cargo build --release

# BLTP only (831 KB binary)
cargo build --release --no-default-features --features bltp

# Everything
cargo build --release --all-features
```

## Tests

```bash
# Unit tests (605 tests)
cargo test --lib --features bltp

# Integration (requires running services)
cargo test --test integration_bltp --features bltp
cargo test --test integration_redis --features redis
cargo test --test integration_jetstream --features jetstream
```

## Benchmarks

```bash
cargo bench --features bltp --bench bltp
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
| `0x0500` | Daemon migration |
| `0x0600` | Subprotocol negotiation |
| `0x0700..0x0702` | Continuity / fork / proof |
| `0x0800..0x0801` | Partition / reconciliation |
| `0x1000..0xEFFF` | Vendor / third-party |
| `0xF000..0xFFFF` | Experimental / ephemeral |

## License

Apache-2.0
