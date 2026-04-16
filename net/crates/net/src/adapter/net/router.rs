//! Net Router for single-hop and multi-hop packet routing.
//!
//! The router handles:
//! - Stream multiplexing across thousands of streams
//! - Fair scheduling to prevent stream starvation
//! - Low-latency packet forwarding
//! - Per-stream statistics

use bytes::{Bytes, BytesMut};
use crossbeam_queue::ArrayQueue;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Notify;

use super::protocol::HEADER_SIZE;
use super::route::{RoutingHeader, RoutingTable, ROUTING_HEADER_SIZE};

/// Router configuration
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Local node ID
    pub local_id: u64,
    /// Bind address
    pub bind_addr: SocketAddr,
    /// Maximum queue depth per stream
    pub max_queue_depth: usize,
    /// Fair scheduling quantum (packets per stream per round)
    pub fair_quantum: usize,
    /// Idle stream timeout (nanoseconds)
    pub idle_timeout_ns: u64,
    /// Enable priority queue bypass
    pub priority_bypass: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            local_id: 0,
            bind_addr: "0.0.0.0:0".parse().unwrap(),
            max_queue_depth: 1024,
            fair_quantum: 16,
            idle_timeout_ns: 30_000_000_000, // 30 seconds
            priority_bypass: true,
        }
    }
}

impl RouterConfig {
    /// Create a new router config with defaults
    pub fn new(local_id: u64, bind_addr: SocketAddr) -> Self {
        Self {
            local_id,
            bind_addr,
            ..Default::default()
        }
    }
}

/// Queued packet for fair scheduling
pub struct QueuedPacket {
    /// Packet data
    pub data: Bytes,
    /// Destination address
    pub dest: SocketAddr,
    /// Stream identifier
    pub stream_id: u64,
    /// Whether this is a priority packet
    pub priority: bool,
    /// Time the packet was queued
    pub queued_at: Instant,
}

/// Per-stream queue for fair scheduling
struct StreamQueue {
    queue: ArrayQueue<QueuedPacket>,
    packets_sent_this_round: AtomicU64,
}

impl StreamQueue {
    fn new(capacity: usize) -> Self {
        Self {
            queue: ArrayQueue::new(capacity),
            packets_sent_this_round: AtomicU64::new(0),
        }
    }

    fn push(&self, packet: QueuedPacket) -> Result<(), QueuedPacket> {
        self.queue.push(packet)
    }

    fn pop(&self) -> Option<QueuedPacket> {
        self.queue.pop()
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.queue.len()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn reset_round(&self) {
        self.packets_sent_this_round.store(0, Ordering::Relaxed);
    }

    fn increment_sent(&self) -> u64 {
        self.packets_sent_this_round.fetch_add(1, Ordering::Relaxed)
    }

    fn sent_this_round(&self) -> u64 {
        self.packets_sent_this_round.load(Ordering::Relaxed)
    }
}

/// Fair scheduler for stream fairness
pub struct FairScheduler {
    /// Per-stream queues
    streams: DashMap<u64, Arc<StreamQueue>>,
    /// Priority queue (bypasses fair scheduling)
    priority_queue: ArrayQueue<QueuedPacket>,
    /// Fair quantum
    quantum: usize,
    /// Max queue depth per stream
    max_depth: usize,
    /// Notification for new packets
    notify: Notify,
    /// Total packets queued
    total_queued: AtomicU64,
    /// Total packets dropped
    total_dropped: AtomicU64,
    /// Rotation index for round-robin fairness across dequeue calls
    round_robin_idx: AtomicU64,
}

impl FairScheduler {
    /// Create a new fair scheduler
    pub fn new(quantum: usize, max_depth: usize) -> Self {
        Self {
            streams: DashMap::new(),
            priority_queue: ArrayQueue::new(max_depth),
            quantum,
            max_depth,
            notify: Notify::new(),
            total_queued: AtomicU64::new(0),
            total_dropped: AtomicU64::new(0),
            round_robin_idx: AtomicU64::new(0),
        }
    }

    /// Enqueue a packet
    pub fn enqueue(&self, packet: QueuedPacket) -> bool {
        if packet.priority {
            // Priority packets bypass fair scheduling
            if self.priority_queue.push(packet).is_ok() {
                self.total_queued.fetch_add(1, Ordering::Relaxed);
                self.notify.notify_one();
                return true;
            }
        } else {
            // Get or create stream queue
            let queue = self
                .streams
                .entry(packet.stream_id)
                .or_insert_with(|| Arc::new(StreamQueue::new(self.max_depth)))
                .clone();

            if queue.push(packet).is_ok() {
                self.total_queued.fetch_add(1, Ordering::Relaxed);
                self.notify.notify_one();
                return true;
            }
        }

        self.total_dropped.fetch_add(1, Ordering::Relaxed);
        false
    }

    /// Dequeue next packet (fair round-robin)
    pub fn dequeue(&self) -> Option<QueuedPacket> {
        // Priority queue first
        if let Some(packet) = self.priority_queue.pop() {
            return Some(packet);
        }

        // Collect stream keys for stable, rotated iteration order
        let keys: Vec<u64> = self.streams.iter().map(|e| *e.key()).collect();
        if keys.is_empty() {
            return None;
        }
        let start = self.round_robin_idx.fetch_add(1, Ordering::Relaxed) as usize % keys.len();

        // Round-robin across streams, starting from the rotated index
        for i in 0..keys.len() {
            let key = keys[(start + i) % keys.len()];
            if let Some(queue) = self.streams.get(&key) {
                if queue.sent_this_round() < self.quantum as u64 && !queue.is_empty() {
                    if let Some(packet) = queue.pop() {
                        queue.increment_sent();
                        return Some(packet);
                    }
                }
            }
        }

        // If all streams exhausted their quantum, reset and try again.
        // Re-collect keys so that streams added since the first snapshot
        // are also considered — using the stale `keys` vec would miss them.
        let keys: Vec<u64> = self.streams.iter().map(|e| *e.key()).collect();
        if keys.is_empty() {
            return None;
        }
        let mut has_packets = false;
        for key in &keys {
            if let Some(queue) = self.streams.get(key) {
                queue.reset_round();
                if !queue.is_empty() {
                    has_packets = true;
                }
            }
        }

        if has_packets {
            let start = self.round_robin_idx.load(Ordering::Relaxed) as usize % keys.len();
            for i in 0..keys.len() {
                let key = keys[(start + i) % keys.len()];
                if let Some(queue) = self.streams.get(&key) {
                    if let Some(packet) = queue.pop() {
                        queue.increment_sent();
                        return Some(packet);
                    }
                }
            }
        }

        None
    }

    /// Wait for new packets
    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    /// Get total queued count
    pub fn total_queued(&self) -> u64 {
        self.total_queued.load(Ordering::Relaxed)
    }

    /// Get total dropped count
    pub fn total_dropped(&self) -> u64 {
        self.total_dropped.load(Ordering::Relaxed)
    }

    /// Get number of active streams
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }

    /// Clean up empty stream queues
    pub fn cleanup_empty(&self) -> usize {
        let mut removed = 0;
        self.streams.retain(|_, queue| {
            if queue.is_empty() {
                removed += 1;
                false
            } else {
                true
            }
        });
        removed
    }
}

/// Router statistics
#[derive(Debug, Clone, Default)]
pub struct RouterStats {
    /// Packets received
    pub packets_received: u64,
    /// Packets forwarded
    pub packets_forwarded: u64,
    /// Packets delivered locally
    pub packets_local: u64,
    /// Packets dropped (TTL, no route, queue full)
    pub packets_dropped: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes forwarded
    pub bytes_forwarded: u64,
    /// Active routes
    pub routes: usize,
    /// Active streams
    pub streams: usize,
    /// Average routing latency (nanoseconds)
    pub avg_latency_ns: u64,
}

/// Net Router
pub struct NetRouter {
    /// Configuration
    #[allow(dead_code)]
    config: RouterConfig,
    /// UDP socket
    socket: Arc<UdpSocket>,
    /// Routing table
    routing_table: Arc<RoutingTable>,
    /// Fair scheduler
    scheduler: Arc<FairScheduler>,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Statistics
    packets_received: AtomicU64,
    packets_forwarded: AtomicU64,
    packets_local: AtomicU64,
    packets_dropped: AtomicU64,
    bytes_received: AtomicU64,
    bytes_forwarded: AtomicU64,
    total_latency_ns: AtomicU64,
    latency_samples: AtomicU64,
}

impl NetRouter {
    /// Create a new router
    pub async fn new(config: RouterConfig) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(config.bind_addr).await?;
        let routing_table = Arc::new(RoutingTable::new(config.local_id));
        let scheduler = Arc::new(FairScheduler::new(
            config.fair_quantum,
            config.max_queue_depth,
        ));

        Ok(Self {
            config,
            socket: Arc::new(socket),
            routing_table,
            scheduler,
            running: Arc::new(AtomicBool::new(false)),
            packets_received: AtomicU64::new(0),
            packets_forwarded: AtomicU64::new(0),
            packets_local: AtomicU64::new(0),
            packets_dropped: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            bytes_forwarded: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
            latency_samples: AtomicU64::new(0),
        })
    }

    /// Get local address
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Get routing table
    pub fn routing_table(&self) -> &Arc<RoutingTable> {
        &self.routing_table
    }

    /// Add a route
    pub fn add_route(&self, dest_id: u64, next_hop: SocketAddr) {
        self.routing_table.add_route(dest_id, next_hop);
    }

    /// Remove a route
    pub fn remove_route(&self, dest_id: u64) {
        self.routing_table.remove_route(dest_id);
    }

    /// Route a packet (called from receive loop)
    pub fn route_packet(&self, data: Bytes, _from: SocketAddr) -> Result<RouteAction, RouterError> {
        let start = Instant::now();
        let len = data.len() as u64;

        self.packets_received.fetch_add(1, Ordering::Relaxed);
        self.bytes_received.fetch_add(len, Ordering::Relaxed);

        // Need at least routing header
        if data.len() < ROUTING_HEADER_SIZE {
            self.packets_dropped.fetch_add(1, Ordering::Relaxed);
            return Err(RouterError::PacketTooSmall);
        }

        // Parse routing header
        let routing_header = RoutingHeader::from_bytes(&data[..ROUTING_HEADER_SIZE])
            .ok_or(RouterError::InvalidHeader)?;

        // Extract stream ID from Net header if present
        let stream_id = if data.len() >= ROUTING_HEADER_SIZE + HEADER_SIZE {
            let net_header = &data[ROUTING_HEADER_SIZE..ROUTING_HEADER_SIZE + HEADER_SIZE];
            u64::from_le_bytes(net_header[32..40].try_into().unwrap_or([0; 8]))
        } else {
            0
        };

        // Record stats
        self.routing_table.record_in(stream_id, len);

        // Check if local delivery
        if self.routing_table.is_local(routing_header.dest_id) {
            self.packets_local.fetch_add(1, Ordering::Relaxed);
            self.record_latency(start);
            return Ok(RouteAction::Local(data.slice(ROUTING_HEADER_SIZE..)));
        }

        // Check TTL
        if routing_header.is_expired() {
            self.packets_dropped.fetch_add(1, Ordering::Relaxed);
            self.routing_table.record_drop(stream_id);
            return Err(RouterError::TtlExpired);
        }

        // Lookup next hop
        let next_hop = self
            .routing_table
            .lookup(routing_header.dest_id)
            .ok_or(RouterError::NoRoute)?;

        // Update header for forwarding
        let mut new_data = BytesMut::with_capacity(data.len());
        let mut fwd_header = routing_header;
        fwd_header.forward();
        fwd_header.write_to(&mut new_data);
        new_data.extend_from_slice(&data[ROUTING_HEADER_SIZE..]);

        // Queue for sending
        let packet = QueuedPacket {
            data: new_data.freeze(),
            dest: next_hop,
            stream_id,
            priority: routing_header.flags.is_priority(),
            queued_at: Instant::now(),
        };

        if self.scheduler.enqueue(packet) {
            self.packets_forwarded.fetch_add(1, Ordering::Relaxed);
            self.bytes_forwarded.fetch_add(len, Ordering::Relaxed);
            self.routing_table.record_out(stream_id, len);
            self.record_latency(start);
            Ok(RouteAction::Forwarded(next_hop))
        } else {
            self.packets_dropped.fetch_add(1, Ordering::Relaxed);
            self.routing_table.record_drop(stream_id);
            Err(RouterError::QueueFull)
        }
    }

    /// Send a packet directly (bypassing routing)
    pub async fn send_to(&self, data: &[u8], dest: SocketAddr) -> std::io::Result<usize> {
        self.socket.send_to(data, dest).await
    }

    /// Receive a packet
    pub async fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf).await
    }

    /// Start the router (spawns send loop)
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        self.running.store(true, Ordering::Release);

        let socket = self.socket.clone();
        let scheduler = self.scheduler.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while running.load(Ordering::Acquire) {
                // Dequeue and send
                if let Some(packet) = scheduler.dequeue() {
                    let _ = socket.send_to(&packet.data, packet.dest).await;
                } else {
                    // Wait for new packets (with timeout)
                    tokio::select! {
                        _ = scheduler.wait() => {}
                        _ = tokio::time::sleep(Duration::from_millis(1)) => {}
                    }
                }
            }
        })
    }

    /// Stop the router
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    /// Get statistics
    pub fn stats(&self) -> RouterStats {
        let samples = self.latency_samples.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ns.load(Ordering::Relaxed);
        let avg_latency = total_latency.checked_div(samples).unwrap_or(0);

        RouterStats {
            packets_received: self.packets_received.load(Ordering::Relaxed),
            packets_forwarded: self.packets_forwarded.load(Ordering::Relaxed),
            packets_local: self.packets_local.load(Ordering::Relaxed),
            packets_dropped: self.packets_dropped.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            bytes_forwarded: self.bytes_forwarded.load(Ordering::Relaxed),
            routes: self.routing_table.route_count(),
            streams: self.routing_table.stream_count(),
            avg_latency_ns: avg_latency,
        }
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.packets_received.store(0, Ordering::Relaxed);
        self.packets_forwarded.store(0, Ordering::Relaxed);
        self.packets_local.store(0, Ordering::Relaxed);
        self.packets_dropped.store(0, Ordering::Relaxed);
        self.bytes_received.store(0, Ordering::Relaxed);
        self.bytes_forwarded.store(0, Ordering::Relaxed);
        self.total_latency_ns.store(0, Ordering::Relaxed);
        self.latency_samples.store(0, Ordering::Relaxed);
    }

    fn record_latency(&self, start: Instant) {
        let latency = start.elapsed().as_nanos() as u64;
        self.total_latency_ns.fetch_add(latency, Ordering::Relaxed);
        self.latency_samples.fetch_add(1, Ordering::Relaxed);
    }
}

/// Result of routing a packet
#[derive(Debug)]
pub enum RouteAction {
    /// Packet is for local delivery
    Local(Bytes),
    /// Packet was forwarded to next hop
    Forwarded(SocketAddr),
}

/// Router errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterError {
    /// Packet too small
    PacketTooSmall,
    /// Invalid routing header
    InvalidHeader,
    /// TTL expired
    TtlExpired,
    /// No route to destination
    NoRoute,
    /// Queue full
    QueueFull,
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PacketTooSmall => write!(f, "packet too small"),
            Self::InvalidHeader => write!(f, "invalid routing header"),
            Self::TtlExpired => write!(f, "TTL expired"),
            Self::NoRoute => write!(f, "no route to destination"),
            Self::QueueFull => write!(f, "queue full"),
        }
    }
}

impl std::error::Error for RouterError {}

#[cfg(test)]
mod tests {
    use super::super::protocol::NetHeader;
    use super::*;

    #[test]
    fn test_fair_scheduler_basic() {
        let scheduler = FairScheduler::new(2, 16);

        // Enqueue packets from different streams
        for stream in 0..3 {
            for _ in 0..4 {
                let packet = QueuedPacket {
                    data: Bytes::from(vec![0u8; 64]),
                    dest: "127.0.0.1:9000".parse().unwrap(),
                    stream_id: stream,
                    priority: false,
                    queued_at: Instant::now(),
                };
                assert!(scheduler.enqueue(packet));
            }
        }

        assert_eq!(scheduler.stream_count(), 3);
        assert_eq!(scheduler.total_queued(), 12);

        // Dequeue should round-robin
        let mut stream_order = Vec::new();
        while let Some(packet) = scheduler.dequeue() {
            stream_order.push(packet.stream_id);
        }

        // Should have processed all 12 packets
        assert_eq!(stream_order.len(), 12);

        // Check fairness: each stream should get ~4 packets
        let mut counts = [0; 3];
        for stream in stream_order {
            counts[stream as usize] += 1;
        }
        assert_eq!(counts, [4, 4, 4]);
    }

    #[test]
    fn test_fair_scheduler_priority() {
        let scheduler = FairScheduler::new(2, 16);

        // Enqueue normal packets
        for _ in 0..4 {
            let packet = QueuedPacket {
                data: Bytes::from(vec![0u8; 64]),
                dest: "127.0.0.1:9000".parse().unwrap(),
                stream_id: 0,
                priority: false,
                queued_at: Instant::now(),
            };
            scheduler.enqueue(packet);
        }

        // Enqueue priority packet
        let priority = QueuedPacket {
            data: Bytes::from(vec![1u8; 64]),
            dest: "127.0.0.1:9000".parse().unwrap(),
            stream_id: 1,
            priority: true,
            queued_at: Instant::now(),
        };
        scheduler.enqueue(priority);

        // Priority should come first
        let first = scheduler.dequeue().unwrap();
        assert_eq!(first.data[0], 1);
        assert!(first.priority);
    }

    #[test]
    fn test_fair_scheduler_no_starvation() {
        // Regression: dequeue() always started iterating from the beginning
        // of the DashMap, so streams appearing earlier in iteration order
        // were systematically preferred, starving later streams.
        //
        // With the rotation fix, each dequeue() starts from a different
        // position, so all streams should get roughly equal service.
        let quantum = 1;
        let scheduler = FairScheduler::new(quantum, 64);

        // Use many streams to make DashMap iteration-order bias visible
        let num_streams = 8u64;
        let packets_per_stream = 20;

        for stream in 0..num_streams {
            for _ in 0..packets_per_stream {
                let packet = QueuedPacket {
                    data: Bytes::from(vec![stream as u8; 1]),
                    dest: "127.0.0.1:9000".parse().unwrap(),
                    stream_id: stream,
                    priority: false,
                    queued_at: Instant::now(),
                };
                scheduler.enqueue(packet);
            }
        }

        // Dequeue all packets and track which stream each came from
        let mut first_half_counts = vec![0u32; num_streams as usize];
        let total = num_streams * packets_per_stream as u64;
        let half = total / 2;

        for i in 0..total {
            let packet = scheduler.dequeue().unwrap();
            if i < half {
                first_half_counts[packet.stream_id as usize] += 1;
            }
        }

        // In the first half of dequeues, every stream should have been served
        // at least once. Without rotation, some streams could get 0 service.
        for (stream, &count) in first_half_counts.iter().enumerate() {
            assert!(
                count > 0,
                "stream {} was starved in the first half of dequeues ({} of {} packets)",
                stream,
                count,
                half
            );
        }
    }

    #[tokio::test]
    async fn test_router_creation() {
        let config = RouterConfig::new(0x1234, "127.0.0.1:0".parse().unwrap());
        let router = NetRouter::new(config).await.unwrap();

        assert!(!router.is_running());
        assert_eq!(router.stats().routes, 0);
    }

    #[tokio::test]
    async fn test_router_routing_table() {
        let config = RouterConfig::new(0x1234, "127.0.0.1:0".parse().unwrap());
        let router = NetRouter::new(config).await.unwrap();

        let dest: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        router.add_route(0x5678, dest);

        assert_eq!(router.routing_table().lookup(0x5678), Some(dest));
        assert_eq!(router.stats().routes, 1);
    }

    #[tokio::test]
    async fn test_router_extracts_stream_id_at_correct_offset() {
        // Regression: stream_id was read from bytes 8..16 (inside the nonce)
        // instead of bytes 40..48 where it actually lives in the Net header.
        let local_id = 0x1234u64;
        let config = RouterConfig::new(local_id, "127.0.0.1:0".parse().unwrap());
        let router = NetRouter::new(config).await.unwrap();

        let expected_stream_id: u64 = 0xDEAD_BEEF_CAFE_BABEu64;

        // Build routing header pointing to local_id so we get RouteAction::Local
        let routing = RoutingHeader::new(local_id, 0x5678, 4);
        let routing_bytes = routing.to_bytes();

        // Build a Net header with a known stream_id
        let net = NetHeader::new(
            0xAAAA,             // session_id
            expected_stream_id, // stream_id
            1,                  // sequence
            [0u8; 12],          // nonce
            0,                  // payload_len
            0,                  // event_count
            super::super::protocol::PacketFlags::NONE,
        );
        let net_bytes = net.to_bytes();

        // Concatenate routing header + Net header
        let mut packet = BytesMut::with_capacity(ROUTING_HEADER_SIZE + HEADER_SIZE);
        packet.extend_from_slice(&routing_bytes);
        packet.extend_from_slice(&net_bytes);

        let from: SocketAddr = "127.0.0.1:5000".parse().unwrap();
        let _ = router.route_packet(packet.freeze(), from);

        // The stream stats should be keyed by the correct stream_id
        let stats = router.routing_table().get_stream_stats(expected_stream_id);
        assert_eq!(
            stats.get_packets_in(),
            1,
            "stream stats should record 1 packet for stream_id 0x{:X}",
            expected_stream_id
        );
    }

    #[test]
    fn test_regression_scheduler_sees_streams_added_after_quantum_exhaustion() {
        // Regression: dequeue() collected stream keys once, then reused
        // the stale snapshot for the retry loop after quantum reset.
        // Streams added between the two loops were invisible until the
        // next dequeue() call, causing extra latency.
        //
        // Fix: re-collect keys before the retry loop.
        let scheduler = FairScheduler::new(1, 16);

        // Stream 0 with 1 packet (quantum = 1, so first pass drains it)
        scheduler.enqueue(QueuedPacket {
            data: Bytes::from_static(b"s0"),
            dest: "127.0.0.1:9000".parse().unwrap(),
            stream_id: 0,
            priority: false,
            queued_at: Instant::now(),
        });

        // Drain stream 0's quantum
        let pkt = scheduler.dequeue().unwrap();
        assert_eq!(pkt.stream_id, 0);

        // Now add stream 1 while the scheduler is "between rounds"
        scheduler.enqueue(QueuedPacket {
            data: Bytes::from_static(b"s1"),
            dest: "127.0.0.1:9000".parse().unwrap(),
            stream_id: 1,
            priority: false,
            queued_at: Instant::now(),
        });

        // Next dequeue should find stream 1
        let pkt = scheduler.dequeue().unwrap();
        assert_eq!(
            pkt.stream_id, 1,
            "newly added stream should be visible after quantum reset"
        );
    }
}
