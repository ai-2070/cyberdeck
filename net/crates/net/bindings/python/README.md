# Net Python

High-performance, schema-agnostic event bus for AI runtime workloads.

## Installation

```bash
pip install net
```

## Quick Start

```python
from net import Net

# Create event bus (defaults to CPU core count shards)
bus = Net()

# Ingest events - fast path with raw JSON strings (23M+ ops/sec)
bus.ingest_raw('{"token": "hello", "index": 0}')

# Or use dict for convenience (4M+ ops/sec)
bus.ingest({"token": "world", "index": 1})

# Batch ingestion for maximum throughput
events = [f'{{"token": "tok_{i}"}}' for i in range(10000)]
count = bus.ingest_raw_batch(events)

# Poll events
response = bus.poll(limit=100)
for event in response:
    print(event.raw)
    # Or parse to dict
    data = event.parse()

# Check stats
stats = bus.stats()
print(f"Ingested: {stats.events_ingested}, Dropped: {stats.events_dropped}")

# Shutdown
bus.shutdown()
```

## Context Manager

```python
with Net(num_shards=4) as bus:
    bus.ingest_raw('{"data": "value"}')
# Automatically shuts down
```

## Configuration

```python
bus = Net(
    num_shards=8,                    # Number of parallel shards
    ring_buffer_capacity=1_048_576,  # Events per shard (must be power of 2)
    backpressure_mode="drop_oldest", # What to do when full
)
```

## Net Encrypted UDP Transport

Net provides encrypted point-to-point UDP transport for high-performance scenarios:

```python
from net import Net, generate_net_keypair
import os

# Generate keypair for responder
keypair = generate_net_keypair()
psk = os.urandom(32).hex()

# Responder side
responder = Net(
    num_shards=2,
    net_bind_addr='127.0.0.1:9001',
    net_peer_addr='127.0.0.1:9000',
    net_psk=psk,
    net_role='responder',
    net_secret_key=keypair.secret_key,
    net_public_key=keypair.public_key,
    net_reliability='light',  # 'none', 'light', or 'full'
)

# Initiator side (knows responder's public key)
initiator = Net(
    num_shards=2,
    net_bind_addr='127.0.0.1:9000',
    net_peer_addr='127.0.0.1:9001',
    net_psk=psk,
    net_role='initiator',
    net_peer_public_key=keypair.public_key,
)

# Use as normal
initiator.ingest_raw('{"event": "data"}')
```

### Backpressure Modes

- `"drop_newest"` - Reject new events when buffer is full
- `"drop_oldest"` - Evict oldest events to make room
- `"fail_producer"` - Raise an error

## Performance Tips

1. **Use `ingest_raw()` for maximum throughput** - Pass pre-serialized JSON strings
2. **Use `ingest_raw_batch()` for bulk operations** - Reduces per-call overhead
3. **Increase `ring_buffer_capacity`** - Larger buffers handle bursts better
4. **Match `num_shards` to CPU cores** - Default is optimal for most cases

## License

Apache-2.0
