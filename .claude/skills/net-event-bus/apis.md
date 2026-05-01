# Net SDK API Reference

Verified against `net-sdk` v0.8.0 (Rust) / `@ai2070/net-sdk` ≥ 0.8.0 peer (TS) / `net-sdk` Python (current `main`). Last full pass: 2026-04-25.

**Drift check before trusting these signatures:**
```bash
# Rust: confirms emit/emit_raw/emit_str/emit_batch/emit_raw_batch are still exported (expect 5)
grep -cE "^\s*pub fn emit" net/crates/net/sdk/src/net.rs
# Rust: confirms the SdkError variant set this skill enumerates (expect 11)
grep -cE "^\s*(Shutdown|Ingestion|Poll|Adapter|Serialization|Config|NoMesh|Backpressure|NotConnected|ChannelRejected|Traversal)" net/crates/net/sdk/src/error.rs
```

If any count drops, the SDK has churned underneath this doc — re-verify from source. If anything else looks wrong, **read the SDK source directly** — it is authoritative. The README is a good intro; the source is ground truth.

| Language | Path | Key files |
|---|---|---|
| Rust | `net/crates/net/sdk/` | `src/net.rs`, `examples/channels.rs`, `examples/stream.rs`, `examples/backpressure.rs` |
| TypeScript | `net/crates/net/sdk-ts/` | `src/node.ts`, `src/channel.ts`, `src/stream.ts` |
| Python | `net/crates/net/sdk-py/` | `src/net_sdk/node.py`, `src/net_sdk/channel.py`, `src/net_sdk/stream.py` |
| Go | `net/crates/net/bindings/go/` | the package's main file + `README.md` |
| C | `net/crates/net/include/` | `net.h` |

---

## Mental model recap (do not skip)

Two surfaces exist across the SDKs:

| Surface | What it is | Available in |
|---|---|---|
| **Named channels** (`node.channel("name")` → publish/subscribe) | Topic-based pub/sub. Channel name is embedded in payload as `_channel` and used as a subscribe filter. Subscriber roster held by publisher. | TypeScript, Python |
| **Raw typed firehose** (`node.emit(struct)` → `node.subscribe()`) | Single stream of typed events. Consumers receive everything; filter/discriminate on the receive side. | Rust, TypeScript, Python |
| **Raw poll** (`bus.IngestRaw` → `bus.Poll(cursor)`) | Push JSON in, poll JSON out with a cursor. No async, no channels. | Go, C |

If the user wants topic-based fan-out and they're in Rust, Go, or C: there is no built-in named-channel API. They filter on the consumer.

---

## TypeScript (`@ai2070/net-sdk`)

```bash
npm install @ai2070/net-sdk @ai2070/net
```

```typescript
import { NetNode } from '@ai2070/net-sdk';

interface TempReading { sensor_id: string; celsius: number }

const node = await NetNode.create({ shards: 4 });
// Other transports: pass `transport: { type: 'redis' | 'jetstream' | 'mesh', ... }`
// to create() — see `Transport` in src/types.ts for per-transport fields.

// Named-channel publisher
const temps = node.channel<TempReading>('sensors/temperature');
temps.publish({ sensor_id: 'A1', celsius: 22.5 });

// Named-channel subscriber (async iterator)
for await (const r of temps.subscribe()) {
  console.log(`${r.sensor_id}: ${r.celsius}°C`);
}

await node.shutdown();
```

**Key facts:**
- `NetNode.create(config)` is **async** — must `await`.
- All ingestion is sync, but the failure shape varies by method:
  - `emit(obj)` and `emitRaw(json)` are typed `Receipt | null`, but the underlying napi binding **throws** on ingestion failure (e.g. `fail_producer` mode, shutdown). The `| null` is vestigial — wrap in `try/catch`, don't null-check.
  - `emitBatch(objs)` / `emitRawBatch(jsons)` return `number` (count ingested). They throw on shutdown; on `drop_*` backpressure they return a short count. Compare against input length to detect partial drops.
  - `channel.publish(event)`, `channel.publishBatch(events)`, `emitBuffer(Buffer)`, `fire(json)`, `fireBatch(jsons)` go through the fire path and return `boolean` / `number`. A `false` / short return means the bus rejected the event — they don't throw.
  - With the default `drop_oldest` / `drop_newest` modes, the throwing methods don't throw on backpressure either: drops are silent and visible only via `node.stats().eventsDropped`.
- `subscribe()` returns an async iterable. Always consume with `for await...of`.
- `emitBuffer(Buffer)` is the zero-copy path. Use when the payload is already serialized.
- Validators are optional: `node.channel<T>('name', validator)` runs your function on each received event.
- `BackpressureError` / `NotConnectedError` and the `sendWithRetry` helper are **mesh-stream APIs** (peer-to-peer streams on `MeshNode`), not bus APIs. The bus emit path never throws them — see `runtime.md`.

## Python (`net-sdk`)

```bash
pip install net-sdk
```

```python
from dataclasses import dataclass
from net_sdk import NetNode

@dataclass
class TempReading:
    sensor_id: str
    celsius: float

with NetNode(shards=4) as node:
    # Other transports: pass redis_url=, jetstream_url=, or mesh_* kwargs
    temps = node.channel('sensors/temperature', TempReading)
    temps.publish(TempReading(sensor_id='A1', celsius=22.5))

    for r in temps.subscribe():           # sync generator
        print(f'{r.sensor_id}: {r.celsius}°C')

    # ...or, in an asyncio app:
    # async for r in temps.subscribe():
    #     ...
```

**Key facts:**
- `NetNode(...)` is **synchronous**. Use the context manager (`with`) for auto-shutdown.
- `subscribe()` returns an `EventStream` that supports **both** `for ... in` (sync) and `async for` (asyncio) (`net/crates/net/sdk-py/src/net_sdk/stream.py:38-58`). Pick one mode per stream instance — interleaving on the same instance is undefined. Note: the `async for` path still calls a blocking FFI poll per step; in an asyncio app where event-loop responsiveness matters, prefer the sync iterator inside `asyncio.to_thread(...)`. See `runtime.md` § Python.
- Models can be `@dataclass`, Pydantic models (anything with `model_dump()`), or plain classes (anything with `__dict__`).
- The native `net` module (PyO3 binding) is the escape hatch — `node.bus` exposes it. Use only for features not surfaced in `net_sdk`.

## Rust (`net-sdk`)

```toml
[dependencies]
net-sdk = "..."
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["rt", "macros"] }
futures = "0.3"
```

```rust
use net_sdk::{Backpressure, Net};
use serde::{Deserialize, Serialize};
use futures::StreamExt;

#[derive(Serialize, Deserialize)]
struct TempReading { sensor_id: String, celsius: f64 }

#[tokio::main(flavor = "current_thread")]
async fn main() -> net_sdk::error::Result<()> {
    let node = Net::builder()
        .shards(4)
        .backpressure(Backpressure::DropOldest)
        .memory()                          // or .redis(RedisAdapterConfig) / .jetstream(JetStreamAdapterConfig) / .mesh(NetAdapterConfig)
        .build()
        .await?;

    node.emit(&TempReading { sensor_id: "A1".into(), celsius: 22.5 })?;

    let mut stream = node.subscribe_typed::<TempReading>(Default::default());
    while let Some(r) = stream.next().await {
        let r = r?;
        println!("{}: {}°C", r.sensor_id, r.celsius);
    }

    node.shutdown().await?;
    Ok(())
}
```

**Key facts:**
- **No `node.channel()` API.** Rust has only the raw firehose. To split topics, use distinct types/enum variants in the payload and match on the consumer, or run separate `Net` instances per logical channel.
- Builder pattern selects transport: `.memory()`, `.redis(...)`, `.jetstream(...)`, `.mesh(...)`. Adapter methods take typed configs (`RedisAdapterConfig`, `JetStreamAdapterConfig`, `NetAdapterConfig`) — not raw URL strings. Each adapter is gated on a feature flag (`redis`, `jetstream`, `net`).
- **Feature umbrella** (from `net/crates/net/sdk/Cargo.toml`): default is `[]` (memory only). `local = ["net", "cortex", "compute", "groups"]` is the right shape for a single-node or LAN-only deployment; `full = ["local", "redis", "jetstream"]` is everything. NAT traversal lives behind its own `nat-traversal` feature (and `port-mapping` builds on top). When wiring `Cargo.toml`, prefer the umbrella feature over hand-listing.
- `emit(&T)` returns `Receipt { shard_id, timestamp }`. `emit_batch(&[T])` returns count (`usize`).
- **Fast-path emit variants** (`net/crates/net/sdk/src/net.rs:128-160`): `emit_raw(impl Into<Bytes>)` for already-serialized bytes (zero-copy), `emit_str(&str)` for JSON-as-string, `emit_raw_batch(Vec<Bytes>)` for batched bytes. All return the same `Receipt` (or `usize` for the batch form).
- `subscribe()` and `subscribe_typed::<T>()` return async streams. Poll with `.next().await`. Both clone the inner `Arc<EventBus>`; `shutdown` is reference-based and tolerates outstanding stream clones — see `runtime.md` § "Rust: subscribe streams and shutdown" for the full contract.
- One-shot pull (no streaming): `node.poll(PollRequest { limit, cursor, filter, ordering, shards }).await?` returns a `PollResponse { events, next_id, has_more }`. Use this when you want explicit cursor management instead of an `EventStream`.
- Lifecycle helpers: `node.flush().await?` waits for pending batches to drain into the adapter (call before `shutdown` if you can't tolerate the in-flight loss); `node.health().await -> bool` and `node.shards() -> u16` for observability; `node.bus() -> &EventBus` is the escape hatch to the underlying core API.
- `Backpressure::{DropOldest (default), DropNewest, FailProducer, Sample(u32)}` set at build time. `Sample(N)` keeps 1 in N events when overloaded.
- Convenience presets on the builder: `.high_throughput()`, `.low_latency()`, `.batch(BatchConfig)`, `.scaling(ScalingPolicy)`, `.adapter_timeout(Duration)`.
- Reference: `net/crates/net/sdk/examples/channels.rs` is the canonical typed-emit example.

## Go (`bindings/go/net`)

```go
import "github.com/ai-2070/net/net/crates/net/bindings/go/net"

bus, err := net.New(&net.Config{NumShards: 4})
if err != nil { log.Fatal(err) }
defer bus.Shutdown()

bus.IngestRaw(`{"sensor_id":"A1","celsius":22.5}`)

resp, _ := bus.Poll(100, "")
for _, raw := range resp.Events {
    // resp.Events is []json.RawMessage (=[][]byte). Convert to string
    // before printing or `fmt.Println` will render the raw bytes.
    fmt.Println(string(raw))
}
if resp.HasMore {
    resp, _ = bus.Poll(100, resp.NextID)   // pass cursor for next page
}
```

**Key facts:**
- **No async iterator and no named-channel API.** Write the polling loop yourself; manage the `NextID` cursor across calls.
- `PollResponse.Events` is `[]json.RawMessage` (= `[][]byte`) — pass each through `string(...)` to print or `json.Unmarshal` to parse.
- All methods are thread-safe.
- Filter by inspecting the JSON in your loop.
- Mesh transport requires `NewMeshNode` (separate constructor — check the README).

## C (`net.h`)

```c
#include "net.h"

net_handle_t node = net_init("{\"num_shards\": 4}");
const char* json = "{\"sensor_id\":\"A1\",\"celsius\":22.5}";
net_ingest_raw(node, json, strlen(json));

net_poll_result_t result;
if (net_poll_ex(node, 100, NULL, &result) == 0) {
    for (size_t i = 0; i < result.count; i++) {
        printf("%.*s\n", (int)result.events[i].raw_len, result.events[i].raw);
    }
    net_free_poll_result(&result);            // MUST free
}

net_shutdown(node);
```

**Key facts:**
- `net_poll_ex` allocates — always pair with `net_free_poll_result`.
- Pass `NULL` as cursor on first call. For subsequent calls, `strdup(result.next_id)` and `free()` it yourself.
- All functions are thread-safe.
- Return codes: 0 = success, negative = error (`NET_ERR_*`).
- Synchronous polling only. No async, no callbacks.

---

## Cross-SDK gotchas

These bite people regardless of language. Internalize them.

- **JSON everywhere.** The wire format is JSON bytes. There is no schema registry. The JSON either parses on the consumer or it doesn't.
- **Shutdown is required.** Don't rely on process exit. Call `shutdown()` / `Shutdown()` / `net_shutdown()`. The ring buffer needs a clean drain.
- **Subscribe is hot.** A subscriber sees events emitted *after* it subscribed, plus whatever's still in the ring buffer. No replay-from-zero. If the user wants replay, they need RedEX or an adapter — not the bus.
- **Backpressure is silent under `drop_*` modes; under `fail_producer` it surfaces per-language.** Always also watch `stats().events_dropped`. Per-SDK error shapes are detailed in each SDK's "Key facts" above and in `runtime.md` § Errors — don't duplicate the matrix here. The one-line summary: Rust returns `SdkError::Ingestion`, TS throws on the `emit*` path / returns `false`/short on the `publish*`/`fire*` path, Python raises from the binding, Go/C return error codes.
- **`_channel` is reserved** in TS/Python channel payloads. Don't put your own field there.
- **Transport is set at construction.** A node can have only one transport. To bridge transports, run two nodes in the same process and forward between them.
- **`shards` is a parallelism knob, not a partitioning scheme.** It does not give you Kafka-style ordered partitions. It just parallelizes ingestion. Default is fine for most workloads.

---

## When the API surface here isn't enough

Out-of-scope features (read these directly from source):

- **Mesh transport configuration** (peer discovery, NAT traversal, port mapping, identity keys) — each SDK exposes mesh-specific kwargs. See SDK README's "Mesh" section.
- **Subnets and capability tags** — set on node construction; affect channel visibility. See `net/README.md` § Subnets and Capabilities.
- **Capability discovery** — `mesh.find_nodes(filter)` / `find_nodes_scoped(filter, scope)` / `find_best_node(req)` for picking the right peer by hardware/model/tag. Reserved `scope:tenant:*` / `scope:region:*` / `scope:subnet-local` tags narrow discovery to per-tenant or per-region pools without channel-level subnet routing.
- **Permission tokens for channel auth** — see `net/README.md` § Security surface.
- **RedEX / CortEX / NetDB** — separate APIs for persistence and queryable state. See `net/README.md` § RedEX and CortEX + NetDB.
- **Mikoshi (live daemon migration)** — separate API for stateful event processors. See `net/README.md` § Daemons and Mikoshi.
