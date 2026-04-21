// Smoke tests for the compute surface — Stage 3 sub-step 1.
//
// Scope: lifecycle only. A TS caller can build a `DaemonRuntime`
// against a `MeshNode`, register a factory (stored but not yet
// invoked), start the runtime, and shut it down. Event delivery,
// migration, snapshot/restore, and cross-language daemon execution
// land in sub-steps 2-5.

import { afterEach, describe, expect, it } from 'vitest';

import {
  DaemonError,
  DaemonHandle,
  DaemonRuntime,
  Identity,
  MeshNode,
} from '../src';

const PSK = '42'.repeat(32);

// Unique ports per test case so repeated runs don't collide.
let portSeed = 29_100;
function nextPort(): string {
  return `127.0.0.1:${portSeed++}`;
}

async function buildMesh(): Promise<MeshNode> {
  return MeshNode.create({ bindAddr: nextPort(), psk: PSK });
}

describe('DaemonRuntime (Stage 3 sub-step 1: skeleton + lifecycle)', () => {
  const cleanups: Array<() => Promise<void>> = [];

  afterEach(async () => {
    while (cleanups.length > 0) {
      const fn = cleanups.pop();
      if (fn) {
        try {
          await fn();
        } catch {
          // Best-effort — we're tearing down fixtures, not asserting on them.
        }
      }
    }
  });

  it('builds against a mesh and reports not-ready before start', async () => {
    const mesh = await buildMesh();
    cleanups.push(() => mesh.shutdown());

    const rt = DaemonRuntime.create(mesh);
    cleanups.push(() => rt.shutdown());

    expect(rt.isReady()).toBe(false);
    expect(rt.daemonCount()).toBe(0);
  });

  it('start flips to ready; shutdown flips back', async () => {
    const mesh = await buildMesh();
    cleanups.push(() => mesh.shutdown());

    const rt = DaemonRuntime.create(mesh);
    await rt.start();
    expect(rt.isReady()).toBe(true);

    await rt.shutdown();
    expect(rt.isReady()).toBe(false);
  });

  it('registerFactory accepts a JS factory; second registration of the same kind throws', async () => {
    const mesh = await buildMesh();
    cleanups.push(() => mesh.shutdown());

    const rt = DaemonRuntime.create(mesh);
    cleanups.push(() => rt.shutdown());

    const stubFactory = () => ({
      name: 'echo',
      process: (_event: unknown) => [],
    });

    // First registration: succeeds.
    rt.registerFactory('echo', stubFactory);

    // Second: rejected with a typed `DaemonError`.
    expect(() => rt.registerFactory('echo', stubFactory)).toThrow(DaemonError);
    expect(() => rt.registerFactory('echo', stubFactory)).toThrow(
      /already registered/,
    );
  });

  it('registerFactory works after start (runtime admits new kinds in Ready state)', async () => {
    const mesh = await buildMesh();
    cleanups.push(() => mesh.shutdown());

    const rt = DaemonRuntime.create(mesh);
    cleanups.push(() => rt.shutdown());

    await rt.start();

    expect(() =>
      rt.registerFactory('late', () => ({
        name: 'late',
        process: () => [],
      })),
    ).not.toThrow();
  });

  it('shutdown is idempotent', async () => {
    const mesh = await buildMesh();
    cleanups.push(() => mesh.shutdown());

    const rt = DaemonRuntime.create(mesh);
    await rt.start();
    await rt.shutdown();
    // Second shutdown: no throw.
    await expect(rt.shutdown()).resolves.toBeUndefined();
  });
});

// Sub-step 2a: spawn / stop lifecycle. Daemon is a no-op bridge
// on the Rust side; factory TSFN is not yet invoked. Sub-step 2b
// replaces the bridge with one that dispatches events to the
// JS-returned object.
describe('DaemonRuntime (Stage 3 sub-step 2a: spawn + stop)', () => {
  const cleanups: Array<() => Promise<void>> = [];

  afterEach(async () => {
    while (cleanups.length > 0) {
      const fn = cleanups.pop();
      if (fn) {
        try {
          await fn();
        } catch {
          // Best-effort teardown.
        }
      }
    }
  });

  async function startedRuntime(): Promise<{
    mesh: MeshNode;
    rt: DaemonRuntime;
  }> {
    const mesh = await buildMesh();
    cleanups.push(() => mesh.shutdown());
    const rt = DaemonRuntime.create(mesh);
    cleanups.push(() => rt.shutdown());
    rt.registerFactory('echo', () => ({ name: 'echo', process: () => [] }));
    await rt.start();
    return { mesh, rt };
  }

  it('spawn returns a handle with originHash + entityId', async () => {
    const { rt } = await startedRuntime();
    const id = Identity.generate();
    const handle = await rt.spawn('echo', id);

    expect(handle).toBeInstanceOf(DaemonHandle);
    expect(typeof handle.originHash).toBe('number');
    expect(handle.entityId).toBeInstanceOf(Buffer);
    expect(handle.entityId.length).toBe(32);
    // originHash matches identity's origin hash (first 4 bytes of
    // the BLAKE2s-derived hash). We don't recompute here — just
    // assert it's non-zero, because `Identity.generate()` produces
    // random bytes that essentially never hash to zero.
    expect(handle.originHash).not.toBe(0);

    expect(rt.daemonCount()).toBe(1);
  });

  it('spawn -> stop reduces daemonCount', async () => {
    const { rt } = await startedRuntime();
    const handle = await rt.spawn('echo', Identity.generate());
    expect(rt.daemonCount()).toBe(1);

    await rt.stop(handle.originHash);
    expect(rt.daemonCount()).toBe(0);
  });

  it('spawn with an unregistered kind throws DaemonError', async () => {
    const { rt } = await startedRuntime();
    await expect(rt.spawn('missing', Identity.generate())).rejects.toThrow(
      DaemonError,
    );
    await expect(rt.spawn('missing', Identity.generate())).rejects.toThrow(
      /no factory registered/,
    );
  });

  it('spawn with the same identity twice rejects the second call', async () => {
    const { rt } = await startedRuntime();
    const id = Identity.generate();
    await rt.spawn('echo', id);
    // Second spawn: same origin_hash -> atomic factory_registry
    // rejects. The underlying SDK surfaces this as the
    // `already registered` message with the `daemon:` prefix.
    await expect(rt.spawn('echo', id)).rejects.toThrow(DaemonError);
    expect(rt.daemonCount()).toBe(1);
  });

  it('spawn many, stop each, daemonCount reaches zero', async () => {
    const { rt } = await startedRuntime();
    const handles: DaemonHandle[] = [];
    for (let i = 0; i < 10; i++) {
      handles.push(await rt.spawn('echo', Identity.generate()));
    }
    expect(rt.daemonCount()).toBe(10);

    for (const h of handles) {
      await rt.stop(h.originHash);
    }
    expect(rt.daemonCount()).toBe(0);
  });

  it('spawn after shutdown rejects with DaemonError', async () => {
    const { rt } = await startedRuntime();
    await rt.shutdown();
    await expect(rt.spawn('echo', Identity.generate())).rejects.toThrow(
      DaemonError,
    );
  });

  it('config with auto-snapshot + max-log-entries is accepted', async () => {
    const { rt } = await startedRuntime();
    const handle = await rt.spawn('echo', Identity.generate(), {
      autoSnapshotInterval: 128n,
      maxLogEntries: 2048,
    });
    expect(handle.originHash).not.toBe(0);
  });

  it('factory is invoked exactly once per spawn; each invocation gets its own closure state', async () => {
    // Factory that closes over a per-invocation counter — proves
    // each spawn gets a fresh instance, not a shared one.
    const mesh = await buildMesh();
    cleanups.push(() => mesh.shutdown());
    const rt = DaemonRuntime.create(mesh);
    cleanups.push(() => rt.shutdown());

    let invocations = 0;
    rt.registerFactory('counter', () => {
      invocations++;
      let localState = 0;
      return {
        name: 'counter',
        // `process` closes over `localState`; if sub-step 3 ever
        // starts dispatching events, each instance will see its
        // own state. Sub-step 2b: just assert the factory ran.
        process: () => {
          localState++;
          return [];
        },
      };
    });
    await rt.start();

    expect(invocations).toBe(0);
    await rt.spawn('counter', Identity.generate());
    expect(invocations).toBe(1);
    await rt.spawn('counter', Identity.generate());
    expect(invocations).toBe(2);
    await rt.spawn('counter', Identity.generate());
    expect(invocations).toBe(3);
    expect(rt.daemonCount()).toBe(3);
  });

  it('async factory is awaited before spawn resolves', async () => {
    const mesh = await buildMesh();
    cleanups.push(() => mesh.shutdown());
    const rt = DaemonRuntime.create(mesh);
    cleanups.push(() => rt.shutdown());

    let factoryResolved = false;
    rt.registerFactory('async-echo', async () => {
      await new Promise((r) => setTimeout(r, 10));
      factoryResolved = true;
      return { name: 'async-echo', process: () => [] };
    });
    await rt.start();

    expect(factoryResolved).toBe(false);
    const handle = await rt.spawn('async-echo', Identity.generate());
    expect(factoryResolved).toBe(true);
    expect(handle.originHash).not.toBe(0);
  });

  it('snapshot / restore methods are optional', async () => {
    const mesh = await buildMesh();
    cleanups.push(() => mesh.shutdown());
    const rt = DaemonRuntime.create(mesh);
    cleanups.push(() => rt.shutdown());

    // Stateless factory — only `process`. snapshot/restore omitted.
    rt.registerFactory('stateless', () => ({
      name: 'stateless',
      process: () => [],
    }));
    await rt.start();

    const handle = await rt.spawn('stateless', Identity.generate());
    expect(handle.originHash).not.toBe(0);
    await rt.stop(handle.originHash);
    expect(rt.daemonCount()).toBe(0);
  });
});
