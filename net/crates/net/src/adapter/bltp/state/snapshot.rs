//! State snapshots for daemon migration and catchup.
//!
//! A snapshot captures an entity's accumulated state at a point in the
//! causal chain. New nodes receive the snapshot + replay events after it,
//! avoiding full log replay.

use bytes::{Buf, Bytes};

use super::causal::CausalLink;
use super::horizon::ObservedHorizon;
use crate::adapter::bltp::identity::EntityId;

/// A serializable state snapshot at a point in the causal chain.
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// Entity this snapshot belongs to.
    pub entity_id: EntityId,
    /// Sequence number this snapshot is valid through.
    pub through_seq: u64,
    /// CausalLink at the snapshot point (for chain verification).
    pub chain_link: CausalLink,
    /// Serialized daemon state (opaque bytes).
    pub state: Bytes,
    /// The entity's observed horizon at snapshot time.
    pub horizon: ObservedHorizon,
    /// Timestamp when snapshot was taken (unix nanos).
    pub created_at: u64,
}

impl StateSnapshot {
    /// Create a new snapshot.
    pub fn new(
        entity_id: EntityId,
        chain_link: CausalLink,
        state: Bytes,
        horizon: ObservedHorizon,
    ) -> Self {
        Self {
            entity_id,
            through_seq: chain_link.sequence,
            chain_link,
            state,
            horizon,
            created_at: current_timestamp(),
        }
    }

    /// Serialize to bytes for transfer.
    ///
    /// Wire format:
    /// ```text
    /// entity_id:     32 bytes
    /// through_seq:    8 bytes
    /// chain_link:    24 bytes
    /// created_at:     8 bytes
    /// state_len:      4 bytes (u32)
    /// state:          state_len bytes
    /// ```
    ///
    /// Note: horizon is not serialized in the compact format — it is
    /// transferred separately or reconstructed from the event log.
    pub fn to_bytes(&self) -> Vec<u8> {
        let header_size = 32 + 8 + 24 + 8 + 4;
        let mut buf = Vec::with_capacity(header_size + self.state.len());

        buf.extend_from_slice(self.entity_id.as_bytes());
        buf.extend_from_slice(&self.through_seq.to_le_bytes());
        buf.extend_from_slice(&self.chain_link.to_bytes());
        buf.extend_from_slice(&self.created_at.to_le_bytes());
        buf.extend_from_slice(&(self.state.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.state);

        buf
    }

    /// Deserialize from bytes.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        let header_size = 32 + 8 + 24 + 8 + 4;
        if data.len() < header_size {
            return None;
        }

        let mut cursor = data;

        let mut entity_bytes = [0u8; 32];
        cursor.copy_to_slice(&mut entity_bytes);
        let entity_id = EntityId::from_bytes(entity_bytes);

        let through_seq = cursor.get_u64_le();

        let mut link_bytes = [0u8; 24];
        cursor.copy_to_slice(&mut link_bytes);
        let chain_link = CausalLink::from_bytes(&link_bytes)?;

        let created_at = cursor.get_u64_le();

        let state_len = cursor.get_u32_le() as usize;
        if cursor.remaining() < state_len {
            return None;
        }

        let state = Bytes::copy_from_slice(&cursor[..state_len]);

        Some(Self {
            entity_id,
            through_seq,
            chain_link,
            state,
            horizon: ObservedHorizon::new(), // reconstructed separately
            created_at,
        })
    }

    /// Compact header size (excluding state payload).
    pub const HEADER_SIZE: usize = 32 + 8 + 24 + 8 + 4; // 76 bytes

    /// Age of this snapshot in seconds.
    pub fn age_secs(&self) -> u64 {
        let now = current_timestamp();
        (now.saturating_sub(self.created_at)) / 1_000_000_000
    }
}

/// Snapshot store — holds the latest snapshot per entity.
pub struct SnapshotStore {
    snapshots: dashmap::DashMap<u32, StateSnapshot>,
}

impl SnapshotStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            snapshots: dashmap::DashMap::new(),
        }
    }

    /// Store a snapshot (replaces any existing snapshot for this entity).
    pub fn store(&self, snapshot: StateSnapshot) {
        let origin_hash = snapshot.entity_id.origin_hash();
        self.snapshots.insert(origin_hash, snapshot);
    }

    /// Get the latest snapshot for an entity.
    pub fn get(
        &self,
        origin_hash: u32,
    ) -> Option<dashmap::mapref::one::Ref<'_, u32, StateSnapshot>> {
        self.snapshots.get(&origin_hash)
    }

    /// Remove a snapshot.
    pub fn remove(&self, origin_hash: u32) -> Option<StateSnapshot> {
        self.snapshots.remove(&origin_hash).map(|(_, s)| s)
    }

    /// Number of stored snapshots.
    pub fn count(&self) -> usize {
        self.snapshots.len()
    }
}

impl Default for SnapshotStore {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SnapshotStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotStore")
            .field("snapshots", &self.snapshots.len())
            .finish()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::bltp::identity::EntityKeypair;
    use crate::adapter::bltp::state::causal::CausalChainBuilder;

    #[test]
    fn test_snapshot_roundtrip() {
        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());

        // Build a short chain
        for i in 0..5 {
            builder.append(Bytes::from(format!("event-{}", i)), 0);
        }

        let state_data = Bytes::from_static(b"serialized daemon state here");
        let snapshot = StateSnapshot::new(
            entity_id.clone(),
            *builder.head(),
            state_data.clone(),
            ObservedHorizon::new(),
        );

        assert_eq!(snapshot.through_seq, 5);

        let bytes = snapshot.to_bytes();
        let parsed = StateSnapshot::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.entity_id, entity_id);
        assert_eq!(parsed.through_seq, 5);
        assert_eq!(parsed.chain_link, *builder.head());
        assert_eq!(parsed.state, state_data);
    }

    #[test]
    fn test_snapshot_store() {
        let store = SnapshotStore::new();

        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let origin_hash = kp.origin_hash();
        let link = CausalLink::genesis(origin_hash, 0);

        let snapshot = StateSnapshot::new(
            entity_id,
            link,
            Bytes::from_static(b"state"),
            ObservedHorizon::new(),
        );

        store.store(snapshot);
        assert_eq!(store.count(), 1);

        let retrieved = store.get(origin_hash).unwrap();
        assert_eq!(retrieved.state, Bytes::from_static(b"state"));
    }

    #[test]
    fn test_snapshot_replaces_older() {
        let store = SnapshotStore::new();
        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let origin_hash = kp.origin_hash();

        let snap1 = StateSnapshot::new(
            entity_id.clone(),
            CausalLink::genesis(origin_hash, 0),
            Bytes::from_static(b"state-v1"),
            ObservedHorizon::new(),
        );
        store.store(snap1);

        let mut builder = CausalChainBuilder::new(origin_hash);
        builder.append(Bytes::from_static(b"e1"), 0);

        let snap2 = StateSnapshot::new(
            entity_id,
            *builder.head(),
            Bytes::from_static(b"state-v2"),
            ObservedHorizon::new(),
        );
        store.store(snap2);

        assert_eq!(store.count(), 1);
        let retrieved = store.get(origin_hash).unwrap();
        assert_eq!(retrieved.state, Bytes::from_static(b"state-v2"));
        assert_eq!(retrieved.through_seq, 1);
    }

    #[test]
    fn test_from_bytes_too_short() {
        assert!(StateSnapshot::from_bytes(&[0u8; 10]).is_none());
    }
}
