# [REDACTED] Implementation Plan

Concrete plan for completing live migration support — transferring a daemon's internal state, memory, connections, and queues from Node A to Node B.

---

## Architecture Overview

Six new modules, built on top of the existing `MigrationState` (6-phase state machine), `StateSnapshot`, `SuperpositionState`, `DaemonRegistry`, and `MeshDaemon` trait.

```
                  ┌─────────────────────────┐
                  │  MigrationOrchestrator   │  Coordinates all 6 phases
                  │  (controller node)       │
                  └────────┬────────────────┘
                           │
              MigrationMessage (0x0500)
                           │
            ┌──────────────┼──────────────┐
            ▼                             ▼
┌───────────────────────┐     ┌───────────────────────┐
│ MigrationSourceHandler│     │ MigrationTargetHandler │
│ (source node)         │     │ (target node)          │
│                       │     │                        │
│ snapshot() ──────────────────> restore()             │
│ buffer_event() ──────────────> replay_events()       │
│ on_cutover() ─────────────────> activate()           │
│ cleanup()             │     │                        │
└───────────────────────┘     └────────────────────────┘
```

---

## Workstream 1: Migration Orchestrator

**Goal:** High-level coordinator that sequences all 6 phases with timeouts and failure handling.

**Deliverable:** `src/adapter/net/compute/orchestrator.rs`

### Types

```rust
pub struct MigrationOrchestrator {
    /// In-flight migrations: daemon_origin -> MigrationState
    migrations: DashMap<u32, Arc<Mutex<MigrationState>>>,
    daemon_registry: Arc<DaemonRegistry>,
    router: Arc<NetRouter>,
    failure_detector: Arc<FailureDetector>,
    subprotocol_registry: Arc<SubprotocolRegistry>,
    pending_phases: DashMap<u32, MigrationPhaseWait>,
}

struct MigrationPhaseWait {
    phase: MigrationPhase,
    started_at: Instant,
    timeout: Duration,
    phase_channel: tokio::sync::oneshot::Receiver<Result<(), MigrationError>>,
}
```

### Migration Message Protocol

Wire messages sent over `SUBPROTOCOL_MIGRATION (0x0500)`:

```rust
pub enum MigrationMessage {
    /// Phase 0→1: Request snapshot on source
    TakeSnapshot { daemon_origin: u32, target_node: u64 },

    /// Phase 1→2: Snapshot taken, payload included
    SnapshotReady { daemon_origin: u32, snapshot_bytes: Vec<u8>, seq_through: u64 },

    /// Phase 2→3: Target restored daemon from snapshot
    RestoreComplete { daemon_origin: u32, restored_seq: u64 },

    /// Phase 3→4: Target finished replaying buffered events
    ReplayComplete { daemon_origin: u32, replayed_seq: u64 },

    /// Phase 4: Source stops accepting writes, routing switches
    CutoverNotify { daemon_origin: u32, target_node: u64 },

    /// Phase 5: Source cleaned up
    CleanupComplete { daemon_origin: u32 },

    /// Any phase: Abort
    MigrationFailed { daemon_origin: u32, reason: String },
}
```

Wire constants:

```rust
pub mod wire {
    pub const MSG_TAKE_SNAPSHOT: u8     = 0;
    pub const MSG_SNAPSHOT_READY: u8    = 1;
    pub const MSG_RESTORE_COMPLETE: u8  = 2;
    pub const MSG_REPLAY_COMPLETE: u8   = 3;
    pub const MSG_CUTOVER_NOTIFY: u8    = 4;
    pub const MSG_CLEANUP_COMPLETE: u8  = 5;
    pub const MSG_FAILED: u8           = 6;
}
```

### Key Methods

```rust
impl MigrationOrchestrator {
    /// Initiate migration (phase 0: Snapshot)
    pub async fn start_migration(
        &self,
        daemon_origin: u32,
        source_node: u64,
        target_node: u64,
        timeout: Duration,
    ) -> Result<(), MigrationError>;

    /// Handle snapshot taken (phase 1→2)
    pub async fn on_snapshot_ready(
        &self, daemon_origin: u32, snapshot_bytes: Vec<u8>, seq_through: u64,
    ) -> Result<(), MigrationError>;

    /// Handle restore complete (phase 2→3)
    pub async fn on_restore_complete(
        &self, daemon_origin: u32, restored_seq: u64,
    ) -> Result<(), MigrationError>;

    /// Handle replay complete (phase 3→4)
    pub async fn on_replay_complete(
        &self, daemon_origin: u32, replayed_seq: u64,
    ) -> Result<(), MigrationError>;

    /// Abort migration at any phase
    pub async fn abort_migration(
        &self, daemon_origin: u32, reason: String,
    ) -> Result<(), MigrationError>;

    /// Query status
    pub fn status(&self, daemon_origin: u32) -> Option<MigrationPhase>;
    pub fn list_migrations(&self) -> Vec<(u32, MigrationPhase, u64)>;
}
```

### Responsibilities

1. Sequence through 6 phases with per-phase timeouts
2. Route `MigrationMessage` to the correct nodes
3. Update `NetRouter` routes at cutover (remove source, add target)
4. Abort if `FailureDetector` reports source or target down
5. Drive event buffering on source between Snapshot and Cutover
6. Update `SuperpositionState` at each phase transition

---

## Workstream 2: Source-Side Handler

**Goal:** Implement the source node's role — snapshot, buffer, cutover, cleanup.

**Deliverable:** `src/adapter/net/compute/migration_source.rs`

### Types

```rust
pub struct MigrationSourceHandler {
    daemon_registry: Arc<DaemonRegistry>,
    migrations: DashMap<u32, SourceMigrationState>,
    router: Arc<NetRouter>,
}

pub struct SourceMigrationState {
    daemon_origin: u32,
    source_node: u64,
    target_node: u64,
    phase: MigrationPhase,
    snapshot: Option<StateSnapshot>,
    buffered_events: Vec<(u64, CausalEvent)>,  // (seq, event), ordered
    last_buffered_seq: u64,
    started_at: Instant,
}
```

### Key Methods

```rust
impl MigrationSourceHandler {
    /// Phase 0: Take snapshot, return it for transfer
    pub async fn start_snapshot(
        &self, daemon_origin: u32, source_node: u64, target_node: u64,
    ) -> Result<StateSnapshot, MigrationError>;

    /// Phases 0-3: Buffer events arriving for this daemon during migration
    pub fn buffer_event(
        &self, daemon_origin: u32, event: CausalEvent,
    ) -> Result<(), MigrationError>;

    /// Phase 4: Stop accepting writes
    pub fn on_cutover(&self, daemon_origin: u32) -> Result<(), MigrationError>;

    /// Get buffered events for replay on target
    pub fn get_buffered_events(
        &self, daemon_origin: u32,
    ) -> Result<Vec<CausalEvent>, MigrationError>;

    /// Phase 5: Unregister daemon, remove routes, clean up
    pub async fn cleanup(&self, daemon_origin: u32) -> Result<(), MigrationError>;

    /// Abort: clear buffers, return to normal operation
    pub async fn abort(&self, daemon_origin: u32) -> Result<(), MigrationError>;
}
```

### Invariants

- Once cutover is reached, source rejects all new events for that daemon
- Buffered events are in strictly increasing sequence order
- Snapshot is taken exactly once per migration
- Source keeps daemon alive until cleanup phase completes

---

## Workstream 3: Target-Side Handler

**Goal:** Implement the target node's role — restore, replay, activate.

**Deliverable:** `src/adapter/net/compute/migration_target.rs`

### Types

```rust
pub struct MigrationTargetHandler {
    daemon_registry: Arc<DaemonRegistry>,
    migrations: DashMap<u32, TargetMigrationState>,
}

pub struct TargetMigrationState {
    daemon_origin: u32,
    source_node: u64,
    target_node: u64,
    phase: MigrationPhase,
    replayed_through: u64,
    pending_events: BTreeMap<u64, CausalEvent>,  // seq -> event, sorted
    target_head: CausalLink,
    started_at: Instant,
}
```

### Key Methods

```rust
impl MigrationTargetHandler {
    /// Phase 2: Deserialize snapshot, create daemon, call daemon.restore()
    pub async fn restore_snapshot(
        &self, daemon_origin: u32, snapshot_bytes: &[u8], source_node: u64,
    ) -> Result<(), MigrationError>;

    /// Phase 3: Replay buffered events in sequence order
    pub async fn replay_events(
        &self, daemon_origin: u32, events: Vec<CausalEvent>,
    ) -> Result<(), MigrationError>;

    /// Buffer out-of-order events arriving during replay
    pub fn buffer_event(
        &self, daemon_origin: u32, event: CausalEvent,
    ) -> Result<(), MigrationError>;

    /// Phase 4: Drain remaining events, daemon goes live
    pub async fn activate(&self, daemon_origin: u32) -> Result<(), MigrationError>;

    /// Abort: unregister daemon, clean up
    pub async fn abort(&self, daemon_origin: u32) -> Result<(), MigrationError>;
}
```

### Invariants

- Events replayed in strict sequence order (BTreeMap ensures sorting)
- Target becomes authoritative only after cutover
- All source events must be replayed before cutover
- Out-of-order arrivals are buffered, not dropped

---

## Workstream 4: Subprotocol Message Handler

**Goal:** Wire migration messages into the Net protocol's subprotocol layer.

**Deliverable:** `src/adapter/net/subprotocol/migration_handler.rs`

### Types

```rust
pub struct MigrationSubprotocolHandler {
    orchestrator: Arc<MigrationOrchestrator>,
    source_handler: Arc<MigrationSourceHandler>,
    target_handler: Arc<MigrationTargetHandler>,
    tx: tokio::sync::mpsc::Sender<(u64, Vec<u8>)>,  // (dest_node, payload)
}
```

### Message Dispatch

```rust
impl MigrationSubprotocolHandler {
    pub async fn handle_message(
        &self, msg: MigrationMessage, from_node: u64,
    ) -> Result<(), MigrationError> {
        match msg {
            TakeSnapshot { .. }     => source_handler.start_snapshot() + send SnapshotReady back,
            SnapshotReady { .. }    => orchestrator.on_snapshot_ready(),
            RestoreComplete { .. }  => orchestrator.on_restore_complete(),
            ReplayComplete { .. }   => orchestrator.on_replay_complete(),
            CutoverNotify { .. }    => source_handler.on_cutover(),
            CleanupComplete { .. }  => orchestrator marks complete,
            MigrationFailed { .. }  => source_handler.abort() + target_handler.abort(),
        }
    }
}
```

### Registration

Register in `SubprotocolRegistry::with_defaults()`:

```rust
reg.register(SubprotocolDescriptor::new(
    SUBPROTOCOL_MIGRATION, "migration", SubprotocolVersion::new(1, 0),
));
```

---

## Workstream 5: Event Buffering & Causal Chain Preservation

**Goal:** Ensure events are buffered and replayed in strict causal order. Integrate migration awareness into `DaemonHost`.

**Deliverable:** Modifications to `src/adapter/net/compute/daemon.rs`

### Changes to DaemonHost

1. **Migration-aware delivery:**

```rust
impl DaemonHost {
    pub fn deliver_with_buffering(
        &mut self,
        event: &CausalEvent,
        source_handler: &MigrationSourceHandler,
    ) -> Result<Vec<CausalEvent>, DaemonError> {
        if source_handler.is_migrating(self.origin_hash()) {
            source_handler.buffer_event(self.origin_hash(), event.clone())?;
            return Ok(vec![]);  // processed on target later
        }
        self.deliver(event)
    }
}
```

2. **Causal chain restoration after snapshot restore:**

```rust
impl DaemonHost {
    pub fn restore_causal_chain(&mut self, snapshot: &StateSnapshot) {
        self.chain_builder = CausalChainBuilder::from_head(
            snapshot.chain_link,
            Bytes::new(),
        );
    }
}
```

### Consistency Guarantees

- All events on source before cutover have sequence < cutover_seq
- Buffered events replayed on target in sequence order
- Target's causal chain is unbroken: `parent_hash` links verified through replay

---

## Workstream 6: Integration Tests

**Deliverable:** `tests/migration_integration.rs`

### Test Matrix

| Test | What It Validates |
|---|---|
| `test_migrate_daemon_happy_path` | Full 6-phase flow: snapshot → transfer → restore → replay → cutover → cleanup |
| `test_migration_abort_snapshot_phase` | Source failure during snapshot → clean abort |
| `test_migration_abort_replay_phase` | Target failure during replay → abort, source keeps daemon |
| `test_causal_ordering_preserved` | 100 events → migrate → verify all replayed in order with valid chain links |
| `test_concurrent_migrations` | Migrate daemon A→B and C→D simultaneously → no interference |
| `test_out_of_order_during_superposition` | Events arrive out-of-order at target → BTreeMap sorts → correct replay |
| `test_large_snapshot_chunking` | Snapshot > MTU → chunked transfer → reassembly on target |
| `test_migration_timeout` | Phase exceeds timeout → auto-abort |
| `test_cyclic_migration_rejected` | A→B then B→A while first in flight → `AlreadyMigrating` error |

### Integration Checklist

- [ ] Orchestrator sequences TakeSnapshot → SnapshotReady → RestoreComplete → ReplayComplete → CutoverNotify → CleanupComplete
- [ ] Source buffers events during Transfer through Replay phases
- [ ] Target restores daemon and replays in sequence order
- [ ] CutoverNotify pauses source, activates target
- [ ] CleanupComplete deregisters source daemon
- [ ] FailureDetector triggers abort if node goes down mid-migration
- [ ] Router updated at cutover — no packet loss window
- [ ] SuperpositionState tracks phase progression correctly
- [ ] Causal links validated end-to-end post-migration
- [ ] Concurrent migrations of different daemons don't interfere

---

## Critical Edge Cases

### 1. Snapshot Larger Than MTU (65535 bytes)
Chunk snapshot over multiple packets with sequence numbers. Add a chunking layer to `SnapshotReady`:
```
chunk_index: u16, total_chunks: u16, chunk_data: Vec<u8>
```
Reassemble on target before calling `restore_snapshot()`.

### 2. Network Partition During Superposed Phase
Source still accepts writes, target accumulates backlog. If partition lasts longer than phase timeout, abort and restart. Heartbeat between source and orchestrator detects this.

### 3. Source Crashes After Cutover
Already safe — target is authoritative. Orphaned buffered events on source are discarded. No special handling needed.

### 4. Events Arriving Out-of-Order at Target During Replay
Target buffers in `BTreeMap<u64, CausalEvent>` keyed by sequence. Replay drains in sorted order, not arrival order.

### 5. Cyclic Migrations (A→B→A)
Reject if daemon is already in `migrations` map. Return `MigrationError::AlreadyMigrating(origin)`.

### 6. Very Large Snapshots + Slow Network
Phase timeout in Transfer phase. Make timeout configurable, default 5 minutes. Consider compression for snapshots > 1MB.

---

## Implementation Order & Dependencies

```
WS1: Orchestrator ──────────────────┐
                                     │
WS2: Source Handler ─────────┐       │
                              ├──> WS4: Message Handler ──> WS5: DaemonHost ──> WS6: Tests
WS3: Target Handler ─────────┘
```

| Phase | Workstream | Files | Depends On |
|-------|-----------|-------|------------|
| **1** | Orchestrator | `compute/orchestrator.rs` | — |
| **2** | Source Handler | `compute/migration_source.rs` | WS1 |
| **3** | Target Handler | `compute/migration_target.rs` | WS1 |
| **4** | Message Handler | `subprotocol/migration_handler.rs` | WS1, WS2, WS3 |
| **5** | DaemonHost integration | `compute/daemon.rs` (modify) | WS4 |
| **6** | Integration tests | `tests/migration_integration.rs` | WS1–WS5 |

WS2 and WS3 can be built in parallel after WS1.

---

## Future Enhancements (Post-MVP)

1. **Connection resumption** — Token-based session transfer so clients don't need a new Noise handshake after cutover. Derive token from snapshot ID.
2. **Client redirection** — Publish target node address in registry after cutover. Clients discover via lookup rather than connection failure.
3. **Graceful drain** — Explicit drain protocol before source cleanup: flush in-flight ACKs, wait for reliable retransmissions to complete.
4. **Queue backpressure** — Wrap `SegQueue` with capacity limits and slow-down signals during replay to prevent memory exhaustion.
5. **Incremental snapshots** — Only transfer state deltas for large daemons.
6. **Live snapshots** — Take snapshots without pausing daemon processing (copy-on-write).
7. **Multi-target replication** — Parallel replay on multiple standby nodes for high availability.
8. **Migration metrics** — Per-phase latency, snapshot size, replay throughput, cutover downtime.
