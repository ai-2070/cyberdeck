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
