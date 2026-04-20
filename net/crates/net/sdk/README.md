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

Behind the `security` feature (bundles `identity` + `capabilities` +
`subnets`, all additive on top of `net`):

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
- Re-exports of `CapabilitySet` / `CapabilityFilter` /
  `CapabilityAnnouncement` / `CapabilityIndex` (local-only
  construction + match-ability today — network-side
  `announce_capabilities` / `find_peers` land next).
- Re-exports of `SubnetId` / `SubnetPolicy` / `SubnetRule` (builder
  hook + gateway wiring land next).

**Treat `Identity::to_bytes()` as secret material** — it's the
32-byte ed25519 seed. The SDK never touches a hardcoded path; where
you put the bytes (disk, vault, enclave, k8s secret) is your call.

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
