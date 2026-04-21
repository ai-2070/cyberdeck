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
        RedexError,
        RedexEvent,
        RedexFile,
        RedexTailIter,
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
            "RedexError",
            "RedexEvent",
            "RedexFile",
            "RedexTailIter",
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
        ChannelAuthError,
        ChannelError,
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
            "ChannelAuthError",
            "ChannelError",
            "NetKeypair",
            "NetMesh",
            "NetStream",
            "NetStreamStats",
            "NotConnectedError",
            "generate_net_keypair",
        ]
    )

# Identity + tokens surface. Present iff the native module was built
# with the `net` feature.
try:
    from ._net import (
        Identity,
        IdentityError,
        TokenError,
        channel_hash,
        delegate_token,
        normalize_gpu_vendor,
        parse_token,
        token_is_expired,
        verify_token,
    )
except ImportError:
    pass
else:
    __all__.extend(
        [
            "Identity",
            "IdentityError",
            "TokenError",
            "channel_hash",
            "delegate_token",
            "normalize_gpu_vendor",
            "parse_token",
            "token_is_expired",
            "verify_token",
        ]
    )

# Compute runtime surface. Present iff the native module was built
# with the `compute` feature. Stage 5 of SDK_COMPUTE_SURFACE_PLAN.md.
try:
    from ._net import (
        CausalEvent,
        DaemonError,
        DaemonHandle,
        DaemonRuntime,
        MigrationError,
        MigrationHandle,
    )
except ImportError:
    pass
else:
    __all__.extend(
        [
            "CausalEvent",
            "DaemonError",
            "DaemonHandle",
            "DaemonRuntime",
            "MigrationError",
            "MigrationHandle",
            "migration_error_kind",
        ]
    )

    def migration_error_kind(exc: "MigrationError") -> str | None:
        """Extract the migration-failure kind from a caught
        ``MigrationError``.

        The Rust side encodes migration failures as messages of the form
        ``"daemon: migration: <kind>[: <detail>]"``. This helper parses
        the kind out so callers can dispatch programmatically::

            try:
                migration.wait()
            except MigrationError as e:
                kind = migration_error_kind(e)
                if kind == "not-ready":
                    # ...retriable...
                elif kind == "factory-not-found":
                    # ...terminal, target mis-configured...

        Returns ``None`` if the message doesn't start with the expected
        prefix (shouldn't happen for exceptions raised by this module).
        """
        msg = str(exc)
        prefix = "daemon: migration:"
        if not msg.startswith(prefix):
            return None
        body = msg[len(prefix) :].strip()
        colon = body.find(":")
        return body if colon == -1 else body[:colon].strip()

__version__ = "0.1.0"
