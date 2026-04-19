// Error-class smoke tests — verifies the napi binding emits stable
// `cortex:` / `netdb:` prefixes and that `classifyError()` rehydrates
// them into typed `CortexError` / `NetDbError` instances.

import { describe, expect, it } from 'vitest'

import { MemoriesAdapter, NetDb, Redex, TasksAdapter } from '../index'
import { classifyError, CortexError, NetDbError } from '../errors'

const ORIGIN = 0xabcdef01

function nowNs(): bigint {
  return BigInt(Date.now()) * 1_000_000n
}

describe('error classification', () => {
  it('tasks operations on a closed adapter throw a cortex-prefixed error', async () => {
    const redex = new Redex()
    const tasks = await TasksAdapter.open(redex, ORIGIN)
    tasks.close()

    let caught: Error | null = null
    try {
      tasks.create(1n, 'x', nowNs())
    } catch (e) {
      caught = e as Error
    }
    expect(caught).not.toBeNull()
    expect(caught!.message.startsWith('cortex:')).toBe(true)

    const typed = classifyError(caught)
    expect(typed).toBeInstanceOf(CortexError)
    expect(typed).toBeInstanceOf(Error)
  })

  it('memories operations on a closed adapter throw a cortex-prefixed error', async () => {
    const redex = new Redex()
    const memories = await MemoriesAdapter.open(redex, ORIGIN)
    memories.close()

    let caught: Error | null = null
    try {
      memories.store(1n, 'x', ['t'], 's', nowNs())
    } catch (e) {
      caught = e as Error
    }
    expect(caught).not.toBeNull()
    expect(classifyError(caught)).toBeInstanceOf(CortexError)
  })

  it('persistent=true without persistent_dir raises a CortexError', async () => {
    const redex = new Redex() // heap-only
    let caught: Error | null = null
    try {
      await TasksAdapter.open(redex, ORIGIN, true)
    } catch (e) {
      caught = e as Error
    }
    expect(caught).not.toBeNull()
    expect(caught!.message).toMatch(/persistent/)
    expect(classifyError(caught)).toBeInstanceOf(CortexError)
  })

  it('NetDb.openFromSnapshot on garbage bytes raises a NetDbError', async () => {
    const garbage = Buffer.from([0xff, 0xff, 0xff, 0x00, 0xff])
    let caught: Error | null = null
    try {
      await NetDb.openFromSnapshot(
        { originHash: ORIGIN, withTasks: true },
        { stateBytes: garbage },
      )
    } catch (e) {
      caught = e as Error
    }
    expect(caught).not.toBeNull()
    expect(caught!.message.startsWith('netdb:')).toBe(true)
    expect(classifyError(caught)).toBeInstanceOf(NetDbError)
  })

  it('classifyError passes unknown errors through unchanged', () => {
    const unrelated = new Error('something totally different')
    expect(classifyError(unrelated)).toBe(unrelated)
  })

  it('CortexError and NetDbError are independent Error subclasses', () => {
    const c = new CortexError('x')
    const n = new NetDbError('y')
    expect(c instanceof Error).toBe(true)
    expect(n instanceof Error).toBe(true)
    expect(c instanceof NetDbError).toBe(false)
    expect(n instanceof CortexError).toBe(false)
    expect(c.name).toBe('CortexError')
    expect(n.name).toBe('NetDbError')
  })
})
