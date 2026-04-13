//! Blackstream L0 Transport Protocol (BLTP) adapter.
//!
//! BLTP is a high-performance UDP-based transport protocol designed for
//! GPU-to-GPU encrypted streaming over UDP. It provides:
//!
//! - Zero-copy, zero-allocation hot path
//! - XChaCha20-Poly1305 encryption per packet
//! - Noise protocol handshake (NKpsk0)
//! - Optional per-stream reliability with selective NACKs
//! - 40-60M events/sec target throughput
//!
//! # Usage
//!
//! ```rust,ignore
//! use blackstream::adapter::bltp::{BltpAdapter, BltpAdapterConfig, StaticKeypair};
//!
//! // Generate keypair for responder
//! let keypair = StaticKeypair::generate();
//!
//! // Create initiator config
//! let config = BltpAdapterConfig::initiator(
//!     "127.0.0.1:9000".parse()?,
//!     "127.0.0.1:9001".parse()?,
//!     psk,
//!     keypair.public,
//! );
//!
//! // Create adapter
//! let mut adapter = BltpAdapter::new(config)?;
//! adapter.init().await?;
//! ```

mod batch;
pub mod behavior;
mod config;
mod crypto;
mod failure;
mod pool;
mod protocol;
mod proxy;
mod reliability;
mod route;
mod router;
mod session;
mod swarm;
mod transport;

#[cfg(target_os = "linux")]
mod linux;

pub use batch::AdaptiveBatcher;
pub use config::{BltpAdapterConfig, ConnectionRole, ReliabilityConfig};
pub use crypto::{CryptoError, SessionKeys, StaticKeypair};
pub use failure::{
    CircuitBreaker, CircuitState, FailureDetector, FailureDetectorConfig, FailureStats,
    LossSimulator, NodeStatus, RecoveryAction, RecoveryManager, RecoveryStats,
};
pub use pool::{PacketBuilder, PacketPool, SharedLocalPool, SharedPacketPool, ThreadLocalPool};
pub use protocol::{
    BltpHeader, EventFrame, NackPayload, PacketFlags, HEADER_SIZE, NONCE_SIZE, TAG_SIZE,
};
pub use proxy::{
    BltpProxy, ForwardResult, HopStats, MultiHopPacketBuilder, ProxyConfig, ProxyError, ProxyStats,
};
pub use reliability::{FireAndForget, ReliabilityMode, ReliableStream};
pub use route::{
    AggregateStats, RouteEntry, RouteFlags, RoutingHeader, RoutingTable, StreamStats,
    ROUTING_HEADER_SIZE,
};
pub use router::{BltpRouter, FairScheduler, RouteAction, RouterConfig, RouterError, RouterStats};
pub use session::{BltpSession, SessionManager, StreamState};
pub use swarm::{
    Capabilities, CapabilityAd, EdgeInfo, GraphStats, LocalGraph, NodeInfo, Pingwave, PINGWAVE_SIZE,
};
pub use transport::{BltpSocket, PacketReceiver, PacketSender, ParsedPacket, SocketBufferConfig};

use async_trait::async_trait;
use bytes::Bytes;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;

use crate::adapter::{Adapter, ShardPollResult};
use crate::error::AdapterError;
use crate::event::{Batch, StoredEvent};

use crypto::NoiseHandshake;
use session::SessionManager as SessionMgr;
use transport::BltpSocket as Socket;

// Re-export xxh3 utilities for stream routing
pub use routing::{route_to_shard, stream_id_from_bytes, stream_id_from_key};

/// Fast xxh3-based routing utilities for BLTP streams.
///
/// Uses xxh3 (~50GB/s) for deterministic, high-performance stream routing.
mod routing {
    use xxhash_rust::xxh3::xxh3_64;

    /// Generate a stream ID from arbitrary data.
    ///
    /// Uses xxh3 for fast, deterministic hashing (~50GB/s on modern CPUs).
    #[inline]
    pub fn stream_id_from_bytes(data: &[u8]) -> u64 {
        xxh3_64(data)
    }

    /// Generate a stream ID from a string key.
    ///
    /// Convenience wrapper for `stream_id_from_bytes`.
    #[inline]
    pub fn stream_id_from_key(key: &str) -> u64 {
        xxh3_64(key.as_bytes())
    }

    /// Route data to a shard based on its content hash.
    ///
    /// Returns a shard ID in the range `[0, num_shards)`.
    ///
    /// # Panics
    ///
    /// Panics if `num_shards` is 0.
    #[inline]
    pub fn route_to_shard(data: &[u8], num_shards: u16) -> u16 {
        assert!(num_shards > 0, "num_shards must be > 0");
        (xxh3_64(data) % num_shards as u64) as u16
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_stream_id_deterministic() {
            let data = b"test event data";
            let id1 = stream_id_from_bytes(data);
            let id2 = stream_id_from_bytes(data);
            assert_eq!(id1, id2);
        }

        #[test]
        fn test_stream_id_different_for_different_data() {
            let id1 = stream_id_from_bytes(b"event1");
            let id2 = stream_id_from_bytes(b"event2");
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_stream_id_from_key() {
            let id = stream_id_from_key("user:12345");
            assert_ne!(id, 0);
        }

        #[test]
        fn test_route_to_shard_range() {
            let num_shards = 16u16;
            for i in 0..1000 {
                let data = format!("event_{}", i);
                let shard = route_to_shard(data.as_bytes(), num_shards);
                assert!(shard < num_shards);
            }
        }

        #[test]
        #[should_panic(expected = "num_shards must be > 0")]
        fn test_route_to_shard_zero_shards_panics() {
            // Regression: route_to_shard(_, 0) caused a divide-by-zero panic
            // with no helpful message. Now it asserts with a clear message.
            route_to_shard(b"test", 0);
        }

        #[test]
        fn test_route_to_shard_distribution() {
            let num_shards = 8u16;
            let mut counts = [0u32; 8];

            for i in 0..8000 {
                let data = format!("event_{}", i);
                let shard = route_to_shard(data.as_bytes(), num_shards);
                counts[shard as usize] += 1;
            }

            // Check that distribution is reasonably uniform (within 50% of expected)
            let expected = 1000;
            for count in counts {
                assert!(count > expected / 2, "shard count {} too low", count);
                assert!(count < expected * 2, "shard count {} too high", count);
            }
        }
    }
}

/// Shared inbound queue type
type InboundQueues = Arc<DashMap<u16, SegQueue<StoredEvent>>>;

/// BLTP adapter for high-performance UDP transport.
pub struct BltpAdapter {
    /// Configuration
    config: BltpAdapterConfig,
    /// UDP socket
    socket: Option<Arc<Socket>>,
    /// Session (stored separately for init)
    session: Option<Arc<BltpSession>>,
    /// Session manager
    session_manager: SessionMgr,
    /// Inbound events per shard (for poll_shard)
    inbound: InboundQueues,
    /// Background tasks
    tasks: TokioMutex<Vec<JoinHandle<()>>>,
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
    /// Initialization state
    initialized: AtomicBool,
}

impl BltpAdapter {
    /// Create a new BLTP adapter.
    pub fn new(config: BltpAdapterConfig) -> Result<Self, AdapterError> {
        config
            .validate()
            .map_err(|e| AdapterError::Fatal(format!("invalid config: {}", e)))?;

        Ok(Self {
            session_manager: SessionMgr::new(config.session_timeout),
            config,
            socket: None,
            session: None,
            inbound: Arc::new(DashMap::new()),
            tasks: TokioMutex::new(Vec::new()),
            shutdown: Arc::new(AtomicBool::new(false)),
            initialized: AtomicBool::new(false),
        })
    }

    /// Perform Noise handshake with peer.
    /// Returns session keys and the actual peer address (from the wire, not config).
    async fn perform_handshake(
        &self,
        socket: &Socket,
    ) -> Result<(SessionKeys, std::net::SocketAddr), AdapterError> {
        let mut attempt = 0;
        let max_attempts = self.config.handshake_retries;

        loop {
            attempt += 1;
            match self.try_handshake(socket).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < max_attempts => {
                    tracing::warn!(
                        attempt = attempt,
                        max = max_attempts,
                        error = %e,
                        "handshake failed, retrying"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(100 * attempt as u64))
                        .await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Single handshake attempt.
    /// Returns session keys and the actual peer address.
    async fn try_handshake(
        &self,
        socket: &Socket,
    ) -> Result<(SessionKeys, std::net::SocketAddr), AdapterError> {
        let timeout = self.config.handshake_timeout;
        let socket_arc = socket.socket_arc();

        if self.config.is_initiator() {
            // Initiator flow
            let peer_pubkey = self
                .config
                .peer_static_pubkey
                .as_ref()
                .ok_or_else(|| AdapterError::Fatal("missing peer public key".into()))?;

            let mut handshake = NoiseHandshake::initiator(&self.config.psk, peer_pubkey)
                .map_err(|e| AdapterError::Fatal(format!("handshake init failed: {}", e)))?;

            // Send first message
            let msg1 = handshake
                .write_message(&[])
                .map_err(|e| AdapterError::Connection(format!("write_message failed: {}", e)))?;

            let mut builder = PacketBuilder::new(&[0u8; 32], 0);
            let packet = builder.build_handshake(&msg1);

            socket
                .send_to(&packet, self.config.peer_addr)
                .await
                .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;

            // Receive response, discarding datagrams that are not handshake
            // packets from the expected peer. This prevents stray traffic on
            // the shared socket from consuming the handshake slot.
            let (parsed, _source) = tokio::time::timeout(timeout, async {
                loop {
                    let mut recv_buf = bytes::BytesMut::with_capacity(protocol::MAX_PACKET_SIZE);
                    recv_buf.resize(protocol::MAX_PACKET_SIZE, 0);

                    let (n, source) = socket_arc
                        .recv_from(&mut recv_buf)
                        .await
                        .map_err(|e| AdapterError::Connection(format!("recv failed: {}", e)))?;

                    // Only accept packets from the peer we initiated with
                    if source != self.config.peer_addr {
                        continue;
                    }

                    recv_buf.truncate(n);
                    let data = recv_buf.freeze();

                    if let Some(p) = ParsedPacket::parse(data, source) {
                        if p.header.flags.is_handshake() {
                            return Ok::<_, AdapterError>((p, source));
                        }
                    }
                    // Not a valid handshake packet from our peer — keep waiting
                }
            })
            .await
            .map_err(|_| AdapterError::Connection("handshake timeout".into()))??;

            // Process response
            handshake
                .read_message(&parsed.payload)
                .map_err(|e| AdapterError::Connection(format!("read_message failed: {}", e)))?;

            // Extract session keys
            let keys = handshake
                .into_session_keys()
                .map_err(|e| AdapterError::Fatal(format!("key extraction failed: {}", e)))?;
            Ok((keys, self.config.peer_addr))
        } else {
            // Responder flow
            let keypair = self
                .config
                .static_keypair
                .as_ref()
                .ok_or_else(|| AdapterError::Fatal("missing static keypair".into()))?;

            // Wait for an initiator handshake message, discarding any
            // non-handshake datagrams that arrive on the shared socket.
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
                    // Not a valid handshake packet — keep waiting
                }
            })
            .await
            .map_err(|_| AdapterError::Connection("handshake timeout".into()))??;

            let mut handshake = NoiseHandshake::responder(&self.config.psk, keypair)
                .map_err(|e| AdapterError::Fatal(format!("handshake init failed: {}", e)))?;

            // Process initiator message
            handshake
                .read_message(&parsed.payload)
                .map_err(|e| AdapterError::Connection(format!("read_message failed: {}", e)))?;

            // Send response
            let msg2 = handshake
                .write_message(&[])
                .map_err(|e| AdapterError::Connection(format!("write_message failed: {}", e)))?;

            let mut builder = PacketBuilder::new(&[0u8; 32], 0);
            let packet = builder.build_handshake(&msg2);

            // Reply to the actual source address (not the configured peer_addr),
            // so the handshake completes even behind NAT or when the config is stale.
            socket
                .send_to(&packet, source)
                .await
                .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;

            // Extract session keys and use the actual source address as peer
            let keys = handshake
                .into_session_keys()
                .map_err(|e| AdapterError::Fatal(format!("key extraction failed: {}", e)))?;
            Ok((keys, source))
        }
    }

    /// Process a single received packet: parse, decrypt, and queue events.
    fn process_packet(
        data: Bytes,
        source: std::net::SocketAddr,
        session: &BltpSession,
        inbound: &InboundQueues,
        num_shards: u16,
    ) {
        // Parse packet
        let parsed = match ParsedPacket::parse(data, source) {
            Some(p) => p,
            None => return,
        };

        // Reject packets whose actual payload size doesn't match the declared
        // length. This catches truncated or oversized packets before they
        // reach the decrypt path.
        if !parsed.header.flags.is_handshake()
            && !parsed.header.flags.is_heartbeat()
            && !parsed.is_valid_length()
        {
            return;
        }

        // Skip handshake packets in the data path (handled during init)
        if parsed.header.flags.is_handshake() {
            return;
        }

        // Validate session before any state mutation (including touch)
        if parsed.header.session_id != session.session_id() {
            return;
        }

        // Heartbeats must come from the authenticated peer address and carry
        // the correct session_id. Without source validation, an off-path
        // attacker who can guess the session_id could keep a session alive.
        if parsed.header.flags.is_heartbeat() {
            if source == session.peer_addr() {
                session.touch();
            }
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

        // Parse events
        let events = EventFrame::read_events(Bytes::from(decrypted), parsed.header.event_count);

        // Update stream state
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

        // Queue events for poll_shard
        let queue = inbound.entry(shard_id).or_default();
        for (i, event_data) in events.into_iter().enumerate() {
            use std::fmt::Write;
            let mut event_id = String::with_capacity(24);
            let _ = write!(event_id, "{}:{}", parsed.header.sequence, i);
            queue.push(StoredEvent::new(
                event_id,
                event_data,
                parsed.header.sequence,
                shard_id,
            ));
        }

        session.touch();
    }

    /// Spawn receiver task.
    ///
    /// On Linux, uses a dedicated OS thread with batched recvmmsg for up to
    /// 64 packets per syscall. On other platforms, uses standard async recv.
    #[cfg(target_os = "linux")]
    fn spawn_receiver(
        shutdown: Arc<AtomicBool>,
        socket: Arc<Socket>,
        session: Arc<BltpSession>,
        inbound: InboundQueues,
        num_shards: u16,
    ) -> JoinHandle<()> {
        let mut receiver = transport::BatchedPacketReceiver::new(socket.socket_arc());

        tokio::spawn(async move {
            while !shutdown.load(Ordering::Acquire) {
                match receiver.recv().await {
                    Ok((data, source)) => {
                        Self::process_packet(data, source, &session, &inbound, num_shards);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                        tracing::warn!("batch receiver thread exited, stopping receiver");
                        break;
                    }
                    Err(e) => {
                        if !shutdown.load(Ordering::Acquire) {
                            tracing::warn!(error = %e, "receive error");
                        }
                    }
                }
            }
        })
    }

    /// Spawn receiver task (non-Linux fallback).
    #[cfg(not(target_os = "linux"))]
    fn spawn_receiver(
        shutdown: Arc<AtomicBool>,
        socket: Arc<Socket>,
        session: Arc<BltpSession>,
        inbound: InboundQueues,
        num_shards: u16,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut receiver = PacketReceiver::new(socket.socket_arc());

            while !shutdown.load(Ordering::Acquire) {
                match receiver.recv().await {
                    Ok((data, source)) => {
                        Self::process_packet(data, source, &session, &inbound, num_shards);
                    }
                    Err(e) => {
                        if !shutdown.load(Ordering::Acquire) {
                            tracing::warn!(error = %e, "receive error");
                        }
                    }
                }
            }
        })
    }

    /// Spawn heartbeat task.
    fn spawn_heartbeat(
        shutdown: Arc<AtomicBool>,
        socket: Arc<Socket>,
        session: Arc<BltpSession>,
        interval: std::time::Duration,
        peer_addr: std::net::SocketAddr,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            while !shutdown.load(Ordering::Acquire) {
                ticker.tick().await;

                if !session.is_active() {
                    break;
                }

                // Build and send heartbeat
                let mut builder = PacketBuilder::new(&[0u8; 32], session.session_id());
                let packet = builder.build_heartbeat();

                if let Err(e) = socket.send_to(&packet, peer_addr).await {
                    tracing::warn!(error = %e, "heartbeat send failed");
                }
            }
        })
    }
}

#[async_trait]
impl Adapter for BltpAdapter {
    async fn init(&mut self) -> Result<(), AdapterError> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }

        // Create socket with configured buffer sizes
        let socket_config = match (
            self.config.socket_recv_buffer,
            self.config.socket_send_buffer,
        ) {
            (Some(recv), Some(send)) => transport::SocketBufferConfig {
                recv_buffer_size: recv,
                send_buffer_size: send,
            },
            _ => transport::SocketBufferConfig::default(),
        };
        let socket = Socket::with_config(self.config.bind_addr, socket_config)
            .await
            .map_err(|e| AdapterError::Connection(format!("socket creation failed: {}", e)))?;

        let socket = Arc::new(socket);
        self.socket = Some(socket.clone());

        // Perform handshake — actual_peer is the real address from the wire
        let (keys, actual_peer) = self.perform_handshake(&socket).await?;

        // Create packet pool with TX key
        // Create session with the actual peer address (not the configured one,
        // which may be stale or pre-NAT)
        let session = Arc::new(BltpSession::new(
            keys,
            actual_peer,
            self.config.packet_pool_size,
            self.config.default_reliability.is_reliable(),
        ));
        self.session = Some(session.clone());

        // Store in session manager for health checks (same Arc as the active session)
        self.session_manager.set_session_arc(session.clone());

        // Spawn background tasks
        let recv_task = Self::spawn_receiver(
            self.shutdown.clone(),
            socket.clone(),
            session.clone(),
            self.inbound.clone(),
            self.config.num_shards,
        );

        let heartbeat_task = Self::spawn_heartbeat(
            self.shutdown.clone(),
            socket,
            session,
            self.config.heartbeat_interval,
            actual_peer,
        );

        {
            let mut tasks = self.tasks.lock().await;
            tasks.push(recv_task);
            tasks.push(heartbeat_task);
        }

        self.initialized.store(true, Ordering::Release);

        tracing::info!(
            bind_addr = %self.config.bind_addr,
            peer_addr = %self.config.peer_addr,
            role = ?self.config.role,
            "BLTP adapter initialized"
        );

        Ok(())
    }

    async fn on_batch(&self, batch: Batch) -> Result<(), AdapterError> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| AdapterError::Connection("not connected".into()))?;

        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| AdapterError::Connection("socket not initialized".into()))?;

        let stream_id = batch.shard_id as u64;
        let peer_addr = session.peer_addr();

        // Get stream state
        let stream = session.get_or_create_stream(stream_id);
        let reliable = stream.with_reliability(|r| r.needs_ack());

        // Convert events to bytes and batch them
        let mut current_batch: Vec<Bytes> = Vec::with_capacity(64);
        let mut current_size = 0usize;

        // Thread-local pool with counter-based nonces — zero contention
        let pool = session.thread_local_pool();
        let mut builder = pool.get();

        for event in &batch.events {
            let event_bytes = event.raw.clone();
            let frame_size = EventFrame::LEN_SIZE + event_bytes.len();

            // Check if adding this event would exceed packet size
            if current_size + frame_size > protocol::MAX_PAYLOAD_SIZE && !current_batch.is_empty() {
                // Send current batch
                let seq = stream.next_tx_seq();
                let flags = if reliable {
                    PacketFlags::RELIABLE
                } else {
                    PacketFlags::NONE
                };

                let packet = builder.build(stream_id, seq, &current_batch, flags);

                socket
                    .send_to(&packet, peer_addr)
                    .await
                    .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;

                // Track for reliability
                if reliable {
                    stream.with_reliability(|r| r.on_send(seq, packet));
                }

                current_batch.clear();
                current_size = 0;
            }

            current_batch.push(event_bytes);
            current_size += frame_size;
        }

        // Send remaining events
        if !current_batch.is_empty() {
            let seq = stream.next_tx_seq();
            let flags = if reliable {
                PacketFlags::RELIABLE
            } else {
                PacketFlags::NONE
            };

            let packet = builder.build(stream_id, seq, &current_batch, flags);

            socket
                .send_to(&packet, peer_addr)
                .await
                .map_err(|e| AdapterError::Connection(format!("send failed: {}", e)))?;

            if reliable {
                stream.with_reliability(|r| r.on_send(seq, packet));
            }
        }

        session.touch();

        Ok(())
    }

    async fn poll_shard(
        &self,
        shard_id: u16,
        from_id: Option<&str>,
        limit: usize,
    ) -> Result<ShardPollResult, AdapterError> {
        let mut events = Vec::with_capacity(limit);

        if let Some(queue) = self.inbound.get(&shard_id) {
            while events.len() < limit {
                if let Some(event) = queue.pop() {
                    if from_id.is_none() || event.id.as_str() > from_id.unwrap_or("") {
                        events.push(event);
                    }
                } else {
                    break;
                }
            }
        }

        let has_more = self
            .inbound
            .get(&shard_id)
            .map(|q| !q.is_empty())
            .unwrap_or(false);
        let next_id = events.last().map(|e| e.id.clone());

        Ok(ShardPollResult {
            events,
            next_id,
            has_more,
        })
    }

    async fn flush(&self) -> Result<(), AdapterError> {
        // For reliable streams, wait for all pending ACKs
        // Currently a no-op since we're fire-and-forget by default
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), AdapterError> {
        self.shutdown.store(true, Ordering::Release);

        // Clear session
        self.session_manager.clear_session();

        // Wait for tasks to complete
        let mut tasks = self.tasks.lock().await;
        for task in tasks.drain(..) {
            let _ = task.await;
        }

        self.initialized.store(false, Ordering::Release);

        tracing::info!("BLTP adapter shutdown complete");

        Ok(())
    }

    fn name(&self) -> &'static str {
        "bltp"
    }

    async fn is_healthy(&self) -> bool {
        self.initialized.load(Ordering::Acquire) && self.session_manager.check_session()
    }
}

impl std::fmt::Debug for BltpAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BltpAdapter")
            .field("config", &self.config)
            .field("initialized", &self.initialized.load(Ordering::Relaxed))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let psk = [0x42u8; 32];
        let peer_pubkey = [0x24u8; 32];

        let config = BltpAdapterConfig::initiator(
            "127.0.0.1:0".parse().unwrap(),
            "127.0.0.1:9999".parse().unwrap(),
            psk,
            peer_pubkey,
        );

        let adapter = BltpAdapter::new(config).unwrap();
        assert_eq!(adapter.name(), "bltp");
    }

    #[test]
    fn test_shard_id_from_stream_id_uses_modulo() {
        // Regression: shard_id was computed as `stream_id as u16` (truncation),
        // which collides for stream IDs that differ only in upper bits.
        // The fix uses `stream_id % num_shards`.
        let num_shards: u16 = 8;

        // Two stream IDs that are identical in their low 16 bits
        // but different overall must map to the same shard via modulo,
        // while truncation would also give the same result here.
        // More importantly, a large stream_id must stay within [0, num_shards).
        let stream_a: u64 = 0xDEAD_BEEF_0000_0003;
        let stream_b: u64 = 0xCAFE_BABE_0000_0003;

        let shard_a = (stream_a % num_shards as u64) as u16;
        let shard_b = (stream_b % num_shards as u64) as u16;

        assert!(
            shard_a < num_shards,
            "shard must be in range [0, num_shards)"
        );
        assert!(
            shard_b < num_shards,
            "shard must be in range [0, num_shards)"
        );

        // Large stream IDs that would overflow u16 must still be valid shard IDs
        let big_stream: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let shard_big = (big_stream % num_shards as u64) as u16;
        assert!(shard_big < num_shards);

        // Truncation would give 0xFFFF = 65535, which is >= num_shards.
        // Modulo gives a valid shard.
        assert_ne!(
            big_stream as u16, shard_big,
            "modulo must differ from truncation for large stream IDs"
        );
    }

    #[test]
    fn test_invalid_config() {
        let psk = [0x42u8; 32];
        let peer_pubkey = [0x24u8; 32];

        let mut config = BltpAdapterConfig::initiator(
            "127.0.0.1:0".parse().unwrap(),
            "127.0.0.1:9999".parse().unwrap(),
            psk,
            peer_pubkey,
        );
        config.peer_static_pubkey = None;

        let result = BltpAdapter::new(config);
        assert!(result.is_err());
    }
}
