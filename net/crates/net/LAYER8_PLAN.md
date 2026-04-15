# Layer 8: Contested Environments (Partial) — Implementation Plan

## Scope

Two items from the L8 roadmap. The rest (anti-jamming, multi-transport,
fragmentation, congestion control) require hardware integration or real
traffic to develop against.

1. **Correlated failure handling** — mass failure detection + throttled recovery
2. **Partition healing** — detection, state tracking, log reconciliation

## New Module: `src/adapter/nltp/contested/`

### Phase 1: `contested/correlation.rs` — Correlated Failure Detection

Wraps `FailureDetector` with a time-windowed correlation layer.

```rust
pub struct CorrelatedFailureConfig {
    pub correlation_window: Duration,        // e.g., 2 seconds
    pub mass_failure_threshold: f32,         // e.g., 0.30 (30% of nodes)
    pub subnet_correlation_threshold: f32,   // e.g., 0.80 (80% in one subnet)
    pub max_concurrent_migrations: usize,    // e.g., 3
}

pub enum CorrelationVerdict {
    Independent { failed_nodes: Vec<u64> },
    MassFailure {
        failed_nodes: Vec<u64>,
        failure_ratio: f32,
        suspected_cause: FailureCause,
    },
}

pub enum FailureCause {
    SubnetFailure { subnet: SubnetId, affected_ratio: f32 },
    BroadOutage,
    Unknown,
}
```

Subnet correlation algorithm:
1. Collect SubnetIds for failed nodes in the window
2. Walk up hierarchy with `SubnetId::parent()`, count failures per ancestor
3. If any ancestor has >= 80% of failures, classify as SubnetFailure

`recovery_budget()` returns throttled count during mass failure, unlimited otherwise.

### Phase 2: `contested/partition.rs` — Partition Detection & Healing

```rust
pub enum PartitionPhase {
    Suspected,
    Confirmed,
    Healing { reappeared: Vec<u64> },
    Healed,
}

pub struct PartitionRecord {
    pub id: u64,
    pub our_side: Vec<u64>,
    pub other_side: Vec<u64>,
    pub partition_subnet: Option<SubnetId>,
    pub phase: PartitionPhase,
    pub our_horizon_at_split: ObservedHorizon,
}
```

- Partition detected when `CorrelationVerdict::MassFailure` with `SubnetFailure` cause
- Healing detected when nodes from `other_side` reappear in `FailureDetector` recovery
- Healed when 50%+ of `other_side` reappears

The asymmetric insight: each side independently detects "mass failure in subnet X"
and enters partition mode. No coordination needed.

### Phase 3: `contested/reconcile.rs` — Log Reconciliation

After partition heals, merge divergent EntityLogs:

```rust
pub enum ReconcileOutcome {
    AlreadyConverged,
    Catchup { origin_hash: u32, missing_events: Vec<CausalEvent> },
    Conflict {
        origin_hash: u32,
        diverge_seq: u64,
        resolution: ConflictResolution,
    },
}

pub enum ConflictResolution {
    Winner { winning_side: Side, fork_record: ForkRecord },
    BothFork { fork_a: ForkRecord, fork_b: ForkRecord },
}
```

Reconciliation algorithm:
1. Use `HorizonDivergence` to identify which entities need reconciliation
2. For each entity, walk both chains from split point
3. **AlreadyConverged** — identical chains
4. **Catchup** — one chain is a prefix of the other
5. **Conflict** — chains diverge at a sequence:
   - Longest chain wins (more active during partition)
   - Equal length: lower `parent_hash` wins (deterministic, both sides agree)
   - Losing chain gets `ForkRecord` via `fork_entity()` from L7
   - Both sides reach same conclusion independently — no coordination

## Existing File Modifications

Only `mod.rs` — add `pub mod contested;` and re-exports.

All existing L0-L7 APIs used read-only. No modifications to `failure.rs`,
`state/log.rs`, `continuity/discontinuity.rs`, etc.

## Key Design Decisions

- **Ratio thresholds, not absolute counts** — scales with mesh size
- **Horizon snapshot at partition time** — baseline for "events during split"
- **Longest-chain-wins** — simple, deterministic, no coordination needed
- **Deterministic tiebreak on parent_hash** — both sides independently agree
- **Auto-confirm subnet failures** — 80%+ in one subnet ancestor = partition
- **Losing chain becomes ForkRecord** — events not lost, just forked (L7 pattern)

~800-1,000 lines across 3 new files. No new dependencies — reuses
`FailureDetector`, `SubnetId`, `EntityLog`, `ObservationWindow`,
`HorizonDivergence`, `ForkRecord`, and `ContinuityProof`.
