// Smoke tests for the CortEX node bindings (tasks + memories).
//
// Exercises CRUD + listFilter snapshots end-to-end through the napi
// boundary. Watch / AsyncIterator is deferred to a follow-up session.

import { describe, expect, it } from 'vitest'

import {
  MemoriesAdapter,
  MemoriesOrderBy,
  Redex,
  TaskStatus,
  TasksAdapter,
  TasksOrderBy,
} from '../index'

const ORIGIN = 0xabcdef01

function nowNs(): bigint {
  return BigInt(Date.now()) * 1_000_000n
}

describe('cortex tasks', () => {
  it('creates, renames, completes, deletes, lists', async () => {
    const redex = new Redex()
    const tasks = await TasksAdapter.open(redex, ORIGIN)

    const t0 = nowNs()
    tasks.create(1n, 'write plan', t0)
    tasks.create(2n, 'ship adapter', t0 + 1n)
    tasks.rename(1n, 'write better plan', t0 + 2n)
    const seq = tasks.complete(2n, t0 + 3n)
    await tasks.waitForSeq(seq)

    const all = tasks.listTasks(null)
    expect(all).toHaveLength(2)

    const t1 = all.find((t) => t.id === 1n)!
    expect(t1.title).toBe('write better plan')
    expect(t1.status).toBe(TaskStatus.Pending)

    const t2 = all.find((t) => t.id === 2n)!
    expect(t2.title).toBe('ship adapter')
    expect(t2.status).toBe(TaskStatus.Completed)

    // Filter: only pending.
    const pending = tasks.listTasks({ status: TaskStatus.Pending })
    expect(pending.map((t) => t.id)).toEqual([1n])

    // Delete + re-list.
    const delSeq = tasks.delete(1n)
    await tasks.waitForSeq(delSeq)
    const after = tasks.listTasks(null)
    expect(after.find((t) => t.id === 1n)).toBeUndefined()
    expect(after.map((t) => t.id)).toEqual([2n])
  })

  it('orders and limits results', async () => {
    const redex = new Redex()
    const tasks = await TasksAdapter.open(redex, ORIGIN)

    for (let i = 1n; i <= 5n; i++) {
      tasks.create(i, `t-${i}`, BigInt(100) * i)
    }
    const last = tasks.complete(1n, 999n)
    await tasks.waitForSeq(last)

    const newest = tasks.listTasks({
      orderBy: TasksOrderBy.CreatedDesc,
      limit: 2,
    })
    expect(newest.map((t) => t.id)).toEqual([5n, 4n])
  })

  it('counts tasks', async () => {
    const redex = new Redex()
    const tasks = await TasksAdapter.open(redex, ORIGIN)
    expect(tasks.count()).toBe(0)
    const seq = tasks.create(1n, 'x', nowNs())
    await tasks.waitForSeq(seq)
    expect(tasks.count()).toBe(1)
  })

  it('rejects ingest after close', async () => {
    const redex = new Redex()
    const tasks = await TasksAdapter.open(redex, ORIGIN)
    tasks.create(1n, 'before close', nowNs())
    tasks.close()
    expect(() => tasks.create(2n, 'after close', nowNs())).toThrow()
  })
})

describe('cortex memories', () => {
  it('stores, retags, pins, lists by tag', async () => {
    const redex = new Redex()
    const memories = await MemoriesAdapter.open(redex, ORIGIN)

    memories.store(1n, 'meeting notes', ['work', 'notes'], 'alice', 100n)
    memories.store(2n, 'grocery list', ['personal', 'todo'], 'alice', 200n)
    memories.store(
      3n,
      'api design',
      ['work', 'design'],
      'bob',
      300n,
    )
    memories.retag(1n, ['work', 'meetings'], 310n)
    memories.pin(3n, 320n)
    const seq = memories.pin(1n, 330n)
    await memories.waitForSeq(seq)

    // All memories.
    expect(memories.count()).toBe(3)

    // Tag predicate: tagged 'work'.
    const work = memories.listMemories({ tag: 'work' })
    const workIds = work.map((m) => m.id).sort()
    expect(workIds).toEqual([1n, 3n])

    // anyTag: design or todo → 2, 3.
    const anyIds = memories
      .listMemories({ anyTag: ['design', 'todo'] })
      .map((m) => m.id)
      .sort()
    expect(anyIds).toEqual([2n, 3n])

    // allTags: work AND meetings → only 1.
    const allIds = memories
      .listMemories({ allTags: ['work', 'meetings'] })
      .map((m) => m.id)
    expect(allIds).toEqual([1n])

    // Pinned only.
    const pinned = memories
      .listMemories({ pinned: true })
      .map((m) => m.id)
      .sort()
    expect(pinned).toEqual([1n, 3n])

    // Source=bob.
    const bob = memories
      .listMemories({ source: 'bob' })
      .map((m) => m.id)
    expect(bob).toEqual([3n])
  })

  it('content search is case-insensitive', async () => {
    const redex = new Redex()
    const memories = await MemoriesAdapter.open(redex, ORIGIN)
    const seq = memories.store(
      1n,
      'Fire in the datacenter',
      [],
      'alice',
      100n,
    )
    await memories.waitForSeq(seq)

    const hit = memories.listMemories({ contentContains: 'DATACENTER' })
    expect(hit.map((m) => m.id)).toEqual([1n])

    const miss = memories.listMemories({ contentContains: 'unicorn' })
    expect(miss).toHaveLength(0)
  })

  it('orders and limits', async () => {
    const redex = new Redex()
    const memories = await MemoriesAdapter.open(redex, ORIGIN)

    for (let i = 1n; i <= 5n; i++) {
      memories.store(i, `m-${i}`, [], 'alice', 100n * i)
    }
    const last = memories.unpin(1n, 999n) // no-op logically but advances fold
    await memories.waitForSeq(last)

    const newest = memories.listMemories({
      orderBy: MemoriesOrderBy.CreatedDesc,
      limit: 2,
    })
    expect(newest.map((m) => m.id)).toEqual([5n, 4n])
  })

  it('delete removes the memory', async () => {
    const redex = new Redex()
    const memories = await MemoriesAdapter.open(redex, ORIGIN)
    memories.store(1n, 'ephemeral', [], 'alice', 100n)
    const seq = memories.delete(1n)
    await memories.waitForSeq(seq)
    expect(memories.count()).toBe(0)
  })
})

describe('cortex multi-model', () => {
  it('tasks and memories coexist on one Redex', async () => {
    const redex = new Redex()
    const tasks = await TasksAdapter.open(redex, ORIGIN)
    const memories = await MemoriesAdapter.open(redex, ORIGIN)

    tasks.create(1n, 'task-1', 100n)
    memories.store(1n, 'mem-1', ['x'], 'alice', 100n)
    memories.store(2n, 'mem-2', ['x'], 'alice', 200n)
    const ts = tasks.complete(1n, 150n)
    const ms = memories.pin(1n, 250n)

    await tasks.waitForSeq(ts)
    await memories.waitForSeq(ms)

    expect(tasks.count()).toBe(1)
    expect(memories.count()).toBe(2)
    expect(memories.listMemories({ pinned: true }).map((m) => m.id)).toEqual([
      1n,
    ])
  })
})
