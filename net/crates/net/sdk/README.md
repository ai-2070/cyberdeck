# Net Rust SDK

Ergonomic Rust SDK for the Net mesh network.

The core `net` crate is the engine. This SDK is what Rust developers import.

## Install

```toml
[dependencies]
net-sdk = { path = "sdk" }
```

Features: `redis`, `jetstream`, `net` (mesh transport), `cortex` (event-sourced tasks/memories + NetDb), `full` (all).

## Quick Start

```rust
use net_sdk::{Net, Backpressure};
use futures::StreamExt;

#[tokio::main]
async fn main() -> net_sdk::error::Result<()> {
    let node = Net::builder()
        .shards(4)
        .backpressure(Backpressure::DropOldest)
        .memory()
        .build()
        .await?;

    // Emit events
    node.emit(&serde_json::json!({"token": "hello", "index": 0}))?;
    node.emit_raw(b"{\"token\": \"world\"}" as &[u8])?;
    node.emit_str("{\"token\": \"foo\"}")?;

    // Batch
    let count = node.emit_batch(&[
        serde_json::json!({"a": 1}),
        serde_json::json!({"a": 2}),
    ])?;

    node.flush().await?;

    // Poll
    let response = node.poll(net_sdk::PollRequest {
        limit: 100,
        ..Default::default()
    }).await?;

    for event in &response.events {
        println!("{}", event.raw_str());
    }

    // Stream
    let mut stream = node.subscribe(Default::default());
    while let Some(event) = stream.next().await {
        println!("{}", event?.raw_str());
    }

    node.shutdown().await
}
```

## Typed Streams

```rust
use serde::Deserialize;
use futures::StreamExt;

#[derive(Deserialize)]
struct TokenEvent {
    token: String,
    index: u32,
}

let mut stream = node.subscribe_typed::<TokenEvent>(Default::default());
while let Some(token) = stream.next().await {
    let token = token?;
    println!("{}: {}", token.index, token.token);
}
```

## Ingestion Methods

| Method | Input | Speed | Returns |
|--------|-------|-------|---------|
| `emit(&T)` | Any `Serialize` | Fast | `Receipt` |
| `emit_raw(bytes)` | `impl Into<Bytes>` | Fastest | `Receipt` |
| `emit_str(json)` | `&str` | Fast | `Receipt` |
| `emit_batch(&[T])` | Slice of `Serialize` | Bulk | `usize` |
| `emit_raw_batch(Vec<Bytes>)` | Raw byte vecs | Bulk fastest | `usize` |

## Transports

```rust
// In-memory (default, single process)
Net::builder().memory()

// Redis Streams
Net::builder().redis(RedisAdapterConfig::new("redis://localhost:6379"))

// NATS JetStream
Net::builder().jetstream(JetStreamAdapterConfig::new("nats://localhost:4222"))

// Encrypted UDP mesh
Net::builder().mesh(NetAdapterConfig::initiator(bind, peer, psk, peer_pubkey))
```

## Mesh Streams (multi-peer + back-pressure)

For direct peer-to-peer messaging outside the event bus — open a typed
stream to a specific peer, send batches, and react to back-pressure:

```rust
use net_sdk::{Mesh, MeshBuilder, StreamConfig, Reliability};
use net_sdk::error::SdkError;
use bytes::Bytes;

let node = MeshBuilder::new("127.0.0.1:9000", &[0x42u8; 32])?
    .build()
    .await?;

// ... handshake with a peer via node.inner().connect(...) ...

// Open a per-peer stream with explicit reliability + back-pressure window.
let stream = node.open_stream(
    peer_node_id,
    0x42,
    StreamConfig::new()
        .with_reliability(Reliability::Reliable)
        .with_window_bytes(256),   // max in-flight packets before Backpressure
)?;

// Three canonical daemon patterns:

// 1. Drop on pressure — best for telemetry / sampled streams.
match node.send_on_stream(&stream, &[Bytes::from_static(b"{}")]).await {
    Ok(()) => {}
    Err(SdkError::Backpressure) => metrics::inc("stream.backpressure_drops"),
    Err(SdkError::NotConnected) => {/* peer gone or stream closed */}
    Err(e) => tracing::warn!(error = %e, "transport error"),
}

// 2. Retry with exponential backoff — best for important events.
node.send_with_retry(&stream, &[Bytes::from_static(b"{}")], 8).await?;

// 3. Block until the network lets up (bounded retry, ~13 min worst case).
node.send_blocking(&stream, &[Bytes::from_static(b"{}")]).await?;

// Live stats — per-stream tx/rx seq, in-flight, window, backpressure count.
let stats = node.stream_stats(peer_node_id, 0x42);
```

`SdkError::Backpressure` is a signal, not a policy — the transport never
retries or buffers on its own behalf. `StreamStats.backpressure_events`
counts cumulative rejections for observability. See
[`docs/TRANSPORT.md`](../docs/TRANSPORT.md) for the full contract and
[`docs/STREAM_BACKPRESSURE_PLAN.md`](../docs/STREAM_BACKPRESSURE_PLAN.md)
for the design.

## Security (identity, tokens, capabilities, subnets)

Identity, capabilities, and subnets ride the `net` feature as a
single security unit — they share the mesh's subprotocol dispatch
and operate together at runtime (subnet enforcement reuses the
capability broadcast; channel auth threads identity + capabilities
+ subnets together), so `--features net` gives you the whole
surface:

```rust
use std::time::Duration;
use net_sdk::{Identity, TokenScope};
use net_sdk::mesh::{Mesh, MeshBuilder};
use net_sdk::ChannelName;

# async fn example() -> net_sdk::error::Result<()> {
// Load once from caller-owned storage (vault / k8s secret / enclave).
let seed: [u8; 32] = [/* 32 bytes */ 0x42u8; 32];
let id = Identity::from_seed(seed);

// Stable node id across restarts — derived from the ed25519 seed.
println!("node_id = {:#x}", id.node_id());

// Issue a scoped subscribe grant to a peer and hand it over.
let subscriber_entity = Identity::generate(); // pretend we received this
let channel = ChannelName::new("sensors/temp").unwrap();
let token = id.issue_token(
    subscriber_entity.entity_id().clone(),
    TokenScope::SUBSCRIBE,
    &channel,
    Duration::from_secs(300),
    0, // delegation depth — 0 forbids re-delegation
);
// Token is a signed, transport-ready blob.
assert_eq!(token.to_bytes().len(), net_sdk::PermissionToken::WIRE_SIZE);

// Pin this identity on the mesh builder — without this call the
// builder generates an ephemeral keypair and the node_id changes on
// every restart.
let _mesh = MeshBuilder::new("127.0.0.1:9001", &[0x42u8; 32])?
    .identity(id)
    .build()
    .await?;
# Ok(())
# }
```

**What's wired in this release:**

- `Identity` generation / seed round-trip / signing / token issuance
  + verification + install + lookup.
- `MeshBuilder::identity(...)` pins the keypair used by the mesh's
  Noise handshake so `node_id()` is stable.
- **Capability announcements — cross-node (direct-peer).** See the
  subsection below.
- Re-exports of `SubnetId` / `SubnetPolicy` / `SubnetRule` (builder
  hook + gateway wiring land next).

**Treat `Identity::to_bytes()` as secret material** — it's the
32-byte ed25519 seed. The SDK never touches a hardcoded path; where
you put the bytes (disk, vault, enclave, k8s secret) is your call.

### Capability announcements

`Mesh::announce_capabilities(caps)` pushes a `CapabilityAnnouncement`
to every directly-connected peer and self-indexes locally.
`Mesh::find_peers(filter)` queries the local index — results include
this node's own id when self matches.

```rust
use net_sdk::capabilities::{CapabilityFilter, CapabilitySet, GpuInfo, GpuVendor, HardwareCapabilities};
use net_sdk::mesh::MeshBuilder;

# async fn example() -> net_sdk::error::Result<()> {
let mesh = MeshBuilder::new("127.0.0.1:0", &[0x42u8; 32])?
    .build()
    .await?;

let hw = HardwareCapabilities::new()
    .with_cpu(16, 32)
    .with_memory(65_536)
    .with_gpu(GpuInfo::new(GpuVendor::Nvidia, "RTX 4090", 24_576));
mesh.announce_capabilities(
    CapabilitySet::new().with_hardware(hw).add_tag("gpu"),
)
.await?;

// Self-match: returns our own node_id.
let hits = mesh.find_peers(
    &CapabilityFilter::new().require_gpu().with_min_vram(16_384),
);
assert!(hits.contains(&mesh.node_id()));
mesh.shutdown().await?;
# Ok(())
# }
```

**Scope today:**

- Multi-hop fan-out bounded by `MAX_CAPABILITY_HOPS = 16`.
  Forwarders re-broadcast every received announcement to their
  other peers (minus the sender and any split-horizon peer),
  bumping `hop_count` outside the signed envelope so the origin's
  signature keeps verifying end-to-end. Dedup on
  `(origin, version)` drops duplicates at diamond-topology
  converge points. See
  [`docs/MULTIHOP_CAPABILITY_PLAN.md`](../docs/MULTIHOP_CAPABILITY_PLAN.md).
- Origin-side rate limiting: `min_announce_interval` (default 10s)
  coalesces rapid `announce_capabilities` calls into a single
  broadcast, preventing a busy-loop announcer from flooding the
  mesh. Self-index + late-joiner session-open push still reflect
  the latest caps inside the window.
- TTL + GC eviction: per-announcement `ttl_secs` drives
  `CapabilityIndex::gc()` on a configurable tick
  (`capability_gc_interval`, default 60 s).
- Signatures are advisory. The `require_signed_capabilities` config
  knob rejects unsigned announcements at the receiver, but
  *signature validity* is not enforced end-to-end yet — it requires
  a `node_id → entity_id` binding that lands with channel auth.

Wire-level details and the subprotocol layout live in
[`docs/CAPABILITY_BROADCAST_PLAN.md`](../docs/CAPABILITY_BROADCAST_PLAN.md).

### Subnets (visibility partitioning)

`MeshBuilder::subnet(id)` pins a node to one of 2³² possible 4-level
subnet ids; `subnet_policy(policy)` derives each *peer's* subnet by
applying a shared tag-matching policy to their inbound
`CapabilityAnnouncement`. Channel visibility then gates publish
fan-out and subscribe authorization against that geometry.

```rust
use std::sync::Arc;
use net_sdk::capabilities::CapabilitySet;
use net_sdk::mesh::MeshBuilder;
use net_sdk::subnets::{SubnetId, SubnetPolicy, SubnetRule};

# async fn example() -> net_sdk::error::Result<()> {
// Mesh-wide policy: `region:<x>` maps to the level 0 byte,
// `fleet:<x>` maps to level 1. Every node in the mesh must
// construct the same policy — mismatched policies yield
// asymmetric views of peer subnets.
let policy = Arc::new(
    SubnetPolicy::new()
        .add_rule(SubnetRule::new("region:", 0).map("us", 3).map("eu", 4))
        .add_rule(SubnetRule::new("fleet:", 1).map("blue", 7).map("green", 8)),
);

let node = MeshBuilder::new("127.0.0.1:0", &[0x42u8; 32])?
    .subnet(SubnetId::new(&[3, 7]))           // us/blue
    .subnet_policy(policy)
    .build()
    .await?;

// Announce with tags matching the policy so peers derive the same
// subnet (`[3, 7]`) when they apply their own policy to our caps.
node.announce_capabilities(
    CapabilitySet::new()
        .add_tag("region:us")
        .add_tag("fleet:blue"),
)
.await?;

// Register a SubnetLocal channel — only peers with the exact same
// SubnetId will be accepted as subscribers and included in publish
// fan-out. Any cross-subnet subscribe rejects with `Unauthorized`.
// (Channel registration uses the channel config types from the
// `Channels` section below.)
node.shutdown().await?;
# Ok(())
# }
```

**Visibility semantics** (from `Visibility` enum):

| Variant | Delivery |
|---|---|
| `Global` | every peer |
| `SubnetLocal` | peers with an identical `SubnetId` |
| `ParentVisible` | same subnet OR either side is an ancestor of the other |
| `Exported` | per-channel export table — **deferred**, drops in v1 |

**Scope today**:

- Enforcement is end-to-end through the publish + subscribe gates.
  Filtered subscribers do not appear in `PublishReport.attempted`.
- Peer subnets are derived locally from each peer's capability
  announcement via `SubnetPolicy::assign`. No dedicated subnet
  subprotocol; announcements piggyback on the capability broadcast
  from Stage C.
- Multi-hop subnet-aware routing (forwarding filters at the packet
  header) is a follow-up.

Wire-level details and the enforcement matrix live in
[`docs/SUBNET_ENFORCEMENT_PLAN.md`](../docs/SUBNET_ENFORCEMENT_PLAN.md).

### Channel authentication

`ChannelConfig` carries three auth knobs that are now enforced
end-to-end at both the subscribe gate and the publish path:

- `publish_caps: CapabilityFilter` — publisher must satisfy before
  fan-out. Failing publishes return an `AdapterError`; no peers are
  attempted.
- `subscribe_caps: CapabilityFilter` — subscribers must satisfy
  before being added to the roster. Failures surface as
  `SdkError::ChannelRejected(Some(Unauthorized))`.
- `require_token: bool` — subscribers must present a valid
  `PermissionToken` whose subject matches their entity id. The
  token rides on the subscribe message; the publisher verifies the
  ed25519 signature on arrival, installs it in its local
  `TokenCache`, then runs `can_subscribe`.

```rust
use std::sync::Arc;
use std::time::Duration;
use net_sdk::capabilities::{CapabilityFilter, CapabilitySet};
use net_sdk::mesh::MeshBuilder;
use net_sdk::{
    ChannelConfig, ChannelId, ChannelName, Identity, PublishConfig, Reliability,
    SubscribeOptions, TokenScope,
};
# async fn example() -> net_sdk::error::Result<()> {
// Both sides bind caller-owned identities so tokens + entity_ids
// are load-bearing.
let publisher_identity = Identity::generate();
let subscriber_identity = Identity::generate();

let publisher = MeshBuilder::new("127.0.0.1:9001", &[0x42u8; 32])?
    .identity(publisher_identity.clone())
    .build()
    .await?;

// Register a channel that requires `gpu` AND a token.
let name = ChannelName::new("events/inference").unwrap();
let filter = CapabilityFilter::new().require_tag("gpu");
publisher.register_channel(
    ChannelConfig::new(ChannelId::new(name.clone()))
        .with_subscribe_caps(filter)
        .with_require_token(true),
);

// Issue a SUBSCRIBE-scope token for the subscriber. This also
// pre-caches it in the publisher's identity (unused for this
// flow since the subscriber will present the same token on the
// wire, but useful for the "pre-seed" pattern).
let token = publisher_identity.issue_token(
    subscriber_identity.entity_id().clone(),
    TokenScope::SUBSCRIBE,
    &name,
    Duration::from_secs(300),
    0,
);

// Subscriber attaches the token.
let subscriber: &net_sdk::Mesh = unimplemented!();
subscriber
    .subscribe_channel_with(
        publisher.node_id(),
        &name,
        SubscribeOptions { token: Some(token) },
    )
    .await?;
# Ok(())
# }
```

**Scope today**:

- Full enforcement at subscribe + publish; empty-caps / missing-
  entity defaults fail closed when `require_token` is set.
- Every publish fan-out consults the `AuthGuard` fast path (4 KB
  bloom filter + verified-subscribe cache) so revocations apply on
  the next publish without a roster refresh. Single-threaded
  microbenchmark: ~20 ns per `check_fast` call.
- Periodic token-expiry sweep (default 30 s,
  `MeshNodeConfig::with_token_sweep_interval`) evicts subscribers
  whose tokens age out of their TTL — they stop receiving events
  within one sweep tick instead of staying on the roster forever.
- Per-peer auth-failure rate limiter (`with_auth_failure_limit`,
  default 16 failures per 60 s window → 30 s throttle) short-
  circuits bad-token subscribe storms with `AckReason::RateLimited`
  before ed25519 verification runs. Successful subscribes clear
  the counter.
- `CapabilityAnnouncement` now carries the sender's `entity_id` and
  is signed — verified end-to-end (closes the "signature advisory"
  caveat from the capability section above).
- `node_id → entity_id` is pinned on first sight (TOFU); rebind
  attempts in later announcements are silently rejected.
- Any auth-rule denial surfaces as `AckReason::Unauthorized`;
  throttled bursts surface as `AckReason::RateLimited`. Sub-reasons
  within the auth rejection (cap-failed vs token-failed vs
  subnet-failed) are not split yet.

Wire-format details and the token presentation flow live in
[`docs/CHANNEL_AUTH_PLAN.md`](../docs/CHANNEL_AUTH_PLAN.md); the
fast-path / sweep / rate-limit design lives in
[`docs/CHANNEL_AUTH_GUARD_PLAN.md`](../docs/CHANNEL_AUTH_GUARD_PLAN.md).

## Channels (distributed pub/sub)

Named pub/sub over the encrypted mesh. Publishers register channels
with access policy; subscribers ask to join via a membership
subprotocol with an Ack round-trip. `publish` / `publish_many` fan
payloads out to every current subscriber.

```rust
use bytes::Bytes;
use net_sdk::mesh::{Mesh, MeshBuilder};
use net_sdk::{ChannelConfig, ChannelId, ChannelName, PublishConfig, Reliability, Visibility};

# async fn example() -> net_sdk::error::Result<()> {
let publisher = MeshBuilder::new("127.0.0.1:9001", &[0x42u8; 32])?
    .build().await?;
let subscriber = MeshBuilder::new("127.0.0.1:9000", &[0x42u8; 32])?
    .build().await?;
// (handshake omitted — see Mesh Streams example)

// Publisher registers a channel.
let channel = ChannelName::new("sensors/temp").unwrap();
publisher.register_channel(
    ChannelConfig::new(ChannelId::new(channel.clone()))
        .with_visibility(Visibility::Global)
        .with_reliable(true)
        .with_priority(2),
);

// Subscriber joins. Network-rejected acks surface as
// `SdkError::ChannelRejected(reason)`.
subscriber.subscribe_channel(publisher.inner().node_id(), &channel).await?;

// Fan out.
let report = publisher.publish(
    &channel,
    Bytes::from_static(b"22.5"),
    PublishConfig {
        reliability: Reliability::Reliable,
        ..Default::default()
    },
).await?;
println!("{}/{} delivered", report.delivered, report.attempted);
# Ok(())
# }
```

`register_channel` stores into a shared `ChannelConfigRegistry`
installed on the underlying `MeshNode` at build time — so multiple
`register_channel` calls are just inserts and require only `&Mesh`,
not `&mut`.

Subscribers today receive payloads via the existing `recv` /
`recv_shard` surface. A dedicated `on_channel(&ChannelName)` stream
is a follow-up.

## CortEX & NetDb (event-sourced state)

For typed, event-sourced state — tasks and memories with filterable
queries and reactive watches — enable the `cortex` feature and import
from `net_sdk::cortex`:

```rust
use net_sdk::cortex::{NetDb, Redex, TaskStatus};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redex = Redex::new();                       // or `.with_persistent_dir("/var/lib/net")`
    let db = NetDb::builder(redex)
        .origin(0xABCD_EF01)                        // producer identity on every event
        .with_tasks()
        .with_memories()
        .build()?;

    // Ingest through the domain API; no EventMeta plumbing.
    let seq = db.tasks().create(1, "write docs", 0)?;
    db.tasks().wait_for_seq(seq).await;             // wait for the fold to apply

    // Query the materialized state.
    assert_eq!(db.tasks().count(), 1);

    // Snapshot + watch: "paint what's there now, then react to changes."
    // The stream drops only leading emissions that equal the snapshot,
    // so a mutation racing during construction is still delivered.
    let watcher = db.tasks().watch().where_status(TaskStatus::Pending);
    let (snapshot, mut deltas) = db.tasks().snapshot_and_watch(watcher);
    println!("initial: {} pending", snapshot.len());
    while let Some(batch) = deltas.next().await {
        println!("delta: {} pending", batch.len());
    }
    Ok(())
}
```

### Persistence

With `redex-disk` (pulled in by `cortex`), point `Redex` at a directory
and flip `persistent(true)` on the builder:

```rust
let redex = Redex::new().with_persistent_dir("/var/lib/net/redex");
let db = NetDb::builder(redex)
    .origin(origin_hash)
    .persistent(true)
    .with_tasks()
    .build()?;
```

Use `RedexFileConfig` + `FsyncPolicy` (both re-exported from
`net_sdk::cortex`) to tune per-file fsync semantics.

### Raw RedEX file

For domain-agnostic persistent logs (no CortEX, no fold, no typed
state), use the `Redex` manager directly via `Redex::open_file`. This
unlocks `RedexFile::append` / `tail` for custom event pipelines.

See [`docs/STORAGE_AND_CORTEX.md`](../docs/STORAGE_AND_CORTEX.md) for
the full narrative and
[`docs/REDEX_PLAN.md`](../docs/REDEX_PLAN.md) /
[`docs/CORTEX_ADAPTER_PLAN.md`](../docs/CORTEX_ADAPTER_PLAN.md) for the
design.

## API

| Method | Description |
|--------|-------------|
| `Net::builder()` | Create a configuration builder |
| `emit(&T)` | Emit a serializable event |
| `emit_raw(bytes)` | Emit raw bytes (fastest) |
| `emit_str(json)` | Emit a JSON string |
| `emit_batch(&[T])` | Batch emit |
| `emit_raw_batch(vecs)` | Batch emit raw bytes |
| `poll(request)` | One-shot poll |
| `subscribe(opts)` | Async event stream |
| `subscribe_typed::<T>(opts)` | Typed async stream |
| `stats()` | Ingestion statistics |
| `shards()` | Number of active shards |
| `health()` | Check node health |
| `flush()` | Flush pending batches |
| `shutdown()` | Graceful shutdown |
| `bus()` | Access underlying `EventBus` |

### Channel surface (feature `net`)

| Method | Description |
|---|---|
| `mesh.register_channel(config)` | Install / replace a channel's access config |
| `mesh.subscribe_channel(peer_id, &name)` | Ask `peer_id` to add us as a subscriber |
| `mesh.unsubscribe_channel(peer_id, &name)` | Leave a channel (idempotent) |
| `mesh.publish(&name, bytes, cfg)` | Fan one payload to all subscribers |
| `mesh.publish_many(&name, &[bytes], cfg)` | Fan a batch to all subscribers |
| `SdkError::ChannelRejected(reason)` | Typed subscribe/unsubscribe rejection |

### CortEX surface (feature `cortex`)

| Entry point | Description |
|---|---|
| `cortex::Redex::new()` | In-memory event-log manager |
| `cortex::Redex::with_persistent_dir(path)` | Disk-backed manager |
| `cortex::NetDb::builder(redex)` | Fluent `NetDb` construction |
| `cortex::TasksAdapter::open(redex, origin)` | Open tasks model standalone |
| `cortex::MemoriesAdapter::open(redex, origin)` | Open memories model standalone |
| `db.tasks() / db.memories()` | Typed adapter handles on `NetDb` |
| `adapter.snapshot_and_watch(watcher)` | Atomic initial-result + delta stream |
| `db.snapshot()` | `NetDbSnapshot` bundle for persistence |
| `NetDb::builder(...).build_from_snapshot(&bundle)` | Restore from bundle |

## License

Apache-2.0
