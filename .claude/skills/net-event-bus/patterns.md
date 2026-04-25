# Net Event Bus — Task → Recipe Map

Match the user's described task to a recipe. Each recipe assumes you've already read `concepts.md` and have `apis.md` open for the actual code templates.

---

## "I want a producer that emits events to a topic"

**Recipe:** named-channel publisher.

- TS / Python: `node.channel("name").publish(event)`. Channel name is the topic.
- Rust / Go / C: emit typed events on the firehose; consumers filter on payload type or content. There is no built-in topic API.

Default transport: `memory` for single-process, `mesh` for multi-host, `redis`/`jetstream` if the user already runs those.

## "I want a consumer that subscribes to a topic"

**Recipe:** named-channel subscriber.

- TS: `for await (const ev of node.channel<T>('name').subscribe()) { ... }`.
- Python: `for ev in node.channel('name', T).subscribe(): ...` (sync generator, not async).
- Rust: `node.subscribe_typed::<T>(opts)` — you'll see all events of type T, not just one channel.
- Go / C: `bus.Poll(limit, cursor)` in a loop, filter by inspecting JSON.

Subscriptions are **hot** — set up the subscriber before the publisher emits, or accept that you'll miss events emitted before you joined.

## "I want a relay node that forwards events between subnets / hosts"

**Recipe:** a `NetNode` with mesh transport and no application logic.

A node that joins the mesh automatically participates in routing. If it sits between two other nodes that can't reach each other directly, it forwards encrypted bytes. **You don't write relay code** — the mesh transport does it. Just spin up a `NetNode` with mesh transport on the relay host and let it run.

Configure mesh peers (at least one bootstrap peer) on construction. See SDK README's "Mesh" section.

## "I want events to survive a process restart" (durability / replay)

**Net does not buffer events for you when no one is consuming.** The bus is transient. Pick one:

- **Redis transport** — events go to Redis Streams; durability and retention are Redis's. Use if you already run Redis.
- **JetStream transport** — same idea via NATS JetStream.
- **RedEX** (in addition to memory transport) — local append-only log per channel, per node, per file. Each node decides retention. No cluster. See `net/README.md` § RedEX. This is the "lightweight, no external dependency" option.

If the user says "I need exactly-once delivery" — Net does not provide this at the bus level. They need application-level idempotency (event hash deduplication) or a persistence adapter that gives them offsets.

## "I want fan-out to many subscribers across machines"

**Recipe:** mesh transport, named channel (TS/Python) or typed firehose (Rust/Go/C).

Fan-out is **N per-peer unicasts** held by the publisher's roster. The cost is linear in subscriber count, paid by the publisher. There is no multicast packet, no group key, no broker amplification.

If the publisher has 1000 subscribers, each emit becomes 1000 unicasts. Each unicast is encrypted with that peer's session key. For high-fan-out workloads, consider running multiple publisher nodes that each hold a partial roster.

## "I want to bridge Net to Kafka / NATS / something else"

**Recipe:** two nodes in one process, one per transport, with a forwarding loop.

```
Node A (mesh transport) → subscribes → drains events → publishes → Node B (kafka producer)
```

Or use the Redis/JetStream transports directly so anything that consumes those streams sees Net events.

There is no built-in bridge product. It's an application — three to ten lines of code per direction.

## "I want a publisher to also see its own events"

By default, **the publisher does not receive its own emits via subscribe.** If you need that, the publisher subscribes to the same channel explicitly, and its own roster will deliver to it.

If the user is confused that emit + subscribe in the same process produces nothing — this is why. Subscribe before emitting and check whether the SDK adds self to the roster automatically (varies by transport).

## "I need ordered delivery"

Per-stream ordering is opt-in. By default, the fast path is unordered.

- Mesh transport: use `Reliability::Reliable` per stream and rely on causal chains (parent hashes link events).
- Other transports: ordering is the underlying system's (Redis Stream IDs are monotonic; JetStream sequences are monotonic; memory is per-shard).

There is no global ordering across the mesh. If the user wants "all events in the same order across all consumers," they're asking for Kafka-style partition leaders — Net does not provide this.

## "I want type safety on payloads"

- TS: `node.channel<MyType>('name', validator?)`. Validator runs on each received event. Without a validator, payload is cast to T; bad data crashes at use site.
- Python: `node.channel('name', MyDataclass)` — model is instantiated as `MyDataclass(**payload)`.
- Rust: `node.subscribe_typed::<T>()` deserializes via serde. Errors surface as `Result::Err`.
- Go / C: parse JSON yourself.

JSON is the wire format. There is no schema registry. Producer and consumer must agree on the shape — typically by sharing the type definition (TS interface, Python dataclass, Rust struct) across crates/packages.

## "I want auth — only some subscribers should see this channel"

Use **permission tokens** (see `net/README.md` § Security surface). The publisher's identity issues signed tokens; subscribers present them when subscribing. Publisher checks via a 20ns bloom filter on every emit. Revocation is immediate.

This is out of scope for the basic event-bus skill — point at the source if the user needs it. The native `net` binding exposes the token API; the SDK wrappers may not surface it cleanly yet.

## "I want a channel only visible inside one subnet (e.g. dev vs prod)"

Use **subnet policies** with capability tags. Tag dev nodes with `env:dev`, prod with `env:prod`, define a subnet policy that maps the tag to a subnet level, declare the channel as `SubnetLocal`. Cross-subnet subscribes are rejected at the publisher.

Out of scope for the basic skill. Point at `net/README.md` § Subnets.

## "I want per-tenant capability discovery without standing up subnets"

Use **scoped capability discovery** with reserved `scope:*` tags. Tag the provider's capability set with `scope:tenant:<id>` (or `scope:region:<name>`, or `scope:subnet-local`); have the consumer call `find_nodes_scoped(filter, ScopeFilter::Tenant("<id>"))`. The consumer gets only the matching pool plus any untagged "Global" providers (permissive default — opt *in* to narrowing, never out by accident).

This is purely a discovery filter, not a routing gate — the wire format and forwarders are unchanged. Useful when:
- A GPU pool is shared across tenants and the placement layer needs per-tenant isolation.
- Region-aware rendezvous selection ("only show me relays in `eu-west`").
- Subnet-local app discovery on a Deck without setting up a `SubnetPolicy`.

If the user actually wants channel routing scoped (publish/subscribe denied across boundaries), that's a *different* feature — point at the Subnets pattern above. Scope tags are about *who can be found*, not *what packets cross*.

## "I want to run a single-process test before deploying multi-host"

**Recipe:** memory transport. Same code, no network.

```typescript
const node = await NetNode.create({ shards: 4 });   // memory transport, default
```

When you switch to mesh transport for production, the application code does not change. Only the constructor changes.

## "I have an embedded/IoT device with limited resources"

Use C SDK (`net.h`) — the whole transport library compiles to ~2MB stripped. Memory transport for in-process; mesh transport if the device has a network interface and can talk UDP.

For very small devices that just produce data, the device doesn't need to subscribe — it just emits. Compute can be elsewhere on the mesh; capability-routed traffic finds it.

---

## Decision tree

```
Single process?
├── Yes → memory transport. Done.
└── No → multi-host
    ├── Already running Redis or NATS? → use that transport, free durability
    ├── Want full Net properties (no broker, encrypted relays, capability routing)? → mesh transport
    └── Want both (durable + mesh)? → mesh transport + RedEX adapter for local persistence
```

```
Need replay / durability?
├── No → done, just use the bus
└── Yes
    ├── Want operational simplicity, no extra infra? → RedEX (local files per node)
    ├── Already run Redis? → redis transport
    └── Already run NATS? → jetstream transport
```

```
Topic-based pub/sub?
├── Yes
│   ├── TS or Python? → node.channel('name')
│   └── Rust / Go / C? → emit typed events; filter on consumer
└── No, single firehose? → node.emit() / node.subscribe(), no channel API needed
```
