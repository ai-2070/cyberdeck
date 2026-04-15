//! Swarm discovery and graph maintenance for NLTP.
//!
//! This module provides:
//! - `Pingwave` - Periodic discovery packets for neighbor detection
//! - `CapabilityAd` - Node capability advertisements
//! - `LocalGraph` - Local view of the network topology (k-hop radius)
//! - `NodeInfo` / `EdgeInfo` - Graph node and edge metadata

use bytes::{Buf, BufMut, Bytes, BytesMut};
use dashmap::DashMap;
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Pingwave packet size in bytes
pub const PINGWAVE_SIZE: usize = 24;

/// Maximum capabilities string length
#[allow(dead_code)]
pub const MAX_CAPABILITIES_LEN: usize = 256;

/// Pingwave packet for neighbor discovery.
///
/// Layout (24 bytes):
/// ```text
/// ┌────────────────────────────────────────────────────────────┐
/// │ origin_id (8) │ seq (8) │ ttl (1) │ hops (1) │ reserved (6)│
/// └────────────────────────────────────────────────────────────┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Pingwave {
    /// Originating node ID
    pub origin_id: u64,
    /// Sequence number (monotonic per origin)
    pub seq: u64,
    /// Time-to-live (usually 2-3 hops)
    pub ttl: u8,
    /// Hop count so far
    pub hop_count: u8,
    /// Reserved for future use
    pub _reserved: [u8; 6],
}

impl Pingwave {
    /// Create a new pingwave
    pub fn new(origin_id: u64, seq: u64, ttl: u8) -> Self {
        Self {
            origin_id,
            seq,
            ttl,
            hop_count: 0,
            _reserved: [0; 6],
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; PINGWAVE_SIZE] {
        let mut buf = [0u8; PINGWAVE_SIZE];
        buf[0..8].copy_from_slice(&self.origin_id.to_le_bytes());
        buf[8..16].copy_from_slice(&self.seq.to_le_bytes());
        buf[16] = self.ttl;
        buf[17] = self.hop_count;
        // reserved bytes already zero
        buf
    }

    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < PINGWAVE_SIZE {
            return None;
        }
        Some(Self {
            origin_id: u64::from_le_bytes(buf[0..8].try_into().ok()?),
            seq: u64::from_le_bytes(buf[8..16].try_into().ok()?),
            ttl: buf[16],
            hop_count: buf[17],
            _reserved: [0; 6],
        })
    }

    /// Write to buffer
    pub fn write_to(&self, buf: &mut BytesMut) {
        buf.put_u64_le(self.origin_id);
        buf.put_u64_le(self.seq);
        buf.put_u8(self.ttl);
        buf.put_u8(self.hop_count);
        buf.put_slice(&[0u8; 6]); // reserved
    }

    /// Read from buffer
    pub fn read_from(buf: &mut Bytes) -> Option<Self> {
        if buf.remaining() < PINGWAVE_SIZE {
            return None;
        }
        Some(Self {
            origin_id: buf.get_u64_le(),
            seq: buf.get_u64_le(),
            ttl: buf.get_u8(),
            hop_count: buf.get_u8(),
            _reserved: {
                buf.advance(6);
                [0; 6]
            },
        })
    }

    /// Check if pingwave has expired (TTL = 0)
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.ttl == 0
    }

    /// Forward the pingwave (decrement TTL, increment hop count)
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

/// Node capabilities for capability-based routing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Capabilities {
    /// Has GPU acceleration
    pub gpu: bool,
    /// Available tools/functions
    pub tools: Vec<String>,
    /// Available memory in MB
    pub memory_mb: u32,
    /// Number of model slots available
    pub model_slots: u8,
    /// Custom tags
    pub tags: Vec<String>,
}

impl Capabilities {
    /// Create empty capabilities
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with GPU flag
    pub fn with_gpu(mut self, gpu: bool) -> Self {
        self.gpu = gpu;
        self
    }

    /// Add a tool
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }

    /// Set memory
    pub fn with_memory(mut self, memory_mb: u32) -> Self {
        self.memory_mb = memory_mb;
        self
    }

    /// Set model slots
    pub fn with_model_slots(mut self, slots: u8) -> Self {
        self.model_slots = slots;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Check if node has a specific tool
    pub fn has_tool(&self, tool: &str) -> bool {
        self.tools.iter().any(|t| t == tool)
    }

    /// Check if node has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Serialize to bytes (simple format)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);

        // Flags byte: bit 0 = gpu
        let flags = if self.gpu { 0x01 } else { 0x00 };
        buf.push(flags);

        // Memory (4 bytes)
        buf.extend_from_slice(&self.memory_mb.to_le_bytes());

        // Model slots (1 byte)
        buf.push(self.model_slots);

        // Tool count + tools (capped at 255 items, 255 bytes per string)
        let tool_count = self.tools.len().min(255);
        buf.push(tool_count as u8);
        for tool in &self.tools[..tool_count] {
            let bytes = tool.as_bytes();
            let len = bytes.len().min(255);
            buf.push(len as u8);
            buf.extend_from_slice(&bytes[..len]);
        }

        // Tag count + tags (capped at 255 items, 255 bytes per string)
        let tag_count = self.tags.len().min(255);
        buf.push(tag_count as u8);
        for tag in &self.tags[..tag_count] {
            let bytes = tag.as_bytes();
            let len = bytes.len().min(255);
            buf.push(len as u8);
            buf.extend_from_slice(&bytes[..len]);
        }

        buf
    }

    /// Deserialize from bytes
    pub fn from_bytes(mut buf: &[u8]) -> Option<Self> {
        if buf.len() < 7 {
            return None;
        }

        let flags = buf[0];
        let gpu = (flags & 0x01) != 0;

        let memory_mb = u32::from_le_bytes(buf[1..5].try_into().ok()?);
        let model_slots = buf[5];
        let tool_count = buf[6] as usize;

        buf = &buf[7..];

        let mut tools = Vec::with_capacity(tool_count);
        for _ in 0..tool_count {
            if buf.is_empty() {
                return None;
            }
            let len = buf[0] as usize;
            buf = &buf[1..];
            if buf.len() < len {
                return None;
            }
            let tool = std::str::from_utf8(&buf[..len]).ok()?.to_string();
            tools.push(tool);
            buf = &buf[len..];
        }

        if buf.is_empty() {
            return None;
        }
        let tag_count = buf[0] as usize;
        buf = &buf[1..];

        let mut tags = Vec::with_capacity(tag_count);
        for _ in 0..tag_count {
            if buf.is_empty() {
                return None;
            }
            let len = buf[0] as usize;
            buf = &buf[1..];
            if buf.len() < len {
                return None;
            }
            let tag = std::str::from_utf8(&buf[..len]).ok()?.to_string();
            tags.push(tag);
            buf = &buf[len..];
        }

        Some(Self {
            gpu,
            tools,
            memory_mb,
            model_slots,
            tags,
        })
    }
}

/// Capability advertisement packet.
#[derive(Debug, Clone)]
pub struct CapabilityAd {
    /// Node ID
    pub node_id: u64,
    /// Version (for updates)
    pub version: u32,
    /// Capabilities
    pub capabilities: Capabilities,
}

impl CapabilityAd {
    /// Create a new capability advertisement
    pub fn new(node_id: u64, version: u32, capabilities: Capabilities) -> Self {
        Self {
            node_id,
            version,
            capabilities,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let cap_bytes = self.capabilities.to_bytes();
        let mut buf = Vec::with_capacity(12 + cap_bytes.len());

        buf.extend_from_slice(&self.node_id.to_le_bytes());
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&cap_bytes);

        buf
    }

    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < 12 {
            return None;
        }

        let node_id = u64::from_le_bytes(buf[0..8].try_into().ok()?);
        let version = u32::from_le_bytes(buf[8..12].try_into().ok()?);
        let capabilities = Capabilities::from_bytes(&buf[12..])?;

        Some(Self {
            node_id,
            version,
            capabilities,
        })
    }
}

/// Information about a node in the local graph.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Node ID
    pub node_id: u64,
    /// Network address
    pub addr: SocketAddr,
    /// Hop distance from local node
    pub hops: u8,
    /// Last seen timestamp
    pub last_seen: Instant,
    /// Latest pingwave sequence from this node
    pub last_seq: u64,
    /// Capabilities (if known)
    pub capabilities: Option<Capabilities>,
    /// Capability version
    pub cap_version: u32,
}

impl NodeInfo {
    /// Create new node info
    pub fn new(node_id: u64, addr: SocketAddr, hops: u8) -> Self {
        Self {
            node_id,
            addr,
            hops,
            last_seen: Instant::now(),
            last_seq: 0,
            capabilities: None,
            cap_version: 0,
        }
    }

    /// Update last seen
    pub fn touch(&mut self) {
        self.last_seen = Instant::now();
    }

    /// Check if node is stale
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }

    /// Update capabilities if newer version
    pub fn update_capabilities(&mut self, version: u32, caps: Capabilities) -> bool {
        if version > self.cap_version {
            self.capabilities = Some(caps);
            self.cap_version = version;
            true
        } else {
            false
        }
    }
}

/// Information about an edge (connection) between nodes.
#[derive(Debug, Clone)]
pub struct EdgeInfo {
    /// Source node ID
    pub from: u64,
    /// Destination node ID
    pub to: u64,
    /// Estimated latency in microseconds
    pub latency_us: u32,
    /// Last update timestamp
    pub last_updated: Instant,
}

impl EdgeInfo {
    /// Create new edge info
    pub fn new(from: u64, to: u64) -> Self {
        Self {
            from,
            to,
            latency_us: 0,
            last_updated: Instant::now(),
        }
    }

    /// Create with latency
    pub fn with_latency(from: u64, to: u64, latency_us: u32) -> Self {
        Self {
            from,
            to,
            latency_us,
            last_updated: Instant::now(),
        }
    }
}

/// Local view of the network graph.
///
/// Maintains a k-hop neighborhood view of the network topology.
pub struct LocalGraph {
    /// Local node ID
    my_id: u64,
    /// Maximum hops to track
    radius: u8,
    /// Known nodes
    nodes: DashMap<u64, NodeInfo>,
    /// Edges (from, to) -> EdgeInfo
    edges: DashMap<(u64, u64), EdgeInfo>,
    /// Seen pingwaves (origin_id, seq) for deduplication
    seen_pingwaves: DashMap<(u64, u64), Instant>,
    /// Next pingwave sequence number
    next_seq: AtomicU64,
    /// Node timeout
    node_timeout: Duration,
    /// Pingwave cache timeout
    pingwave_cache_timeout: Duration,
}

impl LocalGraph {
    /// Create a new local graph
    pub fn new(my_id: u64, radius: u8) -> Self {
        Self {
            my_id,
            radius,
            nodes: DashMap::new(),
            edges: DashMap::new(),
            seen_pingwaves: DashMap::new(),
            next_seq: AtomicU64::new(1),
            node_timeout: Duration::from_secs(30),
            pingwave_cache_timeout: Duration::from_secs(10),
        }
    }

    /// Set node timeout
    pub fn with_node_timeout(mut self, timeout: Duration) -> Self {
        self.node_timeout = timeout;
        self
    }

    /// Get local node ID
    pub fn my_id(&self) -> u64 {
        self.my_id
    }

    /// Get radius
    pub fn radius(&self) -> u8 {
        self.radius
    }

    /// Create a new pingwave to broadcast
    pub fn create_pingwave(&self) -> Pingwave {
        let seq = self.next_seq.fetch_add(1, Ordering::Relaxed);
        Pingwave::new(self.my_id, seq, self.radius)
    }

    /// Process an incoming pingwave.
    ///
    /// Returns Some(pingwave) if it should be forwarded, None otherwise.
    pub fn on_pingwave(&self, mut pw: Pingwave, from: SocketAddr) -> Option<Pingwave> {
        // Ignore our own pingwaves
        if pw.origin_id == self.my_id {
            return None;
        }

        // Check if already seen
        let key = (pw.origin_id, pw.seq);
        if self.seen_pingwaves.contains_key(&key) {
            return None;
        }

        // Mark as seen
        self.seen_pingwaves.insert(key, Instant::now());

        // Update or create node info
        let hops = pw.hop_count + 1;
        self.nodes
            .entry(pw.origin_id)
            .and_modify(|n| {
                // Only update if this is a newer sequence or closer hop
                if pw.seq > n.last_seq || hops < n.hops {
                    n.last_seq = pw.seq;
                    n.hops = hops;
                    n.addr = from;
                    n.touch();
                }
            })
            .or_insert_with(|| {
                let mut info = NodeInfo::new(pw.origin_id, from, hops);
                info.last_seq = pw.seq;
                info
            });

        // Check if we should forward
        if pw.is_expired() {
            return None;
        }

        // Forward
        pw.forward();
        Some(pw)
    }

    /// Process a capability advertisement
    pub fn on_capability(&self, ad: CapabilityAd, from: SocketAddr) {
        self.nodes
            .entry(ad.node_id)
            .and_modify(|n| {
                n.update_capabilities(ad.version, ad.capabilities.clone());
            })
            .or_insert_with(|| {
                let mut info = NodeInfo::new(ad.node_id, from, 0);
                info.update_capabilities(ad.version, ad.capabilities.clone());
                info
            });
    }

    /// Add or update an edge
    pub fn add_edge(&self, from: u64, to: u64, latency_us: u32) {
        let key = (from, to);
        self.edges
            .entry(key)
            .and_modify(|e| {
                e.latency_us = latency_us;
                e.last_updated = Instant::now();
            })
            .or_insert_with(|| EdgeInfo::with_latency(from, to, latency_us));
    }

    /// Get node info
    pub fn get_node(&self, node_id: u64) -> Option<NodeInfo> {
        self.nodes.get(&node_id).map(|r| r.clone())
    }

    /// Get all known nodes
    pub fn all_nodes(&self) -> Vec<NodeInfo> {
        self.nodes.iter().map(|r| r.value().clone()).collect()
    }

    /// Get nodes within a specific hop distance
    pub fn nodes_within_hops(&self, max_hops: u8) -> Vec<NodeInfo> {
        self.nodes
            .iter()
            .filter(|r| r.hops <= max_hops)
            .map(|r| r.value().clone())
            .collect()
    }

    /// Find nodes with a specific capability (tool)
    pub fn find_by_tool(&self, tool: &str) -> Vec<NodeInfo> {
        self.nodes
            .iter()
            .filter(|r| {
                r.capabilities
                    .as_ref()
                    .map(|c| c.has_tool(tool))
                    .unwrap_or(false)
            })
            .map(|r| r.value().clone())
            .collect()
    }

    /// Find nodes with a specific tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<NodeInfo> {
        self.nodes
            .iter()
            .filter(|r| {
                r.capabilities
                    .as_ref()
                    .map(|c| c.has_tag(tag))
                    .unwrap_or(false)
            })
            .map(|r| r.value().clone())
            .collect()
    }

    /// Find nodes with GPU
    pub fn find_with_gpu(&self) -> Vec<NodeInfo> {
        self.nodes
            .iter()
            .filter(|r| r.capabilities.as_ref().map(|c| c.gpu).unwrap_or(false))
            .map(|r| r.value().clone())
            .collect()
    }

    /// Get shortest path to a node (BFS)
    pub fn path_to(&self, dest: u64) -> Option<Vec<u64>> {
        if dest == self.my_id {
            return Some(vec![self.my_id]);
        }

        // Build adjacency list from edges
        let mut adjacency: HashMap<u64, Vec<u64>> = HashMap::new();
        for edge in self.edges.iter() {
            adjacency.entry(edge.from).or_default().push(edge.to);
        }

        // BFS from my_id to dest
        let mut visited: HashSet<u64> = HashSet::new();
        let mut queue: VecDeque<Vec<u64>> = VecDeque::new();

        queue.push_back(vec![self.my_id]);
        visited.insert(self.my_id);

        while let Some(path) = queue.pop_front() {
            let current = *path.last()?;

            if current == dest {
                return Some(path);
            }

            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        let mut new_path = path.clone();
                        new_path.push(neighbor);
                        queue.push_back(new_path);
                    }
                }
            }
        }

        None
    }

    /// Clean up stale nodes and old pingwave cache entries
    pub fn cleanup(&self) -> (usize, usize) {
        let mut removed_nodes = 0;
        let mut removed_pingwaves = 0;

        // Remove stale nodes
        self.nodes.retain(|_, node| {
            if node.is_stale(self.node_timeout) {
                removed_nodes += 1;
                false
            } else {
                true
            }
        });

        // Remove old pingwave cache entries
        self.seen_pingwaves.retain(|_, instant| {
            if instant.elapsed() > self.pingwave_cache_timeout {
                removed_pingwaves += 1;
                false
            } else {
                true
            }
        });

        (removed_nodes, removed_pingwaves)
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            pingwave_cache_size: self.seen_pingwaves.len(),
        }
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl std::fmt::Debug for LocalGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalGraph")
            .field("my_id", &format!("{:016x}", self.my_id))
            .field("radius", &self.radius)
            .field("nodes", &self.nodes.len())
            .field("edges", &self.edges.len())
            .finish()
    }
}

/// Graph statistics
#[derive(Debug, Clone, Default)]
pub struct GraphStats {
    /// Number of known nodes
    pub node_count: usize,
    /// Number of known edges
    pub edge_count: usize,
    /// Pingwave cache size
    pub pingwave_cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pingwave_roundtrip() {
        let pw = Pingwave::new(0x123456789ABCDEF0, 42, 3);
        let bytes = pw.to_bytes();
        let parsed = Pingwave::from_bytes(&bytes).unwrap();
        assert_eq!(pw, parsed);
    }

    #[test]
    fn test_pingwave_forward() {
        let mut pw = Pingwave::new(0x1234, 1, 3);
        assert_eq!(pw.ttl, 3);
        assert_eq!(pw.hop_count, 0);

        assert!(pw.forward());
        assert_eq!(pw.ttl, 2);
        assert_eq!(pw.hop_count, 1);

        assert!(pw.forward());
        assert!(pw.forward());
        assert_eq!(pw.ttl, 0);
        assert_eq!(pw.hop_count, 3);

        // Can't forward with TTL=0
        assert!(!pw.forward());
    }

    #[test]
    fn test_capabilities_roundtrip() {
        let caps = Capabilities::new()
            .with_gpu(true)
            .with_memory(16384)
            .with_model_slots(4)
            .with_tool("python")
            .with_tool("rust")
            .with_tag("inference")
            .with_tag("training");

        let bytes = caps.to_bytes();
        let parsed = Capabilities::from_bytes(&bytes).unwrap();

        assert_eq!(caps.gpu, parsed.gpu);
        assert_eq!(caps.memory_mb, parsed.memory_mb);
        assert_eq!(caps.model_slots, parsed.model_slots);
        assert_eq!(caps.tools, parsed.tools);
        assert_eq!(caps.tags, parsed.tags);
    }

    #[test]
    fn test_capability_ad_roundtrip() {
        let caps = Capabilities::new().with_gpu(true).with_tool("test");
        let ad = CapabilityAd::new(0x1234, 5, caps);

        let bytes = ad.to_bytes();
        let parsed = CapabilityAd::from_bytes(&bytes).unwrap();

        assert_eq!(ad.node_id, parsed.node_id);
        assert_eq!(ad.version, parsed.version);
        assert_eq!(ad.capabilities.gpu, parsed.capabilities.gpu);
    }

    #[test]
    fn test_capabilities_large_strings_capped() {
        // Regression: tools.len() and string lengths were cast to u8 without
        // bounds checks, silently truncating counts > 255 and strings > 255 bytes.
        let long_tool = "x".repeat(300);
        let caps = Capabilities::new().with_tool(&long_tool);

        let bytes = caps.to_bytes();
        let parsed = Capabilities::from_bytes(&bytes).unwrap();

        // String should be truncated to 255 bytes, not wrap around
        assert_eq!(parsed.tools.len(), 1);
        assert_eq!(parsed.tools[0].len(), 255);
    }

    #[test]
    fn test_capabilities_many_items_capped() {
        // More than 255 tools should be capped, not truncated via `as u8`
        let mut caps = Capabilities::new();
        for i in 0..300 {
            caps = caps.with_tool(&format!("t{}", i));
        }

        let bytes = caps.to_bytes();
        let parsed = Capabilities::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.tools.len(), 255);
    }

    #[test]
    fn test_local_graph_pingwave_processing() {
        let graph = LocalGraph::new(0x1111, 3);

        // Simulate receiving a pingwave from a neighbor
        let pw = Pingwave::new(0x2222, 1, 3);
        let from: SocketAddr = "127.0.0.1:9000".parse().unwrap();

        let forwarded = graph.on_pingwave(pw, from);
        assert!(forwarded.is_some());

        // Check node was added
        let node = graph.get_node(0x2222).unwrap();
        assert_eq!(node.hops, 1);
        assert_eq!(node.addr, from);

        // Same pingwave should be deduplicated
        let pw2 = Pingwave::new(0x2222, 1, 3);
        assert!(graph.on_pingwave(pw2, from).is_none());

        // New sequence should be accepted
        let pw3 = Pingwave::new(0x2222, 2, 3);
        assert!(graph.on_pingwave(pw3, from).is_some());
    }

    #[test]
    fn test_local_graph_capability_search() {
        let graph = LocalGraph::new(0x1111, 3);

        // Add some nodes
        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();
        graph.nodes.insert(0x2222, NodeInfo::new(0x2222, addr, 1));
        graph.nodes.insert(0x3333, NodeInfo::new(0x3333, addr, 2));

        // Add capabilities
        let caps1 = Capabilities::new().with_gpu(true).with_tool("python");
        let caps2 = Capabilities::new().with_gpu(false).with_tool("rust");

        graph.on_capability(CapabilityAd::new(0x2222, 1, caps1), addr);
        graph.on_capability(CapabilityAd::new(0x3333, 1, caps2), addr);

        // Search by tool
        let python_nodes = graph.find_by_tool("python");
        assert_eq!(python_nodes.len(), 1);
        assert_eq!(python_nodes[0].node_id, 0x2222);

        // Search by GPU
        let gpu_nodes = graph.find_with_gpu();
        assert_eq!(gpu_nodes.len(), 1);
        assert_eq!(gpu_nodes[0].node_id, 0x2222);
    }

    #[test]
    fn test_capability_ad_creates_unknown_node() {
        // Regression: on_capability() only called and_modify(), so capability
        // advertisements for nodes not yet seen via pingwave were silently
        // dropped. The node never became searchable by capability.
        let graph = LocalGraph::new(0x1111, 3);
        let addr: SocketAddr = "127.0.0.1:9000".parse().unwrap();

        // No pingwave — node 0x2222 is completely unknown
        assert!(graph.get_node(0x2222).is_none());

        // Send a capability advertisement directly
        let caps = Capabilities::new().with_gpu(true).with_tool("python");
        graph.on_capability(CapabilityAd::new(0x2222, 1, caps), addr);

        // Node should now exist and be searchable
        let node = graph.get_node(0x2222);
        assert!(node.is_some(), "node should be created from capability ad");
        let node = node.unwrap();
        assert!(node.capabilities.is_some());

        let gpu_nodes = graph.find_with_gpu();
        assert_eq!(gpu_nodes.len(), 1);
        assert_eq!(gpu_nodes[0].node_id, 0x2222);
    }

    #[test]
    fn test_local_graph_path_finding() {
        let graph = LocalGraph::new(0x1111, 3);

        // Add edges: 1111 -> 2222 -> 3333 -> 4444
        graph.add_edge(0x1111, 0x2222, 100);
        graph.add_edge(0x2222, 0x3333, 100);
        graph.add_edge(0x3333, 0x4444, 100);

        // Find path to 4444
        let path = graph.path_to(0x4444);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path, vec![0x1111, 0x2222, 0x3333, 0x4444]);

        // No path to unknown node
        let no_path = graph.path_to(0x9999);
        assert!(no_path.is_none());
    }
}
