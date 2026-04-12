/**
 * Bun-based throughput benchmarks for Blackstream Node.js SDK.
 *
 * Measures pure SDK throughput without hitting Redis or JetStream
 * (uses in-memory noop adapter).
 *
 * Run with: bun run test/benchmark.bun.ts
 */

import { Blackstream } from "../index";

// Type augmentation for sync methods
declare module "../index" {
  interface Blackstream {
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

const WARMUP_EVENTS = 10_000;
const BENCHMARK_EVENTS = 100_000;
const BATCH_SIZE = 1000;

// Sample events
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
const LARGE_EVENT_BUF = Buffer.from(LARGE_EVENT);

function formatNumber(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + "M";
  if (n >= 1_000) return (n / 1_000).toFixed(2) + "K";
  return n.toFixed(2);
}

interface BenchResult {
  name: string;
  opsPerSec: number;
  avgNs: number;
  totalOps: number;
}

function bench(
  name: string,
  fn: () => void,
  ops: number = BENCHMARK_EVENTS
): BenchResult {
  // Warmup
  for (let i = 0; i < Math.min(ops / 10, 10000); i++) {
    fn();
  }

  const start = Bun.nanoseconds();
  for (let i = 0; i < ops; i++) {
    fn();
  }
  const elapsed = Bun.nanoseconds() - start;

  const avgNs = elapsed / ops;
  const opsPerSec = (ops / elapsed) * 1e9;

  return { name, opsPerSec, avgNs, totalOps: ops };
}

function benchBatch(
  name: string,
  fn: () => number,
  totalOps: number = BENCHMARK_EVENTS,
  batchSize: number = BATCH_SIZE
): BenchResult {
  const numBatches = Math.ceil(totalOps / batchSize);

  // Warmup
  for (let i = 0; i < Math.min(numBatches / 10, 100); i++) {
    fn();
  }

  const start = Bun.nanoseconds();
  let ingested = 0;
  for (let i = 0; i < numBatches; i++) {
    ingested += fn();
  }
  const elapsed = Bun.nanoseconds() - start;

  const avgNs = elapsed / ingested;
  const opsPerSec = (ingested / elapsed) * 1e9;

  return { name, opsPerSec, avgNs, totalOps: ingested };
}

function printResult(r: BenchResult, extra?: string): void {
  const ns = r.avgNs < 1000 ? `${r.avgNs.toFixed(1)} ns` : `${(r.avgNs / 1000).toFixed(2)} us`;
  console.log(
    `  ${r.name.padEnd(24)} ${formatNumber(r.opsPerSec).padStart(10)}/sec  ${ns.padStart(12)}  ${extra || ""}`
  );
}

async function main() {
  console.log("=".repeat(60));
  console.log("  Blackstream Bun Benchmarks");
  console.log("  Runtime:", Bun.version);
  console.log("=".repeat(60));
  console.log();

  const bus = await Blackstream.create({
    numShards: 4,
    ringBufferCapacity: 1 << 20,
    backpressureMode: "drop_oldest",
  });

  console.log("--- Single Event Ingestion ---\n");

  // Async (baseline)
  const asyncOps = 10_000; // Fewer ops for slow async
  const asyncStart = Bun.nanoseconds();
  for (let i = 0; i < asyncOps; i++) {
    await bus.ingestRaw(MEDIUM_EVENT);
  }
  const asyncElapsed = Bun.nanoseconds() - asyncStart;
  printResult({
    name: "ingestRaw (async)",
    opsPerSec: (asyncOps / asyncElapsed) * 1e9,
    avgNs: asyncElapsed / asyncOps,
    totalOps: asyncOps,
  });

  // Sync methods
  printResult(bench("ingestRawSync", () => bus.ingestRawSync(MEDIUM_EVENT)));
  printResult(bench("ingestFire", () => bus.ingestFire(MEDIUM_EVENT)));
  printResult(bench("push(Buffer)", () => bus.push(MEDIUM_EVENT_BUF)));

  // Pre-hashed
  const hashed = bus.prehash(MEDIUM_EVENT_BUF);
  printResult(
    bench("pushWithHash", () => bus.pushWithHash(hashed.data, hashed.hash))
  );

  console.log("\n--- Batch Ingestion ---\n");

  const batch = Array(BATCH_SIZE).fill(MEDIUM_EVENT);
  const batchBuf = Array(BATCH_SIZE).fill(MEDIUM_EVENT_BUF);

  printResult(
    benchBatch("ingestRawBatchSync", () => bus.ingestRawBatchSync(batch))
  );
  printResult(
    benchBatch("ingestBatchFire", () => bus.ingestBatchFire(batch))
  );
  printResult(
    benchBatch("pushBatch(Buffer[])", () => bus.pushBatch(batchBuf))
  );

  console.log("\n--- Event Size Comparison ---\n");

  const sizes = [
    { name: "small", event: SMALL_EVENT, buf: SMALL_EVENT_BUF },
    { name: "medium", event: MEDIUM_EVENT, buf: MEDIUM_EVENT_BUF },
    { name: "large", event: LARGE_EVENT, buf: LARGE_EVENT_BUF },
  ];

  for (const { name, buf } of sizes) {
    printResult(
      bench(`push (${name} ${buf.length}B)`, () => bus.push(buf)),
      `${buf.length} bytes`
    );
  }

  console.log("\n--- Shard Scaling ---\n");

  for (const numShards of [1, 2, 4, 8]) {
    const testBus = await Blackstream.create({
      numShards,
      ringBufferCapacity: 1 << 20,
      backpressureMode: "drop_oldest",
    });

    const batchBufLocal = Array(BATCH_SIZE).fill(MEDIUM_EVENT_BUF);
    const result = benchBatch(
      `${numShards} shards`,
      () => testBus.pushBatch(batchBufLocal)
    );
    printResult(result);

    await testBus.shutdown();
  }

  console.log("\n" + "=".repeat(60));
  console.log("  Benchmark Complete");
  console.log("=".repeat(60));

  await bus.shutdown();
}

main().catch(console.error);
