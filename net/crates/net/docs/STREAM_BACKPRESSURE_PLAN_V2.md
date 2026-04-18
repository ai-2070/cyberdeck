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
- **Credit refund policy per reliability mode.** `Reliable` streams refund on ACK (uses the existing reliability infrastructure). `FireAndForget` refunds optimistically on socket-send success plus a recovery timer for lost in-flight bytes.
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

### 3. Receive-side: credit budget + threshold-emit cadence

Each stream gains receive-side state on `NetSession`:

```rust
pub(crate) struct RxCreditState {
    /// Total credit the receiver has granted this sender since stream open.
    granted: AtomicU64,
    /// Total bytes consumed off the inbound queue (drained by the daemon
    /// via `poll_shard`, forwarded by the scheduler, etc.).
    consumed: AtomicU64,
    /// Configured initial / per-grant chunk size.
    window_bytes: u32,
}
```

The invariant: `consumed` never exceeds `granted`. The **outstanding** credit the receiver has extended is `granted - consumed`. When `consumed` catches up (crosses 50% of `window_bytes`), the receiver emits a `StreamWindow { credit_bytes }` to refill.

Threshold-emit amortizes control traffic: for a 64 KB window, the receiver sends a grant roughly every 32 KB consumed, not once per packet. Cadence tuning is a per-stream concern; v2 ships one policy (50% refill threshold), knob comes later if needed.

On **open**: the receiver emits one `StreamWindow { credit_bytes: window_bytes }` proactively so the sender has credit before it asks. Alternative "implicit initial window" (sender assumes `initial_credit_bytes` without a grant) is cleaner from a latency standpoint — chosen, see §6.

### 4. Send-side: credit accounting

`send_on_stream` now passes `packet.len()` (bytes) to the acquire call:

```rust
let needed = packet_bytes_for(events);
match session.try_acquire_tx_credit_matching_epoch(stream_id, stream.epoch, needed) {
    TxAdmit::Acquired(guard) => { /* build + send + drop(guard) */ }
    TxAdmit::WindowFull => return Err(StreamError::Backpressure),
    TxAdmit::StreamClosed => return Err(StreamError::NotConnected),
}
```

On `StreamWindow` receipt, the sender's dispatch bumps `tx_window` additively:

```rust
state.tx_window.fetch_add(grant.credit_bytes, AcqRel);
```

`tx_window` is now `AtomicU32` (was plain `u32` — v1 treated it as immutable once set). Saturating add; a pathological receiver sending billions of tiny grants can't wrap.

### 5. Credit refund: per-reliability policy

**`Reliable` streams.** The existing `ReliabilityMode` handles ACKs. When an ACK for seq N arrives, refund the byte count associated with seq N. Needs a small side-table on `StreamState`:

```rust
/// Bytes-in-flight attributed to each unacknowledged sequence number.
/// Capped via tx_window, so bounded size. Cleared on ACK receipt.
inflight_by_seq: DashMap<u64, u32>,
```

On send (after acquire): insert `(seq, bytes)`. On ACK for seq N: `inflight_by_seq.remove(&N)` and `tx_inflight.fetch_sub(bytes)`. On retransmission: no new acquire — the credit is already held. On give-up / max-retries: refund and evict.

**`FireAndForget` streams.** No ACKs. Two-part policy:

- **Optimistic refund on socket-send success.** The kernel accepted the packet; if it gets dropped on the wire, the receiver never credits it back, but the sender has already moved on — the `TxSlotGuard` releases immediately on socket-send success. This matches v1 behavior and means FireAndForget doesn't actually consume cross-peer credit long-term. Practically, FireAndForget streams don't need credit accounting at all — they're always at `tx_inflight = 0` between sends.
- **Acknowledgment of that reality.** For `FireAndForget`, the v2 window check is effectively a no-op (the receiver's unread events pile up in its `SegQueue`, but there's no credit-based pushback). That's a known limitation; FireAndForget callers accept loss as the norm, and if they want credit-based pushback they should use `Reliable`. Document this.

### 6. Initial window

On stream open, the sender assumes an **implicit initial window** of `StreamConfig::window_bytes` bytes — no handshake, no waiting for the receiver's first grant. The sender can start sending immediately. As it sends, `tx_inflight` grows; when it crosses `tx_window`, subsequent sends get `Backpressure` until a `StreamWindow` grant arrives.

What if the receiver never opens / never consumes? The sender will stall at `tx_inflight >= tx_window`. `send_with_retry` / `send_blocking` will back off indefinitely (bounded at 4096 retries). That matches the current v1 behavior for partitioned peers — the stall is honest.

What if the receiver's `window_bytes` is *smaller* than the sender's? The first `StreamWindow` grant will carry the receiver's smaller value; subsequent grants will too. The sender's effective window converges down. If the sender has already burned through the initial window's worth of bytes, it'll hit `Backpressure` and wait for the smaller grants to trickle in. No protocol-level disagreement.

### 7. Stream close reconciliation

On `close_stream`:
- Sender drops all outstanding credit state (`tx_window`, `tx_inflight`, `inflight_by_seq`).
- Receiver stops emitting `StreamWindow` grants.
- A late `StreamWindow` grant for a closed stream is dropped silently (the stream isn't in `sessions.streams` → dispatch no-ops).
- The RAII `TxSlotGuard` epoch check (already in place) handles any in-flight sender-side sends that try to release against a closed stream.

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

1. **Units shift.** Change `tx_inflight` / `tx_window` from "packets" to "bytes" semantics. Update `try_acquire_tx_slot` → `try_acquire_tx_credit(bytes)`. Update v1 tests to pass byte sizes. Bump default `window_bytes` to 65536.
2. **`StreamWindow` codec.** New `subprotocol/stream_window.rs` with `SUBPROTOCOL_STREAM_WINDOW = 0x0B00` and 12-byte encode/decode. Register in `SubprotocolRegistry::with_defaults`.
3. **Receive-side credit state.** `RxCreditState` per stream; `poll_shard` and the scheduler's forward path increment `consumed`; threshold check emits `StreamWindow`.
4. **Send-side credit update.** `tx_window` becomes `AtomicU32`; dispatch handler for `SUBPROTOCOL_STREAM_WINDOW` bumps it.
5. **`Reliable` refund.** `inflight_by_seq` side-table; hook into `ReliabilityMode`'s ACK callback to release credit.
6. **`FireAndForget` policy.** Document as "credit check is effectively a no-op"; guard releases on socket-send success (unchanged from v1 shape).
7. **Stats.** Add `tx_credit_remaining`, `credit_grants_received`, `credit_grants_sent`. Plumb through Rust/TS/Python stat accessors.
8. **Docs.** Update `STREAM_BACKPRESSURE_PLAN.md` → "Status: shipped, v2 in `STREAM_BACKPRESSURE_PLAN_V2.md`". Update `TRANSPORT.md` back-pressure section to say "catches both concurrent callers AND slow receivers via per-stream credit windows." Update SDK READMEs' Backpressure sections to drop the "v1 only catches concurrent callers" qualifier.
9. **Tests.**

## Tests

- **Unit (`session.rs`)** — byte-counted acquire/release; grant arrival bumps window; `inflight_by_seq` refund on ACK; threshold-emit fires at 50% consumed.
- **Unit (`subprotocol/stream_window.rs`)** — 12-byte round-trip; truncated-input rejected; garbage-tag rejected.
- **Integration (regression of v1 limitation)** — single serial sender with a slow receiver. v1 never saw `Backpressure`; v2 does. Assert `backpressure_events > 0` and `tx_credit_remaining = 0` at peak.
- **Integration — grant flow.** Sender stalls on exhausted credit; receiver drains and emits grant; sender resumes. Measure RTT between grant emission and sender resumption.
- **Integration — partition recovery.** Partition sender from receiver mid-send. Sender stalls at `tx_inflight >= tx_window`. Unpartition; queued grants arrive; sender resumes.
- **Regression** — v1's concurrent-caller test still passes under byte-counted admission.

## Risks and open questions

- **Default `window_bytes` shift breaks v1 callers.** A caller who wrote `StreamConfig::new().with_window_bytes(256)` in v1 meaning "256 packets" gets "256 bytes" in v2 and sees immediate `Backpressure`. Migration: change the default in a bump-minor version and require callers to opt into the old behavior with `with_window_bytes(0)`. Document in changelog + SDK READMEs.
- **Receiver-side credit is per-stream, not per-peer.** A chatty stream can exhaust its own window but doesn't affect other streams to the same peer. That's intentional — per-stream isolation is the point of streams. But it does mean a misbehaving receiver that never drains one stream can accumulate `granted - consumed` debt on that stream forever; the receiver's `RxCreditState` grows without bound. Mitigation: receiver-side `max_outstanding_credit` cap (stop granting once exceeded). Flag for implementation.
- **Threshold-emit cadence is fixed.** 50% refill is a guess. Workloads with bursty drains may want 25% (more aggressive refill), steady workloads 75% (fewer grants). Start with 50%; add a `StreamConfig::credit_refill_threshold` knob only if needed.
- **`FireAndForget` credit is a no-op.** Honest answer — FireAndForget doesn't benefit from v2 credit windows. Callers that need pushback use `Reliable`. Document; don't paper over.
- **Credit grant loss.** If a `StreamWindow` message is dropped on the wire (unreliable transport for control messages would be a design error — they ride encrypted sessions, which are reliable only in the "Reliable" reliability mode). The `StreamWindow` subprotocol must go on a `Reliable` stream to the peer, or its own subprotocol-reliability layer. Simplest: encode `SUBPROTOCOL_STREAM_WINDOW` traffic on a dedicated reliable-mode stream per peer. TBD during implementation.
- **Interaction with `send_to_peer` / `send_routed` legacy paths.** These don't use `Stream` handles and don't go through the credit window. Leave them unchanged; credit windows are an opt-in feature of the typed `Stream` API. Document.
- **Stats field width.** `tx_credit_remaining: u32` caps at ~4 GB per-stream inflight. More than enough for any realistic workload.

## Summary

Same API, same SDK surface, same `BackpressureError`. Under the hood: bytes instead of packets, a 12-byte receiver→sender grant message, one refund policy per reliability mode. The v1 "catches concurrent callers" caveat disappears from the READMEs. ~300 LoC core + ~100 LoC codec + ~150 LoC tests. Closes the remaining Backpressure item in the README Status.
