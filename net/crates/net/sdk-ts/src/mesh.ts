/**
 * MeshNode — the multi-peer encrypted mesh handle.
 *
 * Wraps the NAPI `NetMesh` with ergonomic TypeScript APIs: typed
 * `StreamConfig`, classed `BackpressureError` / `NotConnectedError`
 * for `instanceof`-based pattern matching, and the `send_with_retry`
 * / `send_blocking` helpers from the Rust core.
 *
 * @example
 * ```typescript
 * import { MeshNode, BackpressureError, Reliability } from '@ai2070/net-sdk';
 *
 * const node = await MeshNode.create({
 *   bindAddr: '127.0.0.1:9000',
 *   psk: '0'.repeat(64),
 * });
 *
 * await node.connect('127.0.0.1:9001', peerPubkey, 0x2222n);
 * node.start();
 *
 * const stream = node.openStream(0x2222n, {
 *   streamId: 7n,
 *   reliability: 'reliable',
 *   windowBytes: 256,
 * });
 *
 * try {
 *   await node.sendOnStream(stream, [Buffer.from('hello')]);
 * } catch (e) {
 *   if (e instanceof BackpressureError) {
 *     // daemon chose: drop, buffer, or retry
 *   } else {
 *     throw e;
 *   }
 * }
 * ```
 */

import { NetMesh as NapiNetMesh } from '@ai2070/net';

/** Reliability mode chosen at stream-open time. */
export type Reliability = 'fire_and_forget' | 'reliable';

/** Per-stream configuration for {@link MeshNode.openStream}. */
export interface StreamConfig {
  /**
   * Caller-chosen stream identifier. Opaque `bigint` at the transport
   * layer; no value range has reserved meaning.
   */
  streamId: bigint;
  /** Reliability mode. Default: `'fire_and_forget'`. */
  reliability?: Reliability;
  /**
   * Initial send-credit window in bytes. Leave unset to inherit the
   * core's `DEFAULT_STREAM_WINDOW_BYTES` (64 KB) — v2 backpressure
   * is ON out of the box. Pass `0` to restore the v1 unbounded-queue
   * behavior on this stream.
   */
  windowBytes?: number;
  /**
   * Fair-scheduler weight. `1` = equal share; higher = proportionally
   * more packets per round. Default: `1`.
   */
  fairnessWeight?: number;
}

/** Per-stream stats snapshot. */
export interface StreamStats {
  txSeq: bigint;
  rxSeq: bigint;
  inboundPending: bigint;
  lastActivityNs: bigint;
  active: boolean;
  /** Cumulative Backpressure rejections since stream opened. */
  backpressureEvents: bigint;
  /**
   * Bytes of send credit still available. `0` means the next send
   * will be rejected as Backpressure. Receiver-driven `StreamWindow`
   * grants replenish this counter.
   */
  txCreditRemaining: number;
  /**
   * Configured initial credit window in bytes. `0` disables
   * backpressure entirely on this stream (escape hatch).
   */
  txWindow: number;
  /** Cumulative StreamWindow grants received from the peer. */
  creditGrantsReceived: bigint;
  /** Cumulative StreamWindow grants emitted to the peer. */
  creditGrantsSent: bigint;
}

/**
 * Thrown by {@link MeshNode.sendOnStream} / `sendWithRetry` /
 * `sendBlocking` when the stream's per-stream in-flight window is
 * full. **The event was NOT sent.** Caller decides whether to drop,
 * retry, or buffer at the app layer — see the "Back-pressure" section
 * in `docs/TRANSPORT.md` for the three canonical patterns.
 */
export class BackpressureError extends Error {
  constructor(detail?: string) {
    super(detail ?? 'stream would block (queue full)');
    this.name = 'BackpressureError';
    Object.setPrototypeOf(this, BackpressureError.prototype);
  }
}

/**
 * Thrown when the stream's peer session is gone (peer never
 * connected, disconnected, or the stream was closed). Distinct from
 * {@link BackpressureError} because this is a "connection lost", not
 * "too fast".
 */
export class NotConnectedError extends Error {
  constructor(detail?: string) {
    super(detail ?? 'stream not connected');
    this.name = 'NotConnectedError';
    Object.setPrototypeOf(this, NotConnectedError.prototype);
  }
}

/**
 * Translate a napi-thrown error into one of the typed stream error
 * classes if it matches the stable prefix contract from the binding.
 * Anything else is passed through unchanged.
 */
function toStreamError(e: unknown): never {
  const msg = (e as Error | undefined)?.message ?? '';
  // Prefixes are part of the binding's stable contract; see
  // `bindings/node/src/lib.rs` (`ERR_BACKPRESSURE_PREFIX` /
  // `ERR_NOT_CONNECTED_PREFIX`).
  if (msg.startsWith('stream would block')) {
    throw new BackpressureError(msg);
  }
  if (msg.startsWith('stream not connected')) {
    throw new NotConnectedError(msg);
  }
  throw e;
}

/**
 * Convert a `bigint` to a `number` with an explicit safe-integer range
 * check. The napi layer accepts `i64`, but JS `number` is IEEE-754
 * double precision — any value outside `Number.MAX_SAFE_INTEGER` loses
 * precision silently. We'd rather fail loudly than corrupt a node or
 * stream id on the way into the binding.
 */
function toSafeNumber(label: string, value: bigint): number {
  if (value < 0n) {
    throw new RangeError(`${label} must be non-negative; got ${value}`);
  }
  if (value > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new RangeError(
      `${label} ${value} exceeds Number.MAX_SAFE_INTEGER — JS has no lossless u64`,
    );
  }
  return Number(value);
}

/**
 * Convert a `number` coming back from the napi layer to a `bigint`
 * after a safe-integer range check. Mirror of {@link toSafeNumber}.
 */
function fromSafeNumber(label: string, value: number): bigint {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new RangeError(
      `${label} ${value} is outside the JS safe integer range (${Number.MAX_SAFE_INTEGER})`,
    );
  }
  return BigInt(value);
}

/** Options for {@link MeshNode.create}. */
export interface MeshNodeConfig {
  /** Local bind address (e.g. `"127.0.0.1:9000"`). */
  bindAddr: string;
  /** Hex-encoded 32-byte pre-shared key (64 hex chars). */
  psk: string;
  /** Heartbeat interval in milliseconds. Default: 5000. */
  heartbeatIntervalMs?: number;
  /** Session timeout in milliseconds. Default: 30000. */
  sessionTimeoutMs?: number;
  /** Inbound shard count. Default: 4. */
  numShards?: number;
}

/**
 * An opaque stream handle. Pass back to `sendOnStream` /
 * `sendWithRetry` / `sendBlocking` / `closeStream`. You normally
 * don't need to read the fields — they're exposed for diagnostics.
 */
export interface MeshStream {
  readonly peerNodeId: bigint;
  readonly streamId: bigint;
  /** @internal napi-backed native handle. */
  readonly _native: unknown;
}

/**
 * A node on the Net mesh with full stream multiplexing + backpressure
 * support.
 */
export class MeshNode {
  private native: NapiNetMesh;

  private constructor(native: NapiNetMesh) {
    this.native = native;
  }

  /** Create and configure a new mesh node. */
  static async create(config: MeshNodeConfig): Promise<MeshNode> {
    const native = await NapiNetMesh.create({
      bindAddr: config.bindAddr,
      psk: config.psk,
      heartbeatIntervalMs: config.heartbeatIntervalMs,
      sessionTimeoutMs: config.sessionTimeoutMs,
      numShards: config.numShards,
    });
    return new MeshNode(native);
  }

  /** Hex-encoded Noise static public key. */
  publicKey(): string {
    return this.native.publicKey();
  }

  /** This node's id. */
  nodeId(): bigint {
    return fromSafeNumber('nodeId', this.native.nodeId());
  }

  /** Connect to a peer as initiator. */
  async connect(peerAddr: string, peerPublicKey: string, peerNodeId: bigint): Promise<void> {
    await this.native.connect(peerAddr, peerPublicKey, toSafeNumber('peerNodeId', peerNodeId));
  }

  /** Accept an incoming connection as responder. Returns the peer's wire address. */
  async accept(peerNodeId: bigint): Promise<string> {
    return await this.native.accept(toSafeNumber('peerNodeId', peerNodeId));
  }

  /** Start the receive loop / heartbeats / router. */
  start(): void {
    this.native.start();
  }

  /** Number of connected peers. */
  peerCount(): number {
    return this.native.peerCount();
  }

  // ─── Stream API ──────────────────────────────────────────────────

  /**
   * Open (or look up) a logical stream to a connected peer. Repeated
   * calls for the same `(peer, streamId)` are idempotent; the first
   * open wins and later differing configs are logged and ignored.
   */
  openStream(peerNodeId: bigint, config: StreamConfig): MeshStream {
    const native = this.native.openStream(toSafeNumber('peerNodeId', peerNodeId), {
      streamId: toSafeNumber('streamId', config.streamId),
      reliability: config.reliability,
      windowBytes: config.windowBytes,
      fairnessWeight: config.fairnessWeight,
    });
    return {
      peerNodeId,
      streamId: config.streamId,
      _native: native,
    };
  }

  /** Close a stream. Idempotent. */
  closeStream(peerNodeId: bigint, streamId: bigint): void {
    this.native.closeStream(
      toSafeNumber('peerNodeId', peerNodeId),
      toSafeNumber('streamId', streamId),
    );
  }

  /**
   * Send a batch of events on an explicit stream. Throws
   * {@link BackpressureError} when the stream's in-flight window is
   * full (no events sent — caller decides what to do),
   * {@link NotConnectedError} when the peer session is gone, or a
   * plain `Error` for underlying transport failures.
   */
  async sendOnStream(stream: MeshStream, events: Buffer[]): Promise<void> {
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      await this.native.sendOnStream(stream._native as any, events);
    } catch (e) {
      toStreamError(e);
    }
  }

  /**
   * Send events, retrying on {@link BackpressureError} with 5 ms → 200 ms
   * exponential backoff up to `maxRetries` times. Transport errors and
   * `NotConnectedError` are re-thrown immediately (they're not a
   * pressure signal).
   */
  async sendWithRetry(
    stream: MeshStream,
    events: Buffer[],
    maxRetries = 8,
  ): Promise<void> {
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      await this.native.sendWithRetry(stream._native as any, events, maxRetries);
    } catch (e) {
      toStreamError(e);
    }
  }

  /**
   * Block the calling task until the send succeeds or a transport
   * error occurs. Retries {@link BackpressureError} with 5 ms → 200 ms
   * exponential backoff up to 4096 times (~13 min worst case) —
   * effectively "block until the network lets up" under practical
   * workloads, but with a hard upper bound so runaway pressure can't
   * hang the caller forever. Use {@link sendWithRetry} for a tighter
   * bound.
   */
  async sendBlocking(stream: MeshStream, events: Buffer[]): Promise<void> {
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      await this.native.sendBlocking(stream._native as any, events);
    } catch (e) {
      toStreamError(e);
    }
  }

  /** Snapshot per-stream stats. `null` if the peer or stream isn't open. */
  streamStats(peerNodeId: bigint, streamId: bigint): StreamStats | null {
    const raw = this.native.streamStats(
      toSafeNumber('peerNodeId', peerNodeId),
      toSafeNumber('streamId', streamId),
    );
    if (!raw) return null;
    // The napi binding marshals u64 fields as `BigInt` so values that
    // exceed `Number.MAX_SAFE_INTEGER` — especially `lastActivityNs`,
    // Unix-epoch nanoseconds always above 2^53 — survive the boundary
    // without a precision trap. The u32 fields stay as regular numbers.
    return {
      txSeq: raw.txSeq,
      rxSeq: raw.rxSeq,
      inboundPending: raw.inboundPending,
      lastActivityNs: raw.lastActivityNs,
      active: raw.active,
      backpressureEvents: raw.backpressureEvents,
      txCreditRemaining: raw.txCreditRemaining,
      txWindow: raw.txWindow,
      creditGrantsReceived: raw.creditGrantsReceived,
      creditGrantsSent: raw.creditGrantsSent,
    };
  }

  // =========================================================
  // Channels (distributed pub/sub)
  // =========================================================

  /**
   * Register a channel on this node. Subscribers who ask to join are
   * validated against `config` before being added to the roster.
   *
   * Mirrors the core `ChannelConfig` field-for-field. v1 omits
   * `publishCaps` / `subscribeCaps` — those land with the security
   * plan's identity surface.
   */
  registerChannel(config: ChannelConfig): void {
    try {
      this.native.registerChannel({
        name: config.name,
        visibility: config.visibility,
        reliable: config.reliable,
        requireToken: config.requireToken,
        priority: config.priority,
        maxRatePps: config.maxRatePps,
      });
    } catch (e) {
      toChannelError(e);
    }
  }

  /**
   * Ask `publisherNodeId` to add this node to `channel`'s subscriber
   * set. Blocks until the publisher's `Ack` arrives or the
   * membership-ack timeout elapses.
   *
   * Throws a {@link ChannelAuthError} or {@link ChannelError} on
   * rejection; network-level failures propagate as plain `Error`.
   */
  async subscribeChannel(publisherNodeId: bigint, channel: string): Promise<void> {
    try {
      await this.native.subscribeChannel(publisherNodeId, channel);
    } catch (e) {
      toChannelError(e);
    }
  }

  /** Mirror of {@link subscribeChannel}. Idempotent on the publisher side. */
  async unsubscribeChannel(publisherNodeId: bigint, channel: string): Promise<void> {
    try {
      await this.native.unsubscribeChannel(publisherNodeId, channel);
    } catch (e) {
      toChannelError(e);
    }
  }

  /**
   * Publish one payload to every subscriber of `channel`. Returns a
   * {@link PublishReport} describing per-peer outcomes.
   */
  async publish(
    channel: string,
    payload: Buffer,
    config?: PublishConfig,
  ): Promise<PublishReport> {
    try {
      const raw = await this.native.publish(channel, payload, {
        reliability: config?.reliability,
        onFailure: config?.onFailure,
        maxInflight: config?.maxInflight,
      });
      return {
        attempted: raw.attempted,
        delivered: raw.delivered,
        errors: raw.errors.map((e: { nodeId: bigint; message: string }) => ({
          nodeId: e.nodeId,
          message: e.message,
        })),
      };
    } catch (e) {
      toChannelError(e);
    }
  }

  /** Shutdown the mesh node. */
  async shutdown(): Promise<void> {
    await this.native.shutdown();
  }
}

// =====================================================
// Channel types and errors
// =====================================================

export type Visibility =
  | 'subnet-local'
  | 'parent-visible'
  | 'exported'
  | 'global';

export type OnFailure = 'best_effort' | 'fail_fast' | 'collect';

/** Channel configuration — mirror of the core `ChannelConfig`. */
export interface ChannelConfig {
  /** Canonical channel name. Crosses the boundary as a string. */
  name: string;
  /** Default: `'global'`. */
  visibility?: Visibility;
  /** Default reliability for streams on this channel. */
  reliable?: boolean;
  /** v1 ships `false`; token enforcement requires the security plan. */
  requireToken?: boolean;
  /** Priority (0 = lowest). */
  priority?: number;
  /** Rate cap in packets per second. */
  maxRatePps?: number;
}

/** Publish-fanout config — mirror of the core `PublishConfig`. */
export interface PublishConfig {
  /** Default: `'fire_and_forget'`. */
  reliability?: Reliability;
  /** Default: `'best_effort'`. */
  onFailure?: OnFailure;
  /** Max concurrent per-peer sends. Default 32. */
  maxInflight?: number;
}

/** Per-peer report returned by {@link MeshNode.publish}. */
export interface PublishReport {
  attempted: number;
  delivered: number;
  errors: Array<{ nodeId: bigint; message: string }>;
}

/**
 * Raised when a channel operation fails for a reason other than
 * auth. The napi binding emits `"channel: ..."` prefixed errors that
 * the SDK classifies into {@link ChannelAuthError} (unauthorized) or
 * this class (everything else).
 */
export class ChannelError extends Error {
  constructor(detail?: string) {
    super(detail ?? 'channel error');
    this.name = 'ChannelError';
    Object.setPrototypeOf(this, ChannelError.prototype);
  }
}

/**
 * Raised when a Subscribe / Unsubscribe request is rejected because
 * the subscriber isn't authorized on the publisher's channel config.
 */
export class ChannelAuthError extends ChannelError {
  constructor(detail?: string) {
    super(detail ?? 'channel: unauthorized');
    this.name = 'ChannelAuthError';
    Object.setPrototypeOf(this, ChannelAuthError.prototype);
  }
}

function toChannelError(e: unknown): never {
  const msg = (e as Error | undefined)?.message ?? '';
  if (msg.startsWith('channel: unauthorized')) {
    throw new ChannelAuthError(msg);
  }
  if (msg.startsWith('channel:')) {
    throw new ChannelError(msg);
  }
  throw e;
}
