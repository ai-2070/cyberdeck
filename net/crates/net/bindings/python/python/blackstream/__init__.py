"""
Blackstream - High-performance event bus for AI runtime workloads.

Example usage:

    from blackstream import Blackstream

    # Create event bus
    bus = Blackstream(num_shards=4)

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
    with Blackstream() as bus:
        bus.ingest_raw('{"data": "value"}')
"""

from ._blackstream import (
    Blackstream,
    IngestResult,
    StoredEvent,
    PollResponse,
    Stats,
)

__all__ = [
    "Blackstream",
    "IngestResult",
    "StoredEvent",
    "PollResponse",
    "Stats",
]

__version__ = "0.1.0"
