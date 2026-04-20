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

## Mesh Streams (multi-peer + back-pressure)

For direct peer-to-peer messaging — open a stream to a specific peer
and react to back-pressure with first-class error classes:

```typescript
import { MeshNode, BackpressureError, NotConnectedError } from '@ai2070/net-sdk';

const node = await MeshNode.create({
  bindAddr: '127.0.0.1:9000',
  psk: '0'.repeat(64),
});
// ... handshake (node.connect(...) / node.accept(...)) ...

const stream = node.openStream(peerNodeId, {
  streamId: 0x42n,
  reliability: 'reliable',
  windowBytes: 256,   // max in-flight packets before BackpressureError
});

// Three canonical daemon patterns:

// 1. Drop on pressure.
try {
  await node.sendOnStream(stream, [Buffer.from('{}')]);
} catch (e) {
  if (e instanceof BackpressureError) {
    metrics.inc('stream.backpressure_drops');
  } else if (e instanceof NotConnectedError) {
    // peer gone or stream closed — re-open if needed
  } else {
    throw e;
  }
}

// 2. Retry with exponential backoff (5 ms → 200 ms, up to maxRetries).
await node.sendWithRetry(stream, [Buffer.from('{}')], 8);

// 3. Block until the network lets up (bounded retry, ~13 min worst case).
await node.sendBlocking(stream, [Buffer.from('{}')]);

// Live stats — tx/rx seq, in-flight, window, backpressure count (BigInts).
const stats = node.streamStats(peerNodeId, 0x42n);
```

`BackpressureError` and `NotConnectedError` both extend `Error`, so
`instanceof` and `try/catch` work as expected. The transport never
retries or buffers on its own behalf — the helper methods are
opt-in policies, not defaults. See `../docs/TRANSPORT.md` for the full
contract.

## Security (identity, tokens, capabilities, subnets)

Identity, capabilities, and subnets ride the underlying NAPI bindings
as a single security unit — the mesh's subprotocol dispatch threads
identity + capabilities + subnets + channel auth together at runtime,
and the TS SDK surfaces all of it through one type hierarchy.

```typescript
import { randomBytes } from 'node:crypto';
import { Identity, MeshNode } from '@ai2070/net-sdk';

// Load once from caller-owned storage (vault / KMS / env secret).
// The persisted form IS the 32-byte seed; treat as secret material.
const seed = randomBytes(32);
const identity = Identity.fromSeed(seed);

// Stable entity_id / node_id across restarts — derived from the seed.
const mesh = await MeshNode.create({
  bindAddr: '127.0.0.1:9001',
  psk: '42'.repeat(32),
  identitySeed: seed,          // mesh and identity share the keypair
});

// mesh.entityId() === identity.entityId (both 32-byte Buffers).

// Issue a scoped subscribe grant for another entity.
const grantee = Identity.generate();
const token = identity.issueToken({
  subject: grantee.entityId,
  scope: ['subscribe'],
  channel: 'sensors/temp',
  ttlSeconds: 300,
  delegationDepth: 0,          // 0 forbids re-delegation
});

// `token.bytes` is the transport-ready 159-byte blob.
// Ship it to the grantee; they hand it back on subscribe.
```

Errors surface as `IdentityError` (malformed inputs — bad seed
length, unknown scope, invalid channel name) and `TokenError` whose
`kind` discriminator is one of `invalid_format` | `invalid_signature`
| `expired` | `not_yet_valid` | `delegation_exhausted` |
`delegation_not_allowed` | `not_authorized`. Both extend `Error`,
so `try/catch` + `instanceof` work as expected.

### Capability announcements

`mesh.announceCapabilities(caps)` broadcasts a `CapabilitySet` to
every directly-connected peer and self-indexes locally.
`mesh.findPeers(filter)` queries the local index — results include
this node's own id when self matches.

```typescript
import { MeshNode } from '@ai2070/net-sdk';

const mesh = await MeshNode.create({
  bindAddr: '127.0.0.1:9002',
  psk: '42'.repeat(32),
});

await mesh.announceCapabilities({
  hardware: {
    cpuCores: 16,
    memoryMb: 65_536,
    gpu: { vendor: 'nvidia', model: 'h100', vramMb: 81_920 },
  },
  models: [
    { modelId: 'llama-3.1-70b', family: 'llama', contextLength: 128_000 },
  ],
  tags: ['gpu', 'prod'],
});

const gpuPeers = mesh.findPeers({
  requireGpu: true,
  gpuVendor: 'nvidia',
  minVramMb: 40_000,
});
// gpuPeers includes mesh.nodeId() on self-match.
```

Propagation is multi-hop, bounded by `MAX_CAPABILITY_HOPS = 16`.
Forwarders re-broadcast every received announcement to their other
peers; dedup on `(origin, version)` drops duplicates at convergence
points, and `hop_count` sits outside the signed envelope so the
origin's signature verifies at every hop.
`capabilityGcIntervalMs` + TTL-driven eviction are configurable on
`MeshNode.create`. See
[`docs/MULTIHOP_CAPABILITY_PLAN.md`](../docs/MULTIHOP_CAPABILITY_PLAN.md).

### Subnets (visibility partitioning)

`subnet` pins a node to a specific 4-level `SubnetId`; `subnetPolicy`
derives each *peer's* subnet from their inbound capability tags so
every node in the mesh agrees on the geometry without a central
directory.

```typescript
import { MeshNode } from '@ai2070/net-sdk';

const policy = {
  rules: [
    { tagPrefix: 'region:', level: 0, values: { us: 3, eu: 4 } },
    { tagPrefix: 'fleet:',  level: 1, values: { blue: 7, green: 8 } },
  ],
};

const mesh = await MeshNode.create({
  bindAddr: '127.0.0.1:9003',
  psk: '42'.repeat(32),
  subnet: { levels: [3, 7] },    // us/blue
  subnetPolicy: policy,
});

// Announce tags matching the policy so peers derive the same
// SubnetId [3, 7] when they apply their own policy to our caps.
await mesh.announceCapabilities({ tags: ['region:us', 'fleet:blue'] });
```

Channel `visibility` gates publish fan-out and subscribe
authorization against the derived geometry. Cross-subnet subscribes
to a `SubnetLocal` channel reject with `Unauthorized`.

### Channel authentication

`ChannelConfig` carries three auth knobs, enforced end-to-end at
both the subscribe gate and the publish path:

- `publishCaps: CapabilityFilter` — publisher must satisfy before
  fan-out. Failing publishes raise an error; no peers are attempted.
- `subscribeCaps: CapabilityFilter` — subscribers must satisfy
  before being added to the roster. Failures surface as
  `ChannelAuthError`.
- `requireToken: true` — subscribers must present a valid `Token`
  whose subject matches their `entityId`. The publisher verifies
  the ed25519 signature, installs the token in its local cache,
  then runs `can_subscribe`.

```typescript
import { Identity, MeshNode } from '@ai2070/net-sdk';

const pubIdentity = Identity.generate();
const subIdentity = Identity.generate();

const publisher = await MeshNode.create({
  bindAddr: '127.0.0.1:9004',
  psk: '42'.repeat(32),
  identitySeed: pubIdentity.toBytes(),
});

publisher.registerChannel({
  name: 'events/inference',
  subscribeCaps: { requireTags: ['gpu'] },
  requireToken: true,
});

// Issue a SUBSCRIBE-scope token for the subscriber.
const token = pubIdentity.issueToken({
  subject: subIdentity.entityId,
  scope: ['subscribe'],
  channel: 'events/inference',
  ttlSeconds: 300,
});

// Subscriber side (different MeshNode) — attach the token.
await subscriber.subscribeChannel(
  publisher.nodeId(),
  'events/inference',
  { token },
);
```

Denied subscribes surface as `ChannelAuthError` (a subclass of
`ChannelError`); malformed token bytes raise `TokenError` before
any network I/O. Successful subscribes populate an `AuthGuard`
bloom filter on the publisher so every subsequent publish admits
the subscriber in constant time (~20 ns per check,
single-threaded). Expired tokens evict within the publisher's
`token_sweep_interval` (default 30 s); repeated subscribe
failures from the same peer throttle via `RateLimited` acks so
bad-token storms never tie up ed25519 verification. Cross-SDK
behaviour is fixed by the Rust integration suite — see
[`SDK_SECURITY_SURFACE_PLAN.md`](../docs/SDK_SECURITY_SURFACE_PLAN.md)
and
[`CHANNEL_AUTH_GUARD_PLAN.md`](../docs/CHANNEL_AUTH_GUARD_PLAN.md)
for the full contract.

## Channels (distributed pub/sub)

Named pub/sub across the encrypted mesh. The publisher registers a
channel config; subscribers ask to join via `subscribeChannel` (the
subscribe goes through a dedicated subprotocol with an Ack round-trip);
`publish` fans one payload out to every current subscriber.

```typescript
import { MeshNode, ChannelAuthError } from '@ai2070/net-sdk';

const psk = '0'.repeat(64);

// Publisher side.
const b = await MeshNode.create({ bindAddr: '127.0.0.1:9001', psk });
b.registerChannel({
  name: 'sensors/temp',
  visibility: 'global',           // or 'subnet-local' / 'parent-visible' / 'exported'
  reliable: true,
  priority: 2,
  maxRatePps: 1000,
});

// Subscriber side + full handshake.
const a = await MeshNode.create({ bindAddr: '127.0.0.1:9002', psk });
const aNodeId = a.nodeId();
const bNodeId = b.nodeId();
// connect/accept must race: the initiator blocks on a handshake reply
// that only shows up once the responder is in accept(). Then both
// sides must start() their receive loops before app traffic flows.
await Promise.all([
  b.accept(aNodeId),
  a.connect('127.0.0.1:9001', b.publicKey(), bNodeId),
]);
await a.start();
await b.start();
await a.subscribeChannel(bNodeId, 'sensors/temp');

// Fan out.
const report = await b.publish(
  'sensors/temp',
  Buffer.from(JSON.stringify({ celsius: 22.5 })),
  { reliability: 'reliable', onFailure: 'best_effort', maxInflight: 32 },
);
console.log(`${report.delivered}/${report.attempted} subscribers received`);

// Rejections surface with typed errors:
try {
  await a.subscribeChannel(bNodeId, 'restricted');
} catch (e) {
  if (e instanceof ChannelAuthError) { /* ACL rejected */ }
}
```

**Channel names always cross the boundary as strings.** The u16 hash
is a transport-layer index only; ACL lookups key on the canonical
name to avoid bypass via hash collision (see `../docs/CHANNELS.md`).

Subscribers today receive payloads through the existing event-bus
`poll()` surface — a dedicated per-channel `AsyncIterable` receive
method is a follow-up.

## CortEX & NetDb (event-sourced state)

Typed, event-sourced state on top of RedEX — tasks and memories with
filterable queries and reactive `AsyncIterable` watches. Includes the
`snapshotAndWatch` primitive whose race fix landed on v2, so you can
safely "paint what's there now, then react to changes" without losing
updates that race during construction.

```typescript
import { NetDb, TaskStatus, CortexError } from '@ai2070/net-sdk';

const db = await NetDb.open({
  originHash: 0xABCDEF01,
  withTasks: true,
  withMemories: true,
  // persistentDir + persistent: true for disk-backed files
});

// CRUD through the domain API — no EventMeta plumbing.
try {
  const seq = db.tasks!.create(1n, 'write docs', 100n);
  await db.tasks!.waitForSeq(seq);  // wait for the fold to apply
} catch (e) {
  if (e instanceof CortexError) { /* handle adapter error */ }
  else { throw e; }
}

// Snapshot + watch: one atomic call, no race.
const { snapshot, updates } = await db.tasks!.snapshotAndWatch({
  status: TaskStatus.Pending,
});
render(snapshot);
for await (const next of updates) {
  render(next);
  if (shouldStop) break;   // automatically closes the native iterator
}

db.close();
```

### Plain watches

`watch()` returns the same `AsyncIterable<T[]>` shape without a
snapshot. Prefer `snapshotAndWatch` when the caller needs the initial
result — calling `listTasks()` + `watch()` separately races, and a
mutation landing between them can be silently lost.

```typescript
for await (const batch of await db.tasks!.watch({ titleContains: 'ship' })) {
  // each batch is the current filter result after a deduplicated fold tick
}
```

### Standalone adapters

If you only need one model, skip the `NetDb` facade and open the
adapter directly against a `Redex`:

```typescript
import { Redex, TasksAdapter } from '@ai2070/net-sdk';

const redex = new Redex({ persistentDir: '/var/lib/net/redex' });
const tasks = await TasksAdapter.open(redex, 0xABCDEF01, { persistent: true });
```

### Raw RedEX file (no CortEX fold)

For domain-agnostic persistent logs — your own event schema, no fold,
no typed adapter — open a `RedexFile` directly from a `Redex`. The
tail iterator is the same `AsyncIterable` shape as the CortEX
watches, so `for await` + `break` cleans up native resources.

```typescript
import { Redex, RedexError } from '@ai2070/net-sdk';

const redex = new Redex({ persistentDir: '/var/lib/net/events' });
const file = redex.openFile('analytics/clicks', {
  persistent: true,
  fsyncIntervalMs: 100,           // or fsyncEveryN: 1000n
  retentionMaxEvents: 1_000_000n,
});

// Append (or batch-append).
const seq = file.append(Buffer.from(JSON.stringify({ url: '/home' })));
file.appendBatch(payloadBuffers);

// Tail — backfills the retained range, then streams live appends.
const stream = await file.tail(0n);
try {
  for await (const event of stream) {
    const parsed = JSON.parse(event.payload.toString());
    console.log(event.seq, parsed);
    if (shouldStop) break;   // automatically closes the native iterator
  }
} catch (e) {
  if (e instanceof RedexError) { /* ... */ }
  throw e;
} finally {
  // Ensure the file is closed even if tailing / parsing throws.
  file.close();
}
```

### Error classes

CortEX-boundary errors are typed and catchable via `instanceof`:

- `CortexError` — adapter errors (fold halted, RedEX I/O, decode failures).
- `NetDbError` — snapshot/restore bundle errors, missing-model lookups.
- `RedexError` — raw file errors (invalid channel name, bad config,
  append / tail / sync / close failures).

All three are re-exported from `@ai2070/net-sdk`; you don't need a
separate import path.

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

### CortEX surface

| Entry point | Description |
|---|---|
| `new Redex({ persistentDir? })` | Local event-log manager |
| `NetDb.open({ originHash, withTasks?, withMemories?, ... })` | Unified handle |
| `NetDb.openFromSnapshot(config, bundle)` | Restore from `db.snapshot()` bundle |
| `db.tasks` / `db.memories` | Typed adapter handles |
| `TasksAdapter.open(redex, origin, opts?)` | Standalone tasks adapter |
| `MemoriesAdapter.open(redex, origin, opts?)` | Standalone memories adapter |
| `adapter.create/rename/complete/delete/...` | Domain CRUD |
| `adapter.listTasks(filter?)` / `listMemories` | Sync snapshot query |
| `adapter.watch(filter?)` | `Promise<AsyncIterable<T[]>>` over deduplicated fold results |
| `adapter.snapshotAndWatch(filter?)` | `Promise<SnapshotAndWatch<T>>` — atomic paint+react |
| `adapter.snapshot()` / `openFromSnapshot` | Model-level persistence |
| `db.snapshot()` / `NetDb.openFromSnapshot` | Bundled multi-model persistence |
| `redex.openFile(name, config?)` | Raw RedEX file — append-only log |
| `file.append(buffer)` / `appendBatch(buffers)` | Append one / many payloads |
| `file.readRange(start, end)` | Range read over retained entries |
| `file.tail(fromSeq?)` | `AsyncIterable<RedexEvent>` |
| `file.sync()` / `file.close()` | Explicit fsync / close |

## License

Apache-2.0
