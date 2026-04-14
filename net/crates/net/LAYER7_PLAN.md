# Layer 7: Observational Continuity — Implementation Plan

## Core Design

L7 formalizes the observational model: each node's truth is what it can observe.
Most of the foundation already exists in L4/L5 — causal chains, horizons, migration.
L7 adds the query/reasoning layer and handles the edge cases: forks, superposition,
propagation speed, and honest discontinuity.

The causal chain IS identity. If the chain is unbroken, the entity is continuous.
If it breaks, a new entity forks. The mesh doesn't need external proof — the chain
IS the proof.

## What Already Exists vs. What's New

| Capability | Status |
|------------|--------|
| Causal chain linking | DONE (state/causal.rs) |
| Chain validation | DONE (validate_chain_link) |
| Compressed horizon encoding | DONE (state/horizon.rs) |
| Bloom-filter causality query | DONE (HorizonEncoder) |
| Horizon update in process loop | DONE (DaemonHost::deliver) |
| Snapshot/restore chain continuity | DONE (DaemonHost::from_snapshot) |
| Migration state machine | DONE (compute/migration.rs) |
| Subnet hierarchy + gateway | DONE (subnet/) |
| hop_count in header | DONE (protocol.rs) |
| **ObservationWindow** | **NEW** |
| **CausalCone queries** | **NEW** |
| **PropagationModel** | **NEW** |
| **ContinuityStatus/Proof** | **NEW** |
| **SuperpositionState** | **NEW** |
| **Discontinuity/ForkRecord** | **NEW** |

## New Module: `src/adapter/bltp/continuity/`

### Phase 1: Chain Formalization

**`continuity/chain.rs`** — ContinuityStatus and ContinuityProof

```rust
pub enum ContinuityStatus {
    Continuous { genesis_hash: u64, head_seq: u64, head_hash: u64 },
    Forked { fork_point: u64, original_hash: u64, fork_hash: u64 },
    Unverifiable { last_verified_seq: u64, gap_start: u64 },
    Migrated { migration_seq: u64, source_node: u64, target_node: u64 },
}

/// Compact proof of continuity (40 bytes), transmittable.
pub struct ContinuityProof {
    pub origin_hash: u32,
    pub from_seq: u64,
    pub to_seq: u64,
    pub from_hash: u64,
    pub to_hash: u64,
    pub event_count: u64,
}
```

- `assess_continuity(log) -> ContinuityStatus`
- `ContinuityProof::from_log(log) -> Self`
- `ContinuityProof::verify_against(log) -> Result`

**`continuity/discontinuity.rs`** — Honest chain breaks

```rust
pub struct Discontinuity {
    pub origin_hash: u32,
    pub last_verified: CausalLink,
    pub failed_link: Option<CausalLink>,
    pub reason: DiscontinuityReason,
    pub detected_at: u64,
}

pub enum DiscontinuityReason {
    NodeCrash { last_snapshot_seq: u64 },
    ChainBreak(ChainError),
    ConflictingChains { seq: u64, hash_a: u64, hash_b: u64 },
    Corruption,
}

pub struct ForkRecord {
    pub original_origin: u32,
    pub forked_origin: u32,
    pub fork_seq: u64,
    pub fork_genesis: CausalLink,
    pub from_snapshot_seq: Option<u64>,
}
```

Fork genesis uses a deterministic sentinel: `parent_hash = xxh3(original_origin ++ fork_seq ++ "fork")`.
Any node can verify a fork record is legitimate.

### Phase 2: Observation Layer

**`continuity/propagation.rs`** — Speed-of-light per subnet level

```rust
pub struct PropagationModel {
    pub base_hop_latency_nanos: u64,
    pub level_multipliers: [f32; 4],  // [1.0, 5.0, 50.0, 500.0]
}
```

- `estimate_latency(source_subnet, dest_subnet, hop_count) -> Duration`
- `crossing_depth(a, b) -> u8` — how many subnet levels differ
- `calibrate(&mut self, measured_rtt)` — self-tuning from real measurements

**`continuity/observation.rs`** — Enriched horizon with temporal bounds

```rust
pub struct ObservationWindow {
    horizon: ObservedHorizon,
    local_subnet: SubnetId,
    last_direct_observation: HashMap<u32, u64>,
    estimated_delay: HashMap<u32, u64>,
}
```

- `observe_with_context(origin_hash, sequence, hop_count, subnet_id)`
- `staleness(origin_hash) -> Option<Duration>`
- `is_within_cone(origin_hash, max_delay) -> bool`
- `divergence_from(other) -> HorizonDivergence`

**`continuity/cone.rs`** — Causal cone queries

```rust
pub struct CausalCone {
    origin_hash: u32,
    sequence: u64,
    horizon: Option<ObservedHorizon>,
    horizon_encoded: u32,
}

pub enum Causality { Definite, Possible, No, Unknown }
```

- `could_have_influenced(other_origin, other_seq) -> Causality`
- `is_concurrent_with(other) -> bool`

Local node has exact cone (full ObservedHorizon). Remote observers see approximate
cone (4-byte compressed). This asymmetry is intentional — you know your own full state,
others see a compressed view.

### Phase 3: Migration Integration

**`continuity/superposition.rs`** — Entity on two nodes simultaneously

```rust
pub enum SuperpositionPhase {
    Localized,        // entity on source only
    Spreading,        // snapshot taken, target restoring
    Superposed,       // both nodes may process
    ReadyToCollapse,  // target caught up
    Collapsed,        // routing switched to target
    Resolved,         // source cleaned up
}

pub struct SuperpositionState {
    origin_hash: u32,
    source_head: CausalLink,
    target_head: CausalLink,
    phase: SuperpositionPhase,
    dual_delivery: bool,
}
```

Maps directly to existing `MigrationPhase`:
- `Spreading` ↔ `Transfer`
- `Superposed` ↔ `Restore + Replay`
- `ReadyToCollapse` ↔ pre-`Cutover`
- `Collapsed` ↔ `Cutover`
- `Resolved` ↔ `Complete`

Wraps MigrationState, doesn't replace it. Adds observational semantics.

## Subprotocol IDs

```
0x0700  = continuity base
0x0701  = fork announcements
0x0702  = continuity proof exchange
```

## Existing File Modifications

| File | Change |
|------|--------|
| `state/horizon.rs` | Add `entries()` and `iter()` accessors to ObservedHorizon |
| `compute/host.rs` | Add `horizon()` and `chain_head()` accessors |
| `mod.rs` | Add `pub mod continuity;`, re-exports |

## What L8 (Contested Environments) Needs from L7

1. **ContinuityProof** — nodes verify each other's chain integrity
2. **Discontinuity detection** — conflicting chains flag compromised entities
3. **ObservationWindow** — staleness detection identifies partitioned nodes
4. **ForkRecord** — clean recovery from corruption/compromise

## MVP Scope

Phases 1-2 (chain formalization + observation layer). Phase 3 (superposition)
follows once L5 migration is used in practice.

~1,500 lines across 6 new files. No new dependencies.
