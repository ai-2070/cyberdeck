# Compute Runtime

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

Snapshots larger than 7,000 bytes (fitting within the 8,192-byte MTU) are automatically chunked into multiple `SnapshotReady` messages. Each carries `chunk_index: u32` and `total_chunks: u32` metadata. The `SnapshotReassembler` on the receiving side collects chunks keyed by `(daemon_origin, seq_through)` and reassembles them in order. Chunks from different snapshot generations cannot be mixed.

### Transfer Limits

| Constraint | Limit | Source |
|---|---|---|
| `MAX_SNAPSHOT_CHUNK_SIZE` | 7,000 bytes per chunk | Wire overhead + 8,192-byte MTU |
| `MAX_SNAPSHOT_SIZE` | ~28 TB (`u32::MAX` chunks x 7,000 bytes) | `chunk_index: u32` / `total_chunks: u32` |
| `StateSnapshot` wire format | ~4 GB | `state_len: u32` in `to_bytes()` |

The practical limit is the `StateSnapshot` serialization at ~4 GB (`state_len: u32`). At present, snapshots beyond that limit panic in `to_bytes()`; `MigrationError::SnapshotTooLarge` applies to chunk-count overflow at `MAX_SNAPSHOT_SIZE` (~28 TB).

### Capability Advertisement

Nodes advertise migration support through the capability graph. `SubprotocolRegistry::enrich_capabilities()` injects `subprotocol:0x0500` into the node's `CapabilitySet`, which is broadcast via `CapabilityAnnouncement`. The `Scheduler` queries the `CapabilityIndex` for this tag when finding migration targets, combined with the daemon's own capability requirements.

### Superposition

During migration, a `SuperpositionState` tracks the entity's observational phase. The entity exists on both nodes briefly during replay, then collapses to the target at cutover. See [CONTINUITY.md](CONTINUITY.md) for details.

## Replica Groups

Where migration moves a daemon 1:1, `ReplicaGroup` replicates a daemon 1:N. Each replica is a normal `DaemonHost` registered in the `DaemonRegistry` — the group is a coordination overlay, not a new runtime concept.

**Identity is deterministic.** Replica keypairs derive from `group_seed + index` via BLAKE2s-MAC (keyed with `"net-replica-v1"`), following the same cryptographic KDF pattern as `EntityId` derivation. The same index always produces the same keypair, making replacement idempotent — a failed replica re-spawns with the same origin_hash on a different node, no migration needed.

**Routing is load-balanced.** For stateless or explicitly key-partitioned daemons, each replica is an `Endpoint` in an internal `LoadBalancer`. `route_event()` returns the `origin_hash` of the selected replica for delivery via `DaemonRegistry::deliver()`. Stateful daemons that need consistent state should use `StandbyGroup` (active-passive) instead, or use `ConsistentHash` strategy for sticky routing by key.

**Health is group-level.** The group is alive as long as at least one replica is healthy. `ReplicaGroupHealth::Degraded { healthy, total }` reports partial availability. On node failure, `on_node_failure()` marks affected replicas unhealthy, re-derives the same keypair, places on a new node, and re-spawns. On recovery, `on_node_recovery()` re-marks them healthy.

**Scaling is deterministic.** `scale_to(n)` adds replicas at the next index or removes the highest-index ones. Because keypairs derive from `group_seed + index`, the identity of each replica is fixed by its position — no coordination needed.

```
ReplicaGroup (group_id: 0xABCD, seed: [...])
├── Replica 0: origin_hash=0x1234, node=0xAAAA, healthy
├── Replica 1: origin_hash=0x5678, node=0xBBBB, healthy
└── Replica 2: origin_hash=0x9ABC, node=0xCCCC, healthy
                    │
                    ▼
              LoadBalancer
         (RoundRobin / LeastConn / ...)
                    │
                    ▼
           DaemonRegistry::deliver(selected_origin_hash, event)
```

Both `ReplicaGroup` and `ForkGroup` (below) delegate to a shared `GroupCoordinator` for load balancing, health tracking, member management, and routing. The coordinator is an internal primitive — the two group types own it and expose their own APIs.

`SUBPROTOCOL_REPLICA_GROUP` (0x0900) is reserved for future cross-node group coordination (membership announcements, coordinated scaling). The current implementation operates as a local coordinator — all cross-node communication uses existing primitives.

## Fork Groups

Where replicas are interchangeable copies with deterministic seed-derived identities, forks are independent entities with cryptographically documented lineage. A `ForkGroup` creates N daemons forked from a common parent at a specific point in its causal chain.

**Lineage is verifiable.** Each fork gets a `ForkRecord` with a sentinel hash: `parent_hash = xxh3(original_origin ++ fork_seq ++ "fork")`. Any node on the mesh can verify the fork by recomputing the sentinel. The fork record is created by `fork_entity()` from the continuity layer.

**Identity is stored, not derived.** Unlike replicas (where keypairs derive deterministically from a seed), fork keypairs are generated randomly by `fork_entity()` and then stored for recovery. On node failure, the fork is re-created from the stored keypair secret — same `origin_hash`, same `ForkRecord`, fresh daemon and chain.

**The chain documents the fork.** `DaemonHost::from_fork()` creates a host with a `CausalChainBuilder` whose genesis link carries the fork sentinel as `parent_hash`. Events produced by the fork chain back through this genesis to the parent's chain at the fork point.

**Scaling works like replicas.** `scale_to(n)` adds new forks from the same parent at the same `fork_seq`, or removes the highest-index ones. Each new fork gets its own random keypair and `ForkRecord`.

```
ForkGroup (parent: 0xAAAA, fork_seq: 100)
├── Fork 0: origin=0x1234, sentinel=xxh3(0xAAAA||100||"fork"), node=0xBBBB
├── Fork 1: origin=0x5678, sentinel=xxh3(0xAAAA||100||"fork"), node=0xCCCC
└── Fork 2: origin=0x9ABC, sentinel=xxh3(0xAAAA||100||"fork"), node=0xDDDD
                    │
                    ▼
              LoadBalancer
                    │
                    ▼
           DaemonRegistry::deliver(selected_origin_hash, event)
```

**Replicas vs Forks:**

| | Replicas | Forks |
|---|---|---|
| Identity | Deterministic from seed | Random, stored for recovery |
| Lineage | None | `ForkRecord` with verifiable sentinel |
| Members | Interchangeable | Independent, divergent |
| Recovery | Re-derive same keypair | Re-create from stored secret |
| Chain genesis | Normal genesis | Fork genesis with sentinel `parent_hash` |
| Use case | Horizontal scale, LB | Fan-out, A/B, specialization |

**Composability with migration.** Forks are normal daemons in the `DaemonRegistry`. MIKOSHI can migrate a fork to another node — the migration system doesn't know or care that the daemon is a fork. The causal chain and fork lineage travel with the snapshot.

## Standby Groups

For stateful daemons that need fault tolerance without duplicate compute, `StandbyGroup` implements active-passive replication. One member processes events. The others hold readiness to promote. No duplicate event processing — standbys consume memory but zero compute.

**The active processes, standbys wait.** Events route exclusively to the active via `active_origin()`. `on_event_delivered()` buffers each event for replay on promotion. Standbys are registered in the `DaemonRegistry` with their own identity but receive no events.

**Sync is snapshot-based.** `sync_standbys()` snapshots the active daemon, records `synced_through` for each standby, and clears the event buffer. The protocol tracks the sequence each standby is synced to. Persistence of snapshot bytes to disk is an application concern — the protocol provides the bytes and the bookkeeping.

**Promotion replays the gap.** On active failure, `promote()` picks the standby with the highest `synced_through` and replays buffered events (same mechanism as MIKOSHI's replay phase). The gap between "last sync" and "failure" is exactly the buffered events.

```
StandbyGroup (group_id: 0xABCD)
├── Member 0 [ACTIVE]:  origin=0x1234, processing events, synced_through=100
├── Member 1 [STANDBY]: origin=0x5678, idle, synced_through=100
└── Member 2 [STANDBY]: origin=0x9ABC, idle, synced_through=100

Active fails → promote Member 1:
├── Member 0 [STANDBY]: marked unhealthy
├── Member 1 [ACTIVE]:  replayed 3 buffered events, now at seq 103
└── Member 2 [STANDBY]: synced_through=100 (will re-sync from new active)
```

**Protocol vs application responsibilities:**

| Protocol (StandbyGroup) | Application |
|---|---|
| Active/standby role tracking | When to call `sync_standbys()` |
| Event buffering for replay | Persisting snapshots to disk |
| Promotion on failure | Consistency verification |
| Standby re-placement | Eventual consistency for durable storage |
| Deterministic identity | Snapshot frequency policy |

## Group Comparison

| | ReplicaGroup | ForkGroup | StandbyGroup |
|---|---|---|---|
| **Members** | Interchangeable | Independent, divergent | 1 active, N-1 passive |
| **Event routing** | LB to any member | LB to any fork | Always to active only |
| **Compute cost** | 1x (per event) | 1x (per event, per fork) | 1x (active only) |
| **State** | Stateless | Stateless | Stateful |
| **Identity** | Deterministic from seed | Random, stored | Deterministic from seed |
| **Lineage** | None | ForkRecord with sentinel | None |
| **Recovery** | Re-derive keypair | Re-create from stored secret | Promote standby + replay |
| **Use case** | Horizontal scale | Fan-out, A/B | Fault-tolerant stateful |

All three group types share `GroupCoordinator` for member management, health tracking, and placement. All three compose with MIKOSHI — any member of any group is a normal daemon that can be individually migrated.

## Source Files

| File | Purpose |
|------|---------|
| `compute/daemon.rs` | `MeshDaemon` trait, `DaemonError` |
| `compute/host.rs` | `DaemonHost`, lifecycle management, `from_snapshot()` restore, `from_fork()` |
| `compute/migration.rs` | `MigrationState`, `MigrationPhase`, 6-phase state machine |
| `compute/orchestrator.rs` | `MigrationOrchestrator`, `MigrationMessage` wire protocol, snapshot chunking, `SnapshotReassembler` |
| `compute/migration_source.rs` | `MigrationSourceHandler`, source-side snapshot/buffer/cutover/cleanup |
| `compute/migration_target.rs` | `MigrationTargetHandler`, target-side restore/replay/activate |
| `compute/group_coord.rs` | `GroupCoordinator`, shared LB/health/routing for replica and fork groups |
| `compute/replica_group.rs` | `ReplicaGroup`, N-way replication with deterministic identity |
| `compute/fork_group.rs` | `ForkGroup`, N-way forking with verifiable lineage and stored keypairs |
| `compute/standby_group.rs` | `StandbyGroup`, active-passive stateful replication with snapshot sync |
| `compute/registry.rs` | `DaemonRegistry`, local daemon tracking |
| `compute/scheduler.rs` | `Scheduler`, `PlacementDecision`, capability-based placement, `find_migration_targets()`, `place_migration()` |
| `subprotocol/migration_handler.rs` | `MigrationSubprotocolHandler`, message dispatch to orchestrator/source/target |
