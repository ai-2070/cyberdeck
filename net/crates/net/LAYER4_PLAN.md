# Layer 4: Distributed State — Implementation Plan

## Core Design

Layer 4 introduces causal metadata, distributed event logs, state snapshots, and
replication. No global coordinator — causal consistency by default, strong consensus
opt-in. The causal chain IS the entity's identity.

Events get a 24-byte `CausalLink` prefix inside EventFrame. The NLTP header stays
unchanged — L4 state is above L0 transport.

## CausalLink (24 bytes on wire)

```rust
#[repr(C)]
pub struct CausalLink {
    pub origin_hash: u32,       // entity identity (matches NLTP header)
    pub sequence: u64,          // monotonic per-entity
    pub parent_hash: u64,       // xxh3 of (prev CausalLink ++ prev payload)
    pub horizon_encoded: u32,   // compressed observed horizon (bloom sketch)
}
```

**parent_hash**: `xxh3_64(prev_causal_link_bytes ++ prev_payload)` — ~1ns, not crypto.
Tamper resistance comes from NLTP's AEAD, not per-event signatures.

**EventFrame format** with causal framing (signaled by `subprotocol_id = 0x0400`):
```
[len: u32][CausalLink: 24 bytes][payload: len-24 bytes]
```

Non-causal events (subprotocol_id == 0) use the existing format unchanged.

## Compressed Observed Horizon (4 bytes)

Full vector clocks scale O(N) — unacceptable at 40M events/sec. Instead:

- 16 bits: bloom filter of recently-observed origin_hashes
- 16 bits: truncated max observed sequence across watched entities

This gives O(1) approximate causality detection. For entities needing exact ordering,
a full `ObservedHorizon` (HashMap-based vector clock) is exchanged out-of-band, not
per-event.

## New Module: `src/adapter/nltp/state/`

### Phase 1: CausalLink + Causal Framing (MVP-critical)

**`state/causal.rs`**

- `CausalLink` with `to_bytes()` / `from_bytes()` (24 bytes, repr(C))
- `CausalChain` validator (verifies parent_hash linkage)
- Subprotocol ID reservation: `SUBPROTOCOL_CAUSAL = 0x0400`

**`protocol.rs` modifications:**
- `EventFrame::write_causal_events()` / `read_causal_events()` — prepend/strip CausalLink

### Phase 2: Compressed Horizon

**`state/horizon.rs`**

- `HorizonEncoder::encode()` / `decode()` for 4-byte bloom sketch
- `ObservedHorizon` full vector clock (for out-of-band sync)
- Approximate causality detection between two horizon values

### Phase 3: Entity Log (MVP-critical)

**`state/log.rs`**

```rust
pub struct EntityLog {
    entity_id: EntityId,
    origin_hash: u32,
    events: Vec<CausalEvent>,
    head_seq: u64,
    snapshot_seq: u64,
}
```

- `LogIndex` — `DashMap<u32, EntityLog>` for per-origin_hash lookup
- Append with chain validation, duplicate rejection, pruning
- Integration into `process_packet` for causal subprotocol events

### Phase 4: State Snapshots (MVP-critical)

**`state/snapshot.rs`**

```rust
pub struct StateSnapshot {
    pub entity_id: EntityId,
    pub through_seq: u64,       // valid through this sequence
    pub chain_hash: u64,        // verification hash
    pub state: Bytes,           // serialized daemon state (opaque)
    pub horizon: ObservedHorizon,
    pub created_at: u64,
}
```

- Snapshot creation API (called by L5 compute runtime)
- Transfer via NLTP fragmentation (`subprotocol_id = 0x0401`)
- Catchup protocol: snapshot + replay events after `through_seq`

### Phase 5: Replication Policy (deferrable past L5 MVP)

**`state/replication.rs`**

```rust
pub struct ReplicationPolicy {
    pub min_replicas: u8,
    pub min_subnets: u8,
    pub subnet_depth: u8,
    pub max_log_entries: u32,
    pub max_snapshot_age_secs: u64,
}
```

- `ReplicationManager` tracks replica counts via gossip on `CapabilityAd`
- Uses `SubnetId` from L3 for geographic diversity requirements
- Piggybacked on existing swarm — no new gossip protocol

### Phase 6: Consistency Boundaries (deferrable past L5 MVP)

**`state/consistency.rs`**

```rust
pub enum ConsistencyLevel {
    Causal,                         // default, zero coordination
    Strong { quorum_size: u8 },     // quorum acks before commit
}
```

- `QuorumTracker` for strong consensus acknowledgments
- Added to `ChannelConfig` as optional field

## Existing File Modifications

| File | Change | Phase |
|------|--------|-------|
| `protocol.rs` | Add `read/write_causal_events` to EventFrame, subprotocol constants | 1 |
| `mod.rs` | Add `pub mod state;`, re-exports, causal branch in `process_packet` | 1, 3 |
| `channel/config.rs` | Add optional `consistency: ConsistencyLevel` | 6 |
| `swarm.rs` | Extend CapabilityAd with replica status tags | 5 |

## What L5 (Compute Runtime) Needs from L4

1. **CausalLink** — so daemons produce causally-linked events
2. **EntityLog** — so daemon state is reconstructible from the event chain
3. **StateSnapshot** — so daemons can checkpoint and migrate
4. **Catchup protocol** — so a new node can restore daemon state

MVP = Phases 1-4. Phases 5-6 can wait until after L5 has basic daemons working.

## Performance Budget

| Operation | Budget | Approach |
|-----------|--------|----------|
| CausalLink serialization | <5ns | `copy_from_slice` on repr(C) |
| parent_hash computation | <3ns | xxh3 (~50GB/s) |
| Horizon encode | <5ns | 4 bloom probes |
| EntityLog append | <50ns | DashMap + Vec push |
| Chain validation | <10ns | Compare parent_hash |
| **Total per-event** | **<73ns** | Within budget at 40M/s with sharding |

## Key Design Decisions

- **xxh3 for parent_hash, not blake2**: blake2 ~2GB/s vs xxh3 ~50GB/s. At 40M events/sec
  blake2 would consume 2 CPU cores. Chain integrity comes from NLTP AEAD, not per-event crypto.
- **24 bytes, not smaller**: 8B parent_hash = 2^64 collision resistance. 8B sequence =
  584 years at 1B events/sec. Can't go smaller without sacrificing correctness.
- **CausalLink in payload, not header**: Header is full at 64 bytes. Adding 24 bytes would
  break cache-line alignment and tax non-causal traffic.
- **Replication gossip on CapabilityAd**: No new protocol. ~10 byte increase per ad.

~1,500-2,000 lines across 6 new files. No new dependencies.
