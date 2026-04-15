# Net TypeScript SDK

Ergonomic TypeScript SDK for the Net mesh network.

Wraps the `@ai2070/net` NAPI bindings with streaming, typed channels, and a developer-friendly API.

## Install

```bash
npm install @ai2070/net-sdk @ai2070/net
```

## Quick Start

```typescript
import { NetNode } from '@ai2070/net-sdk';

const node = await NetNode.create({ shards: 4 });

// Emit events
node.emit({ token: 'hello', index: 0 });
node.emitRaw('{"token": "world"}');
node.emitBuffer(Buffer.from('{"token": "foo"}'));

// Batch
node.emitBatch([{ a: 1 }, { a: 2 }, { a: 3 }]);

await node.flush();

// Poll
const response = await node.poll({ limit: 100 });
for (const event of response.events) {
  console.log(event.raw);
}

// Stream (async iterator)
for await (const event of node.subscribe({ limit: 100 })) {
  console.log(event.raw);
}

await node.shutdown();
```

## Typed Streams

```typescript
interface TokenEvent {
  token: string;
  index: number;
}

for await (const token of node.subscribeTyped<TokenEvent>({ limit: 100 })) {
  console.log(`${token.index}: ${token.token}`);
}
```

## Typed Channels

```typescript
interface TemperatureReading {
  sensor_id: string;
  celsius: number;
  timestamp: number;
}

const temps = node.channel<TemperatureReading>('sensors/temperature');

// Publish
temps.publish({ sensor_id: 'A1', celsius: 22.5, timestamp: Date.now() });

// Subscribe
for await (const reading of temps.subscribe()) {
  console.log(`${reading.sensor_id}: ${reading.celsius}°C`);
}
```

## Ingestion Methods

| Method | Input | Speed | Returns |
|--------|-------|-------|---------|
| `emit(obj)` | Object | Fast | `Receipt` |
| `emitRaw(json)` | String | Fast | `Receipt` |
| `emitBuffer(buf)` | Buffer | Fastest | `boolean` |
| `emitBatch(objs)` | Object[] | Bulk | `number` |
| `emitRawBatch(jsons)` | String[] | Bulk | `number` |
| `fire(json)` | String | Fire-and-forget | `boolean` |
| `fireBatch(jsons)` | String[] | Fire-and-forget | `number` |

## Transports

```typescript
// In-memory (default)
await NetNode.create({ shards: 4 });

// Redis
await NetNode.create({ transport: { type: 'redis', url: 'redis://localhost:6379' } });

// JetStream
await NetNode.create({ transport: { type: 'jetstream', url: 'nats://localhost:4222' } });

// Encrypted mesh
await NetNode.create({
  transport: {
    type: 'mesh',
    bind: '0.0.0.0:9000',
    peer: '192.168.1.10:9001',
    psk: '...',
    peerPublicKey: '...',
  },
});
```

## API

| Method | Description |
|--------|-------------|
| `NetNode.create(config)` | Create a new node |
| `emit(obj)` | Emit a typed event |
| `emitRaw(json)` | Emit a JSON string |
| `emitBuffer(buf)` | Emit a Buffer (fastest) |
| `emitBatch(objs)` | Batch emit |
| `emitRawBatch(jsons)` | Batch emit strings |
| `fire(json)` | Fire-and-forget |
| `fireBatch(jsons)` | Fire-and-forget batch |
| `poll(request)` | One-shot poll |
| `pollOne()` | Poll a single event |
| `subscribe(opts)` | Async iterable stream |
| `subscribeTyped<T>(opts)` | Typed async iterable |
| `channel<T>(name)` | Create a typed channel |
| `stats()` | Ingestion statistics |
| `shards()` | Number of active shards |
| `flush()` | Flush pending batches |
| `shutdown()` | Graceful shutdown |
| `napi` | Access underlying NAPI binding |

## License

Apache-2.0
