// Typed error classes for CortEX / NetDB operations.
//
// The napi binding throws plain `Error` objects with stable prefixes
// (`cortex:` / `netdb:`) that `classifyError()` inspects to re-throw
// a typed `CortexError` / `NetDbError`. Catch with `instanceof`:
//
//   import { NetDb } from '@ai2070/net';
//   import { CortexError, classifyError } from '@ai2070/net/errors';
//
//   try {
//     db.tasks.create(1n, 'x', 100n);
//   } catch (e) {
//     throw classifyError(e); // CortexError / NetDbError / original
//   }
//
// Prefixes mirror `ERR_CORTEX_PREFIX` / `ERR_NETDB_PREFIX` in
// `bindings/node/src/cortex.rs`. Keep the strings in lockstep.

'use strict'

const ERR_CORTEX_PREFIX = 'cortex:'
const ERR_NETDB_PREFIX = 'netdb:'

class CortexError extends Error {
  constructor(detail) {
    super(detail ?? 'cortex adapter error')
    this.name = 'CortexError'
    Object.setPrototypeOf(this, CortexError.prototype)
  }
}

class NetDbError extends Error {
  constructor(detail) {
    super(detail ?? 'netdb error')
    this.name = 'NetDbError'
    Object.setPrototypeOf(this, NetDbError.prototype)
  }
}

/**
 * Inspect an error's message prefix and return a typed error if it
 * matches the napi binding's contract. Non-matching errors are
 * returned unchanged — caller can `throw` the result unconditionally.
 */
function classifyError(e) {
  const msg = (e && e.message) || ''
  if (msg.startsWith(ERR_CORTEX_PREFIX)) {
    return new CortexError(msg)
  }
  if (msg.startsWith(ERR_NETDB_PREFIX)) {
    return new NetDbError(msg)
  }
  return e
}

module.exports = {
  CortexError,
  NetDbError,
  classifyError,
}
