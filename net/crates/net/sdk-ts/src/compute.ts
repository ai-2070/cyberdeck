/**
 * Compute surface — `MeshDaemon` + `DaemonRuntime`.
 *
 * Stage 3 of `SDK_COMPUTE_SURFACE_PLAN.md`. Sub-step 1 lands the
 * skeleton: a caller can build a runtime against an existing
 * {@link MeshNode}, register a factory (stored but not yet
 * invoked), start the runtime, and shut it down. Event delivery,
 * migration, and snapshot/restore land in subsequent sub-steps.
 *
 * @example
 * ```ts
 * import { MeshNode, DaemonRuntime } from '@ai2070/net-sdk';
 *
 * const mesh = await MeshNode.create({ bindAddr: '127.0.0.1:0', psk: '...' });
 * const rt = DaemonRuntime.create(mesh);
 *
 * // Sub-step 1: register a factory shape the TS side can see.
 * // Sub-step 2+ will actually invoke the returned object on
 * // events delivered by Rust.
 * rt.registerFactory('echo', () => ({
 *   name: 'echo',
 *   process: (event) => [event.payload],
 * }));
 *
 * await rt.start();
 * // ... daemons would run here (sub-step 3+) ...
 * await rt.shutdown();
 * ```
 */

import {
  DaemonRuntime as NapiDaemonRuntime,
} from '@ai2070/net';

import { MeshNode } from './mesh.js';

// ----------------------------------------------------------------------------
// Errors — `daemon:` prefix dispatch, mirrors identity/token/cortex pattern.
// ----------------------------------------------------------------------------

/**
 * Base class for daemon-layer errors: factory registration, runtime
 * lifecycle, spawn/stop, migration. The Rust side prefixes every
 * message with `daemon:`; this file peels the prefix and rethrows
 * the typed class so TS callers can `catch (e: DaemonError)`.
 */
export class DaemonError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'DaemonError';
    Object.setPrototypeOf(this, DaemonError.prototype);
  }
}

function toDaemonError(e: unknown): never {
  const msg = (e as Error | undefined)?.message ?? String(e);
  if (msg.startsWith('daemon:')) {
    throw new DaemonError(msg.slice('daemon:'.length).trim());
  }
  throw e;
}

// ----------------------------------------------------------------------------
// MeshDaemon shape — what user factories return.
// ----------------------------------------------------------------------------

/**
 * A causal event delivered to a daemon's `process`. Sub-step 3
 * will plumb this through NAPI; sub-step 1 declares the shape so
 * the factory signature is callable today.
 */
export interface CausalEvent {
  /** 32-bit origin hash of the emitting entity. */
  readonly originHash: number;
  /** Sequence number in the emitter's causal chain. */
  readonly sequence: bigint;
  /** Opaque payload bytes. */
  readonly payload: Buffer;
}

/**
 * User-implemented daemon. The object returned by the factory
 * passed to {@link DaemonRuntime.registerFactory}.
 *
 * `process` is synchronous by contract — do not return a Promise.
 * Snapshot/restore are optional; stateless daemons omit them.
 */
export interface MeshDaemon {
  /** Stable human-readable name. Used only for diagnostics. */
  readonly name: string;
  /**
   * Handle one inbound event. Return zero or more output payloads
   * (buffers); each is wrapped in a fresh causal link by the host.
   *
   * Must be synchronous — the core's `process` contract is sync,
   * and the TSFN bridge in sub-step 3 blocks the calling tokio
   * task until this returns.
   */
  process(event: CausalEvent): Buffer[];
  /** Optional: serialize current state for migration / persistence. */
  snapshot?(): Buffer | null;
  /** Optional: restore state from a snapshot produced by `snapshot`. */
  restore?(state: Buffer): void;
}

/** A zero-arg function returning a {@link MeshDaemon} or a Promise of one. */
export type DaemonFactory = () => MeshDaemon | Promise<MeshDaemon>;

// ----------------------------------------------------------------------------
// DaemonRuntime — thin wrapper over the NAPI class.
// ----------------------------------------------------------------------------

/**
 * Per-mesh compute runtime. Holds the kind-keyed factory table and
 * drives the `Registering → Ready → ShuttingDown` lifecycle.
 *
 * Construct via {@link create}; the runtime shares the given mesh's
 * underlying `MeshNode` (no second socket). Shutting down the
 * runtime does NOT shut down the mesh — the caller owns that.
 */
export class DaemonRuntime {
  private readonly inner: NapiDaemonRuntime;

  private constructor(inner: NapiDaemonRuntime) {
    this.inner = inner;
  }

  /**
   * Build a compute runtime against an existing {@link MeshNode}.
   */
  static create(mesh: MeshNode): DaemonRuntime {
    try {
      return new DaemonRuntime(NapiDaemonRuntime.create(mesh._napiNetMesh()));
    } catch (e) {
      return toDaemonError(e);
    }
  }

  /**
   * Promote to `Ready`. Installs the migration subprotocol handler.
   * Idempotent on an already-ready runtime; rejects on a runtime
   * that has been shut down.
   */
  async start(): Promise<void> {
    try {
      await this.inner.start();
    } catch (e) {
      toDaemonError(e);
    }
  }

  /**
   * Tear down the runtime. Drains daemons, clears factory
   * registrations, uninstalls the migration handler. Idempotent:
   * a second call on an already-shut-down runtime is a no-op.
   */
  async shutdown(): Promise<void> {
    try {
      await this.inner.shutdown();
    } catch (e) {
      toDaemonError(e);
    }
  }

  /**
   * `true` iff the runtime has transitioned to `Ready` and has not
   * yet begun shutting down.
   */
  isReady(): boolean {
    return this.inner.isReady();
  }

  /** Number of daemons currently registered with the runtime. */
  daemonCount(): number {
    return this.inner.daemonCount();
  }

  /**
   * Register a factory closure under `kind`. The factory returns a
   * {@link MeshDaemon}-shaped object. Second registration of the
   * same `kind` throws {@link DaemonError}.
   *
   * Sub-step 1 stores the factory but does not invoke it — event
   * dispatch to daemon `process` lands in sub-step 3.
   *
   * ## Migration targeting
   *
   * `registerFactory` alone is **not sufficient** to accept
   * inbound migrations — it registers the kind-to-factory mapping
   * only on the SDK side. Migrations lookup by `origin_hash`, not
   * by kind. Future sub-steps will surface `expectMigration` and
   * `registerMigrationTargetIdentity` for that wiring.
   */
  registerFactory(kind: string, factory: DaemonFactory): void {
    try {
      this.inner.registerFactory(kind, factory as unknown as () => unknown);
    } catch (e) {
      toDaemonError(e);
    }
  }
}
