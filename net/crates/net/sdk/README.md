# Net Rust SDK

Ergonomic Rust SDK for the Net mesh network.

The core `net` crate is the engine. This SDK is what Rust developers import.

## Install

```toml
[dependencies]
net-sdk = { path = "sdk" }
```

Features: `redis`, `jetstream`, `net` (mesh transport), `full` (all).

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

## License

Apache-2.0
