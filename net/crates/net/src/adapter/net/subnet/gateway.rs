//! Subnet gateway — the causal membrane at subnet boundaries.
//!
//! A gateway node sits at the boundary between subnets and enforces
//! visibility policy. It reads header fields (no decryption) to make
//! forward/drop decisions. Encrypted payloads pass through untouched.

use dashmap::DashMap;

use super::id::SubnetId;
use crate::adapter::net::channel::{ChannelConfigRegistry, Visibility};

/// Reason a packet was dropped at a gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    /// Channel is SubnetLocal — never crosses boundaries.
    SubnetLocal,
    /// Channel is ParentVisible but destination is not an ancestor.
    NotAncestor,
    /// Channel is Exported but destination is not in the export table.
    NotExported,
    /// Packet's subnet_id doesn't match any known subnet.
    UnknownSubnet,
    /// TTL expired.
    TtlExpired,
}

impl std::fmt::Display for DropReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SubnetLocal => write!(f, "channel is subnet-local"),
            Self::NotAncestor => write!(f, "destination is not ancestor of source"),
            Self::NotExported => write!(f, "channel not exported to destination subnet"),
            Self::UnknownSubnet => write!(f, "unknown subnet"),
            Self::TtlExpired => write!(f, "TTL expired"),
        }
    }
}

/// Gateway forwarding decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForwardDecision {
    /// Packet should be forwarded.
    Forward,
    /// Packet should be dropped.
    Drop(DropReason),
}

/// Subnet gateway that enforces visibility policy at subnet boundaries.
///
/// The gateway reads only header fields — it does not decrypt or modify
/// packet payloads. This is the "causal membrane" that filters traffic
/// between subnets.
pub struct SubnetGateway {
    /// This gateway's subnet.
    local_subnet: SubnetId,
    /// Known peer subnets this gateway bridges to.
    peer_subnets: Vec<SubnetId>,
    /// Export table: channel_hash -> allowed destination subnets.
    /// Only consulted for `Visibility::Exported` channels.
    export_table: DashMap<u16, Vec<SubnetId>>,
    /// Channel config registry for looking up visibility.
    channel_configs: ChannelConfigRegistry,
    /// Gateway stats.
    forwarded: std::sync::atomic::AtomicU64,
    dropped: std::sync::atomic::AtomicU64,
}

impl SubnetGateway {
    /// Create a new gateway for a subnet.
    pub fn new(local_subnet: SubnetId, channel_configs: ChannelConfigRegistry) -> Self {
        Self {
            local_subnet,
            peer_subnets: Vec::new(),
            export_table: DashMap::new(),
            channel_configs,
            forwarded: std::sync::atomic::AtomicU64::new(0),
            dropped: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Add a peer subnet this gateway bridges to.
    pub fn add_peer(&mut self, subnet: SubnetId) {
        if !self.peer_subnets.contains(&subnet) {
            self.peer_subnets.push(subnet);
        }
    }

    /// Export a channel to specific subnets.
    pub fn export_channel(&self, channel_hash: u16, targets: Vec<SubnetId>) {
        self.export_table.insert(channel_hash, targets);
    }

    /// Get this gateway's local subnet.
    #[inline]
    pub fn local_subnet(&self) -> SubnetId {
        self.local_subnet
    }

    /// Make a forwarding decision for a packet crossing this gateway.
    ///
    /// Reads only header fields: `subnet_id`, `channel_hash`, `hop_ttl`, `hop_count`.
    /// No decryption, no payload inspection.
    pub fn should_forward(
        &self,
        source_subnet: SubnetId,
        dest_subnet: SubnetId,
        channel_hash: u16,
        hop_ttl: u8,
        hop_count: u8,
    ) -> ForwardDecision {
        // TTL check
        if hop_ttl > 0 && hop_count >= hop_ttl {
            self.dropped
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return ForwardDecision::Drop(DropReason::TtlExpired);
        }

        // Look up channel visibility
        let visibility = self
            .channel_configs
            .get(channel_hash)
            .map(|c| c.visibility)
            .unwrap_or(Visibility::Global);

        let decision = match visibility {
            Visibility::SubnetLocal => ForwardDecision::Drop(DropReason::SubnetLocal),

            Visibility::ParentVisible => {
                if dest_subnet.is_ancestor_of(source_subnet)
                    || source_subnet.is_ancestor_of(dest_subnet)
                {
                    ForwardDecision::Forward
                } else {
                    ForwardDecision::Drop(DropReason::NotAncestor)
                }
            }

            Visibility::Exported => {
                if let Some(targets) = self.export_table.get(&channel_hash) {
                    if targets
                        .iter()
                        .any(|t| t.is_same_subnet(dest_subnet) || t.is_ancestor_of(dest_subnet))
                    {
                        ForwardDecision::Forward
                    } else {
                        ForwardDecision::Drop(DropReason::NotExported)
                    }
                } else {
                    ForwardDecision::Drop(DropReason::NotExported)
                }
            }

            Visibility::Global => ForwardDecision::Forward,
        };

        match decision {
            ForwardDecision::Forward => {
                self.forwarded
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            ForwardDecision::Drop(_) => {
                self.dropped
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        decision
    }

    /// Get the number of forwarded packets.
    pub fn forwarded_count(&self) -> u64 {
        self.forwarded.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get the number of dropped packets.
    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl std::fmt::Debug for SubnetGateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubnetGateway")
            .field("local_subnet", &self.local_subnet)
            .field("peer_subnets", &self.peer_subnets)
            .field("exports", &self.export_table.len())
            .field("forwarded", &self.forwarded_count())
            .field("dropped", &self.dropped_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::channel::{ChannelConfig, ChannelId};

    use crate::adapter::net::channel::ChannelName;

    fn make_channel(name: &str, vis: Visibility, reg: &ChannelConfigRegistry) -> u16 {
        let id = ChannelId::new(ChannelName::new(name).unwrap());
        let hash = id.hash();
        reg.insert(ChannelConfig::new(id).with_visibility(vis));
        hash
    }

    #[test]
    fn test_global_always_forwards() {
        let reg = ChannelConfigRegistry::new();
        let ch = make_channel("test/global", Visibility::Global, &reg);
        let gw = SubnetGateway::new(SubnetId::new(&[1]), reg);

        let decision = gw.should_forward(SubnetId::new(&[1, 1]), SubnetId::new(&[2, 1]), ch, 0, 0);
        assert_eq!(decision, ForwardDecision::Forward);
    }

    #[test]
    fn test_subnet_local_always_drops() {
        let reg = ChannelConfigRegistry::new();
        let ch = make_channel("test/local", Visibility::SubnetLocal, &reg);
        let gw = SubnetGateway::new(SubnetId::new(&[1]), reg);

        let decision = gw.should_forward(SubnetId::new(&[1, 1]), SubnetId::new(&[1, 2]), ch, 0, 0);
        assert_eq!(decision, ForwardDecision::Drop(DropReason::SubnetLocal));
    }

    #[test]
    fn test_parent_visible_allows_ancestor() {
        let reg = ChannelConfigRegistry::new();
        let ch = make_channel("test/parent-vis", Visibility::ParentVisible, &reg);
        let gw = SubnetGateway::new(SubnetId::new(&[1]), reg);

        // Child to parent — allowed
        let decision = gw.should_forward(SubnetId::new(&[1, 2]), SubnetId::new(&[1]), ch, 0, 0);
        assert_eq!(decision, ForwardDecision::Forward);

        // Sibling to sibling — not allowed
        let decision = gw.should_forward(SubnetId::new(&[1, 2]), SubnetId::new(&[1, 3]), ch, 0, 0);
        assert_eq!(decision, ForwardDecision::Drop(DropReason::NotAncestor));
    }

    #[test]
    fn test_exported_channel() {
        let reg = ChannelConfigRegistry::new();
        let ch = make_channel("test/exported", Visibility::Exported, &reg);
        let gw = SubnetGateway::new(SubnetId::new(&[1]), reg);

        gw.export_channel(ch, vec![SubnetId::new(&[2])]);

        // Forward to exported target
        let decision = gw.should_forward(SubnetId::new(&[1]), SubnetId::new(&[2]), ch, 0, 0);
        assert_eq!(decision, ForwardDecision::Forward);

        // Drop to non-exported target
        let decision = gw.should_forward(SubnetId::new(&[1]), SubnetId::new(&[3]), ch, 0, 0);
        assert_eq!(decision, ForwardDecision::Drop(DropReason::NotExported));
    }

    #[test]
    fn test_ttl_expired() {
        let reg = ChannelConfigRegistry::new();
        let ch = make_channel("test/ttl", Visibility::Global, &reg);
        let gw = SubnetGateway::new(SubnetId::new(&[1]), reg);

        let decision = gw.should_forward(
            SubnetId::new(&[1]),
            SubnetId::new(&[2]),
            ch,
            4, // ttl = 4
            4, // hop_count = 4 (expired)
        );
        assert_eq!(decision, ForwardDecision::Drop(DropReason::TtlExpired));
    }

    #[test]
    fn test_unknown_channel_defaults_global() {
        let reg = ChannelConfigRegistry::new();
        let gw = SubnetGateway::new(SubnetId::new(&[1]), reg);

        let decision = gw.should_forward(SubnetId::new(&[1]), SubnetId::new(&[2]), 0x9999, 0, 0);
        assert_eq!(decision, ForwardDecision::Forward);
    }

    #[test]
    fn test_stats() {
        let reg = ChannelConfigRegistry::new();
        let ch_global = make_channel("test/stats-global", Visibility::Global, &reg);
        let ch_local = make_channel("test/stats-local", Visibility::SubnetLocal, &reg);
        let gw = SubnetGateway::new(SubnetId::new(&[1]), reg);

        gw.should_forward(SubnetId::new(&[1]), SubnetId::new(&[2]), ch_global, 0, 0);
        gw.should_forward(SubnetId::new(&[1]), SubnetId::new(&[2]), ch_local, 0, 0);
        gw.should_forward(SubnetId::new(&[1]), SubnetId::new(&[2]), ch_global, 0, 0);

        assert_eq!(gw.forwarded_count(), 2);
        assert_eq!(gw.dropped_count(), 1);
    }
}
