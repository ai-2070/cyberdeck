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
export {
  MeshNode,
  BackpressureError,
  NotConnectedError,
  ChannelError,
  ChannelAuthError,
} from './mesh';
export type {
  MeshNodeConfig,
  MeshStream,
  StreamConfig,
  StreamStats,
  Reliability,
  Visibility,
  OnFailure,
  ChannelConfig,
  PublishConfig,
  PublishReport,
} from './mesh';

// CortEX + NetDb (event-sourced state with reactive watches).
export {
  Redex,
  RedexFile,
  NetDb,
  TasksAdapter,
  MemoriesAdapter,
  TaskStatus,
  TasksOrderBy,
  MemoriesOrderBy,
  CortexError,
  NetDbError,
  RedexError,
} from './cortex';
export type {
  RedexOptions,
  RedexFileConfig,
  RedexEvent,
  SnapshotAndWatch,
  Task,
  Memory,
  TaskFilter,
  MemoryFilter,
  NetDbOpenConfig,
  NetDbBundle,
  CortexSnapshot,
} from './cortex';

// Identity + tokens (security surface).
export {
  Identity,
  Token,
  IdentityError,
  TokenError,
  channelHash,
  delegateToken,
} from './identity';
export type { TokenScope, TokenErrorKind, IssueTokenOptions } from './identity';

// Capabilities (announce + find-peers).
export type {
  CapabilitySet,
  CapabilityFilter,
  CapabilityLimits,
  Hardware,
  Software,
  SoftwarePair,
  GpuInfo,
  GpuVendor,
  Accelerator,
  AcceleratorKind,
  ModelCapability,
  ToolCapability,
  Modality,
} from './capabilities';

// Subnets (visibility enforcement).
export { subnetId, GLOBAL_SUBNET } from './subnets';
export type { SubnetId, SubnetRule, SubnetPolicy } from './subnets';

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
