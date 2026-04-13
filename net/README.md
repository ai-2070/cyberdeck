# Net

High-performance event bus and encrypted mesh transport for AI runtime workloads.

Net is the networking layer of the cyberdeck. It ingests, relays, and replays AI-generated streaming output at GPU-native speeds -- token streams, multi-agent event flows, tool-use results, guardrail decisions, consensus votes, and session lifecycle events.

## Performance

Benchmarked on Apple M1 Max, macOS.

| Layer | Operation | Latency | Throughput |
|-------|-----------|---------|------------|
| **Core** | Event ingestion | < 1 us p99 | 10M+ events/sec sustained |
| **BLTP** | Header serialize | 1.9 ns | 526M ops/sec |
| **BLTP** | Packet build (50 events) | 10.4 us | -- |
| **BLTP** | Encryption (ChaCha20) | 709 ns (64B) | -- |
| **Routing** | Header roundtrip | 0.94 ns | 1.07G ops/sec |
| **Routing** | Lookup hit | 14.9 ns | 67M ops/sec |
| **Routing** | Decision pipeline | 15.3 ns | 65M ops/sec |
| **Forwarding** | Per-hop (64B) | 28.9 ns | -- |
| **Forwarding** | 5-hop chain | 277 ns | 3.6M ops/sec |
| **Swarm** | Pingwave roundtrip | 0.94 ns | 1.06G ops/sec |
| **Swarm** | Graph (5,000 nodes) | 173 us | 28.8M/sec |
| **Failure** | Heartbeat | 29.2 ns | 34.2M ops/sec |
| **Failure** | Full recovery cycle | 312 ns | 3.2M ops/sec |
| **Capability** | Filter (single tag) | 10 ns | 99.8M ops/sec |
| **Capability** | GPU check | 0.32 ns | 3.13G ops/sec |
| **SDK** | Go raw ingest | 377 ns | 2.65M/sec |
| **SDK** | Python batch ingest | 0.36 us | 2.78M/sec |
| **SDK** | Node.js push batch | 0.35 us | 2.89M/sec |
| **SDK** | Bun batch ingest | 0.30 us | 3.37M/sec |

Thread-local packet pools scale to **32x contention advantage** over shared pools at 32 threads. Multi-hop forwarding scales linearly with hop count. All SDKs exceed **2M events/sec** with optimal ingestion patterns.

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
                              ┌─────────────────┼─────────────────┐
                              │                 │                 │
                        ┌─────┴─────┐    ┌──────┴──────┐   ┌─────┴──────┐
                        │  Routing   │    │   Swarm /   │   │  Failure   │
                        │  & Proxy   │    │  Discovery  │   │ Detection  │
                        │ (multi-hop)│    │ (pingwave)  │   │ & Recovery │
                        └───────────┘    └────────────┘   └────────────┘
```

**EventBus** is the core API. Events flow into sharded ring buffers with adaptive batching and backpressure. Adapters push events to backends.

**Adapters** are pluggable backends:

- **Redis Streams** -- durable event storage via Redis
- **JetStream** -- NATS JetStream for distributed messaging
- **BLTP** -- custom encrypted UDP transport with Noise protocol handshakes and ChaCha20-Poly1305 encryption. Zero-copy forwarding, multi-hop proxy routing, and no intermediate node ever touches plaintext payloads.

**BLTP Mesh** layers on top of the transport:

- **Routing** -- per-stream routing tables, fair scheduling, sub-nanosecond header operations
- **Swarm** -- pingwave-based node discovery, local graph maintenance within a configurable hop radius, capability advertisements
- **Failure detection** -- 29ns heartbeats, circuit breakers, recovery manager with automatic rerouting
- **Capability system** -- declarative hardware/software/model/tool capability sets, filtered queries across the mesh at 100-300M ops/sec
- **Behavior plane** -- load balancing, autonomy rules, safety envelopes, context fabric, API schemas, proximity-weighted capability graphs

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

BLTP is a custom binary protocol over UDP with:

- Noise NK handshakes for key exchange
- ChaCha20-Poly1305 authenticated encryption
- Adaptive batching with zero-allocation packet pools
- Selective NACK reliability or fire-and-forget mode
- Multi-hop forwarding where relay nodes never decrypt payloads

## Language Bindings

All bindings wrap the same Rust core. Performance differences come from FFI overhead and serialization.

### Node.js / Bun

```js
const { Blackstream } = require("@the/net");

const bus = await Blackstream.create({ numShards: 4 });
bus.push(Buffer.from('{"token": "hello"}'));

const response = await bus.poll({ limit: 100 });
```

Sync `push()` and `pushBatch()` methods bypass the async boundary for maximum throughput. Bun achieves 3.37M events/sec on batch ingestion.

### Python

```python
from blackstream import Blackstream

bus = Blackstream(num_shards=4)
bus.ingest_raw('{"token": "hello"}')

response = bus.poll(limit=100)
for event in response:
    print(event.parse())
```

### Go

```go
import "github.com/ai-2070/blackstream/bindings/go/blackstream"

bus, _ := blackstream.New(&blackstream.Config{NumShards: 4})
bus.IngestRaw(`{"token": "hello"}`)

resp, _ := bus.Poll(100, "")
```

Zero-allocation raw ingestion at 2.65M events/sec.

## Features

| Feature | Flag | Dependencies |
|---------|------|--------------|
| Redis Streams | `redis` | `redis` crate |
| NATS JetStream | `jetstream` | `async-nats` |
| BLTP transport | `bltp` | `chacha20poly1305`, `snow`, `blake2`, `dashmap`, `socket2` |
| Regex filters | `regex` | `regex` crate |
| C FFI | `ffi` | -- |

Default feature is `redis`.

## Building

```bash
cd net/crates/net

# Default (Redis adapter)
cargo build --release

# BLTP only
cargo build --release --no-default-features --features bltp

# Everything
cargo build --release --all-features
```

## Tests

```bash
# Unit tests
cargo test --lib --all-features

# Integration (requires running services)
cargo test --test integration_redis --features redis
cargo test --test integration_jetstream --features jetstream
cargo test --test integration_bltp --features bltp
```

## Benchmarks

```bash
# Core + BLTP
cargo bench --features bltp --bench bltp

# Ingestion
cargo bench --bench ingestion

# Parallel scaling
cargo bench --bench parallel

# SDK benchmarks
cd bindings/go/blackstream && go test -bench=. -benchmem .
cd bindings/python && uv run python tests/benchmark.py
cd bindings/node && npm run bench
```

## Design

The transport layer is fast enough that network operations become viable at timescales traditionally reserved for local function calls. 55ns per hop means state can propagate across a mesh faster than a method call in most languages. 29ns heartbeats mean a node's failure is detected before the OS on that node finishes crashing. 312ns recovery means traffic has already rerouted before the hardware hits the ground.

This floor enables a mesh where nodes discover each other through pingwaves, advertise capabilities, route traffic through encrypted multi-hop paths, detect and recover from failures autonomously, and balance load across heterogeneous hardware -- all without centralized coordination.

Each node observes its local neighborhood and derives the state of the wider mesh from those observations. There is no global consensus, no coordinator, no privileged node. Consistency emerges from causal ordering and the speed of propagation, the same way it does in physics. Two nodes may disagree about the mesh state at a given instant, but their views are always causally consistent within their own observation window.

## License

Apache-2.0
