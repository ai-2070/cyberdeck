# Stream Multiplexing — fill in the gaps

## Status

Design only. The transport-layer plumbing for per-stream state is already in place (sequence counters, reliability mode, fairness scheduling, AEAD binding). The gaps are all above that plumbing: there's no public API for callers to pick a stream, no lifecycle for stream state (it leaks forever under churn), no per-stream flow control, and no exposed statistics. This plan closes those gaps without touching wire format or the existing per-stream machinery.

## What already works

- **Per-stream TX/RX sequence counters** in `StreamState` (`session.rs`), independent across streams.
- **Per-stream reliability** — either `FireAndForget` or `ReliableStream`, set at stream creation.
- **Per-stream fair scheduling** — `FairScheduler` in `router.rs` does round-robin with a 16-packet quantum across streams, preventing one stream from starving the rest.
- **AEAD binds `stream_id`** via `aad[20..28]` in `protocol.rs`, so a relay swapping packets between streams in flight is detected.
- **xxh3 helpers** (`stream_id_from_bytes`, `stream_id_from_key`) for deterministic stream-id derivation.

The whole receive path and the fairness layer assume streams already exist and are isolated — this plan inherits that. No changes to the wire header, AEAD binding, or the scheduler internals.

## Gaps this plan fills

1. **No caller-facing API to choose a stream.** Today `MeshNode::send_to_peer` takes a `Batch` and sets `stream_id = batch.shard_id as u64`. A caller that wants to send two categories of traffic with different reliability modes, independent HOL blocking, or controlled fairness weight has no way to express that at the public API.

2. **Stream state leaks.** `NetSession::get_or_create_stream` springs a `StreamState` into existence on first use and never removes it. A long-lived session that sees N unique stream ids over its lifetime grows to N `StreamState` entries forever. This is a memory leak under any workload that uses hashed stream ids with high cardinality.

3. **No per-stream flow control.** The only backpressure at the transport layer is a global router queue depth. A runaway stream can fill the scheduler's buffer and evict quanta from every other stream until the fair scheduler catches up.

4. **No exposed per-stream statistics.** The router tracks bytes/packets/drops per stream internally; there's no public accessor, so callers can't see which stream is misbehaving.

5. **Under-specified semantics.** "Different streams can have different reliability modes" is true in the code but nowhere documented. "No inter-stream ordering guarantee" is also true but implicit. Callers who read the code have to reverse-engineer the contract.

## Goals

- Public `Stream` API on `MeshNode` that lets callers pick stream ids and reliability modes per call.
- Stream lifecycle: explicit open + close, idle eviction, bounded `StreamState` map size.
- Per-stream flow control (credit-based window) so one stream's queue pressure doesn't bleed into others.
- Public per-stream statistics.
- Clear documented contract: ordering within a stream, no ordering across streams, reliability per stream.

## Non-goals

- Wire-format changes. `stream_id: u64` stays exactly where it is.
- Stream IDs with protocol meaning. Ranges like "0..1024 reserved for system streams" are **out**; stream IDs are caller-chosen opaque `u64` values. Subprotocol dispatch already has its own field.
- Streaming video / large-file transfer optimizations. The existing `StreamState` + fair scheduler are enough for the event-bus workload this crate targets.
- Multi-peer streams ("this stream exists across all of my peers"). Streams are scoped to a single session (one peer).

## Design

### Public API — a `Stream` handle on `MeshNode`

```rust
/// A named logical stream within an encrypted session to a specific peer.
/// Multiple streams share one session and one Noise cipher, but have
/// independent sequence numbers, reliability mode, and flow-control
/// windows. Ordering is guaranteed *within* a stream and NOT across
/// streams.
pub struct Stream {
    peer_node_id: u64,
    stream_id: u64,
    // internal handles into NetSession / FairScheduler
}

impl MeshNode {
    /// Open (or reuse) a stream to a connected peer.
    ///
    /// Fails with `AdapterError::Connection` if there is no session to
    /// `peer_node_id`. Subsequent `send` calls on the returned handle
    /// use the chosen `stream_id` and `StreamConfig`. Repeated `open`
    /// calls for the same `(peer, stream_id)` are idempotent — they
    /// return a handle backed by the same underlying `StreamState`.
    pub fn open_stream(
        &self,
        peer_node_id: u64,
        stream_id: u64,
        config: StreamConfig,
    ) -> Result<Stream, AdapterError>;

    /// Close a stream: mark it inactive, drain or drop pending packets
    /// per `StreamConfig.close_behavior`, remove the `StreamState`
    /// entry from the session. Idempotent.
    pub fn close_stream(&self, peer_node_id: u64, stream_id: u64);
}

pub struct StreamConfig {
    pub reliability: Reliability,        // FireAndForget | Reliable
    pub window_bytes: u32,                // per-stream flow-control budget
    pub fairness_weight: u8,              // scheduler quantum multiplier, default 1
    pub close_behavior: CloseBehavior,    // DrainThenClose | DropAndClose
}
```

`Stream` exposes `send(&self, events: &[Bytes]) -> Result<(), StreamError>` and `poll(...) -> Option<StoredEvent>` (or a stream-scoped channel receiver). The implementation funnels into the existing per-stream machinery — nothing new at the transport layer, just a typed handle that carries the `(peer, stream_id)` pair and the caller's config.

### Backward compatibility with `send_to_peer` / `send_routed`

Today's convenience methods keep working unchanged. They internally open (or look up) a stream id derived from `batch.shard_id`, with reliability = session default. Callers that want per-stream control opt in by holding a `Stream` handle; callers who don't, don't.

### Stream lifecycle

**Open is idempotent.** Repeated `open_stream(peer, sid, cfg)` calls return handles to the same `StreamState`. `cfg` on the second and later calls is ignored (warn-log if it differs from the original).

**Close drops the `StreamState` entry.** `close_stream` removes the entry from both `NetSession::streams` and the router's per-stream queue. Once closed, the stream id can be re-opened fresh with new config. No ID reuse guarantees — the caller is responsible for making stream ids distinct across logical categories.

**Idle eviction.** `StreamState` gets a `last_activity: Instant` field (already trivially derivable from `tx_seq` / `rx_seq` updates but make it explicit). A periodic sweep on the heartbeat tick removes streams with `last_activity.elapsed() > stream_idle_timeout` (config, default 5× session timeout). Evicting a stream means calling `close_stream` internally.

**Hard cap.** `NetSession` grows a `max_streams: usize` config (default 4096). Exceeding the cap forces eviction of the least-recently-active stream before creating a new one. Loud tracing warning so deployments notice.

### Per-stream flow control

Credit-based window, symmetric to the reliability layer's NACK window but simpler (no retransmission):

- Each `StreamState` has `window_bytes: u32` (total) and `bytes_in_flight: u32` (current).
- `Stream::send` returns `StreamError::WouldBlock` if the send would exceed `window_bytes`. Caller waits / retries / drops per its own policy — no built-in blocking for now (keeps the API simple; backpressure is expressed, not absorbed).
- A `StreamWindow { stream_id, new_window_bytes }` control message on the reliability channel grants credit. Actually, this can piggyback on existing NACK / ACK logic for the reliable mode; for fire-and-forget there is nothing to grant back and the window is a pure TX-side budget.

**Alternative, simpler scheme** (if credit windows feel over-engineered for the workload): a pure per-stream TX budget (bytes queued at the scheduler). No in-flight accounting, no credit messages; `send` returns `WouldBlock` when the stream's router queue is full. This is arguably enough — the scheduler already bounds per-stream queue depth; surfacing that limit as a `StreamError` is 90% of the value of a full windowing scheme.

Recommendation: **ship the simpler scheme first**, add credit windows only if measured need arises. The plan is designed so swapping one for the other is a stream-internal change — the caller API doesn't move.

### Fairness weight

`FairScheduler::quantum` is currently a session-wide constant. Extend to a per-stream `quantum_multiplier: u8` so high-priority streams can get more packets per round. The scheduler already iterates streams with per-stream state; multiplying the quantum is cheap. Default = 1 = current behavior.

### Stream statistics

```rust
pub struct StreamStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_dropped: u64,  // scheduler drops + reliability drops
    pub retransmissions: u64,  // reliable mode only; 0 for FireAndForget
    pub current_window_bytes: u32,
    pub current_bytes_in_flight: u32,
    pub last_activity: Instant,
}

impl MeshNode {
    pub fn stream_stats(&self, peer_node_id: u64, stream_id: u64) -> Option<StreamStats>;
    pub fn all_stream_stats(&self, peer_node_id: u64) -> Vec<(u64 /*stream_id*/, StreamStats)>;
}
```

Counters already exist in the router's `StreamStats` struct and in `StreamState::reliability`. The plan is to expose them via the public accessors above.

### Documented contract

Add a "Streams" section to `docs/TRANSPORT.md` stating:

- A stream is a logical channel within an encrypted session to a single peer.
- Ordering is guaranteed in FIFO order **within** a stream.
- **No ordering across streams.** A later-sent packet on stream A may arrive before an earlier-sent packet on stream B.
- Reliability is chosen per stream at `open_stream`. A session can run fire-and-forget and reliable streams simultaneously.
- Stream IDs are opaque `u64`s. No range has reserved meaning at the transport layer. Callers derive IDs however they want (hash of a logical channel name is typical; `stream_id_from_key` is the helper).
- Closing a stream immediately stops delivery of buffered inbound events and drops outbound packets that haven't hit the wire. Durable send = "await completion before closing."

## Implementation

Each step is independently reviewable.

### Step 1: `StreamConfig` + typed `Stream` handle (~120 lines)

- New `src/adapter/net/stream.rs` module with `Stream`, `StreamConfig`, `Reliability`, `CloseBehavior`, `StreamError`.
- `MeshNode::open_stream` / `close_stream` delegating to `NetSession` under the hood.
- `Stream::send`, `Stream::poll` wiring to existing per-stream machinery.
- `Batch`-based `send_to_peer` unchanged — internally creates an implicit stream keyed by `batch.shard_id` with default config.

### Step 2: Lifecycle (idempotent open, close, idle eviction, cap) (~80 lines)

- `StreamState` gets `last_activity: AtomicInstant` (or `u64` nanos for lock-free updates).
- `NetSession::close_stream(stream_id)` removes the entry + router-side queue.
- Heartbeat-loop piggyback: sweep streams with `last_activity.elapsed() > idle_timeout`, up to the `max_streams` cap.
- `MeshNodeConfig` grows `stream_idle_timeout: Duration` and `max_streams: usize` with sane defaults.

### Step 3: Per-stream TX budget + `StreamError::WouldBlock` (~60 lines)

- Scheduler already has `max_queue_depth` per stream. Expose the queue's "full" state to callers via a `StreamError::WouldBlock` returned from `Stream::send`.
- No wire changes — purely local back-pressure signaling.
- Document that callers can choose to block, drop, or buffer on WouldBlock.

### Step 4: Public per-stream statistics (~50 lines)

- Unify `StreamStats` across `router::StreamStats` (what's there today) and the new state this plan adds.
- `MeshNode::stream_stats(peer, stream)` + `all_stream_stats(peer)` accessors.
- Expose `last_activity` from the lifecycle work.

### Step 5: Per-stream fairness weight (~40 lines)

- `FairScheduler` quantum becomes per-stream. Stored in the per-stream queue alongside the existing `max_depth`.
- `StreamConfig.fairness_weight` maps to `quantum_multiplier` on insert.
- Default multiplier = 1; no behavior change unless callers opt in.

### Step 6: Documentation (~30 lines in `docs/TRANSPORT.md`)

- New "Streams" section articulating the contract listed under "Documented contract" above.
- Reference to this plan for rationale; the public doc is the caller-facing contract only.

### Step 7: Tests

Unit:
- `open_stream` idempotency; second open returns handle to same underlying state.
- `close_stream` removes the entry; subsequent `open_stream` creates fresh state.
- Idle eviction evicts exactly the streams past the timeout, leaves fresh ones.
- `max_streams` cap forces LRU eviction.
- `Stream::send` with full router queue returns `WouldBlock`.
- `StreamStats` round-trip — send/receive some packets, read stats, values match.

Integration (`tests/three_node_integration.rs`):
- Two streams between same peers, one Reliable one FireAndForget; inject loss on one; reliable recovers, fire-and-forget doesn't, cross-stream ordering not assumed.
- Fairness weight: open two streams, one with weight 1, one with weight 4; pump packets; observe ~4:1 egress ratio at the scheduler.
- High-cardinality soak: open+close 10_000 unique stream ids on one session in a loop; assert session's `StreamState` map stays under `max_streams`.

## What stays the same

- Wire format: unchanged.
- AEAD AAD: unchanged (`stream_id` is already bound).
- Reliability implementation: unchanged; it's already per-stream.
- FairScheduler core algorithm: unchanged; the only extension is per-stream weight.
- Existing `send_to_peer` / `send_routed` / `send_subprotocol` signatures: unchanged.

## Scope

~380 lines across `stream.rs` (new), `session.rs`, `router.rs`, `mesh.rs`, and `config.rs`. ~200 lines of tests. ~30 lines of new docs. Each step is independently reviewable; none depends on another shipping first.

## Open questions

- **Backpressure: `WouldBlock` vs. async `send_blocking`.** The plan proposes `WouldBlock` and letting the caller manage the retry loop. Should `Stream::send` have an async flavor that awaits credit? Probably yes — but as a thin wrapper, not a different API.
- **Cross-stream priority at the AEAD / cipher level.** Today the session's one Noise cipher serializes all streams at the crypto layer. For very high throughput, a per-stream cipher (derived via HKDF from the session keys) would allow parallel encryption. Out of scope for this plan; flag as future work.
- **Stream close notification on the wire.** Currently closing a stream is purely local — the peer doesn't learn. For `ReliableStream`, a `FIN` flag on the last packet would be ergonomic (it's already in `PacketFlags::FIN` per `protocol.rs`). Worth wiring; small add.

The first two are deferred; the third (wire `FIN` on close) could reasonably slot in as Step 2.5 if it pops up during implementation review.
