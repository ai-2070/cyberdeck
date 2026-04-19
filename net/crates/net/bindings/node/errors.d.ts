/**
 * Thrown by CortEX adapter operations (tasks, memories) on adapter-
 * level failures: `adapter closed`, `fold stopped at seq N`, and
 * underlying RedEX storage errors.
 *
 * Rehydrate via `classifyError(e)` — the napi binding itself throws
 * a plain `Error` with a `cortex:` message prefix; this class exists
 * so callers can use `instanceof CortexError` at their catch sites.
 */
export class CortexError extends Error {
  constructor(detail?: string)
}

/**
 * Thrown by NetDB handle-level operations: snapshot encode / decode,
 * missing-model accesses. Per-adapter failures inside a NetDb still
 * classify as {@link CortexError} — NetDbError is reserved for errors
 * that belong to the NetDb handle itself.
 */
export class NetDbError extends Error {
  constructor(detail?: string)
}

/**
 * Inspect a caught error's message prefix and return a typed
 * {@link CortexError} / {@link NetDbError} if it matches the binding's
 * contract. Non-matching errors are returned unchanged so you can
 * `throw classifyError(e)` unconditionally.
 */
export function classifyError(e: unknown): unknown
