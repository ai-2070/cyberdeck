/**
 * Throughput benchmarks for Net Node.js SDK.
 *
 * Measures pure SDK throughput using in-memory noop adapter.
 *
 * Run with:
 *   npx tsx test/benchmark.ts      (Node.js)
 *   bun run test/benchmark.bun.ts  (Bun - faster)
 */

import { Net, EventBusOptions } from "../index";

interface BenchmarkResult {
  name: string;
  totalEvents: number;
  durationMs: number;
  eventsPerSecond: number;
  avgLatencyUs: number;
}

const WARMUP_EVENTS = 10_000;
const BENCHMARK_EVENTS = 100_000;
const BATCH_SIZE = 1000;

// Type augmentation for new sync methods
declare module "../index" {
  interface Net {
    ingestRawSync(json: string): { shardId: number; timestamp: number };
    ingestRawBatchSync(events: string[]): number;
    ingestFire(json: string): boolean;
    ingestBatchFire(events: string[]): number;
    push(data: Buffer): boolean;
    pushBatch(events: Buffer[]): number;
    prehash(data: Buffer): { data: Buffer; hash: bigint };
    pushWithHash(data: Buffer, hash: bigint): boolean;
  }
}

// Sample events of different sizes
const SMALL_EVENT = '{"t":"x"}';
const SMALL_EVENT_BUF = Buffer.from(SMALL_EVENT);
const MEDIUM_EVENT = JSON.stringify({
  token: "hello_world",
  index: 42,
  session_id: "abc123def456",
  timestamp: 1234567890,
  metadata: { key: "value" },
});
const MEDIUM_EVENT_BUF = Buffer.from(MEDIUM_EVENT);
const LARGE_EVENT = JSON.stringify({
  token: "hello_world_this_is_a_longer_token",
  index: 42,
  session_id: "abc123def456ghi789jkl012mno345",
  timestamp: 1234567890,
  user_id: "user_12345678901234567890",
  request_id: "req_abcdefghijklmnopqrstuvwxyz",
  metadata: {
    key1: "value1_with_some_extra_data",
    key2: "value2_with_more_extra_data",
    key3: "value3_with_even_more_data",
    nested: { a: 1, b: 2, c: 3, d: 4, e: 5 },
  },
  tags: ["tag1", "tag2", "tag3", "tag4", "tag5"],
  content:
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
});

async function createBus(numShards: number = 4): Promise<Net> {
  // No adapter config = uses noop in-memory adapter
  return await Net.create({
    numShards,
    ringBufferCapacity: 1 << 20, // 1M events per shard
    backpressureMode: "drop_oldest",
  });
}

async function warmup(bus: Net, event: string): Promise<void> {
  for (let i = 0; i < WARMUP_EVENTS; i++) {
    bus.ingestRawSync(event);
  }
}

async function benchmarkSingleIngestion(
  bus: Net,
  event: string,
  count: number,
): Promise<BenchmarkResult> {
  const start = performance.now();

  for (let i = 0; i < count; i++) {
    await bus.ingestRaw(event);
  }

  const durationMs = performance.now() - start;
  const eventsPerSecond = (count / durationMs) * 1000;
  const avgLatencyUs = (durationMs / count) * 1000;

  return {
    name: "single_async",
    totalEvents: count,
    durationMs,
    eventsPerSecond,
    avgLatencyUs,
  };
}

function benchmarkSyncIngestion(
  bus: Net,
  event: string,
  count: number,
): BenchmarkResult {
  const start = performance.now();

  for (let i = 0; i < count; i++) {
    bus.ingestRawSync(event);
  }

  const durationMs = performance.now() - start;
  const eventsPerSecond = (count / durationMs) * 1000;
  const avgLatencyUs = (durationMs / count) * 1000;

  return {
    name: "single_sync",
    totalEvents: count,
    durationMs,
    eventsPerSecond,
    avgLatencyUs,
  };
}

function benchmarkFireAndForgetSync(
  bus: Net,
  event: string,
  count: number,
): BenchmarkResult {
  const start = performance.now();

  for (let i = 0; i < count; i++) {
    bus.ingestFire(event);
  }

  const durationMs = performance.now() - start;
  const eventsPerSecond = (count / durationMs) * 1000;
  const avgLatencyUs = (durationMs / count) * 1000;

  return {
    name: "fire_forget_sync",
    totalEvents: count,
    durationMs,
    eventsPerSecond,
    avgLatencyUs,
  };
}

function benchmarkBatchSync(
  bus: Net,
  event: string,
  totalEvents: number,
  batchSize: number,
): BenchmarkResult {
  const batch = Array(batchSize).fill(event);
  const numBatches = Math.ceil(totalEvents / batchSize);

  const start = performance.now();

  let ingested = 0;
  for (let i = 0; i < numBatches; i++) {
    ingested += bus.ingestRawBatchSync(batch);
  }

  const durationMs = performance.now() - start;
  const eventsPerSecond = (ingested / durationMs) * 1000;
  const avgLatencyUs = (durationMs / ingested) * 1000;

  return {
    name: "batch_sync",
    totalEvents: ingested,
    durationMs,
    eventsPerSecond,
    avgLatencyUs,
  };
}

// =========================================================================
// BUFFER-BASED BENCHMARKS (Ultra fast path)
// =========================================================================

function benchmarkPush(
  bus: Net,
  eventBuf: Buffer,
  count: number,
): BenchmarkResult {
  const start = performance.now();

  for (let i = 0; i < count; i++) {
    bus.push(eventBuf);
  }

  const durationMs = performance.now() - start;
  const eventsPerSecond = (count / durationMs) * 1000;
  const avgLatencyUs = (durationMs / count) * 1000;

  return {
    name: "push",
    totalEvents: count,
    durationMs,
    eventsPerSecond,
    avgLatencyUs,
  };
}

function benchmarkPushBatch(
  bus: Net,
  eventBuf: Buffer,
  totalEvents: number,
  batchSize: number,
): BenchmarkResult {
  const batch = Array(batchSize).fill(eventBuf);
  const numBatches = Math.ceil(totalEvents / batchSize);

  const start = performance.now();

  let ingested = 0;
  for (let i = 0; i < numBatches; i++) {
    ingested += bus.pushBatch(batch);
  }

  const durationMs = performance.now() - start;
  const eventsPerSecond = (ingested / durationMs) * 1000;
  const avgLatencyUs = (durationMs / ingested) * 1000;

  return {
    name: "push_batch",
    totalEvents: ingested,
    durationMs,
    eventsPerSecond,
    avgLatencyUs,
  };
}

function benchmarkPushWithHash(
  bus: Net,
  eventBuf: Buffer,
  count: number,
): BenchmarkResult {
  // Pre-compute hash once
  const hashed = bus.prehash(eventBuf);

  const start = performance.now();

  for (let i = 0; i < count; i++) {
    bus.pushWithHash(hashed.data, hashed.hash);
  }

  const durationMs = performance.now() - start;
  const eventsPerSecond = (count / durationMs) * 1000;
  const avgLatencyUs = (durationMs / count) * 1000;

  return {
    name: "push_with_hash",
    totalEvents: count,
    durationMs,
    eventsPerSecond,
    avgLatencyUs,
  };
}

async function benchmarkBatchIngestion(
  bus: Net,
  event: string,
  totalEvents: number,
  batchSize: number,
): Promise<BenchmarkResult> {
  const batch = Array(batchSize).fill(event);
  const numBatches = Math.ceil(totalEvents / batchSize);

  const start = performance.now();

  let ingested = 0;
  for (let i = 0; i < numBatches; i++) {
    ingested += await bus.ingestRawBatch(batch);
  }

  const durationMs = performance.now() - start;
  const eventsPerSecond = (ingested / durationMs) * 1000;
  const avgLatencyUs = (durationMs / ingested) * 1000;

  return {
    name: "batch_ingestion",
    totalEvents: ingested,
    durationMs,
    eventsPerSecond,
    avgLatencyUs,
  };
}

async function benchmarkFireAndForget(
  bus: Net,
  event: string,
  count: number,
): Promise<BenchmarkResult> {
  const start = performance.now();

  // Fire all ingestions without awaiting
  const promises: Promise<any>[] = [];
  for (let i = 0; i < count; i++) {
    promises.push(bus.ingestRaw(event));
  }

  // Wait for all to complete
  await Promise.all(promises);

  const durationMs = performance.now() - start;
  const eventsPerSecond = (count / durationMs) * 1000;
  const avgLatencyUs = (durationMs / count) * 1000;

  return {
    name: "fire_and_forget",
    totalEvents: count,
    durationMs,
    eventsPerSecond,
    avgLatencyUs,
  };
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) {
    return (n / 1_000_000).toFixed(2) + "M";
  }
  if (n >= 1_000) {
    return (n / 1_000).toFixed(2) + "K";
  }
  return n.toFixed(2);
}

function printResult(result: BenchmarkResult, eventSize: string): void {
  console.log(
    `  ${result.name.padEnd(20)} | ${eventSize.padEnd(10)} | ` +
      `${formatNumber(result.eventsPerSecond).padStart(10)} events/sec | ` +
      `${result.avgLatencyUs.toFixed(2).padStart(10)} μs/event | ` +
      `${result.durationMs.toFixed(0).padStart(8)} ms total`,
  );
}

async function runSyncVsAsyncBenchmarks(): Promise<void> {
  console.log("\n=== Sync vs Async Comparison ===\n");

  const bus = await createBus(4);
  await warmup(bus, MEDIUM_EVENT);

  // Async single
  const asyncResult = await benchmarkSingleIngestion(
    bus,
    MEDIUM_EVENT,
    BENCHMARK_EVENTS,
  );
  printResult(asyncResult, "medium");

  // Sync single
  const syncResult = benchmarkSyncIngestion(
    bus,
    MEDIUM_EVENT,
    BENCHMARK_EVENTS,
  );
  printResult(syncResult, "medium");

  // Fire and forget sync
  const fireResult = benchmarkFireAndForgetSync(
    bus,
    MEDIUM_EVENT,
    BENCHMARK_EVENTS,
  );
  printResult(fireResult, "medium");

  // Buffer push (ultra fast)
  const pushResult = benchmarkPush(bus, MEDIUM_EVENT_BUF, BENCHMARK_EVENTS);
  printResult(pushResult, "medium");

  // Buffer push with pre-computed hash (even faster for repeated events)
  const pushHashResult = benchmarkPushWithHash(
    bus,
    MEDIUM_EVENT_BUF,
    BENCHMARK_EVENTS,
  );
  printResult(pushHashResult, "medium");

  // Batch sync
  const batchSyncResult = benchmarkBatchSync(
    bus,
    MEDIUM_EVENT,
    BENCHMARK_EVENTS,
    BATCH_SIZE,
  );
  printResult(batchSyncResult, "medium");

  // Buffer batch push
  const pushBatchResult = benchmarkPushBatch(
    bus,
    MEDIUM_EVENT_BUF,
    BENCHMARK_EVENTS,
    BATCH_SIZE,
  );
  printResult(pushBatchResult, "medium");

  // Batch async
  const batchAsyncResult = await benchmarkBatchIngestion(
    bus,
    MEDIUM_EVENT,
    BENCHMARK_EVENTS,
    BATCH_SIZE,
  );
  printResult(batchAsyncResult, "medium");

  const speedup = pushResult.eventsPerSecond / asyncResult.eventsPerSecond;
  console.log(
    `\n  push() is ${speedup.toFixed(1)}x faster than async single ingestion`,
  );

  await bus.shutdown();
}

async function runEventSizeBenchmarks(): Promise<void> {
  console.log("\n=== Event Size Benchmarks (Sync Ingestion) ===\n");

  const bus = await createBus(4);

  for (const [name, event] of [
    ["small", SMALL_EVENT],
    ["medium", MEDIUM_EVENT],
    ["large", LARGE_EVENT],
  ] as const) {
    await warmup(bus, event);
    const result = benchmarkSyncIngestion(bus, event, BENCHMARK_EVENTS);
    printResult(result, `${name} (${event.length}B)`);
  }

  await bus.shutdown();
}

async function runShardCountBenchmarks(): Promise<void> {
  console.log("\n=== Shard Count Benchmarks (Batch Ingestion) ===\n");

  for (const numShards of [1, 2, 4, 8, 16]) {
    const bus = await createBus(numShards);
    await warmup(bus, MEDIUM_EVENT);

    const result = await benchmarkBatchIngestion(
      bus,
      MEDIUM_EVENT,
      BENCHMARK_EVENTS,
      BATCH_SIZE,
    );
    console.log(
      `  ${numShards} shards`.padEnd(20) +
        ` | ${formatNumber(result.eventsPerSecond).padStart(10)} events/sec | ` +
        `${result.avgLatencyUs.toFixed(2).padStart(10)} μs/event`,
    );

    await bus.shutdown();
  }
}

async function runIngestionPatternBenchmarks(): Promise<void> {
  console.log("\n=== Ingestion Pattern Benchmarks ===\n");

  const bus = await createBus(4);
  await warmup(bus, MEDIUM_EVENT);

  // Single sequential
  const single = await benchmarkSingleIngestion(
    bus,
    MEDIUM_EVENT,
    BENCHMARK_EVENTS,
  );
  printResult(single, "medium");

  // Batch
  const batch = await benchmarkBatchIngestion(
    bus,
    MEDIUM_EVENT,
    BENCHMARK_EVENTS,
    BATCH_SIZE,
  );
  printResult(batch, "medium");

  // Fire and forget (concurrent)
  const fireForget = await benchmarkFireAndForget(
    bus,
    MEDIUM_EVENT,
    BENCHMARK_EVENTS,
  );
  printResult(fireForget, "medium");

  await bus.shutdown();
}

async function runBatchSizeBenchmarks(): Promise<void> {
  console.log("\n=== Batch Size Benchmarks ===\n");

  const bus = await createBus(4);
  await warmup(bus, MEDIUM_EVENT);

  for (const batchSize of [10, 100, 500, 1000, 5000]) {
    const result = await benchmarkBatchIngestion(
      bus,
      MEDIUM_EVENT,
      BENCHMARK_EVENTS,
      batchSize,
    );
    console.log(
      `  batch_size=${batchSize}`.padEnd(20) +
        ` | ${formatNumber(result.eventsPerSecond).padStart(10)} events/sec | ` +
        `${result.avgLatencyUs.toFixed(2).padStart(10)} μs/event`,
    );
  }

  await bus.shutdown();
}

async function main(): Promise<void> {
  console.log("=============================================");
  console.log("   Net Node.js SDK Throughput Benchmarks");
  console.log("   (In-memory adapter)");
  console.log("=============================================");
  console.log(`\nWarmup: ${formatNumber(WARMUP_EVENTS)} events`);
  console.log(`Benchmark: ${formatNumber(BENCHMARK_EVENTS)} events per test\n`);

  try {
    await runSyncVsAsyncBenchmarks();
    await runEventSizeBenchmarks();
    await runShardCountBenchmarks();

    console.log("\n=== Benchmark Complete ===\n");
  } catch (err) {
    console.error("Benchmark failed:", err);
    process.exit(1);
  }
}

main();
