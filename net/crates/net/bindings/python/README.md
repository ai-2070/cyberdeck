# Blackstream Python

High-performance, schema-agnostic event bus for AI runtime workloads.

## Installation

```bash
pip install blackstream
```

## Quick Start

```python
from blackstream import Blackstream

# Create event bus (defaults to CPU core count shards)
bus = Blackstream()

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
with Blackstream(num_shards=4) as bus:
    bus.ingest_raw('{"data": "value"}')
# Automatically shuts down
```

## Configuration

```python
bus = Blackstream(
    num_shards=8,                    # Number of parallel shards
    ring_buffer_capacity=1_048_576,  # Events per shard (must be power of 2)
    backpressure_mode="drop_oldest", # What to do when full
)
```

## BLTP Encrypted UDP Transport

BLTP provides encrypted point-to-point UDP transport for high-performance scenarios:

```python
from blackstream import Blackstream, generate_bltp_keypair
import os

# Generate keypair for responder
keypair = generate_bltp_keypair()
psk = os.urandom(32).hex()

# Responder side
responder = Blackstream(
    num_shards=2,
    bltp_bind_addr='127.0.0.1:9001',
    bltp_peer_addr='127.0.0.1:9000',
    bltp_psk=psk,
    bltp_role='responder',
    bltp_secret_key=keypair.secret_key,
    bltp_public_key=keypair.public_key,
    bltp_reliability='light',  # 'none', 'light', or 'full'
)

# Initiator side (knows responder's public key)
initiator = Blackstream(
    num_shards=2,
    bltp_bind_addr='127.0.0.1:9000',
    bltp_peer_addr='127.0.0.1:9001',
    bltp_psk=psk,
    bltp_role='initiator',
    bltp_peer_public_key=keypair.public_key,
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
