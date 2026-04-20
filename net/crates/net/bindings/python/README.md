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

## Security Surface (Stage A–E)

The mesh layer surfaces the same identity / capabilities / subnets /
channel-auth story that the Rust SDK and the TypeScript / Node SDKs
ship. Full staging and rationale:
[`docs/SDK_SECURITY_SURFACE_PLAN.md`](../../docs/SDK_SECURITY_SURFACE_PLAN.md).
Python-binding parity details:
[`docs/SDK_PYTHON_PARITY_PLAN.md`](../../docs/SDK_PYTHON_PARITY_PLAN.md).

### Identity + permission tokens

Every node has an ed25519 identity; permission tokens are ed25519-
signed delegations that authorize a subject to `publish` /
`subscribe` / `delegate` / `admin` on a channel, optionally with
further delegation depth.

```python
from net import Identity, parse_token, verify_token, delegate_token

alice = Identity.generate()
bob = Identity.generate()

# Alice issues Bob a subscribe+delegate token good for 5 min, with
# one re-delegation hop remaining.
token = alice.issue_token(
    subject=bob.entity_id,
    scope=["subscribe", "delegate"],
    channel="sensors/temp",
    ttl_seconds=300,
    delegation_depth=1,
)
assert verify_token(token) is True

# Bob re-delegates to Carol; depth drops to 0 (leaf).
carol = Identity.generate()
child = delegate_token(bob, token, carol.entity_id, ["subscribe"])
assert parse_token(child)["delegation_depth"] == 0
```

### Capability announcements + peer discovery

Announce hardware / software / model / tool / tag fingerprints, then
query the local capability index with a filter.

```python
mesh.announce_capabilities({
    "hardware": {
        "cpu_cores": 16,
        "memory_mb": 65536,
        "gpu": {"vendor": "nvidia", "model": "h100", "vram_mb": 81920},
    },
    "models": [{"model_id": "llama-3.1-70b", "family": "llama",
                "context_length": 128_000}],
    "tags": ["gpu", "prod"],
})

gpu_peers = mesh.find_peers({
    "require_gpu": True,
    "gpu_vendor": "nvidia",
    "min_vram_mb": 40_000,
})
```

Capability propagation is one-hop in v1; peers >1 hop away will not
see the announcement.

### Subnets

Nodes can bind to a hierarchical `SubnetId` (1–4 levels, each 0–255)
directly, or derive one from announced tags via a `SubnetPolicy`.

```python
# Explicit subnet.
mesh = NetMesh("127.0.0.1:9000", PSK, subnet=[3, 7, 2])

# Or derive from tags.
mesh = NetMesh(
    "127.0.0.1:9001", PSK,
    subnet_policy={
        "rules": [
            {"tag_prefix": "region:", "level": 0,
             "values": {"eu": 1, "us": 2, "apac": 3}},
            {"tag_prefix": "zone:", "level": 1,
             "values": {"a": 1, "b": 2, "c": 3}},
        ]
    },
)
```

### Channel authentication

Publishers set `publish_caps` / `subscribe_caps` / `require_token` on
`register_channel`. Subscribers present a `PermissionToken` via the
optional `token=bytes` kwarg on `subscribe_channel`.

```python
mesh.register_channel(
    "gpu/jobs",
    subscribe_caps={"require_gpu": True, "min_vram_mb": 16_000},
    require_token=True,
)

# Subscriber side, with a token issued by the publisher:
mesh.subscribe_channel(publisher_node_id, "gpu/jobs", token=token_bytes)
```

Denied subscribes raise `ChannelAuthError` (a subclass of
`ChannelError`); malformed tokens raise `TokenError` whose message
has the form `"token: <kind>"` (`invalid_signature`, `expired`,
`delegation_exhausted`, …). Cross-SDK behaviour is fixed by the
Rust integration suite; see
[`tests/channel_auth.rs`](../../tests/channel_auth.rs).

## Performance Tips

1. **Use `ingest_raw()` for maximum throughput** - Pass pre-serialized JSON strings
2. **Use `ingest_raw_batch()` for bulk operations** - Reduces per-call overhead
3. **Increase `ring_buffer_capacity`** - Larger buffers handle bursts better
4. **Match `num_shards` to CPU cores** - Default is optimal for most cases

## License

Apache-2.0
