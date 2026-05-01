/**
 * Redis Streams consumer-side dedup helper — re-export of the NAPI
 * class from `@ai2070/net`.
 *
 * The Redis adapter writes a stable `dedup_id` field on every XADD
 * entry — see [BUG #57] in the audit doc and the producer-side
 * contract in `sdk-ts/README.md`. The dedup helper filters
 * producer-retry-induced duplicates at consume time. Default
 * capacity is 4096; production callers should size to roughly
 * `events_per_sec * dedup_window_seconds`.
 *
 * CR-3: pre-fix the underlying NAPI class was buildable but the
 * `@ai2070/net-sdk` package never re-exported it, so users following
 * the README's `import { RedisStreamDedup } from '@ai2070/net-sdk'`
 * pattern hit a runtime undefined. This shim closes that gap.
 *
 * @example
 * ```typescript
 * import { RedisStreamDedup } from '@ai2070/net-sdk';
 * import { createClient } from 'redis';
 *
 * // Sizing: ~10k events/sec * 1 min dedup window → ~600,000.
 * const dedup = new RedisStreamDedup(600_000);
 *
 * const r = createClient();
 * await r.connect();
 *
 * let cursor = '0';
 * while (true) {
 *   // After the first page, use the exclusive form `(<id>` so we
 *   // don't re-read the entry the cursor points at.
 *   const start = cursor === '0' ? cursor : `(${cursor}`;
 *   const entries = await r.xRange('net:shard:0', start, '+', { COUNT: 100 });
 *   if (entries.length === 0) break;
 *   for (const entry of entries) {
 *     const dedupId = entry.message.dedup_id;
 *     if (dedupId && dedup.isDuplicate(dedupId)) continue;
 *     await process(entry);
 *     cursor = entry.id;
 *   }
 * }
 * ```
 *
 * @packageDocumentation
 */

export { RedisStreamDedup } from '@ai2070/net';
