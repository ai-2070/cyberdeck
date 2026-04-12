//! Routing primitives for BLTP multi-hop transport.
//!
//! This module provides:
//! - `RoutingHeader`: Fixed-size header for multi-hop routing
//! - `RoutingTable`: Stream-to-destination mapping
//! - `StreamStats`: Per-stream statistics for fairness monitoring

use bytes::{Buf, BufMut, Bytes, BytesMut};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Routing header size in bytes (16 bytes, cache-line friendly)
pub const ROUTING_HEADER_SIZE: usize = 16;

/// Maximum TTL for multi-hop routing
pub const _MAX_TTL: u8 = 16;

/// Route flags (bitflags — multiple flags can be set simultaneously)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct RouteFlags(u8);

impl RouteFlags {
    /// No special flags
    pub const NONE: Self = Self(0x00);
    /// Control packet (pingwave, capability update)
    pub const CONTROL: Self = Self(0x01);
    /// Requires acknowledgment
    pub const REQUIRES_ACK: Self = Self(0x02);
    /// Priority packet (skip fairness queue)
    pub const PRIORITY: Self = Self(0x04);
    /// Last packet in stream
    pub const END_OF_STREAM: Self = Self(0x08);

    /// Parse flags from u8 (preserves all set bits in the lower nibble)
    pub fn from_u8(v: u8) -> Self {
        Self(v & 0x0F)
    }

    /// Convert to u8
    pub fn as_u8(self) -> u8 {
        self.0
    }

    /// Check if a flag is set
    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Check if this is a control packet
    pub fn is_control(self) -> bool {
        self.contains(Self::CONTROL)
    }

    /// Check if this is a priority packet
    pub fn is_priority(self) -> bool {
        self.contains(Self::PRIORITY)
    }
}

/// Routing header for multi-hop BLTP packets.
///
/// Layout (16 bytes):
/// ```text
/// ┌────────────────────────────────────────────────────────┐
/// │ dest_id (8 bytes) │ src_id (4 bytes) │ ttl │ hops │ flags │ reserved │
/// └────────────────────────────────────────────────────────┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct RoutingHeader {
    /// Final destination node ID (64-bit)
    pub dest_id: u64,
    /// Source node ID (truncated to 32-bit for space)
    pub src_id: u32,
    /// Time-to-live (decremented at each hop)
    pub ttl: u8,
    /// Hop count so far
    pub hop_count: u8,
    /// Route flags
    pub flags: RouteFlags,
    /// Reserved for future use
    pub _reserved: u8,
}

impl RoutingHeader {
    /// Create a new routing header
    pub fn new(dest_id: u64, src_id: u32, ttl: u8) -> Self {
        Self {
            dest_id,
            src_id,
            ttl,
            hop_count: 0,
            flags: RouteFlags::NONE,
            _reserved: 0,
        }
    }

    /// Create a control packet header
    pub fn control(dest_id: u64, src_id: u32, ttl: u8) -> Self {
        Self {
            dest_id,
            src_id,
            ttl,
            hop_count: 0,
            flags: RouteFlags::CONTROL,
            _reserved: 0,
        }
    }

    /// Create a priority packet header
    pub fn priority(dest_id: u64, src_id: u32, ttl: u8) -> Self {
        Self {
            dest_id,
            src_id,
            ttl,
            hop_count: 0,
            flags: RouteFlags::PRIORITY,
            _reserved: 0,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; ROUTING_HEADER_SIZE] {
        let mut buf = [0u8; ROUTING_HEADER_SIZE];
        buf[0..8].copy_from_slice(&self.dest_id.to_le_bytes());
        buf[8..12].copy_from_slice(&self.src_id.to_le_bytes());
        buf[12] = self.ttl;
        buf[13] = self.hop_count;
        buf[14] = self.flags.as_u8();
        buf[15] = self._reserved;
        buf
    }

    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < ROUTING_HEADER_SIZE {
            return None;
        }
        Some(Self {
            dest_id: u64::from_le_bytes(buf[0..8].try_into().ok()?),
            src_id: u32::from_le_bytes(buf[8..12].try_into().ok()?),
            ttl: buf[12],
            hop_count: buf[13],
            flags: RouteFlags::from_u8(buf[14]),
            _reserved: buf[15],
        })
    }

    /// Write to a buffer
    pub fn write_to(&self, buf: &mut BytesMut) {
        buf.put_u64_le(self.dest_id);
        buf.put_u32_le(self.src_id);
        buf.put_u8(self.ttl);
        buf.put_u8(self.hop_count);
        buf.put_u8(self.flags.as_u8());
        buf.put_u8(self._reserved);
    }

    /// Read from a buffer
    pub fn read_from(buf: &mut Bytes) -> Option<Self> {
        if buf.remaining() < ROUTING_HEADER_SIZE {
            return None;
        }
        Some(Self {
            dest_id: buf.get_u64_le(),
            src_id: buf.get_u32_le(),
            ttl: buf.get_u8(),
            hop_count: buf.get_u8(),
            flags: RouteFlags::from_u8(buf.get_u8()),
            _reserved: buf.get_u8(),
        })
    }

    /// Check if TTL is expired
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.ttl == 0
    }

    /// Decrement TTL and increment hop count (for forwarding)
    #[inline]
    pub fn forward(&mut self) -> bool {
        if self.ttl == 0 {
            return false;
        }
        self.ttl -= 1;
        self.hop_count = self.hop_count.saturating_add(1);
        true
    }
}

/// Per-stream statistics for fairness monitoring
#[derive(Debug)]
pub struct StreamStats {
    /// Packets received
    pub packets_in: AtomicU64,
    /// Packets forwarded
    pub packets_out: AtomicU64,
    /// Packets dropped (fairness, TTL, etc.)
    pub packets_dropped: AtomicU64,
    /// Bytes received
    pub bytes_in: AtomicU64,
    /// Bytes forwarded
    pub bytes_out: AtomicU64,
    /// Last activity timestamp (for idle detection)
    last_activity: AtomicU64,
}

impl StreamStats {
    /// Create new stream stats
    pub fn new() -> Self {
        Self {
            packets_in: AtomicU64::new(0),
            packets_out: AtomicU64::new(0),
            packets_dropped: AtomicU64::new(0),
            bytes_in: AtomicU64::new(0),
            bytes_out: AtomicU64::new(0),
            last_activity: AtomicU64::new(Self::now_nanos()),
        }
    }

    /// Record incoming packet
    #[inline]
    pub fn record_in(&self, bytes: u64) {
        self.packets_in.fetch_add(1, Ordering::Relaxed);
        self.bytes_in.fetch_add(bytes, Ordering::Relaxed);
        self.last_activity
            .store(Self::now_nanos(), Ordering::Relaxed);
    }

    /// Record outgoing packet
    #[inline]
    pub fn record_out(&self, bytes: u64) {
        self.packets_out.fetch_add(1, Ordering::Relaxed);
        self.bytes_out.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record dropped packet
    #[inline]
    pub fn record_drop(&self) {
        self.packets_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Get packets in count
    #[inline]
    pub fn get_packets_in(&self) -> u64 {
        self.packets_in.load(Ordering::Relaxed)
    }

    /// Get packets out count
    #[inline]
    pub fn get_packets_out(&self) -> u64 {
        self.packets_out.load(Ordering::Relaxed)
    }

    /// Get drop count
    #[inline]
    pub fn get_drops(&self) -> u64 {
        self.packets_dropped.load(Ordering::Relaxed)
    }

    /// Check if stream is idle (no activity for given duration)
    pub fn is_idle(&self, idle_nanos: u64) -> bool {
        let last = self.last_activity.load(Ordering::Relaxed);
        Self::now_nanos().saturating_sub(last) > idle_nanos
    }

    #[inline]
    fn now_nanos() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

impl Default for StreamStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Route entry in the routing table
#[derive(Debug, Clone)]
pub struct RouteEntry {
    /// Next hop address
    pub next_hop: SocketAddr,
    /// Metric (lower is better)
    pub metric: u16,
    /// Route is active
    pub active: bool,
    /// Last update timestamp
    pub updated_at: Instant,
}

impl RouteEntry {
    /// Create a new route entry with default metric
    pub fn new(next_hop: SocketAddr) -> Self {
        Self {
            next_hop,
            metric: 1,
            active: true,
            updated_at: Instant::now(),
        }
    }

    /// Create a route entry with specified metric
    pub fn with_metric(next_hop: SocketAddr, metric: u16) -> Self {
        Self {
            next_hop,
            metric,
            active: true,
            updated_at: Instant::now(),
        }
    }
}

/// Routing table for stream-to-destination mapping
pub struct RoutingTable {
    /// Node ID -> next hop address
    routes: DashMap<u64, RouteEntry>,
    /// Stream ID -> per-stream stats
    stream_stats: DashMap<u64, StreamStats>,
    /// Local node ID
    local_id: u64,
}

impl RoutingTable {
    /// Create a new routing table
    pub fn new(local_id: u64) -> Self {
        Self {
            routes: DashMap::new(),
            stream_stats: DashMap::new(),
            local_id,
        }
    }

    /// Get local node ID
    #[inline]
    pub fn local_id(&self) -> u64 {
        self.local_id
    }

    /// Add or update a route
    pub fn add_route(&self, dest_id: u64, next_hop: SocketAddr) {
        self.routes.insert(dest_id, RouteEntry::new(next_hop));
    }

    /// Add route with metric
    pub fn add_route_with_metric(&self, dest_id: u64, next_hop: SocketAddr, metric: u16) {
        self.routes
            .insert(dest_id, RouteEntry::with_metric(next_hop, metric));
    }

    /// Remove a route
    pub fn remove_route(&self, dest_id: u64) -> Option<RouteEntry> {
        self.routes.remove(&dest_id).map(|(_, v)| v)
    }

    /// Look up next hop for destination
    pub fn lookup(&self, dest_id: u64) -> Option<SocketAddr> {
        self.routes
            .get(&dest_id)
            .filter(|r| r.active)
            .map(|r| r.next_hop)
    }

    /// Check if destination is local
    #[inline]
    pub fn is_local(&self, dest_id: u64) -> bool {
        dest_id == self.local_id
    }

    /// Get or create stream stats
    pub fn get_stream_stats(
        &self,
        stream_id: u64,
    ) -> dashmap::mapref::one::Ref<'_, u64, StreamStats> {
        self.stream_stats.entry(stream_id).or_default().downgrade()
    }

    /// Record incoming packet for stream
    pub fn record_in(&self, stream_id: u64, bytes: u64) {
        self.stream_stats
            .entry(stream_id)
            .or_default()
            .record_in(bytes);
    }

    /// Record outgoing packet for stream
    pub fn record_out(&self, stream_id: u64, bytes: u64) {
        self.stream_stats
            .entry(stream_id)
            .or_default()
            .record_out(bytes);
    }

    /// Record dropped packet for stream
    pub fn record_drop(&self, stream_id: u64) {
        self.stream_stats
            .entry(stream_id)
            .or_default()
            .record_drop();
    }

    /// Get number of routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Get number of active streams
    pub fn stream_count(&self) -> usize {
        self.stream_stats.len()
    }

    /// Mark route as inactive (on failure)
    pub fn deactivate_route(&self, dest_id: u64) {
        if let Some(mut entry) = self.routes.get_mut(&dest_id) {
            entry.active = false;
        }
    }

    /// Reactivate route
    pub fn activate_route(&self, dest_id: u64) {
        if let Some(mut entry) = self.routes.get_mut(&dest_id) {
            entry.active = true;
            entry.updated_at = Instant::now();
        }
    }

    /// Get all routes (for debugging/stats)
    pub fn all_routes(&self) -> Vec<(u64, RouteEntry)> {
        self.routes
            .iter()
            .map(|r| (*r.key(), r.value().clone()))
            .collect()
    }

    /// Clean up idle streams (no activity for given duration)
    pub fn cleanup_idle_streams(&self, idle_nanos: u64) -> usize {
        let mut removed = 0;
        self.stream_stats.retain(|_, stats| {
            if stats.is_idle(idle_nanos) {
                removed += 1;
                false
            } else {
                true
            }
        });
        removed
    }

    /// Get aggregate stats
    pub fn aggregate_stats(&self) -> AggregateStats {
        let mut total_in = 0u64;
        let mut total_out = 0u64;
        let mut total_drops = 0u64;

        for entry in self.stream_stats.iter() {
            total_in += entry.get_packets_in();
            total_out += entry.get_packets_out();
            total_drops += entry.get_drops();
        }

        AggregateStats {
            routes: self.routes.len(),
            streams: self.stream_stats.len(),
            packets_in: total_in,
            packets_out: total_out,
            packets_dropped: total_drops,
        }
    }
}

impl std::fmt::Debug for RoutingTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoutingTable")
            .field("local_id", &format!("{:016x}", self.local_id))
            .field("routes", &self.routes.len())
            .field("streams", &self.stream_stats.len())
            .finish()
    }
}

/// Aggregate routing statistics
#[derive(Debug, Clone, Default)]
pub struct AggregateStats {
    /// Number of routes
    pub routes: usize,
    /// Number of active streams
    pub streams: usize,
    /// Total packets received
    pub packets_in: u64,
    /// Total packets forwarded
    pub packets_out: u64,
    /// Total packets dropped
    pub packets_dropped: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_header_roundtrip() {
        let header = RoutingHeader::new(0x123456789ABCDEF0, 0xDEADBEEF, 8);
        let bytes = header.to_bytes();
        let parsed = RoutingHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header, parsed);
    }

    #[test]
    fn test_routing_header_forward() {
        let mut header = RoutingHeader::new(0x1234, 0x5678, 3);
        assert_eq!(header.ttl, 3);
        assert_eq!(header.hop_count, 0);

        assert!(header.forward());
        assert_eq!(header.ttl, 2);
        assert_eq!(header.hop_count, 1);

        assert!(header.forward());
        assert!(header.forward());
        assert_eq!(header.ttl, 0);
        assert_eq!(header.hop_count, 3);

        // Can't forward with TTL=0
        assert!(!header.forward());
    }

    #[test]
    fn test_routing_header_flags() {
        let control = RoutingHeader::control(0x1234, 0x5678, 2);
        assert!(control.flags.is_control());

        let priority = RoutingHeader::priority(0x1234, 0x5678, 2);
        assert!(priority.flags.is_priority());
    }

    #[test]
    fn test_route_flags_combined() {
        // Regression: from_u8 used to match only single-flag values.
        // Combined flags (e.g., Control | RequiresAck) mapped to None.
        let combined = RouteFlags::CONTROL.as_u8() | RouteFlags::REQUIRES_ACK.as_u8();
        let parsed = RouteFlags::from_u8(combined);
        assert!(
            parsed.is_control(),
            "Control bit must survive combined parse"
        );
        assert!(
            parsed.contains(RouteFlags::REQUIRES_ACK),
            "RequiresAck bit must survive combined parse"
        );

        let all = RouteFlags::CONTROL.as_u8()
            | RouteFlags::REQUIRES_ACK.as_u8()
            | RouteFlags::PRIORITY.as_u8()
            | RouteFlags::END_OF_STREAM.as_u8();
        let parsed_all = RouteFlags::from_u8(all);
        assert!(parsed_all.is_control());
        assert!(parsed_all.is_priority());
        assert!(parsed_all.contains(RouteFlags::REQUIRES_ACK));
        assert!(parsed_all.contains(RouteFlags::END_OF_STREAM));
    }

    #[test]
    fn test_route_flags_roundtrip() {
        // Verify combined flags survive to_bytes/from_bytes roundtrip
        let mut header = RoutingHeader::new(0x1234, 0x5678, 4);
        header.flags =
            RouteFlags::from_u8(RouteFlags::PRIORITY.as_u8() | RouteFlags::REQUIRES_ACK.as_u8());

        let bytes = header.to_bytes();
        let parsed = RoutingHeader::from_bytes(&bytes).unwrap();
        assert!(parsed.flags.is_priority());
        assert!(parsed.flags.contains(RouteFlags::REQUIRES_ACK));
    }

    #[test]
    fn test_routing_table_basic() {
        let table = RoutingTable::new(0x1234);

        let addr1: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:9001".parse().unwrap();

        table.add_route(0x5678, addr1);
        table.add_route(0x9ABC, addr2);

        assert_eq!(table.lookup(0x5678), Some(addr1));
        assert_eq!(table.lookup(0x9ABC), Some(addr2));
        assert_eq!(table.lookup(0xFFFF), None);

        assert!(table.is_local(0x1234));
        assert!(!table.is_local(0x5678));
    }

    #[test]
    fn test_routing_table_deactivate() {
        let table = RoutingTable::new(0x1234);
        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();

        table.add_route(0x5678, addr);
        assert_eq!(table.lookup(0x5678), Some(addr));

        table.deactivate_route(0x5678);
        assert_eq!(table.lookup(0x5678), None);

        table.activate_route(0x5678);
        assert_eq!(table.lookup(0x5678), Some(addr));
    }

    #[test]
    fn test_stream_stats() {
        let stats = StreamStats::new();

        stats.record_in(100);
        stats.record_in(200);
        stats.record_out(100);
        stats.record_drop();

        assert_eq!(stats.get_packets_in(), 2);
        assert_eq!(stats.get_packets_out(), 1);
        assert_eq!(stats.get_drops(), 1);
    }

    #[test]
    fn test_routing_table_stats() {
        let table = RoutingTable::new(0x1234);

        table.record_in(1, 100);
        table.record_in(1, 200);
        table.record_in(2, 150);
        table.record_out(1, 100);
        table.record_drop(2);

        let stats = table.aggregate_stats();
        assert_eq!(stats.streams, 2);
        assert_eq!(stats.packets_in, 3);
        assert_eq!(stats.packets_out, 1);
        assert_eq!(stats.packets_dropped, 1);
    }
}
