//! Routing primitives for Net multi-hop transport.
//!
//! This module provides:
//! - `RoutingHeader`: Fixed-size header for multi-hop routing
//! - `RoutingTable`: Stream-to-destination mapping
//! - `SchedulerStreamStats`: Per-stream statistics for fairness monitoring

use bytes::{Buf, BufMut, Bytes, BytesMut};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Routing header size in bytes.
///
/// Layout is `magic(2) + ttl(1) + hop_count(1) + flags(1) + _reserved(1)
/// + src_id(4) + dest_id(8)` = 18 bytes. The magic tag at bytes 0-1
/// unambiguously distinguishes routing headers from direct Net
/// packets (whose own magic is [`super::protocol::MAGIC`]) so the
/// receive-loop discriminator doesn't depend on `dest_id` happening
/// to not collide with `0x4E45`.
pub const ROUTING_HEADER_SIZE: usize = 18;

/// Magic bytes identifying a routing header: `[0x52, 0x54]` on the
/// wire — ASCII "RT" in read order, for "routing". Stored as a u16
/// little-endian value, that's `0x5452`. Chosen disjoint from the
/// Net packet magic (`0x4E45`) so the receive-loop can discriminate
/// on the first two bytes alone.
pub const ROUTING_MAGIC: u16 = 0x5452;

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

/// Routing header for multi-hop Net packets.
///
/// Layout (18 bytes):
/// ```text
/// ┌───────────────────────────────────────────────────────────────────┐
/// │ magic (2) │ ttl │ hops │ flags │ rsvd │ src_id (4) │ dest_id (8) │
/// └───────────────────────────────────────────────────────────────────┘
/// ```
///
/// `magic` is always [`ROUTING_MAGIC`] (`0x5254`), distinct from the
/// direct-packet magic `0x4E45` ([`super::protocol::MAGIC`]). The
/// receive-loop discriminator reads bytes 0-1 alone and dispatches
/// unambiguously — the previous 16-byte layout put `dest_id` at bytes
/// 0-7, and any recipient whose `node_id` had low-16-bits equal to
/// the direct-packet magic (~1 in 65 536) silently mis-classified
/// its own incoming routed packets as Net packets.
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

    /// Serialize to bytes.
    ///
    /// The magic tag rides at bytes 0-1 so the receive-loop
    /// discriminator reads it directly — see [`ROUTING_MAGIC`].
    pub fn to_bytes(&self) -> [u8; ROUTING_HEADER_SIZE] {
        let mut buf = [0u8; ROUTING_HEADER_SIZE];
        buf[0..2].copy_from_slice(&ROUTING_MAGIC.to_le_bytes());
        buf[2] = self.ttl;
        buf[3] = self.hop_count;
        buf[4] = self.flags.as_u8();
        buf[5] = self._reserved;
        buf[6..10].copy_from_slice(&self.src_id.to_le_bytes());
        buf[10..18].copy_from_slice(&self.dest_id.to_le_bytes());
        buf
    }

    /// Deserialize from bytes. Returns `None` on short input, wrong
    /// magic, or malformed numeric fields.
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < ROUTING_HEADER_SIZE {
            return None;
        }
        let magic = u16::from_le_bytes([buf[0], buf[1]]);
        if magic != ROUTING_MAGIC {
            return None;
        }
        Some(Self {
            ttl: buf[2],
            hop_count: buf[3],
            flags: RouteFlags::from_u8(buf[4]),
            _reserved: buf[5],
            src_id: u32::from_le_bytes(buf[6..10].try_into().ok()?),
            dest_id: u64::from_le_bytes(buf[10..18].try_into().ok()?),
        })
    }

    /// Write to a buffer
    pub fn write_to(&self, buf: &mut BytesMut) {
        buf.put_u16_le(ROUTING_MAGIC);
        buf.put_u8(self.ttl);
        buf.put_u8(self.hop_count);
        buf.put_u8(self.flags.as_u8());
        buf.put_u8(self._reserved);
        buf.put_u32_le(self.src_id);
        buf.put_u64_le(self.dest_id);
    }

    /// Read from a buffer. Returns `None` on short input or wrong
    /// magic; fields are consumed only on successful parse.
    pub fn read_from(buf: &mut Bytes) -> Option<Self> {
        if buf.remaining() < ROUTING_HEADER_SIZE {
            return None;
        }
        // Peek at magic without advancing so a bad prefix leaves
        // the cursor intact for callers that want to try another
        // decoder.
        let magic = u16::from_le_bytes([buf[0], buf[1]]);
        if magic != ROUTING_MAGIC {
            return None;
        }
        let _ = buf.get_u16_le();
        let ttl = buf.get_u8();
        let hop_count = buf.get_u8();
        let flags = RouteFlags::from_u8(buf.get_u8());
        let _reserved = buf.get_u8();
        let src_id = buf.get_u32_le();
        let dest_id = buf.get_u64_le();
        Some(Self {
            dest_id,
            src_id,
            ttl,
            hop_count,
            flags,
            _reserved,
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
pub struct SchedulerStreamStats {
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

impl SchedulerStreamStats {
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

impl Default for SchedulerStreamStats {
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
    stream_stats: DashMap<u64, SchedulerStreamStats>,
    /// Local node ID
    local_id: u64,
    /// Maximum age a route may have before `lookup` rejects it.
    /// Stored as nanoseconds in an `AtomicU64` so `set_max_route_age` is
    /// cheap and lock-free. Initialized to `u64::MAX` (effectively
    /// disabled) — `MeshNode` sets this at construction.
    max_route_age_nanos: AtomicU64,
}

impl RoutingTable {
    /// Create a new routing table
    pub fn new(local_id: u64) -> Self {
        Self {
            routes: DashMap::new(),
            stream_stats: DashMap::new(),
            local_id,
            max_route_age_nanos: AtomicU64::new(u64::MAX),
        }
    }

    /// Get local node ID
    #[inline]
    pub fn local_id(&self) -> u64 {
        self.local_id
    }

    /// Add or update a direct route.
    ///
    /// Called by `MeshNode::connect` and `::accept` as part of direct
    /// session setup. Unconditionally inserts — a direct route is always
    /// preferred over any indirect one (direct uses default metric 1;
    /// indirect routes installed from pingwaves carry `hop_count + 2`, so
    /// they're never below 2). Also refreshes `updated_at`.
    pub fn add_route(&self, dest_id: u64, next_hop: SocketAddr) {
        self.routes.insert(dest_id, RouteEntry::new(next_hop));
    }

    /// Add or update a route with an explicit metric.
    ///
    /// Used by the pingwave-driven route installer. The existing entry is
    /// preserved (but its `updated_at` is refreshed) if its metric is
    /// **strictly better** (lower) than the incoming one — this keeps a
    /// direct route from being replaced by a worse indirect one, while
    /// still letting a periodic refresh extend the direct route's
    /// freshness window via the heartbeat loop.
    ///
    /// If the new metric is equal or better, the entry is replaced
    /// (capturing any next-hop change and refreshing `updated_at`).
    pub fn add_route_with_metric(&self, dest_id: u64, next_hop: SocketAddr, metric: u16) {
        use dashmap::mapref::entry::Entry;
        match self.routes.entry(dest_id) {
            Entry::Vacant(v) => {
                v.insert(RouteEntry::with_metric(next_hop, metric));
            }
            Entry::Occupied(mut o) => {
                if metric <= o.get().metric {
                    o.insert(RouteEntry::with_metric(next_hop, metric));
                } else {
                    // Existing route is strictly better. Keep it, but
                    // refresh its freshness — if the better route is
                    // still there, the worse one's arrival is evidence
                    // the destination is reachable, so the direct route
                    // shouldn't time out just because its heartbeat
                    // happens less often than pingwaves.
                    o.get_mut().updated_at = Instant::now();
                }
            }
        }
    }

    /// Remove a route
    pub fn remove_route(&self, dest_id: u64) -> Option<RouteEntry> {
        self.routes.remove(&dest_id).map(|(_, v)| v)
    }

    /// Remove the route for `dest_id` only if its current `next_hop`
    /// still equals `expected_next_hop`. Used by rollback paths that
    /// registered a specific route and need to undo it without clobbering
    /// a newer concurrently-written entry. Returns `true` if the entry
    /// was removed.
    pub fn remove_route_if_next_hop_is(&self, dest_id: u64, expected_next_hop: SocketAddr) -> bool {
        self.routes
            .remove_if(&dest_id, |_, entry| entry.next_hop == expected_next_hop)
            .is_some()
    }

    /// Look up next hop for destination.
    ///
    /// Returns `None` for stale routes — an entry whose `updated_at` is
    /// older than the configured `max_route_age` (default: very large;
    /// call [`Self::set_max_route_age`] to enable expiry). Stale entries
    /// stay in the map until a periodic [`Self::sweep_stale`] call removes
    /// them.
    pub fn lookup(&self, dest_id: u64) -> Option<SocketAddr> {
        let max_age = self.max_route_age();
        self.routes
            .get(&dest_id)
            .filter(|r| r.active && r.updated_at.elapsed() <= max_age)
            .map(|r| r.next_hop)
    }

    /// Like [`Self::lookup`], but returns `None` if the installed
    /// route's `next_hop` equals `exclude_next_hop`. Used by
    /// [`crate::adapter::net::ReroutePolicy`] so a single failed-peer
    /// check against the routing table answers "do I have a usable
    /// alternate?" — if `Some(addr)`, use it directly; if `None`,
    /// fall back to a graph-based alternate lookup.
    ///
    /// Today the routing table stores one entry per destination, so
    /// the "alternate" is either the current entry (if not excluded)
    /// or nothing. When the table grows to hold ranked alternates
    /// per destination, the signature stays the same and the
    /// implementation picks the lowest-metric non-excluded entry.
    pub fn lookup_alternate(
        &self,
        dest_id: u64,
        exclude_next_hop: SocketAddr,
    ) -> Option<SocketAddr> {
        let max_age = self.max_route_age();
        self.routes
            .get(&dest_id)
            .filter(|r| {
                r.active && r.updated_at.elapsed() <= max_age && r.next_hop != exclude_next_hop
            })
            .map(|r| r.next_hop)
    }

    /// Remove all routes whose `updated_at` is older than `max_age`.
    /// Returns the number of entries removed.
    ///
    /// Called periodically from the heartbeat loop to keep dead routes
    /// out of the table.
    pub fn sweep_stale(&self, max_age: std::time::Duration) -> usize {
        let mut removed = 0;
        self.routes.retain(|_, entry| {
            let keep = entry.updated_at.elapsed() <= max_age;
            if !keep {
                removed += 1;
            }
            keep
        });
        removed
    }

    /// Configure the maximum route age for `lookup` staleness checks.
    ///
    /// Defaults to `Duration::MAX` (effectively disabled). `MeshNode`
    /// sets this to `3 × session_timeout` at construction.
    pub fn set_max_route_age(&self, age: std::time::Duration) {
        self.max_route_age_nanos.store(
            age.as_nanos().min(u64::MAX as u128) as u64,
            Ordering::Relaxed,
        );
    }

    fn max_route_age(&self) -> std::time::Duration {
        let nanos = self.max_route_age_nanos.load(Ordering::Relaxed);
        std::time::Duration::from_nanos(nanos)
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
    ) -> dashmap::mapref::one::Ref<'_, u64, SchedulerStreamStats> {
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
    fn test_routing_header_magic_at_offset_zero() {
        // ROUTING_MAGIC must appear at bytes 0-1 regardless of
        // dest_id / src_id values. The receive-loop discriminator
        // peeks at bytes 0-1 and relies on this.
        let header = RoutingHeader::new(0x4E45_4E45_4E45_4E45, 0x4E45_4E45, 8);
        let bytes = header.to_bytes();
        assert_eq!(
            u16::from_le_bytes([bytes[0], bytes[1]]),
            ROUTING_MAGIC,
            "magic must live at bytes 0-1 independent of dest_id's own byte pattern",
        );
    }

    #[test]
    fn test_routing_header_rejects_wrong_magic() {
        // from_bytes must refuse buffers whose bytes 0-1 aren't
        // ROUTING_MAGIC — this is what lets the receive-loop
        // discriminator short-circuit cleanly without parsing the
        // rest of the header.
        let mut bytes = RoutingHeader::new(0x1234, 0x5678, 4).to_bytes();
        // Overwrite magic with direct-packet MAGIC.
        bytes[0..2].copy_from_slice(&0x4E45_u16.to_le_bytes());
        assert!(RoutingHeader::from_bytes(&bytes).is_none());

        // Overwrite with arbitrary garbage.
        bytes[0..2].copy_from_slice(&0xFFFF_u16.to_le_bytes());
        assert!(RoutingHeader::from_bytes(&bytes).is_none());
    }

    #[test]
    fn test_regression_routing_discriminator_survives_magic_collision_node_id() {
        // Regression (LOW, BUGS.md): the old 16-byte layout put
        // `dest_id` at bytes 0-7. When a recipient's own node_id
        // had low-16-bits equal to 0x4E45 (the direct Net-packet
        // magic), routed packets to that node were
        // mis-discriminated as direct packets and silently dropped
        // at the AEAD layer — 1-in-65 536 node_ids affected.
        //
        // The new layout puts ROUTING_MAGIC at bytes 0-1 and
        // shifts dest_id to bytes 10-17, so the discriminator is
        // unambiguous for every possible dest_id value.
        //
        // This test constructs a header whose dest_id has low-16
        // bits equal to the old ambiguous value and verifies that
        // the header still serializes with ROUTING_MAGIC at the
        // front and round-trips correctly.
        let ambiguous_dest: u64 = 0xDEAD_BEEF_FFFF_4E45;
        let header = RoutingHeader::new(ambiguous_dest, 0x1111_2222, 8);
        let bytes = header.to_bytes();
        assert_eq!(
            u16::from_le_bytes([bytes[0], bytes[1]]),
            ROUTING_MAGIC,
            "magic at offset 0 must be independent of dest_id",
        );
        let parsed = RoutingHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.dest_id, ambiguous_dest);
        assert_eq!(parsed.src_id, 0x1111_2222);
        assert_eq!(parsed.ttl, 8);
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
        let stats = SchedulerStreamStats::new();

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

    /// A direct route (metric 1) must NOT be replaced by an indirect
    /// route with a worse (higher) metric arriving later. This is the
    /// precedence invariant that makes pingwave-driven install safe: a
    /// pingwave from a far node via the same peer that IS our direct
    /// peer for some other destination can't accidentally downgrade us.
    #[test]
    fn test_add_route_with_metric_preserves_better_direct_route() {
        let table = RoutingTable::new(0x1111);
        let direct: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        let indirect: SocketAddr = "127.0.0.1:3000".parse().unwrap();

        // Direct insert (metric=1).
        table.add_route(0x2222, direct);
        assert_eq!(table.lookup(0x2222), Some(direct));

        // Indirect arrives with worse metric — must be ignored.
        table.add_route_with_metric(0x2222, indirect, 5);
        assert_eq!(
            table.lookup(0x2222),
            Some(direct),
            "worse indirect route must not displace the direct route"
        );

        // An equal-or-better metric DOES replace (captures a next-hop
        // change, e.g., if the direct peer moved).
        let better: SocketAddr = "127.0.0.1:4000".parse().unwrap();
        table.add_route_with_metric(0x2222, better, 1);
        assert_eq!(
            table.lookup(0x2222),
            Some(better),
            "equal-metric update must replace next_hop"
        );
    }

    /// Staleness: `lookup` must return `None` for entries whose
    /// `updated_at` is older than `max_route_age`. `sweep_stale`
    /// physically removes them.
    #[test]
    fn test_sweep_stale_and_staleness_aware_lookup() {
        use std::time::Duration;

        let table = RoutingTable::new(0x1111);
        let addr_a: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        let addr_b: SocketAddr = "127.0.0.1:3000".parse().unwrap();

        table.add_route(0x2222, addr_a);
        table.add_route(0x3333, addr_b);

        // Backdate 0x2222's entry so it looks stale.
        {
            let mut e = table.routes.get_mut(&0x2222).unwrap();
            e.updated_at = Instant::now() - Duration::from_secs(3600);
        }

        // With a small max-age, the backdated entry is stale but the
        // fresh one is still visible.
        table.set_max_route_age(Duration::from_secs(60));
        assert_eq!(table.lookup(0x2222), None);
        assert_eq!(table.lookup(0x3333), Some(addr_b));

        // Sweep physically removes the stale entry.
        let removed = table.sweep_stale(Duration::from_secs(60));
        assert_eq!(removed, 1);
        assert!(table.routes.get(&0x2222).is_none());
        assert!(table.routes.get(&0x3333).is_some());
    }

    #[test]
    fn test_regression_remove_route_if_next_hop_is() {
        // Regression: rollback paths (e.g., routed-handshake msg2 send
        // failure) used to call `remove_route` unconditionally and could
        // clobber a newer valid route written concurrently for the same
        // dest. `remove_route_if_next_hop_is` is the safe alternative —
        // it only removes when the current next_hop still matches the
        // address the caller wrote.
        let table = RoutingTable::new(0x1111);
        let original: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        let newer: SocketAddr = "127.0.0.1:3000".parse().unwrap();

        // Install original route.
        table.add_route(0x4444, original);

        // Concurrent rewrite to a different next hop.
        table.add_route(0x4444, newer);

        // Rollback keyed on the original next_hop must NOT remove the
        // newer entry.
        let removed = table.remove_route_if_next_hop_is(0x4444, original);
        assert!(
            !removed,
            "rollback must not evict an entry whose next_hop changed under us"
        );
        assert_eq!(
            table.lookup(0x4444),
            Some(newer),
            "newer route must survive a stale rollback attempt"
        );

        // Rollback keyed on the current next_hop DOES remove it.
        let removed = table.remove_route_if_next_hop_is(0x4444, newer);
        assert!(removed);
        assert!(table.lookup(0x4444).is_none());

        // Rolling back a non-existent route is a no-op, returns false.
        assert!(!table.remove_route_if_next_hop_is(0x4444, newer));
    }

    #[test]
    fn test_lookup_alternate() {
        let table = RoutingTable::new(0x1);
        let b: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        let c: SocketAddr = "127.0.0.1:3000".parse().unwrap();

        // Empty table — no alternate.
        assert!(table.lookup_alternate(0x4444, b).is_none());

        // Install `(0x4444 → B)`. Excluding B returns None; excluding
        // C returns B (the installed entry).
        table.add_route(0x4444, b);
        assert_eq!(table.lookup_alternate(0x4444, b), None);
        assert_eq!(table.lookup_alternate(0x4444, c), Some(b));
    }

    #[test]
    fn test_lookup_alternate_respects_staleness() {
        use std::time::Duration;
        let table = RoutingTable::new(0x1);
        let b: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        let c: SocketAddr = "127.0.0.1:3000".parse().unwrap();

        table.add_route(0x4444, b);
        // Backdate the entry so `updated_at.elapsed() > max_route_age`.
        {
            let mut e = table.routes.get_mut(&0x4444).unwrap();
            e.updated_at = Instant::now() - Duration::from_secs(3600);
        }
        table.set_max_route_age(Duration::from_secs(60));

        // Even though the next_hop isn't excluded, staleness drops it.
        assert!(table.lookup_alternate(0x4444, c).is_none());
    }

    // ========================================================================
    // TEST_COVERAGE_PLAN §P2-10 — routing-table concurrency safety.
    //
    // The mesh's receive loop calls `add_route_with_metric` from
    // whatever task decoded the pingwave; under high pingwave
    // volume multiple tasks hit the same entry simultaneously.
    // DashMap entry semantics + the metric-precedence rule must
    // converge on a deterministic best-metric winner without
    // torn writes or lost inserts.
    // ========================================================================

    /// N threads inserting routes with mixed metrics for the
    /// same destination must converge on the lowest metric seen.
    /// Pins the `Entry::Occupied` + metric-compare contract
    /// under contention. No assertion about *which* next_hop
    /// wins (ties are tolerant of the interleaving), only that
    /// the final metric is the minimum any thread inserted.
    #[test]
    fn concurrent_add_route_with_metric_converges_on_lowest_metric() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let table = Arc::new(RoutingTable::new(0x1111));
        let dest = 0x2222u64;
        let start = Arc::new(Barrier::new(8));

        let mut handles = Vec::new();
        for metric in 1u16..=8 {
            let table = table.clone();
            let start = start.clone();
            handles.push(thread::spawn(move || {
                start.wait();
                // Each thread hammers its own metric on the
                // same destination 500 times. The dashmap entry
                // API guarantees atomic compare-and-swap per
                // iteration.
                let next_hop: SocketAddr =
                    format!("127.0.0.1:{}", 10_000 + metric).parse().unwrap();
                for _ in 0..500 {
                    table.add_route_with_metric(dest, next_hop, metric);
                }
            }));
        }
        for h in handles {
            h.join().expect("thread panicked");
        }

        // After the race, the entry must exist and its metric
        // must be the lowest any thread offered.
        let entry = table
            .routes
            .get(&dest)
            .expect("route must exist after all threads inserted");
        assert_eq!(
            entry.metric, 1,
            "final metric must be the minimum (1) across all concurrent inserts — \
             a metric > 1 indicates a lost update or a torn compare-and-swap",
        );
        // Lookup returns the winning next_hop.
        let winner = table.lookup(dest).expect("dest must resolve");
        assert_eq!(
            winner,
            "127.0.0.1:10001".parse::<SocketAddr>().unwrap(),
            "lookup should return the next_hop paired with the winning metric",
        );
    }

    /// Direct routes (metric=1 via `add_route`) must never be
    /// displaced by concurrent pingwave-driven `add_route_with_metric`
    /// inserts carrying `metric >= 2`. Proves the metric-precedence
    /// rule holds under contention — a direct route's freshness
    /// timestamp may update (evidence of reachability from a
    /// pingwave along the same path) but the next_hop + metric
    /// stay pinned.
    #[test]
    fn direct_route_survives_concurrent_worse_indirect_inserts() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let table = Arc::new(RoutingTable::new(0x1111));
        let dest = 0x2222u64;
        let direct: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        table.add_route(dest, direct);
        assert_eq!(table.lookup(dest), Some(direct));
        let start = Arc::new(Barrier::new(9));

        let mut handles = Vec::new();
        for metric in 2u16..=10 {
            let table = table.clone();
            let start = start.clone();
            handles.push(thread::spawn(move || {
                start.wait();
                let indirect: SocketAddr =
                    format!("127.0.0.1:{}", 20_000 + metric).parse().unwrap();
                for _ in 0..500 {
                    table.add_route_with_metric(dest, indirect, metric);
                }
            }));
        }
        for h in handles {
            h.join().expect("thread panicked");
        }

        // The direct route must still be in place.
        assert_eq!(
            table.lookup(dest),
            Some(direct),
            "direct route (metric=1) must not be displaced by any \
             concurrent indirect insert with metric >= 2",
        );
        let entry = table.routes.get(&dest).unwrap();
        assert_eq!(entry.metric, 1, "metric must still be 1 (direct)");
    }
}
