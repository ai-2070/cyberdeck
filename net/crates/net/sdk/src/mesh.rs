//! Multi-peer mesh handle for the SDK.
//!
//! `Mesh` wraps [`MeshNode`] with ergonomic methods for connecting to
//! peers, sending events, and polling received events. Unlike
//! [`crate::Net`], which is backed by an `EventBus` + adapter, `Mesh`
//! manages its own encrypted peer sessions and routing.
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

/// Options passed to [`Mesh::subscribe_channel_with`].
///
/// Today the only knob is a presented
/// [`PermissionToken`](crate::identity::PermissionToken) — the
/// shape is a struct rather than a bare `Option<Token>` so future
/// additions (request-side timeout override, subscribe priority,
/// etc.) don't break callers.
///
/// # Round-trip shape
///
/// ```no_run
/// # use std::time::Duration;
/// # use net_sdk::{ChannelName, Identity, SubscribeOptions, TokenScope};
/// # use net_sdk::mesh::MeshBuilder;
/// # async fn example(
/// #     publisher: &net_sdk::Mesh,
/// #     publisher_identity: &Identity,
/// #     subscriber_entity_id: net::adapter::net::identity::EntityId,
/// # ) -> net_sdk::error::Result<()> {
/// // Publisher issues a SUBSCRIBE-scope token for the subscriber.
/// // The publisher's own `Mesh` is bound to an `Identity`, so the
/// // token lands in its local cache when `issue_token` is called.
/// let channel = ChannelName::new("events/trades").unwrap();
/// let token = publisher_identity.issue_token(
///     subscriber_entity_id,
///     TokenScope::SUBSCRIBE,
///     &channel,
///     Duration::from_secs(600),
///     0, // no further delegation
/// );
///
/// // Subscriber (another `Mesh`) calls `subscribe_channel_with`,
/// // attaching the same token bytes they received from the
/// // publisher out of band. The publisher verifies the signature,
/// // checks `subject == subscriber_entity_id`, installs it in its
/// // cache, then runs `can_subscribe`.
/// let subscriber: &net_sdk::Mesh = unimplemented!();
/// subscriber
///     .subscribe_channel_with(
///         publisher.node_id(),
///         &channel,
///         SubscribeOptions { token: Some(token) },
///     )
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Default, Debug, Clone)]
pub struct SubscribeOptions {
    /// Token to attach to the subscribe request. The publisher
    /// verifies + installs it before running
    /// `ChannelConfig::can_subscribe`, so a matching token
    /// satisfies `require_token` channels end-to-end.
    pub token: Option<net::adapter::net::PermissionToken>,
}

/// Builder for configuring a [`Mesh`] node.
pub struct MeshBuilder {
    bind_addr: SocketAddr,
    psk: [u8; 32],
    heartbeat_interval: Duration,
    session_timeout: Duration,
    num_shards: u16,
    identity: Option<crate::identity::Identity>,
    subnet: Option<net::adapter::net::SubnetId>,
    subnet_policy: Option<Arc<net::adapter::net::SubnetPolicy>>,
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
            identity: None,
            subnet: None,
            subnet_policy: None,
        })
    }

    /// Pin this node to a caller-owned [`Identity`](crate::Identity).
    ///
    /// Without this call, `build()` generates an ephemeral keypair —
    /// fine for one-off sessions, but every restart produces a new
    /// entity id (and therefore a new node id). Provide an identity
    /// loaded from disk / vault / enclave to keep stable addressing.
    ///
    /// The identity's [`TokenCache`](crate::identity::TokenCache) is
    /// also bound to this mesh; tokens installed via
    /// [`Identity::install_token`](crate::identity::Identity::install_token)
    /// become available to the channel auth path at subscribe time.
    pub fn identity(mut self, identity: crate::identity::Identity) -> Self {
        self.identity = Some(identity);
        self
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

    /// Pin this node to a specific subnet. Defaults to
    /// [`SubnetId::GLOBAL`](crate::subnets::SubnetId) — no
    /// restriction. Visibility checks on the publish + subscribe
    /// paths compare against this value.
    pub fn subnet(mut self, id: net::adapter::net::SubnetId) -> Self {
        self.subnet = Some(id);
        self
    }

    /// Install a subnet policy that derives each peer's subnet from
    /// their capability announcement. Mesh-wide policy consistency
    /// is assumed — mismatched policies across nodes lead to
    /// asymmetric views of peer subnets.
    ///
    /// Accepts either an owned `SubnetPolicy` or an `Arc<SubnetPolicy>`
    /// via blanket `Into` support — useful when several builders
    /// share one policy at node construction time.
    pub fn subnet_policy(
        mut self,
        policy: impl Into<Arc<net::adapter::net::SubnetPolicy>>,
    ) -> Self {
        self.subnet_policy = Some(policy.into());
        self
    }

    /// Build the mesh node.
    pub async fn build(self) -> Result<Mesh> {
        // Use the caller's identity if one was set, otherwise mint an
        // ephemeral one. `MeshNode::new` takes the keypair by value,
        // so clone it out of the Arc when we have a shared identity.
        let (keypair, sdk_identity) = match self.identity {
            Some(id) => (id.keypair().as_ref().clone(), Some(id)),
            None => (EntityKeypair::generate(), None),
        };

        let mut config = MeshNodeConfig::new(self.bind_addr, self.psk)
            .with_heartbeat_interval(self.heartbeat_interval)
            .with_session_timeout(self.session_timeout)
            .with_num_shards(self.num_shards)
            .with_handshake(3, Duration::from_secs(5));
        if let Some(id) = self.subnet {
            config = config.with_subnet(id);
        }
        if let Some(policy) = self.subnet_policy {
            config = config.with_subnet_policy(policy);
        }

        let mut node = MeshNode::new(keypair, config).await?;
        // Install a shared ChannelConfigRegistry so `register_channel`
        // can add entries without needing `&mut Mesh` — the registry
        // itself uses interior mutability (DashMap).
        let channel_configs = Arc::new(ChannelConfigRegistry::new());
        node.set_channel_configs(channel_configs.clone());
        // Hand the caller's TokenCache to the mesh so channel auth
        // (`require_token` / `can_subscribe` / `can_publish`) has a
        // cache to consult + install incoming tokens into. Without
        // an identity, no cache is installed and `require_token`
        // channels will reject.
        if let Some(id) = sdk_identity.as_ref() {
            node.set_token_cache(id.token_cache().clone());
        }
        Ok(Mesh {
            node: Arc::new(node),
            channel_configs,
            identity: sdk_identity,
        })
    }
}

/// A multi-peer mesh node.
///
/// Manages encrypted connections to multiple peers over a single UDP
/// socket. Supports direct peer-to-peer sends, routed multi-hop sends,
/// automatic failure detection, and rerouting.
pub struct Mesh {
    /// Shared `MeshNode`. `Arc` rather than by-value so NAPI /
    /// FFI bindings can hand the same live node to multiple
    /// wrappers (e.g. a `DaemonRuntime` alongside the existing
    /// `NetMesh` class) without double-owning the underlying
    /// socket. All public methods go through `.inner()` (Arc
    /// deref), so holding the `Arc` changes no existing call
    /// sites.
    node: Arc<MeshNode>,
    /// Channel config registry shared with the underlying `MeshNode`
    /// so `register_channel` / subscriber ACL checks operate on the
    /// same live data.
    channel_configs: Arc<ChannelConfigRegistry>,
    /// Held onto so the caller's `TokenCache` (and future capability
    /// announcement state) stays alive for the mesh's lifetime —
    /// `MeshNode` was already handed a clone of the keypair, so this
    /// is purely for the auxiliary state that rides alongside.
    identity: Option<crate::identity::Identity>,
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
        self.subscribe_channel_with(publisher_node_id, channel, SubscribeOptions::default())
            .await
    }

    /// Subscribe with options — optionally presenting a
    /// [`PermissionToken`](crate::identity::PermissionToken).
    ///
    /// Use this when the publisher registered the channel with
    /// `require_token = true` and/or a `subscribe_caps` filter that
    /// your node's capabilities alone don't satisfy. The publisher
    /// verifies the token on arrival (signature + subject matches
    /// the subscribing peer's `EntityId`) and installs it into its
    /// local `TokenCache` before the ACL check runs.
    pub async fn subscribe_channel_with(
        &self,
        publisher_node_id: u64,
        channel: &ChannelName,
        opts: SubscribeOptions,
    ) -> Result<()> {
        let result = match opts.token {
            Some(token) => {
                self.node
                    .subscribe_channel_with_token(publisher_node_id, channel.clone(), token)
                    .await
            }
            None => {
                self.node
                    .subscribe_channel(publisher_node_id, channel.clone())
                    .await
            }
        };
        match result {
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

    // ---- Capability announcements ----

    /// Announce this node's capabilities to every directly-connected
    /// peer. Self-indexes too, so `find_peers` called from this same
    /// node matches on the announcement. Multi-hop propagation is
    /// deferred — peers more than one hop away will not see the
    /// announcement.
    ///
    /// Default TTL is 5 minutes; use
    /// [`Self::announce_capabilities_with`] to override.
    pub async fn announce_capabilities(
        &self,
        caps: crate::capabilities::CapabilitySet,
    ) -> Result<()> {
        self.node.announce_capabilities(caps).await?;
        Ok(())
    }

    /// Extended announce with explicit TTL and signing opt-in.
    /// `sign = true` is accepted but currently a no-op; signatures
    /// tie in with Stage E (channel auth), once `node_id` →
    /// `EntityId` binding is wired.
    pub async fn announce_capabilities_with(
        &self,
        caps: crate::capabilities::CapabilitySet,
        ttl: std::time::Duration,
        sign: bool,
    ) -> Result<()> {
        self.node
            .announce_capabilities_with(caps, ttl, sign)
            .await?;
        Ok(())
    }

    /// Query the capability index. Returns node ids whose latest
    /// announcement matches `filter`; includes our own `node_id` if
    /// our own announcement matches.
    pub fn find_peers(&self, filter: &crate::capabilities::CapabilityFilter) -> Vec<u64> {
        self.node.find_peers_by_filter(filter)
    }

    /// Rank peers for a scored placement requirement. Returns the
    /// single best-scoring node's id, or `None` if no peer matches.
    pub fn rank_peers(&self, req: &crate::capabilities::CapabilityRequirement) -> Option<u64> {
        self.node.rank_peers(req)
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

    /// Clone the `Arc`-shared `MeshNode` handle out of the mesh.
    ///
    /// Used by FFI bindings (currently: NAPI) that need to hand
    /// the same live node to the `net-sdk::compute::DaemonRuntime`
    /// **and** to their own wrapper class without constructing a
    /// second UDP socket. All public `MeshNode` operations go
    /// through `&MeshNode`, so two Arc holders observe exactly
    /// the same state.
    pub fn node_arc(&self) -> Arc<MeshNode> {
        self.node.clone()
    }

    /// Construct a `Mesh` that shares an existing `MeshNode` with
    /// another owner. Used by FFI bindings that already hold an
    /// `Arc<MeshNode>` (e.g. NAPI's `NetMesh`) and need a `Mesh`
    /// wrapper so the SDK's `DaemonRuntime` can be built against
    /// the same live node.
    ///
    /// Does not re-install `channel_configs` or a `TokenCache` —
    /// the owner of the original `MeshNode` is responsible for
    /// that wiring. Supplied `channel_configs` / `identity`
    /// arguments are held onto here purely so the `Mesh`'s own
    /// helpers (channel registration lookup, identity getter)
    /// have data to return.
    pub fn from_node_arc(
        node: Arc<MeshNode>,
        channel_configs: Arc<ChannelConfigRegistry>,
        identity: Option<crate::identity::Identity>,
    ) -> Self {
        Self {
            node,
            channel_configs,
            identity,
        }
    }

    /// Caller-owned identity bound to this mesh, if any. Returns
    /// `None` for meshes built without `.identity(...)` (ephemeral
    /// keypair).
    pub fn identity(&self) -> Option<&crate::identity::Identity> {
        self.identity.as_ref()
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
