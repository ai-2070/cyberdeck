# Rust SDK Plan

Separate crate (`sdk/`) as a new workspace member. The core `net` crate is the engine. The SDK is what Rust developers actually import.

## Phase 1: Core — Net + Builder

```rust
use net_sdk::Net;

let node = Net::builder()
    .shards(4)
    .buffer_capacity(1 << 20)
    .backpressure(Backpressure::DropOldest)
    // Transport selection (mutually exclusive):
    .memory()                           // In-process only
    .redis("redis://localhost:6379")    // Redis Streams
    .jetstream("nats://localhost:4222") // NATS JetStream
    .mesh(MeshConfig::initiator(        // Net encrypted UDP mesh
        "0.0.0.0:9000",
        "peer:9001",
        psk,
        peer_pubkey,
    ))
    .build()
    .await?;
```

### Ingestion

```rust
// Typed via serde (serializes to JSON)
node.emit(&token_event)?;

// Raw bytes (fastest path, matches FFI SDKs)
node.emit_raw(bytes)?;

// Batch
node.emit_batch(&[event1, event2, event3])?;
node.emit_raw_batch(vec![b1, b2, b3])?;
```

### Consumption

```rust
// One-shot poll (matches FFI SDKs)
let response = node.poll(PollRequest { limit: 100, ..Default::default() }).await?;

// Lifecycle
node.stats();
node.flush().await?;
node.shutdown().await?;
```

## Phase 2: Async Streams

The Rust-only advantage. None of the FFI SDKs have this.

```rust
use futures::StreamExt;

// Raw event stream
let mut stream = node.subscribe(SubscribeOpts::default().limit(100));
while let Some(event) = stream.next().await {
    let event = event?;
    println!("{}", std::str::from_utf8(&event.raw)?);
}

// Typed stream (automatic deserialization)
let mut tokens = node.subscribe_typed::<TokenEvent>(SubscribeOpts::default());
while let Some(token) = tokens.next().await {
    process(token?);
}
```

`EventStream` internally manages cursor state and adaptive backoff — polls tightly when events flow, backs off when idle.

## Phase 3: Mesh Features

Exposed via `MeshHandle`, only available when built with mesh transport.

```rust
let mesh = node.mesh().unwrap();

// Capabilities
mesh.capabilities();
mesh.announce(capabilities);

// Topology
mesh.graph();    // local neighborhood (proximity graph)
mesh.routes();   // routing table

// Compute
mesh.host_daemon(my_daemon, config);

// Health
mesh.session_health();
```

## Phase 4: Typed Events

`emit<T: Serialize>` and `subscribe_typed<T: DeserializeOwned>` via serde cover 95% of use cases. A `#[derive(NetEvent)]` macro is deferred — if added later, it lives in a separate `net-sdk-derive` crate:

```rust
#[derive(NetEvent)]
#[net(channel = "tokens")]
struct TokenEvent {
    token: String,
    index: u32,
}
```

## Phase 5: Ready-Made Components

The core crate is minimal. The SDK is not. Ship components that developers would otherwise build themselves.

### Daemons

Pre-built `MeshDaemon` implementations for common patterns:

```rust
use net_sdk::daemons::{Router, Aggregator, Mirror, Gateway};

// Route events between channels based on rules
node.host(Router::new()
    .route("sensors/*", "analytics/ingest")
    .route("alerts/*", "ops/alerts")
);

// Aggregate events over a time window
node.host(Aggregator::new("sensors/temperature")
    .window(Duration::from_secs(5))
    .reduce(|events| avg(events))
    .emit_to("analytics/temperature_avg")
);

// Mirror a channel to a persistence adapter
node.host(Mirror::new("critical/*").to_redis("redis://localhost:6379"));

// Bridge between subnets
node.host(Gateway::new()
    .expose("sensors/public/*")
    .hide("sensors/internal/*")
);
```

### Middleware

Composable event processing pipeline:

```rust
use net_sdk::middleware::{Logger, RateLimit, Transform, Filter};

let node = Net::builder()
    .shards(4)
    .mesh(mesh_config)
    .middleware(Logger::new())
    .middleware(RateLimit::per_second(10_000))
    .middleware(Filter::channel("sensors/*"))
    .middleware(Transform::map(|event| enrich(event)))
    .build()
    .await?;
```

### Primitives

Building blocks for custom daemons:

```rust
use net_sdk::primitives::{
    Fanout,          // one-to-many event distribution
    Merge,           // many-to-one event collection
    Dedup,           // idempotency by event hash
    Buffer,          // bounded in-memory buffer with backpressure
    Retry,           // retry with exponential backoff
    CircuitBreaker,  // fail-fast on degraded peers
    Ticker,          // periodic event emission
    Watchdog,        // health monitoring with timeout
};
```

### Inference

First-class support for the primary use case:

```rust
use net_sdk::inference::{InferenceNode, ModelConfig};

let node = Net::builder()
    .mesh(mesh_config)
    .build()
    .await?;

// Announce this node serves inference
node.host(InferenceNode::new()
    .model(ModelConfig::new("gemma-21b").vram_gb(24))
    .max_concurrent(8)
    .channel("inference/requests")
    .respond_to("inference/responses")
);
```

```rust
// Request inference from the mesh (routes to best available node)
let response = node.infer("inference/requests", prompt).await?;
```

### Discovery

Find nodes by what they can do:

```rust
use net_sdk::discover::Query;

let gpu_nodes = node.discover(Query::capability("gpu").min_vram(16)).await;
let nearby = node.discover(Query::within_hops(3)).await;
let models = node.discover(Query::serves_model("gemma-21b")).await;
```

### Typed Channels

Strongly typed pub/sub over named channels:

```rust
use net_sdk::channel::TypedChannel;

let temps: TypedChannel<TemperatureReading> = node.channel("sensors/temperature");
temps.publish(&reading)?;

let mut sub = temps.subscribe();
while let Some(reading) = sub.next().await {
    process(reading?);
}
```

### Testing Utilities

Spin up a local mesh in tests. Simulate partitions, failures, recovery.

```rust
use net_sdk::testing::TestMesh;

let mesh = TestMesh::new(3).await;  // 3 interconnected nodes
mesh[0].emit(&event)?;
let received = mesh[2].poll_one().await?;
mesh.partition(0, 2);               // simulate network partition
mesh.heal();                         // restore connectivity
```

### Metrics / Observability

Built-in hooks for monitoring:

```rust
node.on_event(|metric| match metric {
    Metric::EventIngested { latency, shard } => ...,
    Metric::PeerDiscovered { node_id, hops } => ...,
    Metric::DaemonMigrated { from, to, duration } => ...,
    Metric::RouteChanged { stream, old, new } => ...,
});
```

### Scheduled Events

Time-based emission on the mesh:

```rust
use net_sdk::schedule::Cron;

node.host(Cron::every(Duration::from_secs(30))
    .emit_to("health/heartbeat")
    .payload(|| node.stats())
);
```

## SDK vs FFI Parity

| FFI SDKs (Go/Python/Node) | Rust SDK |
|---|---|
| `ingest_raw(json_str)` | `node.emit_raw(bytes)` |
| `ingest_raw_batch(strs)` | `node.emit_raw_batch(vec)` |
| `poll(limit, cursor)` | `node.poll(PollRequest { .. })` |
| `stats()` | `node.stats()` |
| `shutdown()` | `node.shutdown().await` |
| — | `node.subscribe(opts)` *Rust-only* |
| — | `node.subscribe_typed::<T>(opts)` *Rust-only* |
| — | `node.mesh()` *Rust-only* |

## Structure

```
sdk/
  Cargo.toml         # net-sdk crate, depends on net
  src/
    lib.rs           # re-exports, prelude
    net.rs           # Net — the main handle
    config.rs        # NetConfig builder
    stream.rs        # EventStream, TypedEventStream<T>
    mesh.rs          # MeshHandle — mesh topology, capabilities, daemon hosting
    error.rs         # unified SDK error type
  tests/
    integration.rs
```

## Changes to Core `net` Crate

1. **`Cargo.toml`** — add `"sdk"` to workspace members
2. **`src/bus.rs`** — add `EventBus::with_adapter()` constructor so the SDK can hold a reference to `NetAdapter` for mesh features
3. **`src/adapter/mod.rs`** — optionally add `as_any()` to `Adapter` trait for downcast support

## Dependencies

`net` (path), `serde`, `serde_json`, `bytes`, `futures`, `tokio`, `pin-project-lite`, `thiserror`
