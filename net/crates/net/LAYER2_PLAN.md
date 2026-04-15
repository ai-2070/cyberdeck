# Layer 2: Channels & Authorization — Implementation Plan

## Core Design

A **channel** is a named, policy-bearing logical endpoint. A **stream** is a transport-level
flow with sequencing. One channel can have many streams (many publishers), sharing the same
`channel_hash: u16` in the header. Forwarding nodes read `channel_hash` + `origin_hash` and
make authorization decisions in under 10ns via bloom filter — no per-packet crypto.

No wire format changes needed. The L0 header already has `channel_hash`, `priority`,
`origin_hash`, and `subnet_id`.

Channel policy uses the existing **capability system** (`CapabilityFilter`, `CapabilitySet`,
`CapabilityAd`) instead of a separate rule engine. Authorization =
`CapabilityFilter::matches(node_caps)` + L1 `PermissionToken` check.

---

## New Module: `src/adapter/nltp/channel/`

### Step 1: `channel/name.rs` — Named Typed Channels

- `ChannelName` — validated hierarchical string (`"sensors/lidar/front"`, max 255 bytes)
- `channel_hash(name: &str) -> u16` — deterministic xxh3 truncation (matches existing
  `stream_id_from_key` pattern)
- `ChannelId` — `(ChannelName, u16)` with hash cached at construction
- `ChannelRegistry` — `DashMap<u16, Vec<ChannelId>>` for collision detection at creation time

The u16 space has collisions (~256 channels hit birthday bound). This is fine — `channel_hash`
is a fast-path filter, not a unique key. Collisions mean a forwarding node might forward a
packet it could have dropped, but never drops one it should have forwarded.

### Step 2: `channel/config.rs` — Channel Configuration

Channel policy is expressed through existing capability requirements + L1 tokens:

```rust
pub struct ChannelConfig {
    pub channel_id: ChannelId,
    pub visibility: Visibility,
    /// Capability requirements for publishing (checked via CapabilityFilter)
    pub publish_caps: Option<CapabilityFilter>,
    /// Capability requirements for subscribing
    pub subscribe_caps: Option<CapabilityFilter>,
    /// Whether a valid PermissionToken is also required (in addition to capabilities)
    pub require_token: bool,
    /// Default priority level for this channel's packets
    pub priority: u8,
    /// Default reliability mode
    pub reliable: bool,
    /// Optional rate limit (packets per second)
    pub max_rate_pps: Option<u32>,
}

pub enum Visibility {
    SubnetLocal,      // never leaves subnet
    ParentVisible,    // visible to parent subnet
    Exported,         // explicitly exported to targets
    Global,           // visible everywhere
}
```

`ChannelConfigRegistry` — `DashMap<u16, ChannelConfig>`, consulted at subscription time (slow path).

Authorization flow:
1. Node announces capabilities via `CapabilityAd` (existing L0 infrastructure)
2. Channel has `publish_caps: Option<CapabilityFilter>` — if set, node must match
3. If `require_token` is true, node must also have a valid `PermissionToken` in `TokenCache`
4. On success, `(origin_hash, channel_hash)` inserted into `AuthGuard` bloom filter
5. All subsequent packets from that origin to that channel pass the fast path

### Step 3: `channel/guard.rs` — Wire-Speed Authorization

The performance-critical piece:

- `AuthGuard` containing:
  - A ~4KB bloom filter keyed by `(origin_hash, channel_hash)` tuples. Fits in L1 cache.
    2-3ns probe time.
  - A small `DashMap<(u32, u16), bool>` as verified-positive cache for bloom hits.
- `check_fast(origin_hash, channel_hash) -> AuthVerdict` — the per-packet hot path (<10ns)
- `authorize(...)` — slow-path check at subscription time:
  1. Look up `ChannelConfig` from registry
  2. If `publish_caps` is set, check `CapabilityFilter::matches(node_caps)`
  3. If `require_token`, check `TokenCache::check(entity, action, channel_hash)`
  4. On success, insert `(origin_hash, channel_hash)` into bloom filter
- `revoke(origin_hash, channel_hash)` — remove from bloom filter (rebuild filter)

Bloom filter sizing: ~1000 active authorized (origin, channel) pairs in 4KB gives <0.1%
false positive rate.

### Step 4: `channel/priority.rs` — Priority Preemption

- Upgrade `FairScheduler` from binary `priority: bool` to `priority_level: u8` with
  tiered queues (4 tiers: safety-critical, high, normal, best-effort)
- Channel priority from `ChannelConfig::priority` maps to packet priority level
- Preemption: safety-critical packet evicts lowest-priority packet when queues are full

---

## Existing File Modifications

| File | Change | Step |
|------|--------|------|
| `mod.rs` | Add `pub mod channel;`, re-exports | 1 |
| `protocol.rs` | Add `NltpHeader::with_channel()` convenience | 1 |
| `pool.rs` | `PacketBuilder` gets `channel_hash: u16`, stamps in `build()` | 3 |
| `router.rs` | Add `AuthGuard` check in `route_packet()`, upgrade `FairScheduler` | 3-4 |
| `proxy.rs` | Same `AuthGuard` check in forwarding path | 3 |
| `session.rs` | Optionally bind channel to stream | 3 |

---

## What L3 (Subnets) Needs from L2

- `ChannelConfig::visibility` — gateways use this to decide cross-subnet forwarding
- `CapabilityFilter` on channels — L3 gateways check capabilities at subnet boundaries
- `AuthGuard::check_fast()` — gateway nodes call this for cross-subnet authorization
- `ChannelConfigRegistry` — L3 queries which channels are exported at subnet boundaries

---

## MVP Scope

Steps 1-3 (name, config with capability-based policy, guard with bloom filter).
Step 4 (priority preemption) follows.

~1,200-1,500 lines across 4 new files. No new dependencies — reuses existing
`CapabilityFilter`, `CapabilitySet`, `CapabilityAd`, `TokenCache`, and `xxhash-rust`.
