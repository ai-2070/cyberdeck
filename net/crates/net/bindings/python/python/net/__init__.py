"""
Net - High-performance event bus for AI runtime workloads.

Example usage:

    from net import Net

    # Create event bus
    bus = Net(num_shards=4)

    # Ingest events (fast path with raw JSON string)
    result = bus.ingest_raw('{"token": "hello", "index": 0}')
    print(f"Ingested to shard {result.shard_id}")

    # Or ingest a dict (convenience method)
    bus.ingest({"token": "world", "index": 1})

    # Batch ingestion for maximum throughput
    events = [f'{{"token": "tok_{i}"}}' for i in range(1000)]
    count = bus.ingest_raw_batch(events)
    print(f"Ingested {count} events")

    # Poll events
    response = bus.poll(limit=100)
    for event in response:
        print(event.raw)

    # Get stats
    stats = bus.stats()
    print(f"Total ingested: {stats.events_ingested}")

    # Shutdown
    bus.shutdown()

    # Or use as context manager
    with Net() as bus:
        bus.ingest_raw('{"data": "value"}')
"""

from ._net import (
    Net,
    IngestResult,
    StoredEvent,
    PollResponse,
    Stats,
)

__all__ = [
    "Net",
    "IngestResult",
    "StoredEvent",
    "PollResponse",
    "Stats",
]

# CortEX + NetDB surface. Present iff the native module was built with
# the `cortex` feature (maturin's default picks it up).
try:
    from ._net import (
        CortexError,
        NetDb,
        NetDbError,
        MemoriesAdapter,
        Memory,
        MemoryWatchIter,
        Redex,
        Task,
        TasksAdapter,
        TaskWatchIter,
    )
except ImportError:
    # `cortex` feature not compiled in; symbols stay undefined.
    pass
else:
    __all__.extend(
        [
            "CortexError",
            "MemoriesAdapter",
            "Memory",
            "MemoryWatchIter",
            "NetDb",
            "NetDbError",
            "Redex",
            "Task",
            "TasksAdapter",
            "TaskWatchIter",
        ]
    )

# Encrypted mesh transport + per-peer streams. Present iff the native
# module was built with the `net` feature.
try:
    from ._net import (
        BackpressureError,
        NetKeypair,
        NetMesh,
        NetStream,
        NetStreamStats,
        NotConnectedError,
        generate_net_keypair,
    )
except ImportError:
    # `net` feature not compiled in; symbols stay undefined.
    pass
else:
    __all__.extend(
        [
            "BackpressureError",
            "NetKeypair",
            "NetMesh",
            "NetStream",
            "NetStreamStats",
            "NotConnectedError",
            "generate_net_keypair",
        ]
    )

__version__ = "0.1.0"
