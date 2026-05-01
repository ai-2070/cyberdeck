# BUG #56 + #57 — Cross-Process At-Least-Once Dedup

Resolves the paired deferred entries from
`BUG_AUDIT_2026_04_30_CORE.md:145, 150`. Both stem from the same
gap (no durable identity for in-flight batches across process
boundaries) and ship together.

## #56 — JetStream cross-process retry duplicates

`adapter/jetstream.rs:285-320` builds `Nats-Msg-Id` as
`{process_nonce}:{shard_id}:{sequence_start}:{i}` where
`process_nonce` is generated at process start (`event.rs:304`).
JetStream's server-side dedup window discards a same-msg-id retry
within the (default 2 min) window. That works **within one process
lifetime**: a `try_join_all(acks)` short-circuit on first error
followed by a retry produces the same msg-ids and the server dedups
them.

**The bug:** if the producer crashes mid-batch (server already
accepted some events but not all of the acks completed locally) and
restarts, the new process gets a fresh `process_nonce`. Retransmits
write *new* msg-ids, JetStream sees them as fresh, and persists the
partial-batch's accepted half twice — once from the original send,
once from the post-restart retry.

## #57 — Redis MULTI/EXEC timeout cancellation duplicates

`adapter/redis.rs:298-319` wraps the EXEC in `tokio::time::timeout`.
The local future is dropped on timeout, but the bytes are already on
the wire and the server may run the EXEC anyway. The retry path
issues another `MULTI/EXEC`, producing two distinct XADD streams of
the same logical events. Each XADD gets a server-generated `*` id, so
downstream consumers cannot dedupe — the application-defined `r.id`
is not necessarily unique.

## Shared shape

Both bugs are "the wire op completed server-side but we don't know
locally, so the retry produces duplicates." Both need a notion of
batch identity that:

1. **Survives process restart.** A nonce regenerated at startup
   (today's `process_nonce`) is insufficient.
2. **Is observable on retry.** The retry path must construct the
   same identity it used the first time, so the server (or our own
   downstream consumer) can dedup.
3. **Is bounded in size.** Can't keep nonces around forever; need
   eviction tied to the dedup window.

### Two design directions

**Direction A — durable producer identity ("persistent nonce").**
Replace the per-process `process_nonce` with a per-installation
nonce stored on disk. On restart the same nonce is loaded. Same-batch
retries (within or across processes) write the same msg-ids. The
JetStream dedup window absorbs duplicates as long as the retry
happens within 2 min.

- **Pro:** small, contained — one new file, one new module that
  reads/writes it. JetStream's existing dedup-window does the work.
- **Con:** crashes that restart >2 min later still produce
  duplicates (dedup window lapsed). And: a single producer-identity
  collision across two co-located processes silently corrupts state.

**Direction B — durable batch ledger ("at-least-once with
client-side dedup").** Maintain a small persistent log of batches
the producer has *attempted* (durable before send). On retry, look
up the prior attempt's ack state; if the server already took it,
skip. If not, reattempt with the same identity.

- **Pro:** correctness across arbitrary crash/restart windows; no
  dependency on server-side dedup window TTL.
- **Con:** significantly more invasive — adds I/O on the hot path
  (durable write before send), needs a recovery scanner on startup,
  needs a garbage-collection policy.

**Pick: Direction A**, with deliberate scope limits. It's the right
fit for the documented audit failure (a producer crash + retry).
The longer crash windows that Direction B would cover are real but
unusual; deferring them keeps the change small enough to ship in
one PR and matches what JetStream / Redis users typically expect
("our dedup window protects you").

## Design — Direction A

New module: `adapter::dedup_state`. Single struct
`PersistentProducerNonce` with:

```rust
pub struct PersistentProducerNonce {
    nonce: u64,
    path: PathBuf,
}

impl PersistentProducerNonce {
    /// Load (or create) the persistent nonce at `path`.
    /// On first call, generates a fresh u64 from getrandom and
    /// writes it. On subsequent calls (post-restart), reads it
    /// back. Returns the nonce.
    pub fn load_or_create(path: impl AsRef<Path>) -> io::Result<Self> { ... }

    pub fn nonce(&self) -> u64 { ... }
}
```

Wire-up:

- `EventBusConfig` gains `producer_nonce_path: Option<PathBuf>`. When
  `Some`, the bus loads the persistent nonce on construction and
  hands a `u64` to the JetStream and Redis adapters via the existing
  per-batch nonce field. When `None`, falls back to a fresh
  per-process random nonce — current behavior, documented as
  "at-most-once across restarts."
- `adapter/jetstream.rs::on_batch` uses
  `format!("{nonce}:{shard_id}:{sequence_start}:{i}")` as today, but
  `nonce` now comes from `producer_nonce` (the persistent value)
  rather than `process_nonce`.
- `adapter/redis.rs::on_batch` learns to embed `nonce:shard:seq:i` into
  the XADD payload as a stream entry's NOMKSTREAM idempotency token,
  and then on retry looks up the most recent token — but Redis Streams
  doesn't have server-side dedup, so this **only works if we shift
  to one-XADD-per-event with `Nx-Msg-Id`-style ids stored in the
  field map**, and our downstream consumers learn to dedupe on that
  field. That last point is a behavioral break for anyone consuming
  the raw stream.

### Redis caveat (the harder half)

JetStream has server-side dedup; Redis does not. Direction A only
fixes Redis if either:

1. We switch the Redis adapter from `XADD MULTI` to a single-event
   pattern with idempotency tokens, and document the consumer-side
   dedup contract; or
2. We accept that #57's "EXEC ran server-side after we dropped the
   future" leaks duplicates that downstream consumers must tolerate
   — and limit the producer-side change to JetStream.

**Recommend (2)** — Direction A is a pure JetStream fix. #57
remains documented as a known limitation with a runtime warning at
startup if the Redis adapter is configured *and* `producer_nonce_path`
is `Some` (signal: caller wanted dedup but Redis can't provide it).
A follow-up PR can address #57 via consumer-side dedup once we have
production data on the actual duplicate rate.

This means **#57 stays "Outstanding"** at the end of this PR. The
audit doc will note it as a JetStream-only fix.

## Files touched

Implementation:
1. `src/adapter/dedup_state.rs` (new module) —
   `PersistentProducerNonce` struct, file-format design, atomic
   write (`tempfile + rename`), getrandom seed.
2. `src/adapter/mod.rs` — re-export.
3. `src/config.rs` — `EventBusConfig::producer_nonce_path: Option<PathBuf>`.
4. `src/bus.rs::EventBus::new_with_adapter` — read the persistent
   nonce on startup; thread into adapter construction.
5. `src/event.rs` — `process_nonce()` becomes `producer_nonce(&self)
   -> u64` taking the bus's loaded nonce, or stays as a fallback for
   the path-not-configured case.
6. `src/adapter/jetstream.rs::on_batch` — use producer nonce instead
   of process nonce.
7. `src/adapter/redis.rs::on_batch` — compile-time check / runtime
   warning if `producer_nonce_path` is set (the persistent nonce
   doesn't help us under Redis's no-server-dedup model).

Tests:
8. Unit tests on `PersistentProducerNonce`:
   - First load creates a nonce file with a random u64.
   - Second load returns the same nonce.
   - Corrupt file (non-u64 contents) → `io::Error`.
   - Concurrent first-load on the same path → both succeed with the
     same nonce (atomic-rename-on-create).
9. Integration test `jetstream_simulated_crash_replays_with_same_msg_ids`:
   - Build EventBus with persistent nonce path. Ingest a batch. Get
     the msg-id JetStream observed. Drop the bus, build a new bus
     against the same path. Ingest the same logical batch. Assert the
     retry msg-id matches the original.
10. Negative test that without `producer_nonce_path`, retries across
    bus instances produce *different* msg-ids (documenting the
    documented behavior).

Audit doc:
11. Mark #56 FIXED, leave #57 in Outstanding with the updated note
    that the JetStream half landed but Redis remains pending the
    consumer-side dedup design.

## Non-goals

- Direction B (durable batch ledger). Real follow-up work for full
  exactly-once semantics across arbitrary crash windows.
- Server-side dedup window tuning. The 2 min default is JetStream's;
  callers configure it server-side.
- The Redis half of the duplicate problem (#57). Scoped out for the
  reasons above; remains in the audit's Outstanding list.
- Cross-machine producer identity (a nonce file shared between
  multiple producer processes via a shared filesystem). The current
  design is one-nonce-per-installation; collisions across machines
  are documented as caller responsibility.

## Implementation order

1. `dedup_state` module + unit tests.
2. `EventBusConfig` + bus wire-up.
3. JetStream adapter wire-up.
4. Redis adapter warning (no behavioral change).
5. Integration tests.
6. Audit doc.
