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

```rust
pub struct MigrationState {
    daemon_origin: u32,
    source_node: u64,
    target_node: u64,
    phase: MigrationPhase,
    snapshot: Option<StateSnapshot>,
    buffered_events: Vec<CausalEvent>,
    started_at: u64,
}
```

Phase transitions are validated -- calling `set_snapshot()` in the wrong phase returns `MigrationError::WrongPhase`. The snapshot's `origin_hash` is verified against the daemon being migrated.

Events arriving during migration are buffered and replayed after restore. This ensures no events are lost during the transfer window.

## Source Files

| File | Purpose |
|------|---------|
| `compute/daemon.rs` | `MeshDaemon` trait, `DaemonError` |
| `compute/host.rs` | `DaemonHost`, lifecycle management |
| `compute/migration.rs` | `MigrationState`, `MigrationPhase`, 6-phase state machine |
| `compute/registry.rs` | `DaemonRegistry`, local daemon tracking |
| `compute/scheduler.rs` | `Scheduler`, `PlacementDecision`, capability-based placement |
