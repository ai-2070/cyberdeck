# TypeScript SDK Plan

Wrapper package (`@ai2070/net-sdk`) over the NAPI bindings (`@ai2070/net`). The NAPI layer stays thin — raw FFI to the Rust core. The SDK adds the developer experience TypeScript developers expect.

## Current State

The NAPI bindings expose:
- `Net.create(options)` — factory
- `push(buf)` / `pushBatch(bufs)` — Buffer ingestion (fastest)
- `ingestRawSync(json)` / `ingestRawBatchSync(jsons)` — string ingestion
- `ingestFire(json)` / `ingestBatchFire(jsons)` — fire-and-forget
- `poll(options)` — cursor-based consumption
- `flush()` / `shutdown()` / `stats()` / `numShards()`

No streaming, no typed events, no middleware, no mesh access.

## Phase 1: Core — `NetNode` + Builder

```typescript
import { NetNode } from '@ai2070/net-sdk';

const node = await NetNode.create({
  shards: 4,
  bufferCapacity: 1 << 20,
  backpressure: 'drop_oldest',
  // Transport (pick one):
  transport: { type: 'memory' },
  transport: { type: 'redis', url: 'redis://localhost:6379' },
  transport: { type: 'mesh', bind: '0.0.0.0:9000', peer: 'peer:9001', psk, peerPublicKey },
});
```

### Ingestion

```typescript
// Typed (serializes to JSON via JSON.stringify)
node.emit({ token: 'hello', index: 0 });

// Raw string (fast path)
node.emitRaw('{"token": "hello"}');

// Raw Buffer (fastest path, zero-copy to Rust)
node.emitBuffer(Buffer.from('{"token": "hello"}'));

// Batch
node.emitBatch([event1, event2, event3]);
node.emitRawBatch(['{"a":1}', '{"a":2}']);

// Fire-and-forget (no return value, maximum speed)
node.fire('{"token": "hello"}');
node.fireBatch(['{"a":1}', '{"a":2}']);
```

### Consumption

```typescript
// One-shot poll
const { events, nextId, hasMore } = await node.poll({ limit: 100 });

// Lifecycle
node.stats();
await node.flush();
await node.shutdown();
```

## Phase 2: Streaming Consumption

The main upgrade over raw NAPI bindings. Three patterns native to TypeScript/Node.js:

### AsyncIterator

```typescript
for await (const event of node.subscribe({ limit: 100 })) {
  console.log(event.raw);
}

// Typed
for await (const token of node.subscribe<TokenEvent>({ limit: 100 })) {
  console.log(token.token, token.index);
}
```

### ReadableStream (Web Streams API)

```typescript
const stream = node.stream({ limit: 100 });

// Pipe to a WritableStream
stream.pipeTo(writableStream);

// Or use a reader
const reader = stream.getReader();
const { value, done } = await reader.read();
```

### EventEmitter

```typescript
const sub = node.on('event', (event: StoredEvent) => {
  console.log(event.raw);
});

// With channel filter
node.on('event:sensors/temperature', (event) => { ... });

// Unsubscribe
sub.off();
```

All three internally use adaptive polling — tight loop when events flow, exponential backoff when idle.

## Phase 3: Typed Channels

```typescript
import { TypedChannel } from '@ai2070/net-sdk';

interface TemperatureReading {
  sensor_id: string;
  celsius: number;
  timestamp: number;
}

const temps = node.channel<TemperatureReading>('sensors/temperature');

// Publish (validates at compile time)
temps.publish({ sensor_id: 'A1', celsius: 22.5, timestamp: Date.now() });

// Subscribe
for await (const reading of temps.subscribe()) {
  console.log(`${reading.sensor_id}: ${reading.celsius}°C`);
}
```

### Zod Integration (Optional)

```typescript
import { z } from 'zod';

const TemperatureSchema = z.object({
  sensor_id: z.string(),
  celsius: z.number(),
  timestamp: z.number(),
});

const temps = node.channel('sensors/temperature', TemperatureSchema);
// Runtime validation on both publish and subscribe
```

## Phase 4: Ready-Made Components

### Middleware

```typescript
import { pipeline, rateLimit, filter, transform, logger } from '@ai2070/net-sdk/middleware';

const node = await NetNode.create({
  shards: 4,
  middleware: pipeline(
    logger(),
    rateLimit({ perSecond: 10_000 }),
    filter({ channel: 'sensors/*' }),
    transform((event) => enrich(event)),
  ),
});
```

### Router

```typescript
import { Router } from '@ai2070/net-sdk/router';

const router = new Router(node);
router.route('sensors/*', 'analytics/ingest');
router.route('alerts/*', 'ops/alerts');
router.route('sensors/temperature', async (event) => {
  // Custom handler
  const reading = JSON.parse(event.raw);
  if (reading.celsius > 100) {
    node.emit({ channel: 'alerts/temperature', ...reading });
  }
});
```

### Aggregator

```typescript
import { Aggregator } from '@ai2070/net-sdk/aggregator';

const avg = new Aggregator(node, 'sensors/temperature', {
  window: 5000, // ms
  reduce: (events) => ({
    avg_celsius: events.reduce((s, e) => s + e.celsius, 0) / events.length,
  }),
  emitTo: 'analytics/temperature_avg',
});
```

## Phase 5: Discovery & Mesh

```typescript
// Find nodes by capability
const gpuNodes = await node.discover({ capability: 'gpu', minVram: 16 });
const nearby = await node.discover({ withinHops: 3 });
const models = await node.discover({ servesModel: 'gemma-21b' });

// Mesh info (only with mesh transport)
const mesh = node.mesh();
mesh.capabilities();
mesh.graph();
mesh.routes();
```

## Phase 6: Inference

First-class support for the primary use case:

```typescript
import { InferenceNode } from '@ai2070/net-sdk/inference';

// Serve inference on this node
const server = new InferenceNode(node, {
  model: 'gemma-21b',
  vramGb: 24,
  maxConcurrent: 8,
  channel: 'inference/requests',
});

// Request inference from the mesh
const response = await node.infer('inference/requests', {
  prompt: 'What is the meaning of life?',
  maxTokens: 256,
});

// Streaming inference
for await (const token of node.inferStream('inference/requests', { prompt })) {
  process.stdout.write(token);
}
```

## Phase 7: Testing Utilities

```typescript
import { TestMesh } from '@ai2070/net-sdk/testing';

const mesh = await TestMesh.create(3); // 3 interconnected nodes

mesh.nodes[0].emit({ hello: 'world' });
const event = await mesh.nodes[2].pollOne();

mesh.partition(0, 2);  // simulate network partition
mesh.heal();           // restore connectivity

// Assertions
expect(mesh.nodes[1].stats().eventsIngested).toBe(1);

afterAll(() => mesh.teardown());
```

## Phase 8: Observability

```typescript
node.on('metric', (m) => {
  switch (m.type) {
    case 'event_ingested': console.log(m.latency, m.shard); break;
    case 'peer_discovered': console.log(m.nodeId, m.hops); break;
    case 'daemon_migrated': console.log(m.from, m.to, m.duration); break;
    case 'route_changed': console.log(m.stream, m.oldNode, m.newNode); break;
  }
});

// OpenTelemetry integration
import { otelExporter } from '@ai2070/net-sdk/otel';
node.use(otelExporter({ endpoint: 'http://localhost:4317' }));
```

## Phase 9: Scheduled Events

```typescript
import { cron } from '@ai2070/net-sdk/schedule';

node.schedule(cron('*/30 * * * * *'), {
  channel: 'health/heartbeat',
  payload: () => node.stats(),
});

// Simple interval
node.every(5000, {
  channel: 'sensors/poll',
  payload: () => readSensor(),
});
```

## Structure

```
packages/net-sdk/
  package.json         # @ai2070/net-sdk
  tsconfig.json
  src/
    index.ts           # re-exports
    node.ts            # NetNode — the main handle
    config.ts          # builder types
    stream.ts          # AsyncIterator, ReadableStream, EventEmitter wrappers
    channel.ts         # TypedChannel<T>
    middleware.ts       # pipeline, rateLimit, filter, transform, logger
    router.ts          # Router
    aggregator.ts      # Aggregator
    discovery.ts       # discover()
    mesh.ts            # MeshHandle
    inference.ts       # InferenceNode, infer(), inferStream()
    testing.ts         # TestMesh
    otel.ts            # OpenTelemetry integration
    schedule.ts        # cron, every
    types.ts           # shared types
  test/
    node.test.ts
    stream.test.ts
    channel.test.ts
    middleware.test.ts
    inference.test.ts
    testing.test.ts
```

## Dependencies

`@ai2070/net` (NAPI bindings), `zod` (optional peer dep for runtime validation)

No other runtime dependencies. The SDK is a pure TypeScript layer over the NAPI bindings. Everything heavy happens in Rust.
