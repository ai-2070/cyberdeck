# Blackstream Benchmarks

Performance benchmarks for the Net Rust core and BLTP transport layer.

**Test System:** Apple M1 Max, macOS

## Net Header Operations

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Serialize | 1.90 ns | **526M ops/sec** |
| Deserialize | 1.87 ns | **534M ops/sec** |
| Roundtrip | 1.87 ns | **534M ops/sec** |
| AAD generation | 0.94 ns | **1.07G ops/sec** |

## Event Frame Serialization

### Single Write

| Payload Size | Latency | Throughput |
|-------------:|---------|------------|
| 64B | 18.48 ns | **3.22 GiB/s** |
| 256B | 49.41 ns | **4.83 GiB/s** |
| 1KB | 37.37 ns | **25.5 GiB/s** |
| 4KB | 88.30 ns | **43.2 GiB/s** |

### Batch Write (64B events)

| Batch Size | Latency | Throughput |
|-----------:|---------|------------|
| 1 | 18.24 ns | **3.27 GiB/s** |
| 10 | 69.81 ns | **8.54 GiB/s** |
| 50 | 147.48 ns | **20.2 GiB/s** |
| 100 | 271.97 ns | **21.9 GiB/s** |

### Batch Read

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Read batch (10 events) | 137.29 ns | **72.8M ops/sec** |

## Packet Pool (Zero-Allocation)

| Pool Type | Get+Return | Throughput |
|-----------|------------|------------|
| Legacy PacketPool | 31.62 ns | **31.6M ops/sec** |
| ThreadLocalFastPool | 43.84 ns | **22.8M ops/sec** |

*ThreadLocalFastPool is slightly slower in microbenchmarks but eliminates contention under concurrent load.*

### Pool Comparison (10x cycles)

| Pool Type | Latency | Throughput |
|-----------|---------|------------|
| Legacy 10x | 287.06 ns | **3.48M ops/sec** |
| ThreadLocal 10x | 507.40 ns | **1.97M ops/sec** |

## Packet Build

| Events/Packet | Latency | Throughput |
|--------------:|---------|------------|
| 1 | 1.22 us | **49.9 MiB/s** |
| 10 | 2.95 us | **207 MiB/s** |
| 50 | 11.14 us | **274 MiB/s** |

## Encryption

### Cipher Comparison (Fast vs Legacy)

| Payload Size | Legacy (XChaCha20) | Fast (ChaCha20) | Improvement |
|-------------:|-------------------:|----------------:|------------:|
| 64B | 1,207 ns | 709 ns | **41% faster** |
| 256B | 1,790 ns | 1,284 ns | **28% faster** |
| 1KB | 4,002 ns | 3,486 ns | **13% faster** |
| 4KB | 12,980 ns | 12,440 ns | **4% faster** |

*Fast mode uses ChaCha20-Poly1305 with counter-based nonces (default). Legacy uses XChaCha20-Poly1305 with random nonces.*

### End-to-End Packet Build (50 events)

| Path | Latency | Improvement |
|------|---------|-------------|
| Legacy | 10,877 ns | baseline |
| Fast | 10,443 ns | **4.0% faster** |

## Adaptive Batcher Overhead

| Operation | Latency |
|-----------|---------|
| optimal_size() | 0.97 ns |
| record() | 2.63 ns |
| full_cycle | 2.81 ns |

*Negligible overhead (~3ns) for adaptive batch sizing decisions.*

## Key Generation

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Keypair generate | 33.26 us | **30.1K ops/sec** |

---

## Multi-threaded Packet Build (1000 packets/thread)

| Threads | Legacy Pool | Fast Pool | Speedup |
|--------:|------------:|----------:|--------:|
| 8 | 2.33 M/s | **8.68 M/s** | **3.7x** |
| 16 | 1.82 M/s | **9.28 M/s** | **5.1x** |
| 24 | 1.78 M/s | **9.66 M/s** | **5.4x** |
| 32 | 1.78 M/s | **9.90 M/s** | **5.6x** |

*Thread-local pool eliminates contention, enabling near-linear scaling.*

### Pool Contention (10,000 acquire/release per thread)

| Threads | Legacy Pool | Fast Pool | Speedup |
|--------:|------------:|----------:|--------:|
| 8 | 4.34 M/s | **117.1 M/s** | **27x** |
| 16 | 4.34 M/s | **125.1 M/s** | **29x** |
| 24 | 4.35 M/s | **130.8 M/s** | **30x** |
| 32 | 4.17 M/s | **135.5 M/s** | **32x** |

*Pure acquire/release stress test. Thread-local caching eliminates lock contention.*

### Mixed Frame Sizes (64B/256B/1KB rotation)

| Threads | Legacy Pool | Fast Pool | Speedup |
|--------:|------------:|----------:|--------:|
| 8 | 5.60 M/s | **9.91 M/s** | **1.8x** |
| 16 | 5.02 M/s | **10.28 M/s** | **2.0x** |
| 24 | 4.92 M/s | **10.50 M/s** | **2.1x** |
| 32 | 4.98 M/s | **10.62 M/s** | **2.1x** |

*Realistic traffic simulation. Encryption cost dominates at larger frame sizes.*

### Throughput Scaling (Fast Pool)

| Threads | Throughput | Scaling |
|--------:|-----------:|--------:|
| 1 | 234 K/s | 1.0x |
| 2 | 453 K/s | 1.9x |
| 4 | 875 K/s | 3.7x |
| 8 | 1.65 M/s | 7.0x |
| 16 | 1.71 M/s | 7.3x |
| 24 | 1.74 M/s | 7.4x |
| 32 | 1.78 M/s | 7.6x |

*Linear scaling up to ~8 threads, then CPU-bound plateau.*

---

## Routing

### Routing Header

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Serialize | 0.57 ns | **1.74G ops/sec** |
| Deserialize | 0.94 ns | **1.07G ops/sec** |
| Roundtrip | 0.94 ns | **1.07G ops/sec** |
| Forward | 0.57 ns | **1.75G ops/sec** |

### Routing Table

| Operation | Latency | Throughput |
|-----------|---------|------------|
| lookup_hit | 14.92 ns | **67.0M ops/sec** |
| lookup_miss | 11.96 ns | **83.6M ops/sec** |
| is_local | 0.31 ns | **3.20G ops/sec** |
| add_route | 291.15 ns | **3.43M ops/sec** |
| record_in | 50.51 ns | **19.8M ops/sec** |
| record_out | 18.03 ns | **55.5M ops/sec** |
| aggregate_stats | 2.11 us | **474.9K ops/sec** |

### Concurrent Routing Lookup

| Threads | Lookup Throughput | Stats Throughput |
|--------:|------------------:|-----------------:|
| 4 | 47.2 M/s | 14.0 M/s |
| 8 | 56.3 M/s | 20.0 M/s |
| 16 | 51.5 M/s | 19.7 M/s |

### Decision Pipeline

| Operation | Latency | Throughput |
|-----------|---------|------------|
| parse + lookup + forward | 15.34 ns | **65.2M ops/sec** |
| full with stats | 73.51 ns | **13.6M ops/sec** |

---

## Multi-hop Forwarding

### Packet Builder

| Payload Size | Build | Build (Priority) | Throughput (Build) |
|-------------:|------:|-----------------:|-------------------:|
| 64B | 24.69 ns | 26.37 ns | **2.41 GiB/s** |
| 256B | 56.54 ns | 54.10 ns | **4.22 GiB/s** |
| 1KB | 41.52 ns | 41.43 ns | **22.97 GiB/s** |
| 4KB | 83.66 ns | 82.99 ns | **45.60 GiB/s** |

### Chain Scaling (forward_chain)

| Hops | Latency | Throughput |
|-----:|---------|------------|
| 1 | 57.99 ns | **17.2M ops/sec** |
| 2 | 113.61 ns | **8.80M ops/sec** |
| 3 | 159.94 ns | **6.25M ops/sec** |
| 4 | 217.30 ns | **4.60M ops/sec** |
| 5 | 276.92 ns | **3.61M ops/sec** |

### Hop Latency

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Single hop process | 1.29 ns | **776M ops/sec** |
| Single hop full | 53.70 ns | **18.6M ops/sec** |

### Hop Scaling by Payload Size

| Payload | 1 hop | 2 hops | 3 hops | 4 hops | 5 hops |
|--------:|------:|-------:|-------:|-------:|-------:|
| 64B | 28.87 ns | 50.55 ns | 73.24 ns | 95.84 ns | 126.26 ns |
| 256B | 57.85 ns | 112.62 ns | 161.15 ns | 215.89 ns | 277.80 ns |
| 1KB | 46.65 ns | 110.43 ns | 154.69 ns | 211.17 ns | 250.01 ns |

### Route and Forward (with routing table lookup)

| Hops | Latency | Throughput |
|-----:|---------|------------|
| 1 | 145.18 ns | **6.89M ops/sec** |
| 2 | 282.62 ns | **3.54M ops/sec** |
| 3 | 424.51 ns | **2.36M ops/sec** |
| 4 | 642.74 ns | **1.56M ops/sec** |
| 5 | 1.09 us | **918K ops/sec** |

### Concurrent Forwarding

| Threads | Throughput |
|--------:|-----------:|
| 4 | 4.67 M/s |
| 8 | 7.48 M/s |
| 16 | 9.70 M/s |

---

## Swarm / Discovery

### Pingwave

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Serialize | 0.80 ns | **1.25G ops/sec** |
| Deserialize | 0.94 ns | **1.07G ops/sec** |
| Roundtrip | 0.94 ns | **1.06G ops/sec** |
| Forward | 0.63 ns | **1.59G ops/sec** |

### Local Graph

| Operation | Latency | Throughput |
|-----------|---------|------------|
| create_pingwave | 2.11 ns | **473.7M ops/sec** |
| on_pingwave_new | 126.60 ns | **7.90M ops/sec** |
| on_pingwave_duplicate | 23.72 ns | **42.2M ops/sec** |
| get_node | 26.72 ns | **37.4M ops/sec** |
| node_count | 200.14 ns | **5.00M ops/sec** |
| stats | 602.22 ns | **1.66M ops/sec** |

### Graph Scaling

| Nodes | all_nodes | nodes_within_hops |
|------:|----------:|------------------:|
| 100 | 2.30 us (43.5M/s) | 2.76 us (36.3M/s) |
| 500 | 7.35 us (68.1M/s) | 9.30 us (53.8M/s) |
| 1,000 | 120.31 us (8.31M/s) | 122.69 us (8.15M/s) |
| 5,000 | 173.43 us (28.8M/s) | 192.91 us (25.9M/s) |

### Path Finding

| Scenario | Latency | Throughput |
|----------|---------|------------|
| 1 hop | 1.52 us | **655.9K ops/sec** |
| 2 hops | 1.61 us | **622.3K ops/sec** |
| 4 hops | 1.85 us | **541.2K ops/sec** |
| Not found | 1.84 us | **542.1K ops/sec** |
| Complex graph | 306.69 us | **3.26K ops/sec** |

### Concurrent Pingwave Processing

| Threads | Throughput |
|--------:|-----------:|
| 4 | 15.7 M/s |
| 8 | 22.0 M/s |
| 16 | 24.7 M/s |

---

## Failure Detection

### Failure Detector

| Operation | Latency | Throughput |
|-----------|---------|------------|
| heartbeat_existing | 29.22 ns | **34.2M ops/sec** |
| heartbeat_new | 270.88 ns | **3.69M ops/sec** |
| status_check | 12.22 ns | **81.9M ops/sec** |
| check_all | 283.85 ms | **3.52 ops/sec** |
| stats | 81.07 ms | **12.3 ops/sec** |

### Circuit Breaker

| Operation | Latency | Throughput |
|-----------|---------|------------|
| allow (closed) | 13.73 ns | **72.8M ops/sec** |
| record_success | 13.51 ns | **74.0M ops/sec** |
| record_failure | 13.50 ns | **74.1M ops/sec** |
| state | 13.58 ns | **73.6M ops/sec** |

### Recovery Manager

| Operation | Latency | Throughput |
|-----------|---------|------------|
| on_failure (with alternates) | 278.80 ns | **3.59M ops/sec** |
| on_failure (no alternates) | 304.75 ns | **3.28M ops/sec** |
| get_action | 36.79 ns | **27.2M ops/sec** |
| is_failed | 10.66 ns | **93.8M ops/sec** |
| on_recovery | 102.86 ns | **9.72M ops/sec** |
| stats | 0.71 ns | **1.42G ops/sec** |

### Full Recovery Cycle

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Fail + recover cycle | 312.53 ns | **3.20M ops/sec** |

### Loss Simulator

| Drop Rate | Latency | Throughput |
|----------:|---------|------------|
| 1% | 2.09 ns | **477M ops/sec** |
| 5% | 2.14 ns | **467M ops/sec** |
| 10% | 2.33 ns | **430M ops/sec** |
| 20% | 3.11 ns | **322M ops/sec** |
| Burst | 2.56 ns | **390M ops/sec** |

### Failure Scaling (check_all)

| Nodes | check_all | healthy_nodes |
|------:|----------:|--------------:|
| 100 | 4.19 us (23.9M/s) | 1.69 us (59.0M/s) |
| 500 | 17.65 us (28.3M/s) | 5.48 us (91.2M/s) |
| 1,000 | 34.62 us (28.9M/s) | 10.28 us (97.3M/s) |
| 5,000 | 170.41 us (29.3M/s) | 49.43 us (101.2M/s) |

### Concurrent Heartbeats

| Threads | Throughput |
|--------:|-----------:|
| 4 | 10.7 M/s |
| 8 | 14.8 M/s |
| 16 | 16.9 M/s |

---

## Stream Multiplexing

| Streams | Lookup All | Stats All |
|--------:|-----------:|----------:|
| 10 | 120.73 ns (82.8M/s) | 494.09 ns (20.2M/s) |
| 100 | 1.21 us (82.9M/s) | 5.03 us (19.9M/s) |
| 1,000 | 12.36 us (80.9M/s) | 52.52 us (19.0M/s) |
| 10,000 | 132.98 us (75.2M/s) | 566.17 us (17.7M/s) |

*Lookup throughput scales linearly. Per-stream overhead is constant at ~12ns.*

---

## Fair Scheduler

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Creation | 287.17 ns | **3.48M ops/sec** |
| Stream count (empty) | 200.36 ns | **4.99M ops/sec** |
| Total queued | 0.31 ns | **3.18G ops/sec** |
| Cleanup (empty) | 200.83 ns | **4.98M ops/sec** |

---

## Capability System

### CapabilitySet

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Create | 514.60 ns | **1.94M ops/sec** |
| Serialize | 925.39 ns | **1.08M ops/sec** |
| Deserialize | 1.75 us | **570.6K ops/sec** |
| Roundtrip | 2.70 us | **371.0K ops/sec** |
| has_tag | 0.76 ns | **1.31G ops/sec** |
| has_model | 0.95 ns | **1.05G ops/sec** |
| has_tool | 0.76 ns | **1.31G ops/sec** |
| has_gpu | 0.32 ns | **3.13G ops/sec** |

### Capability Announcement

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Create | 388.80 ns | **2.57M ops/sec** |
| Serialize | 1.09 us | **919K ops/sec** |
| Deserialize | 1.92 us | **521K ops/sec** |
| is_expired | 26.00 ns | **38.5M ops/sec** |

### Capability Serialization (Simple vs Complex)

| Type | Serialize | Deserialize |
|------|-----------|-------------|
| Simple | 19.35 ns (51.7M/s) | 4.83 ns (206.9M/s) |
| Complex | 40.08 ns (24.9M/s) | 156.70 ns (6.38M/s) |

### Capability Filter

| Filter Type | Latency | Throughput |
|-------------|---------|------------|
| Single tag | 10.02 ns | **99.8M ops/sec** |
| Require GPU | 4.07 ns | **245.5M ops/sec** |
| GPU vendor | 3.78 ns | **264.3M ops/sec** |
| Min memory | 3.77 ns | **265.4M ops/sec** |
| Complex | 10.34 ns | **96.7M ops/sec** |
| No match | 3.15 ns | **317.8M ops/sec** |

### Capability Index (Insert)

| Nodes | Latency | Throughput |
|------:|---------|------------|
| 100 | 124.67 us | **802K ops/sec** |
| 1,000 | 1.23 ms | **810K ops/sec** |
| 10,000 | 19.88 ms | **503K ops/sec** |

### Capability Index (Query, 10,000 nodes)

| Query Type | Latency | Throughput |
|------------|---------|------------|
| Single tag | 170.20 us | **5.88K ops/sec** |
| Require GPU | 224.68 us | **4.45K ops/sec** |
| GPU vendor | 720.70 us | **1.39K ops/sec** |
| Min memory | 730.71 us | **1.37K ops/sec** |
| Complex | 478.26 us | **2.09K ops/sec** |
| Model | 100.18 us | **9.98K ops/sec** |
| Tool | 642.03 us | **1.56K ops/sec** |
| No results | 22.85 ns | **43.8M ops/sec** |

### Capability Index (Find Best)

| Query Type | Latency | Throughput |
|------------|---------|------------|
| Simple | 369.49 us | **2.71K ops/sec** |
| With preferences | 705.65 us | **1.42K ops/sec** |

### Capability Search

| Query | Latency | Throughput |
|-------|---------|------------|
| find_with_gpu | 17.94 us | **55.8K ops/sec** |
| find_by_tool (Python) | 31.40 us | **31.8K ops/sec** |
| find_by_tool (Rust) | 40.42 us | **24.7K ops/sec** |

### Capability Index Scaling

| Nodes | Tag Query | Complex Query |
|------:|----------:|--------------:|
| 1,000 | 12.69 us (78.8K/s) | 41.08 us (24.3K/s) |
| 5,000 | 72.20 us (13.9K/s) | 211.51 us (4.73K/s) |
| 10,000 | 170.05 us (5.88K/s) | 512.64 us (1.95K/s) |
| 50,000 | 2.58 ms (387/s) | 4.57 ms (219/s) |

### Concurrent Capability Index

| Threads | Insert Throughput |
|--------:|------------------:|
| 4 | 5.04 M/s |

---

## SDK Benchmarks

### Go

| Benchmark | Throughput | Latency | Allocations |
|-----------|------------|---------|-------------|
| IngestRaw (small 9B) | **2.65M/sec** | 377 ns | 0 allocs |
| IngestRaw (medium 112B) | **2.27M/sec** | 441 ns | 0 allocs |
| IngestRaw (large 554B) | **1.63M/sec** | 615 ns | 0 allocs |
| Ingest (struct) | 633K/sec | 1.58 us | 16 allocs |
| Batch (1000) | 2.44M/sec | 411 ns/event | 2 allocs |
| Parallel (8 goroutines) | 885K/sec | 1.13 us | 0 allocs |

### Python

| Benchmark | Throughput | Latency |
|-----------|------------|---------|
| ingest_raw (small 9B) | **2.53M/sec** | 0.40 us |
| ingest_raw (medium 122B) | **2.16M/sec** | 0.46 us |
| ingest_raw (large 591B) | **1.67M/sec** | 0.60 us |
| ingest (dict) | 237K/sec | 4.2 us |
| Batch ingestion (1000) | **2.78M/sec** | 0.36 us |

### TypeScript/Node.js

| Benchmark | Throughput | Latency |
|-----------|------------|---------|
| pushBatch(Buffer[]) | **2.89M/sec** | 0.35 us |
| push(Buffer) | **2.26M/sec** | 0.44 us |
| pushWithHash(Buffer, hash) | **2.19M/sec** | 0.46 us |
| ingestFire (fire & forget) | **2.18M/sec** | 0.46 us |
| ingestRawBatchSync (1000) | **2.66M/sec** | 0.38 us |
| ingestRawSync (single) | **1.03M/sec** | 0.97 us |
| ingestRaw (async single) | 73K/sec | 13.7 us |

**Buffer-based `push()` is 31x faster than async for single-event ingestion.**

Uses lock-free `ArcSwap` for zero-contention access to the event bus.

### Bun Runtime

Bun provides even higher throughput than Node.js:

| Benchmark | Throughput | Latency |
|-----------|------------|---------|
| ingestBatchFire (1000) | **3.37M/sec** | 0.30 us |
| ingestRawBatchSync (1000) | **3.24M/sec** | 0.31 us |
| pushBatch(Buffer[]) | **2.63M/sec** | 0.38 us |
| push(Buffer) | **2.05M/sec** | 0.49 us |
| ingestFire | **2.03M/sec** | 0.49 us |
| ingestRawSync | **1.13M/sec** | 0.88 us |
| ingestRaw (async) | 43K/sec | 23.4 us |

**Bun batch ingestion is ~17% faster than Node.js.**

## Key Insights

1. **All SDKs achieve 2M+ events/sec** with optimal ingestion patterns
2. **Go has zero allocations** for raw string ingestion - ideal for high-throughput scenarios
3. **Raw string ingestion is 3-10x faster** than object/struct serialization across all SDKs
4. **Node.js: Use sync methods** - `ingestRawSync()` is 14x faster than `await ingestRaw()`
5. **Thread-local pool scales to 32x contention advantage** over legacy at 32 threads
6. **Routing header operations run at 1.7G ops/sec** - sub-nanosecond serialization
7. **Circuit breaker checks are ~13.5ns** - negligible overhead per packet
8. **Capability filters run at 100-300M ops/sec** - fast enough for inline packet decisions

## Shard Count Scaling

| Shards | Go | Python | Node.js |
|--------|-----|--------|---------|
| 1 | 2.10M/sec | 2.51M/sec | 2.49M/sec |
| 2 | 2.05M/sec | 2.47M/sec | 2.42M/sec |
| 4 | 2.09M/sec | 2.69M/sec | 2.46M/sec |
| 8 | 1.70M/sec | 1.76M/sec | 2.04M/sec |
| 16 | 1.44M/sec | 1.35M/sec | 1.70M/sec |

## Running Benchmarks

### BLTP
```bash
cargo bench --features bltp --bench bltp
```

### Go
```bash
cd bindings/go/blackstream
go test -bench=. -benchmem .
```

### Python
```bash
cd bindings/python
uv run python tests/benchmark.py
```

### TypeScript/Node.js
```bash
cd bindings/node
npx tsx test/benchmark.ts
```

### Bun
```bash
cd bindings/node
bun run test/benchmark.bun.ts
```

## Benchmark Files

- BLTP: `benches/bltp.rs`
- Go: `bindings/go/blackstream/benchmark_test.go`
- Python: `bindings/python/tests/benchmark.py`
- TypeScript: `bindings/node/test/benchmark.ts`
- Bun: `bindings/node/test/benchmark.bun.ts`
