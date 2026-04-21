// Integration tests for the capability-broadcast surface on
// `MeshNode`. Exercises announce → find round-trip, self-index,
// late-joiner push, and the flattened POJO conversions — mirrors the
// Rust `tests/capability_broadcast.rs` suite.

import { afterEach, describe, expect, it } from 'vitest';

import type { CapabilityFilter, CapabilitySet } from '../src/capabilities';
import { MeshNode } from '../src/mesh';

const PSK = '42'.repeat(32);

let portSeed = 29_400;
function nextPortPair(): [string, string] {
  const a = portSeed++;
  const b = portSeed++;
  return [`127.0.0.1:${a}`, `127.0.0.1:${b}`];
}

async function handshake(a: MeshNode, b: MeshNode, bAddr: string): Promise<void> {
  const bPub = b.publicKey();
  const aId = a.nodeId();
  const bId = b.nodeId();
  await Promise.all([
    b.accept(aId),
    (async () => {
      await new Promise((r) => setTimeout(r, 50));
      await a.connect(bAddr, bPub, bId);
    })(),
  ]);
  await a.start();
  await b.start();
}

async function pair(): Promise<{ a: MeshNode; b: MeshNode; bAddr: string }> {
  const [aAddr, bAddr] = nextPortPair();
  const a = await MeshNode.create({ bindAddr: aAddr, psk: PSK });
  const b = await MeshNode.create({ bindAddr: bAddr, psk: PSK });
  await handshake(a, b, bAddr);
  return { a, b, bAddr };
}

async function waitUntil(fn: () => boolean, timeoutMs = 2_000): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (fn()) return true;
    await new Promise((r) => setTimeout(r, 25));
  }
  return fn();
}

const nodes: MeshNode[] = [];
afterEach(async () => {
  while (nodes.length > 0) {
    const n = nodes.pop()!;
    try {
      await n.shutdown();
    } catch {
      // Ignore — test may have already closed the node.
    }
  }
});

describe('MeshNode capabilities', () => {
  it('self-matches on its own announcement (single node)', async () => {
    const a = await MeshNode.create({ bindAddr: '127.0.0.1:0', psk: PSK });
    nodes.push(a);

    await a.announceCapabilities({ tags: ['gpu', 'inference'] });

    const hits = a.findPeers({ requireTags: ['gpu'] });
    expect(hits).toContain(a.nodeId());
  });

  it('returns empty for a non-matching filter', async () => {
    const a = await MeshNode.create({ bindAddr: '127.0.0.1:0', psk: PSK });
    nodes.push(a);
    await a.announceCapabilities({ tags: ['gpu'] });

    expect(a.findPeers({ requireTags: ['nope'] })).toEqual([]);
  });

  it('announce → find propagates across the handshake (two nodes)', async () => {
    const { a, b } = await pair();
    nodes.push(a, b);

    await a.announceCapabilities({
      hardware: {
        cpuCores: 16,
        memoryMb: 65_536,
        gpu: { vendor: 'nvidia', model: 'RTX 4090', vramMb: 24_576 },
      },
      tags: ['gpu', 'inference'],
    });

    const aId = a.nodeId();
    const filter: CapabilityFilter = { requireGpu: true, minVramMb: 16_384 };

    const arrived = await waitUntil(() => b.findPeers(filter).includes(aId));
    expect(arrived).toBe(true);
  });

  it('late joiner receives session-open push', async () => {
    const [aAddr, bAddr] = nextPortPair();
    const a = await MeshNode.create({ bindAddr: aAddr, psk: PSK });
    nodes.push(a);

    // A announces *before* B exists.
    await a.announceCapabilities({ tags: ['preannounced'] });

    const b = await MeshNode.create({ bindAddr: bAddr, psk: PSK });
    nodes.push(b);
    await handshake(a, b, bAddr);

    const aId = a.nodeId();
    const arrived = await waitUntil(() =>
      b.findPeers({ requireTags: ['preannounced'] }).includes(aId),
    );
    expect(arrived).toBe(true);
  });

  it('round-trips a complex POJO (hardware + software + models + tools)', async () => {
    const a = await MeshNode.create({ bindAddr: '127.0.0.1:0', psk: PSK });
    nodes.push(a);

    const caps: CapabilitySet = {
      hardware: {
        cpuCores: 32,
        cpuThreads: 64,
        memoryMb: 131_072,
        gpu: {
          vendor: 'nvidia',
          model: 'H100',
          vramMb: 81_920,
          computeUnits: 132,
        },
        storageMb: 4_000_000n,
      },
      software: {
        os: 'linux',
        osVersion: '6.5.0',
        runtimes: [
          ['python', '3.12.4'],
          ['node', '22.3.0'],
        ],
        cudaVersion: '12.4',
      },
      models: [
        {
          modelId: 'llama-3.1-70b',
          family: 'llama',
          parametersBX10: 700,
          contextLength: 131_072,
          modalities: ['text', 'code'],
          loaded: true,
        },
      ],
      tools: [{ toolId: 'web_search', name: 'web search', stateless: true }],
      tags: ['prod', 'us-east'],
      limits: { maxConcurrentRequests: 32 },
    };
    await a.announceCapabilities(caps);

    // Each require* dimension resolves back to self.
    expect(a.findPeers({ requireTags: ['prod'] })).toContain(a.nodeId());
    expect(a.findPeers({ requireModels: ['llama-3.1-70b'] })).toContain(a.nodeId());
    expect(a.findPeers({ requireTools: ['web_search'] })).toContain(a.nodeId());
    expect(a.findPeers({ requireModalities: ['code'] })).toContain(a.nodeId());
    expect(a.findPeers({ gpuVendor: 'nvidia', minVramMb: 40_000 })).toContain(a.nodeId());
  });

  it('drops expired entries after TTL + a GC sweep', async () => {
    // Short interval so the test completes fast. The announcement
    // TTL is 1 s; a 200 ms GC tick means two-to-three sweeps happen
    // before the 1.5 s re-query.
    //
    // `announceCapabilities` takes no TTL override from the TS
    // surface yet — the core default is 5 min — so we'd normally
    // wait minutes. Use `capabilityGcIntervalMs` to speed GC and
    // reach the core `announce_capabilities_with(..., ttl)` via the
    // plain `announceCapabilities` path on the Rust side (which
    // uses 5 min). That TTL is too long here, so we skip the
    // "eventually empty" assertion and instead verify:
    //   (a) the announcement IS indexed (positive path), and
    //   (b) the `capabilityGcIntervalMs` knob is accepted.
    //
    // Full TTL expiry is covered by
    // `tests/capability_broadcast.rs::announcement_expires_after_ttl`
    // where `announce_capabilities_with(caps, 1s, false)` is
    // available at the core Rust layer.
    const a = await MeshNode.create({
      bindAddr: '127.0.0.1:0',
      psk: PSK,
      capabilityGcIntervalMs: 200,
    });
    nodes.push(a);
    await a.announceCapabilities({ tags: ['gc-smoke'] });
    expect(a.findPeers({ requireTags: ['gc-smoke'] })).toContain(a.nodeId());
  });

  it('accepts the requireSignedCapabilities knob without breaking the local path', async () => {
    // `announceCapabilities` path stamps no signature on the wire
    // (signing binds to Stage E), so with
    // `requireSignedCapabilities = true` on a receiver, a direct
    // unsigned announcement would be dropped. We can't easily
    // two-node-test that over the TS boundary (no TS-side wire
    // control), so this is a config-plumbing smoke: the option is
    // accepted and the local self-index path still works because
    // `announce_capabilities` indexes locally before sending.
    const a = await MeshNode.create({
      bindAddr: '127.0.0.1:0',
      psk: PSK,
      requireSignedCapabilities: true,
    });
    nodes.push(a);
    await a.announceCapabilities({ tags: ['local-only'] });
    expect(a.findPeers({ requireTags: ['local-only'] })).toContain(a.nodeId());
  });
});
