/**
 * @ai2070/net-sdk — Ergonomic TypeScript SDK for the Net mesh network.
 *
 * @example
 * ```typescript
 * import { NetNode } from '@ai2070/net-sdk';
 *
 * const node = await NetNode.create({ shards: 4 });
 *
 * // Emit events
 * node.emit({ token: 'hello', index: 0 });
 * node.emitRaw('{"token": "world"}');
 *
 * // Subscribe to a stream
 * for await (const event of node.subscribe()) {
 *   console.log(event.raw);
 * }
 *
 * // Typed channels
 * const temps = node.channel<{ celsius: number }>('sensors/temperature');
 * temps.publish({ celsius: 22.5 });
 *
 * await node.shutdown();
 * ```
 *
 * @packageDocumentation
 */

// Main handle.
export { NetNode } from './node';

// Streaming.
export { EventStream, TypedEventStream } from './stream';

// Typed channels.
export { TypedChannel } from './channel';

// Mesh + streams.
export { MeshNode, BackpressureError, NotConnectedError } from './mesh';
export type {
  MeshNodeConfig,
  MeshStream,
  StreamConfig,
  StreamStats,
  Reliability,
} from './mesh';

// Types.
export type {
  NetNodeConfig,
  Transport,
  Receipt,
  PollRequest,
  PollResponseData,
  Stats,
  SubscribeOpts,
  StoredEvent,
} from './types';
