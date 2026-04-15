# Layer 3: Subnets & Hierarchy — Implementation Plan

## Core Design

Subnets are label-based — nodes belong by identity/capability, not static configuration.
The `subnet_id: u32` field is already in every NLTP header. Hierarchy is encoded as
fixed-width level packing (8/8/8/8) giving 4 levels with 256 values each.

Gateway nodes sit at subnet boundaries and enforce visibility policy (filter/drop) without
decrypting or re-encrypting packets. The MVP gateway only makes forward/drop decisions
based on `subnet_id` + `Visibility` — no header rewriting, no AAD issues.

## subnet_id Encoding

```
subnet_id (u32):
  [level_0: 8 bits] [level_1: 8 bits] [level_2: 8 bits] [level_3: 8 bits]
   ^region (256)     ^fleet (256)       ^vehicle (256)     ^subsystem (256)
```

- Prefix comparison for hierarchy = single integer comparison: `(a & mask) == (b & mask)`
- Parent/child/sibling relationships resolved with bitwise ops at wire speed
- Zero (`0x00000000`) = no subnet / global

```rust
pub struct SubnetId(u32);

impl SubnetId {
    pub const GLOBAL: Self = Self(0);
    pub fn new(levels: &[u8]) -> Self;
    pub fn level(&self, n: u8) -> u8;
    pub fn depth(&self) -> u8;                    // number of non-zero levels
    pub fn parent(&self) -> Self;                 // zero out deepest level
    pub fn is_ancestor_of(&self, other: &Self) -> bool;  // prefix match
    pub fn is_same_subnet(&self, other: &Self) -> bool;
}
```

## New Module: `src/adapter/nltp/subnet/`

### Phase 1: Foundation

**`subnet/id.rs`** — `SubnetId` type with hierarchy encoding and bitwise operations.

**`subnet/assignment.rs`** — Label-based subnet assignment from `CapabilitySet` tags.

```rust
pub struct SubnetPolicy {
    rules: Vec<SubnetRule>,
}

pub struct SubnetRule {
    tag_pattern: String,              // e.g., "region:*"
    level: u8,                        // which hierarchy level
    value_map: HashMap<String, u8>,   // "us-west" -> 1
}
```

Nodes propose their own `subnet_id` based on their labels. Gateways validate the
claim against capabilities before accepting traffic.

### Phase 2: Gateway

**`subnet/gateway.rs`** — The causal membrane at subnet boundaries.

```rust
pub struct SubnetGateway {
    local_subnet: SubnetId,
    peer_subnets: Vec<SubnetId>,
    export_table: DashMap<u16, Vec<SubnetId>>,   // channel_hash -> allowed targets
}

pub enum ForwardDecision {
    Forward,
    Drop(DropReason),
}
```

Gateway logic per packet (reads only header fields, no decryption):
- `SubnetLocal` channels → drop at boundary
- `ParentVisible` → forward only if destination is ancestor
- `Exported` → forward only to configured target subnets
- `Global` → always forward

Integration point: `NltpRouter::route_packet()` gains an optional `SubnetGateway`
check between TTL validation and route lookup.

### Phase 3: Discovery

**`subnet/scope.rs`** — Scoped pingwave with observation horizons.

- Within-subnet pingwaves use existing TTL/radius
- Cross-subnet pingwaves stopped at gateway boundaries (non-gateway nodes ignore them)
- Gateway-to-gateway discovery propagates at parent level

`Pingwave` struct uses 4 of its 6 reserved bytes for `subnet_id`. `LocalGraph`
gains `my_subnet_id` and `is_gateway` fields. `NodeInfo` gains `subnet_id`.

### Post-MVP (defer past L4)

**`subnet/query.rs`** — Cascading capability queries (local first, escalate through
hierarchy). Uses `subprotocol_id = 0x0003`.

**`subnet/federation.rs`** — Gateway peering, cross-subnet route advertisements
(BGP-like).

## Existing File Modifications

| File | Change | Phase |
|------|--------|-------|
| `mod.rs` | Add `pub mod subnet;`, re-exports | 1 |
| `swarm.rs` | Add `subnet_id` to `Pingwave` (reserved bytes), `NodeInfo` | 1, 3 |
| `router.rs` | Add optional `SubnetGateway` to `NltpRouter`, subnet filtering in `route_packet` | 2 |
| `channel/config.rs` | Add visibility-checking helpers against `SubnetId` pairs | 2 |
| `route.rs` | Tag routes with subnet for subnet-aware lookup | 2 |

## What L4 (Distributed State) Needs from L3

1. **`SubnetId` type** — so distributed state can be scoped ("replicate within this subnet")
2. **Gateway filtering by Visibility** — so `SubnetLocal` state updates never leak
3. **`subnet_id` stamped on outgoing packets** — so the router distinguishes local vs cross-subnet

## MVP Scope

Phases 1-2 (SubnetId type, label assignment, gateway filtering). Phase 3 (scoped
pingwave) follows. Post-MVP items can wait until after L4.

~1,000-1,500 lines across 4 new files. No new dependencies — reuses existing
`CapabilitySet`, `CapabilityFilter`, `ChannelConfigRegistry`, and `Visibility`.

## Key Design Decisions

- **No header rewriting in MVP**: Gateways only forward/drop, they don't mutate
  `subnet_id`. Packets originate with the correct `subnet_id` from the sender.
  This avoids AAD issues entirely.
- **No re-encryption at gateways**: Gateways make decisions from header fields
  (plaintext). The encrypted payload passes through untouched.
- **subnet_id stays in AAD**: Since gateways don't rewrite it, it can remain
  authenticated. Unlike `hop_count` (mutable in transit), `subnet_id` is set
  once by the sender.
