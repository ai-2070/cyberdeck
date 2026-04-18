# RedEX — append-only streaming filesystem on Net

## Status

Design only. RedEX does not exist yet. This plan proposes a `cfg`-gated persistence + streaming layer that sits on top of Net's existing primitives — channels, streams, causal events, subscriber roster — and exposes a filesystem-shaped API over it. Eventual consistency is the correctness model; 20-byte fixed-size event records are the storage primitive; tailing subscribers are the primary read path.

## One-line frame

RedEX is a log-structured file server that speaks Net. A "file" is a named append-only log. Events in a file are addressed by monotonic sequence number. Writes are fire-and-forget appends; reads are live tailing. Persistence is tiered (hot mmap, cold rollout). Replicas reconcile via Net's causal-chain partition-healing machinery.

Think: Kafka's partition-as-log shape, collapsed to Net's latency envelope, with no broker, no offset commits, no zookeeper. The mesh *is* the broker.

## What Net already provides (RedEX is glue, not greenfield)

- **`ChannelName` + `ChannelConfig`** — hierarchical named endpoints with per-channel policy (`docs/CHANNELS.md`). RedEX files map 1:1 to channel names.
- **`ChannelPublisher` + `SubscriberRoster` + `SUBPROTOCOL_CHANNEL_MEMBERSHIP`** — the fan-out primitive. RedEX replicas subscribe to a file's channel; writes propagate via per-peer unicast (the existing daemon-layer fan-out).
- **`CausalEvent` + `CausalLink`** — 24-byte causal chain with origin, sequence, parent_hash. RedEX event records are a 20-byte projection of this.
- **Stream multiplexing + back-pressure** — RedEX subscribers open a `Stream` per file with `Reliability::Reliable`; `StreamError::Backpressure` flows through to readers that can't keep up.
- **Partition healing / `SUBPROTOCOL_PARTITION` / `SUBPROTOCOL_RECONCILE`** — divergent causal chains on two replicas reconcile via Net's existing mass-partition healing: longest chain wins, losers fork with documented lineage (`docs/CONTESTED.md`).
- **Subprotocol dispatch + registry** — RedEX claims `SUBPROTOCOL_REDEX = 0x0C00` for control-plane messages (open, close, retention, rollout). Event traffic itself rides the existing causal channel.

None of this needs to be built from scratch. RedEX is the glue that composes them into a filesystem shape.

## Design

### 1. The 20-byte event record

Every event in a RedEX file occupies exactly 20 bytes of index space. Payloads live elsewhere (see §2). Layout:

```rust
#[repr(C, packed)]
pub struct RedexEntry {
    /// Monotonic per-file sequence (never resets; wraps at u64::MAX).
    pub seq: u64,                  // 8 bytes
    /// Byte offset into the file's payload segment. 32-bit gives 4 GB
    /// per live segment — past that, compaction rolls into a new
    /// segment (see §5).
    pub payload_offset: u32,       // 4 bytes
    /// Payload length in bytes. `0` means inline: the payload IS the
    /// low bits of this record's extension (see below).
    pub payload_len: u32,          // 4 bytes
    /// High nibble: flags (INLINE, TOMBSTONE, COMPACTED, …). Low 28
    /// bits: xxh3 truncation of the payload, for cheap tamper/dedup
    /// detection on replay.
    pub flags_and_checksum: u32,   // 4 bytes
}
// Total: 20 bytes.
```

**Inline payloads.** When `flags & INLINE != 0`, `payload_offset` and `payload_len` are reinterpreted as 8 bytes of inline payload (plus the 4 flags_and_checksum bytes if the checksum slot is borrowed — call it 8 or 12 bytes of inline capacity). For small structured events (sensor readings, counters, tick timestamps) this avoids the payload-segment indirection entirely.

**Cache-line geometry.** 20 bytes doesn't divide cleanly into 64-byte cache lines (3.2 records/line). This is deliberate: the user spec'd 20 bytes, and the tradeoff is understood — we pay a ~20% per-access cache efficiency penalty versus 16-byte records in exchange for a meaningful payload locator in every entry. 24-byte records (aligning to 64-byte lines at 2.67 records/line) would actually be *worse* for density. 20 is the sweet spot under the constraint.

### 2. Payload storage — inline + heap hybrid

Each file has one or more **segments**. A segment is a contiguous byte region addressed by `(segment_id, offset)`. Two backing modes:

- **In-memory heap** (default for hot files): `Vec<u8>` grown append-only. Zero sync cost. Bounded by configured `max_memory_bytes` per file; overflow triggers rollout to disk.
- **Memory-mapped disk** (`mmap` for durability-required files): fixed-size segments (256 MB default) rotated when full. Append is a bump of a segment cursor; no fsync on the write path — durability is a background task with configurable lag.

Records with `INLINE` flag don't consume payload-segment bytes — the payload rides in the 20-byte record itself. Callers targeting small fixed-size events (8 or 12 bytes, depending on whether the checksum slot is claimed) hit zero per-event allocation cost.

### 3. Append semantics

```rust
pub struct RedexFile { /* handle */ }

impl RedexFile {
    /// Append one event. Returns the assigned sequence number.
    /// Fire-and-forget: the local index and payload segment are
    /// updated before return; replication to peers is async.
    pub fn append(&self, payload: &[u8]) -> Result<u64, RedexError>;

    /// Append many events. Local write is atomic per-batch (all
    /// events land in the index segment contiguously); replication
    /// is still async.
    pub fn append_batch(&self, payloads: &[Bytes]) -> Result<u64, RedexError>;
}
```

Sequences are assigned by the **local appender** at append time, derived from a per-file `AtomicU64`. No cross-node coordination on the write path — that's the "eventual consistency" half of the spec. Concurrent appends from different writers on the same file create divergent local sequences, reconciled on replication (§6).

Write latency target: one CAS (seq allocate) + one memcpy (payload to segment) + one store (index record) = **tens of nanoseconds** for inline payloads, low hundreds for heap payloads. No locks, no syscalls on the hot path.

### 4. Subscribe / tail semantics — streaming-first

```rust
impl RedexFile {
    /// Start a tailing subscription. Receives every event from `from_seq`
    /// onward. Backpressure via `Stream`'s existing `Backpressure`
    /// signal; slow subscribers see their tail drift.
    pub async fn tail(
        &self,
        from_seq: u64,
    ) -> impl Stream<Item = Result<RedexEvent, RedexError>>;

    /// Read a fixed range [start, end) as a one-shot batch. Second-class
    /// — explicit opt-in — and bounded to the hot tier; cold reads
    /// require a separate `read_cold` call.
    pub async fn read_range(&self, start: u64, end: u64) -> Vec<RedexEvent>;
}
```

**No consumer-offset tracking.** Subscribers pass their `from_seq` on every `tail()` call and track their own position. RedEX doesn't remember who has read what — matches Net's `poll_shard` pattern and avoids the Kafka-style consumer-group coordination state. If a subscriber wants durability of its own position, it writes the position into a RedEX file itself (turtles).

**No ack, no commit.** The stream delivers events in sequence order; the caller observes them. Dropping the stream is the unsubscribe signal. No protocol state accumulates at the publisher.

**Random access is second-class.** `read_range` exists but is bounded to the hot tier and explicitly slower. RedEX is a log, not a key-value store; workloads that want random access over the full history use a separate index (probably built by a daemon that tails RedEX).

### 5. Persistence tiers

Three tiers, all append-only:

- **Hot (in-memory).** The head segment of each file lives in RAM. All reads and writes on recent events hit this tier. Size cap per file (default 256 MB); when exceeded, trigger rollout.
- **Warm (mmap, disk-backed).** Closed segments are mmap'd from disk. `tail()` for old events reads from mmap; performance is page-cache dependent. Segments are immutable once closed.
- **Cold (archive).** Segments older than the retention threshold are compressed and moved to a configured archive location. `read_cold(file, range)` is an explicit opt-in with an async fetch.

Tiering is a background task. The write path never blocks on tier transitions.

### 6. Eventual consistency — reuse Net's partition machinery

Replicas of a RedEX file are daemons (`RedexReplicaDaemon`) running on multiple nodes. They:

1. Subscribe to the file's channel via `subscribe_channel(publisher_node_id, channel_name)`.
2. Receive appended events via per-stream fan-out (`ChannelPublisher` → N unicasts).
3. Append locally, preserving the CausalLink parent-hash chain from the event.
4. On network partition: each replica continues to accept local appends with locally-assigned sequences. Divergent chains are reconciled via the existing `SUBPROTOCOL_RECONCILE` (`0x0801`) machinery: longest chain wins, losers fork with a documented `fork_sentinel` marker, and applications decide whether to follow the winner or keep the forked branch.

Conflict resolution is **forked, not merged.** If two replicas both append to file `/sensors/lidar` during a partition, the healed file has two chains — the winner and a forked-off loser. Readers see the winner by default; the losing chain is reachable via an explicit fork-aware API for applications that care.

This matches Net's existing continuity model (`docs/CONTINUITY.md`); RedEX inherits it wholesale.

### 7. File naming + ACL

A RedEX file name IS a `ChannelName`. Everything that applies to channels applies to files:

- Hierarchical naming (`/sensors/lidar/front`).
- Capability-gated publish (`ChannelConfig::publish_caps`) — who can append.
- Capability-gated subscribe (`ChannelConfig::subscribe_caps`) — who can tail.
- Wire-speed `AuthGuard` bloom-filter check on every event.
- Visibility scopes (`SubnetLocal`, `ParentVisible`, `Exported`, `Global`) — a file can be private to a subnet or exported across the mesh.

Permission tokens (`PermissionToken` with `TokenScope::PUBLISH` / `SUBSCRIBE`) work unchanged. No new ACL surface.

### 8. Retention

Per-file retention policy, set at file creation:

- **Time-based:** drop events older than N seconds.
- **Size-based:** keep newest M bytes of payload.
- **Count-based:** keep newest K events.
- **Infinite:** never drop (for audit logs, compliance data).

Retention runs as a background task that marks segments as droppable; the next rollout cycle actually deletes them. No retention check on the read path.

## Wire format — `SUBPROTOCOL_REDEX = 0x0C00`

Event traffic itself does NOT use this subprotocol — it rides the existing `SUBPROTOCOL_CAUSAL` (`0x0400`) through `ChannelPublisher`. `SUBPROTOCOL_REDEX` carries **file-level control messages**:

```rust
pub enum RedexControl {
    /// Create a new file. `config` carries retention + segment size.
    CreateFile { name: ChannelName, config: RedexFileConfig, nonce: u64 },
    /// Close a file. Pending events flush; subscribers get EOS.
    CloseFile { name: ChannelName, nonce: u64 },
    /// Replica seed: request the current head segment for catch-up.
    SeedRequest { name: ChannelName, from_seq: u64, nonce: u64 },
    /// Replica seed response: one segment of history.
    SeedResponse { name: ChannelName, seq_range: Range<u64>, payload: Bytes, nonce: u64 },
    /// Ack for the above.
    Ack { nonce: u64, accepted: bool, reason: Option<AckReason> },
}
```

Same shape as `MembershipMsg` in `SUBPROTOCOL_CHANNEL_MEMBERSHIP` — request/response with a nonce. Reuses the same dispatch plumbing.

## Non-goals

- **Strong consistency.** No consensus, no linearizability, no serializable transactions. Writers never coordinate. This is the price of the latency target and is explicit in the name "eventual consistency."
- **Random-access KV store.** RedEX is a log. If you want "give me event X by some arbitrary key," build an index daemon on top.
- **Query language.** No SQL, no pushdown filters. Subscribers get every event in sequence; filtering is the caller's job (or a daemon's).
- **Cross-file transactions.** Appending to two files atomically is not supported. If you need it, put both payloads in one file.
- **Replication quorum / `acks=all`.** Fire-and-forget writes. Durability is the rollout task's responsibility, not the write path's. If you need write-ack semantics, you're on the wrong substrate — use a Reliable stream directly and build your own ack protocol.
- **Automatic consumer offsets.** Subscribers track their own position.
- **Schema registry.** Payloads are opaque bytes. Schema evolution is a caller concern.
- **Compaction beyond retention.** No log-compaction-by-key (Kafka compacted topic style). Retention drops old events; it doesn't coalesce by key.

## Implementation sketch — where the code lives

- **Gated behind `#[cfg(feature = "redex")]`** in the core `net` crate, submodule `net/crates/net/src/adapter/net/redex/`. If it grows past ~2 KLoC, graduate to a sibling crate `net-redex` in the workspace.
- New file structure:
  - `redex/file.rs` — `RedexFile` handle + `append` / `tail` / `read_range`.
  - `redex/entry.rs` — 20-byte `RedexEntry` record + codec.
  - `redex/segment.rs` — payload segment (heap + mmap backends).
  - `redex/retention.rs` — retention policy + background task.
  - `redex/replica.rs` — `RedexReplicaDaemon` (implements `MeshDaemon`).
  - `redex/control.rs` — `SUBPROTOCOL_REDEX` wire codec.
- `Cargo.toml`: `redex = ["net/redex"]` feature flag; `memmap2` dep (behind the feature).
- Public re-exports at `adapter::net::redex::{RedexFile, RedexConfig, RedexError}`.

## Implementation steps

1. **`RedexEntry` codec** — 20-byte packed struct + `to_bytes` / `from_bytes`. Unit tests: round-trip, inline vs heap flag, checksum.
2. **Segment backends** — heap-only first; mmap behind a sub-feature. `append(payload) -> u32` returning offset. Unit tests: grow, bounds, rollover.
3. **`RedexFile::append` + in-memory index** — `Vec<RedexEntry>` backing the index. Per-file `AtomicU64` sequence. Write path: allocate seq, write payload to segment, write record to index.
4. **`RedexFile::tail`** — subscriber-side stream that polls the index for `seq >= from_seq` and emits `RedexEvent { seq, payload }`. Backpressure via the existing `Stream::send_with_retry`-style helper shape.
5. **`SUBPROTOCOL_REDEX` codec + dispatch** — request/response nonce flow.
6. **`RedexReplicaDaemon`** — `MeshDaemon` impl that subscribes to a file's channel and appends locally on each received event. Uses Net's existing channel fan-out + causal-chain machinery.
7. **Retention background task** — runs on `MeshNode` heartbeat tick; per-file evaluation.
8. **Rollout (mmap + disk)** — sub-feature; landed as a second increment.
9. **Cold tier (archive)** — third increment; likely lands in a follow-up plan once the hot tier ships.
10. **Docs** — `REDEX.md` (caller-facing contract). Link from `TRANSPORT.md` and `CHANNELS.md`.

## Tests

- **Unit**: 20-byte codec round-trip; segment append + read; retention policy eviction; INLINE vs heap payload decoding.
- **Integration (three-node)**: A appends 10k events to `/bench`; B and C tail from seq=0; assert both receive all events in order. Partition A from B+C mid-stream; verify B+C keep receiving each other's appends; heal; assert reconciliation produces a single winning chain (or a documented fork).
- **Benchmark**: append throughput (events/sec), tail latency (append → subscriber observes), memory overhead per file.
- **Soak**: 1 M events into a file with 64 MB heap cap; verify retention rollover keeps memory bounded.

## Risks and open questions

- **Is "filesystem" the right framing?** It hints at POSIX-like semantics (open/read/write/close) that RedEX doesn't provide — there's no seek, no truncate, no random write, no file descriptors in the POSIX sense. Consider framing as "append log server" or "streaming log store" to avoid a false expectation. The user explicitly said "filesystem" so keep the name, but the docstring should be precise about what it isn't.
- **20 bytes vs 24.** 24-byte records align with `CausalLink` + cache lines (2.67 records/line still, but cleaner math). Staying at 20 is the user's call; flag the alignment cost.
- **Sequence space per file.** u64 per file means 1.8e19 events before wraparound — safe. If RedEX ever moves to global sequences (one monotonic across all files), this becomes a bottleneck; don't do that.
- **Fan-out scaling.** `ChannelPublisher` is O(subscribers) per append. A file with 1000 replicas = 1000 unicasts per event. Works for small replica counts; breaks past ~256. Mitigation: replica tree (primary replicates to K secondaries, each to K tertiaries, etc.) — explicitly out of scope for v1.
- **Disk durability guarantees.** "Fire-and-forget" writes mean a crash between memory append and mmap sync loses the tail. If that's unacceptable, callers opt into a `sync_on_append` mode that blocks on msync — 1–10 ms per append on typical SSDs. Most RedEX workloads (telemetry, sensor logs) don't need this.
- **Conflict-forking UX.** Forking a chain on partition is "correct" but surprises callers who assumed their append landed. Document prominently; surface fork events through a `RedexFile::fork_listener()` API for applications that care.
- **Name collision.** RedEX is a sibling name to CortEX. Both use the `EX` suffix; both sound like systems products. Keep the aesthetic consistent; the names should not be confused in practice because CortEX is a forward-looking control plane and RedEX is a storage primitive.

## Summary

RedEX is the smallest amount of storage + tailing glue that makes Net's channel-fan-out + causal-chain + partition-healing machinery behave like a log-structured file server. 20-byte index records, inline/heap hybrid payloads, tiered persistence, per-channel ACL reuse, subscription-based reads. Eventual consistency inherits from Net's existing continuity model; no new consensus primitive. `SUBPROTOCOL_REDEX = 0x0C00` carries control traffic only; event traffic rides the existing causal subprotocol. Gated behind `#[cfg(feature = "redex")]`; ships as a submodule of the core crate first, graduates to its own crate if it grows.

The design bet: log-structured storage at Net's latency envelope is a valuable primitive that doesn't exist today because nobody had a mesh transport fast enough for it to be worth building. Same bet as Mikoshi, same bet as the broader Net thesis.
