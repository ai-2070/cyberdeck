# Net SDK API Reference

Verified against the SDK source as of skill creation. If anything looks wrong, **read the SDK source directly** — it is authoritative. The README is a good intro; the source is ground truth.

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
// Other transports: pass redisUrl, jetstreamUrl, or mesh* options to create()

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
- `emit`, `publish`, `emitRaw`, `emitBatch` are sync. Return value may be `null` under backpressure.
- `subscribe()` returns an async iterable. Always consume with `for await...of`.
- `emitBuffer(Buffer)` is the zero-copy path. Use when the payload is already serialized.
- Validators are optional: `node.channel<T>('name', validator)` runs your function on each received event.

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

    for r in temps.subscribe():           # generator, NOT async
        print(f'{r.sensor_id}: {r.celsius}°C')
```

**Key facts:**
- `NetNode(...)` is **synchronous**. Use the context manager (`with`) for auto-shutdown.
- `subscribe()` returns a regular generator. Use `for ... in`, never `async for`.
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
        .memory()                          // or .redis(url) / .jetstream(url) / .mesh(config)
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
- Builder pattern selects transport: `.memory()`, `.redis(...)`, `.jetstream(...)`, `.mesh(...)`.
- `emit(&T)` returns `Receipt { shard_id, timestamp }`. `emit_batch(&[T])` returns count.
- `subscribe()` and `subscribe_typed::<T>()` return async streams. Poll with `.next().await`.
- `Backpressure::{DropOldest, DropNewest, FailProducer}` set at build time.
- Reference: `net/crates/net/sdk/examples/channels.rs` is the canonical typed-emit example.

## Go (`bindings/go/net`)

```go
import "github.com/ai-2070/cyberdeck/net/crates/net/bindings/go/net"

bus, err := net.New(&net.Config{NumShards: 4})
if err != nil { log.Fatal(err) }
defer bus.Shutdown()

bus.IngestRaw(`{"sensor_id":"A1","celsius":22.5}`)

resp, _ := bus.Poll(100, "")
for _, raw := range resp.Events {
    fmt.Println(raw)
}
if resp.HasMore {
    resp, _ = bus.Poll(100, resp.NextID)   // pass cursor for next page
}
```

**Key facts:**
- **No async iterator and no named-channel API.** Write the polling loop yourself; manage the `NextID` cursor across calls.
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
- **Backpressure is silent.** Producers may quietly fail. Check return values: TS returns `null`; batch APIs return ingested counts; Rust returns `Result`.
- **`_channel` is reserved** in TS/Python channel payloads. Don't put your own field there.
- **Transport is set at construction.** A node can have only one transport. To bridge transports, run two nodes in the same process and forward between them.
- **`shards` is a parallelism knob, not a partitioning scheme.** It does not give you Kafka-style ordered partitions. It just parallelizes ingestion. Default is fine for most workloads.

---

## When the API surface here isn't enough

Out-of-scope features (read these directly from source):

- **Mesh transport configuration** (peer discovery, NAT traversal, port mapping, identity keys) — each SDK exposes mesh-specific kwargs. See SDK README's "Mesh" section.
- **Subnets and capability tags** — set on node construction; affect channel visibility. See `net/README.md` § Subnets and Capabilities.
- **Permission tokens for channel auth** — see `net/README.md` § Security surface.
- **RedEX / CortEX / NetDB** — separate APIs for persistence and queryable state. See `net/README.md` § RedEX and CortEX + NetDB.
- **Mikoshi (live daemon migration)** — separate API for stateful event processors. See `net/README.md` § Daemons and Mikoshi.
