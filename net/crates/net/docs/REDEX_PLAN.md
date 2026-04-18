# RedEX — append-only streaming log on Net

## Status

Design only. RedEX does not exist yet.

Scope is intentionally tight: **v1 is a thin local slice** — append, tail, read_range, single node — that turns the mesh into a real-time event store without any replication, no dedicated control-plane subprotocol, no partition healing. Replication, the `SUBPROTOCOL_REDEX` control plane, cold-tier archival, and CortEX fold/query integration are tracked as v1.1+ increments below.

The frozen design decisions (20-byte records, inline+heap hybrid payloads, file = `ChannelName`, hot→warm tiering) carry through from v1 into v1.1+ unchanged. Shipping the slice first gives us a real primitive to fold CortEX onto; adding replication is additive, not a redesign.

## One-line frame

RedEX is a log-structured append-only event store that lives inside `MeshNode`. A "file" is a named monotonic log. Writes are fire-and-forget appends; reads are live tails. No broker, no commit, no consensus — v1 is one node, v1.1+ will replicate through Net's existing channel fan-out + causal-chain machinery.

## What Net already provides (RedEX is glue, not greenfield)

- **`ChannelName` + `ChannelConfig`** — hierarchical named endpoints with per-channel policy (`docs/CHANNELS.md`). RedEX files map 1:1 to channel names.
- **`ChannelPublisher` + `SubscriberRoster` + `SUBPROTOCOL_CHANNEL_MEMBERSHIP`** — the fan-out primitive. v1.1 replication will layer on this.
- **`CausalEvent` + `CausalLink`** — 24-byte causal chain with origin, sequence, parent_hash. RedEX records are a 20-byte projection of this.
- **Stream multiplexing + back-pressure** — RedEX tail subscribers will use `Stream` with `Reliability::Reliable` once remote tailing lands in v1.1.
- **Partition healing / `SUBPROTOCOL_PARTITION` / `SUBPROTOCOL_RECONCILE`** — reused by v1.1+ replica reconciliation; v1 has no remote to diverge from.
- **Subprotocol dispatch + registry** — reserves `SUBPROTOCOL_REDEX = 0x0C00` for the v1.1 control plane. Not used in v1.
- **`AuthGuard`** — wire-speed channel ACL check. v1 applies this on local `append` and `tail` even without remote peers so the ACL surface is consistent when replication turns on.

None of this needs to be built from scratch. RedEX is the glue.

---

## v1 scope — the local slice

What ships in v1:

- `RedexFile` handle with `append`, `tail`, `read_range`, `close`.
- In-memory index (`Vec<RedexEntry>`) + in-memory payload segment (`Vec<u8>`).
- One optional disk-backed segment (simple append-only file per file, no mmap, no rollout) for durability-required callers.
- Per-channel ACL enforcement on `append` / `tail` via the existing `AuthGuard`.
- Basic retention policy: count-based (keep newest K events) and size-based (keep newest M bytes of payload). Time-based lands in v1.1.
- Feature gate: `#[cfg(feature = "redex")]` on the core `net` crate.

What v1 does NOT do (by design, not oversight):

- No replication. `append` touches local state only.
- No `SUBPROTOCOL_REDEX` dispatch. No control-plane wire messages.
- No `RedexReplicaDaemon`. No remote tailing.
- No partition / conflict / forking logic.
- No cold-tier archive.
- No CortEX fold / NetDB query. Integration hook exists (§8) but lives in the CortEX adapter, not in RedEX.

The goal is a primitive that CortEX can build on. Distribution comes later, once the single-node shape is right.

---

## Design (frozen; carries through v1 → v1.1 → v2 unchanged)

### 1. The 20-byte event record

Every event occupies exactly 20 bytes of index space. Payloads live inline or in a separate segment (§2). Layout:

```rust
#[repr(C, packed)]
pub struct RedexEntry {
    /// Monotonic per-file sequence. Allocated by the local appender
    /// via an `AtomicU64`; never resets.
    pub seq: u64,                  // 8 bytes
    /// Byte offset into the file's payload segment. 32-bit gives 4 GB
    /// per live segment; larger files roll into the next segment when
    /// tiering lands (v1.1).
    pub payload_offset: u32,       // 4 bytes
    /// Payload length in bytes. `0` + `INLINE` flag means the payload
    /// rides in the record's own bytes (see below).
    pub payload_len: u32,          // 4 bytes
    /// High nibble: flags (INLINE, TOMBSTONE, COMPACTED, …).
    /// Low 28 bits: xxh3 truncation of the payload, for tamper/dedup.
    pub flags_and_checksum: u32,   // 4 bytes
}
// Total: 20 bytes.
```

**Inline payloads.** When `flags & INLINE != 0`, `payload_offset` and `payload_len` are reinterpreted as 8 inline payload bytes. Small structured events (sensor readings, tick counters) avoid the payload-segment indirection entirely.

**Cache-line geometry.** 20 bytes doesn't divide cleanly into 64-byte cache lines (3.2 records/line). Deliberate trade-off under the 20-byte spec: we pay ~20% per-access cache efficiency vs 16-byte records in exchange for carrying a payload locator in every record. 24-byte records would be denser at 2.67 records/line but lose record atomicity on some wider loads. 20 is the right sweet spot under the constraint.

### 2. Payload storage — inline + heap hybrid

Each file has one or more **segments**. A segment is a contiguous byte region addressed by `(segment_id, offset)`. Backing modes:

- **In-memory heap** (default for v1 files): `Vec<u8>` grown append-only.
- **Simple disk segment** (v1 opt-in): append-only file, no mmap, no rollover. Durability is an explicit caller opt-in via `RedexFileConfig::persistent: bool`. Writes fsync in the background (config: `sync_every: Duration`).

Records with the `INLINE` flag don't consume payload-segment bytes. Callers emitting small fixed-size events (8 bytes of inline capacity) hit zero per-event segment allocation.

**Tiering (v1.1):** hot heap → warm mmap → cold archive. Rollover triggers on `max_memory_bytes` per file; closed segments become immutable. Schema for §2's backing modes extends cleanly — v1's simple-disk-segment is a degenerate case of the v1.1 mmap tier.

### 3. Append semantics

```rust
pub struct RedexFile { /* handle */ }

impl RedexFile {
    /// Append one event. Returns the assigned sequence number.
    /// Fire-and-forget: the local index and payload segment are
    /// updated before return. Durability is background (if enabled).
    pub fn append(&self, payload: &[u8]) -> Result<u64, RedexError>;

    /// Append many events. Local write is atomic per-batch: all
    /// events land in the index segment contiguously or none do.
    pub fn append_batch(&self, payloads: &[Bytes]) -> Result<u64, RedexError>;
}
```

Sequences are assigned by the local appender from a per-file `AtomicU64`. No cross-node coordination on the write path — that's the "eventual consistency" axis, even though v1 has no remote peers to be eventual with. Under v1.1 replication, concurrent appends from different writers on the same file produce divergent local sequences that reconcile via Net's existing partition-healing machinery.

Write latency target: one CAS (seq allocate) + one memcpy (payload to segment) + one store (index record) = **tens of nanoseconds** for inline payloads, low hundreds for heap payloads. No locks, no syscalls on the hot path (durability is async).

### 4. Subscribe / tail semantics — streaming-first

```rust
impl RedexFile {
    /// Tailing subscription. Receives every event from `from_seq`
    /// onward. Backpressure flows through the `Stream` API (v1.1 for
    /// remote tailers; v1 is local-only, so the channel is effectively
    /// unbounded — slow local subscribers drift their offset).
    pub async fn tail(
        &self,
        from_seq: u64,
    ) -> impl Stream<Item = Result<RedexEvent, RedexError>>;

    /// One-shot read of [start, end). Second-class by design; bounded
    /// to the hot tier. Cold reads (v2) require an explicit opt-in.
    pub async fn read_range(&self, start: u64, end: u64) -> Vec<RedexEvent>;
}
```

**No consumer offsets.** Subscribers pass `from_seq` on every `tail()` and track their own position. RedEX remembers nothing about who has read what. Matches `poll_shard`; avoids Kafka-style consumer-group state.

**No ack, no commit.** The stream delivers events in sequence order; the caller observes them. Dropping the stream is the unsubscribe signal.

**Random access is second-class.** `read_range` exists but is bounded to the hot tier. Workloads that want random access over full history build an index on top (this is where the CortEX fold hook lands — §8).

### 5. File naming + ACL

A RedEX file name IS a `ChannelName`. Everything that applies to channels applies to files:

- Hierarchical naming (`/sensors/lidar/front`).
- Capability-gated append (`ChannelConfig::publish_caps`) and tail (`subscribe_caps`).
- Wire-speed `AuthGuard` bloom-filter check on every operation.
- Permission tokens (`PermissionToken`) work unchanged.

v1 applies ACL locally even without remote peers — keeps the surface consistent when replication turns on.

### 6. Retention

Per-file retention policy set at file creation. v1 ships count-based + size-based; time-based lands in v1.1:

- **Count-based:** keep newest K events.
- **Size-based:** keep newest M bytes of payload.
- **Time-based** *(v1.1)*: drop events older than N seconds.
- **Infinite:** never drop.

Retention runs as a background task on the heartbeat-loop tick; the next rollover cycle actually deletes old segments. No retention check on the read path.

---

## Deferred to v1.1+ (not built in v1, but scope-frozen so v1 doesn't paint a corner)

### 7. Replication (v1.1)

Replicas of a RedEX file are `RedexReplicaDaemon`s running on multiple nodes. They:

1. `subscribe_channel(publisher_node_id, channel_name)` on the file's channel.
2. Receive appended events via per-stream fan-out (`ChannelPublisher` → N unicasts). Every event carries a `CausalLink`.
3. Append locally, preserving the parent-hash chain.
4. On network partition: each replica continues to accept local appends with locally-assigned sequences. Divergent chains reconcile via the existing `SUBPROTOCOL_RECONCILE` (`0x0801`): longest chain wins, losers fork with a `fork_sentinel` marker, applications decide whether to follow the winner or keep the forked branch.

Conflict resolution is **forked, not merged.** Same shape as Net's existing continuity model (`docs/CONTINUITY.md`).

### 8. CortEX adapter integration hook (v1.1)

RedEX is a primitive; CortEX is the query layer. The integration is NOT in this plan — it lives in a separate `CORTEX_ADAPTER_PLAN.md` (to be written). What v1 pre-reserves:

- **`EventMeta` projection** — a thin struct carrying `(dispatch, origin_hash, seq_or_ts, flags, checksum)` that lines up 1:1 with `RedexEntry`. CortEX's `EventEnvelope` from Net arrives → project into `EventMeta` → append to RedEX → fold into in-memory state.
- **Fold hook** — a trait or closure `Fn(&RedexEvent, &mut State)` that CortEX installs per file. Called synchronously on the tail stream. Keeps the folded materialized view in lockstep with the log.
- **NetDB query surface** — Rust queries over the folded state; optional TS client that just calls the Rust query API. Not RedEX's concern; RedEX just exposes the folded state and the tail stream; the query shape is CortEX's call.

None of this is built in v1. The plan's contract is that v1's public API (`RedexFile::tail`) is expressive enough for CortEX to install its fold against when it lands.

### 9. SUBPROTOCOL_REDEX = 0x0C00 (v1.1)

Control-plane messages for file creation/close, replica seed request/response, retention updates. Same request/response-with-nonce shape as `SUBPROTOCOL_CHANNEL_MEMBERSHIP`. Event traffic itself rides existing `SUBPROTOCOL_CAUSAL`.

Reserved in the ID space now so v1's feature-flag introduction doesn't conflict.

### 10. Cold tier / archive (v2)

Compressed segments moved to a configured archive location; `read_cold(file, range)` is an explicit async fetch. Out of scope for v1 and v1.1. Landed as its own plan doc when the hot + warm tiers are battle-tested.

---

## Non-goals (all versions)

- **Strong consistency.** No consensus, no linearizability, no serializable transactions.
- **Random-access KV store.** RedEX is a log. If you want "give me event X by key", build an index (CortEX's job).
- **Query language.** No SQL, no pushdown filters.
- **Cross-file transactions.** Appending to two files atomically is not supported.
- **Replication quorum / `acks=all`.** Fire-and-forget writes.
- **Automatic consumer offsets.** Subscribers track their own position.
- **Schema registry.** Payloads are opaque bytes.
- **Log compaction by key** (Kafka compacted-topic style).

## v1 implementation steps

1. **`RedexEntry` codec** — 20-byte packed struct + `to_bytes` / `from_bytes`. Unit tests: round-trip, inline vs heap flag, checksum.
2. **`HeapSegment`** — `Vec<u8>` payload store with `append(payload) -> u32` returning offset. Unit tests: grow, bounds, capacity.
3. **`DiskSegment`** *(opt-in)* — append-only file with background fsync. Sub-feature `redex-disk`.
4. **`RedexFile` core** — per-file `AtomicU64` seq, `Vec<RedexEntry>` index, segment handle. `append` and `append_batch`.
5. **`RedexFile::tail`** — poll the index for `seq >= from_seq`, emit `RedexEvent { seq, payload }`. Async stream using a `Notify` or channel for new-event wake-up.
6. **`RedexFile::read_range`** — bounded scan of the in-memory index.
7. **Retention background task** — count-based + size-based eviction on the heartbeat tick.
8. **ACL plumbing** — wire `AuthGuard::check_fast` into `append` and `tail` so remote-ready ACL is exercised from day one.
9. **Docs** — `REDEX.md` (caller-facing contract). Link from `TRANSPORT.md` and `CHANNELS.md`.

## v1 tests

- **Unit**: 20-byte codec round-trip; `HeapSegment` append + read; retention eviction; INLINE vs heap payload decoding; ACL denial on append / tail.
- **Single-node integration**: open a file; append 10k events; tail from `seq=0` on a spawned task; assert every event is received in order with matching payload and checksum. Repeat with INLINE-only payloads to exercise the zero-segment-allocation path.
- **Durability** *(with `redex-disk`)*: append, crash-simulate (drop handle without explicit close), reopen, assert recovered index matches pre-crash minus unsynced tail.
- **Retention**: append beyond count cap; assert oldest events drop on the sweep; assert `read_range` over evicted range returns the expected short-read.
- **Benchmark**: append throughput (events/sec) for inline vs heap payloads; tail latency (append → subscriber observes).

## Risks and open questions

- **"Filesystem" framing vs reality.** RedEX has no POSIX semantics — no seek, no truncate, no file descriptors in the OS sense. Name stays (user spec) but the docstring is precise about what it isn't. Consider a "log store" or "event log" secondary label in callsite docs.
- **20 vs 24 bytes.** 24 aligns with `CausalLink` + 64-byte cache lines (2.67 records/line). 20 is the user's call; the ~20% cache-efficiency cost is documented.
- **Single-node before replication.** The v1 slice doesn't exercise Net's channel fan-out / causal / partition paths — those light up in v1.1. Risk: v1's API contract doesn't match what replication needs. Mitigation: freeze the design now (this plan) so v1.1 adds replication without reshaping `RedexFile`.
- **CortEX coupling.** The fold/query hook is mentioned but not specified here. The CortEX adapter plan owns that contract; RedEX just commits to "`tail` is expressive enough to fold against." If CortEX grows a requirement that needs a shape change (e.g., back-pressure on the fold), we revisit — but unlikely.
- **Name collision.** RedEX and CortEX both use the `EX` suffix. Keep the aesthetic; callsite docs should disambiguate (RedEX = storage primitive, CortEX = control/query plane).
- **Durability semantics.** Fire-and-forget loses the tail on crash by default. Opt-in `sync_on_append` is available (blocks on fsync). Most RedEX workloads (telemetry, sensor logs) don't need this. Document prominently.
- **Ordering across INLINE and heap payloads.** Both paths go through the same `AtomicU64` seq allocator → one monotonic sequence per file regardless of payload mode. No ordering hazard; documented for the record.

## Summary

v1 is the smallest possible slice: `RedexFile` + `append` + `tail` + `read_range`, in-memory by default, optional simple disk segment, file-as-channel ACL surface, count/size retention. One node. No replication, no subprotocol, no partition healing, no cold tier, no CortEX fold. All those live in v1.1+ and are scope-frozen here so v1 doesn't paint us into a corner.

The 20-byte record layout, inline+heap hybrid payload, and channel-name-as-file choices are frozen across versions. Ship the local slice; add replication once the single-node shape is right; add CortEX fold once RedEX is real.
