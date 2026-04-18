# Channel Publisher — multicast / fan-out at the daemon layer

## Status

Design only. Fan-out ("same content to many peers") is deliberately **not** a transport-layer feature in Net. The `STREAM_MULTIPLEXING_PLAN.md` explicitly excluded it as a non-goal; this plan picks it up one layer up, as a thin helper over `MeshNode` that does N per-peer sends behind a channel-shaped API. No new header bits, no multicast packet primitive, no group crypto in transport.

## What already works

- **`ChannelName` / `ChannelId` / `ChannelRegistry`** in `adapter/net/channel/name.rs` — hierarchical names, `u16` hash for headers, DashMap-backed registry.
- **`ChannelConfig`** in `adapter/net/channel/config.rs` — per-channel visibility, `publish_caps` / `subscribe_caps` (capability filters), `require_token`, default priority and reliability.
- **`AuthGuard`** in `adapter/net/channel/guard.rs` — wire-speed <10 ns bloom-filter authorization for `(origin_hash, channel_hash)` pairs. `authorize()` / `revoke()` / `check_fast()` already exist.
- **`CapabilityFilter` + `PermissionToken`** — the full auth check behind the bloom filter.
- **`MeshNode::send_routed(dest_node_id, batch)`** in `mesh.rs` — the per-peer unicast send that the helper will call in a loop.
- **`MeshNode::send_on_stream(stream, events)`** in `mesh.rs` — the per-peer stream-aware send for callers that want FireAndForget vs Reliable per channel.
- **Subprotocol dispatch** — `SubprotocolRegistry` in `subprotocol/registry.rs`, dispatched by `subprotocol_id` in the routing header. Existing IDs: `0x0400 CAUSAL`, `0x0401 SNAPSHOT`, `0x0500 MIGRATION`, `0x0600 NEGOTIATION`, `0x0700 CONTINUITY`, `0x0800 PARTITION`, `0x0801 RECONCILE`, `0x0900 REPLICA_GROUP`.
- **Failure detector** — `failure.rs` already publishes peer up/down lifecycle that a roster can consume.

## What is missing

1. **No per-channel subscriber roster.** `ChannelRegistry` knows the channel exists; nothing knows *who is listening to it*. Subscriber state lives only as bloom-filter bits in `AuthGuard` — fine for enforcement, useless for enumeration.
2. **No subscribe / unsubscribe wire protocol.** Nothing tells a would-be publisher "node X is now a subscriber of `sensors/lidar`." Without that, the helper has nothing to fan out to.
3. **No publish helper.** Callers who want "send this event to everyone subscribed to channel C" would have to maintain their own peer list, iterate it, and call `send_routed` per peer. Every daemon that wants fan-out re-implements the same loop.
4. **No roster churn handling.** When a peer fails, disappears, or revokes its subscription, nothing removes it from the publisher's fan-out set.

## Goals

- A **`ChannelPublisher`** (a.k.a. `GroupSender`) that takes a `ChannelName` + payload and fans it out to all known subscribers by doing per-peer sends.
- A **`SubscriberRoster`** on `MeshNode` keyed by `ChannelId`, populated by explicit subscribe / unsubscribe messages.
- A **`SUBPROTOCOL_CHANNEL_MEMBERSHIP`** carrying `Subscribe(channel_name)` / `Unsubscribe(channel_name)` / `SubscribeAck` messages.
- Publish-side ACL enforcement: before fan-out, check that the local node is allowed to publish on the channel (`ChannelConfig::can_publish`).
- Churn handling: a subscriber that disappears from the failure detector is evicted from the roster; explicit `Unsubscribe` also evicts.
- Per-publish result: callers get a summary of how many peers the fan-out reached and which ones errored, so they can decide what "success" means for their workload.

## Non-goals

- **No multicast packet primitive.** One `publish` call = N independent unicasts, one per subscriber. There is no "send this packet to many peers in one socket op," no tree-based dissemination, no gossip layer.
- **No group cryptography in transport.** Each unicast uses the existing per-peer Noise session. There is no group key, no shared session, no fan-out-specific AEAD.
- **No new header bits.** Routing header stays at 16 bytes as-is. Channel membership messages are a subprotocol, not a new packet flag.
- **No implicit "everyone on the mesh" broadcast.** Fan-out targets only nodes that have explicitly subscribed to the specific channel. "All peers" is an anti-pattern this plan rejects.
- **No history / catch-up on subscribe.** A node that subscribes at time T never receives publishes from before T. Subscribers that briefly disconnect miss whatever was published while they were down. Durable streams are a separate concern (EventBus / Mikoshi snapshots).
- **No atomic fan-out.** Publish to N subscribers succeeds partially by design. The per-publish result tells callers exactly which peers failed.
- **No scale optimizations.** O(subscribers) sends per publish. Fine for small channels (<~256 subscribers); explicitly not the right primitive for millions-of-subscribers pub/sub. That's a v2+ conversation (tree dissemination, epidemic broadcast) and out of scope.
- **No transport changes.** Zero edits to `session.rs`, `protocol.rs`, the Noise layer, or the fair scheduler.

## Design

### Wire layer — `SUBPROTOCOL_CHANNEL_MEMBERSHIP`

New subprotocol id `0x0A00` (next free range after replica group at `0x0900`). Payloads use a hand-rolled length-prefixed codec (see `channel/membership.rs`) — `u8` tag + `u64_le` nonce + `u8` length + UTF-8 channel name for Subscribe/Unsubscribe, `u8` tag + `u64_le` nonce + `u8` accepted-flag + `u8` reason for Ack. No CBOR, no bincode; chosen to keep the wire format tight and avoid pulling a general serializer onto the hot path. Carried on existing encrypted sessions — these are routed, not broadcast:

```rust
pub const SUBPROTOCOL_CHANNEL_MEMBERSHIP: u16 = 0x0A00;

pub enum MembershipMsg {
    Subscribe   { channel: ChannelName, nonce: u64 },
    Unsubscribe { channel: ChannelName, nonce: u64 },
    Ack         { nonce: u64, accepted: bool, reason: Option<AckReason> },
}

pub enum AckReason {
    Unauthorized,   // subscriber_caps mismatch or missing token
    UnknownChannel, // channel not registered on the publisher side
    RateLimited,    // membership churn throttled
}
```

No new flags in the routing header; these messages ride `send_subprotocol` over existing sessions. If the publisher has no session to the subscriber, the subscriber's `Subscribe` triggers handshake as a side effect the way any other first-contact message does today.

### Roster — `SubscriberRoster` on `MeshNode`

```rust
struct SubscriberRoster {
    // ChannelId -> set of subscriber node_ids.
    subs: DashMap<ChannelId, DashSet<u64>>,
    // Reverse index for cheap peer-died cleanup.
    by_peer: DashMap<u64, DashSet<ChannelId>>,
}
```

- `add(channel_id, node_id)` — called on `Subscribe` after ACL check.
- `remove(channel_id, node_id)` — called on `Unsubscribe` or peer failure.
- `remove_peer(node_id)` — called from the failure-detector hook when a peer goes `Failed` (not `Suspected`); clears `by_peer[node_id]` and every referenced channel.
- `members(channel_id) -> Vec<u64>` — snapshot for the publisher.

Roster lives on `MeshNode` alongside `peers`, `router`, `failure_detector`. One per node.

### Publisher — `ChannelPublisher`

Handle-style, analogous to `Stream`:

```rust
pub struct ChannelPublisher {
    channel: ChannelId,
    config: PublishConfig,
    mesh: Weak<MeshNode>,
}

pub struct PublishConfig {
    pub reliability: Reliability,     // FireAndForget | Reliable, per fan-out hop
    pub stream_id: Option<u64>,       // if Some, use send_on_stream; else send_routed
    pub fairness_weight: u8,          // passed through to stream open
    pub on_failure: OnFailure,        // BestEffort | FailFast | Collect
}

pub enum OnFailure {
    BestEffort,  // log per-peer errors, return overall Ok if >0 peers delivered
    FailFast,    // abort on first per-peer error
    Collect,     // always return per-peer PublishReport
}

pub struct PublishReport {
    pub channel: ChannelId,
    pub attempted: usize,
    pub delivered: usize,
    pub errors: Vec<(u64, AdapterError)>,
}

impl MeshNode {
    pub fn channel_publisher(
        &self,
        channel: ChannelName,
        config: PublishConfig,
    ) -> Result<ChannelPublisher, AdapterError>;
}

impl ChannelPublisher {
    pub async fn publish(&self, payload: Bytes) -> Result<PublishReport, AdapterError>;
    pub async fn publish_many(&self, events: &[Bytes]) -> Result<PublishReport, AdapterError>;
    pub fn subscribers(&self) -> Vec<u64>;
    pub fn channel(&self) -> &ChannelId;
}
```

Resolution order inside `publish`:

1. ACL check: `ChannelConfig::can_publish(origin_hash, ...)` — if denied, return `AdapterError::Authorization` without a send.
2. Snapshot roster: `roster.members(self.channel)` returns a `Vec<u64>`.
3. If empty, return `PublishReport { attempted: 0, delivered: 0, errors: vec![] }`. Not an error.
4. If `stream_id` is set, ensure a stream per peer via `MeshNode::open_stream` (idempotent), then `send_on_stream`. Otherwise `send_routed` per peer.
5. Per-peer sends run with `futures::future::join_all` — bounded concurrency via a `Semaphore` (default 32) to avoid socket contention on large rosters.
6. Apply `OnFailure` policy and return.

### Subscriber side — `MeshNode::subscribe_channel` / `unsubscribe_channel`

```rust
impl MeshNode {
    pub async fn subscribe_channel(
        &self,
        publisher_node_id: u64,
        channel: ChannelName,
    ) -> Result<(), AdapterError>;

    pub async fn unsubscribe_channel(
        &self,
        publisher_node_id: u64,
        channel: ChannelName,
    ) -> Result<(), AdapterError>;
}
```

These just pack a `MembershipMsg::Subscribe` / `Unsubscribe` into a `SUBPROTOCOL_CHANNEL_MEMBERSHIP` packet and `send_subprotocol` to the publisher. They return once the `Ack` is received (or after a short timeout → `AdapterError::Connection` — `AdapterError` has no dedicated `Timeout` variant today; the error message is prefixed with `membership ack timeout`).

Subscriber identity for ACL purposes is the session origin: the publisher validates `subscribe_caps` against the *sender's* capability set, not against anything in the message itself. The wire message just names the channel.

### ACL — reuse what exists

- Publish check: `ChannelConfig::can_publish(local_origin_hash, local_caps, local_token)` on every `publish()`. Fast because the local side's caps are already resolved.
- Subscribe check: on incoming `Subscribe`, run `ChannelConfig::can_subscribe(sender_origin_hash, sender_caps, sender_token)`. If it passes, `AuthGuard::authorize(sender_origin_hash, channel_hash)` so downstream wire-speed checks on future publishes stay on the fast path. On `Unsubscribe` (or revoke), call `AuthGuard::revoke`.
- No new ACL code paths. The helper only *calls* the existing guard.

### Failure handling

- Failure detector hook: on `Failed` transition for peer `P`, call `roster.remove_peer(P)` (synchronous, DashMap). Eventual churn only; `Suspected` transitions are ignored so that brief flaps don't thrash the roster.
- Per-peer send errors inside `publish()` do NOT auto-evict. A subscriber that's temporarily slow / queue-full should stay in the roster. Only the failure detector evicts.
- `Unsubscribe` is authoritative: an explicit unsubscribe removes the entry immediately regardless of liveness.

## Implementation steps

1. **Step 1 — roster module.** New `adapter/net/channel/roster.rs` with `SubscriberRoster` + unit tests. No mesh changes yet.
2. **Step 2 — membership subprotocol.** Add `SUBPROTOCOL_CHANNEL_MEMBERSHIP` constant, `MembershipMsg` enum, encode/decode, subprotocol descriptor. Registered in `SubprotocolRegistry` next to the others.
3. **Step 3 — MeshNode wiring.** `MeshNode::subscribe_channel` / `unsubscribe_channel`; incoming-packet dispatch for the new subprotocol that runs ACL + updates roster + sends `Ack`. Failure-detector hook to `remove_peer` on `Failed`.
4. **Step 4 — publisher helper.** `ChannelPublisher` struct, `MeshNode::channel_publisher` constructor, `publish` / `publish_many` implementations with bounded concurrency semaphore. `OnFailure` variants wired through.
5. **Step 5 — docs.** Extend `CHANNELS.md` with a "Fan-out publishers" section: how the roster is built, the non-goals (no group crypto, no transport multicast, no catch-up), the `ChannelPublisher` API, concurrency bound.
6. **Step 6 — tests.**
   - Unit: roster add/remove/remove_peer/members; membership codec round-trip; ACL deny path on subscribe.
   - Integration (three-node): two subscribers on one publisher, `publish` reaches both; kill one subscriber, verify it's evicted via failure detector and next `publish` goes only to the survivor; `Unsubscribe` from a live peer evicts immediately.
   - Integration: publish-side ACL denial returns `Authorization` without any packet on the wire.
   - Integration: `OnFailure::FailFast` returns first error; `OnFailure::Collect` returns full per-peer map.

## Risks and open questions

- **Roster consistency.** The roster is eventually consistent: a late subscribe may miss in-flight publishes, and an un-ack'd subscribe may create roster divergence between subscriber and publisher. Document it; do not try to solve it at this layer.
- **Unbounded roster growth.** A malicious peer could spam `Subscribe` for thousands of channels. Rate-limit via `max_rate_pps` on the membership subprotocol and a hard per-peer cap (e.g., 1024 channels). `AckReason::RateLimited` covers the reject path.
- **Send concurrency tuning.** Default 32-concurrent per publish works for small rosters. Large ones (>256) need a knob; expose `PublishConfig::max_inflight` later if needed, not in v1.
- **Ordering across subscribers.** None. Two subscribers may receive publishes in different order relative to unrelated traffic. If a caller needs consistent ordering, they need a causal channel (`SUBPROTOCOL_CAUSAL`), not this helper.
- **Stream reuse vs per-publish stream.** Default is to open one stream per `(peer, channel)` pair and reuse it across publishes, so that per-stream ordering within a channel holds for each subscriber. Callers who want FireAndForget without stream state can leave `stream_id = None` and get `send_routed` semantics.

## Summary

This is a thin helper, not a protocol. Everything underneath — channels, ACL, unicast sessions, subprotocol dispatch, failure detection — already exists. The plan adds a subscriber roster, one subprotocol for membership churn, and a `ChannelPublisher` that loops over `send_routed` or `send_on_stream`. No new header bits, no group cryptography, no low-level multicast packet. Fan-out stays at the control plane where it belongs.
