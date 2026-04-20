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

## Channels (distributed pub/sub)

Named pub/sub over the encrypted mesh. Publishers register channels
with access policy; subscribers ask to join via a membership
subprotocol; `publish` fans payloads out to every current subscriber.

```python
from net import NetMesh, ChannelAuthError, ChannelError

pub = NetMesh('127.0.0.1:9001', '42' * 32)
try:
    pub.register_channel(
        'sensors/temp',
        visibility='global',      # or 'subnet-local' | 'parent-visible' | 'exported'
        reliable=True,
        priority=2,
        max_rate_pps=1000,
    )

    # Subscriber side (after handshake with pub):
    # sub.subscribe_channel(pub.node_id, 'sensors/temp')

    # Fan a payload out to all subscribers.
    report = pub.publish(
        'sensors/temp',
        b'{"celsius": 22.5}',
        reliability='reliable',
        on_failure='best_effort',
        max_inflight=32,
    )
    print(f"{report['delivered']}/{report['attempted']} subscribers received")
finally:
    pub.shutdown()

# Typed errors for ACL outcomes:
# try: sub.subscribe_channel(peer_id, 'restricted')
# except ChannelAuthError: ...   # publisher denied
# except ChannelError: ...       # unknown channel / other rejection
```

Channel names always cross the binding as strings (not the u16 hash)
to avoid ACL bypass via collision. The Python binding does not yet
expose a dedicated per-channel receive API; that is a follow-up.

## CortEX & NetDb (event-sourced state)

Typed, event-sourced state on top of RedEX — tasks and memories with
filterable queries and sync watch iterators. Includes the
`snapshot_and_watch` primitive whose race fix landed on v2, so you
can safely "paint what's there now, then react to changes" without
losing updates that race during construction.

```python
from net import NetDb, CortexError

db = NetDb.open(origin_hash=0xABCDEF01, with_tasks=True, with_memories=True)
tasks = db.tasks

try:
    seq = tasks.create(1, 'write docs', 100)
    tasks.wait_for_seq(seq)   # block until the fold has applied
except CortexError as e:
    # adapter-level failure (RedEX I/O, fold halted, etc.)
    ...

# Snapshot + watch, one atomic call — no race.
snap, it = tasks.snapshot_and_watch_tasks(status='pending')
print('initial:', len(snap), 'pending tasks')
for batch in it:
    print('update:', len(batch), 'pending tasks')
    if len(batch) == 0:
        it.close()    # idempotent; ends the iterator
        break

db.close()
```

### Standalone adapters

If you only need one model, skip the `NetDb` facade:

```python
from net import Redex, TasksAdapter

redex = Redex(persistent_dir='/var/lib/net/redex')
tasks = TasksAdapter.open(redex, origin_hash=0xABCDEF01, persistent=True)
```

`MemoriesAdapter` exposes the same shape with `store` / `retag` /
`pin` / `unpin` / `delete` / `list_memories` / `watch_memories` /
`snapshot_and_watch_memories`.

### Raw RedEX file (no CortEX fold)

For domain-agnostic persistent logs — your own event schema, no
fold, no typed adapter — open a `RedexFile` directly from a `Redex`.
The tail is a sync Python iterator; call `close()` or let
`StopIteration` fire when the file closes.

```python
from net import Redex, RedexError

redex = Redex(persistent_dir='/var/lib/net/events')
file = redex.open_file(
    'analytics/clicks',
    persistent=True,
    fsync_interval_ms=100,           # or fsync_every_n=1000
    retention_max_events=1_000_000,
)

# Append (or batch-append).
seq = file.append(b'{"url": "/home"}')
first = file.append_batch([b'{"a": 1}', b'{"a": 2}'])

# Tail — backfills the retained range, then streams live appends.
try:
    for event in file.tail(from_seq=0):
        print(event.seq, bytes(event.payload))
        if should_stop:
            break           # idempotent; ends the iterator via close()
except RedexError as e:
    ...

file.close()
```

Errors from the RedEX surface raise `RedexError` (invalid channel
name, bad config, append / tail / sync / close failures).

### Why `snapshot_and_watch_*`?

Calling `list_tasks()` then `watch_tasks()` takes two independent
state reads. A mutation landing between them would be silently lost
under the old `skip(1)` implementation. The atomic primitive returns
the snapshot and an iterator seeded so that any divergent initial
emission is forwarded through instead of dropped — see
[`docs/STORAGE_AND_CORTEX.md`](../../docs/STORAGE_AND_CORTEX.md).

## Performance Tips

1. **Use `ingest_raw()` for maximum throughput** - Pass pre-serialized JSON strings
2. **Use `ingest_raw_batch()` for bulk operations** - Reduces per-call overhead
3. **Increase `ring_buffer_capacity`** - Larger buffers handle bursts better
4. **Match `num_shards` to CPU cores** - Default is optimal for most cases

## License

Apache-2.0
