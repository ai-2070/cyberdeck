//! MeshNode: multi-peer mesh runtime composing all protocol layers.
//!
//! `MeshNode` is the composition layer that turns independent components
//! (encrypted sessions, router, failure detector) into a functioning mesh
//! node that can communicate with multiple peers simultaneously over a
//! single UDP socket.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │                  MeshNode                   │
//! │                                             │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │ Session A│  │ Session B│  │ Session C│  │
//! │  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
//! │       │              │              │       │
//! │  ┌────┴──────────────┴──────────────┴────┐  │
//! │  │          Receive Loop (single)        │  │
//! │  │  demux by source_addr → session       │  │
//! │  │  local → decrypt → queue              │  │
//! │  │  forward → router (no decrypt)        │  │
//! │  └───────────────┬───────────────────────┘  │
//! │                  │                          │
//! │  ┌───────────────┴───────────────────────┐  │
//! │  │         UDP Socket (shared)           │  │
//! │  └───────────────────────────────────────┘  │
//! └─────────────────────────────────────────────┘
//! ```

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use arc_swap::ArcSwapOption;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use super::crypto::{handshake_prologue, CryptoError, NoiseHandshake, SessionKeys, StaticKeypair};
use super::failure::{FailureDetector, FailureDetectorConfig};
use super::identity::{EntityId, EntityKeypair, PermissionToken, TokenCache, TokenScope};
use super::pool::PacketBuilder;

use super::behavior::broadcast::SUBPROTOCOL_CAPABILITY_ANN;
use super::behavior::capability::{
    CapabilityAnnouncement, CapabilityFilter, CapabilityIndex, CapabilityRequirement,
    CapabilitySet, MAX_CAPABILITY_HOPS,
};
use super::behavior::loadbalance::HealthStatus;
use super::behavior::proximity::{EnhancedPingwave, ProximityConfig, ProximityGraph};
use super::channel::membership::{self, MembershipMsg, SUBPROTOCOL_CHANNEL_MEMBERSHIP};
use super::channel::{
    AckReason, AuthGuard, AuthVerdict, ChannelConfigRegistry, ChannelId, ChannelName,
    ChannelPublisher, OnFailure, PublishConfig, PublishReport, SubscriberRoster,
};
use super::compute::SUBPROTOCOL_MIGRATION;
use super::protocol::{self, EventFrame, PacketFlags, HEADER_SIZE, MAGIC, TAG_SIZE};

/// Wire overhead added to the AEAD-encrypted payload by every Net
/// packet: the 64-byte header plus the 16-byte Poly1305 tag. Credit
/// accounting charges this against the sender's `tx_credit_remaining`
/// alongside the payload so the byte window matches the bandwidth
/// the sender actually pumps onto the link. The receiver's
/// `on_bytes_consumed` adds the same overhead, keeping sender and
/// receiver in lockstep.
const PACKET_WIRE_OVERHEAD: usize = HEADER_SIZE + TAG_SIZE;

/// Total wire bytes for a single Net packet carrying `payload_bytes`
/// of AEAD-encrypted content. Saturating at `u32::MAX` so a
/// pathological `payload_bytes` can't silently wrap the credit math.
#[inline]
fn wire_bytes_for_payload(payload_bytes: usize) -> u32 {
    payload_bytes
        .saturating_add(PACKET_WIRE_OVERHEAD)
        .min(u32::MAX as usize) as u32
}
use super::reroute::ReroutePolicy;
use super::route::{RoutingHeader, ROUTING_HEADER_SIZE};
use super::router::{NetRouter, RouterConfig};
use super::session::{NetSession, TxAdmit, CONTROL_STREAM_ID};
use super::stream::{Stream, StreamConfig, StreamError, StreamStats};
use super::subnet::{SubnetId, SubnetPolicy};
use super::subprotocol::stream_window::{StreamWindow, SUBPROTOCOL_STREAM_WINDOW};
use super::subprotocol::MigrationSubprotocolHandler;
use super::transport::{NetSocket, PacketReceiver, ParsedPacket, SocketBufferConfig};
use super::Visibility;
use tokio::sync::oneshot;

use crate::adapter::{Adapter, ShardPollResult};
use crate::error::AdapterError;
use crate::event::{Batch, StoredEvent};

/// Inbound event queues (same type as NetAdapter uses).
type InboundQueues = Arc<DashMap<u16, SegQueue<StoredEvent>>>;

/// Convert a u64 node_id to a 32-byte graph NodeId.
///
/// The proximity graph uses 32-byte ed25519 public keys as NodeId.
/// For nodes where we only have the derived u64 node_id, we zero-pad
/// it to 32 bytes. This preserves uniqueness for topology tracking
/// without requiring the full public key exchange.
fn node_id_to_graph_id(node_id: u64) -> [u8; 32] {
    let mut id = [0u8; 32];
    id[0..8].copy_from_slice(&node_id.to_le_bytes());
    id
}

/// Inverse of `node_id_to_graph_id`: read the u64 back from the first 8
/// bytes of a 32-byte proximity `NodeId`. Assumes the id was produced by
/// `node_id_to_graph_id` (which is how every peer in this codebase is
/// seeded into the graph).
fn graph_id_to_node_id(graph_id: &[u8; 32]) -> u64 {
    u64::from_le_bytes(graph_id[0..8].try_into().unwrap())
}

/// Set of peer addresses whose packets should be silently dropped.
///
/// Used by test harnesses to simulate network partitions. When a peer's
/// address is in this set, both inbound and outbound packets are dropped
/// as if the network link is severed.
pub type PartitionFilter = Arc<dashmap::DashSet<SocketAddr>>;

/// Shared context for the packet dispatch loop.
struct DispatchCtx {
    local_node_id: u64,
    peers: Arc<DashMap<u64, PeerInfo>>,
    addr_to_node: Arc<DashMap<SocketAddr, u64>>,
    /// Node-id → addr map shared with the reroute policy. Must be kept in
    /// sync with `peers` on every registration so the reroute policy can
    /// resolve failed peers.
    peer_addrs: Arc<DashMap<u64, SocketAddr>>,
    router: Arc<NetRouter>,
    failure_detector: Arc<FailureDetector>,
    inbound: InboundQueues,
    num_shards: u16,
    /// Optional subprotocol handler for migration messages.
    migration_handler: Option<Arc<MigrationSubprotocolHandler>>,
    /// In-flight initiator handshakes; dispatch completes them when a
    /// matching routed msg2 arrives.
    pending_handshakes: Arc<DashMap<u64, PendingHandshake>>,
    /// Our Noise static keypair — needed to construct responder state
    /// when a routed msg1 arrives for us.
    static_keypair: StaticKeypair,
    /// PSK shared across the mesh.
    psk: [u8; 32],
    /// Socket for sending outbound subprotocol responses.
    socket: Arc<NetSocket>,
    /// Proximity graph for topology awareness.
    proximity_graph: Arc<ProximityGraph>,
    /// Partition filter — packets from blocked addresses are dropped.
    partition_filter: PartitionFilter,
    /// Settings for sessions we create during inbound dispatch (relayed
    /// handshake responder completes here).
    packet_pool_size: usize,
    default_reliable: bool,
    /// Subscriber roster for channel fan-out.
    roster: Arc<SubscriberRoster>,
    /// Channel config registry used to authorize incoming Subscribe.
    /// `None` disables channel-level ACL checks (any caller accepted).
    channel_configs: Option<Arc<ChannelConfigRegistry>>,
    /// In-flight Subscribe/Unsubscribe requests awaiting an Ack, keyed by nonce.
    pending_membership_acks: Arc<DashMap<u64, oneshot::Sender<MembershipAck>>>,
    /// Max distinct channels a single peer may subscribe to.
    max_channels_per_peer: usize,
    /// Capability index shared with `MeshNode`. Inbound
    /// `SUBPROTOCOL_CAPABILITY_ANN` packets are indexed here.
    capability_index: Arc<CapabilityIndex>,
    /// Dedup cache for multi-hop capability announcements, keyed by
    /// `(origin_node_id, version)`. Written by the dispatch handler
    /// before indexing + forwarding so a `(origin, version)` tuple
    /// is processed at most once per node.
    seen_announcements: Arc<DashMap<(u64, u64), std::time::Instant>>,
    /// Whether inbound `CapabilityAnnouncement` packets without a
    /// signature are dropped. Validity is not enforced yet.
    require_signed_capabilities: bool,
    /// This node's subnet (copy of `config.subnet`).
    local_subnet: SubnetId,
    /// Policy applied to each inbound `CapabilityAnnouncement` to
    /// derive the sender's subnet. `None` disables tracking.
    local_subnet_policy: Option<Arc<SubnetPolicy>>,
    /// Per-peer subnet map, written by the capability-announcement
    /// dispatch and read by the subscribe gate + publish fan-out.
    peer_subnets: Arc<DashMap<u64, SubnetId>>,
    /// Per-peer entity-id map, written by the capability-
    /// announcement dispatch after signature verification. Load-
    /// bearing for channel auth.
    peer_entity_ids: Arc<DashMap<u64, EntityId>>,
    /// Shared token cache, populated by subscriber-presented tokens
    /// plus caller-side pre-installs. `None` disables the
    /// `require_token` path — unset is equivalent to "no token is
    /// ever valid."
    token_cache: Option<Arc<TokenCache>>,
    /// Per-packet authorization fast path. `authorize_subscribe`
    /// writes on success (via `allow_channel`) so the publish
    /// fan-out can use the bloom filter + verified cache to admit
    /// or drop subscribers in constant time.
    auth_guard: Arc<AuthGuard>,
    /// Per-peer auth-failure state (for the subscribe rate limit).
    auth_failures: Arc<DashMap<u64, AuthFailureState>>,
    /// Failures-per-window threshold from the parent config.
    max_auth_failures_per_window: u16,
    /// Rolling window length for auth-failure counting.
    auth_failure_window: Duration,
    /// How long a peer stays throttled after tripping the threshold.
    auth_throttle_duration: Duration,
}

/// Result passed through the pending-ack oneshot.
#[derive(Debug, Clone)]
pub(crate) struct MembershipAck {
    pub accepted: bool,
    pub reason: Option<AckReason>,
}

/// Configuration for a MeshNode.
#[derive(Debug, Clone)]
pub struct MeshNodeConfig {
    /// Local bind address
    pub bind_addr: SocketAddr,
    /// Pre-shared key (32 bytes, shared across the mesh)
    pub psk: [u8; 32],
    /// Heartbeat interval for failure detection
    pub heartbeat_interval: Duration,
    /// Session timeout
    pub session_timeout: Duration,
    /// Number of shards for inbound event routing
    pub num_shards: u16,
    /// Packet pool size per session
    pub packet_pool_size: usize,
    /// Default reliability mode
    pub default_reliable: bool,
    /// Handshake timeout per attempt
    pub handshake_timeout: Duration,
    /// Handshake retries
    pub handshake_retries: usize,
    /// Socket buffer config
    pub socket_buffers: SocketBufferConfig,
    /// Max queue depth per stream for the fair scheduler.
    pub max_queue_depth: usize,
    /// Fair scheduling quantum (packets per stream per round).
    pub fair_quantum: usize,
    /// Idle timeout before a stream is evicted from its session. A
    /// stream with no send or receive activity for this long is dropped
    /// on the heartbeat-loop sweep. Protects against unbounded
    /// `StreamState` growth under workloads that hash into stream ids.
    pub stream_idle_timeout: Duration,
    /// Hard cap on the number of streams per session. When exceeded,
    /// the least-recently-active stream is evicted via the same path as
    /// `close_stream` (logged with `reason=cap_exceeded`).
    pub max_streams: usize,
    /// Max channels a single peer may subscribe to via
    /// `SUBPROTOCOL_CHANNEL_MEMBERSHIP`. Extra Subscribe requests are
    /// rejected with `AckReason::TooManyChannels`. Protects the roster
    /// from a peer that spams subscriptions.
    pub max_channels_per_peer: usize,
    /// Timeout for `subscribe_channel` / `unsubscribe_channel` to wait
    /// for an `Ack` before returning `AdapterError::Timeout`.
    pub membership_ack_timeout: Duration,
    /// Drop inbound `CapabilityAnnouncement` packets whose signature
    /// is missing. Defaults to `true` because the cap data feeds
    /// channel-auth (`can_publish` / `can_subscribe` cap filters)
    /// and subnet visibility — an unsigned announcement is
    /// attacker-controlled input, and accepting it silently meant a
    /// peer could claim any caps or subnet just by announcing. The
    /// dispatch path still applies a second belt-and-braces guard
    /// on individual auth-load-bearing state updates
    /// (`peer_entity_ids`, `peer_subnets`), so explicitly setting
    /// this to `false` for discovery-only deployments is
    /// defensible; flipping this on simply makes the rejection
    /// happen up-front instead of silently no-oping the state
    /// writes downstream.
    pub require_signed_capabilities: bool,
    /// How often the capability index sweeps expired entries. Low
    /// values waste CPU; high values keep stale peers queryable past
    /// their TTL.
    pub capability_gc_interval: Duration,
    /// This node's subnet. Defaults to [`SubnetId::GLOBAL`] — "no
    /// restriction." Visibility checks compare against this value on
    /// both the publish and subscribe paths.
    pub subnet: SubnetId,
    /// Policy applied to inbound [`CapabilityAnnouncement`]s to
    /// derive each peer's subnet. `None` disables per-peer subnet
    /// tracking; every peer is treated as `GLOBAL`, which in
    /// practice means `Visibility::SubnetLocal` channels ship only
    /// when both sides are `GLOBAL`.
    pub subnet_policy: Option<Arc<SubnetPolicy>>,
    /// Minimum time between successive
    /// [`MeshNode::announce_capabilities`] broadcasts from this
    /// origin. Calls within the window coalesce: the local index
    /// and `local_announcement` are updated so self-queries + late-
    /// joiner session-open pushes reflect the latest caps, but the
    /// network broadcast is skipped. Rate-limits apps that
    /// re-announce in tight loops.
    pub min_announce_interval: Duration,
    /// Period between `TokenCache` expiry sweeps. A subscriber
    /// whose token expires mid-subscription is evicted from the
    /// [`SubscriberRoster`] and revoked from the [`AuthGuard`]
    /// within one sweep interval. Set to [`Duration::MAX`] (or any
    /// value longer than the mesh's lifetime) to disable the
    /// sweep — publishes will still re-check the guard, so this
    /// mainly affects how quickly stale tokens drop off the
    /// roster.
    pub token_sweep_interval: Duration,
    /// Authorization-failure threshold per peer per window. A peer
    /// that exceeds this count across a rolling
    /// [`Self::auth_failure_window`] gets throttled — subsequent
    /// subscribes short-circuit with `AckReason::RateLimited` for
    /// [`Self::auth_throttle_duration`] without running the
    /// cap-filter + ed25519 path. Set to `u16::MAX` to disable.
    pub max_auth_failures_per_window: u16,
    /// Rolling window over which failed subscribes are counted for
    /// the throttle check above. Default: 60 s.
    pub auth_failure_window: Duration,
    /// How long a peer stays throttled after tripping the
    /// failure threshold. Default: 30 s.
    pub auth_throttle_duration: Duration,
}

impl MeshNodeConfig {
    /// Create with minimal required fields.
    pub fn new(bind_addr: SocketAddr, psk: [u8; 32]) -> Self {
        Self {
            bind_addr,
            psk,
            heartbeat_interval: Duration::from_secs(5),
            session_timeout: Duration::from_secs(30),
            num_shards: 4,
            packet_pool_size: 64,
            default_reliable: false,
            handshake_timeout: Duration::from_secs(5),
            handshake_retries: 3,
            socket_buffers: SocketBufferConfig::for_testing(),
            max_queue_depth: 1024,
            fair_quantum: 16,
            stream_idle_timeout: Duration::from_secs(300),
            max_streams: 4096,
            max_channels_per_peer: 1024,
            membership_ack_timeout: Duration::from_secs(5),
            require_signed_capabilities: true,
            capability_gc_interval: Duration::from_secs(60),
            subnet: SubnetId::GLOBAL,
            subnet_policy: None,
            min_announce_interval: Duration::from_secs(10),
            token_sweep_interval: Duration::from_secs(30),
            max_auth_failures_per_window: 16,
            auth_failure_window: Duration::from_secs(60),
            auth_throttle_duration: Duration::from_secs(30),
        }
    }

    /// Set heartbeat interval.
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Set session timeout.
    pub fn with_session_timeout(mut self, timeout: Duration) -> Self {
        self.session_timeout = timeout;
        self
    }

    /// Set number of shards.
    pub fn with_num_shards(mut self, n: u16) -> Self {
        self.num_shards = n;
        self
    }

    /// Set handshake timing.
    pub fn with_handshake(mut self, retries: usize, timeout: Duration) -> Self {
        self.handshake_retries = retries;
        self.handshake_timeout = timeout;
        self
    }

    /// Require inbound `CapabilityAnnouncement` packets to carry a
    /// signature. Unsigned announcements are dropped silently (a
    /// trace is emitted).
    pub fn with_require_signed_capabilities(mut self, require: bool) -> Self {
        self.require_signed_capabilities = require;
        self
    }

    /// Set the capability-index GC sweep interval.
    pub fn with_capability_gc_interval(mut self, interval: Duration) -> Self {
        self.capability_gc_interval = interval;
        self
    }

    /// Set the minimum interval between outbound capability-
    /// announcement broadcasts. See [`Self::min_announce_interval`].
    pub fn with_min_announce_interval(mut self, interval: Duration) -> Self {
        self.min_announce_interval = interval;
        self
    }

    /// Set the token-expiry sweep interval. See
    /// [`Self::token_sweep_interval`].
    pub fn with_token_sweep_interval(mut self, interval: Duration) -> Self {
        self.token_sweep_interval = interval;
        self
    }

    /// Tune the per-peer authorization-failure rate limit. See
    /// [`Self::max_auth_failures_per_window`].
    pub fn with_auth_failure_limit(
        mut self,
        max_per_window: u16,
        window: Duration,
        throttle: Duration,
    ) -> Self {
        self.max_auth_failures_per_window = max_per_window;
        self.auth_failure_window = window;
        self.auth_throttle_duration = throttle;
        self
    }

    /// Pin this node to a specific subnet.
    pub fn with_subnet(mut self, subnet: SubnetId) -> Self {
        self.subnet = subnet;
        self
    }

    /// Derive each peer's subnet locally by applying this policy to
    /// their inbound [`CapabilityAnnouncement`]s. Mesh-wide policy
    /// consistency is assumed; mismatched policies lead to
    /// asymmetric views of peer subnets.
    pub fn with_subnet_policy(mut self, policy: Arc<SubnetPolicy>) -> Self {
        self.subnet_policy = Some(policy);
        self
    }
}

/// Peer connection info.
struct PeerInfo {
    /// Node ID (derived from keypair or assigned)
    node_id: u64,
    /// Address used for direct sends. For peers reached via a relay, this
    /// is the relay's address — packets to the destination go there first.
    addr: SocketAddr,
    /// Encrypted session
    session: Arc<NetSession>,
}

/// In-flight initiator handshake. The dispatch loop consumes this when a
/// routed msg2 arrives for `peer_node_id`: it pulls the Noise state out,
/// runs `read_message`, derives the session keys, and signals the
/// awaiting `connect_via` caller via the oneshot.
///
/// Keyed in `pending_handshakes` by `peer_node_id as u32 as u64` because
/// the routing header's `src_id` field is only 32 bits — msg2's routing
/// header carries the truncated value, so the dispatch loop can only
/// look up by that. The full `u64` is stored here for peer registration.
struct PendingHandshake {
    noise: NoiseHandshake,
    tx: oneshot::Sender<Result<SessionKeys, CryptoError>>,
}

/// 32-bit "routing identity" projection of a `u64` node_id, used as the
/// key across the routing plane (routing header's `src_id` is `u32`).
/// Encoded back into a `u64` as the low 32 bits, high bits zero, so the
/// same projection is visible on both sides of a routed packet.
#[inline]
fn routing_id(node_id: u64) -> u64 {
    (node_id as u32) as u64
}

/// 64-bit origin-hash projection used as the `AuthGuard` key.
///
/// A prior version truncated to `u32` so the key matched the
/// routing-plane's 32-bit `src_id`, but truncating to 32 bits
/// birthday-collides at ~65 k peers — inside the practical reach of
/// a medium mesh — and lets one subscriber's grant admit a different
/// subscriber's packets. The fan-out fast path keys on the full
/// 64-bit `node_id` (the value it already has in hand), which pushes
/// the collision floor out of reach. The `src_id` field on wire
/// packets is not consulted for authorization.
#[inline]
fn subscriber_origin_hash(node_id: u64) -> u64 {
    node_id
}

/// Replace a zero `Duration` with a 1 s floor.
/// `tokio::time::interval` panics on a zero period; this is the
/// guard every `spawn_*_loop` call site applies before handing the
/// caller-configured interval to tokio. A legitimate "disable this
/// timer" sentinel is `Duration::MAX`, documented on the relevant
/// config fields — `Duration::ZERO` is just pathological input.
///
/// The floor is deliberately coarse (1 s, not 1 ms): a mis-configured
/// zero-interval used to spin the maintenance loop at 1 kHz, calling
/// `index.gc()` + `seen.retain()` a thousand times a second and
/// burning a whole core. 1 Hz keeps the loop obviously alive for
/// observability without an appreciable CPU cost, and is still
/// finer than the default intervals (capability GC ≈ announcement
/// TTL, token sweep 30 s), so it never masks a legitimate
/// fine-grained config — those values are already well above 1 s.
#[inline]
fn nonzero_interval(d: Duration) -> Duration {
    if d.is_zero() {
        Duration::from_secs(1)
    } else {
        d
    }
}

/// Rolling-window auth-failure tracker, one entry per peer.
/// Lives behind a per-key `Mutex` so updates from concurrent
/// subscribes don't race each other on the same peer's counter.
#[derive(Debug, Default)]
struct AuthFailureState {
    /// Failures accumulated inside the current window.
    failures: u16,
    /// Start of the window. Resets to `Instant::now()` once
    /// `auth_failure_window` has elapsed since the current window
    /// opened.
    window_start: Option<std::time::Instant>,
    /// If set, the peer is throttled until this instant and every
    /// subscribe short-circuits with `RateLimited`.
    throttled_until: Option<std::time::Instant>,
}

/// Evict subscribers whose tokens have expired. Walks the roster by
/// peer and, for every `require_token` channel they hold, runs the
/// full token-cache check. Expired entries are revoked in the
/// [`AuthGuard`] and removed from the [`SubscriberRoster`].
///
/// Skip conditions (short-circuits to no-op):
///
/// - No `token_cache`: `require_token` channels reject every
///   subscribe anyway, so the roster contains no token-gated
///   entries.
/// - No `channel_configs`: nothing to check `require_token`
///   against, so every roster entry is treated as open and left
///   alone.
///
/// Pulled into a free fn (not a method) so the sweep loop can
/// call it without capturing `&self` through the async closure.
fn sweep_expired_subscribers(
    roster: &SubscriberRoster,
    guard: &AuthGuard,
    token_cache: Option<&Arc<TokenCache>>,
    peer_entity_ids: &DashMap<u64, EntityId>,
    channel_configs: Option<&Arc<ChannelConfigRegistry>>,
) {
    let (Some(cache), Some(configs)) = (token_cache, channel_configs) else {
        return;
    };
    // Snapshot (node_id, entity_id) pairs so we don't hold the
    // DashMap read guard across the token checks below.
    let peers: Vec<(u64, EntityId)> = peer_entity_ids
        .iter()
        .map(|e| (*e.key(), e.value().clone()))
        .collect();
    for (node_id, entity_id) in peers {
        for channel_id in roster.channels_for(node_id) {
            let name = channel_id.name();
            let Some(cfg) = configs.get_by_name(name.as_str()) else {
                continue;
            };
            if !cfg.require_token {
                continue;
            }
            // `check` validates signature + time bounds. Any error
            // (expired, not_yet_valid, invalid_signature, not_authorized)
            // means this subscriber is no longer authorized.
            let authorized = cache
                .check(
                    &entity_id,
                    super::identity::TokenScope::SUBSCRIBE,
                    name.hash(),
                )
                .is_ok();
            if !authorized {
                guard.revoke_channel(subscriber_origin_hash(node_id), name);
                roster.remove(&channel_id, node_id);
                tracing::debug!(
                    node_id = format!("{:#x}", node_id),
                    channel = name.as_str(),
                    "auth: evicted subscriber with expired/invalid token",
                );
            }
        }
    }
}

/// Default TTL for the routing header we stamp on routed handshake
/// packets. Far above any realistic relay chain; the routing layer
/// drops at zero.
const DEFAULT_HANDSHAKE_TTL: u8 = 16;

/// Maximum hop count a pingwave may carry on receipt. Pingwaves with
/// `hop_count >= MAX_HOPS` are dropped — they install no route, no
/// graph edge, and are not re-broadcast. TTL bounds forwarding at the
/// emitter; `MAX_HOPS` is the receive-time counterpart that prevents
/// an inflated-hop-count advertisement (malicious or buggy) from
/// populating the routing table with an arbitrarily-distant entry.
/// Value sized to accommodate the largest plausibly-useful mesh depth
/// while still bounding count-to-infinity worst cases.
const MAX_HOPS: u8 = 16;

/// Multi-peer mesh node.
///
/// Composes `NetSession` (per-peer encryption), `NetRouter` (forwarding),
/// and `FailureDetector` (heartbeat monitoring) behind a single UDP socket.
pub struct MeshNode {
    /// This node's identity (ed25519, for signing and node_id derivation).
    /// Used in Phase 3 for subprotocol message signing.
    #[allow(dead_code)]
    identity: EntityKeypair,
    /// Noise static keypair (Curve25519, for handshakes)
    static_keypair: StaticKeypair,
    /// Derived node ID
    node_id: u64,
    /// Configuration
    config: MeshNodeConfig,
    /// Shared UDP socket
    socket: Arc<NetSocket>,
    /// Per-peer sessions keyed by node_id. Keying by node_id (rather than
    /// SocketAddr) is required for relayed sessions: if A connects to C via
    /// relay B, both peers share B's wire address, so a SocketAddr-keyed map
    /// would overwrite B's session with C's.
    peers: Arc<DashMap<u64, PeerInfo>>,
    /// Reverse lookup for dispatch: incoming source address → node_id. Only
    /// populated for directly-connected peers; relayed peers are resolved by
    /// session_id during dispatch.
    addr_to_node: Arc<DashMap<SocketAddr, u64>>,
    /// Router for forwarding decisions
    router: Arc<NetRouter>,
    /// Failure detector
    failure_detector: Arc<FailureDetector>,
    /// Inbound event queues (shared with receive loop)
    inbound: InboundQueues,
    /// Optional migration subprotocol handler
    migration_handler: Option<Arc<MigrationSubprotocolHandler>>,
    /// In-flight routed-handshake initiators, keyed by the responder's
    /// node_id. Populated by `connect_via`; consumed by the dispatch
    /// loop when the matching msg2 arrives.
    pending_handshakes: Arc<DashMap<u64, PendingHandshake>>,
    /// Proximity graph — topology awareness from pingwave propagation
    proximity_graph: Arc<ProximityGraph>,
    /// Automatic reroute policy
    reroute_policy: Arc<ReroutePolicy>,
    /// Node ID → SocketAddr map (shared with reroute policy)
    peer_addrs: Arc<DashMap<u64, SocketAddr>>,
    /// Partition filter for simulating network splits
    partition_filter: PartitionFilter,
    /// Per-channel subscriber roster (daemon-layer fan-out).
    roster: Arc<SubscriberRoster>,
    /// Channel config registry consulted by incoming `Subscribe` packets
    /// for ACL decisions. When `None`, ACL is bypassed and all subscribes
    /// are accepted — used by tests and by nodes that don't run channels.
    channel_configs: Option<Arc<ChannelConfigRegistry>>,
    /// In-flight Subscribe/Unsubscribe requests keyed by nonce.
    pending_membership_acks: Arc<DashMap<u64, oneshot::Sender<MembershipAck>>>,
    /// Capability index populated by inbound
    /// `SUBPROTOCOL_CAPABILITY_ANN` packets and the local
    /// `announce_capabilities` path. Self-index so single-node
    /// queries return us too.
    capability_index: Arc<CapabilityIndex>,
    /// Dedup cache for multi-hop capability announcements. Keyed by
    /// `(origin_node_id, version)` — the same discriminator
    /// `CapabilityIndex` uses to skip stale announcements. Entries
    /// are evicted by the capability GC loop once their
    /// announcement's effective lifetime (2× `ttl_secs`) has
    /// elapsed. Mirrors the `seen_pingwaves` cache in
    /// [`ProximityGraph`].
    seen_announcements: Arc<DashMap<(u64, u64), std::time::Instant>>,
    /// Timestamp of the most recent outbound capability-announcement
    /// broadcast from this origin. Compared against
    /// `config.min_announce_interval` on every `announce_capabilities_with`
    /// call; within-window calls coalesce to a local self-index
    /// update without a network broadcast.
    last_announce_at: Arc<parking_lot::Mutex<Option<std::time::Instant>>>,
    /// Most recent `CapabilityAnnouncement` this node published.
    /// Pushed to new peers right after `accept` / `connect`
    /// completes, so late joiners pick up our caps without waiting
    /// for a re-announce. `None` until the first `announce_*` call.
    local_announcement: Arc<ArcSwapOption<CapabilityAnnouncement>>,
    /// Monotonic version counter used when stamping our own
    /// announcements. `CapabilityIndex::index` skips older versions,
    /// so this must move forward across restarts if the caller wants
    /// their announcements accepted.
    capability_version: Arc<AtomicU64>,
    /// This node's subnet. Copy of `config.subnet`, hoisted to the
    /// top level because the publish + subscribe hot paths read it
    /// without going through the config struct.
    local_subnet: SubnetId,
    /// Subnet policy applied to inbound `CapabilityAnnouncement`s.
    /// `None` disables per-peer subnet tracking.
    local_subnet_policy: Option<Arc<SubnetPolicy>>,
    /// Per-peer subnet map. Keys are `node_id`; values are the
    /// subnet derived from each peer's most recent announcement via
    /// `local_subnet_policy`. Unknown peers default to
    /// [`SubnetId::GLOBAL`] at read time.
    peer_subnets: Arc<DashMap<u64, SubnetId>>,
    /// Per-peer entity-id map. Keys are `node_id`; values are the
    /// 32-byte ed25519 public key carried on the peer's most recent
    /// `CapabilityAnnouncement`. Load-bearing for channel auth —
    /// without it, `require_token` channels can't match a token's
    /// `subject` to the subscribing peer.
    peer_entity_ids: Arc<DashMap<u64, EntityId>>,
    /// Shared token cache used by the channel-auth path. When
    /// `None`, `can_publish` / `can_subscribe` fall back to a
    /// fresh empty cache — which means `require_token` channels
    /// always reject. SDK builders wire this up from the caller's
    /// `Identity`.
    token_cache: Option<Arc<TokenCache>>,
    /// Per-packet authorization fast path. Populated when a
    /// subscribe clears `authorize_subscribe`; consulted on every
    /// publish fan-out via `check_fast`. The bloom filter + verified
    /// cache keep authorization at O(1) without per-packet
    /// signature verification. See
    /// [`docs/CHANNEL_AUTH_GUARD_PLAN.md`](../../../../docs/CHANNEL_AUTH_GUARD_PLAN.md).
    auth_guard: Arc<AuthGuard>,
    /// Per-peer auth-failure tracker. Counts failed
    /// `authorize_subscribe` attempts per `auth_failure_window` and
    /// throttles bursts — peers that exceed
    /// `max_auth_failures_per_window` short-circuit with
    /// `RateLimited` for `auth_throttle_duration` without running
    /// the cap-filter + ed25519 verify path. Successful subscribes
    /// clear the counter for that peer.
    auth_failures: Arc<DashMap<u64, AuthFailureState>>,
    /// Background tasks
    tasks: Arc<tokio::sync::Mutex<Vec<JoinHandle<()>>>>,
    /// Shutdown flag
    shutdown: Arc<AtomicBool>,
    /// Shutdown notifier
    shutdown_notify: Arc<Notify>,
    /// Whether the node has been started
    started: AtomicBool,
}

impl MeshNode {
    /// Get the Noise static public key (for peers to connect to this node).
    pub fn public_key(&self) -> &[u8; 32] {
        &self.static_keypair.public
    }

    /// Whether [`Self::shutdown`] has been invoked on this node.
    ///
    /// Exposed for tests and for FFI callers that want to verify a
    /// shutdown actually landed (rather than being a silent no-op
    /// because extra `Arc` references were outstanding, as an earlier
    /// `net_mesh_shutdown` variant did).
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Acquire)
    }

    /// Create a new mesh node.
    ///
    /// Binds a UDP socket but does not connect to any peers yet.
    /// Call `connect()` to establish sessions with peers, then
    /// `start()` to begin the receive loop.
    pub async fn new(
        identity: EntityKeypair,
        config: MeshNodeConfig,
    ) -> Result<Self, AdapterError> {
        let node_id = identity.node_id();
        let static_keypair = StaticKeypair::generate();

        let socket = NetSocket::with_config(config.bind_addr, config.socket_buffers)
            .await
            .map_err(|e| AdapterError::Connection(format!("bind failed: {}", e)))?;
        let socket = Arc::new(socket);

        let router_config = RouterConfig {
            local_id: node_id,
            // Router binds to an ephemeral port for its send loop. It uses
            // this socket only for forwarding packets — the main socket
            // handles all receives.
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            max_queue_depth: config.max_queue_depth,
            fair_quantum: config.fair_quantum,
            ..Default::default()
        };
        let router = NetRouter::new(router_config)
            .await
            .map_err(|e| AdapterError::Connection(format!("router bind failed: {}", e)))?;

        let router = Arc::new(router);

        // Configure route staleness. Routes learned from pingwaves age
        // out if a fresh pingwave hasn't refreshed them in this window;
        // direct routes are refreshed by the heartbeat loop, so they
        // stay fresh as long as the session is alive.
        router
            .routing_table()
            .set_max_route_age(config.session_timeout.saturating_mul(3));

        let peer_addrs: Arc<DashMap<u64, SocketAddr>> = Arc::new(DashMap::new());

        // Create proximity graph for topology awareness.
        //
        // Peers are seeded into the graph via `node_id_to_graph_id(peer_node_id)`
        // (see `connect`/`accept`). The local node must use the *same*
        // encoding or path lookups between local and peers would miss —
        // `entity_id().as_bytes()` would put this node under a different
        // key than what peers see for it.
        let graph_node_id = node_id_to_graph_id(node_id);
        let proximity_graph = Arc::new(ProximityGraph::new(
            graph_node_id,
            ProximityConfig::default(),
        ));

        // Create reroute policy with proximity graph for topology-aware alternates
        let reroute_policy = Arc::new(
            ReroutePolicy::new(router.routing_table().clone(), peer_addrs.clone())
                .with_proximity_graph(proximity_graph.clone()),
        );

        // Subscriber roster for channel fan-out; also wired into the
        // failure-detector `on_failure` callback so that a peer going
        // Failed is removed from every channel it was subscribed to.
        let roster: Arc<SubscriberRoster> = Arc::new(SubscriberRoster::new());

        // Peer-subnet map (Stage D). Populated when inbound
        // `SUBPROTOCOL_CAPABILITY_ANN` packets arrive and the local
        // `SubnetPolicy` derives a subnet for the sender. Created
        // here so the failure callback can evict stale entries on
        // session loss — otherwise reconnects would silently reuse
        // the old subnet until the next announcement arrived.
        let peer_subnets: Arc<DashMap<u64, SubnetId>> = Arc::new(DashMap::new());
        // Peer entity-id map (Stage E). Populated from each inbound
        // `CapabilityAnnouncement`. Evicted alongside `peer_subnets`
        // on session failure so a reconnect doesn't silently reuse
        // the old identity.
        let peer_entity_ids: Arc<DashMap<u64, EntityId>> = Arc::new(DashMap::new());

        // Wire failure detector with reroute callbacks + roster eviction.
        let rp_failure = reroute_policy.clone();
        let rp_recovery = reroute_policy.clone();
        let roster_failure = roster.clone();
        let peer_subnets_failure = peer_subnets.clone();
        let peer_entity_ids_failure = peer_entity_ids.clone();
        let failure_detector = FailureDetector::with_config(FailureDetectorConfig {
            timeout: config.session_timeout,
            miss_threshold: 3,
            suspicion_threshold: 2,
            cleanup_interval: Duration::from_secs(60),
        })
        .on_failure(move |node_id| {
            rp_failure.on_failure(node_id);
            let removed = roster_failure.remove_peer(node_id);
            if !removed.is_empty() {
                tracing::debug!(
                    node_id = format!("{:#x}", node_id),
                    channels = removed.len(),
                    "roster: evicted failed peer from channels"
                );
            }
            peer_subnets_failure.remove(&node_id);
            peer_entity_ids_failure.remove(&node_id);
        })
        .on_recovery(move |node_id| rp_recovery.on_recovery(node_id));

        let pending_handshakes: Arc<DashMap<u64, PendingHandshake>> = Arc::new(DashMap::new());

        // Hoist the subnet knobs before `config` is moved into the
        // struct literal; the publish + subscribe paths read these
        // without going back through `config`.
        let local_subnet = config.subnet;
        let local_subnet_policy = config.subnet_policy.clone();

        Ok(Self {
            identity,
            static_keypair,
            node_id,
            config,
            socket,
            peers: Arc::new(DashMap::new()),
            addr_to_node: Arc::new(DashMap::new()),
            router,
            failure_detector: Arc::new(failure_detector),
            inbound: Arc::new(DashMap::new()),
            migration_handler: None,
            pending_handshakes,
            proximity_graph,
            reroute_policy,
            peer_addrs,
            partition_filter: Arc::new(dashmap::DashSet::new()),
            roster,
            channel_configs: None,
            pending_membership_acks: Arc::new(DashMap::new()),
            capability_index: Arc::new(CapabilityIndex::new()),
            seen_announcements: Arc::new(DashMap::new()),
            last_announce_at: Arc::new(parking_lot::Mutex::new(None)),
            local_announcement: Arc::new(ArcSwapOption::empty()),
            capability_version: Arc::new(AtomicU64::new(0)),
            local_subnet,
            local_subnet_policy,
            peer_subnets,
            peer_entity_ids,
            token_cache: None,
            auth_guard: Arc::new(AuthGuard::new()),
            auth_failures: Arc::new(DashMap::new()),
            tasks: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            shutdown: Arc::new(AtomicBool::new(false)),
            shutdown_notify: Arc::new(Notify::new()),
            started: AtomicBool::new(false),
        })
    }

    /// Get this node's ID.
    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    /// The per-packet authorization fast path. Writes land here on
    /// successful subscribe (via `AuthGuard::allow_channel`) and
    /// reads happen on every publish fan-out. Exposed primarily for
    /// tests + operator observability; production code should reach
    /// for `register_channel` / `subscribe_channel` instead.
    pub fn auth_guard(&self) -> &Arc<AuthGuard> {
        &self.auth_guard
    }

    /// The shared `TokenCache` installed on this node, if any. Only
    /// populated when a caller registered one via
    /// [`Self::set_token_cache`]. Exposed for tests that need to
    /// assert the cache is *not* populated as a side effect of a
    /// rejected subscribe.
    pub fn token_cache(&self) -> Option<&Arc<TokenCache>> {
        self.token_cache.as_ref()
    }

    /// Get this node's ed25519 entity id (derived from the
    /// keypair handed to `MeshNode::new`). 32 bytes. Used by
    /// `CapabilityAnnouncement` + channel-auth path.
    pub fn entity_id(&self) -> &EntityId {
        self.identity.entity_id()
    }

    /// Look up a peer's pinned `entity_id`, if the TOFU binding
    /// has been established. Returns `None` before we've received
    /// a signature-verified `CapabilityAnnouncement` from the peer.
    /// Exposed primarily for tests + operator observability; the
    /// channel-auth subscribe gate consults this map internally.
    pub fn peer_entity_id(&self, node_id: u64) -> Option<EntityId> {
        self.peer_entity_ids
            .get(&node_id)
            .map(|e| e.value().clone())
    }

    /// Look up a peer's assigned subnet, if one has been recorded.
    /// Only populated from signature-verified
    /// `CapabilityAnnouncement`s — unsigned announcements do not
    /// write here even when a node is running with
    /// `require_signed_capabilities = false`. Exposed for tests +
    /// operator observability; `subnet_visible` consults this map
    /// on the publish / subscribe fan-out path.
    pub fn peer_subnet(&self, node_id: u64) -> Option<SubnetId> {
        self.peer_subnets.get(&node_id).map(|e| *e.value())
    }

    /// Get the local bind address.
    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr()
    }

    /// Get the router (for adding routes, checking stats).
    pub fn router(&self) -> &Arc<NetRouter> {
        &self.router
    }

    /// Get the failure detector.
    pub fn failure_detector(&self) -> &Arc<FailureDetector> {
        &self.failure_detector
    }

    /// Set the migration subprotocol handler.
    ///
    /// Must be called before `start()`. When set, inbound packets with
    /// `subprotocol_id == 0x0500` are dispatched to this handler instead
    /// of being queued as events.
    pub fn set_migration_handler(&mut self, handler: Arc<MigrationSubprotocolHandler>) {
        self.migration_handler = Some(handler);
    }

    /// Block packets from/to a peer address (simulates network partition).
    pub fn block_peer(&self, addr: SocketAddr) {
        self.partition_filter.insert(addr);
    }

    /// Unblock a peer address (simulates partition healing).
    pub fn unblock_peer(&self, addr: &SocketAddr) {
        self.partition_filter.remove(addr);
    }

    /// Check if a peer is blocked.
    pub fn is_blocked(&self, addr: &SocketAddr) -> bool {
        self.partition_filter.contains(addr)
    }

    /// Get the proximity graph.
    pub fn proximity_graph(&self) -> &Arc<ProximityGraph> {
        &self.proximity_graph
    }

    /// Get the reroute policy (for checking reroute stats in tests).
    pub fn reroute_policy(&self) -> &Arc<ReroutePolicy> {
        &self.reroute_policy
    }

    /// Number of connected peers.
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Connect to a peer. Performs a Noise NKpsk0 handshake as initiator.
    ///
    /// The peer must be listening and ready to accept the handshake.
    /// Returns the peer's node ID on success.
    pub async fn connect(
        &self,
        peer_addr: SocketAddr,
        peer_pubkey: &[u8; 32],
        peer_node_id: u64,
    ) -> Result<u64, AdapterError> {
        let keys = self
            .handshake_initiator(peer_addr, peer_pubkey, peer_node_id)
            .await?;

        let session = Arc::new(NetSession::new(
            keys,
            peer_addr,
            self.config.packet_pool_size,
            self.config.default_reliable,
        ));

        // Add route so the router can forward packets to this peer
        self.router.add_route(peer_node_id, peer_addr);

        self.peers.insert(
            peer_node_id,
            PeerInfo {
                node_id: peer_node_id,
                addr: peer_addr,
                session,
            },
        );
        self.addr_to_node.insert(peer_addr, peer_node_id);

        // Register in peer address map (used by reroute policy)
        self.peer_addrs.insert(peer_node_id, peer_addr);

        // Register in proximity graph (1-hop peer). The synthetic
        // pingwave's `origin == peer`, so the back-compat shim
        // attributes the edge correctly (`from_node = origin`).
        let peer_graph_id = node_id_to_graph_id(peer_node_id);
        let pw = EnhancedPingwave::new(peer_graph_id, 0, 1).with_load(0, HealthStatus::Healthy);
        self.proximity_graph.on_pingwave(pw, peer_addr);

        // Register with failure detector
        self.failure_detector.heartbeat(peer_node_id, peer_addr);

        // Push our most recent capability announcement to the new peer,
        // so late joiners pick up our caps without waiting for a
        // re-announce. Races the session's first inbound packet but
        // that's harmless: the receiver's `index()` is version-skip
        // safe and DashMap inserts are idempotent.
        self.push_local_announcement(peer_addr).await;

        Ok(peer_node_id)
    }

    /// Accept a connection from a peer. Performs Noise NKpsk0 as responder.
    ///
    /// Waits for an incoming handshake packet and completes the handshake.
    /// Returns the peer's address and assigns the given node_id.
    pub async fn accept(&self, peer_node_id: u64) -> Result<(SocketAddr, u64), AdapterError> {
        let (keys, peer_addr) = self.handshake_responder(peer_node_id).await?;

        let session = Arc::new(NetSession::new(
            keys,
            peer_addr,
            self.config.packet_pool_size,
            self.config.default_reliable,
        ));

        self.router.add_route(peer_node_id, peer_addr);

        self.peers.insert(
            peer_node_id,
            PeerInfo {
                node_id: peer_node_id,
                addr: peer_addr,
                session,
            },
        );
        self.addr_to_node.insert(peer_addr, peer_node_id);

        self.peer_addrs.insert(peer_node_id, peer_addr);

        let peer_graph_id = node_id_to_graph_id(peer_node_id);
        let pw = EnhancedPingwave::new(peer_graph_id, 0, 1).with_load(0, HealthStatus::Healthy);
        self.proximity_graph.on_pingwave(pw, peer_addr);

        self.failure_detector.heartbeat(peer_node_id, peer_addr);

        // See the matching comment in `connect`.
        self.push_local_announcement(peer_addr).await;

        Ok((peer_addr, peer_node_id))
    }

    /// Start the receive loop and heartbeat tasks.
    ///
    /// Must be called after `connect()` / `accept()` to begin processing
    /// inbound packets.
    pub fn start(&self) {
        if self.started.swap(true, Ordering::AcqRel) {
            return; // already started
        }

        let recv_handle = self.spawn_receive_loop();
        let heartbeat_handle = self.spawn_heartbeat_loop();
        let router_handle = self.router.start();
        let capability_gc_handle = self.spawn_capability_gc_loop();
        let token_sweep_handle = self.spawn_token_sweep_loop();

        // Store handles — can't block here, but we need them for shutdown
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            let mut tasks = tasks.lock().await;
            tasks.push(recv_handle);
            tasks.push(heartbeat_handle);
            tasks.push(router_handle);
            tasks.push(capability_gc_handle);
            tasks.push(token_sweep_handle);
        });
    }

    /// Spawn a periodic sweep that evicts expired entries from the
    /// capability index plus stale `(origin, version)` tuples from
    /// the multi-hop dedup cache. Interval from
    /// `config.capability_gc_interval` (default 60 s). Exits on
    /// `shutdown_notify`.
    fn spawn_capability_gc_loop(&self) -> JoinHandle<()> {
        let index = self.capability_index.clone();
        let seen = self.seen_announcements.clone();
        let interval = self.config.capability_gc_interval;
        // Dedup retention = 2× the announcement's own TTL. Longer
        // than one announcement lifetime so the re-announced bumped
        // version isn't confused with the previous one; shorter than
        // `index.gc` retention so the dedup set never outlives the
        // index it guards.
        let dedup_retention =
            std::time::Duration::from_secs(2 * u64::from(CapabilityAnnouncement::DEFAULT_TTL_SECS));
        let shutdown = self.shutdown.clone();
        let shutdown_notify = self.shutdown_notify.clone();

        tokio::spawn(async move {
            // `tokio::time::interval` panics on a zero period — guard
            // against a caller who set `capability_gc_interval` to
            // `Duration::ZERO` in `MeshNodeConfig`. A zero interval
            // isn't a sentinel for "disabled" (that's
            // `Duration::MAX`), it's just pathological input; clamp
            // to 1 s (see `nonzero_interval` for the rationale).
            let mut tick = tokio::time::interval(nonzero_interval(interval));
            // First tick fires immediately; skip it so we don't GC
            // empty state before any announcements have landed.
            tick.tick().await;
            while !shutdown.load(Ordering::Acquire) {
                tokio::select! {
                    _ = tick.tick() => {
                        let _removed = index.gc();
                        seen.retain(|_, instant| instant.elapsed() < dedup_retention);
                    }
                    _ = shutdown_notify.notified() => break,
                }
            }
        })
    }

    /// Spawn a periodic sweep that evicts subscribers whose tokens
    /// have expired. Walks the roster by peer (via
    /// `peer_entity_ids`) and, for every `require_token` channel the
    /// peer is subscribed to, runs the full token-cache check. An
    /// expired or invalid entry causes:
    ///
    /// 1. Revocation in the [`AuthGuard`] (so the next publish
    ///    fan-out sees the denial instantly, before the next
    ///    sweep tick).
    /// 2. Removal from the [`SubscriberRoster`] (so `members()`
    ///    returns the pruned list).
    ///
    /// Interval from `config.token_sweep_interval` (default 30 s).
    /// Skipped when the `channel_configs` registry is `None` — a
    /// node without a registry has no `require_token` channels to
    /// begin with. Similarly, skipped when `token_cache` is `None`.
    fn spawn_token_sweep_loop(&self) -> JoinHandle<()> {
        let roster = self.roster.clone();
        let guard = self.auth_guard.clone();
        let cache = self.token_cache.clone();
        let peer_entity_ids = self.peer_entity_ids.clone();
        let channel_configs = self.channel_configs.clone();
        let interval = self.config.token_sweep_interval;
        let shutdown = self.shutdown.clone();
        let shutdown_notify = self.shutdown_notify.clone();

        tokio::spawn(async move {
            // Same zero-guard as `spawn_capability_gc_loop` —
            // tokio panics if the period is zero.
            let mut tick = tokio::time::interval(nonzero_interval(interval));
            // First tick fires immediately; skip it so we don't
            // sweep empty state before any subscribes have landed.
            tick.tick().await;
            while !shutdown.load(Ordering::Acquire) {
                tokio::select! {
                    _ = tick.tick() => {
                        sweep_expired_subscribers(
                            &roster,
                            &guard,
                            cache.as_ref(),
                            &peer_entity_ids,
                            channel_configs.as_ref(),
                        );
                    }
                    _ = shutdown_notify.notified() => break,
                }
            }
        })
    }

    /// Spawn the main receive loop.
    ///
    /// This is the heart of the mesh node. Every packet from every peer
    /// arrives here. The loop:
    /// 1. Looks up the session by source address
    /// 2. For local packets: decrypts and queues events
    /// 3. For forwarded packets: passes to router (no decryption)
    /// 4. For heartbeats: updates failure detector
    fn spawn_receive_loop(&self) -> JoinHandle<()> {
        let socket = self.socket.socket_arc();
        let shutdown = self.shutdown.clone();
        let shutdown_notify = self.shutdown_notify.clone();

        let ctx = DispatchCtx {
            local_node_id: self.node_id,
            peers: self.peers.clone(),
            addr_to_node: self.addr_to_node.clone(),
            peer_addrs: self.peer_addrs.clone(),
            router: self.router.clone(),
            failure_detector: self.failure_detector.clone(),
            inbound: self.inbound.clone(),
            num_shards: self.config.num_shards,
            migration_handler: self.migration_handler.clone(),
            pending_handshakes: self.pending_handshakes.clone(),
            static_keypair: self.static_keypair.clone(),
            psk: self.config.psk,
            socket: self.socket.clone(),
            proximity_graph: self.proximity_graph.clone(),
            partition_filter: self.partition_filter.clone(),
            packet_pool_size: self.config.packet_pool_size,
            default_reliable: self.config.default_reliable,
            roster: self.roster.clone(),
            channel_configs: self.channel_configs.clone(),
            pending_membership_acks: self.pending_membership_acks.clone(),
            max_channels_per_peer: self.config.max_channels_per_peer,
            capability_index: self.capability_index.clone(),
            seen_announcements: self.seen_announcements.clone(),
            require_signed_capabilities: self.config.require_signed_capabilities,
            local_subnet: self.local_subnet,
            local_subnet_policy: self.local_subnet_policy.clone(),
            peer_subnets: self.peer_subnets.clone(),
            peer_entity_ids: self.peer_entity_ids.clone(),
            token_cache: self.token_cache.clone(),
            auth_guard: self.auth_guard.clone(),
            auth_failures: self.auth_failures.clone(),
            max_auth_failures_per_window: self.config.max_auth_failures_per_window,
            auth_failure_window: self.config.auth_failure_window,
            auth_throttle_duration: self.config.auth_throttle_duration,
        };

        tokio::spawn(async move {
            let mut receiver = PacketReceiver::new(socket);

            while !shutdown.load(Ordering::Acquire) {
                tokio::select! {
                    result = receiver.recv() => {
                        match result {
                            Ok((data, source)) => {
                                Self::dispatch_packet(data, source, &ctx);
                            }
                            Err(e) => {
                                if !shutdown.load(Ordering::Acquire) {
                                    tracing::warn!(error = %e, "mesh receive error");
                                }
                            }
                        }
                    }
                    _ = shutdown_notify.notified() => {
                        break;
                    }
                }
            }
        })
    }

    /// Dispatch a single received packet.
    ///
    /// This is the routing decision point:
    /// - Handshake packets are ignored (handled during connect/accept)
    /// - Heartbeat packets update the failure detector
    /// - Data packets are decrypted if local, forwarded if not
    fn dispatch_packet(data: Bytes, source: SocketAddr, ctx: &DispatchCtx) {
        // Partition filter: silently drop packets from blocked peers
        if ctx.partition_filter.contains(&source) {
            return;
        }

        // Check for pingwave. Pingwaves are a fixed 72-byte wire format
        // that does NOT carry the Net header magic. We reject anything
        // that starts with `MAGIC` so a legitimate Net packet that happens
        // to be 72 bytes is never mis-handled (defense in depth — the
        // current Net packet layout has an 80-byte minimum, but relying
        // on that is fragile). The leading-MAGIC check does not
        // authenticate pingwaves against a spoofing attacker; that is a
        // separate protocol concern.
        if data.len() == EnhancedPingwave::SIZE && u16::from_le_bytes([data[0], data[1]]) != MAGIC {
            if let Some(pw) = EnhancedPingwave::from_bytes(&data) {
                let origin_nid = graph_id_to_node_id(&pw.origin_id);

                // DV loop-avoidance rule 1: origin self-check. Drop
                // any pingwave claiming `origin_id == self_id`. This
                // defends against (a) a buggy peer echoing our own
                // origin back at us, and (b) a stale buffered
                // pingwave from a partitioned-then-healed peer.
                if origin_nid == ctx.local_node_id {
                    return;
                }

                // DV loop-avoidance rule 2: MAX_HOPS cap. TTL bounds
                // forwarding; MAX_HOPS bounds install. A pingwave
                // claiming an inflated hop_count can't populate a
                // usable route or graph edge.
                if pw.hop_count >= MAX_HOPS {
                    return;
                }

                // DV loop-avoidance rule 4: only accept pingwaves
                // from registered direct peers. An unknown source
                // addr means either (a) a stale packet from before
                // a handshake was torn down, (b) a peer that never
                // handshaked, or (c) an attacker injecting forged
                // pingwaves. In all three cases we refuse to install
                // route or graph state — otherwise an unauthenticated
                // sender could poison our routing table by claiming
                // to be a next-hop for arbitrary origins.
                let from_node_id = match ctx.addr_to_node.get(&source) {
                    Some(e) => *e.value(),
                    None => return,
                };

                // Install an indirect route `(origin, via=source)`
                // with metric `hop_count + 2`. The `+2` keeps direct
                // routes (metric 1) strictly better than any pingwave
                // route — `add_route_with_metric` preserves the
                // better entry.
                let metric = (pw.hop_count as u16).saturating_add(2);
                ctx.router
                    .routing_table()
                    .add_route_with_metric(origin_nid, source, metric);

                // Hand to the proximity graph to update nodes +
                // edges. `source` is guaranteed registered at this
                // point, so `from_graph_id` faithfully attributes
                // the edge to the forwarding peer's node_id.
                let from_graph_id = node_id_to_graph_id(from_node_id);
                if let Some(fwd_pw) =
                    ctx.proximity_graph
                        .on_pingwave_from(pw, from_graph_id, source)
                {
                    let fwd_bytes = fwd_pw.to_bytes();
                    let socket = ctx.socket.clone();
                    let peers = ctx.peers.clone();
                    let filter = ctx.partition_filter.clone();
                    let router = ctx.router.clone();
                    // DV loop-avoidance rule 3: split horizon on
                    // re-broadcast. If we installed `(origin_nid,
                    // next_hop=X)` — i.e. we'd use X to reach the
                    // origin — don't re-advertise the origin on the
                    // link to X. Prevents X from learning "we can
                    // reach origin in N+1 hops" and installing a
                    // backward loop.
                    tokio::spawn(async move {
                        let next_hop = router.routing_table().lookup(origin_nid);
                        for entry in peers.iter() {
                            let addr = entry.value().addr;
                            if addr == source {
                                continue; // never send back to sender
                            }
                            if Some(addr) == next_hop {
                                continue; // split horizon: that's our path to origin
                            }
                            if filter.contains(&addr) {
                                continue;
                            }
                            let _ = socket.send_to(&fwd_bytes, addr).await;
                        }
                    });
                }
                return;
            }
        }

        let local_node_id = ctx.local_node_id;
        let peers = &ctx.peers;
        let router = &ctx.router;
        let failure_detector = &ctx.failure_detector;
        // Distinguish routed packets from direct packets.
        //
        // Direct packets start with the Net header magic (0x4E45).
        // Routed packets start with a 16-byte routing header (dest_id,
        // src_id, ttl, hop_count, flags) followed by the Net header.
        // Since dest_id is a u64 node ID, its first two bytes will
        // almost never equal 0x4E45 by accident.
        let is_routed = data.len() >= ROUTING_HEADER_SIZE + protocol::HEADER_SIZE
            && u16::from_le_bytes([data[0], data[1]]) != MAGIC;

        if is_routed {
            // Routed packet: parse routing header, decide forward or local
            if let Some(routing_header) = RoutingHeader::from_bytes(&data[..ROUTING_HEADER_SIZE]) {
                if routing_header.dest_id == local_node_id {
                    // For us — strip routing header, process the inner Net packet.
                    // The inner packet is encrypted with the *sender's* session key
                    // (not the relay's), so we look up the session by session_id
                    // in the inner header, not by source address.
                    let inner = data.slice(ROUTING_HEADER_SIZE..);
                    let parsed = match ParsedPacket::parse(inner, source) {
                        Some(p) => p,
                        None => return,
                    };
                    // Heartbeats are link-local and don't make sense over
                    // the routing layer — drop.
                    if parsed.header.flags.is_heartbeat() {
                        return;
                    }
                    // Routed handshake arrival. Strip routing header and
                    // hand to the responder/msg2 dispatcher.
                    if parsed.header.flags.is_handshake() {
                        Self::handle_routed_handshake(&parsed, &routing_header, source, ctx);
                        return;
                    }
                    // Find the session that matches this packet's session_id
                    let session_id = parsed.header.session_id;
                    let matching_session = peers
                        .iter()
                        .find(|e| e.value().session.session_id() == session_id)
                        .map(|e| e.value().session.clone());
                    if let Some(session) = matching_session {
                        Self::process_local_packet(&parsed, &session, ctx);
                        session.touch();
                    }
                } else {
                    // Not for us — forward without decrypting (header-only
                    // routing). We send via the main socket so the
                    // receiving node sees `source` = our bound addr,
                    // which it can use as a reply path. `router.start()`'s
                    // internal scheduler has a separate ephemeral socket
                    // and would make `source` unusable for replies.
                    if routing_header.is_expired() {
                        return;
                    }
                    let next_hop = match router.routing_table().lookup(routing_header.dest_id) {
                        Some(addr) => addr,
                        None => return,
                    };
                    if ctx.partition_filter.contains(&next_hop) {
                        return;
                    }
                    let mut fwd_header = routing_header;
                    fwd_header.forward();
                    let mut new_data = bytes::BytesMut::with_capacity(data.len());
                    new_data.extend_from_slice(&fwd_header.to_bytes());
                    new_data.extend_from_slice(&data[ROUTING_HEADER_SIZE..]);
                    let forwarded = new_data.freeze();
                    let socket = ctx.socket.clone();
                    tokio::spawn(async move {
                        let _ = socket.send_to(&forwarded, next_hop).await;
                    });
                }
            }
            return;
        }

        // Direct packet (no routing header) — standard path.
        //
        // `session_id` is the authoritative logical-peer key. `addr_to_node`
        // gives a fast path when source addr maps to exactly one session,
        // but we must still validate session_id against the resolved peer
        // and fall back to a session_id scan if it doesn't match. Otherwise
        // two peers that share a wire address (e.g., a direct peer and a
        // relay-peer reachable via the same relay addr) would collide.
        let parsed = match ParsedPacket::parse(data, source) {
            Some(p) => p,
            None => return,
        };

        if parsed.header.flags.is_handshake() {
            return;
        }

        let session_id = parsed.header.session_id;
        let matched = ctx
            .addr_to_node
            .get(&source)
            .map(|e| *e.value())
            .and_then(|nid| peers.get(&nid))
            .filter(|p| p.session.session_id() == session_id)
            .map(|p| (p.value().node_id, p.value().session.clone()))
            .or_else(|| {
                peers
                    .iter()
                    .find(|e| e.value().session.session_id() == session_id)
                    .map(|e| (e.value().node_id, e.value().session.clone()))
            });
        let (peer_node_id, session) = match matched {
            Some(x) => x,
            None => return,
        };

        if parsed.header.flags.is_heartbeat() {
            failure_detector.heartbeat(peer_node_id, source);
            session.touch();
            return;
        }

        Self::process_local_packet(&parsed, &session, ctx);
        session.touch();
    }

    /// Handle a routed handshake packet that arrived at this node.
    ///
    /// Two cases, discriminated by whether we have a pending initiator
    /// state for `routing_header.src_id`:
    ///
    /// 1. **msg2 for an in-flight initiator.** We started a `connect_via`
    ///    earlier and registered a `PendingHandshake` keyed by the
    ///    responder's node_id. The arriving packet completes that
    ///    initiator state — we run `read_message`, derive keys, and
    ///    signal the caller via the oneshot.
    ///
    /// 2. **msg1 from a new initiator.** We build a responder state with
    ///    the prologue derived from `(routing_header.src_id, self.node_id)`,
    ///    read msg1, write msg2, and send msg2 back via the routing
    ///    table (reversing src/dest in the routing header). On success
    ///    we register the new peer with the routing-path addr (the
    ///    immediate upstream `source`) so that subsequent routed data
    ///    finds a session.
    fn handle_routed_handshake(
        parsed: &ParsedPacket,
        routing_header: &RoutingHeader,
        source: SocketAddr,
        ctx: &DispatchCtx,
    ) {
        // Routing id of the remote party: what we see in the routing
        // header's 32-bit src_id, zero-extended into u64 so it can sit
        // alongside full node_ids in maps without ambiguity.
        let peer_routing_id = routing_header.src_id as u64;

        // Case 1: msg2 for an in-flight initiator. Look up pending state
        // by routing id (that's how it was keyed on insert).
        if let Some((_, pending)) = ctx.pending_handshakes.remove(&peer_routing_id) {
            let PendingHandshake { mut noise, tx } = pending;
            let result = (|| -> Result<SessionKeys, CryptoError> {
                noise.read_message(&parsed.payload)?;
                noise.into_session_keys()
            })();
            let _ = tx.send(result);
            return;
        }

        // Case 2: msg1 from a new initiator.
        //
        // Prologue binds (peer_routing_id, self_routing_id) — same u32
        // projection the initiator used. Full u64 identities don't
        // fit in the routing header (src_id is u32), so we bind what
        // both sides CAN see, and carry the full src node_id inside
        // the Noise payload where it's AEAD-authenticated.
        let self_routing_id = routing_id(ctx.local_node_id);
        let prologue = handshake_prologue(peer_routing_id, self_routing_id);
        let mut noise =
            match NoiseHandshake::responder_with_prologue(&ctx.psk, &ctx.static_keypair, &prologue)
            {
                Ok(n) => n,
                Err(e) => {
                    tracing::warn!(error = %e, "routed handshake: responder build failed");
                    return;
                }
            };
        let msg1_payload = match noise.read_message(&parsed.payload) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(error = %e, "routed handshake: read_message failed (msg1 tampered or wrong PSK)");
                return;
            }
        };

        // Extract the initiator's full u64 node_id from the decrypted
        // payload. Verify its routing id matches the one we got on the
        // wire — a mismatch means the payload was crafted for a
        // different address than what arrived.
        if msg1_payload.len() < 8 {
            tracing::warn!(
                "routed handshake: msg1 payload too short ({}); need 8 bytes of src node_id",
                msg1_payload.len()
            );
            return;
        }
        let peer_node_id = u64::from_le_bytes(msg1_payload[..8].try_into().unwrap());
        if routing_id(peer_node_id) != peer_routing_id {
            tracing::warn!(
                payload = format!("{:#x}", peer_node_id),
                routing = format!("{:#x}", peer_routing_id),
                "routed handshake: src_node_id in payload does not match routing header"
            );
            return;
        }

        let msg2 = match noise.write_message(&[]) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "routed handshake: write_message failed");
                return;
            }
        };
        let keys = match noise.into_session_keys() {
            Ok(k) => k,
            Err(e) => {
                tracing::warn!(error = %e, "routed handshake: key extraction failed");
                return;
            }
        };

        // Build the msg2 packet: Net header (handshake flag) + Noise
        // bytes, wrapped in a routing header with dest = FULL peer
        // node_id (from payload). The initiator's local_node_id check
        // on arrival matches the full u64, so we must put the full
        // value here.
        let mut builder = PacketBuilder::new(&[0u8; 32], 0);
        let inner = builder.build_handshake(&msg2);
        let reply_routing = RoutingHeader::new(
            peer_node_id,
            ctx.local_node_id as u32,
            DEFAULT_HANDSHAKE_TTL,
        );
        let mut routed = bytes::BytesMut::with_capacity(ROUTING_HEADER_SIZE + inner.len());
        routed.extend_from_slice(&reply_routing.to_bytes());
        routed.extend_from_slice(&inner);

        // Pick the next hop for the reply. Prefer the routing table
        // (same path the routed handshake arrived on, symmetrically).
        // Fall back to `source` (the immediate upstream that sent us
        // msg1) — that's a direct peer by construction and guaranteed
        // to have a route back.
        let next_hop = ctx
            .router
            .routing_table()
            .lookup(peer_node_id)
            .unwrap_or(source);

        // Register the new peer. The wire `addr` we record is `source`
        // — the immediate upstream peer that forwarded msg1. That is
        // NOT necessarily the final responder's addr (for multi-hop it
        // isn't), but it's the correct place for future routed data
        // to flow through. Direct data uses this addr; routed data
        // uses the routing table.
        //
        // Registration happens BEFORE the send so that even if the
        // spawned send task is cancelled or panics post-send, the
        // initiator that just derived matching keys finds us.
        let session = Arc::new(NetSession::new(
            keys,
            source,
            ctx.packet_pool_size,
            ctx.default_reliable,
        ));
        ctx.peers.insert(
            peer_node_id,
            PeerInfo {
                node_id: peer_node_id,
                addr: source,
                session,
            },
        );
        ctx.peer_addrs.insert(peer_node_id, source);
        ctx.router.add_route(peer_node_id, source);

        // Spawn the send. If it fails, roll back all three registrations
        // (peer session, peer-addr map, and routing table entry). Leaving
        // the route behind would silently blackhole future routed traffic
        // for `peer_node_id` through an addr we never confirmed was
        // reachable; removing peers without removing the route would also
        // inject a stale entry into rerouting decisions.
        //
        // Route rollback is conditional on the current entry still
        // pointing at `source` — if a concurrent handshake for the same
        // `peer_node_id` already installed a newer (valid) route, we must
        // not overwrite it.
        let socket = ctx.socket.clone();
        let peers = ctx.peers.clone();
        let peer_addrs = ctx.peer_addrs.clone();
        let router = ctx.router.clone();
        let registered_next_hop = source;
        let payload = routed.freeze();
        tokio::spawn(async move {
            if let Err(e) = socket.send_to(&payload, next_hop).await {
                tracing::warn!(
                    peer = format!("{:#x}", peer_node_id),
                    error = %e,
                    "routed handshake: msg2 send failed; unregistering peer"
                );
                // Only remove entries that still match what we wrote;
                // a concurrent retry for the same peer may have already
                // replaced them with a fresh, valid registration.
                peers.remove_if(&peer_node_id, |_, pi| pi.addr == registered_next_hop);
                peer_addrs.remove_if(&peer_node_id, |_, addr| *addr == registered_next_hop);
                router
                    .routing_table()
                    .remove_route_if_next_hop_is(peer_node_id, registered_next_hop);
            }
        });
    }

    /// Process a locally-destined packet: decrypt and queue events.
    ///
    /// This is the same logic as `NetAdapter::process_packet` but extracted
    /// to work with the multi-session dispatch.
    fn process_local_packet(parsed: &ParsedPacket, session: &NetSession, ctx: &DispatchCtx) {
        let inbound = &ctx.inbound;
        let num_shards = ctx.num_shards;
        // Validate payload length
        if !parsed.header.flags.is_handshake()
            && !parsed.header.flags.is_heartbeat()
            && !parsed.is_valid_length()
        {
            return;
        }

        // Decrypt payload
        let aad = parsed.header.aad();
        let counter = u64::from_le_bytes(parsed.header.nonce[4..12].try_into().unwrap_or([0u8; 8]));
        let rx_cipher = session.rx_cipher();
        if !rx_cipher.is_valid_rx_counter(counter) {
            return;
        }
        let decrypted = match rx_cipher.decrypt(counter, &aad, &parsed.payload) {
            Ok(d) => {
                // Commit-time replay check: closes the TOCTOU race where
                // two threads decrypt the same replayed packet concurrently.
                if !rx_cipher.update_rx_counter(counter) {
                    return;
                }
                d
            }
            Err(_) => return,
        };

        // Check subprotocol — migration messages are sent as single event frames
        if parsed.header.subprotocol_id == SUBPROTOCOL_MIGRATION {
            if let Some(ref handler) = ctx.migration_handler {
                // Extract the payload from the event frame wrapper
                let events =
                    EventFrame::read_events(Bytes::from(decrypted), parsed.header.event_count);
                let payload = match events.into_iter().next() {
                    Some(data) => data,
                    None => return,
                };

                // Find the sender's node_id
                let from_node = ctx
                    .peers
                    .iter()
                    .find(|e| e.value().session.session_id() == session.session_id())
                    .map(|e| e.value().node_id)
                    .unwrap_or(0);

                match handler.handle_message(&payload, from_node) {
                    Ok(outbound) => {
                        // Send outbound responses asynchronously
                        for msg in outbound {
                            let dest_session = ctx
                                .peers
                                .get(&msg.dest_node)
                                .map(|e| (e.value().addr, e.value().session.clone()));

                            if let Some((dest_addr, dest_sess)) = dest_session {
                                // Respect partition filter on outbound path
                                if ctx.partition_filter.contains(&dest_addr) {
                                    continue;
                                }
                                let socket = ctx.socket.clone();
                                let payload = Bytes::from(msg.payload);
                                tokio::spawn(async move {
                                    let pool = dest_sess.thread_local_pool();
                                    let mut builder = pool.get();
                                    let seq = {
                                        let stream = dest_sess
                                            .get_or_create_stream(SUBPROTOCOL_MIGRATION as u64);
                                        stream.next_tx_seq()
                                    };
                                    let events = vec![payload];
                                    let packet = builder.build_subprotocol(
                                        SUBPROTOCOL_MIGRATION as u64,
                                        seq,
                                        &events,
                                        PacketFlags::NONE,
                                        SUBPROTOCOL_MIGRATION,
                                    );
                                    let _ = socket.send_to(&packet, dest_addr).await;
                                });
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "migration handler error");
                    }
                }
                return; // handler processed it
            }
            // No handler set — fall through to standard event path
        }

        // Stream-window credit grant: apply to the named stream's
        // `tx_credit_remaining` without touching the inbound event
        // queue. The grant payload is an event frame carrying a
        // 12-byte `StreamWindow` message.
        if parsed.header.subprotocol_id == SUBPROTOCOL_STREAM_WINDOW {
            let events = EventFrame::read_events(Bytes::from(decrypted), parsed.header.event_count);
            if let Some(payload) = events.into_iter().next() {
                match StreamWindow::decode(&payload) {
                    Ok(grant) => {
                        // Quarantine guard: a grant that arrives for a
                        // stream closed within `GRANT_QUARANTINE_WINDOW`
                        // is dropped, even if the stream has already been
                        // reopened with the same id. Without this the
                        // in-flight grant from the *previous* lifetime
                        // would spuriously credit the new `StreamState`
                        // and let the sender exceed its intended window.
                        // Grants for closed / unknown streams are also
                        // dropped silently — the sender will time out on
                        // its own.
                        if session.is_grant_quarantined(grant.stream_id) {
                            tracing::debug!(
                                stream_id = format!("{:#x}", grant.stream_id),
                                "dropping StreamWindow grant for recently-closed stream"
                            );
                        } else if let Some(state) = session.try_stream(grant.stream_id) {
                            state.apply_authoritative_grant(grant.total_consumed);
                        }
                    }
                    Err(e) => {
                        tracing::debug!(error = %e, "malformed StreamWindow grant");
                    }
                }
            }
            return;
        }

        // Channel membership: Subscribe / Unsubscribe / Ack.
        if parsed.header.subprotocol_id == SUBPROTOCOL_CHANNEL_MEMBERSHIP {
            let events = EventFrame::read_events(Bytes::from(decrypted), parsed.header.event_count);
            let payload = match events.into_iter().next() {
                Some(data) => data,
                None => return,
            };

            // Resolve the sender's node_id from the session they arrived on.
            let from_node = ctx
                .peers
                .iter()
                .find(|e| e.value().session.session_id() == session.session_id())
                .map(|e| e.value().node_id)
                .unwrap_or(0);

            Self::handle_membership_message(&payload, from_node, ctx);
            return;
        }

        // Capability announcement: signed, versioned capability metadata.
        // Feeds the local `CapabilityIndex`; never responded to.
        if parsed.header.subprotocol_id == SUBPROTOCOL_CAPABILITY_ANN {
            let events = EventFrame::read_events(Bytes::from(decrypted), parsed.header.event_count);
            let Some(payload) = events.into_iter().next() else {
                return;
            };

            let from_node = ctx
                .peers
                .iter()
                .find(|e| e.value().session.session_id() == session.session_id())
                .map(|e| e.value().node_id)
                .unwrap_or(0);

            Self::handle_capability_announcement(&payload, from_node, ctx);
            return;
        }

        // Standard event path: parse event frames and queue.
        //
        // Credit accounting charges the full on-wire size (Net
        // header + AEAD tag + payload) so sender and receiver stay
        // symmetric — the sender debits the same quantity via
        // `wire_bytes_for_payload` on admission.
        let payload_bytes = (decrypted.len() + PACKET_WIRE_OVERHEAD) as u64;
        let events = EventFrame::read_events(Bytes::from(decrypted), parsed.header.event_count);

        let stream_id = parsed.header.stream_id;
        let shard_id = if num_shards > 0 {
            (stream_id % num_shards as u64) as u16
        } else {
            0
        };

        // Credit-window bookkeeping: charge only *accepted* inbound
        // bytes against the stream's RxCreditState. `on_receive`
        // returns `false` for duplicates (already-acked sequences)
        // and for sequences past the Reliable receive window —
        // crediting those would refund send credit for
        // retransmissions / replays, letting a chatty peer inflate
        // `tx_credit_remaining` past what it actually pushed through
        // the protocol. Accounting runs at receive time (not drain
        // time); this closes the v1 gap where a single serial sender
        // ran `Transport(io::Error)` into a full kernel buffer. A
        // separately slow daemon is still backstopped by the
        // existing shard-queue-depth limits.
        let grant_bytes = {
            let stream = session.get_or_create_stream(stream_id);
            let accepted = stream.with_reliability(|r| r.on_receive(parsed.header.sequence));
            if accepted {
                stream.update_rx_seq(parsed.header.sequence);
                stream.on_bytes_consumed(payload_bytes)
            } else {
                None
            }
        };

        if let Some(total_consumed) = grant_bytes {
            // Resolve the sending peer via two O(1) DashMap lookups
            // (`addr_to_node` → `peers`) instead of a linear scan over
            // `ctx.peers`. At high peer counts the scan would make
            // packet receive cost proportional to peer count — the
            // hot path needs to stay constant-time.
            //
            // The addr-based lookup is validated against the arriving
            // `session.session_id()` because multiple peers can share
            // a source address in relay scenarios: `addr_to_node` is
            // keyed by last-seen source and may resolve to a different
            // peer than the one this packet authenticated as. On
            // mismatch (or miss) fall back to a session_id scan so the
            // grant is guaranteed to go back on the session the
            // accepted bytes arrived on.
            let peer_addr = session.peer_addr();
            let peer = ctx
                .addr_to_node
                .get(&peer_addr)
                .and_then(|node_id| {
                    ctx.peers.get(&*node_id).and_then(|p| {
                        (p.value().session.session_id() == session.session_id())
                            .then(|| (p.value().addr, p.value().session.clone()))
                    })
                })
                .or_else(|| {
                    ctx.peers
                        .iter()
                        .find(|e| e.value().session.session_id() == session.session_id())
                        .map(|e| (e.value().addr, e.value().session.clone()))
                });
            if let Some((peer_addr, peer_session)) = peer {
                Self::spawn_stream_window_grant(
                    ctx,
                    peer_session,
                    peer_addr,
                    stream_id,
                    total_consumed,
                );
            }
        }

        let queue = inbound.entry(shard_id).or_default();
        let seq = parsed.header.sequence;
        for (i, event_data) in events.into_iter().enumerate() {
            use std::fmt::Write;
            let mut event_id = String::with_capacity(24);
            let _ = write!(event_id, "{}:{}", seq, i);
            queue.push(StoredEvent::new(event_id, event_data, seq, shard_id));
        }
    }

    /// Emit a `StreamWindow` credit grant back to `peer_addr` on the
    /// existing encrypted session. Fire-and-forget — grants are
    /// **authoritative**, so a lost grant is reconciled by the next
    /// one that successfully arrives (each carries the receiver's
    /// full `total_consumed` picture).
    ///
    /// The grant packet rides on the sentinel [`CONTROL_STREAM_ID`]
    /// (`u64::MAX`) with a sequence drawn from
    /// `NetSession::next_control_tx_seq`. This is a dedicated
    /// session-level counter that cannot collide with user stream
    /// state — a caller who opens a stream numerically equal to
    /// `SUBPROTOCOL_STREAM_WINDOW` (0x0B00) won't see their
    /// sequence space polluted by control traffic.
    fn spawn_stream_window_grant(
        ctx: &DispatchCtx,
        session: Arc<NetSession>,
        peer_addr: SocketAddr,
        stream_id: u64,
        total_consumed: u64,
    ) {
        if ctx.partition_filter.contains(&peer_addr) {
            return;
        }
        let socket = ctx.socket.clone();
        tokio::spawn(async move {
            let payload = StreamWindow {
                stream_id,
                total_consumed,
            }
            .encode();
            let pool = session.thread_local_pool();
            let mut builder = pool.get();
            let seq = session.next_control_tx_seq();
            let events = vec![Bytes::copy_from_slice(&payload)];
            let packet = builder.build_subprotocol(
                CONTROL_STREAM_ID,
                seq,
                &events,
                PacketFlags::NONE,
                SUBPROTOCOL_STREAM_WINDOW,
            );
            if let Err(e) = socket.send_to(&packet, peer_addr).await {
                tracing::debug!(error = %e, "StreamWindow grant send failed");
                return;
            }
            // Grant reached the wire — count it on the emitting
            // stream so the receiver side of stats reflects
            // cumulative grants sent.
            if let Some(state) = session.try_stream(stream_id) {
                state.note_grant_sent();
            }
        });
    }

    /// Spawn heartbeat sender for all peers.
    fn spawn_heartbeat_loop(&self) -> JoinHandle<()> {
        let socket = self.socket.clone();
        let peers = self.peers.clone();
        let interval = self.config.heartbeat_interval;
        let shutdown = self.shutdown.clone();
        let shutdown_notify = self.shutdown_notify.clone();
        let partition_filter = self.partition_filter.clone();
        let proximity_graph = self.proximity_graph.clone();
        let router = self.router.clone();
        // Sweep routes that haven't been refreshed for 3× the session
        // timeout. Direct routes are refreshed by this loop's own
        // pingwave emission; indirect (pingwave-learned) routes age out
        // here if their origin goes silent.
        let max_route_age = self.config.session_timeout.saturating_mul(3);
        // Stream lifecycle: drop idle streams past `stream_idle_timeout`
        // and enforce `max_streams` cap via LRU.
        let stream_idle_timeout = self.config.stream_idle_timeout;
        let max_streams = self.config.max_streams;

        tokio::spawn(async move {
            while !shutdown.load(Ordering::Acquire) {
                tokio::select! {
                    _ = tokio::time::sleep(interval) => {
                        // Create a pingwave for this heartbeat cycle
                        let pw = proximity_graph.create_pingwave(HealthStatus::Healthy);
                        let pw_bytes = pw.to_bytes();

                        for entry in peers.iter() {
                            let peer_addr = entry.value().addr;
                            if partition_filter.contains(&peer_addr) {
                                continue;
                            }
                            let session = &entry.value().session;
                            let mut builder =
                                PacketBuilder::new(&[0u8; 32], session.session_id());
                            // Heartbeat
                            let packet = builder.build_heartbeat();
                            let _ = socket.send_to(&packet, peer_addr).await;
                            // Pingwave (raw UDP — not encrypted, topology is public)
                            let _ = socket.send_to(&pw_bytes, peer_addr).await;
                        }

                        // Drop routes whose `updated_at` is past the age
                        // limit. Small scan of the routing table; cheap.
                        router.routing_table().sweep_stale(max_route_age);

                        // Age out proximity graph edges in lockstep
                        // with the routing table. If the peer that
                        // used to relay pingwaves for an origin went
                        // silent, both the (peer→origin) edge and the
                        // routing-table entry that depended on it
                        // disappear on the same tick.
                        proximity_graph.sweep_stale_edges(max_route_age);

                        // Sweep idle streams per-session and enforce the
                        // per-session `max_streams` cap. Each session is
                        // independent; large deployments with many peers
                        // each with many streams pay O(P + total_streams).
                        for entry in peers.iter() {
                            entry.value().session.evict_idle_streams(
                                stream_idle_timeout,
                                max_streams,
                                "idle_timeout",
                            );
                        }
                    }
                    _ = shutdown_notify.notified() => {
                        break;
                    }
                }
            }
        })
    }

    /// Send a batch of events to a specific peer by address.
    pub async fn send_to_peer(
        &self,
        peer_addr: SocketAddr,
        batch: Batch,
    ) -> Result<(), AdapterError> {
        // Partition filter: silently drop sends to blocked peers
        if self.partition_filter.contains(&peer_addr) {
            return Ok(());
        }

        let node_id = self
            .addr_to_node
            .get(&peer_addr)
            .map(|e| *e.value())
            .ok_or_else(|| AdapterError::Connection("unknown peer".into()))?;
        let peer = self
            .peers
            .get(&node_id)
            .ok_or_else(|| AdapterError::Connection("unknown peer".into()))?;

        let session = &peer.session;
        let stream_id = batch.shard_id as u64;

        let reliable = {
            let stream = session.get_or_create_stream(stream_id);
            stream.with_reliability(|r| r.needs_ack())
        };

        let pool = session.thread_local_pool();
        let mut builder = pool.get();

        let mut current_batch: Vec<Bytes> = Vec::with_capacity(64);
        let mut current_size = 0usize;

        for event in &batch.events {
            let event_bytes = event.raw.clone();
            let frame_size = EventFrame::LEN_SIZE + event_bytes.len();

            if current_size + frame_size > protocol::MAX_PAYLOAD_SIZE && !current_batch.is_empty() {
                let seq = {
                    let stream = session.get_or_create_stream(stream_id);
                    stream.next_tx_seq()
                };
                let flags = if reliable {
                    PacketFlags::RELIABLE
                } else {
                    PacketFlags::NONE
                };
                let packet = builder.build(stream_id, seq, &current_batch, flags);
                self.socket
                    .send_to(&packet, peer_addr)
                    .await
                    .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;

                current_batch.clear();
                current_size = 0;
            }

            current_batch.push(event_bytes);
            current_size += frame_size;
        }

        if !current_batch.is_empty() {
            let seq = {
                let stream = session.get_or_create_stream(stream_id);
                stream.next_tx_seq()
            };
            let flags = if reliable {
                PacketFlags::RELIABLE
            } else {
                PacketFlags::NONE
            };
            let packet = builder.build(stream_id, seq, &current_batch, flags);
            self.socket
                .send_to(&packet, peer_addr)
                .await
                .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;
        }

        // builder is dropped here — auto-released back to the pool
        drop(builder);
        session.touch();
        Ok(())
    }

    /// Send a batch of events to a destination node via the routing table.
    ///
    /// The events are encrypted with the destination's session key and
    /// a routing header is prepended so intermediate nodes can forward
    /// without decrypting. The packet is sent to the next hop from the
    /// routing table, not directly to the destination.
    ///
    /// Requires:
    /// - A session with `dest_node_id` (for encryption)
    /// - A route to `dest_node_id` in the routing table (for next hop)
    pub async fn send_routed(&self, dest_node_id: u64, batch: Batch) -> Result<(), AdapterError> {
        // Find the session for the destination (needed for encryption)
        let (dest_addr, session) = self
            .peers
            .get(&dest_node_id)
            .map(|e| (e.value().addr, e.value().session.clone()))
            .ok_or_else(|| {
                AdapterError::Connection(format!("no session for node {:#x}", dest_node_id))
            })?;

        // Find the next hop from the routing table
        let next_hop = self
            .router
            .routing_table()
            .lookup(dest_node_id)
            .unwrap_or(dest_addr); // fall back to direct if no route

        let stream_id = batch.shard_id as u64;
        let reliable = {
            let stream = session.get_or_create_stream(stream_id);
            stream.with_reliability(|r| r.needs_ack())
        };

        let pool = session.thread_local_pool();
        let mut builder = pool.get();

        // Build routing header
        let routing_header = RoutingHeader::new(dest_node_id, self.node_id as u32, 8);
        let routing_bytes = routing_header.to_bytes();

        let mut current_batch: Vec<Bytes> = Vec::with_capacity(64);
        let mut current_size = 0usize;

        for event in &batch.events {
            let event_bytes = event.raw.clone();
            let frame_size = EventFrame::LEN_SIZE + event_bytes.len();

            if current_size + frame_size > protocol::MAX_PAYLOAD_SIZE && !current_batch.is_empty() {
                let seq = {
                    let stream = session.get_or_create_stream(stream_id);
                    stream.next_tx_seq()
                };
                let flags = if reliable {
                    PacketFlags::RELIABLE
                } else {
                    PacketFlags::NONE
                };
                // Build encrypted packet, then prepend routing header
                let net_packet = builder.build(stream_id, seq, &current_batch, flags);
                let mut routed =
                    bytes::BytesMut::with_capacity(ROUTING_HEADER_SIZE + net_packet.len());
                routed.extend_from_slice(&routing_bytes);
                routed.extend_from_slice(&net_packet);

                self.socket
                    .send_to(&routed, next_hop)
                    .await
                    .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;

                current_batch.clear();
                current_size = 0;
            }

            current_batch.push(event_bytes);
            current_size += frame_size;
        }

        if !current_batch.is_empty() {
            let seq = {
                let stream = session.get_or_create_stream(stream_id);
                stream.next_tx_seq()
            };
            let flags = if reliable {
                PacketFlags::RELIABLE
            } else {
                PacketFlags::NONE
            };
            let net_packet = builder.build(stream_id, seq, &current_batch, flags);
            let mut routed = bytes::BytesMut::with_capacity(ROUTING_HEADER_SIZE + net_packet.len());
            routed.extend_from_slice(&routing_bytes);
            routed.extend_from_slice(&net_packet);

            self.socket
                .send_to(&routed, next_hop)
                .await
                .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;
        }

        drop(builder);
        session.touch();
        Ok(())
    }

    // ── Channel membership API ─────────────────────────────────────────

    /// Access the per-channel subscriber roster. Used by `ChannelPublisher`
    /// to enumerate subscribers; exposed for diagnostics.
    pub fn roster(&self) -> &Arc<SubscriberRoster> {
        &self.roster
    }

    /// Install a `ChannelConfigRegistry` whose `can_subscribe` /
    /// `can_publish` rules are consulted for incoming Subscribe
    /// messages.
    ///
    /// When unset (the default), all subscribes are accepted. Full
    /// capability/token-based authorization additionally requires a
    /// `TokenCache` — see [`Self::set_token_cache`].
    pub fn set_channel_configs(&mut self, configs: Arc<ChannelConfigRegistry>) {
        self.channel_configs = Some(configs);
    }

    /// Install a shared `TokenCache` used by the channel-auth path.
    /// When set, `authorize_subscribe` and `publish_many` consult
    /// it via `ChannelConfig::can_subscribe` / `can_publish`.
    /// Subscribers that present a token on the wire have their
    /// token installed into this cache (after signature
    /// verification) before the ACL check runs.
    ///
    /// When unset, `require_token` channels always reject —
    /// without a cache there's no way to validate presented tokens
    /// or find pre-cached ones.
    pub fn set_token_cache(&mut self, cache: Arc<TokenCache>) {
        self.token_cache = Some(cache);
    }

    /// Ask `publisher_node_id` to add this node to `channel`'s subscriber set.
    ///
    /// Blocks until the publisher's `Ack` arrives or
    /// `membership_ack_timeout` elapses. Returns `Ok(())` iff the publisher
    /// accepted the subscribe; `AckReason` failures surface as
    /// `AdapterError::Connection`. No token is presented — use
    /// [`Self::subscribe_channel_with_token`] for channels with
    /// `require_token` set.
    pub async fn subscribe_channel(
        &self,
        publisher_node_id: u64,
        channel: ChannelName,
    ) -> Result<(), AdapterError> {
        self.send_membership_request(publisher_node_id, channel, true, None)
            .await
    }

    /// Subscribe with a pre-issued [`PermissionToken`] attached.
    /// The publisher verifies the token and, on success, installs
    /// it in its local `TokenCache` before the
    /// `ChannelConfig::can_subscribe` check.
    pub async fn subscribe_channel_with_token(
        &self,
        publisher_node_id: u64,
        channel: ChannelName,
        token: PermissionToken,
    ) -> Result<(), AdapterError> {
        self.send_membership_request(publisher_node_id, channel, true, Some(token.to_bytes()))
            .await
    }

    /// Ask `publisher_node_id` to remove this node from `channel`'s
    /// subscriber set. Mirror of `subscribe_channel`.
    pub async fn unsubscribe_channel(
        &self,
        publisher_node_id: u64,
        channel: ChannelName,
    ) -> Result<(), AdapterError> {
        self.send_membership_request(publisher_node_id, channel, false, None)
            .await
    }

    async fn send_membership_request(
        &self,
        publisher_node_id: u64,
        channel: ChannelName,
        subscribe: bool,
        token: Option<Vec<u8>>,
    ) -> Result<(), AdapterError> {
        let peer_addr = {
            let peer = self.peers.get(&publisher_node_id).ok_or_else(|| {
                AdapterError::Connection(format!(
                    "no session to publisher {:#x}",
                    publisher_node_id
                ))
            })?;
            peer.addr
        };

        let nonce = {
            use std::sync::atomic::AtomicU64;
            static COUNTER: AtomicU64 = AtomicU64::new(1);
            COUNTER.fetch_add(1, Ordering::Relaxed)
        };
        let msg = if subscribe {
            MembershipMsg::Subscribe {
                channel: channel.clone(),
                nonce,
                token,
            }
        } else {
            MembershipMsg::Unsubscribe {
                channel: channel.clone(),
                nonce,
            }
        };
        let bytes = membership::encode(&msg);

        let (tx, rx) = oneshot::channel::<MembershipAck>();
        self.pending_membership_acks.insert(nonce, tx);

        // Scoped send; if it fails, drop the pending entry so memory
        // doesn't accumulate.
        if let Err(e) = self
            .send_subprotocol(peer_addr, SUBPROTOCOL_CHANNEL_MEMBERSHIP, &bytes)
            .await
        {
            self.pending_membership_acks.remove(&nonce);
            return Err(e);
        }

        let ack = match tokio::time::timeout(self.config.membership_ack_timeout, rx).await {
            Ok(Ok(ack)) => ack,
            Ok(Err(_)) => {
                self.pending_membership_acks.remove(&nonce);
                return Err(AdapterError::Connection(
                    "membership ack channel closed".into(),
                ));
            }
            Err(_) => {
                self.pending_membership_acks.remove(&nonce);
                return Err(AdapterError::Connection(format!(
                    "membership ack timeout ({:?}) for channel {}",
                    self.config.membership_ack_timeout, channel
                )));
            }
        };

        if !ack.accepted {
            return Err(AdapterError::Connection(format!(
                "membership request rejected: {:?}",
                ack.reason
            )));
        }
        Ok(())
    }

    /// Dispatch an inbound Subscribe / Unsubscribe / Ack on the
    /// membership subprotocol.
    fn handle_membership_message(payload: &[u8], from_node: u64, ctx: &DispatchCtx) {
        let msg = match membership::decode(payload) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "membership decode failed");
                return;
            }
        };

        match msg {
            MembershipMsg::Subscribe {
                channel,
                nonce,
                token,
            } => {
                let (accepted, reason) =
                    Self::authorize_subscribe(&channel, from_node, token.as_deref(), ctx);
                if accepted {
                    // Populate the AuthGuard fast path so publish
                    // fan-out can admit this subscriber in <10 ns
                    // without re-walking the ACL. Mirrors the
                    // `roster.add` below — both are keyed on the
                    // channel name so they stay consistent.
                    ctx.auth_guard
                        .allow_channel(subscriber_origin_hash(from_node), &channel);
                    let id = ChannelId::new(channel);
                    ctx.roster.add(id, from_node);
                    Self::clear_auth_failures(from_node, ctx);
                } else if !matches!(
                    reason,
                    Some(AckReason::TooManyChannels) | Some(AckReason::RateLimited)
                ) {
                    // Count auth-rule rejections toward the
                    // failure budget. Resource limits
                    // (TooManyChannels) and throttle short-
                    // circuits (RateLimited) don't — the former
                    // is orthogonal, the latter is the *result*
                    // of past failures and would double-count.
                    Self::record_auth_failure(from_node, ctx);
                }
                Self::send_membership_ack(from_node, nonce, accepted, reason, ctx);
            }
            MembershipMsg::Unsubscribe { channel, nonce } => {
                // Revoke from the fast path first so any in-flight
                // publish stops admitting this subscriber even
                // before the roster update is visible.
                ctx.auth_guard
                    .revoke_channel(subscriber_origin_hash(from_node), &channel);
                let id = ChannelId::new(channel);
                ctx.roster.remove(&id, from_node);
                // Unsubscribe is always accepted — idempotent even if the
                // peer wasn't actually subscribed.
                Self::send_membership_ack(from_node, nonce, true, None, ctx);
            }
            MembershipMsg::Ack {
                nonce,
                accepted,
                reason,
            } => {
                if let Some((_, tx)) = ctx.pending_membership_acks.remove(&nonce) {
                    let _ = tx.send(MembershipAck { accepted, reason });
                } else {
                    tracing::debug!(
                        nonce,
                        "membership ack with no pending request (duplicate or timed out)"
                    );
                }
            }
        }
    }

    /// Dispatch an inbound `CapabilityAnnouncement` into the local
    /// capability index. Drops announcements that:
    /// - fail to decode (malformed bytes),
    /// - carry a `node_id` that doesn't match the session's peer
    ///   (a peer can only announce for itself),
    /// - are missing a signature when
    ///   `require_signed_capabilities` is on,
    /// - carry a signature that fails verification against the
    ///   announcement's own `entity_id` (Stage E upgrade).
    ///
    /// `node_id` and `entity_id` are independent values on the
    /// wire; we pin `node_id → entity_id` on first sight so a
    /// later announcement claiming a different `entity_id` for the
    /// same `node_id` won't silently rebind identity.
    fn handle_capability_announcement(payload: &[u8], from_node: u64, ctx: &DispatchCtx) {
        let Some(ann) = CapabilityAnnouncement::from_bytes(payload) else {
            tracing::trace!(
                from_node = format!("{:#x}", from_node),
                len = payload.len(),
                "capability: decode failed"
            );
            return;
        };

        // Direct peers may only announce their own caps. Forwarded
        // announcements (hop_count > 0) are relayed through a peer
        // that isn't the origin, so we skip the check in that path
        // and rely on signature verification plus the TOFU binding
        // to keep forgers out.
        if ann.hop_count == 0 && ann.node_id != from_node {
            tracing::trace!(
                from_node = format!("{:#x}", from_node),
                ann_node = format!("{:#x}", ann.node_id),
                "capability: node_id mismatch (peer can only announce for itself)"
            );
            return;
        }

        // Origin self-check — if we're the origin, drop. A mesh
        // loop could bounce our own announcement back to us; no
        // reason to re-index or re-broadcast.
        if ann.node_id == ctx.local_node_id {
            return;
        }

        // Dedup on (origin, version). A `(node_id, version)` tuple
        // is processed at most once — protects against diamond
        // topologies where the same announcement arrives twice via
        // different paths. Insert happens AFTER validation so a
        // malformed announcement doesn't poison the cache.
        let dedup_key = (ann.node_id, ann.version);
        if ctx.seen_announcements.contains_key(&dedup_key) {
            return;
        }

        if ctx.require_signed_capabilities && ann.signature.is_none() {
            tracing::trace!(
                from_node = format!("{:#x}", from_node),
                "capability: unsigned announcement rejected"
            );
            return;
        }

        // Verify the signature (when present) against the
        // announcement's self-claimed entity_id. Unsigned
        // announcements skip this branch; receivers that care about
        // authenticity set `require_signed_capabilities = true`.
        let signature_verified = if ann.signature.is_some() {
            if ann.verify().is_err() {
                tracing::trace!(
                    from_node = format!("{:#x}", from_node),
                    "capability: signature verification failed"
                );
                return;
            }
            true
        } else {
            false
        };

        // Bind `node_id` to `entity_id` cryptographically. The
        // signature covers `entity_id` but NOT `node_id` — without
        // this check a signed announcement could claim any
        // `node_id`, poisoning the capability index and route
        // learning for an unrelated peer. `EntityId::node_id()` is
        // a blake2s derivation over the public key, so a forger who
        // doesn't control the key can't produce matching bytes.
        if ann.entity_id.node_id() != ann.node_id {
            tracing::trace!(
                from_node = format!("{:#x}", from_node),
                claimed_node = format!("{:#x}", ann.node_id),
                derived_node = format!("{:#x}", ann.entity_id.node_id()),
                "capability: node_id does not match entity_id derivation"
            );
            return;
        }

        // First-seen identity pin — TOFU. A peer that tries to
        // rebind its `entity_id` in a later announcement is
        // silently rejected. Two preconditions must hold before we
        // pin anything:
        //
        // 1. The announcement is **signature-verified**. An
        //    unsigned announcement's `entity_id` is attacker-
        //    controlled and would poison the binding; unauthenticated
        //    deployments skip the pin entirely, and channel-auth
        //    paths fall through to "missing entity" instead of
        //    trusting forged input.
        // 2. The announcement arrived **directly** from the origin
        //    (`hop_count == 0`). On that path `ann.node_id` was
        //    already checked to equal `from_node` above, so pinning
        //    `from_node → ann.entity_id` binds the session to the
        //    key that actually signed for it. A forwarded
        //    announcement (`hop_count > 0`) travels through an
        //    arbitrary middle peer; pinning `from_node → victim_id`
        //    in that path would let the forwarder pose as the
        //    origin for subsequent channel auth (`authorize_subscribe`
        //    keys on `peer_entity_ids.get(from_node)`). Forwarded
        //    caps still update the capability index + routing, but
        //    the entity binding is deferred to the eventual direct
        //    announcement.
        if signature_verified && ann.hop_count == 0 {
            if let Some(existing) = ctx.peer_entity_ids.get(&from_node) {
                if *existing.value() != ann.entity_id {
                    tracing::trace!(
                        from_node = format!("{:#x}", from_node),
                        "capability: entity_id rebind rejected (TOFU)"
                    );
                    return;
                }
            } else {
                ctx.peer_entity_ids.insert(from_node, ann.entity_id.clone());
            }
        }

        // Derive the peer's subnet *before* moving `ann` into the
        // index — the policy needs `ann.capabilities` and `index()`
        // consumes the announcement by value.
        //
        // Gated on `signature_verified && ann.hop_count == 0` for
        // the same reasons the TOFU pin above has that exact pair
        // of conditions:
        //
        // 1. Unsigned `ann.capabilities` is attacker-controlled, so
        //    a deployment running with
        //    `require_signed_capabilities = false` for discovery
        //    must not let unsigned input feed `peer_subnets` —
        //    that map is read by `subnet_visible` on the
        //    publish / subscribe paths, and a spoofed subnet would
        //    admit a peer to `SubnetLocal` channels it shouldn't
        //    see.
        // 2. On a forwarded announcement (`hop_count > 0`) the
        //    `from_node` in our hands is the relay peer, not the
        //    origin. Writing the origin's derived subnet under the
        //    relay's `node_id` would overwrite the relay's legitimate
        //    subnet binding — a crafted forwarded announcement could
        //    shift any legitimate peer into a different subnet just
        //    by being the last hop on its path. The real binding
        //    comes from the origin's own direct announcement.
        if signature_verified && ann.hop_count == 0 {
            if let Some(policy) = ctx.local_subnet_policy.as_ref() {
                let subnet = policy.assign(&ann.capabilities);
                ctx.peer_subnets.insert(from_node, subnet);
            }
        }

        // Cache BEFORE indexing (the index consumes `ann` by value,
        // but the dedup key is already captured above). Insert at
        // this point so a subsequent duplicate short-circuits at the
        // `contains_key` check without re-parsing + re-verifying.
        ctx.seen_announcements
            .insert(dedup_key, std::time::Instant::now());

        // Topology learning from multi-hop receipt. An announcement
        // arriving with `hop_count > 0` traveled through `from_node`
        // to reach us, so install a route to the origin with metric
        // `hop_count + 2`. The `+2` offset matches the pingwave
        // convention so direct routes (metric 1) always strictly
        // beat any announcement-installed route. Routes from
        // capability announcements compete with pingwave-installed
        // routes via the routing table's "better metric wins" rule.
        // Direct announcements (hop_count == 0) skip this — the
        // session itself is already the authority for that peer.
        if ann.hop_count > 0 {
            if let Some(entry) = ctx.peer_addrs.get(&from_node) {
                let sender_addr = *entry.value();
                let metric = u16::from(ann.hop_count) + 2;
                ctx.router
                    .routing_table()
                    .add_route_with_metric(ann.node_id, sender_addr, metric);
            }
        }

        // Multi-hop forwarding: if we haven't exhausted the hop
        // budget, increment `hop_count` and re-broadcast to every
        // directly-connected peer except the sender and the peer we
        // use to reach the origin (split horizon). Do this BEFORE
        // handing `ann` to the index (which consumes by value) so
        // the forwarder has the current view of `hop_count`.
        if ann.hop_count < MAX_CAPABILITY_HOPS - 1 {
            let mut forwarded = ann.clone();
            forwarded.hop_count += 1;
            // `to_bytes` on a clone with the bumped counter —
            // signature remains valid because `signed_payload()`
            // zeros `hop_count` on verify.
            let fwd_bytes = forwarded.to_bytes();
            Self::forward_capability_announcement(fwd_bytes, ann.node_id, from_node, ctx);
        }

        ctx.capability_index.index(ann);
    }

    /// Fan an already-serialized capability announcement out to every
    /// directly-connected peer, minus the sender and any split-
    /// horizon-excluded peer. Spawned onto the runtime so the
    /// synchronous dispatch handler isn't blocked on per-peer
    /// encryption and network send. Mirrors the pingwave forwarding
    /// loop at the top of `dispatch_packet` — same split-horizon
    /// rule, same best-effort send semantics.
    fn forward_capability_announcement(
        payload: Vec<u8>,
        origin_node_id: u64,
        sender_node_id: u64,
        ctx: &DispatchCtx,
    ) {
        let peers = ctx.peers.clone();
        let socket = ctx.socket.clone();
        let partition_filter = ctx.partition_filter.clone();
        let router = ctx.router.clone();

        tokio::spawn(async move {
            // Split-horizon: consult the routing table for the
            // origin's best next hop and skip that peer. Matches the
            // pingwave rule so capability forwarding + pingwave
            // forwarding contribute to the same DV loop-avoidance
            // invariant.
            let next_hop_addr = router.routing_table().lookup(origin_node_id);

            for entry in peers.iter() {
                let peer = entry.value();
                if peer.node_id == sender_node_id {
                    continue; // never send back to whoever gave it to us
                }
                if Some(peer.addr) == next_hop_addr {
                    continue; // split horizon: that's our path to the origin
                }
                if partition_filter.contains(&peer.addr) {
                    continue;
                }

                // Build + send a subprotocol packet through this
                // peer's session. Same path as `send_subprotocol`;
                // inlined because the dispatch handler has no `self`.
                let session = &peer.session;
                let stream_id = SUBPROTOCOL_CAPABILITY_ANN as u64;
                let pool = session.thread_local_pool();
                let mut builder = pool.get();
                let seq = {
                    let stream = session.get_or_create_stream(stream_id);
                    stream.next_tx_seq()
                };
                let events = vec![Bytes::copy_from_slice(&payload)];
                let packet = builder.build_subprotocol(
                    stream_id,
                    seq,
                    &events,
                    PacketFlags::NONE,
                    SUBPROTOCOL_CAPABILITY_ANN,
                );
                let _ = socket.send_to(&packet, peer.addr).await;
                drop(builder);
                session.touch();
            }
        });
    }

    /// Decide whether a Subscribe from `from_node` on `channel` is allowed.
    ///
    /// Rules, in order:
    /// 1. Per-peer channel cap — rejects with `TooManyChannels`.
    /// 2. If a `channel_configs` registry is set and the channel isn't in
    ///    it, reject with `UnknownChannel`.
    /// 3. Channel [`Visibility`] must permit the subscriber's subnet
    ///    — reject cross-subnet subscribes with `Unauthorized`.
    /// 4. Channel auth — `publish_caps` / `subscribe_caps` /
    ///    `require_token` on `ChannelConfig` are honored via
    ///    `ChannelConfig::can_subscribe`. A presented token is
    ///    installed into the local `TokenCache` (after signature
    ///    verification) before the check runs.
    fn authorize_subscribe(
        channel: &ChannelName,
        from_node: u64,
        token_bytes: Option<&[u8]>,
        ctx: &DispatchCtx,
    ) -> (bool, Option<AckReason>) {
        // Rate-limit check runs first — a throttled peer short-
        // circuits without consuming any ed25519 work. The
        // failure counter increments only on actual auth-rule
        // rejections below (not on `TooManyChannels`, which is a
        // resource-limit failure, not an auth failure).
        if Self::is_auth_throttled(from_node, ctx) {
            return (false, Some(AckReason::RateLimited));
        }
        if ctx.roster.channels_for_peer_count(from_node) >= ctx.max_channels_per_peer {
            return (false, Some(AckReason::TooManyChannels));
        }
        let Some(ref configs) = ctx.channel_configs else {
            // No registry → no ACL (test / permissive deployments).
            return (true, None);
        };
        let Some(cfg_ref) = configs.get_by_name(channel.as_str()) else {
            return (false, Some(AckReason::UnknownChannel));
        };
        // Clone the cfg so we can drop the DashMap guard before
        // any further work — the cfg fields are all cheap to clone
        // and doing so releases the registry's read lock early.
        let cfg = cfg_ref.clone();
        drop(cfg_ref);

        let peer_subnet = ctx
            .peer_subnets
            .get(&from_node)
            .map(|e| *e.value())
            .unwrap_or(SubnetId::GLOBAL);
        if !Self::subnet_visible(ctx.local_subnet, peer_subnet, cfg.visibility) {
            return (false, Some(AckReason::Unauthorized));
        }

        // Parse + verify the presented token into a LOCAL scratch
        // value. We do NOT insert into the shared cache until the
        // full auth check passes — otherwise an attacker can spam
        // self-signed subscribes (which fail at cap/visibility
        // checks) yet leave their tokens permanently in the shared
        // cache, exhausting memory keyed under attacker-controlled
        // `(subject, channel_hash)` slots.
        let presented_token = token_bytes
            .and_then(|bytes| PermissionToken::from_bytes(bytes).ok())
            .filter(|tok| tok.verify().is_ok());

        // Whether any cap / token gate is in play. A fully open
        // channel (no filters, no require_token) short-circuits
        // without needing a peer entity_id at all.
        let has_auth_gates =
            cfg.publish_caps.is_some() || cfg.subscribe_caps.is_some() || cfg.require_token;
        if !has_auth_gates {
            return (true, None);
        }

        // Peer caps default to empty — subscribe-before-announce
        // races return `None` from the index; we treat that as an
        // empty capability set, which makes `subscribe_caps` filters
        // fail closed.
        let peer_caps = ctx.capability_index.get(from_node).unwrap_or_default();

        // Peer entity — load-bearing for `require_token`. Without
        // it we can't validate the subject. Missing entity +
        // require_token = reject.
        let Some(peer_entity) = ctx
            .peer_entity_ids
            .get(&from_node)
            .map(|e| e.value().clone())
        else {
            if cfg.require_token {
                return (false, Some(AckReason::Unauthorized));
            }
            // Cap-filter-only mode without a known entity — build a
            // dummy id so `can_subscribe` can still run the cap
            // match. Token check is skipped via `require_token=false`.
            let dummy = EntityId::from_bytes([0u8; 32]);
            let empty_cache = Arc::new(TokenCache::new());
            return if cfg.can_subscribe(&peer_caps, &dummy, &empty_cache) {
                (true, None)
            } else {
                (false, Some(AckReason::Unauthorized))
            };
        };

        // Run the ACL check against a scratch cache containing only
        // the presented token first. A positive verdict means the
        // fresh token alone is sufficient. If that fails, fall back
        // to the shared cache so a peer relying on a previously-
        // stored delegation can still re-subscribe without having
        // to re-present. The shared cache is read-only for this
        // decision.
        let scratch_cache = Arc::new(TokenCache::new());
        if let Some(ref tok) = presented_token {
            scratch_cache.insert_unchecked(tok.clone());
        }
        let passed_with_scratch = cfg.can_subscribe(&peer_caps, &peer_entity, &scratch_cache);
        let passed = passed_with_scratch
            || ctx
                .token_cache
                .as_ref()
                .is_some_and(|shared| cfg.can_subscribe(&peer_caps, &peer_entity, shared));
        if !passed {
            return (false, Some(AckReason::Unauthorized));
        }

        // Auth passed — now and only now promote the presented token
        // to the shared cache so future subscribes on the same
        // (subject, channel_hash) can skip re-presenting.
        if let (Some(tok), Some(shared)) = (presented_token, ctx.token_cache.as_ref()) {
            let _ = shared.insert(tok);
        }
        (true, None)
    }

    /// Check whether `from_node` is currently auth-throttled.
    /// Reads + clears the `throttled_until` instant atomically so
    /// an expired throttle state doesn't leak into future windows.
    fn is_auth_throttled(from_node: u64, ctx: &DispatchCtx) -> bool {
        if ctx.max_auth_failures_per_window == u16::MAX {
            return false; // threshold disabled
        }
        let Some(mut entry) = ctx.auth_failures.get_mut(&from_node) else {
            return false;
        };
        match entry.throttled_until {
            Some(until) if std::time::Instant::now() < until => true,
            Some(_) => {
                // Throttle elapsed — reset so the peer gets a
                // clean slate next time around.
                entry.throttled_until = None;
                entry.failures = 0;
                entry.window_start = None;
                false
            }
            None => false,
        }
    }

    /// Record an authorization-rule rejection against `from_node`.
    /// Increments the rolling-window counter; once it crosses
    /// `max_auth_failures_per_window`, marks the peer as throttled
    /// for `auth_throttle_duration`.
    fn record_auth_failure(from_node: u64, ctx: &DispatchCtx) {
        if ctx.max_auth_failures_per_window == u16::MAX {
            return;
        }
        let now = std::time::Instant::now();
        let mut entry = ctx.auth_failures.entry(from_node).or_default();
        // Window reset: if the current window has elapsed, start
        // fresh. Keeps failure counts from leaking across honest
        // retry storms separated by long idle periods.
        let reset_window = match entry.window_start {
            Some(start) => now.duration_since(start) >= ctx.auth_failure_window,
            None => true,
        };
        if reset_window {
            entry.window_start = Some(now);
            entry.failures = 0;
        }
        entry.failures = entry.failures.saturating_add(1);
        if entry.failures >= ctx.max_auth_failures_per_window {
            entry.throttled_until = Some(now + ctx.auth_throttle_duration);
        }
    }

    /// Wipe `from_node`'s failure counter. Called after a successful
    /// subscribe so honest peers that occasionally fail (stale
    /// token, renewal race) don't accumulate toward the throttle.
    fn clear_auth_failures(from_node: u64, ctx: &DispatchCtx) {
        ctx.auth_failures.remove(&from_node);
    }

    /// `true` if a packet with `visibility` originating in `source`
    /// should be delivered to a peer in `dest`.
    ///
    /// Mirrors the `SubnetGateway::should_forward` visibility matrix
    /// but doesn't need the gateway's state (`peer_subnets`,
    /// `export_table`). Regular participants use this for
    /// publish-fan-out filtering + subscribe-gate checks;
    /// border-gateway nodes with richer routing state should use the
    /// full `SubnetGateway` instead.
    ///
    /// `Exported` is conservative — returns `false` unless a
    /// per-channel export table is consulted elsewhere. Wiring that
    /// is a documented follow-up.
    fn subnet_visible(source: SubnetId, dest: SubnetId, visibility: Visibility) -> bool {
        match visibility {
            Visibility::Global => true,
            Visibility::SubnetLocal => source.is_same_subnet(dest),
            Visibility::ParentVisible => {
                source.is_same_subnet(dest)
                    || source.is_ancestor_of(dest)
                    || dest.is_ancestor_of(source)
            }
            Visibility::Exported => false,
        }
    }

    /// Send an `Ack` on the membership subprotocol back to `to_node`.
    /// Non-fatal if `to_node` is not in the peer map or the send fails;
    /// the requester will simply hit its ack timeout.
    fn send_membership_ack(
        to_node: u64,
        nonce: u64,
        accepted: bool,
        reason: Option<AckReason>,
        ctx: &DispatchCtx,
    ) {
        let Some(peer_entry) = ctx.peers.get(&to_node) else {
            return;
        };
        let dest_addr = peer_entry.value().addr;
        if ctx.partition_filter.contains(&dest_addr) {
            return;
        }
        let dest_sess = peer_entry.value().session.clone();
        let socket = ctx.socket.clone();
        let ack = MembershipMsg::Ack {
            nonce,
            accepted,
            reason,
        };
        let bytes = Bytes::from(membership::encode(&ack));
        drop(peer_entry);

        tokio::spawn(async move {
            let pool = dest_sess.thread_local_pool();
            let mut builder = pool.get();
            let stream_id = SUBPROTOCOL_CHANNEL_MEMBERSHIP as u64;
            let seq = {
                let stream = dest_sess.get_or_create_stream(stream_id);
                stream.next_tx_seq()
            };
            let events = vec![bytes];
            let packet = builder.build_subprotocol(
                stream_id,
                seq,
                &events,
                PacketFlags::NONE,
                SUBPROTOCOL_CHANNEL_MEMBERSHIP,
            );
            let _ = socket.send_to(&packet, dest_addr).await;
        });
    }

    // ── Channel fan-out (ChannelPublisher) ─────────────────────────────

    /// Build a [`ChannelPublisher`] recipe. Does NOT talk to the wire —
    /// combine with [`publish`](Self::publish) or
    /// [`publish_many`](Self::publish_many) to actually fan out.
    pub fn channel_publisher(
        &self,
        channel: ChannelName,
        config: PublishConfig,
    ) -> ChannelPublisher {
        ChannelPublisher::new(channel, config)
    }

    /// Fan `payload` out to every subscriber of the publisher's channel.
    ///
    /// One per-peer unicast per subscriber — no multicast primitive, no
    /// group crypto. Per-peer concurrency is bounded by
    /// `PublishConfig::max_inflight`. The failure policy controls whether
    /// per-peer errors short-circuit the fan-out (see [`OnFailure`]).
    pub async fn publish(
        &self,
        publisher: &ChannelPublisher,
        payload: Bytes,
    ) -> Result<PublishReport, AdapterError> {
        self.publish_many(publisher, &[payload]).await
    }

    /// Fan multiple payloads out to every subscriber of the publisher's
    /// channel. Semantics are the same as [`publish`](Self::publish); the
    /// whole `events` slice is delivered as one batch per subscriber.
    pub async fn publish_many(
        &self,
        publisher: &ChannelPublisher,
        events: &[Bytes],
    ) -> Result<PublishReport, AdapterError> {
        // Publisher-side auth: if the channel is registered with
        // `publish_caps` / `require_token`, the local node must
        // satisfy them *before* fan-out begins. Keeps a node from
        // silently publishing to a channel whose own ACL it doesn't
        // match. Channels absent from the registry are treated as
        // open (permissive default).
        let cfg_snapshot = self.channel_configs.as_ref().and_then(|cr| {
            cr.get_by_name(publisher.channel().name().as_str())
                .map(|c| c.clone())
        });
        if let Some(cfg) = cfg_snapshot.as_ref() {
            if cfg.publish_caps.is_some() || cfg.require_token {
                let self_caps = self
                    .local_announcement
                    .load()
                    .as_deref()
                    .map(|ann| ann.capabilities.clone())
                    .unwrap_or_default();
                let self_entity = self.identity.entity_id().clone();
                let cache = self
                    .token_cache
                    .clone()
                    .unwrap_or_else(|| Arc::new(TokenCache::new()));
                if !cfg.can_publish(&self_caps, &self_entity, &cache) {
                    return Err(AdapterError::Connection(
                        "channel: publish denied by channel ACL".into(),
                    ));
                }
            }
        }

        // Snapshot subscribers at call time; late subscribers won't see
        // this publish, early-unsubscribes may still receive it — both
        // are documented non-goals.
        let mut subscribers = self.roster.members(publisher.channel());

        // Subnet visibility filter. Look up the channel's visibility
        // (default `Global` for unregistered names so a publisher
        // without a registry still delivers) and drop any subscriber
        // whose subnet isn't visible under that rule. Filtered
        // subscribers don't show up in `attempted` or `errors` —
        // they're policy decisions, not failures.
        let visibility = cfg_snapshot
            .as_ref()
            .map(|c| c.visibility)
            .unwrap_or(Visibility::Global);
        subscribers.retain(|peer_id| {
            let peer_subnet = self
                .peer_subnets
                .get(peer_id)
                .map(|e| *e.value())
                .unwrap_or(SubnetId::GLOBAL);
            Self::subnet_visible(self.local_subnet, peer_subnet, visibility)
        });

        // AuthGuard fast path. Populated by `authorize_subscribe`;
        // revoked on unsubscribe and by the expiry sweep. Consulted
        // on every publish so revocations take effect on the next
        // fan-out without waiting for a roster refresh.
        //
        // Three-way verdict:
        //
        // - `Allowed`: bloom hit + verified-cache entry says yes.
        //   The verified cache is keyed on the 16-bit `channel_hash`
        //   that rides the wire header, which collides routinely at
        //   mesh scale — one subscriber's grant on channel A can
        //   falsely admit them on channel B when the hashes alias.
        //   Cross-check the canonical name against `exact` before
        //   trusting the verdict; a mismatch means the allow came
        //   from a different channel that happened to collide.
        // - `Denied`: bloom miss — no auth entry exists for this
        //   (origin, channel). Skip the subscriber.
        // - `Unknown`: bloom hit but verified cache missed. Fall
        //   back to the exact-channel ACL. On hit, promote back
        //   into the verified cache so subsequent publishes take
        //   the fast path.
        //
        // Open channels (no auth configured) are admitted on every
        // subscribe via `allow_channel`, so the fast path trivially
        // passes for them — no conditional branch needed.
        //
        // Additionally, when the channel is `require_token`, we do a
        // **lazy expiry check** on each admitted subscriber. The
        // periodic token sweep (`spawn_token_sweep_loop`) is the
        // primary eviction path, but a caller may deliberately
        // configure `token_sweep_interval = Duration::MAX` to opt out
        // — and without a second line of defence an expired token
        // would keep authorizing packets forever. Probing the token
        // cache per admitted subscriber costs one DashMap lookup +
        // a timestamp compare per publish, which is only paid on
        // token-gated channels (`require_token = false` skips the
        // branch entirely). An expired subscriber is revoked inline
        // so the next publish takes the `Denied` path.
        let channel_name = publisher.channel().name().clone();
        let channel_hash = channel_name.hash();
        let auth_guard = self.auth_guard.clone();
        let require_token = cfg_snapshot
            .as_ref()
            .map(|c| c.require_token)
            .unwrap_or(false);
        subscribers.retain(|peer_id| {
            let origin = subscriber_origin_hash(*peer_id);
            let admitted = match auth_guard.check_fast(origin, channel_hash) {
                AuthVerdict::Allowed => auth_guard.is_authorized_full(origin, &channel_name),
                AuthVerdict::Denied => false,
                AuthVerdict::NeedsFullCheck => {
                    if auth_guard.is_authorized_full(origin, &channel_name) {
                        auth_guard.allow_channel(origin, &channel_name);
                        true
                    } else {
                        false
                    }
                }
            };
            if !admitted {
                return false;
            }
            if !require_token {
                return true;
            }
            // Token-gated branch: ensure the subscriber still has a
            // valid token. If the cache answer is "no valid token
            // authorizes SUBSCRIBE on this channel," revoke inline.
            let (Some(cache), Some(entity)) = (
                self.token_cache.as_ref(),
                self.peer_entity_ids.get(peer_id).map(|e| e.value().clone()),
            ) else {
                // Missing entity binding or no cache installed —
                // treat as unauthorized. The subscribe path would
                // have rejected this peer in the first place;
                // reaching here means a config drift that we must
                // not paper over by admitting the publish.
                auth_guard.revoke_channel(origin, &channel_name);
                return false;
            };
            if cache
                .check(&entity, TokenScope::SUBSCRIBE, channel_hash)
                .is_err()
            {
                auth_guard.revoke_channel(origin, &channel_name);
                return false;
            }
            true
        });

        let mut report = PublishReport {
            attempted: subscribers.len(),
            delivered: 0,
            errors: Vec::new(),
        };
        if subscribers.is_empty() {
            return Ok(report);
        }

        let reliable = publisher.config().reliability.is_reliable();
        let stream_id = Self::publish_stream_id(publisher.channel());
        let max_inflight = publisher.config().max_inflight;
        let on_failure = publisher.config().on_failure;

        use tokio::sync::Semaphore;
        let sem = Arc::new(Semaphore::new(max_inflight.max(1)));

        match on_failure {
            OnFailure::FailFast => {
                // Sequential; stop on first error. Concurrency isn't
                // meaningful here because we'd be discarding in-flight
                // results anyway.
                for peer_id in &subscribers {
                    match self
                        .publish_to_peer(*peer_id, stream_id, reliable, events)
                        .await
                    {
                        Ok(()) => report.delivered += 1,
                        Err(e) => {
                            report.errors.push((*peer_id, e));
                            return Ok(report);
                        }
                    }
                }
                Ok(report)
            }
            OnFailure::BestEffort | OnFailure::Collect => {
                let mut handles = Vec::with_capacity(subscribers.len());
                for peer_id in subscribers {
                    let permit = Arc::clone(&sem);
                    let events_owned: Vec<Bytes> = events.to_vec();
                    let fut = async move {
                        let _permit = permit.acquire_owned().await.ok();
                        (
                            peer_id,
                            self.publish_to_peer(peer_id, stream_id, reliable, &events_owned)
                                .await,
                        )
                    };
                    handles.push(fut);
                }
                let results = futures::future::join_all(handles).await;
                for (peer_id, res) in results {
                    match res {
                        Ok(()) => report.delivered += 1,
                        Err(e) => report.errors.push((peer_id, e)),
                    }
                }
                // BestEffort returns Ok as long as at least one subscriber
                // got the payload — empty roster was handled above, so
                // here there was at least one attempt.
                if matches!(on_failure, OnFailure::BestEffort)
                    && report.delivered == 0
                    && !report.errors.is_empty()
                {
                    let first = report
                        .errors
                        .first()
                        .map(|(id, e)| {
                            format!(
                                "all {} peers failed (first: {:#x}: {})",
                                report.attempted, id, e
                            )
                        })
                        .unwrap_or_else(|| "all peers failed".into());
                    return Err(AdapterError::Connection(first));
                }
                Ok(report)
            }
        }
    }

    /// Encode the channel hash into a `u64` stream id so that per-channel
    /// ordering holds within a session. Hash collisions between channels
    /// are possible but harmless here — streams are opaque u64 to the
    /// transport and have no ACL meaning.
    fn publish_stream_id(channel: &ChannelId) -> u64 {
        // Place channel hash in the low 16 bits; the upper bits stay zero
        // so that channel-keyed publisher streams don't alias the common
        // subprotocol range (0x0400..0x0A00).
        0x0001_0000_0000_0000 | (channel.hash() as u64)
    }

    /// Send one per-peer leg of a publish. Reuses the same packet-build
    /// path as `send_on_stream`, with an explicit stream opened per
    /// `(peer, channel)` pair.
    async fn publish_to_peer(
        &self,
        peer_node_id: u64,
        stream_id: u64,
        reliable: bool,
        events: &[Bytes],
    ) -> Result<(), AdapterError> {
        let (dest_addr, session) = match self.peers.get(&peer_node_id) {
            Some(p) => (p.value().addr, p.value().session.clone()),
            None => {
                return Err(AdapterError::Connection(format!(
                    "publish: no session for subscriber {:#x}",
                    peer_node_id
                )));
            }
        };

        if self.partition_filter.contains(&dest_addr) {
            return Err(AdapterError::Connection(format!(
                "publish: peer {:#x} is partitioned",
                peer_node_id
            )));
        }

        // Ensure a stream is open with the right reliability mode.
        // `open_stream_with` seeds the stream with
        // `DEFAULT_STREAM_WINDOW_BYTES` so publish traffic rides
        // the same v2 byte-credit window as `send_on_stream`.
        session.open_stream_with(stream_id, reliable, 1);

        // Charge credit on the wire-byte size of the packet we're
        // about to build. The `TxSlotGuard` refunds on Drop unless
        // we `commit()` after a successful socket send, so a failed
        // send doesn't strand credit.
        let payload_bytes: usize = events.iter().map(|e| EventFrame::LEN_SIZE + e.len()).sum();
        let needed = wire_bytes_for_payload(payload_bytes);
        let (guard, seq) = match session.try_acquire_tx_credit_guard(stream_id, needed) {
            TxAdmit::Acquired { guard, seq } => (guard, seq),
            TxAdmit::WindowFull => {
                return Err(AdapterError::Connection(format!(
                    "publish: stream {:#x} backpressured",
                    stream_id
                )));
            }
            TxAdmit::StreamClosed => {
                return Err(AdapterError::Connection(format!(
                    "publish: stream {:#x} closed",
                    stream_id
                )));
            }
        };

        let pool = session.thread_local_pool();
        let mut builder = pool.get();
        let packet = builder.build_subprotocol(
            stream_id,
            seq,
            events,
            PacketFlags::NONE,
            0, /* subprotocol_id 0 = event-plane */
        );

        let next_hop = self
            .router
            .routing_table()
            .lookup(peer_node_id)
            .unwrap_or(dest_addr);

        self.socket
            .send_to(&packet, next_hop)
            .await
            .map_err(|e| AdapterError::Connection(format!("publish send failed: {}", e)))?;
        guard.commit(); // wire-accepted — bytes now belong to the receiver

        drop(builder);
        session.touch();
        Ok(())
    }

    /// Send a raw subprotocol message to a peer.
    ///
    /// The payload is sent as a single event frame with the specified
    /// `subprotocol_id` set in the Net header (included in AEAD AAD).
    pub async fn send_subprotocol(
        &self,
        peer_addr: SocketAddr,
        subprotocol_id: u16,
        payload: &[u8],
    ) -> Result<(), AdapterError> {
        if self.partition_filter.contains(&peer_addr) {
            return Ok(());
        }

        let node_id = self
            .addr_to_node
            .get(&peer_addr)
            .map(|e| *e.value())
            .ok_or_else(|| AdapterError::Connection("unknown peer".into()))?;
        let peer = self
            .peers
            .get(&node_id)
            .ok_or_else(|| AdapterError::Connection("unknown peer".into()))?;

        let session = &peer.session;
        let stream_id = subprotocol_id as u64;

        let pool = session.thread_local_pool();
        let mut builder = pool.get();

        let seq = {
            let stream = session.get_or_create_stream(stream_id);
            stream.next_tx_seq()
        };

        let events = vec![Bytes::copy_from_slice(payload)];
        let packet =
            builder.build_subprotocol(stream_id, seq, &events, PacketFlags::NONE, subprotocol_id);

        self.socket
            .send_to(&packet, peer_addr)
            .await
            .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;

        drop(builder);
        session.touch();
        Ok(())
    }

    // ── Capability announcements ──────────────────────────────────────
    //
    // `SUBPROTOCOL_CAPABILITY_ANN` payloads; the on-wire form is
    // `CapabilityAnnouncement::to_bytes`. Direct-peer push only in
    // v1; multi-hop gossip is a follow-up.

    /// Announce this node's capabilities to every directly-connected
    /// peer. Also self-indexes so single-node `find_peers_by_filter`
    /// queries return us too.
    ///
    /// TTL defaults to 5 minutes. Unsigned (signatures tie in with
    /// Stage E channel auth). For explicit control over TTL or
    /// signing, see [`Self::announce_capabilities_with`].
    pub async fn announce_capabilities(&self, caps: CapabilitySet) -> Result<(), AdapterError> {
        // Default to signed — the node always has a keypair (either
        // caller-supplied or ephemeral at construction time), so
        // signing is free and closes the trust-on-first-use gap.
        self.announce_capabilities_with(caps, Duration::from_secs(300), true)
            .await
    }

    /// Extended announce with explicit TTL and signing opt-in.
    ///
    /// `sign = true` signs the announcement with the node's
    /// [`EntityKeypair`] so receivers can validate end-to-end.
    /// `sign = false` broadcasts unsigned — useful in trusted
    /// environments where the wire signature adds no value.
    /// Receivers with `require_signed_capabilities = true` drop
    /// unsigned announcements regardless.
    pub async fn announce_capabilities_with(
        &self,
        caps: CapabilitySet,
        ttl: Duration,
        sign: bool,
    ) -> Result<(), AdapterError> {
        let version = self.capability_version.fetch_add(1, Ordering::Relaxed) + 1;
        let mut ann = CapabilityAnnouncement::new(
            self.node_id,
            self.identity.entity_id().clone(),
            version,
            caps,
        )
        .with_ttl(ttl.as_secs().min(u32::MAX as u64) as u32);
        if sign {
            ann.sign(&self.identity);
        }

        // Self-index so local queries see our own caps. Always runs
        // regardless of rate limit — the self-index reflects the
        // latest intended announcement.
        self.capability_index.index(ann.clone());

        // Publish as the latest local announcement so future
        // session-opens push this version to new peers. Also always
        // runs so late joiners get the latest caps even when we've
        // rate-limited away the broadcast.
        self.local_announcement.store(Some(Arc::new(ann.clone())));

        // Origin-side rate limit: within-window calls update the
        // self-index + `local_announcement` but skip the network
        // broadcast. Callers that want to force an immediate
        // re-broadcast should lower `min_announce_interval` on
        // `MeshNodeConfig`.
        let now = std::time::Instant::now();
        let min_interval = self.config.min_announce_interval;
        let should_broadcast = {
            let mut last = self.last_announce_at.lock();
            let elapsed = last.map(|t| now.saturating_duration_since(t));
            let can_send = elapsed.is_none_or(|e| e >= min_interval);
            if can_send {
                *last = Some(now);
            }
            can_send
        };
        if !should_broadcast {
            return Ok(());
        }

        // Fan out to currently-connected peers. Best-effort — a
        // per-peer send failure is logged and skipped rather than
        // short-circuiting the broadcast.
        let bytes = ann.to_bytes();
        let peer_addrs: Vec<SocketAddr> = self.peers.iter().map(|e| e.value().addr).collect();
        for addr in peer_addrs {
            if let Err(e) = self
                .send_subprotocol(addr, SUBPROTOCOL_CAPABILITY_ANN, &bytes)
                .await
            {
                tracing::trace!(peer = %addr, error = %e, "capability: announce send failed");
            }
        }
        Ok(())
    }

    /// Query the capability index. Returns node ids (including our
    /// own `node_id`) whose latest announcement matches `filter`.
    pub fn find_peers_by_filter(&self, filter: &CapabilityFilter) -> Vec<u64> {
        self.capability_index.query(filter)
    }

    /// Rank peers for a scored requirement. Returns the best-
    /// scoring node's id, or `None` if no peer matches.
    pub fn rank_peers(&self, req: &CapabilityRequirement) -> Option<u64> {
        self.capability_index.find_best(req)
    }

    /// Shared reference to the capability index. Use this for
    /// queries the two helpers above don't cover (listing all known
    /// peers, reading stats, etc.).
    pub fn capability_index(&self) -> &Arc<CapabilityIndex> {
        &self.capability_index
    }

    /// Push the currently-stored local announcement (if any) to
    /// `peer_addr`. Called from the end of `connect` / `accept` so
    /// late joiners don't have to wait for a re-announce. No-op
    /// when we haven't yet announced anything.
    async fn push_local_announcement(&self, peer_addr: SocketAddr) {
        let Some(ann) = self.local_announcement.load_full() else {
            return;
        };
        let bytes = ann.to_bytes();
        if let Err(e) = self
            .send_subprotocol(peer_addr, SUBPROTOCOL_CAPABILITY_ANN, &bytes)
            .await
        {
            tracing::trace!(
                peer = %peer_addr,
                error = %e,
                "capability: session-open push failed"
            );
        }
    }

    // ── Stream API ─────────────────────────────────────────────────────

    /// Open (or look up) a logical stream to a connected peer.
    ///
    /// A stream is one ordered, independently reliability-configured
    /// channel inside the encrypted session to `peer_node_id`. Multiple
    /// streams share one session, one cipher, and one UDP socket, but
    /// have independent sequence numbers and reliability state. See
    /// [`Stream`] for the full contract.
    ///
    /// **Idempotent:** repeated calls for the same `(peer_node_id,
    /// stream_id)` return handles backed by the same underlying state;
    /// a config argument that differs from the first call's is ignored
    /// with a warning log. Close + re-open to change a stream's config.
    pub fn open_stream(
        &self,
        peer_node_id: u64,
        stream_id: u64,
        config: StreamConfig,
    ) -> Result<Stream, AdapterError> {
        let peer = self.peers.get(&peer_node_id).ok_or_else(|| {
            AdapterError::Connection(format!(
                "open_stream: no session for peer {:#x}",
                peer_node_id
            ))
        })?;
        let reliable = config.reliability.is_reliable();
        // Capture the freshly-allocated (or existing, on idempotent
        // re-open) epoch so the returned `Stream` handle can later
        // reject stale sends after a close+reopen.
        let epoch = peer.session.open_stream_full(
            stream_id,
            reliable,
            config.fairness_weight,
            config.window_bytes,
        );
        // Propagate the weight to the router's fair scheduler so
        // forwarded traffic on this stream (e.g., multi-hop relays
        // where we're an intermediate) respects the weight too. v1
        // caveat: local outbound sends via `send_on_stream` bypass the
        // scheduler; the weight only becomes observable on the wire
        // when a packet with this stream_id transits *this* node as
        // a forwarder. Documented in STREAM_MULTIPLEXING_PLAN.md.
        self.router
            .scheduler()
            .set_stream_weight(stream_id, config.fairness_weight);
        // Opportunistic eviction: if this open just pushed us over the
        // cap, trim via the same path as close_stream (idle==0 means
        // only the cap-exceeded pass runs).
        if peer.session.stream_count() > self.config.max_streams {
            peer.session.evict_idle_streams(
                Duration::from_nanos(u64::MAX),
                self.config.max_streams,
                "cap_exceeded",
            );
        }
        Ok(Stream {
            peer_node_id,
            stream_id,
            epoch,
            config,
        })
    }

    /// Close a stream: drop its `StreamState` from the session, ending
    /// delivery of any buffered inbound events for the stream and
    /// dropping outbound packets that haven't hit the wire yet.
    /// Idempotent. `CloseBehavior::DrainThenClose` is honored only to
    /// the extent the router's scheduler has already flushed; there is
    /// no wire "drain-then-close" signal in v1.
    pub fn close_stream(&self, peer_node_id: u64, stream_id: u64) {
        if let Some(peer) = self.peers.get(&peer_node_id) {
            peer.session.close_stream(stream_id);
        }
    }

    /// Send a batch of events on an explicit stream.
    ///
    /// Uses the stream's reliability mode from its original `open_stream`
    /// config. Returns `Backpressure` when the stream's in-flight count
    /// (`tx_inflight`) would exceed its configured `tx_window`; the event
    /// was not enqueued — the caller decides what to do (drop, retry,
    /// or buffer at the app layer). `tx_window == 0` disables the check
    /// and preserves pre-backpressure behavior. `Transport` is returned
    /// for underlying socket send failures.
    ///
    /// Returns `NotConnected` when the stream was never opened or has
    /// been closed since (`close_stream`, idle eviction, cap-exceeded
    /// LRU). A previously-closed `Stream` handle is inert by design —
    /// reusing it does NOT silently re-create the stream with default
    /// config; the caller must explicitly re-open.
    pub async fn send_on_stream(
        &self,
        stream: &Stream,
        events: &[Bytes],
    ) -> Result<(), StreamError> {
        let peer = self
            .peers
            .get(&stream.peer_node_id)
            .ok_or(StreamError::NotConnected)?;
        let peer_addr = peer.addr;
        let session = peer.session.clone();
        drop(peer);

        if self.partition_filter.contains(&peer_addr) {
            return Ok(()); // matches send_to_peer's silent drop
        }

        let stream_id = stream.stream_id;
        let reliable = stream.config.reliability.is_reliable();

        // Refuse to send on a stream that isn't currently open, OR
        // whose live state has a different epoch than the handle. The
        // second case covers the subtle "close + reopen with the same
        // id" bug: the handle's epoch was captured at its original
        // open, but a reopen allocates a fresh `StreamState` with a
        // new epoch. A naive existence-only check would silently
        // reroute the send onto the new stream — wrong config, wrong
        // stats, wrong tx_window accounting.
        match session.try_stream(stream_id) {
            None => return Err(StreamError::NotConnected),
            Some(state) if state.epoch() != stream.epoch => {
                return Err(StreamError::NotConnected);
            }
            Some(_) => {}
        }

        let pool = session.thread_local_pool();
        let mut builder = pool.get();

        let mut current_batch: Vec<Bytes> = Vec::with_capacity(64);
        let mut current_size = 0usize;

        // Each socket send acquires byte credit from the stream's
        // `tx_credit_remaining`. On success we `commit()` the guard —
        // the bytes now belong to the receiver, which will refund via
        // `StreamWindow` grants once it drains them. On any failure
        // (socket error, cancellation, `close_stream` race) the guard
        // drops without commit and refunds the bytes — the bytes
        // never hit the wire, so pretending they did would strand
        // credit. `NotConnected` is surfaced when the stream
        // disappears mid-call.
        let flags = if reliable {
            PacketFlags::RELIABLE
        } else {
            PacketFlags::NONE
        };

        for event in events {
            let frame_size = EventFrame::LEN_SIZE + event.len();
            if current_size + frame_size > protocol::MAX_PAYLOAD_SIZE && !current_batch.is_empty() {
                // Charge the **wire size** (Net header + AEAD tag +
                // payload) rather than just the event-frame payload
                // so the byte window matches the bandwidth the sender
                // actually pumps onto the link. Both ends add the
                // same fixed per-packet overhead, so sender and
                // receiver accounting stay symmetric.
                //
                // `TxAdmit::Acquired` returns credit + sequence under
                // the same DashMap lookup — a close+reopen race can't
                // slip a stale sequence from the old lifetime onto
                // the new state.
                let needed = wire_bytes_for_payload(current_size);
                let (guard, seq) = match session.try_acquire_tx_credit_matching_epoch(
                    stream_id,
                    stream.epoch,
                    needed,
                ) {
                    TxAdmit::Acquired { guard, seq } => (guard, seq),
                    TxAdmit::WindowFull => return Err(StreamError::Backpressure),
                    TxAdmit::StreamClosed => return Err(StreamError::NotConnected),
                };
                let packet = builder.build(stream_id, seq, &current_batch, flags);
                let send_res = self.socket.send_to(&packet, peer_addr).await;
                send_res.map_err(|e| StreamError::Transport(format!("send failed: {}", e)))?;
                guard.commit(); // socket accepted the packet — bytes are the receiver's now
                current_batch.clear();
                current_size = 0;
            }
            current_batch.push(event.clone());
            current_size += frame_size;
        }

        if !current_batch.is_empty() {
            let needed = wire_bytes_for_payload(current_size);
            let (guard, seq) =
                match session.try_acquire_tx_credit_matching_epoch(stream_id, stream.epoch, needed)
                {
                    TxAdmit::Acquired { guard, seq } => (guard, seq),
                    TxAdmit::WindowFull => return Err(StreamError::Backpressure),
                    TxAdmit::StreamClosed => return Err(StreamError::NotConnected),
                };
            let packet = builder.build(stream_id, seq, &current_batch, flags);
            let send_res = self.socket.send_to(&packet, peer_addr).await;
            send_res.map_err(|e| StreamError::Transport(format!("send failed: {}", e)))?;
            guard.commit();
        }

        drop(builder);
        session.touch();
        Ok(())
    }

    /// Send `events` on `stream`, retrying on `Backpressure` with
    /// exponential backoff (5 ms → 200 ms, doubling) up to `max_retries`
    /// times. Transport failures are returned immediately — they're a
    /// real error, not a pressure signal, and retrying would just mask
    /// them. Returns the final `Backpressure` error if the stream stays
    /// saturated across every attempt.
    pub async fn send_with_retry(
        &self,
        stream: &Stream,
        events: &[Bytes],
        max_retries: usize,
    ) -> Result<(), StreamError> {
        let mut delay = Duration::from_millis(5);
        let cap = Duration::from_millis(200);
        let mut last_backpressure: Option<StreamError> = None;
        for _ in 0..max_retries.saturating_add(1) {
            match self.send_on_stream(stream, events).await {
                Ok(()) => return Ok(()),
                Err(StreamError::Backpressure) => {
                    last_backpressure = Some(StreamError::Backpressure);
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(cap);
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_backpressure.unwrap_or(StreamError::Backpressure))
    }

    /// Convenience wrapper around [`send_with_retry`](Self::send_with_retry)
    /// with a generous retry count. Blocks the calling task until the
    /// send succeeds or a transport error occurs. Use when you'd rather
    /// wait than drop; prefer `send_with_retry` if you need a concrete
    /// upper bound on retry attempts.
    pub async fn send_blocking(
        &self,
        stream: &Stream,
        events: &[Bytes],
    ) -> Result<(), StreamError> {
        // 4096 retries × 200 ms cap = ~13 minutes in the worst case,
        // effectively "block until the network lets up or something is
        // actually wrong." Callers that need a tighter bound should use
        // `send_with_retry` directly.
        self.send_with_retry(stream, events, 4096).await
    }

    /// Snapshot of per-stream stats for a single stream.
    ///
    /// Returns `None` if either the peer or the stream doesn't exist.
    pub fn stream_stats(&self, peer_node_id: u64, stream_id: u64) -> Option<StreamStats> {
        let peer = self.peers.get(&peer_node_id)?;
        let state = peer.session.get_stream(stream_id)?;
        Some(StreamStats {
            tx_seq: state.current_tx_seq(),
            rx_seq: state.current_rx_seq(),
            inbound_pending: state.inbound_len() as u64,
            last_activity_ns: state.last_activity_ns(),
            active: state.is_active(),
            backpressure_events: state.backpressure_events(),
            tx_credit_remaining: state.tx_credit_remaining(),
            tx_window: state.tx_window(),
            credit_grants_received: state.credit_grants_received(),
            credit_grants_sent: state.credit_grants_sent(),
        })
    }

    /// Snapshot of per-stream stats for every stream in the session to
    /// `peer_node_id`. Empty vec if the peer doesn't exist.
    pub fn all_stream_stats(&self, peer_node_id: u64) -> Vec<(u64, StreamStats)> {
        let peer = match self.peers.get(&peer_node_id) {
            Some(p) => p,
            None => return Vec::new(),
        };
        let session = peer.session.clone();
        drop(peer);
        session
            .stream_ids()
            .into_iter()
            .filter_map(|sid| {
                let state = session.get_stream(sid)?;
                Some((
                    sid,
                    StreamStats {
                        tx_seq: state.current_tx_seq(),
                        rx_seq: state.current_rx_seq(),
                        inbound_pending: state.inbound_len() as u64,
                        last_activity_ns: state.last_activity_ns(),
                        active: state.is_active(),
                        backpressure_events: state.backpressure_events(),
                        tx_credit_remaining: state.tx_credit_remaining(),
                        tx_window: state.tx_window(),
                        credit_grants_received: state.credit_grants_received(),
                        credit_grants_sent: state.credit_grants_sent(),
                    },
                ))
            })
            .collect()
    }

    /// Connect to a peer whose first hop on the wire is `relay_addr`.
    ///
    /// The handshake is an ordinary Net packet with the `HANDSHAKE` flag
    /// plus a routing header addressed to `dest_node_id`. The routing
    /// layer forwards it hop-by-hop (like any other packet); the
    /// responder's msg2 comes back the same way. There's no separate
    /// subprotocol, no per-hop re-encryption — Noise NKpsk0 provides
    /// end-to-end confidentiality and authenticity, and the prologue
    /// binds `(src_node_id, dest_node_id)` so a relay that rewrites
    /// either identity in the routing header fails the responder's MAC
    /// check on msg1.
    ///
    /// `start()` must have been called before `connect_via` — the
    /// receive loop has to be running to deliver msg2 back to us.
    pub async fn connect_via(
        &self,
        relay_addr: SocketAddr,
        dest_pubkey: &[u8; 32],
        dest_node_id: u64,
    ) -> Result<u64, AdapterError> {
        // Build msg1. Prologue uses *routing-identity* (32-bit) versions
        // of (self, dest) — that's what a malicious relay could see and
        // rewrite in the routing header, so binding those bits into the
        // Noise transcript catches tampering. The FULL u64 self.node_id
        // is carried inside the msg1 payload (Noise-AEAD-authenticated),
        // so the responder learns it after decryption and can address
        // msg2 back to the correct u64 identity.
        let pending_key = routing_id(dest_node_id);
        let prologue = handshake_prologue(routing_id(self.node_id), pending_key);
        let mut noise =
            NoiseHandshake::initiator_with_prologue(&self.config.psk, dest_pubkey, &prologue)
                .map_err(|e| AdapterError::Fatal(format!("handshake init failed: {}", e)))?;
        let msg1 = noise
            .write_message(&self.node_id.to_le_bytes())
            .map_err(|e| AdapterError::Connection(format!("write_message failed: {}", e)))?;

        // Register pending-initiator state so the dispatch loop can
        // complete the handshake when msg2 arrives. Keyed by the
        // 32-bit routing identity because msg2's routing header carries
        // the truncated src_id — that's the only key the dispatch loop
        // has when it tries to find the matching initiator.
        let (tx, rx) = oneshot::channel();
        match self.pending_handshakes.entry(pending_key) {
            dashmap::mapref::entry::Entry::Occupied(_) => {
                return Err(AdapterError::Connection(format!(
                    "connect_via: handshake already in flight for peer {:#x}",
                    dest_node_id
                )));
            }
            dashmap::mapref::entry::Entry::Vacant(v) => {
                v.insert(PendingHandshake { noise, tx });
            }
        }

        // Wrap msg1 in a Net handshake packet + routing header and
        // send to the first hop. No session encryption — the handshake
        // payload is the raw Noise bytes, authenticated/confidential
        // by Noise itself.
        let inner = {
            let mut builder = PacketBuilder::new(&[0u8; 32], 0);
            builder.build_handshake(&msg1)
        };
        let routing = RoutingHeader::new(dest_node_id, self.node_id as u32, DEFAULT_HANDSHAKE_TTL);
        let mut routed = bytes::BytesMut::with_capacity(ROUTING_HEADER_SIZE + inner.len());
        routed.extend_from_slice(&routing.to_bytes());
        routed.extend_from_slice(&inner);
        if let Err(e) = self.socket.send_to(&routed, relay_addr).await {
            self.pending_handshakes.remove(&pending_key);
            return Err(AdapterError::Connection(format!("send failed: {}", e)));
        }

        // Wait for the dispatch loop to complete msg2.
        let keys = match tokio::time::timeout(self.config.handshake_timeout, rx).await {
            Ok(Ok(Ok(k))) => k,
            Ok(Ok(Err(e))) => {
                self.pending_handshakes.remove(&pending_key);
                return Err(AdapterError::Fatal(format!("handshake failed: {}", e)));
            }
            Ok(Err(_)) => {
                self.pending_handshakes.remove(&pending_key);
                return Err(AdapterError::Connection("handshake channel dropped".into()));
            }
            Err(_) => {
                self.pending_handshakes.remove(&pending_key);
                return Err(AdapterError::Connection("handshake timeout".into()));
            }
        };

        // Register the new peer with `relay_addr` as the wire address.
        // Packets to `dest_node_id` go to the relay first; the routing
        // table does the rest. `addr_to_node` is deliberately NOT
        // updated — `relay_addr` still maps to the relay's own node_id
        // for direct-packet dispatch.
        let session = Arc::new(NetSession::new(
            keys,
            relay_addr,
            self.config.packet_pool_size,
            self.config.default_reliable,
        ));
        self.router.add_route(dest_node_id, relay_addr);
        self.peers.insert(
            dest_node_id,
            PeerInfo {
                node_id: dest_node_id,
                addr: relay_addr,
                session,
            },
        );
        self.peer_addrs.insert(dest_node_id, relay_addr);

        Ok(dest_node_id)
    }

    /// Connect to a peer by node id, using the routing table to pick the
    /// first hop. Fails with `Connection("no route to ...")` if the
    /// routing table doesn't have a route to the destination yet — in
    /// which case the caller can retry once pingwaves have propagated.
    pub async fn connect_routed(
        &self,
        dest_pubkey: &[u8; 32],
        dest_node_id: u64,
    ) -> Result<u64, AdapterError> {
        let first_hop = self
            .router
            .routing_table()
            .lookup(dest_node_id)
            .ok_or_else(|| {
                AdapterError::Connection(format!(
                    "connect_routed: no route to peer {:#x}",
                    dest_node_id
                ))
            })?;
        self.connect_via(first_hop, dest_pubkey, dest_node_id).await
    }

    // ── Handshake helpers ───────────────────────────────────────────────

    async fn handshake_initiator(
        &self,
        peer_addr: SocketAddr,
        peer_pubkey: &[u8; 32],
        peer_node_id: u64,
    ) -> Result<SessionKeys, AdapterError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self
                .try_handshake_initiator(peer_addr, peer_pubkey, peer_node_id)
                .await
            {
                Ok(keys) => return Ok(keys),
                Err(e) if attempt < self.config.handshake_retries => {
                    tracing::warn!(attempt, error = %e, "mesh handshake failed, retrying");
                    tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_handshake_initiator(
        &self,
        peer_addr: SocketAddr,
        peer_pubkey: &[u8; 32],
        peer_node_id: u64,
    ) -> Result<SessionKeys, AdapterError> {
        let timeout = self.config.handshake_timeout;
        let socket_arc = self.socket.socket_arc();

        // Prologue uses the 32-bit `routing_id` projection of the node
        // ids — the same projection routed handshakes use, so the two
        // paths share one prologue convention. Direct handshakes don't
        // traverse the routing plane, but the unified convention
        // simplifies reasoning and future code reuse.
        let prologue = handshake_prologue(routing_id(self.node_id), routing_id(peer_node_id));
        let mut handshake =
            NoiseHandshake::initiator_with_prologue(&self.config.psk, peer_pubkey, &prologue)
                .map_err(|e| AdapterError::Fatal(format!("handshake init failed: {}", e)))?;

        let msg1 = handshake
            .write_message(&[])
            .map_err(|e| AdapterError::Connection(format!("write_message failed: {}", e)))?;

        let mut builder = PacketBuilder::new(&[0u8; 32], 0);
        let packet = builder.build_handshake(&msg1);

        self.socket
            .send_to(&packet, peer_addr)
            .await
            .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;

        // Wait for response
        let parsed = tokio::time::timeout(timeout, async {
            loop {
                let mut recv_buf = bytes::BytesMut::with_capacity(protocol::MAX_PACKET_SIZE);
                recv_buf.resize(protocol::MAX_PACKET_SIZE, 0);

                let (n, source) = socket_arc
                    .recv_from(&mut recv_buf)
                    .await
                    .map_err(|e| AdapterError::Connection(format!("recv failed: {}", e)))?;

                if source != peer_addr {
                    continue;
                }

                recv_buf.truncate(n);
                let data = recv_buf.freeze();

                if let Some(p) = ParsedPacket::parse(data, source) {
                    if p.header.flags.is_handshake() {
                        return Ok::<_, AdapterError>(p);
                    }
                }
            }
        })
        .await
        .map_err(|_| AdapterError::Connection("handshake timeout".into()))??;

        handshake
            .read_message(&parsed.payload)
            .map_err(|e| AdapterError::Connection(format!("read_message failed: {}", e)))?;

        handshake
            .into_session_keys()
            .map_err(|e| AdapterError::Fatal(format!("key extraction failed: {}", e)))
    }

    async fn handshake_responder(
        &self,
        peer_node_id: u64,
    ) -> Result<(SessionKeys, SocketAddr), AdapterError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_handshake_responder(peer_node_id).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.config.handshake_retries => {
                    tracing::warn!(attempt, error = %e, "mesh accept failed, retrying");
                    tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_handshake_responder(
        &self,
        peer_node_id: u64,
    ) -> Result<(SessionKeys, SocketAddr), AdapterError> {
        let timeout = self.config.handshake_timeout;
        let socket_arc = self.socket.socket_arc();

        // Wait for initiator's handshake
        let (parsed, source) = tokio::time::timeout(timeout, async {
            loop {
                let mut recv_buf = bytes::BytesMut::with_capacity(protocol::MAX_PACKET_SIZE);
                recv_buf.resize(protocol::MAX_PACKET_SIZE, 0);

                let (n, source) = socket_arc
                    .recv_from(&mut recv_buf)
                    .await
                    .map_err(|e| AdapterError::Connection(format!("recv failed: {}", e)))?;

                recv_buf.truncate(n);
                let data = recv_buf.freeze();

                if let Some(p) = ParsedPacket::parse(data, source) {
                    if p.header.flags.is_handshake() {
                        return Ok::<_, AdapterError>((p, source));
                    }
                }
            }
        })
        .await
        .map_err(|_| AdapterError::Connection("handshake timeout".into()))??;

        // Direct responder: mirror the initiator's `routing_id`-based
        // prologue so direct and routed share one convention.
        let prologue = handshake_prologue(routing_id(peer_node_id), routing_id(self.node_id));
        let mut handshake = NoiseHandshake::responder_with_prologue(
            &self.config.psk,
            &self.static_keypair,
            &prologue,
        )
        .map_err(|e| AdapterError::Fatal(format!("handshake init failed: {}", e)))?;

        handshake
            .read_message(&parsed.payload)
            .map_err(|e| AdapterError::Connection(format!("read_message failed: {}", e)))?;

        let msg2 = handshake
            .write_message(&[])
            .map_err(|e| AdapterError::Connection(format!("write_message failed: {}", e)))?;

        let mut builder = PacketBuilder::new(&[0u8; 32], 0);
        let packet = builder.build_handshake(&msg2);

        self.socket
            .send_to(&packet, source)
            .await
            .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;

        let keys = handshake
            .into_session_keys()
            .map_err(|e| AdapterError::Fatal(format!("key extraction failed: {}", e)))?;

        Ok((keys, source))
    }
}

// ── Adapter trait impl ──────────────────────────────────────────────────

#[async_trait]
impl Adapter for MeshNode {
    async fn init(&mut self) -> Result<(), AdapterError> {
        // MeshNode is initialized via new() + connect(). This is a no-op.
        Ok(())
    }

    async fn on_batch(&self, batch: Batch) -> Result<(), AdapterError> {
        // Send to the first connected peer. For a real mesh, this should
        // use the routing table to pick the right peer based on the
        // event's destination. For now, round-robin or first-match.
        let peer_addr = self
            .peers
            .iter()
            .next()
            .map(|e| e.value().addr)
            .ok_or_else(|| AdapterError::Connection("no peers connected".into()))?;

        self.send_to_peer(peer_addr, batch).await
    }

    async fn flush(&self) -> Result<(), AdapterError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), AdapterError> {
        self.shutdown.store(true, Ordering::Release);
        self.shutdown_notify.notify_waiters();
        self.router.stop();

        // Deactivate all sessions
        for entry in self.peers.iter() {
            entry.value().session.deactivate();
        }

        // Wait for background tasks
        let tasks = std::mem::take(&mut *self.tasks.lock().await);
        for handle in tasks {
            let _ = handle.await;
        }

        Ok(())
    }

    async fn poll_shard(
        &self,
        shard_id: u16,
        from_id: Option<&str>,
        limit: usize,
    ) -> Result<ShardPollResult, AdapterError> {
        let queue = match self.inbound.get(&shard_id) {
            Some(q) => q,
            None => return Ok(ShardPollResult::empty()),
        };

        let mut events = Vec::with_capacity(limit.min(1000));
        let mut last_id = None;
        // from_id is ignored — the SegQueue is consume-once, so every pop
        // removes the event permanently. Cursor-based skipping would destroy
        // events that have already been consumed. Callers should consume
        // from the head without a cursor.
        let _ = from_id;

        for _ in 0..limit {
            match queue.pop() {
                Some(event) => {
                    last_id = Some(event.id.clone());
                    events.push(event);
                }
                None => break,
            }
        }

        let has_more = !queue.is_empty();

        Ok(ShardPollResult {
            events,
            next_id: last_id,
            has_more,
        })
    }

    fn name(&self) -> &'static str {
        "mesh"
    }

    async fn is_healthy(&self) -> bool {
        self.started.load(Ordering::Acquire) && !self.peers.is_empty()
    }
}

impl Drop for MeshNode {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.shutdown_notify.notify_waiters();
        self.router.stop();
    }
}
