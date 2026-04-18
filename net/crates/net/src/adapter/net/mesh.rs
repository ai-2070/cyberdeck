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
use std::sync::atomic::{AtomicBool, Ordering};
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
use super::identity::EntityKeypair;
use super::pool::PacketBuilder;

use super::behavior::loadbalance::HealthStatus;
use super::behavior::proximity::{EnhancedPingwave, ProximityConfig, ProximityGraph};
use super::channel::membership::{self, MembershipMsg, SUBPROTOCOL_CHANNEL_MEMBERSHIP};
use super::channel::{
    AckReason, ChannelConfigRegistry, ChannelId, ChannelName, ChannelPublisher, OnFailure,
    PublishConfig, PublishReport, SubscriberRoster,
};
use super::compute::SUBPROTOCOL_MIGRATION;
use super::protocol::{self, EventFrame, PacketFlags, MAGIC};
use super::reroute::ReroutePolicy;
use super::route::{RoutingHeader, ROUTING_HEADER_SIZE};
use super::router::{NetRouter, RouterConfig};
use super::session::NetSession;
use super::stream::{Stream, StreamConfig, StreamError, StreamStats};
use super::subprotocol::MigrationSubprotocolHandler;
use super::transport::{NetSocket, PacketReceiver, ParsedPacket, SocketBufferConfig};
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

/// Default TTL for the routing header we stamp on routed handshake
/// packets. Far above any realistic relay chain; the routing layer
/// drops at zero.
const DEFAULT_HANDSHAKE_TTL: u8 = 16;

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

        // Wire failure detector with reroute callbacks + roster eviction.
        let rp_failure = reroute_policy.clone();
        let rp_recovery = reroute_policy.clone();
        let roster_failure = roster.clone();
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
        })
        .on_recovery(move |node_id| rp_recovery.on_recovery(node_id));

        let pending_handshakes: Arc<DashMap<u64, PendingHandshake>> = Arc::new(DashMap::new());

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

        // Register in proximity graph (1-hop peer)
        let peer_graph_id = node_id_to_graph_id(peer_node_id);
        let pw = EnhancedPingwave::new(peer_graph_id, 0, 1).with_load(0, HealthStatus::Healthy);
        self.proximity_graph.on_pingwave(pw, peer_addr);

        // Register with failure detector
        self.failure_detector.heartbeat(peer_node_id, peer_addr);

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

        // Store handles — can't block here, but we need them for shutdown
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            let mut tasks = tasks.lock().await;
            tasks.push(recv_handle);
            tasks.push(heartbeat_handle);
            tasks.push(router_handle);
        });
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
                // Before handing to the proximity graph, seed a routing
                // entry for the origin if it's not us and not already a
                // direct peer. `source` is the direct peer that just
                // forwarded this pingwave to us, so it's a valid
                // next-hop by construction. `hop_count` in the wire
                // field is "hops the pingwave traversed before we
                // received it"; real distance from origin is
                // `hop_count + 1` (the final hop from `source` to us),
                // and we add +1 more so direct routes (metric 1) always
                // win over any indirect route.
                let origin_nid = graph_id_to_node_id(&pw.origin_id);
                if origin_nid != ctx.local_node_id {
                    let metric = (pw.hop_count as u16).saturating_add(2);
                    ctx.router
                        .routing_table()
                        .add_route_with_metric(origin_nid, source, metric);
                }
                // Process and optionally re-broadcast
                if let Some(fwd_pw) = ctx.proximity_graph.on_pingwave(pw, source) {
                    let fwd_bytes = fwd_pw.to_bytes();
                    let socket = ctx.socket.clone();
                    let peers = ctx.peers.clone();
                    let filter = ctx.partition_filter.clone();
                    // Re-broadcast to all peers except the sender
                    tokio::spawn(async move {
                        for entry in peers.iter() {
                            let addr = entry.value().addr;
                            if addr != source && !filter.contains(&addr) {
                                let _ = socket.send_to(&fwd_bytes, addr).await;
                            }
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
        let socket = ctx.socket.clone();
        let peers = ctx.peers.clone();
        let peer_addrs = ctx.peer_addrs.clone();
        let router = ctx.router.clone();
        let payload = routed.freeze();
        tokio::spawn(async move {
            if let Err(e) = socket.send_to(&payload, next_hop).await {
                tracing::warn!(
                    peer = format!("{:#x}", peer_node_id),
                    error = %e,
                    "routed handshake: msg2 send failed; unregistering peer"
                );
                peers.remove(&peer_node_id);
                peer_addrs.remove(&peer_node_id);
                router.routing_table().remove_route(peer_node_id);
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
                rx_cipher.update_rx_counter(counter);
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

        // Standard event path: parse event frames and queue
        let events = EventFrame::read_events(Bytes::from(decrypted), parsed.header.event_count);

        let stream_id = parsed.header.stream_id;
        let shard_id = if num_shards > 0 {
            (stream_id % num_shards as u64) as u16
        } else {
            0
        };

        {
            let stream = session.get_or_create_stream(stream_id);
            stream.with_reliability(|r| {
                r.on_receive(parsed.header.sequence);
            });
            stream.update_rx_seq(parsed.header.sequence);
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
    /// `can_publish` rules are consulted for incoming Subscribe messages.
    ///
    /// When unset (the default), all subscribes are accepted. This is fine
    /// for testing and for deployments that rely on network-layer isolation
    /// rather than per-channel ACL. Full capability/token-based authorization
    /// requires threading the sender's `CapabilitySet` through dispatch and
    /// is a follow-up.
    pub fn set_channel_configs(&mut self, configs: Arc<ChannelConfigRegistry>) {
        self.channel_configs = Some(configs);
    }

    /// Ask `publisher_node_id` to add this node to `channel`'s subscriber set.
    ///
    /// Blocks until the publisher's `Ack` arrives or
    /// `membership_ack_timeout` elapses. Returns `Ok(())` iff the publisher
    /// accepted the subscribe; `AckReason` failures surface as
    /// `AdapterError::Connection`.
    pub async fn subscribe_channel(
        &self,
        publisher_node_id: u64,
        channel: ChannelName,
    ) -> Result<(), AdapterError> {
        self.send_membership_request(publisher_node_id, channel, true)
            .await
    }

    /// Ask `publisher_node_id` to remove this node from `channel`'s
    /// subscriber set. Mirror of `subscribe_channel`.
    pub async fn unsubscribe_channel(
        &self,
        publisher_node_id: u64,
        channel: ChannelName,
    ) -> Result<(), AdapterError> {
        self.send_membership_request(publisher_node_id, channel, false)
            .await
    }

    async fn send_membership_request(
        &self,
        publisher_node_id: u64,
        channel: ChannelName,
        subscribe: bool,
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
            MembershipMsg::Subscribe { channel, nonce } => {
                let (accepted, reason) = Self::authorize_subscribe(&channel, from_node, ctx);
                if accepted {
                    let id = ChannelId::new(channel);
                    ctx.roster.add(id, from_node);
                }
                Self::send_membership_ack(from_node, nonce, accepted, reason, ctx);
            }
            MembershipMsg::Unsubscribe { channel, nonce } => {
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

    /// Decide whether a Subscribe from `from_node` on `channel` is allowed.
    ///
    /// Rules, in order:
    /// 1. Per-peer channel cap — rejects with `TooManyChannels`.
    /// 2. If a `channel_configs` registry is set and the channel isn't in
    ///    it, reject with `UnknownChannel`.
    /// 3. Otherwise accept. Full capability/token checks are deferred
    ///    until the dispatch context carries the sender's `CapabilitySet`.
    fn authorize_subscribe(
        channel: &ChannelName,
        from_node: u64,
        ctx: &DispatchCtx,
    ) -> (bool, Option<AckReason>) {
        if ctx.roster.channels_for_peer_count(from_node) >= ctx.max_channels_per_peer {
            return (false, Some(AckReason::TooManyChannels));
        }
        if let Some(ref configs) = ctx.channel_configs {
            if configs.get_by_name(channel.as_str()).is_none() {
                return (false, Some(AckReason::UnknownChannel));
            }
        }
        (true, None)
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
        // Snapshot subscribers at call time; late subscribers won't see
        // this publish, early-unsubscribes may still receive it — both
        // are documented non-goals.
        let subscribers = self.roster.members(publisher.channel());
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
        session.open_stream_with(stream_id, reliable, 1);

        let seq = {
            let stream = session.get_or_create_stream(stream_id);
            stream.next_tx_seq()
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
        peer.session
            .open_stream_with(stream_id, reliable, config.fairness_weight);
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
    /// config. `WouldBlock` is reserved for a future credit-windowed
    /// flow-control implementation — v1 returns `Transport` for any
    /// underlying send failure and success otherwise.
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

        let pool = session.thread_local_pool();
        let mut builder = pool.get();

        let mut current_batch: Vec<Bytes> = Vec::with_capacity(64);
        let mut current_size = 0usize;

        for event in events {
            let frame_size = EventFrame::LEN_SIZE + event.len();
            if current_size + frame_size > protocol::MAX_PAYLOAD_SIZE && !current_batch.is_empty() {
                let seq = {
                    let state = session.get_or_create_stream(stream_id);
                    state.next_tx_seq()
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
                    .map_err(|e| StreamError::Transport(format!("send failed: {}", e)))?;
                current_batch.clear();
                current_size = 0;
            }
            current_batch.push(event.clone());
            current_size += frame_size;
        }

        if !current_batch.is_empty() {
            let seq = {
                let state = session.get_or_create_stream(stream_id);
                state.next_tx_seq()
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
                .map_err(|e| StreamError::Transport(format!("send failed: {}", e)))?;
        }

        drop(builder);
        session.touch();
        Ok(())
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
