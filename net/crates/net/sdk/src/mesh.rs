//! Multi-peer mesh handle for the SDK.
//!
//! `Mesh` wraps [`MeshNode`] with ergonomic methods for connecting to
//! peers, sending events, and polling received events. Unlike [`Net`],
//! which is backed by an [`EventBus`] + adapter, `Mesh` manages its own
//! encrypted peer sessions and routing.
//!
//! # Example
//!
//! ```rust,no_run
//! use net_sdk::mesh::{Mesh, MeshBuilder};
//!
//! # async fn example() -> net_sdk::error::Result<()> {
//! let mut node = Mesh::builder("127.0.0.1:9000", b"my-32-byte-preshared-key-here!!!")?
//!     .heartbeat_ms(200)
//!     .session_timeout_ms(5000)
//!     .build()
//!     .await?;
//!
//! // Connect to a peer
//! let peer_pubkey = [0u8; 32]; // get from peer.public_key()
//! node.connect("127.0.0.1:9001", &peer_pubkey, 0x2222).await?;
//! node.start();
//!
//! // Send events
//! node.send(0x2222, &serde_json::json!({"token": "hello"})).await?;
//!
//! // Poll received events
//! let events = node.recv(100).await?;
//!
//! node.shutdown().await?;
//! # Ok(())
//! # }
//! ```

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use serde::Serialize;

use net::adapter::net::{
    AckReason, ChannelConfig, ChannelConfigRegistry, ChannelName, ChannelPublisher, EntityKeypair,
    MeshNode, MeshNodeConfig, MigrationSubprotocolHandler, PublishConfig, PublishReport, Stream,
    StreamConfig, StreamStats,
};
use net::adapter::Adapter;
use net::event::StoredEvent;

use crate::error::{Result, SdkError};

/// Builder for configuring a [`Mesh`] node.
pub struct MeshBuilder {
    bind_addr: SocketAddr,
    psk: [u8; 32],
    heartbeat_interval: Duration,
    session_timeout: Duration,
    num_shards: u16,
}

impl MeshBuilder {
    /// Create a new builder.
    pub fn new(bind_addr: &str, psk: &[u8; 32]) -> Result<Self> {
        let addr: SocketAddr = bind_addr
            .parse()
            .map_err(|e| SdkError::Config(format!("invalid bind address: {}", e)))?;
        Ok(Self {
            bind_addr: addr,
            psk: *psk,
            heartbeat_interval: Duration::from_secs(5),
            session_timeout: Duration::from_secs(30),
            num_shards: 4,
        })
    }

    /// Set heartbeat interval in milliseconds.
    pub fn heartbeat_ms(mut self, ms: u64) -> Self {
        self.heartbeat_interval = Duration::from_millis(ms);
        self
    }

    /// Set session timeout in milliseconds.
    pub fn session_timeout_ms(mut self, ms: u64) -> Self {
        self.session_timeout = Duration::from_millis(ms);
        self
    }

    /// Set number of inbound shards.
    pub fn shards(mut self, n: u16) -> Self {
        self.num_shards = n;
        self
    }

    /// Build the mesh node.
    pub async fn build(self) -> Result<Mesh> {
        let identity = EntityKeypair::generate();
        let config = MeshNodeConfig::new(self.bind_addr, self.psk)
            .with_heartbeat_interval(self.heartbeat_interval)
            .with_session_timeout(self.session_timeout)
            .with_num_shards(self.num_shards)
            .with_handshake(3, Duration::from_secs(5));

        let mut node = MeshNode::new(identity, config).await?;
        // Install a shared ChannelConfigRegistry so `register_channel`
        // can add entries without needing `&mut Mesh` — the registry
        // itself uses interior mutability (DashMap).
        let channel_configs = Arc::new(ChannelConfigRegistry::new());
        node.set_channel_configs(channel_configs.clone());
        Ok(Mesh {
            node,
            channel_configs,
        })
    }
}

/// A multi-peer mesh node.
///
/// Manages encrypted connections to multiple peers over a single UDP
/// socket. Supports direct peer-to-peer sends, routed multi-hop sends,
/// automatic failure detection, and rerouting.
pub struct Mesh {
    node: MeshNode,
    /// Channel config registry shared with the underlying `MeshNode`
    /// so `register_channel` / subscriber ACL checks operate on the
    /// same live data.
    channel_configs: Arc<ChannelConfigRegistry>,
}

impl Mesh {
    /// Create a builder.
    pub fn builder(bind_addr: &str, psk: &[u8; 32]) -> Result<MeshBuilder> {
        MeshBuilder::new(bind_addr, psk)
    }

    /// Get this node's Noise public key.
    ///
    /// Share this with peers so they can connect to this node.
    pub fn public_key(&self) -> &[u8; 32] {
        self.node.public_key()
    }

    /// Get this node's ID (derived from ed25519 identity).
    pub fn node_id(&self) -> u64 {
        self.node.node_id()
    }

    /// Get the local bind address.
    pub fn local_addr(&self) -> SocketAddr {
        self.node.local_addr()
    }

    /// Connect to a peer as initiator.
    ///
    /// The peer must be listening (call `accept()` on their side).
    /// `peer_pubkey` is the peer's Noise public key from `public_key()`.
    pub async fn connect(
        &self,
        peer_addr: &str,
        peer_pubkey: &[u8; 32],
        peer_node_id: u64,
    ) -> Result<()> {
        let addr: SocketAddr = peer_addr
            .parse()
            .map_err(|e| SdkError::Config(format!("invalid peer address: {}", e)))?;
        self.node.connect(addr, peer_pubkey, peer_node_id).await?;
        Ok(())
    }

    /// Accept an incoming connection as responder.
    ///
    /// Waits for a peer to initiate a Noise handshake.
    /// Returns the peer's address.
    pub async fn accept(&self, peer_node_id: u64) -> Result<SocketAddr> {
        let (addr, _) = self.node.accept(peer_node_id).await?;
        Ok(addr)
    }

    /// Start the receive loop, heartbeat sender, and router.
    ///
    /// Call this after connecting to peers. Events won't be received
    /// until `start()` is called.
    pub fn start(&self) {
        self.node.start();
    }

    /// Number of connected peers.
    pub fn peer_count(&self) -> usize {
        self.node.peer_count()
    }

    // ---- Sending ----

    /// Send a serializable event to a direct peer.
    pub async fn send_to(&self, peer_addr: &str, event: &impl Serialize) -> Result<()> {
        let addr: SocketAddr = peer_addr
            .parse()
            .map_err(|e| SdkError::Config(format!("invalid address: {}", e)))?;
        let json = serde_json::to_vec(event)?;
        let batch = net::event::Batch {
            shard_id: 0,
            events: vec![net::event::InternalEvent::new(Bytes::from(json), 0, 0)],
            sequence_start: 0,
        };
        self.node.send_to_peer(addr, batch).await?;
        Ok(())
    }

    /// Send a serializable event via the routing table.
    ///
    /// The event is encrypted for the destination and forwarded through
    /// intermediate nodes if needed. Requires a route to `dest_node_id`
    /// in the routing table and a session with the destination.
    pub async fn send(&self, dest_node_id: u64, event: &impl Serialize) -> Result<()> {
        let json = serde_json::to_vec(event)?;
        let batch = net::event::Batch {
            shard_id: 0,
            events: vec![net::event::InternalEvent::new(Bytes::from(json), 0, 0)],
            sequence_start: 0,
        };
        self.node.send_routed(dest_node_id, batch).await?;
        Ok(())
    }

    /// Send raw bytes to a direct peer.
    pub async fn send_raw_to(&self, peer_addr: &str, data: &[u8]) -> Result<()> {
        let addr: SocketAddr = peer_addr
            .parse()
            .map_err(|e| SdkError::Config(format!("invalid address: {}", e)))?;
        let batch = net::event::Batch {
            shard_id: 0,
            events: vec![net::event::InternalEvent::new(
                Bytes::copy_from_slice(data),
                0,
                0,
            )],
            sequence_start: 0,
        };
        self.node.send_to_peer(addr, batch).await?;
        Ok(())
    }

    // ---- Receiving ----

    /// Poll for received events.
    ///
    /// Returns up to `limit` events from all shards.
    pub async fn recv(&self, limit: usize) -> Result<Vec<StoredEvent>> {
        // Poll shard 0 (most events land here for single-stream sends)
        let result = self.node.poll_shard(0, None, limit).await?;
        Ok(result.events)
    }

    /// Poll a specific shard for events.
    pub async fn recv_shard(&self, shard_id: u16, limit: usize) -> Result<Vec<StoredEvent>> {
        let result = self.node.poll_shard(shard_id, None, limit).await?;
        Ok(result.events)
    }

    // ---- Channels (distributed pub/sub) ----

    /// Register a channel on this publisher. Subscribers who ask to
    /// join are validated against `config` before being added to the
    /// subscriber roster.
    ///
    /// `config.channel_id` must be built from the same canonical name
    /// subscribers pass to `subscribe_channel`. The registry keys on
    /// the canonical name (not the u16 hash) to avoid ACL bypass via
    /// hash collision.
    ///
    /// Idempotent: re-registering the same channel replaces the prior
    /// config.
    pub fn register_channel(&self, config: ChannelConfig) {
        self.channel_configs.insert(config);
    }

    /// Ask `publisher_node_id` to add this node to `channel`'s
    /// subscriber set. Blocks until the publisher's `Ack` arrives or
    /// the mesh's membership-ack timeout elapses.
    ///
    /// Returns `Ok(())` on acceptance; rejection (unauthorized /
    /// unknown channel / rate-limited / too-many-channels) surfaces
    /// as `SdkError::ChannelRejected(reason)`. Network-level failures
    /// surface as `SdkError::Adapter(...)`.
    pub async fn subscribe_channel(
        &self,
        publisher_node_id: u64,
        channel: &ChannelName,
    ) -> Result<()> {
        match self
            .node
            .subscribe_channel(publisher_node_id, channel.clone())
            .await
        {
            Ok(()) => Ok(()),
            Err(e) => Err(adapter_to_channel_error(e)),
        }
    }

    /// Mirror of [`Self::subscribe_channel`]. Idempotent on the
    /// publisher side — unsubscribing a non-subscriber still returns
    /// `Ok(())`.
    pub async fn unsubscribe_channel(
        &self,
        publisher_node_id: u64,
        channel: &ChannelName,
    ) -> Result<()> {
        match self
            .node
            .unsubscribe_channel(publisher_node_id, channel.clone())
            .await
        {
            Ok(()) => Ok(()),
            Err(e) => Err(adapter_to_channel_error(e)),
        }
    }

    /// Publish one payload to every subscriber of `channel`.
    /// `config.on_failure` controls whether per-peer errors
    /// short-circuit the fan-out. Returns a [`PublishReport`]
    /// describing per-peer outcomes.
    pub async fn publish(
        &self,
        channel: &ChannelName,
        payload: Bytes,
        config: PublishConfig,
    ) -> Result<PublishReport> {
        let publisher = ChannelPublisher::new(channel.clone(), config);
        Ok(self.node.publish(&publisher, payload).await?)
    }

    /// Fan multiple payloads to every subscriber of `channel` as one
    /// batch per subscriber. Semantics match [`Self::publish`].
    pub async fn publish_many(
        &self,
        channel: &ChannelName,
        payloads: &[Bytes],
        config: PublishConfig,
    ) -> Result<PublishReport> {
        let publisher = ChannelPublisher::new(channel.clone(), config);
        Ok(self.node.publish_many(&publisher, payloads).await?)
    }

    // ---- Routing ----

    /// Add a route to a destination node.
    ///
    /// Packets sent to `dest_node_id` via `send()` will be forwarded
    /// through `next_hop_addr`.
    pub fn add_route(&self, dest_node_id: u64, next_hop_addr: &str) -> Result<()> {
        let addr: SocketAddr = next_hop_addr
            .parse()
            .map_err(|e| SdkError::Config(format!("invalid address: {}", e)))?;
        self.node.router().add_route(dest_node_id, addr);
        Ok(())
    }

    /// Remove a route.
    pub fn remove_route(&self, dest_node_id: u64) {
        self.node.router().remove_route(dest_node_id);
    }

    // ---- Mesh topology ----

    /// Block a peer (simulate network partition).
    pub fn block_peer(&self, peer_addr: &str) -> Result<()> {
        let addr: SocketAddr = peer_addr
            .parse()
            .map_err(|e| SdkError::Config(format!("invalid address: {}", e)))?;
        self.node.block_peer(addr);
        Ok(())
    }

    /// Unblock a peer.
    pub fn unblock_peer(&self, peer_addr: &str) -> Result<()> {
        let addr: SocketAddr = peer_addr
            .parse()
            .map_err(|e| SdkError::Config(format!("invalid address: {}", e)))?;
        self.node.unblock_peer(&addr);
        Ok(())
    }

    /// Number of nodes discovered via pingwave propagation.
    pub fn discovered_nodes(&self) -> usize {
        self.node.proximity_graph().node_count()
    }

    /// Number of active reroutes (routes using alternates after failure).
    pub fn active_reroutes(&self) -> usize {
        self.node.reroute_policy().active_reroutes()
    }

    // ---- Streams ----

    /// Open (or look up) a logical stream to a peer. See
    /// [`net::adapter::net::MeshNode::open_stream`] for the full contract.
    /// Repeated calls for the same `(peer, stream_id)` are idempotent;
    /// the first open wins and subsequent configs are logged and
    /// ignored.
    pub fn open_stream(
        &self,
        peer_node_id: u64,
        stream_id: u64,
        config: StreamConfig,
    ) -> Result<Stream> {
        self.node
            .open_stream(peer_node_id, stream_id, config)
            .map_err(SdkError::from)
    }

    /// Close a stream: drop its `StreamState` and free the window. Idempotent.
    pub fn close_stream(&self, peer_node_id: u64, stream_id: u64) {
        self.node.close_stream(peer_node_id, stream_id);
    }

    /// Send a batch of events on an explicit stream.
    ///
    /// Returns [`SdkError::Backpressure`] when the stream's per-stream
    /// in-flight window is full (no events were sent — the caller
    /// decides whether to drop, retry, or buffer). [`SdkError::NotConnected`]
    /// when the peer session is gone. All other failures surface as
    /// [`SdkError::Adapter`].
    pub async fn send_on_stream(&self, stream: &Stream, events: &[Bytes]) -> Result<()> {
        self.node
            .send_on_stream(stream, events)
            .await
            .map_err(SdkError::from)
    }

    /// Send events, retrying on `Backpressure` with exponential backoff
    /// (5 ms → 200 ms, doubling) up to `max_retries` times. Transport
    /// errors and `NotConnected` are returned immediately.
    pub async fn send_with_retry(
        &self,
        stream: &Stream,
        events: &[Bytes],
        max_retries: usize,
    ) -> Result<()> {
        self.node
            .send_with_retry(stream, events, max_retries)
            .await
            .map_err(SdkError::from)
    }

    /// Block the calling task until the send succeeds or a transport
    /// error occurs. See [`Mesh::send_with_retry`] for finer control.
    pub async fn send_blocking(&self, stream: &Stream, events: &[Bytes]) -> Result<()> {
        self.node
            .send_blocking(stream, events)
            .await
            .map_err(SdkError::from)
    }

    /// Snapshot of per-stream stats (tx/rx seq, window, in-flight,
    /// backpressure count, activity).
    pub fn stream_stats(&self, peer_node_id: u64, stream_id: u64) -> Option<StreamStats> {
        self.node.stream_stats(peer_node_id, stream_id)
    }

    /// Snapshot stats for every stream in the session to `peer_node_id`.
    pub fn all_stream_stats(&self, peer_node_id: u64) -> Vec<(u64, StreamStats)> {
        self.node.all_stream_stats(peer_node_id)
    }

    // ---- Lifecycle ----

    /// Set a migration handler (for Mikoshi daemon migration).
    pub fn set_migration_handler(&mut self, handler: Arc<MigrationSubprotocolHandler>) {
        self.node.set_migration_handler(handler);
    }

    /// Gracefully shut down.
    pub async fn shutdown(self) -> Result<()> {
        self.node.shutdown().await?;
        Ok(())
    }

    /// Get a reference to the underlying `MeshNode`.
    pub fn inner(&self) -> &MeshNode {
        &self.node
    }
}

/// Map an `AdapterError` from a subscribe / unsubscribe / publish
/// call into the channel-aware `SdkError` variant. Rejection acks
/// come through as `AdapterError::Connection("membership request
/// rejected: Some(<reason>)")`; parse that into
/// [`SdkError::ChannelRejected`].
fn adapter_to_channel_error(err: net::error::AdapterError) -> SdkError {
    use net::error::AdapterError;
    if let AdapterError::Connection(ref msg) = err {
        let prefix = "membership request rejected: ";
        if let Some(tail) = msg.strip_prefix(prefix) {
            let reason = parse_ack_reason(tail);
            return SdkError::ChannelRejected(reason);
        }
    }
    SdkError::from(err)
}

fn parse_ack_reason(s: &str) -> Option<AckReason> {
    // `{:?}` of `Option<AckReason>` produces `Some(Unauthorized)` etc.
    let inside = s.trim().strip_prefix("Some(")?.strip_suffix(')')?;
    match inside {
        "Unauthorized" => Some(AckReason::Unauthorized),
        "UnknownChannel" => Some(AckReason::UnknownChannel),
        "RateLimited" => Some(AckReason::RateLimited),
        "TooManyChannels" => Some(AckReason::TooManyChannels),
        _ => None,
    }
}
