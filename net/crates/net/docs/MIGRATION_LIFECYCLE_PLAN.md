# Migration Lifecycle: Full 6-Phase Auto-Chaining

> **Status:** Implemented. `DaemonFactoryRegistry` lives in `src/adapter/net/compute/daemon_factory.rs`; `ActivateTarget` / `ActivateAck` wire messages (tags 0x08 / 0x09) are in `compute/orchestrator.rs`; reassembly + target-side restore + activation branches are in `subprotocol/migration_handler.rs`. The wire-level three-node test is `three_node_integration::test_migration_full_lifecycle_over_wire`, and handler-level coverage is in `migration_integration::test_migration_full_lifecycle_over_subprotocol_*` (single-chunk, multi-chunk, no-factory, corrupt snapshot, activate-without-restore).

## Problem

The migration state machine has 6 phases: **Snapshot → Transfer → Restore → Replay → Cutover → Complete**. Only the first round-trip (TakeSnapshot → SnapshotReady) runs autonomously through the subprotocol; the remaining phases are exercised only when a test drives them by calling `target_handler.restore_snapshot()`, `target_handler.activate()`, etc. directly.

The blocker is on the **target side** of the subprotocol handler:

1. When the target receives `SnapshotReady`, the handler hands it to `orchestrator.on_snapshot_ready()` and forwards it again instead of actually restoring the daemon (`migration_handler.rs:94-121`). No `RestoreComplete` is ever produced.
2. Multi-chunk snapshots need `SnapshotReassembler` (it exists and is tested at `orchestrator.rs:567`+) but the handler never uses it.
3. There is no wire message or handler path that tells the target "cutover is done, call `activate()` now". The test fakes it at `migration_integration.rs:257`.
4. `target_handler.restore_snapshot()` takes a `daemon_factory: FnOnce() -> Box<dyn MeshDaemon>` (`migration_target.rs:68-78`) — a closure that cannot be serialized across the wire. We need a local mechanism for the subprotocol handler to look up the right factory from the incoming `daemon_origin`.

## Design

### Target-side flow (what has to run when snapshots arrive)

```
Target receives SnapshotReady                       Source
     │                                                │
     │  1. Feed chunk into SnapshotReassembler        │
     │  2. If complete:                               │
     │     a. Look up DaemonFactory by origin         │
     │     b. Look up EntityKeypair by origin         │
     │     c. target_handler.restore_snapshot(...)    │
     │     d. Emit RestoreComplete ──────────────────►│ (orchestrator)
     │                                                │
     │  ◄──── BufferedEvents (buffered during Snap/Xfer)
     │                                                │
     │  3. target_handler.replay_events(...)          │
     │  4. Emit ReplayComplete ──────────────────────►│ (orchestrator)
     │                                                │
     │                   orchestrator emits CutoverNotify to source
     │                                                │
     │  ◄──── (optional) BufferedEvents (final batch) │
     │                                                │
     │  5. After source's CleanupComplete arrives at   
     │     orchestrator, orchestrator emits a new       
     │     ActivateTarget message to the target        
     │                                                │
     │  6. target_handler.activate(...)               │
     │  7. Emit ActivateAck ─────────────────────────►│ (orchestrator)
     │                                                │
     │  Migration complete; daemon is authoritative   │
     │  on target.                                    │
```

### Key design decisions

1. **Daemon factory registry on every node that might be a migration target.** A new `DaemonFactoryRegistry`, owned by the `MigrationTargetHandler`, maps `origin_hash → Box<dyn Fn() -> Box<dyn MeshDaemon>>`. The subprotocol handler calls `registry.factory_for(origin)` when a restored snapshot arrives. Without a registered factory, the handler returns `MigrationFailed` to the orchestrator.

2. **Keypair provisioning is out of scope for the plan but sits in the same registry.** In this pass we assume the target has the `EntityKeypair` for any daemon it might host (provided by the caller, same as the test at `migration_integration.rs:208-219`). Long-term, secure key transfer is a separate security problem. For now: `DaemonFactoryRegistry` entries carry `{factory, keypair}`.

3. **Reassembly lives in the subprotocol handler, not the orchestrator.** The orchestrator stays wire-agnostic and continues to be called once per chunk; the handler turns N chunks into one `StateSnapshot` before invoking the target handler's restore. This matches where `chunk_snapshot` is already invoked on the source side (`migration_handler.rs:81`).

4. **Add one new wire message: `ActivateTarget { daemon_origin }`.** This is the "go live" signal sent by the orchestrator after it observes `CleanupComplete`. The target responds with `ActivateAck { daemon_origin, replayed_seq }`. Without this, there is no deterministic signal that tells the target when the source has actually stopped accepting writes.

5. **The orchestrator does not need to know about factories or keypairs.** It only coordinates phase transitions. All daemon-construction concerns stay in `MigrationTargetHandler` and the subprotocol handler.

## Implementation

### Step 1: `DaemonFactoryRegistry` (~40 lines, new)

New type in `compute/migration_target.rs` (or a sibling module):

```rust
pub struct DaemonFactoryRegistry {
    entries: DashMap<u32, FactoryEntry>,
}

struct FactoryEntry {
    factory: Box<dyn Fn() -> Box<dyn MeshDaemon> + Send + Sync>,
    keypair: EntityKeypair,
    config: DaemonHostConfig,
}

impl DaemonFactoryRegistry {
    pub fn register<F>(&self, origin_hash: u32, keypair: EntityKeypair,
                      config: DaemonHostConfig, factory: F)
        where F: Fn() -> Box<dyn MeshDaemon> + Send + Sync + 'static;
    pub fn take(&self, origin_hash: u32) -> Option<FactoryEntry>;
}
```

`take` (not `get`) — a factory is consumed on first restore. If a daemon migrates back later, the caller re-registers. Alternatively, keep the factory and consume only the keypair; either is fine, but `take` surfaces double-restore bugs loudly.

`MigrationTargetHandler::new_with_factories(registry, ...)` gives the handler access. Existing callers of `MigrationTargetHandler::new` continue to work with an empty registry (no daemon auto-restore).

### Step 2: Reassembly in the subprotocol handler (~60 lines)

In `subprotocol/migration_handler.rs`, add a per-migration reassembler map:

```rust
pub struct MigrationSubprotocolHandler {
    // ...existing fields...
    reassemblers: DashMap<u32, SnapshotReassembler>,
}
```

Update the `SnapshotReady` match arm (`migration_handler.rs:94-121`) so that after `orchestrator.on_snapshot_ready()` forwards the chunk:

- If we are the **target** for `daemon_origin` (check `orchestrator.target_node(daemon_origin) == local_node_id`), feed the chunk into the reassembler instead of forwarding.
- On completion (`SnapshotReassembler::feed` returns assembled bytes + `through_seq`), parse `StateSnapshot::from_bytes`, look up the factory via `DaemonFactoryRegistry::take(daemon_origin)`, call `target_handler.restore_snapshot(...)`, and emit `RestoreComplete` back to the orchestrator (source node).
- Remove the reassembler entry on success/failure to free memory.

Missing factory → emit `MigrationFailed { reason: "no factory registered for origin" }` to the source.

### Step 3: `ActivateTarget` / `ActivateAck` wire messages (~30 lines)

In `compute/orchestrator.rs` (the `MigrationMessage` enum and `wire::encode/decode`):

```rust
MigrationMessage::ActivateTarget { daemon_origin },     // 0x08
MigrationMessage::ActivateAck    { daemon_origin, replayed_seq }, // 0x09
```

In `MigrationOrchestrator`:

- `on_cleanup_complete` (orchestrator.rs:989) currently just removes the record. Change it to return `Option<MigrationMessage>`: after cleanup, emit `ActivateTarget` addressed to the target node. The caller (subprotocol handler) dispatches.
- Add `on_activate_ack(daemon_origin) -> Result<()>` that removes the record (the Complete phase terminus now lives after activation, not cleanup).

### Step 4: Wire `ActivateTarget` / `ActivateAck` in the subprotocol handler (~30 lines)

In `subprotocol/migration_handler.rs`:

- `CleanupComplete` branch (line 194): call `orchestrator.on_cleanup_complete()`. If it returns `ActivateTarget`, push it as an outbound message to the target.
- New `ActivateTarget` branch on the target side: call `target_handler.activate(daemon_origin)`, emit `ActivateAck { replayed_seq }` back to the orchestrator.
- New `ActivateAck` branch on the orchestrator side: call `orchestrator.on_activate_ack(daemon_origin)`.

### Step 5: Integration test — full wire-driven migration (~150 lines)

New test in `tests/migration_integration.rs`:

```rust
#[tokio::test]
async fn test_migration_full_lifecycle_over_subprotocol() { ... }
```

- Three nodes wired up (orchestrator O, source S, target T) either via real `MeshNode`s or a mock message bus that plumbs `OutboundMigrationMessage`s back into the appropriate handler's `handle_message`.
- Register a `TestDaemon` factory on T via `DaemonFactoryRegistry`.
- Register the daemon on S.
- Call `orch.start_migration(...)` on O.
- Pump the message loop until no more messages are produced.
- Assert the daemon is registered in T's registry, has replayed the expected sequence, has processed a post-cutover event, and is absent from S's registry.
- Multi-chunk variant: snapshot large enough to chunk, same assertions.

### Step 6: Tests for failure paths (~80 lines)

- Restore with no factory → source receives `MigrationFailed`, orchestrator aborts.
- Restore with snapshot parse failure (corrupted bytes) → same.
- ActivateTarget arrives but daemon not yet restored → graceful error, not a panic.

## Out of scope (explicit non-goals)

- Secure transfer of `EntityKeypair` from source to target. Tests provision keypairs on the target out-of-band; production deployments need key-material transport designed separately.
- Persistent event-log replay (replay-from-arbitrary-seq after migration completes). Current model buffers in memory during the migration window only; this plan does not change that.
- Superposition-state semantics. The orchestrator already updates `SuperpositionPhase` on transitions; it keeps doing so unchanged. If it turns out superposition is vestigial from fork-handling, deleting it is a separate cleanup.

## Files touched

| File | Purpose |
|---|---|
| `compute/migration_target.rs` | Add `DaemonFactoryRegistry`; `MigrationTargetHandler::new_with_factories` |
| `compute/orchestrator.rs` | Add `ActivateTarget` / `ActivateAck` variants, `on_activate_ack`, update `on_cleanup_complete` |
| `subprotocol/migration_handler.rs` | Reassembly, target-side `SnapshotReady` restore, `ActivateTarget` / `ActivateAck` branches |
| `tests/migration_integration.rs` | Full-lifecycle wire test (single-chunk + multi-chunk), failure-path tests |

## Scope

~400 lines total (40 registry + 60 reassembly + 30 new wire messages + 30 handler wiring + 230 tests). 3 new tests on the happy path + 3 on failure paths. No changes to `MigrationPhase` or `MigrationState`.
