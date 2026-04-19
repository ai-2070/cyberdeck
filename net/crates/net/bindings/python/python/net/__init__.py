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

try:
    from ._net import CortexError, NetDbError
except ImportError:
    CortexError = None  # type: ignore[assignment]
    NetDbError = None  # type: ignore[assignment]

__all__ = [
    "Net",
    "IngestResult",
    "StoredEvent",
    "PollResponse",
    "Stats",
    "CortexError",
    "NetDbError",
]

__version__ = "0.1.0"
