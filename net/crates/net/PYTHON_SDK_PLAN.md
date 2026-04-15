# Python SDK Plan

Pure Python package (`net-sdk`) over the PyO3 bindings (`net`). The PyO3 layer stays thin — raw FFI to the Rust core. The SDK adds the Pythonic developer experience.

## Current State

The PyO3 bindings expose:
- `Net(num_shards=4, ...)` — constructor with kwargs
- `ingest_raw(json_str)` / `ingest_raw_batch(strs)` — string ingestion
- `ingest(dict)` — dict convenience method
- `poll(limit, cursor, filter, ordering)` — cursor-based consumption
- `shutdown()` / `stats()` / `num_shards()`
- Context manager support (`with Net() as node:`)

No streaming, no typed events, no async, no middleware, no mesh access.

## Phase 1: Core — `NetNode`

```python
from net_sdk import NetNode

node = NetNode(
    shards=4,
    buffer_capacity=1 << 20,
    backpressure='drop_oldest',
    # Transport (pick one):
    transport='memory',
    transport=Redis('redis://localhost:6379'),
    transport=Mesh(bind='0.0.0.0:9000', peer='peer:9001', psk=psk, peer_public_key=pk),
)
```

### Ingestion

```python
# Dict (serializes to JSON)
node.emit({'token': 'hello', 'index': 0})

# Raw string (fast path)
node.emit_raw('{"token": "hello"}')

# Batch
node.emit_batch([event1, event2, event3])
node.emit_raw_batch(['{"a":1}', '{"a":2}'])

# Fire-and-forget (no return value)
node.fire('{"token": "hello"}')
```

### Consumption

```python
# One-shot poll
response = node.poll(limit=100)
for event in response:
    print(event.raw)

# Context manager
with NetNode(shards=4) as node:
    node.emit({'hello': 'world'})
```

## Phase 2: Generators & Async

The Pythonic way to consume streams.

### Generator (synchronous)

```python
# Infinite generator — yields events as they arrive
for event in node.subscribe(limit=100):
    print(event.raw)

# Typed — auto-deserializes to dataclass/dict
for reading in node.subscribe(limit=100, model=TemperatureReading):
    print(f'{reading.sensor_id}: {reading.celsius}°C')

# With timeout
for event in node.subscribe(limit=100, timeout=5.0):
    process(event)
```

### AsyncIO

```python
import asyncio
from net_sdk import AsyncNetNode

node = await AsyncNetNode.create(shards=4)

# Async emit
await node.emit({'token': 'hello'})

# Async generator
async for event in node.subscribe(limit=100):
    print(event.raw)

# Async context manager
async with AsyncNetNode(shards=4) as node:
    await node.emit({'hello': 'world'})
```

## Phase 3: Typed Events

### Dataclass Integration

```python
from dataclasses import dataclass
from net_sdk import NetEvent

@dataclass
class TokenEvent(NetEvent):
    channel = 'tokens'
    token: str
    index: int

# Emit typed
node.emit(TokenEvent(token='hello', index=0))

# Subscribe typed
for token in node.subscribe(model=TokenEvent):
    print(token.token, token.index)
```

### Pydantic Integration

```python
from pydantic import BaseModel
from net_sdk import NetEvent

class TemperatureReading(BaseModel, NetEvent):
    channel = 'sensors/temperature'
    sensor_id: str
    celsius: float
    timestamp: float

# Runtime validation on emit and subscribe
node.emit(TemperatureReading(sensor_id='A1', celsius=22.5, timestamp=time.time()))

for reading in node.subscribe(model=TemperatureReading):
    assert isinstance(reading.celsius, float)  # guaranteed by pydantic
```

## Phase 4: Typed Channels

```python
from net_sdk import TypedChannel

temps: TypedChannel[TemperatureReading] = node.channel('sensors/temperature', TemperatureReading)

temps.publish(TemperatureReading(sensor_id='A1', celsius=22.5, timestamp=time.time()))

for reading in temps.subscribe():
    print(f'{reading.sensor_id}: {reading.celsius}°C')
```

## Phase 5: Ready-Made Components

### Decorators (the Pythonic way to define daemons)

```python
from net_sdk import daemon, on_event, on_channel

@daemon
class TemperatureMonitor:
    @on_channel('sensors/temperature', model=TemperatureReading)
    def handle_temperature(self, reading: TemperatureReading):
        if reading.celsius > 100:
            self.node.emit({'channel': 'alerts/temperature', **reading.dict()})

    @on_event
    def handle_any(self, event):
        log.info(f'Event: {event.raw}')

node.host(TemperatureMonitor())
```

### Router

```python
from net_sdk import Router

router = Router(node)
router.route('sensors/*', 'analytics/ingest')
router.route('alerts/*', 'ops/alerts')

@router.handle('sensors/temperature')
def on_temperature(event):
    reading = json.loads(event.raw)
    if reading['celsius'] > 100:
        node.emit({'channel': 'alerts/temperature', **reading})
```

### Aggregator

```python
from net_sdk import Aggregator

avg = Aggregator(
    node,
    channel='sensors/temperature',
    window=5.0,  # seconds
    reduce=lambda events: {'avg_celsius': sum(e['celsius'] for e in events) / len(events)},
    emit_to='analytics/temperature_avg',
)
```

### Middleware

```python
from net_sdk.middleware import pipeline, rate_limit, filter_channel, transform, logger

node = NetNode(
    shards=4,
    middleware=pipeline(
        logger(),
        rate_limit(per_second=10_000),
        filter_channel('sensors/*'),
        transform(lambda event: enrich(event)),
    ),
)
```

### Primitives

```python
from net_sdk.primitives import Fanout, Merge, Dedup, Buffer, Retry, Ticker, Watchdog

fanout = Fanout(node, 'events/raw', ['analytics', 'logging', 'alerts'])
merge = Merge(node, ['sensors/temp', 'sensors/humidity'], 'sensors/all')
dedup = Dedup(node, 'events/raw', window=10.0)
```

## Phase 6: Discovery & Mesh

```python
# Find nodes by capability
gpu_nodes = node.discover(capability='gpu', min_vram=16)
nearby = node.discover(within_hops=3)
models = node.discover(serves_model='gemma-21b')

# Mesh info
mesh = node.mesh()
mesh.capabilities()
mesh.graph()
mesh.routes()
```

## Phase 7: Inference

First-class support for the primary use case.

```python
from net_sdk.inference import InferenceNode

# Serve inference on this node
server = InferenceNode(
    node,
    model='gemma-21b',
    vram_gb=24,
    max_concurrent=8,
    channel='inference/requests',
)

# Request inference from the mesh
response = node.infer('inference/requests', prompt='What is the meaning of life?', max_tokens=256)

# Streaming inference (generator)
for token in node.infer_stream('inference/requests', prompt=prompt):
    print(token, end='', flush=True)

# Async streaming
async for token in node.ainfer_stream('inference/requests', prompt=prompt):
    print(token, end='', flush=True)
```

## Phase 8: Testing Utilities

```python
from net_sdk.testing import TestMesh
import pytest

@pytest.fixture
async def mesh():
    m = TestMesh(3)  # 3 interconnected nodes
    yield m
    m.teardown()

def test_event_propagation(mesh):
    mesh[0].emit({'hello': 'world'})
    event = mesh[2].poll_one()
    assert event is not None
    assert json.loads(event.raw) == {'hello': 'world'}

def test_partition_recovery(mesh):
    mesh.partition(0, 2)
    mesh[0].emit({'during': 'partition'})
    assert mesh[2].poll_one() is None
    mesh.heal()
    # events flow again
```

## Phase 9: Observability

```python
from net_sdk.metrics import MetricsCollector

collector = MetricsCollector(node)

@collector.on('event_ingested')
def on_ingest(latency, shard):
    histogram.observe(latency)

@collector.on('peer_discovered')
def on_peer(node_id, hops):
    log.info(f'New peer: {node_id} at {hops} hops')

# Prometheus integration
from net_sdk.metrics.prometheus import prometheus_exporter
node.use(prometheus_exporter(port=9090))

# OpenTelemetry integration
from net_sdk.metrics.otel import otel_exporter
node.use(otel_exporter(endpoint='http://localhost:4317'))
```

## Phase 10: Scheduled Events

```python
from net_sdk.schedule import every, cron

# Simple interval
node.every(30, channel='health/heartbeat', payload=lambda: node.stats())

# Cron expression
node.cron('*/5 * * * *', channel='sensors/poll', payload=lambda: read_sensors())
```

## Structure

```
net-sdk/
  pyproject.toml        # net-sdk package
  src/
    net_sdk/
      __init__.py       # re-exports
      node.py           # NetNode, AsyncNetNode
      config.py         # transport configs (Redis, Mesh, etc.)
      stream.py         # generators, async generators
      channel.py        # TypedChannel
      event.py          # NetEvent base, dataclass/pydantic integration
      daemon.py         # @daemon, @on_event, @on_channel decorators
      router.py         # Router
      aggregator.py     # Aggregator
      middleware.py      # pipeline, rate_limit, filter, transform, logger
      primitives.py     # Fanout, Merge, Dedup, Buffer, Retry, Ticker, Watchdog
      discovery.py      # discover()
      mesh.py           # MeshHandle
      inference.py      # InferenceNode, infer(), infer_stream()
      testing.py        # TestMesh, pytest fixtures
      schedule.py       # every(), cron()
      metrics/
        __init__.py     # MetricsCollector
        prometheus.py   # Prometheus exporter
        otel.py         # OpenTelemetry exporter
      types.py          # shared types
  tests/
    test_node.py
    test_stream.py
    test_channel.py
    test_middleware.py
    test_inference.py
    test_testing.py
```

## Dependencies

`net` (PyO3 bindings) — required
`pydantic` — optional, for runtime validation
`prometheus-client` — optional, for Prometheus export
`opentelemetry-api` — optional, for OTel export

Pure Python over PyO3 bindings. Everything heavy happens in Rust.
