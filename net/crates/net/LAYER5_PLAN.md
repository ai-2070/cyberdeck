# Layer 5: Compute Runtime — Implementation Plan

## Core Design

L5 defines what a "daemon" is in the mesh: a stateful or stateless event processor
with a causal identity, capability requirements for placement, and snapshot-based
migration. The `MeshDaemon` trait is minimal and sync — compatible with both native
Rust and future WASM guests.

## MeshDaemon Trait

```rust
pub trait MeshDaemon: Send + Sync {
    /// Human-readable name.
    fn name(&self) -> &'static str;

    /// Capability requirements for placement (empty = runs anywhere).
    fn requirements(&self) -> CapabilityFilter;

    /// Process one inbound causal event, returning zero or more output payloads.
    /// The runtime wraps outputs in CausalLinks automatically.
    fn process(&mut self, event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError>;

    /// Serialize state for migration. None = stateless.
    fn snapshot(&self) -> Option<Bytes> { None }

    /// Restore from snapshot. Called before any process() after migration.
    fn restore(&mut self, _state: Bytes) -> Result<(), DaemonError> { Ok(()) }
}
```

Key decisions:
- **Sync `process()`** — no async, keeps hot path allocation-free, WASM-compatible
- **`&CausalEvent` input, `Vec<Bytes>` output** — daemon sees full causal context,
  only produces payloads. Runtime handles chain building.
- **`requirements()` returns `CapabilityFilter`** — reuses existing L0 type directly

## New Module: `src/adapter/bltp/compute/`

### Phase 1: Core (MVP — unblocks L6)

**`compute/daemon.rs`** — `MeshDaemon` trait, `DaemonError`, `DaemonHostConfig`, `DaemonStats`

**`compute/host.rs`** — `DaemonHost` wrapping daemon + CausalChainBuilder + ObservedHorizon

```rust
pub struct DaemonHost {
    daemon: Box<dyn MeshDaemon>,
    keypair: EntityKeypair,
    chain: CausalChainBuilder,
    horizon: ObservedHorizon,
    config: DaemonHostConfig,
    stats: DaemonStats,
}
```

Hot path (`deliver`):
1. `horizon.observe(event.link.origin_hash, event.link.sequence)` — ~50ns
2. `daemon.process(event)` — target <1us
3. `chain.append(payload, horizon.encode())` per output — ~30ns
4. Total: <2us per event

**`compute/registry.rs`** — `DaemonRegistry` (DashMap + parking_lot::Mutex per daemon)
- `register()`, `unregister()`, `deliver()`, `snapshot()`, `list()`

### Phase 2: Placement

**`compute/scheduler.rs`** — `Scheduler` querying `CapabilityIndex`

```rust
pub struct Scheduler {
    capability_index: Arc<CapabilityIndex>,
    local_node_id: u64,
}
```

- `place(filter) -> PlacementDecision` — query index, prefer local, tie-break by load
- `can_run_locally(filter, local_caps) -> bool` — fast check

### Phase 3: Migration

**`compute/migration.rs`** — State machine using L4 snapshots

```rust
pub enum MigrationPhase {
    Snapshot,   // take snapshot on source
    Transfer,   // send via SUBPROTOCOL_SNAPSHOT
    Restore,    // restore on target, buffer events
    Replay,     // replay buffered events
    Cutover,    // atomic routing update
    Complete,   // cleanup source
}
```

Migration preserves chain continuity: target's CausalChainBuilder starts from
snapshot's chain head, so parent_hash links across the migration boundary.

### Phase 4: WASM (post-MVP)

- `wasmtime` dependency (feature-gated as `wasm`)
- `compute/wasm.rs` — `WasmDaemon` implementing `MeshDaemon`
- Guest ABI: `process(ptr, len) -> ptr`, `snapshot() -> ptr`, `restore(ptr, len)`
- Fuel metering for execution limits

**wasmtime** over wasmer: Bytecode Alliance backing, Component Model support,
built-in fuel metering, better Cranelift codegen.

## Existing File Modifications

| File | Change | Phase |
|------|--------|-------|
| `mod.rs` | Add `pub mod compute;`, re-exports | 1 |
| `Cargo.toml` | No new deps for MVP. `wasmtime` in Phase 4 only. | 4 |

## What L6 (Subprotocols) Needs from L5

1. **`MeshDaemon` trait** — subprotocol handlers implement this
2. **`DaemonHost`** — wraps a handler with causal chain production
3. **`DaemonRegistry`** — routes events to the right handler by origin_hash

MVP = Phase 1. Phases 2-4 can follow.

~800-1,200 lines across 4 new files (Phase 1). No new dependencies.

## Key Invariants

1. Every daemon has exactly one `EntityKeypair` — its mesh identity
2. Every output event is causally linked via `CausalChainBuilder` — no orphans
3. `ObservedHorizon` updated before `process()` — outputs carry correct horizon
4. Migration preserves chain continuity — no gaps in parent_hash linkage
5. No `async` on `MeshDaemon` — critical for WASM and microsecond latency
