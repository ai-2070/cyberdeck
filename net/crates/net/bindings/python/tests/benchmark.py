#!/usr/bin/env python3
"""
Throughput benchmarks for Net Python SDK.

Measures pure SDK throughput using the in-memory noop adapter.

Run with: python tests/benchmark.py
"""

import json
import time
from dataclasses import dataclass
from typing import List

from net import Net


@dataclass
class BenchmarkResult:
    name: str
    total_events: int
    duration_ms: float
    events_per_second: float
    avg_latency_us: float


WARMUP_EVENTS = 10_000
BENCHMARK_EVENTS = 100_000
BATCH_SIZE = 1000

# Sample events of different sizes
SMALL_EVENT = '{"t":"x"}'
MEDIUM_EVENT = json.dumps(
    {
        "token": "hello_world",
        "index": 42,
        "session_id": "abc123def456",
        "timestamp": 1234567890,
        "metadata": {"key": "value"},
    }
)
LARGE_EVENT = json.dumps(
    {
        "token": "hello_world_this_is_a_longer_token",
        "index": 42,
        "session_id": "abc123def456ghi789jkl012mno345",
        "timestamp": 1234567890,
        "user_id": "user_12345678901234567890",
        "request_id": "req_abcdefghijklmnopqrstuvwxyz",
        "metadata": {
            "key1": "value1_with_some_extra_data",
            "key2": "value2_with_more_extra_data",
            "key3": "value3_with_even_more_data",
            "nested": {"a": 1, "b": 2, "c": 3, "d": 4, "e": 5},
        },
        "tags": ["tag1", "tag2", "tag3", "tag4", "tag5"],
        "content": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. "
        "Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    }
)


def create_bus(num_shards: int = 4) -> Net:
    """Create a bus with noop in-memory adapter."""
    return Net(
        num_shards=num_shards,
        ring_buffer_capacity=1 << 20,  # 1M events per shard
        backpressure_mode="drop_oldest",
    )


def warmup(bus: Net, event: str) -> None:
    """Warmup the bus with events."""
    for _ in range(WARMUP_EVENTS):
        bus.ingest_raw(event)


def benchmark_single_ingestion(
    bus: Net, event: str, count: int
) -> BenchmarkResult:
    """Benchmark single event ingestion."""
    start = time.perf_counter()

    for _ in range(count):
        bus.ingest_raw(event)

    duration_s = time.perf_counter() - start
    duration_ms = duration_s * 1000
    events_per_second = count / duration_s
    avg_latency_us = (duration_s / count) * 1_000_000

    return BenchmarkResult(
        name="single_ingestion",
        total_events=count,
        duration_ms=duration_ms,
        events_per_second=events_per_second,
        avg_latency_us=avg_latency_us,
    )


def benchmark_batch_ingestion(
    bus: Net, event: str, total_events: int, batch_size: int
) -> BenchmarkResult:
    """Benchmark batch event ingestion."""
    batch = [event] * batch_size
    num_batches = (total_events + batch_size - 1) // batch_size

    start = time.perf_counter()

    ingested = 0
    for _ in range(num_batches):
        ingested += bus.ingest_raw_batch(batch)

    duration_s = time.perf_counter() - start
    duration_ms = duration_s * 1000

    # Guard against division by zero if all ingestions failed
    if ingested == 0:
        return BenchmarkResult(
            name="batch_ingestion",
            total_events=0,
            duration_ms=duration_ms,
            events_per_second=0.0,
            avg_latency_us=float("inf"),
        )

    events_per_second = ingested / duration_s
    avg_latency_us = (duration_s / ingested) * 1_000_000

    return BenchmarkResult(
        name="batch_ingestion",
        total_events=ingested,
        duration_ms=duration_ms,
        events_per_second=events_per_second,
        avg_latency_us=avg_latency_us,
    )


def benchmark_dict_ingestion(
    bus: Net, event_dict: dict, count: int
) -> BenchmarkResult:
    """Benchmark dict/object ingestion (includes JSON serialization)."""
    start = time.perf_counter()

    for _ in range(count):
        bus.ingest(event_dict)

    duration_s = time.perf_counter() - start
    duration_ms = duration_s * 1000
    events_per_second = count / duration_s
    avg_latency_us = (duration_s / count) * 1_000_000

    return BenchmarkResult(
        name="dict_ingestion",
        total_events=count,
        duration_ms=duration_ms,
        events_per_second=events_per_second,
        avg_latency_us=avg_latency_us,
    )


def format_number(n: float) -> str:
    """Format a number with K/M suffix."""
    if n >= 1_000_000:
        return f"{n / 1_000_000:.2f}M"
    if n >= 1_000:
        return f"{n / 1_000:.2f}K"
    return f"{n:.2f}"


def print_result(result: BenchmarkResult, event_size: str) -> None:
    """Print a benchmark result."""
    print(
        f"  {result.name:<20} | {event_size:<10} | "
        f"{format_number(result.events_per_second):>10} events/sec | "
        f"{result.avg_latency_us:>10.2f} us/event | "
        f"{result.duration_ms:>8.0f} ms total"
    )


def run_event_size_benchmarks() -> None:
    """Run benchmarks for different event sizes."""
    print("\n=== Event Size Benchmarks (Single Ingestion) ===\n")

    with create_bus(4) as bus:
        for name, event in [
            ("small", SMALL_EVENT),
            ("medium", MEDIUM_EVENT),
            ("large", LARGE_EVENT),
        ]:
            warmup(bus, event)
            result = benchmark_single_ingestion(bus, event, BENCHMARK_EVENTS)
            print_result(result, f"{name} ({len(event)}B)")


def run_shard_count_benchmarks() -> None:
    """Run benchmarks for different shard counts."""
    print("\n=== Shard Count Benchmarks (Batch Ingestion) ===\n")

    for num_shards in [1, 2, 4, 8, 16]:
        with create_bus(num_shards) as bus:
            warmup(bus, MEDIUM_EVENT)
            result = benchmark_batch_ingestion(
                bus, MEDIUM_EVENT, BENCHMARK_EVENTS, BATCH_SIZE
            )
            print(
                f"  {num_shards} shards".ljust(20)
                + f" | {format_number(result.events_per_second):>10} events/sec | "
                f"{result.avg_latency_us:>10.2f} us/event"
            )


def run_ingestion_pattern_benchmarks() -> None:
    """Run benchmarks for different ingestion patterns."""
    print("\n=== Ingestion Pattern Benchmarks ===\n")

    with create_bus(4) as bus:
        warmup(bus, MEDIUM_EVENT)

        # Single sequential (raw string)
        single = benchmark_single_ingestion(bus, MEDIUM_EVENT, BENCHMARK_EVENTS)
        print_result(single, "medium")

        # Batch
        batch = benchmark_batch_ingestion(
            bus, MEDIUM_EVENT, BENCHMARK_EVENTS, BATCH_SIZE
        )
        print_result(batch, "medium")

        # Dict ingestion (includes JSON serialization overhead)
        medium_dict = json.loads(MEDIUM_EVENT)
        dict_result = benchmark_dict_ingestion(bus, medium_dict, BENCHMARK_EVENTS)
        print_result(dict_result, "medium")


def run_batch_size_benchmarks() -> None:
    """Run benchmarks for different batch sizes."""
    print("\n=== Batch Size Benchmarks ===\n")

    with create_bus(4) as bus:
        warmup(bus, MEDIUM_EVENT)

        for batch_size in [10, 100, 500, 1000, 5000]:
            result = benchmark_batch_ingestion(
                bus, MEDIUM_EVENT, BENCHMARK_EVENTS, batch_size
            )
            print(
                f"  batch_size={batch_size}".ljust(20)
                + f" | {format_number(result.events_per_second):>10} events/sec | "
                f"{result.avg_latency_us:>10.2f} us/event"
            )


def run_raw_vs_dict_benchmarks() -> None:
    """Compare raw string ingestion vs dict ingestion."""
    print("\n=== Raw String vs Dict Ingestion ===\n")

    with create_bus(4) as bus:
        warmup(bus, MEDIUM_EVENT)

        # Raw string (pre-serialized)
        raw_result = benchmark_single_ingestion(bus, MEDIUM_EVENT, BENCHMARK_EVENTS)
        print(
            f"  ingest_raw()".ljust(20)
            + f" | {format_number(raw_result.events_per_second):>10} events/sec | "
            f"{raw_result.avg_latency_us:>10.2f} us/event"
        )

        # Dict (requires JSON serialization)
        medium_dict = json.loads(MEDIUM_EVENT)
        dict_result = benchmark_dict_ingestion(bus, medium_dict, BENCHMARK_EVENTS)
        print(
            f"  ingest()".ljust(20)
            + f" | {format_number(dict_result.events_per_second):>10} events/sec | "
            f"{dict_result.avg_latency_us:>10.2f} us/event"
        )

        speedup = raw_result.events_per_second / dict_result.events_per_second
        print(f"\n  Raw ingestion is {speedup:.2f}x faster than dict ingestion")


def main() -> None:
    print("=============================================")
    print("   Net Python SDK Throughput Benchmarks")
    print("   (In-memory adapter)")
    print("=============================================")
    print(f"\nWarmup: {format_number(WARMUP_EVENTS)} events")
    print(f"Benchmark: {format_number(BENCHMARK_EVENTS)} events per test\n")

    try:
        run_event_size_benchmarks()
        run_shard_count_benchmarks()
        run_ingestion_pattern_benchmarks()
        run_batch_size_benchmarks()
        run_raw_vs_dict_benchmarks()

        print("\n=== Benchmark Complete ===\n")
    except Exception as e:
        print(f"Benchmark failed: {e}")
        raise


if __name__ == "__main__":
    main()
