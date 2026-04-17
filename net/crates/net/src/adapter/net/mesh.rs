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

use super::crypto::{NoiseHandshake, SessionKeys, StaticKeypair};
use super::failure::{FailureDetector, FailureDetectorConfig};
use super::identity::EntityKeypair;
use super::pool::PacketBuilder;

use super::compute::SUBPROTOCOL_MIGRATION;
use super::protocol::{self, EventFrame, PacketFlags, MAGIC};
use super::route::{RoutingHeader, ROUTING_HEADER_SIZE};
use super::router::{NetRouter, RouterConfig};
use super::session::NetSession;
use super::subprotocol::MigrationSubprotocolHandler;
use super::transport::{NetSocket, PacketReceiver, ParsedPacket, SocketBufferConfig};

use crate::adapter::{Adapter, ShardPollResult};
use crate::error::AdapterError;
use crate::event::{Batch, StoredEvent};

/// Inbound event queues (same type as NetAdapter uses).
type InboundQueues = Arc<DashMap<u16, SegQueue<StoredEvent>>>;

/// Set of peer addresses whose packets should be silently dropped.
///
/// Used by test harnesses to simulate network partitions. When a peer's
/// address is in this set, both inbound and outbound packets are dropped
/// as if the network link is severed.
pub type PartitionFilter = Arc<dashmap::DashSet<SocketAddr>>;

/// Shared context for the packet dispatch loop.
struct DispatchCtx {
    local_node_id: u64,
    peers: Arc<DashMap<SocketAddr, PeerInfo>>,
    router: Arc<NetRouter>,
    failure_detector: Arc<FailureDetector>,
    inbound: InboundQueues,
    num_shards: u16,
    /// Optional subprotocol handler for migration messages.
    migration_handler: Option<Arc<MigrationSubprotocolHandler>>,
    /// Socket for sending outbound subprotocol responses.
    socket: Arc<NetSocket>,
    /// Partition filter — packets from blocked addresses are dropped.
    partition_filter: PartitionFilter,
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
    /// Encrypted session
    session: Arc<NetSession>,
}

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
    /// Per-peer sessions keyed by peer address
    peers: Arc<DashMap<SocketAddr, PeerInfo>>,
    /// Router for forwarding decisions
    router: Arc<NetRouter>,
    /// Failure detector
    failure_detector: Arc<FailureDetector>,
    /// Inbound event queues (shared with receive loop)
    inbound: InboundQueues,
    /// Optional migration subprotocol handler
    migration_handler: Option<Arc<MigrationSubprotocolHandler>>,
    /// Partition filter for simulating network splits
    partition_filter: PartitionFilter,
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

        let failure_detector = FailureDetector::with_config(FailureDetectorConfig {
            timeout: config.session_timeout,
            miss_threshold: 3,
            suspicion_threshold: 2,
            cleanup_interval: Duration::from_secs(60),
        });

        Ok(Self {
            identity,
            static_keypair,
            node_id,
            config,
            socket,
            peers: Arc::new(DashMap::new()),
            router: Arc::new(router),
            failure_detector: Arc::new(failure_detector),
            inbound: Arc::new(DashMap::new()),
            migration_handler: None,
            partition_filter: Arc::new(dashmap::DashSet::new()),
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
        let keys = self.handshake_initiator(peer_addr, peer_pubkey).await?;

        let session = Arc::new(NetSession::new(
            keys,
            peer_addr,
            self.config.packet_pool_size,
            self.config.default_reliable,
        ));

        // Add route so the router can forward packets to this peer
        self.router.add_route(peer_node_id, peer_addr);

        self.peers.insert(
            peer_addr,
            PeerInfo {
                node_id: peer_node_id,
                session,
            },
        );

        // Register with failure detector
        self.failure_detector.heartbeat(peer_node_id, peer_addr);

        Ok(peer_node_id)
    }

    /// Accept a connection from a peer. Performs Noise NKpsk0 as responder.
    ///
    /// Waits for an incoming handshake packet and completes the handshake.
    /// Returns the peer's address and assigns the given node_id.
    pub async fn accept(&self, peer_node_id: u64) -> Result<(SocketAddr, u64), AdapterError> {
        let (keys, peer_addr) = self.handshake_responder().await?;

        let session = Arc::new(NetSession::new(
            keys,
            peer_addr,
            self.config.packet_pool_size,
            self.config.default_reliable,
        ));

        self.router.add_route(peer_node_id, peer_addr);

        self.peers.insert(
            peer_addr,
            PeerInfo {
                node_id: peer_node_id,
                session,
            },
        );

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
            router: self.router.clone(),
            failure_detector: self.failure_detector.clone(),
            inbound: self.inbound.clone(),
            num_shards: self.config.num_shards,
            migration_handler: self.migration_handler.clone(),
            socket: self.socket.clone(),
            partition_filter: self.partition_filter.clone(),
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
                    if parsed.header.flags.is_handshake() || parsed.header.flags.is_heartbeat() {
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
                    // Not for us — forward without decrypting (header-only routing)
                    let _ = router.route_packet(data, source);
                }
            }
            return;
        }

        // Direct packet (no routing header) — standard path
        let parsed = match ParsedPacket::parse(data, source) {
            Some(p) => p,
            None => return,
        };

        if parsed.header.flags.is_handshake() {
            return;
        }

        let peer = match peers.get(&source) {
            Some(p) => p,
            None => return,
        };

        if parsed.header.flags.is_heartbeat() {
            if parsed.header.session_id == peer.session.session_id() {
                failure_detector.heartbeat(peer.node_id, source);
                peer.session.touch();
            }
            return;
        }

        if parsed.header.session_id != peer.session.session_id() {
            return;
        }

        Self::process_local_packet(&parsed, &peer.session, ctx);
        peer.session.touch();
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
                                .iter()
                                .find(|e| e.value().node_id == msg.dest_node)
                                .map(|e| (*e.key(), e.value().session.clone()));

                            if let Some((dest_addr, dest_sess)) = dest_session {
                                let socket = ctx.socket.clone();
                                let payload = Bytes::from(msg.payload);
                                // Fire-and-forget: send the response packet
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
            }
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

        tokio::spawn(async move {
            while !shutdown.load(Ordering::Acquire) {
                tokio::select! {
                    _ = tokio::time::sleep(interval) => {
                        for entry in peers.iter() {
                            let peer_addr = *entry.key();
                            // Skip blocked peers (partition simulation)
                            if partition_filter.contains(&peer_addr) {
                                continue;
                            }
                            let session = &entry.value().session;
                            let mut builder =
                                PacketBuilder::new(&[0u8; 32], session.session_id());
                            let packet = builder.build_heartbeat();
                            let _ = socket.send_to(&packet, peer_addr).await;
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

        let peer = self
            .peers
            .get(&peer_addr)
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
            .iter()
            .find(|e| e.value().node_id == dest_node_id)
            .map(|e| (*e.key(), e.value().session.clone()))
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

        let peer = self
            .peers
            .get(&peer_addr)
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

    // ── Handshake helpers ───────────────────────────────────────────────

    async fn handshake_initiator(
        &self,
        peer_addr: SocketAddr,
        peer_pubkey: &[u8; 32],
    ) -> Result<SessionKeys, AdapterError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_handshake_initiator(peer_addr, peer_pubkey).await {
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
    ) -> Result<SessionKeys, AdapterError> {
        let timeout = self.config.handshake_timeout;
        let socket_arc = self.socket.socket_arc();

        let mut handshake = NoiseHandshake::initiator(&self.config.psk, peer_pubkey)
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

    async fn handshake_responder(&self) -> Result<(SessionKeys, SocketAddr), AdapterError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_handshake_responder().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.config.handshake_retries => {
                    tracing::warn!(attempt, error = %e, "mesh accept failed, retrying");
                    tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_handshake_responder(&self) -> Result<(SessionKeys, SocketAddr), AdapterError> {
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

        let mut handshake = NoiseHandshake::responder(&self.config.psk, &self.static_keypair)
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
            .map(|e| *e.key())
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
        let skip_until = from_id.map(|s| s.to_string());

        // Drain from the concurrent queue
        let mut skipping = skip_until.is_some();
        let max_drain = limit * 2; // drain extra to handle skip
        for _ in 0..max_drain {
            match queue.pop() {
                Some(event) => {
                    if skipping {
                        if Some(&event.id) == skip_until.as_ref() {
                            skipping = false;
                        }
                        continue;
                    }
                    last_id = Some(event.id.clone());
                    events.push(event);
                    if events.len() >= limit {
                        break;
                    }
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
