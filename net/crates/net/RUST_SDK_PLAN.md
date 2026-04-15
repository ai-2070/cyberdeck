# Rust SDK Plan

Separate crate (`sdk/`) as a new workspace member. The core `net` crate is the engine. The SDK is what Rust developers actually import.

## Phase 1: Core — NetClient + Builder

```rust
use net_sdk::NetClient;

let client = NetClient::builder()
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
client.emit(&token_event)?;

// Raw bytes (fastest path, matches FFI SDKs)
client.emit_raw(bytes)?;

// Batch
client.emit_batch(&[event1, event2, event3])?;
client.emit_raw_batch(vec![b1, b2, b3])?;
```

### Consumption

```rust
// One-shot poll (matches FFI SDKs)
let response = client.poll(PollRequest { limit: 100, ..Default::default() }).await?;

// Lifecycle
client.stats();
client.flush().await?;
client.shutdown().await?;
```

## Phase 2: Async Streams

The Rust-only advantage. None of the FFI SDKs have this.

```rust
use futures::StreamExt;

// Raw event stream
let mut stream = client.subscribe(SubscribeOpts::default().limit(100));
while let Some(event) = stream.next().await {
    let event = event?;
    println!("{}", std::str::from_utf8(&event.raw)?);
}

// Typed stream (automatic deserialization)
let mut tokens = client.subscribe_typed::<TokenEvent>(SubscribeOpts::default());
while let Some(token) = tokens.next().await {
    process(token?);
}
```

`EventStream` internally manages cursor state and adaptive backoff — polls tightly when events flow, backs off when idle.

## Phase 3: Mesh Features

Exposed via `MeshHandle`, only available when built with mesh transport.

```rust
let mesh = client.mesh().unwrap();

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

## SDK vs FFI Parity

| FFI SDKs (Go/Python/Node) | Rust SDK |
|---|---|
| `ingest_raw(json_str)` | `client.emit_raw(bytes)` |
| `ingest_raw_batch(strs)` | `client.emit_raw_batch(vec)` |
| `poll(limit, cursor)` | `client.poll(PollRequest { .. })` |
| `stats()` | `client.stats()` |
| `shutdown()` | `client.shutdown().await` |
| — | `client.subscribe(opts)` *Rust-only* |
| — | `client.subscribe_typed::<T>(opts)` *Rust-only* |
| — | `client.mesh()` *Rust-only* |

## Structure

```
sdk/
  Cargo.toml         # net-sdk crate, depends on net
  src/
    lib.rs           # re-exports, prelude
    client.rs        # NetClient — the main handle
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
