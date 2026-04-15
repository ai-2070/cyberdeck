//! Origin binding for NLTP packets.
//!
//! The `OriginStamp` holds the entity identity and provides the cached
//! `origin_hash` value that gets written into every outbound packet header.
//! This is computed once at session creation — zero per-packet crypto.

use super::entity::{EntityId, EntityKeypair};

/// Cached origin binding for packet building.
///
/// Created once per session from the entity keypair. The `origin_hash`
/// is a truncated BLAKE2s of the entity's public key, suitable for
/// wire-speed filtering by forwarding nodes.
#[derive(Debug, Clone)]
pub struct OriginStamp {
    entity_id: EntityId,
    origin_hash: u32,
    node_id: u64,
}

impl OriginStamp {
    /// Create an origin stamp from an entity keypair.
    pub fn from_keypair(keypair: &EntityKeypair) -> Self {
        Self {
            entity_id: keypair.entity_id().clone(),
            origin_hash: keypair.origin_hash(),
            node_id: keypair.node_id(),
        }
    }

    /// Create an origin stamp from an entity ID (no signing capability).
    pub fn from_entity_id(entity_id: EntityId) -> Self {
        let origin_hash = entity_id.origin_hash();
        let node_id = entity_id.node_id();
        Self {
            entity_id,
            origin_hash,
            node_id,
        }
    }

    /// Get the origin hash for the NLTP header (4 bytes).
    #[inline]
    pub fn origin_hash(&self) -> u32 {
        self.origin_hash
    }

    /// Get the node ID for swarm/routing (8 bytes).
    #[inline]
    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    /// Get the full entity identity.
    #[inline]
    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_stamp_from_keypair() {
        let kp = EntityKeypair::generate();
        let stamp = OriginStamp::from_keypair(&kp);

        assert_eq!(stamp.origin_hash(), kp.origin_hash());
        assert_eq!(stamp.node_id(), kp.node_id());
        assert_eq!(stamp.entity_id(), kp.entity_id());
    }

    #[test]
    fn test_origin_stamp_from_entity_id() {
        let kp = EntityKeypair::generate();
        let stamp = OriginStamp::from_entity_id(kp.entity_id().clone());

        assert_eq!(stamp.origin_hash(), kp.origin_hash());
        assert_eq!(stamp.node_id(), kp.node_id());
    }
}
