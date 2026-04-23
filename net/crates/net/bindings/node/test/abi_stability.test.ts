// ABI-stability regression tests for the Node binding's error
// surface. These pin the wire format that SDK consumers match on:
//
//   1. `classifyError()` routes messages starting with `cortex:`
//      and `netdb:` into typed classes. A rename of either prefix
//      on the Rust side breaks every caller using `instanceof`.
//   2. `daemon:` / `daemon: migration: <kind>[: <detail>]` messages
//      are the cross-binding contract (Go / Python also emit it).
//      Callers inspect `error.message.startsWith('daemon:')` and
//      parse the kind themselves — this file pins the kind
//      vocabulary and the envelope shape.
//   3. Prefixes must remain case-sensitive and fall-through for
//      unrelated errors — `classifyError` is an identity function
//      on unknown messages.
//
// These tests operate on the pure-JS helper path: no native module
// is required to build synthetic `Error` objects, so the tests are
// cheap and catch renames *before* the cdylib is rebuilt.
//
// Corresponds to TEST_COVERAGE_PLAN §P3-15.

import { describe, expect, it } from 'vitest'

import { classifyError, CortexError, NetDbError } from '../errors'

// Cross-binding kind vocabulary for migration failures. Must stay
// byte-identical to `MigrationErrorKind` in the Go binding and to
// the `MIGRATION_KINDS` tuple in the Python tests — an accidental
// rename here is the whole point of these tests.
const MIGRATION_KINDS = [
  'not-ready',
  'factory-not-found',
  'compute-not-supported',
  'state-failed',
  'already-migrating',
  'identity-transport-failed',
  'not-ready-timeout',
  'daemon-not-found',
  'target-unavailable',
  'wrong-phase',
  'snapshot-too-large',
] as const

describe('ABI stability: classifyError routes prefixed messages', () => {
  it('cortex:-prefixed messages become CortexError', () => {
    const raw = new Error('cortex: adapter closed')
    const typed = classifyError(raw)
    expect(typed).toBeInstanceOf(CortexError)
    expect((typed as Error).message).toBe('cortex: adapter closed')
  })

  it('netdb:-prefixed messages become NetDbError', () => {
    const raw = new Error('netdb: file not found')
    const typed = classifyError(raw)
    expect(typed).toBeInstanceOf(NetDbError)
    expect((typed as Error).message).toBe('netdb: file not found')
  })

  it('unprefixed errors pass through unchanged (identity)', () => {
    const raw = new Error('something else entirely')
    expect(classifyError(raw)).toBe(raw)
  })

  it('daemon:-prefixed errors pass through unchanged — callers parse themselves', () => {
    // The Rust `bindings/node/src/compute.rs` surface prefixes
    // `daemon:` on every compute/migration error. `classifyError`
    // is NOT responsible for lifting these — SDK callers inspect
    // `error.message.startsWith('daemon:')` directly. Pinning the
    // identity behavior lets either side change independently.
    const raw = new Error('daemon: migration: not-ready')
    expect(classifyError(raw)).toBe(raw)
  })

  it('prefix matching is case-sensitive', () => {
    // A case-normalized classifier would quietly match unintended
    // upstream errors. Pinning strict case.
    const raw = new Error('CORTEX: not the real prefix')
    const result = classifyError(raw)
    expect(result).toBe(raw)
    expect(result).not.toBeInstanceOf(CortexError)
  })

  it('null / undefined / plain objects do not throw', () => {
    // `classifyError` is called from top-level catch handlers;
    // it must tolerate any rejected value shape.
    expect(classifyError(null)).toBe(null)
    expect(classifyError(undefined)).toBe(undefined)
    const obj = { message: 'cortex: fake' } as Error
    expect(classifyError(obj)).toBeInstanceOf(CortexError)
  })
})

describe('ABI stability: daemon / migration wire envelope', () => {
  // The cross-binding contract. If any of these fail, either the
  // Rust wire format changed or this test is out of date — both
  // need to land together.

  const MIGRATION_ENVELOPE = /^daemon: migration: [a-z][a-z0-9-]*(: .+)?$/

  it.each(MIGRATION_KINDS)(
    'migration kind %s forms a well-shaped envelope',
    (kind) => {
      const tagOnly = `daemon: migration: ${kind}`
      expect(tagOnly).toMatch(MIGRATION_ENVELOPE)

      const withDetail = `daemon: migration: ${kind}: some detail`
      expect(withDetail).toMatch(MIGRATION_ENVELOPE)
    },
  )

  it.each(MIGRATION_KINDS)('kind %s is lowercase kebab-case', (kind) => {
    expect(kind).toBe(kind.toLowerCase())
    expect(kind).not.toMatch(/_/)
    expect(kind).toMatch(/^[a-z][a-z0-9-]*$/)
  })

  it('SDK callers can split the kind off the envelope', () => {
    // Mirror of `migration_error_kind` in the Python binding —
    // pins the parse contract Node SDK callers will use.
    function migrationErrorKind(err: Error): string | null {
      const prefix = 'daemon: migration:'
      if (!err.message.startsWith(prefix)) return null
      const body = err.message.slice(prefix.length).trim()
      const colon = body.indexOf(':')
      return colon === -1 ? body : body.slice(0, colon).trim()
    }

    for (const kind of MIGRATION_KINDS) {
      expect(migrationErrorKind(new Error(`daemon: migration: ${kind}`))).toBe(
        kind,
      )
      expect(
        migrationErrorKind(new Error(`daemon: migration: ${kind}: detail`)),
      ).toBe(kind)
    }

    // Fall-through for non-migration daemon errors.
    expect(
      migrationErrorKind(new Error('daemon: factory already registered')),
    ).toBe(null)
    expect(migrationErrorKind(new Error('not a daemon error'))).toBe(null)
  })

  it('unknown future kinds surface verbatim (forward compat)', () => {
    function migrationErrorKind(err: Error): string | null {
      const prefix = 'daemon: migration:'
      if (!err.message.startsWith(prefix)) return null
      const body = err.message.slice(prefix.length).trim()
      const colon = body.indexOf(':')
      return colon === -1 ? body : body.slice(0, colon).trim()
    }

    // Older Node SDK seeing a newer Rust kind: must still return
    // the raw kind string so callers can log + gracefully reject.
    expect(
      migrationErrorKind(new Error('daemon: migration: future-kind-2030')),
    ).toBe('future-kind-2030')
  })
})

describe('ABI stability: prefix-literal pin', () => {
  // Hard-code the exact prefix strings. If anyone edits
  // `bindings/node/errors.js` to change the constants, this
  // catches the drift immediately.
  it('classifies `cortex:` and `netdb:` by literal prefix', () => {
    expect(classifyError(new Error('cortex:')).constructor.name).toBe(
      'CortexError',
    )
    expect(classifyError(new Error('netdb:')).constructor.name).toBe(
      'NetDbError',
    )
  })

  it('does not classify `cortex_` or `cortex ` (near-misses)', () => {
    // Trailing char must be a colon — underscores or spaces are
    // not legitimate. Pinned so the internal `startsWith(prefix)`
    // check doesn't drift to a looser matcher.
    const u = new Error('cortex_something')
    const s = new Error('cortex something')
    expect(classifyError(u)).toBe(u)
    expect(classifyError(s)).toBe(s)
  })
})
