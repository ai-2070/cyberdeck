# Stream Backpressure — v2: round-trip credit windows

## Status

Design only. v1 (local in-flight counter) shipped in `STREAM_BACKPRESSURE_PLAN.md`: `StreamError::Backpressure` fires when concurrent callers race on the same stream. v2 closes the gap that v1 explicitly doesn't: a **single serial sender outrunning a slow receiver across the network**. Same `StreamError::Backpressure` variant, same SDK helpers, same daemon patterns — only the internal condition that triggers it moves from "local counter full" to "no credit from peer."

No caller-facing API changes. No new `StreamError` variants. No changes to the Rust/TS/Python SDK surface.

## What v1 already gives us (recap)

- **`StreamError::Backpressure`** — public API variant, already pattern-matched in daemon code.
- **`StreamState.tx_inflight: AtomicU32`** — per-stream counter, CAS-loop admission.
- **`StreamState.tx_window: u32`** — configured cap. v1 interprets as "max in-flight **packets**".
- **`StreamState.backpressure_events: AtomicU64`** — rejection counter, exposed via `StreamStats`.
- **RAII `TxSlotGuard`** — releases on Drop; survives async cancellation. Works identically whether the counter is in packets or bytes.
- **Epoch-guarded stream handles** — a `Stream` held across close+reopen already returns `NotConnected`.
- **SDK helpers (`send_with_retry`, `send_blocking`)** — back off on `Backpressure` only; transport errors pass through.
- **Origin / topology / session plumbing** — peer sessions, subprotocol dispatch, `SubprotocolRegistry`, and the channel-publisher-style request/response pattern from `SUBPROTOCOL_CHANNEL_MEMBERSHIP` are the reference shape for the new control subprotocol.

## What v1 doesn't catch

v1 is a **concurrent-caller** guard, not a network-speed guard. A single serial sender doing:

```rust
for event in events {
    mesh.send_on_stream(&stream, &[event]).await?;
}
```

stays inside the window (always `0 → 1 → 0` per packet) and **never** sees `Backpressure`. It gets implicit OS-level backpressure as `StreamError::Transport(io::Error)` when the kernel send buffer fills — indistinguishable from a hard socket failure. Meanwhile the *receiver* has no way to tell the sender "stop, my ingest queue is full" without layering its own app-level flow control.

That's the v2 gap.

## Gaps this plan fills

1. **No receiver-driven flow control.** A slow daemon on the receive side fills its inbound `SegQueue` without back-pressure reaching the sender. The only defense today is `poll_shard` eventually draining — but the sender keeps pumping regardless.
2. **Single-sender flooding is invisible.** Unlike v1's concurrent-caller case, a serial sender outrunning the network sees `Transport(io::Error)` for kernel-buffer-full, which pattern-matches against real failures. No clean "slow down" signal.
3. **`window_bytes` semantics still say "packets".** The field is named `window_bytes` but v1 counts packets. v2 realizes the name: the accounting unit becomes bytes, matching what the network actually rate-limits on.
4. **No observability into why a send stalled.** `backpressure_events` counts rejections, but can't distinguish "local concurrency pile-up" from "receiver grants exhausted."

## Goals

- **Same API surface.** `StreamError::Backpressure`, `send_with_retry`, `send_blocking`, `BackpressureError` classes in TS/Python — all unchanged. The v1→v2 swap is invisible to daemons.
- **Bytes, not packets.** `tx_inflight` and `tx_window` both move to byte accounting.
- **Receiver-driven credit.** New subprotocol `SUBPROTOCOL_STREAM_WINDOW` (`0x0B00`) carries per-stream credit grants. Sender's window grows on grant, shrinks on send.
- **One counter, one source of truth.** The sender tracks a single `tx_credit_remaining` (bytes). Sends decrement it; receiver grants increment it. **ACKs do not touch flow control** — they're for reliable retransmission only. This avoids the double-counting trap where an ACK-based refund *plus* a grant-based top-up would inflate capacity beyond what the receiver actually has space for.
- **Implicit initial window.** Sender starts with a configured initial credit (propose 64 KB default). No handshake round-trip before the first send.
- **Observable stall cause.** `StreamStats.tx_credit_remaining: u32` exposes how close to zero the sender is.

## Non-goals

- **No SDK API changes.** Daemon code and `BackpressureError` / `NotConnectedError` classes stay identical. The helpers still back off on `Backpressure`.
- **No per-peer (as opposed to per-stream) credit pools.** Each stream has its own window; adding a peer-level aggregate would double the bookkeeping for marginal benefit. A misbehaving stream can starve others via the fair scheduler; that's the scheduler's job, not credit windows.
- **No signed/authenticated credit messages.** Credit grants ride existing encrypted sessions; AEAD covers integrity. A compromised peer session is already game-over.
- **No dynamic window-size negotiation at handshake.** Initial window is a configured constant; callers that want different values set `StreamConfig::window_bytes` at open time. If the peer wants a smaller window, its first `StreamWindow` grant sets the cap.
- **No TCP-style congestion control.** No slow start, no CUBIC, no RTT estimation for window sizing. The sender either has credit or doesn't; the receiver decides how fast to refund. This is a flow-control primitive, not a congestion-control one.
- **No renaming.** `window_bytes` stays `window_bytes`. The meaning matches the name after v2; the transition is in-place.

## Design

### 1. Units shift: packets → bytes

`tx_inflight: AtomicU32` stops counting packets and starts counting bytes. `tx_window` likewise. `TxSlotGuard` now acquires `bytes` credit on admission; its Drop still releases, but releases the same `bytes` it acquired.

`try_acquire_tx_slot` becomes `try_acquire_tx_credit(bytes: u32)` — the CAS loop compares `cur + bytes <= tx_window` instead of `cur < tx_window`. The admission rule is otherwise identical.

Back-compat: a stream opened with `window_bytes = 0` still means "unbounded" (pre-backpressure behavior). Callers who migrate from v1 and had `window_bytes = 256` (meant as "256 packets") get a 256-byte window and will hit backpressure immediately on anything bigger than one small packet. **Document this in the v2 changelog**; callers need to recalibrate. Default shifts from `0` (unbounded, v1) to a reasonable byte value (propose 65536).

### 2. Wire: `SUBPROTOCOL_STREAM_WINDOW` = `0x0B00`

Grant message — fixed-size, no length prefixes:

```rust
pub const SUBPROTOCOL_STREAM_WINDOW: u16 = 0x0B00;

/// Receiver → sender credit grant. Additive: each arrival bumps
/// the sender's `tx_window` by `credit_bytes` (saturating at u32::MAX).
pub struct StreamWindow {
    pub stream_id: u64,
    pub credit_bytes: u32,
}
```

Wire layout: `u64 stream_id LE` + `u32 credit_bytes LE` = 12 bytes. Rides encrypted per-peer sessions via the existing subprotocol dispatch path. No new flags, no new routing header bits.

`SUBPROTOCOL_STREAM_WINDOW` is registered in `SubprotocolRegistry::with_defaults`. Handlers are wired alongside the channel-membership dispatch in `MeshNode`.

### 3. One-counter accounting

The whole flow-control state at the sender is a single atomic: the
number of bytes the sender is currently allowed to send before
hitting Backpressure. No `tx_window` / `tx_inflight` split.

```rust
/// Bytes the sender may still send on this stream without
/// Backpressure. Decremented on each send, incremented on each
/// `StreamWindow` grant from the receiver. Starts at
/// `initial_credit_bytes` (implicit initial window, no handshake
/// — see §5).
tx_credit_remaining: AtomicU32,
```

`StreamState` retains the v1 field names for back-compat at the
stats-snapshot boundary, but the meaning changes: `tx_window` in
v2 exposes the *current* credit-remaining value, not a configured
cap. (The "cap" concept goes away — the receiver decides how much
credit to extend.)

Why one counter, not two:

- **Grants AND ACK-refund is double-counting.** If an ACK refunds
  byte credit *and* a grant adds more byte credit for the same
  bytes being consumed by the receiver, the sender's effective
  capacity grows by 2× on every round trip — defeats the whole
  point of the window. The bug is subtle: each halves look
  sound ("ACK means the data is gone from the network,"
  "grant means the receiver has room") but together they
  inflate capacity.
- **ACKs belong to reliability, not flow control.** `Reliable`
  streams use ACKs to drive retransmit. Credit is driven by
  `StreamWindow` grants, which the receiver emits only when it
  has actually made room (via its consume cadence below). That
  boundary is clean: flow control is the receiver's choice
  surfaced as a grant; retransmit is the sender's concern
  surfaced as an ACK. No field is touched by both.

### 4. Receive-side: threshold-emit cadence

The receiver tracks how much credit it has *extended* vs how much
it has *consumed*:

```rust
pub(crate) struct RxCreditState {
    /// Total credit granted to this sender since stream open,
    /// including the implicit initial window.
    granted: AtomicU64,
    /// Total bytes consumed off the inbound queue (drained by the
    /// daemon via `poll_shard`, forwarded by the scheduler, etc.).
    consumed: AtomicU64,
    /// Per-grant chunk size.
    window_bytes: u32,
}
```

Invariant: `consumed <= granted`. Outstanding (what the sender
has "used") is `granted - consumed`. When `consumed` catches up
enough that the sender is within `window_bytes / 2` of exhaustion,
the receiver emits a `StreamWindow { credit_bytes: window_bytes }`
and bumps `granted`.

Concretely: after each `poll_shard` / scheduler dequeue, check
whether `granted - consumed <= (window_bytes / 2).max(1)`. If so, emit a
grant. Amortizes control traffic — for a 64 KB window, roughly one
grant per 32 KB consumed, not per-packet.

### 5. Send-side: decrement on send, increment on grant

`send_on_stream` passes `packet.len()` (bytes) to the acquire:

```rust
let needed = packet_bytes_for(events);
match session.try_acquire_tx_credit_matching_epoch(
    stream_id,
    stream.epoch,
    needed,
) {
    TxAdmit::Acquired(guard) => { /* build + send + drop(guard) */ }
    TxAdmit::WindowFull => return Err(StreamError::Backpressure),
    TxAdmit::StreamClosed => return Err(StreamError::NotConnected),
}
```

`try_acquire_tx_credit_matching_epoch` does a CAS loop:

```rust
loop {
    let cur = state.tx_credit_remaining.load(Acquire);
    if cur < needed {
        state.backpressure_events.fetch_add(1, Relaxed);
        return TxAdmit::WindowFull;
    }
    if state.tx_credit_remaining
        .compare_exchange_weak(cur, cur - needed, AcqRel, Acquire)
        .is_ok()
    {
        break;
    }
}
```

The `TxSlotGuard` shape stays (drop-on-cancellation is still
valuable), but it holds the acquired byte count and on Drop
refunds that count back to `tx_credit_remaining` **only if the
send never actually happened** (early return, future cancelled
before socket send). Successful sends don't refund on Drop — the
bytes are now the receiver's to credit.

On `StreamWindow` receipt:

```rust
state
    .tx_credit_remaining
    .fetch_add(grant.credit_bytes as u32, AcqRel);
```

Saturating; a pathological receiver sending billions of tiny
grants can't wrap.

**`Reliable` streams** use their existing ACK path unchanged — no
credit side-table, no `inflight_by_seq`. Retransmission of a lost
packet does NOT re-acquire credit (the bytes were debited once,
at the original send; the retransmit flies under the same
accounting). If `Reliable` gives up after max retries, the credit
stays consumed until the receiver grants more — honest, since
the receiver might still have the original copy buffered.

**`FireAndForget` streams** follow the same rules. No credit side-table, no ACK coupling; sends just decrement the counter. The receiver's `RxCreditState` tracks `consumed` based on packets it actually received (including duplicates — dedup happens above this layer). If packets are dropped on the wire, the receiver under-consumes and the sender's credit exhausts faster than data actually arrived — but that's exactly the "my receiver can't keep up" signal Backpressure should surface.

### 6. Initial window

On stream open, the sender assumes an **implicit initial window**
of `StreamConfig::window_bytes` bytes — no handshake, no
receiver-sent first grant. `tx_credit_remaining` starts at
`window_bytes` (or a configured `initial_credit_bytes` that may
differ from subsequent grant sizes). The receiver does **not**
emit a proactive `StreamWindow` on open; that would be redundant
with the implicit initial window and waste a round trip.

If the receiver's `window_bytes` is smaller than the sender's
initial-window assumption, the sender may overshoot before the
first grant arrives. That's recoverable: the sender will
`Backpressure` soon after, and subsequent sends wait for grants.
No protocol-level disagreement — just a short burst past the
receiver's preferred size on the first round.

If the receiver never consumes (partitioned, frozen daemon,
etc.), the sender stalls at `tx_credit_remaining = 0`.
`send_with_retry` / `send_blocking` back off up to their retry
cap. Same shape as v1's partitioned-peer stall — honest.

### 7. Stream close reconciliation

On `close_stream`:
- Sender drops `tx_credit_remaining`; there's no `inflight_by_seq` or separate window cap, so cleanup is trivial.
- Receiver stops emitting `StreamWindow` grants and drops `RxCreditState`.
- A late `StreamWindow` grant for a closed stream is dropped silently (the stream isn't in `sessions.streams` → dispatch no-ops).
- The RAII `TxSlotGuard` epoch check (already in place) handles any in-flight sender-side sends whose guard tries to refund against a stream that was closed mid-send.

Nothing new here; the epoch guard + existing close path covers it.

### 8. Stats

Extend `StreamStats`:

```rust
pub struct StreamStats {
    // ...existing fields...
    /// Current remaining send credit in bytes. `0` = next send will
    /// be Backpressure; near tx_window = plenty of headroom.
    pub tx_credit_remaining: u32,
    /// Cumulative `StreamWindow` grants received (sender side).
    pub credit_grants_received: u64,
    /// Cumulative `StreamWindow` grants emitted (receiver side).
    pub credit_grants_sent: u64,
}
```

Observability wins: a daemon author seeing `tx_credit_remaining = 0` and `backpressure_events` climbing knows it's receiver-driven, not concurrent-caller-driven.

## Implementation steps

1. **Collapse `tx_window` / `tx_inflight` → `tx_credit_remaining`.** Single `AtomicU32` on `StreamState`. Sends do a CAS-loop subtract; grants do a saturating add. `tx_window` as a configured cap goes away — the receiver drives capacity. Back-compat: expose `tx_credit_remaining` through the existing `StreamStats.tx_window` field so callers who read stats don't break; the meaning shifts from "configured cap" to "current remaining credit."
2. **`StreamWindow` codec.** New `subprotocol/stream_window.rs` with `SUBPROTOCOL_STREAM_WINDOW = 0x0B00` and 12-byte encode/decode. Register in `SubprotocolRegistry::with_defaults`.
3. **Send-side credit update.** Dispatch handler for `SUBPROTOCOL_STREAM_WINDOW` calls `tx_credit_remaining.fetch_add(grant.credit_bytes, AcqRel)` with saturating semantics.
4. **Receive-side credit state.** `RxCreditState` per stream; `poll_shard` and the scheduler's forward path increment `consumed`; after each increment, if `granted - consumed < window_bytes / 2`, emit a `StreamWindow` grant and bump `granted`.
5. **`TxSlotGuard` adjustment.** Guard now refunds the acquired byte count on Drop **only when the send didn't happen** (early cancellation, etc.). Successful-send path consumes the drop without refund — add a `commit()` method on the guard that flags "the bytes are the receiver's now" and suppresses the Drop refund.
6. **Stats.** Add `credit_grants_received`, `credit_grants_sent`. Plumb through Rust/TS/Python stat accessors. `tx_credit_remaining` is the existing `tx_window` slot, repurposed.
7. **Docs.** Update `STREAM_BACKPRESSURE_PLAN.md` Status → "v1 shipped; v2 tracked in `STREAM_BACKPRESSURE_PLAN_V2.md`." Update `TRANSPORT.md` back-pressure section to say "catches both concurrent callers AND slow receivers via per-stream credit windows." Update SDK READMEs' Backpressure sections to drop the "v1 only catches concurrent callers" qualifier.
8. **Tests.**

## Tests

- **Unit (`session.rs`)** — CAS-loop decrement on send; saturating add on grant; `TxSlotGuard::commit()` suppresses refund on success; cancelled guard refunds; threshold-emit fires at 50% consumed on the receive side.
- **Unit (`subprotocol/stream_window.rs`)** — 12-byte round-trip; truncated-input rejected; garbage-tag rejected.
- **Unit (double-counting regression)** — a grant for N bytes and a "send completed" of N bytes must NOT both increase `tx_credit_remaining` for the same N. Drive a controlled sequence and assert invariants on the counter.
- **Integration (the v1 gap)** — single serial sender with a slow receiver. v1 never saw `Backpressure`; v2 does. Assert `backpressure_events > 0` and `tx_credit_remaining = 0` at peak.
- **Integration — grant flow.** Sender stalls on exhausted credit; receiver drains and emits grant; sender resumes. Measure RTT between grant emission and sender resumption.
- **Integration — partition recovery.** Partition sender from receiver mid-send. Sender stalls at `tx_credit_remaining = 0`. Unpartition; queued grants arrive; sender resumes.
- **Regression** — v1's concurrent-caller test still passes under byte-counted admission.

## Risks and open questions

- **Default `window_bytes` shift breaks v1 callers.** A caller who wrote `StreamConfig::new().with_window_bytes(256)` in v1 meaning "256 packets" gets "256 bytes" in v2 and sees immediate `Backpressure`. Migration: change the default in a bump-minor version; document in changelog + SDK READMEs. Callers that want the v1-style "unbounded" behavior set `with_window_bytes(0)`.
- **Receiver-side credit is per-stream, not per-peer.** A chatty stream can exhaust its own window without affecting other streams to the same peer — intentional. A misbehaving receiver that never drains one stream accumulates `granted - consumed` debt forever on that stream; the `RxCreditState` counters grow without bound (they're u64, so realistically fine, but the receiver will stop granting once the sender stalls — the counters don't force bytes to exist). No-op in practice; flagged for completeness.
- **Threshold-emit cadence is fixed at 50%.** A guess. Bursty drains may want 25% (more aggressive refill); steady workloads 75% (fewer grants). Start with 50%; add a `StreamConfig::credit_refill_threshold` knob only if workloads need it.
- **Credit grant loss.** `StreamWindow` rides the existing encrypted session. A grant lost on the wire means the sender waits longer than necessary for the *next* grant, which will include the same credit (grants are cumulative in the receiver's state — the sender sees one larger grant later rather than two smaller ones). No special retransmission needed; the worst case is latency, not deadlock. Document.
- **Interaction with `send_to_peer` / `send_routed` legacy paths.** These don't use `Stream` handles and don't go through the credit window. Leave them unchanged; credit windows are an opt-in feature of the typed `Stream` API. Document.
- **Stats field width.** `tx_credit_remaining: u32` caps at ~4 GB per-stream inflight credit. More than enough for any realistic workload.

## Summary

Same API, same SDK surface, same `BackpressureError`. Under the hood: bytes instead of packets, a 12-byte receiver→sender grant message, **one counter** (`tx_credit_remaining`) governing flow control — no double-counting with ACK-based refunds. The v1 "catches concurrent callers" caveat disappears from the READMEs. ~250 LoC core + ~100 LoC codec + ~150 LoC tests. Closes the remaining Backpressure item in the README Status.
