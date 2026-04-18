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
//! let mut node = Mesh::builder("127.0.0.1:9000", b"my-32-byte-preshared-key-here!!")
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

use net::adapter::net::{EntityKeypair, MeshNode, MeshNodeConfig, MigrationSubprotocolHandler};
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

        let node = MeshNode::new(identity, config).await?;
        Ok(Mesh { node })
    }
}

/// A multi-peer mesh node.
///
/// Manages encrypted connections to multiple peers over a single UDP
/// socket. Supports direct peer-to-peer sends, routed multi-hop sends,
/// automatic failure detection, and rerouting.
pub struct Mesh {
    node: MeshNode,
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
