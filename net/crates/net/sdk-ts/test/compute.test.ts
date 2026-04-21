// Smoke tests for the compute surface — Stage 3 sub-step 1.
//
// Scope: lifecycle only. A TS caller can build a `DaemonRuntime`
// against a `MeshNode`, register a factory (stored but not yet
// invoked), start the runtime, and shut it down. Event delivery,
// migration, snapshot/restore, and cross-language daemon execution
// land in sub-steps 2-5.

import { afterEach, describe, expect, it } from 'vitest';

import { DaemonError, DaemonRuntime, MeshNode } from '../src';

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
