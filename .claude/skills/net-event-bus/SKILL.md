---
name: net-event-bus
description: "Use this skill when the user is integrating the Net library (`@ai2070/net-sdk`, Rust `net-sdk`, Python `net-sdk`, Go `net` binding, or C `net.h`) as an event bus — anything involving publishing to or subscribing from a Net channel, wiring a producer/consumer/relay against the Net SDK, or migrating from Kafka/NATS/Redis Streams/Pulsar. Triggers on imports of those packages and on phrases like 'use Net for events', 'pub/sub with Net', 'wire up a Net channel', 'Net subscriber', 'Net publisher'. Skip for unrelated event-bus work or for editing Net's own internals."
allowed-tools: ["Read", "Grep", "Glob", "Bash", "Edit", "Write"]
---

# Net as an Event Bus

Net is **not Kafka**. It is not NATS, not Redis Streams, not Pulsar. The API surface looks superficially similar (publish, subscribe, channels) but the underlying model is different in ways that will produce wrong code if you assume it's just another broker.

**Before you write or edit any integration code, read `concepts.md` in this skill directory.** It is the conceptual prerequisite for everything else. The API templates in `apis.md` will look identical to a dozen broker SDKs and you will write something that compiles and runs and is wrong.

## How to use this skill

You have several reference files in this directory. Load them on demand — do not read them all up front.

| File | Read when |
|---|---|
| `concepts.md` | **Always** — before writing any integration code. The mental model. ~5 min read. |
| `apis.md` | When generating actual code. Verified per-SDK templates for publish, subscribe, lifecycle. |
| `patterns.md` | When the user describes a task ("I need a relay", "I need persistence", "I need fan-out across machines"). Maps tasks to recipes. |
| `runtime.md` | When writing a `shutdown` path, handling errors, integrating into an existing async runtime (axum, FastAPI, Express), or debugging "why are my events missing?" |
| `payloads.md` | When the user is shaping their event schema or asking about size limits, large blobs, or batching. |
| `testing.md` | When writing unit/integration tests against the SDK. Covers fixtures, race conditions, CI gotchas. |
| `gotchas.md` | When the user is migrating from Kafka / NATS / Redis Streams / Pulsar, or when their question reveals broker-thinking. |
| `examples/` | When the user is starting from scratch — minimal, runnable hello-world for each SDK. Use as the first thing they run after install, before they write application code. |

## TL;DR mental model (the absolute minimum)

If you remember nothing else from `concepts.md`, remember these five things — they are what makes Net different from every other bus:

1. **There is no broker.** A channel is a name, not a process. The publisher holds the subscriber list. Fan-out is N per-peer unicasts. On the **mesh** transport those unicasts ride already-encrypted sessions end-to-end; on **memory** there is no wire at all; on **Redis / JetStream** payloads sit in plaintext at the broker and rely on that system's TLS for transport security. "The bus" is the mesh of nodes themselves; nothing to provision, scale, or fail over.

2. **Backpressure is silence, not a signal.** Overloaded nodes drop packets and stop responding. They do not tell the sender. Neighbors detect the silence within a heartbeat and the mesh routes around them. Producers do not slow down — the mesh finds a different consumer.

3. **Subscribers are hot, not cold.** A subscriber sees events emitted *after* it subscribed (plus whatever's still in the ring buffer). There is no replay-from-beginning. If you need durable replay, you need a persistence layer (Redis adapter, JetStream adapter, or RedEX) — that's a separate decision, not a default.

4. **Every node is a peer.** No clients, no servers. Producer and consumer are the same primitive (`NetNode` / `Net`). A node can publish, subscribe, relay, and persist all at once.

5. **In TS, Python, and Rust, the transport is a runtime choice, not a code change.** The same publish/subscribe code works in-process (memory), across a LAN (mesh), through Redis, or via JetStream — you pick the transport at node construction and application logic doesn't know which one it got. **Go and C are different**: their API surface is poll-based regardless of transport, and the binding currently exposes a smaller, transport-specific surface, so swapping transports may require code changes (e.g. `NewMeshNode` constructor for mesh). Confirm against the SDK source when designing for portability across these.

If the user's design language conflicts with any of these (e.g. "the broker", "the cluster", "consumer group", "partition leader"), stop and read `gotchas.md` — they're carrying assumptions from another system that will break here.

## Workflow when integrating

1. **Identify the language** — Rust, TypeScript, Python, Go, or C. There are no other bindings.
2. **Read `concepts.md`** if this is your first invocation in the session.
3. **If the user is starting from scratch**, run the matching script in `examples/` first. Confirm the SDK is installed and working before writing application code.
4. **Clarify the task shape** — single-process or multi-host? Channels (named topics) or raw firehose? Need persistence? Need typed payloads? Read `patterns.md` for the recipe that matches.
5. **Pick the transport** — memory (single process), mesh (peer-to-peer over UDP), Redis, or JetStream. `concepts.md` covers the trade-offs. Default to `memory` for single-process tests and `mesh` for production.
6. **Generate code from `apis.md`** — these templates are verified against the SDK source. Adapt the payload type and channel name; do not invent new methods.
7. **Wire the lifecycle correctly** — read `runtime.md` for the shutdown contract and async-runtime integration before plugging into the user's existing app. Always add a `shutdown` path. The ring buffer needs a clean drain.
8. **Handle errors per `runtime.md`** — `Backpressure` is the only retry-safe error; everything else indicates state change, bug, or config issue.
9. **Shape the payload using `payloads.md`** — small JSON events on the bus, references for large blobs, batched events for telemetry streams.
10. **Write tests using `testing.md`** — memory transport, two in-process nodes, subscribe-before-publish, clean shutdown in teardown.
11. **If the user is migrating** from Kafka / NATS / Redis Streams / Pulsar, read `gotchas.md` first — broker assumptions will produce broken-but-compiling code.
12. **If you're unsure about an API**, read the SDK source directly:
   - Rust: `net/crates/net/sdk/src/` and `net/crates/net/sdk/examples/`
   - TypeScript: `net/crates/net/sdk-ts/src/`
   - Python: `net/crates/net/sdk-py/src/net_sdk/`
   - Go: `net/crates/net/bindings/go/`
   - C: `net/crates/net/include/net.h`
   These are authoritative. The README is a good intro; the source is ground truth.

## What this skill deliberately does not cover

The event-bus surface is a small slice of Net. The following exist but are out of scope here — they have their own primitives and would bloat this skill:

- **Daemons / Mikoshi (live state migration)** — stateful event processors that move between nodes carrying their causal chain. Read `net/README.md` § Daemons + Mikoshi if the user asks.
- **CortEX / NetDB (queryable folded state)** — local materialized views over RedEX logs. Read `net/README.md` § CortEX + NetDB.
- **Subprotocols** — custom protocols deployed incrementally over the same mesh. Out of scope unless the user is building one.
- **Capability-addressed routing, subnets, identity/permission tokens** — covered briefly in `concepts.md` because they shape channel visibility, but the full surface is a separate concern.
- **NAT traversal, port mapping, mesh transport internals** — operational concerns, not application API. Point at `net/crates/net/docs/`.

If the user asks about these, point them at the relevant section of `net/README.md` rather than guessing.
