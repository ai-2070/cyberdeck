# Compute Runtime (Layer 5)

Stateful event processors that run on the mesh. The `MeshDaemon` trait defines the processing contract. The runtime handles causal chain production, horizon tracking, snapshot packaging, capability-based placement, and 6-phase migration.

## MeshDaemon Trait

The core abstraction for event processors. Daemons consume causal events and produce output payloads. The runtime wraps outputs in `CausalLink`s automatically.

```rust
pub trait MeshDaemon: Send + Sync {
    fn name(&self) -> &str;
    fn requirements(&self) -> CapabilityFilter;
    fn process(&mut self, event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError>;
    fn snapshot(&self) -> Option<Bytes>;           // None for stateless daemons
    fn restore(&mut self, state: Bytes) -> Result<(), DaemonError>;
}
```

**Design constraints:**
- `process()` must complete in microseconds -- heavy work should be deferred to background tasks
- All methods are synchronous (no async) for WASM compatibility
- Input/output are `Bytes` -- maps cleanly to WASM linear memory
- No generics or associated types

## Daemon Host

`DaemonHost` manages the lifecycle of a daemon instance: spawning, feeding events, collecting output, and coordinating migration.

The host wraps a `MeshDaemon` with:
- A `CausalChainBuilder` that automatically chains output events
- Horizon tracking (what the daemon has observed)
- Stats collection (`DaemonStats`)

## Daemon Registry

`DaemonRegistry` tracks all locally-running daemons. Lookup by origin hash or name. Used by the scheduler to know what's running where.

## Capability-Based Placement

`Scheduler` decides where to place daemons based on capability requirements.

```rust
pub struct PlacementDecision {
    pub target_node: u64,
    pub reason: PlacementReason,
}

pub enum PlacementReason {
    CapabilityMatch,       // Node has required capabilities
    AffinityMatch,         // Node has affinity tags
    LoadBalance,           // Least-loaded node with required caps
    Migration,             // Migrating from overloaded node
}
```

The scheduler queries the `CapabilityIndex` from the behavior plane to find nodes matching a daemon's `requirements()`. Placement considers capability match, load, and proximity.

## 6-Phase Migration

Migration moves a daemon between nodes while preserving causal chain continuity. The process is a strict state machine:

```
Snapshot -> Transfer -> Restore -> Replay -> Cutover -> Complete
```

| Phase | What happens |
|-------|-------------|
| **Snapshot** | Take `StateSnapshot` on source node (daemon state + chain head + horizon) |
| **Transfer** | Send snapshot to target node via `SUBPROTOCOL_MIGRATION` (0x0500) |
| **Restore** | Call `daemon.restore(state)` on target, start buffering new events |
| **Replay** | Replay buffered events on target (events that arrived during transfer) |
| **Cutover** | Atomic routing switch -- new events go to target |
| **Complete** | Cleanup source, migration done |

Phase transitions are validated -- calling `set_snapshot()` in the wrong phase returns `MigrationError::WrongPhase`. The snapshot's `origin_hash` is verified against the daemon being migrated.

Events arriving during migration are buffered and replayed after restore. This ensures no events are lost during the transfer window.

### Migration Orchestrator

`MigrationOrchestrator` coordinates the full 6-phase lifecycle from a controller node (which may be the source, target, or a third party). It tracks in-flight migrations, manages phase transitions, and produces outbound messages for the source and target handlers.

```
                  ┌─────────────────────────┐
                  │  MigrationOrchestrator   │
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

**Auto-target selection:** `start_migration_auto()` uses the `Scheduler` to find the best migration-capable target by querying the `CapabilityIndex` for nodes advertising `subprotocol:0x0500`.

### Source Handler

`MigrationSourceHandler` manages the source node's role:
- Takes a snapshot of the local daemon
- Buffers events arriving during transfer/replay phases
- Stops accepting writes at cutover
- Unregisters the daemon after cleanup

### Target Handler

`MigrationTargetHandler` manages the target node's role:
- Restores a daemon from a snapshot via `DaemonHost::from_snapshot()`
- Replays buffered events in strict sequence order (uses `BTreeMap` for out-of-order arrival handling)
- Activates as the authoritative copy after cutover

### Migration Wire Protocol

8 message types over `SUBPROTOCOL_MIGRATION` (0x0500):

| Message | Direction | Purpose |
|---------|-----------|---------|
| `TakeSnapshot` | Orchestrator → Source | Request snapshot |
| `SnapshotReady` | Source → Orchestrator → Target | Snapshot data (chunked for large snapshots) |
| `RestoreComplete` | Target → Orchestrator | Daemon restored |
| `BufferedEvents` | Orchestrator → Target | Events to replay |
| `ReplayComplete` | Target → Orchestrator | Replay done |
| `CutoverNotify` | Orchestrator → Source | Stop writes |
| `CleanupComplete` | Source → Orchestrator | Source cleaned up |
| `MigrationFailed` | Any → All | Abort |

### Snapshot Chunking

Snapshots larger than 7,000 bytes (fitting within the 8,192-byte MTU) are automatically chunked into multiple `SnapshotReady` messages. Each carries `chunk_index` and `total_chunks` metadata. The `SnapshotReassembler` on the receiving side collects chunks keyed by `(daemon_origin, seq_through)` and reassembles them in order. Chunks from different snapshot generations cannot be mixed.

### Capability Advertisement

Nodes advertise migration support through the capability graph. `SubprotocolRegistry::enrich_capabilities()` injects `subprotocol:0x0500` into the node's `CapabilitySet`, which is broadcast via `CapabilityAnnouncement`. The `Scheduler` queries the `CapabilityIndex` for this tag when finding migration targets, combined with the daemon's own capability requirements.

### Superposition

During migration, a `SuperpositionState` (Layer 7) tracks the entity's observational phase. The entity exists on both nodes briefly during replay, then collapses to the target at cutover. See [CONTINUITY.md](CONTINUITY.md) for details.

## Source Files

| File | Purpose |
|------|---------|
| `compute/daemon.rs` | `MeshDaemon` trait, `DaemonError` |
| `compute/host.rs` | `DaemonHost`, lifecycle management, `from_snapshot()` restore |
| `compute/migration.rs` | `MigrationState`, `MigrationPhase`, 6-phase state machine |
| `compute/orchestrator.rs` | `MigrationOrchestrator`, `MigrationMessage` wire protocol, snapshot chunking, `SnapshotReassembler` |
| `compute/migration_source.rs` | `MigrationSourceHandler`, source-side snapshot/buffer/cutover/cleanup |
| `compute/migration_target.rs` | `MigrationTargetHandler`, target-side restore/replay/activate |
| `compute/registry.rs` | `DaemonRegistry`, local daemon tracking |
| `compute/scheduler.rs` | `Scheduler`, `PlacementDecision`, capability-based placement, `find_migration_targets()`, `place_migration()` |
| `subprotocol/migration_handler.rs` | `MigrationSubprotocolHandler`, message dispatch to orchestrator/source/target |
