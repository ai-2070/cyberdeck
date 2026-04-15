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
