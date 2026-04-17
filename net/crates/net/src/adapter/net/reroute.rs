//! Automatic rerouting policy.
//!
//! Watches the failure detector and updates the routing table when a peer
//! dies. When a peer recovers, restores the original route if it's better
//! than the current alternate.
//!
//! This module is wired into `MeshNode` via the `FailureDetector`'s
//! `on_failure` and `on_recovery` callbacks.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use dashmap::DashMap;

use super::behavior::proximity::ProximityGraph;
use super::route::RoutingTable;

/// Saved original route before reroute, for recovery.
struct SavedRoute {
    /// Original next-hop address
    next_hop: SocketAddr,
    /// The alternate we rerouted to
    #[allow(dead_code)]
    alternate: SocketAddr,
}

/// Policy that automatically reroutes traffic when peers fail.
///
/// When a peer is marked as failed by the `FailureDetector`:
/// 1. Find all routes whose next-hop is the failed peer's address
/// 2. For each, find an alternate peer (any other connected peer)
/// 3. Update the routing table to use the alternate
///
/// When the peer recovers:
/// 1. Restore the original routes (direct path is typically better)
pub struct ReroutePolicy {
    /// Routing table to update
    routing_table: Arc<RoutingTable>,
    /// Connected peers (node_id → addr mapping)
    peer_addrs: Arc<DashMap<u64, SocketAddr>>,
    /// Proximity graph for multi-hop alternate selection
    proximity_graph: Option<Arc<ProximityGraph>>,
    /// Saved original routes for recovery (dest_node_id → saved route)
    saved_routes: DashMap<u64, SavedRoute>,
    /// Total reroutes performed
    pub reroute_count: AtomicU64,
    /// Total recoveries performed
    pub recovery_count: AtomicU64,
}

/// Convert a u64 node_id to a 32-byte graph NodeId.
fn node_id_to_graph_id(node_id: u64) -> [u8; 32] {
    let mut id = [0u8; 32];
    id[0..8].copy_from_slice(&node_id.to_le_bytes());
    id
}

/// Extract u64 node_id from a 32-byte graph NodeId.
fn graph_id_to_node_id(id: &[u8; 32]) -> u64 {
    u64::from_le_bytes(id[0..8].try_into().unwrap())
}

impl ReroutePolicy {
    /// Create a new reroute policy.
    pub fn new(
        routing_table: Arc<RoutingTable>,
        peer_addrs: Arc<DashMap<u64, SocketAddr>>,
    ) -> Self {
        Self {
            routing_table,
            peer_addrs,
            proximity_graph: None,
            saved_routes: DashMap::new(),
            reroute_count: AtomicU64::new(0),
            recovery_count: AtomicU64::new(0),
        }
    }

    /// Set the proximity graph for multi-hop alternate selection.
    pub fn with_proximity_graph(mut self, graph: Arc<ProximityGraph>) -> Self {
        self.proximity_graph = Some(graph);
        self
    }

    /// Called when the failure detector marks a peer as failed.
    ///
    /// Finds all routes through the failed peer and reroutes them
    /// through an alternate peer. The original routes are saved
    /// for restoration on recovery.
    pub fn on_failure(&self, failed_node_id: u64) {
        // Resolve failed node's address
        let failed_addr = match self.peer_addrs.get(&failed_node_id) {
            Some(addr) => *addr,
            None => return, // unknown node, nothing to reroute
        };

        // Find all routes whose next-hop is the failed peer
        let affected: Vec<u64> = self
            .routing_table
            .all_routes()
            .into_iter()
            .filter(|(_, entry)| entry.next_hop == failed_addr)
            .map(|(dest_id, _)| dest_id)
            .collect();

        if affected.is_empty() {
            return;
        }

        // Find an alternate. Try the proximity graph first (topology-aware,
        // picks the nearest node that can reach each destination), then
        // fall back to any direct peer.
        let alt_addr = self
            .find_graph_alternate(failed_node_id, &affected)
            .or_else(|| {
                self.peer_addrs
                    .iter()
                    .find(|e| *e.key() != failed_node_id)
                    .map(|e| *e.value())
            });

        let alt_addr = match alt_addr {
            Some(a) => a,
            None => return, // no alternates
        };

        // Reroute each affected destination through the alternate
        for dest_id in &affected {
            self.saved_routes.insert(
                *dest_id,
                SavedRoute {
                    next_hop: failed_addr,
                    alternate: alt_addr,
                },
            );
            self.routing_table.add_route(*dest_id, alt_addr);
        }

        self.reroute_count.fetch_add(1, Ordering::Relaxed);

        tracing::info!(
            failed_node = format!("{:#x}", failed_node_id),
            affected_routes = affected.len(),
            alternate = %alt_addr,
            "auto-rerouted {} routes away from failed peer",
            affected.len()
        );
    }

    /// Called when the failure detector marks a peer as recovered.
    ///
    /// Find an alternate via the proximity graph.
    ///
    /// For each affected destination, queries `path_to(dest)`. If a path
    /// exists that doesn't go through the failed node, the first hop of
    /// that path becomes the alternate. Falls back to the best-scored
    /// node if no path is found.
    ///
    /// Returns the address of the best alternate, or None if the graph
    /// has no suggestions.
    fn find_graph_alternate(
        &self,
        failed_node_id: u64,
        affected_dests: &[u64],
    ) -> Option<SocketAddr> {
        let graph = self.proximity_graph.as_ref()?;

        for dest_id in affected_dests {
            let dest_graph_id = node_id_to_graph_id(*dest_id);

            // Try path_to — returns a Vec<NodeId> from self to dest
            if let Some(path) = graph.path_to(&dest_graph_id) {
                // path[0] is self, path[1] is the first hop
                if path.len() >= 2 {
                    let first_hop = graph_id_to_node_id(&path[1]);
                    // Skip if the first hop IS the failed node
                    if first_hop != failed_node_id {
                        if let Some(addr) = self.peer_addrs.get(&first_hop) {
                            return Some(*addr);
                        }
                    }
                    // If path has 3+ hops, try the second hop
                    if path.len() >= 3 {
                        let second_hop = graph_id_to_node_id(&path[2]);
                        if second_hop != failed_node_id {
                            if let Some(addr) = self.peer_addrs.get(&second_hop) {
                                return Some(*addr);
                            }
                        }
                    }
                }
            }
        }

        // Fallback: pick the best-scored node from the graph that we have
        // a direct connection to and that isn't the failed node
        let all_nodes = graph.all_nodes();
        for node in &all_nodes {
            let nid = graph_id_to_node_id(&node.node_id);
            if nid != failed_node_id && nid != 0 {
                if let Some(addr) = self.peer_addrs.get(&nid) {
                    return Some(*addr);
                }
            }
        }

        None
    }

    /// Called when the failure detector marks a peer as recovered.
    ///
    /// Restores original routes that were rerouted when this peer failed.
    /// The direct path is typically better (fewer hops, lower latency)
    /// than the alternate.
    pub fn on_recovery(&self, recovered_node_id: u64) {
        let recovered_addr = match self.peer_addrs.get(&recovered_node_id) {
            Some(addr) => *addr,
            None => return,
        };

        // Find routes that were rerouted away from this peer
        let to_restore: Vec<u64> = self
            .saved_routes
            .iter()
            .filter(|e| e.value().next_hop == recovered_addr)
            .map(|e| *e.key())
            .collect();

        if to_restore.is_empty() {
            return;
        }

        // Restore original routes
        for dest_id in &to_restore {
            self.routing_table.add_route(*dest_id, recovered_addr);
            self.saved_routes.remove(dest_id);
        }

        self.recovery_count.fetch_add(1, Ordering::Relaxed);

        tracing::info!(
            recovered_node = format!("{:#x}", recovered_node_id),
            restored_routes = to_restore.len(),
            "restored {} routes to recovered peer",
            to_restore.len()
        );
    }

    /// Number of active reroutes (routes currently using alternates).
    pub fn active_reroutes(&self) -> usize {
        self.saved_routes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_routing_table() -> Arc<RoutingTable> {
        Arc::new(RoutingTable::new(0x1111))
    }

    #[test]
    fn test_reroute_on_failure() {
        let rt = make_routing_table();
        let peers = Arc::new(DashMap::new());

        let addr_b: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        let addr_c: SocketAddr = "127.0.0.1:3000".parse().unwrap();

        peers.insert(0x2222u64, addr_b);
        peers.insert(0x3333u64, addr_c);

        // Route to 0x4444 goes through B
        rt.add_route(0x4444, addr_b);

        let policy = ReroutePolicy::new(rt.clone(), peers);

        // B fails
        policy.on_failure(0x2222);

        // Route should now go through C
        let next_hop = rt.lookup(0x4444).unwrap();
        assert_eq!(next_hop, addr_c, "should reroute to C");
        assert_eq!(policy.active_reroutes(), 1);
        assert_eq!(policy.reroute_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_recovery_restores_original() {
        let rt = make_routing_table();
        let peers = Arc::new(DashMap::new());

        let addr_b: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        let addr_c: SocketAddr = "127.0.0.1:3000".parse().unwrap();

        peers.insert(0x2222u64, addr_b);
        peers.insert(0x3333u64, addr_c);

        rt.add_route(0x4444, addr_b);

        let policy = ReroutePolicy::new(rt.clone(), peers);

        // B fails → reroute to C
        policy.on_failure(0x2222);
        assert_eq!(rt.lookup(0x4444).unwrap(), addr_c);

        // B recovers → restore to B
        policy.on_recovery(0x2222);
        assert_eq!(rt.lookup(0x4444).unwrap(), addr_b);
        assert_eq!(policy.active_reroutes(), 0);
        assert_eq!(policy.recovery_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_no_alternate_does_nothing() {
        let rt = make_routing_table();
        let peers = Arc::new(DashMap::new());

        let addr_b: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        peers.insert(0x2222u64, addr_b);

        rt.add_route(0x4444, addr_b);

        let policy = ReroutePolicy::new(rt.clone(), peers);

        // B fails but there's no alternate
        policy.on_failure(0x2222);

        // Route unchanged (still points to B — no better option)
        assert_eq!(rt.lookup(0x4444).unwrap(), addr_b);
        assert_eq!(policy.active_reroutes(), 0);
    }

    #[test]
    fn test_multiple_routes_through_failed_peer() {
        let rt = make_routing_table();
        let peers = Arc::new(DashMap::new());

        let addr_b: SocketAddr = "127.0.0.1:2000".parse().unwrap();
        let addr_c: SocketAddr = "127.0.0.1:3000".parse().unwrap();

        peers.insert(0x2222u64, addr_b);
        peers.insert(0x3333u64, addr_c);

        // Two routes through B
        rt.add_route(0x4444, addr_b);
        rt.add_route(0x5555, addr_b);
        // One route through C (unaffected)
        rt.add_route(0x6666, addr_c);

        let policy = ReroutePolicy::new(rt.clone(), peers);

        policy.on_failure(0x2222);

        // Both B routes should be rerouted to C
        assert_eq!(rt.lookup(0x4444).unwrap(), addr_c);
        assert_eq!(rt.lookup(0x5555).unwrap(), addr_c);
        // C route unchanged
        assert_eq!(rt.lookup(0x6666).unwrap(), addr_c);
        assert_eq!(policy.active_reroutes(), 2);
    }
}
