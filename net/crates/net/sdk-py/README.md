# Net Python SDK

Ergonomic Python SDK for the Net mesh network.

Wraps the `net` PyO3 bindings with generators, typed events, typed channels, and a Pythonic API.

## Install

```bash
pip install net-sdk
```

Requires the `net` native package (PyO3 bindings).

## Quick Start

```python
from net_sdk import NetNode

node = NetNode(shards=4)

# Emit events
node.emit({'token': 'hello', 'index': 0})
node.emit_raw('{"token": "world"}')

# Batch
count = node.emit_batch([{'a': 1}, {'a': 2}, {'a': 3}])

# Poll
response = node.poll(limit=100)
for event in response:
    print(event.raw)

# Stream (generator)
for event in node.subscribe(limit=100):
    print(event.raw)

node.shutdown()
```

## Context Manager

```python
with NetNode(shards=4) as node:
    node.emit({'hello': 'world'})
    for event in node.subscribe(limit=10, timeout=5.0):
        print(event.raw)
```

## Typed Streams

### Dataclass

```python
from dataclasses import dataclass

@dataclass
class TokenEvent:
    token: str
    index: int

for token in node.subscribe_typed(TokenEvent, limit=100):
    print(f'{token.index}: {token.token}')
```

### Pydantic

```python
from pydantic import BaseModel

class TemperatureReading(BaseModel):
    sensor_id: str
    celsius: float
    timestamp: float

for reading in node.subscribe_typed(TemperatureReading, limit=100):
    print(f'{reading.sensor_id}: {reading.celsius}°C')
```

## Typed Channels

```python
from net_sdk import TypedChannel

temps = node.channel('sensors/temperature', TemperatureReading)

# Publish
temps.publish(TemperatureReading(sensor_id='A1', celsius=22.5, timestamp=1700000000.0))

# Subscribe
for reading in temps.subscribe():
    print(f'{reading.sensor_id}: {reading.celsius}°C')
```

## Ingestion Methods

| Method | Input | Speed | Returns |
|--------|-------|-------|---------|
| `emit(obj)` | dict, dataclass, Pydantic | Fast | `Receipt` |
| `emit_raw(json)` | str | Fastest | `Receipt` |
| `emit_batch(objs)` | list | Bulk | `int` |
| `emit_raw_batch(jsons)` | list[str] | Bulk fastest | `int` |
| `fire(json)` | str | Fire-and-forget | None |

## Transports

```python
# In-memory (default)
node = NetNode(shards=4)

# Redis
node = NetNode(shards=4, redis_url='redis://localhost:6379')

# JetStream
node = NetNode(shards=4, jetstream_url='nats://localhost:4222')

# Encrypted mesh
node = NetNode(
    shards=4,
    mesh_bind='0.0.0.0:9000',
    mesh_peer='192.168.1.10:9001',
    mesh_psk='...',
    mesh_role='initiator',
    mesh_peer_public_key='...',
)
```

## NAT Traversal (optimization, not correctness)

Two NATed peers already reach each other through the mesh's routed-handshake path. NAT traversal opens a shorter direct path when the NAT shape allows it; it's never required for connectivity. The surface is exposed on the underlying `net` PyO3 module and is a no-op when the native package was built without `--features nat-traversal`.

```python
# Access via the underlying PyO3 handle. Ergonomic sdk-py
# wrappers are a planned follow-up; the PyO3 methods mirror
# the Rust SDK surface directly.
mesh = node._native_mesh  # PyMeshNode

mesh.reclassify_nat()

klass  = mesh.nat_type()            # "open" | "cone" | "symmetric" | "unknown"
reflex = mesh.reflex_addr()         # "203.0.113.5:9001" | None

observed = mesh.probe_reflex(peer_node_id)   # "ip:port"

# Attempt a direct connection via the pair-type matrix.
# `coordinator` mediates the punch when the matrix picks one.
# Always returns — stats tell you which path won.
mesh.connect_direct(peer_node_id, peer_pubkey_hex, coordinator_node_id)

# Cumulative counters — all int, monotonic.
s = mesh.traversal_stats()
s.punches_attempted   # coordinator mediated a PunchRequest + Introduce
s.punches_succeeded   # ack arrived AND direct handshake landed
s.relay_fallbacks     # landed on the routed path after skip/fail
```

Operators with a known-public address skip the classifier sweep entirely. The override pins `"open"` + the supplied address on every capability announcement; call `announce_capabilities()` after to propagate (the setter resets the rate-limit floor so the next announce is guaranteed to broadcast).

```python
mesh.set_reflex_override('203.0.113.5:9001')
mesh.announce_capabilities(caps)
# later:
mesh.clear_reflex_override()
mesh.announce_capabilities(caps)
```

`TraversalError` carries a stable `kind` discriminator (`reflex-timeout` | `peer-not-reachable` | `transport` | `rendezvous-no-relay` | `rendezvous-rejected` | `punch-failed` | `port-map-unavailable` | `unsupported`). A build without the `nat-traversal` feature surfaces every traversal call as `unsupported` — the routed path keeps working regardless.

## Mesh Streams (multi-peer + back-pressure)

For direct peer-to-peer messaging — open a stream to a specific peer
and catch back-pressure as a first-class exception:

```python
from net_sdk import MeshNode, BackpressureError, NotConnectedError

node = MeshNode(bind_addr='127.0.0.1:9000', psk='00' * 32)
# ... handshake (node.connect(...) / node.accept(...)) ...

stream = node.open_stream(
    peer_node_id=peer_id,
    stream_id=0x42,
    reliability='reliable',
    window_bytes=256,    # max in-flight packets before BackpressureError
)

# Three canonical daemon patterns:

# 1. Drop on pressure — best for telemetry / sampled streams.
try:
    node.send_on_stream(stream, [b'{}'])
except BackpressureError:
    metrics.inc('stream.backpressure_drops')
except NotConnectedError:
    # peer gone or stream closed — reopen if needed
    pass

# 2. Retry with exponential backoff (5 ms → 200 ms, up to max_retries).
node.send_with_retry(stream, [b'{}'], max_retries=8)

# 3. Block until the network lets up (bounded retry, ~13 min worst case).
# Releases the GIL for the duration, so other Python threads keep running.
node.send_blocking(stream, [b'{}'])

# Live stats — tx/rx seq, in-flight, window, backpressure count.
stats = node.stream_stats(peer_id, 0x42)
```

Both exceptions inherit from `Exception` and are re-exported from
`net_sdk`, so `try`/`except` works as expected. The transport never
retries or buffers on its own behalf — the helper methods are
opt-in policies, not defaults. See `docs/TRANSPORT.md` for the full
contract.

## Security (identity, tokens, capabilities, subnets)

The full security surface — ed25519 `Identity`, `PermissionToken`
issue / install / delegate, `CapabilityAnnouncement` broadcast +
`find_peers`, `SubnetId` / `SubnetPolicy`, channel auth with
`publish_caps` / `subscribe_caps` / `require_token` — is shipped
on the underlying **`net`** PyO3 package, not this wrapper. Import
directly:

```python
from net import (
    Identity, TokenError, IdentityError,
    parse_token, verify_token, delegate_token, channel_hash,
)
from net import NetMesh  # adds announce_capabilities / find_peers /
                        # entity_id / subscribe_channel(..., token=)
```

Quick example — issue a token and round-trip it through the mesh:

```python
import os
from net import Identity, NetMesh

seed = os.urandom(32)                     # persist via your own secret manager
identity = Identity.from_seed(seed)

# Mesh reuses the same keypair — `entity_id` is stable across restarts.
mesh = NetMesh(
    "127.0.0.1:9000",
    psk="42" * 32,
    identity_seed=seed,
)
assert mesh.entity_id == identity.entity_id

# Issue a SUBSCRIBE-scope token for a grantee.
grantee = Identity.generate()
token = identity.issue_token(
    subject=grantee.entity_id,
    scope=["subscribe"],
    channel="sensors/temp",
    ttl_seconds=300,
)

# Publisher gates the channel on tokens; subscribers attach them.
mesh.register_channel("sensors/temp", require_token=True)
# subscriber_mesh.subscribe_channel(mesh.node_id, "sensors/temp", token=token)
```

`TokenError` messages have the form `"token: <kind>"` where `<kind>`
is one of `invalid_format | invalid_signature | expired |
not_yet_valid | delegation_exhausted | delegation_not_allowed |
not_authorized`. Parse with `str(e).removeprefix("token: ")` for
programmatic dispatch.

Full surface + runnable examples:
[`bindings/python/README.md`](../bindings/python/README.md#security-surface-stage-ae).
Cross-SDK contract + rationale:
[`docs/SDK_SECURITY_SURFACE_PLAN.md`](../docs/SDK_SECURITY_SURFACE_PLAN.md).

> **Note.** The `net_sdk` wrapper (generators / typed channels /
> Pydantic) doesn't yet re-export the security types — use `net`
> directly for the identity / capability / subnet / channel-auth
> paths. Follow-up work to proxy them through `net_sdk` is tracked
> in [`SDK_PYTHON_PARITY_PLAN.md`](../docs/SDK_PYTHON_PARITY_PLAN.md).

## Compute (daemons + migration)

The full compute surface — `DaemonRuntime`, `MeshDaemon`
(duck-typed), `DaemonHandle`, `MigrationHandle`, plus the six-
phase migration orchestrator — ships on the native **`net`** PyO3
package (when built with the `compute` feature). Import directly:

```python
from net import (
    DaemonRuntime, NetMesh, Identity, CausalEvent,
    DaemonError, MigrationError, migration_error_kind,
)


class EchoDaemon:
    name = "echo"

    def process(self, event):
        return [bytes(event.payload)]


mesh = NetMesh("127.0.0.1:9000", "42" * 32)
rt = DaemonRuntime(mesh)
rt.register_factory("echo", lambda: EchoDaemon())
rt.start()

handle = rt.spawn("echo", Identity.generate())
# rt.deliver(handle.origin_hash, CausalEvent(handle.origin_hash, 1, b"hi"))
rt.stop(handle.origin_hash)
rt.shutdown()
```

Live migration (`rt.start_migration(origin, src, dst)`) returns a
`MigrationHandle` whose `wait()` drives the cutover to completion;
failures raise `MigrationError` with a stable `kind` parseable by
`migration_error_kind(e)`. Full surface + runnable examples:
[`bindings/python/README.md`](../bindings/python/README.md#compute-daemons--migration).
Cross-SDK contract + rationale:
[`docs/SDK_COMPUTE_SURFACE_PLAN.md`](../docs/SDK_COMPUTE_SURFACE_PLAN.md).

> **Note.** Like the security types, the `net_sdk` wrapper doesn't
> yet re-export `DaemonRuntime` / `MigrationHandle` — use the
> native `net` package directly. Proxying these through `net_sdk`
> is tracked in
> [`SDK_PYTHON_PARITY_PLAN.md`](../docs/SDK_PYTHON_PARITY_PLAN.md).

## Groups (replica / fork / standby)

HA / scaling overlays on top of `DaemonRuntime` — `ReplicaGroup`,
`ForkGroup`, `StandbyGroup` — ship on the native **`net`** PyO3
package (when built with the `groups` feature). Import directly:

```python
from net import (
    DaemonRuntime, NetMesh, Identity, CausalEvent,
    ReplicaGroup, ForkGroup, StandbyGroup,
    GroupError, group_error_kind,
)


class CounterDaemon:
    """Minimal stateful daemon — increments on every event."""

    name = "counter"

    def __init__(self):
        self._count = 0

    def process(self, event: CausalEvent) -> list[bytes]:
        self._count += 1
        return [self._count.to_bytes(4, "little")]


# Build a mesh + runtime, register the factory, flip to Ready.
mesh = NetMesh("127.0.0.1:9000", "42" * 32)
rt = DaemonRuntime(mesh)
rt.register_factory("counter", lambda: CounterDaemon())
rt.start()

# A sample event — `rt.deliver` expects a `CausalEvent` per the
# compute surface. The origin/sequence match the replica the
# group routes to.
event = CausalEvent(0x1234_5678, sequence=1, payload=b"tick")

# N interchangeable replicas with deterministic per-index identity.
replicas = ReplicaGroup.spawn(
    rt, "counter",
    replica_count=3,
    group_seed=bytes([0x11] * 32),
    lb_strategy="consistent-hash",   # or "round-robin" | "least-load" | ...
)
origin = replicas.route_event({"routing_key": "user:42"})
rt.deliver(origin, CausalEvent(origin, sequence=1, payload=b"tick"))
replicas.scale_to(5)

# N independent daemons forked from a common parent; verifiable lineage.
forks = ForkGroup.fork(
    rt, "counter",
    parent_origin=0xABCDEF01,
    fork_seq=42,
    fork_count=3,
    lb_strategy="round-robin",
)
assert forks.verify_lineage()

# Active-passive replication with replay on promotion.
hot = StandbyGroup.spawn(
    rt, "counter",
    member_count=3,                  # 1 active + 2 standbys
    group_seed=bytes([0x77] * 32),
)
active_origin = hot.active_origin()
active_event = CausalEvent(active_origin, sequence=1, payload=b"tick")
rt.deliver(active_origin, active_event)
hot.on_event_delivered(active_event)  # keep standby replay buffer accurate
hot.sync_standbys()                   # periodic catchup

try:
    ReplicaGroup.spawn(
        rt, "never-registered",
        replica_count=2, group_seed=bytes(32),
        lb_strategy="round-robin",
    )
except GroupError as e:
    kind = group_error_kind(e)
    # kind ∈ { "not-ready", "factory-not-found", "no-healthy-member",
    #         "placement-failed", "registry-failed", "invalid-config",
    #         "daemon", "unknown" }
```

Full surface + runnable examples:
[`bindings/python/README.md`](../bindings/python/README.md#compute-groups-replica--fork--standby).
Cross-SDK contract + rationale:
[`docs/SDK_GROUPS_SURFACE_PLAN.md`](../docs/SDK_GROUPS_SURFACE_PLAN.md).

> **Note.** `net_sdk` does not yet proxy the groups surface; use the
> native `net` package directly, the same way as the security types.

## API

| Method | Description |
|--------|-------------|
| `NetNode(shards=4)` | Create a new node |
| `emit(obj)` | Emit dict, dataclass, or Pydantic model |
| `emit_raw(json)` | Emit a JSON string (fastest) |
| `emit_batch(objs)` | Batch emit |
| `emit_raw_batch(jsons)` | Batch emit strings |
| `fire(json)` | Fire-and-forget |
| `poll(limit)` | One-shot poll |
| `poll_one()` | Poll a single event |
| `subscribe(limit, timeout)` | Generator stream |
| `subscribe_typed(model)` | Typed generator stream |
| `channel(name, model)` | Create a typed channel |
| `stats()` | Ingestion statistics |
| `shards()` | Number of active shards |
| `shutdown()` | Graceful shutdown |
| `bus` | Access underlying PyO3 binding |

## License

Apache-2.0
