//! State snapshots for daemon migration and catchup.
//!
//! A snapshot captures an entity's accumulated state at a point in the
//! causal chain. New nodes receive the snapshot + replay events after it,
//! avoiding full log replay.
//!
//! # Wire versioning
//!
//! v0 was the pre-identity-migration layout: a bare header + state
//! payload, no hint of which version the decoder is looking at. v1
//! (introduced by `DAEMON_IDENTITY_MIGRATION_PLAN.md` + shared with
//! `DAEMON_CHANNEL_REBIND_PLAN.md`) prepends a 4-byte magic +
//! version byte so readers can unambiguously distinguish the two
//! and so future bumps can introduce new trailing fields without a
//! guessing game. v1 readers still decode v0 bytes for rolling-
//! upgrade compatibility: v0 content is surfaced with empty
//! `bindings_bytes` + `identity_envelope: None`, the same defaults
//! a fresh v1 snapshot with no extras would produce. Writers always
//! emit v1.

use bytes::{Buf, Bytes};

use super::causal::CausalLink;
use super::horizon::ObservedHorizon;
use crate::adapter::net::identity::{EntityId, IdentityEnvelope, IDENTITY_ENVELOPE_SIZE};

/// 4-byte magic prefix for v1 snapshots. v0's first 4 bytes are the
/// first 32 bytes of an `EntityId` (arbitrary); this ASCII marker is
/// a ~1/2^32 collision with any given v0 snapshot and lets the
/// decoder branch unambiguously. `CDS` = *Compute-Daemon Snapshot*;
/// the `1` is the version digit, bumped when an on-wire field
/// changes shape.
const V1_MAGIC: [u8; 4] = *b"CDS1";

/// Current snapshot wire version. The plan reserves this byte for
/// future bumps — unknown versions are rejected up-front by
/// [`StateSnapshot::from_bytes`].
pub const SNAPSHOT_VERSION: u8 = 1;

/// A serializable state snapshot at a point in the causal chain.
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// Wire version this snapshot was produced under. Writers
    /// always stamp [`SNAPSHOT_VERSION`]; readers accept v0 bytes
    /// by surfacing them with the v1 defaults populated.
    pub version: u8,
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
    /// Opaque wire slot for channel-re-bind metadata populated by
    /// [`DAEMON_CHANNEL_REBIND_PLAN.md`](../../../../docs/DAEMON_CHANNEL_REBIND_PLAN.md).
    /// Stage 1 of the identity-migration plan lands this as an
    /// always-empty `Vec` so the wire format is forward-compatible
    /// with the channel-re-bind work even though the typed
    /// `DaemonBindings` decoder isn't yet present. Plan #1 will
    /// decode these bytes into its own struct at restore time.
    pub bindings_bytes: Vec<u8>,
    /// Encrypted ed25519 seed + attestation for cross-node identity
    /// transport. Populated by
    /// [`DAEMON_IDENTITY_MIGRATION_PLAN.md`](../../../../docs/DAEMON_IDENTITY_MIGRATION_PLAN.md)
    /// Stage 3; Stage 1 always emits `None`. A `None` envelope on
    /// restore means "public-identity migration" — the target gets
    /// a read-only keypair that can still serve `entity_id` /
    /// `origin_hash` queries but refuses to sign anything new.
    pub identity_envelope: Option<IdentityEnvelope>,
}

impl StateSnapshot {
    /// Create a new snapshot stamped with the current wire version
    /// and empty v1 extension fields.
    pub fn new(
        entity_id: EntityId,
        chain_link: CausalLink,
        state: Bytes,
        horizon: ObservedHorizon,
    ) -> Self {
        Self {
            version: SNAPSHOT_VERSION,
            entity_id,
            through_seq: chain_link.sequence,
            chain_link,
            state,
            horizon,
            created_at: current_timestamp(),
            bindings_bytes: Vec::new(),
            identity_envelope: None,
        }
    }

    /// Serialize to bytes for transfer.
    ///
    /// # v1 wire format
    ///
    /// ```text
    /// magic:             4 bytes (b"CDS1")
    /// version:           1 byte  (SNAPSHOT_VERSION)
    /// entity_id:        32 bytes
    /// through_seq:       8 bytes
    /// chain_link:       24 bytes
    /// created_at:        8 bytes
    /// state_len:         4 bytes (u32)
    /// state:             state_len bytes
    /// bindings_len:      4 bytes (u32)
    /// bindings:          bindings_len bytes (opaque; see `bindings_bytes`)
    /// envelope_flag:     1 byte (0 = none, 1 = present)
    /// [envelope:       208 bytes]  (if envelope_flag == 1)
    /// ```
    ///
    /// Horizon is not serialized in the compact format — it is
    /// transferred separately or reconstructed from the event log.
    pub fn to_bytes(&self) -> Vec<u8> {
        let envelope_bytes_len = if self.identity_envelope.is_some() {
            IDENTITY_ENVELOPE_SIZE
        } else {
            0
        };
        let header_size = V1_MAGIC.len() + 1 + 32 + 8 + 24 + 8 + 4;
        let trailer_size = 4 + self.bindings_bytes.len() + 1 + envelope_bytes_len;
        let mut buf = Vec::with_capacity(header_size + self.state.len() + trailer_size);

        buf.extend_from_slice(&V1_MAGIC);
        buf.push(SNAPSHOT_VERSION);
        buf.extend_from_slice(self.entity_id.as_bytes());
        buf.extend_from_slice(&self.through_seq.to_le_bytes());
        buf.extend_from_slice(&self.chain_link.to_bytes());
        buf.extend_from_slice(&self.created_at.to_le_bytes());
        let state_len = u32::try_from(self.state.len()).expect("state snapshot exceeds 4 GiB");
        buf.extend_from_slice(&state_len.to_le_bytes());
        buf.extend_from_slice(&self.state);

        let bindings_len = u32::try_from(self.bindings_bytes.len())
            .expect("bindings_bytes exceeds 4 GiB — this is almost certainly a bug");
        buf.extend_from_slice(&bindings_len.to_le_bytes());
        buf.extend_from_slice(&self.bindings_bytes);

        match &self.identity_envelope {
            None => buf.push(0),
            Some(env) => {
                buf.push(1);
                buf.extend_from_slice(&env.to_bytes());
            }
        }

        buf
    }

    /// Deserialize from bytes.
    ///
    /// Accepts both v1 (current) and v0 (pre-identity-migration)
    /// layouts. v0 bytes surface with defaulted `bindings_bytes` +
    /// `identity_envelope` so a rolling upgrade between pre- and
    /// post-v1 nodes doesn't stall.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() >= V1_MAGIC.len() && data[..V1_MAGIC.len()] == V1_MAGIC {
            Self::from_bytes_v1(&data[V1_MAGIC.len()..])
        } else {
            Self::from_bytes_v0(data)
        }
    }

    fn from_bytes_v1(data: &[u8]) -> Option<Self> {
        let mut cursor = data;
        if cursor.remaining() < 1 {
            return None;
        }
        let version = cursor.get_u8();
        if version != SNAPSHOT_VERSION {
            // A future v2 reader can match on this byte; today we
            // reject cleanly instead of mis-parsing.
            return None;
        }

        // Core header — same layout as v0 past this point, except
        // the reader branches to the v1 trailer at the end.
        let header_remaining = 32 + 8 + 24 + 8 + 4;
        if cursor.remaining() < header_remaining {
            return None;
        }

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
        cursor = &cursor[state_len..];

        // v1 trailer — bindings (length-prefixed opaque bytes) then
        // optional envelope.
        if cursor.remaining() < 4 {
            return None;
        }
        let bindings_len = cursor.get_u32_le() as usize;
        if cursor.remaining() < bindings_len {
            return None;
        }
        let bindings_bytes = cursor[..bindings_len].to_vec();
        cursor = &cursor[bindings_len..];

        if cursor.remaining() < 1 {
            return None;
        }
        let envelope_flag = cursor.get_u8();
        let identity_envelope = match envelope_flag {
            0 => None,
            1 => {
                if cursor.remaining() < IDENTITY_ENVELOPE_SIZE {
                    return None;
                }
                let env = IdentityEnvelope::from_bytes(&cursor[..IDENTITY_ENVELOPE_SIZE])?;
                cursor = &cursor[IDENTITY_ENVELOPE_SIZE..];
                Some(env)
            }
            _ => return None,
        };

        // Strict length match — trailing garbage after the envelope
        // is a framing bug on the source, not forward-compat.
        if !cursor.is_empty() {
            return None;
        }

        // Consistency checks, same set as v0.
        if chain_link.sequence != through_seq {
            return None;
        }
        if chain_link.origin_hash != entity_id.origin_hash() {
            return None;
        }

        Some(Self {
            version: SNAPSHOT_VERSION,
            entity_id,
            through_seq,
            chain_link,
            state,
            horizon: ObservedHorizon::new(),
            created_at,
            bindings_bytes,
            identity_envelope,
        })
    }

    fn from_bytes_v0(data: &[u8]) -> Option<Self> {
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

        // Validate internal consistency
        if chain_link.sequence != through_seq {
            return None;
        }
        if chain_link.origin_hash != entity_id.origin_hash() {
            return None;
        }

        Some(Self {
            // Surface the snapshot as v1 for downstream uniformity;
            // writers always emit v1, so a fresh round-trip is a
            // v1 → v1 operation after this read.
            version: SNAPSHOT_VERSION,
            entity_id,
            through_seq,
            chain_link,
            state,
            horizon: ObservedHorizon::new(), // reconstructed separately
            created_at,
            bindings_bytes: Vec::new(),
            identity_envelope: None,
        })
    }

    /// Compact header size (excluding state payload). Kept for the
    /// v0 compatibility path — v1 writers allocate from
    /// [`Self::to_bytes`]'s own sizing math.
    pub const HEADER_SIZE: usize = 32 + 8 + 24 + 8 + 4; // 76 bytes

    /// Age of this snapshot in seconds.
    pub fn age_secs(&self) -> u64 {
        let now = current_timestamp();
        (now.saturating_sub(self.created_at)) / 1_000_000_000
    }
}

/// Snapshot store — holds the latest snapshot per entity.
///
/// Keyed by full EntityId (32 bytes) to avoid origin_hash collisions.
pub struct SnapshotStore {
    snapshots: dashmap::DashMap<[u8; 32], StateSnapshot>,
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
        let key = *snapshot.entity_id.as_bytes();
        self.snapshots.insert(key, snapshot);
    }

    /// Get the latest snapshot for an entity.
    pub fn get(
        &self,
        entity_id: &EntityId,
    ) -> Option<dashmap::mapref::one::Ref<'_, [u8; 32], StateSnapshot>> {
        self.snapshots.get(entity_id.as_bytes())
    }

    /// Remove a snapshot.
    pub fn remove(&self, entity_id: &EntityId) -> Option<StateSnapshot> {
        self.snapshots.remove(entity_id.as_bytes()).map(|(_, s)| s)
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

use crate::adapter::net::current_timestamp;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::identity::EntityKeypair;
    use crate::adapter::net::state::causal::CausalChainBuilder;

    #[test]
    fn test_snapshot_roundtrip() {
        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());

        // Build a short chain
        for i in 0..5 {
            builder
                .append(Bytes::from(format!("event-{}", i)), 0)
                .unwrap();
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
            entity_id.clone(),
            link,
            Bytes::from_static(b"state"),
            ObservedHorizon::new(),
        );

        store.store(snapshot);
        assert_eq!(store.count(), 1);

        let retrieved = store.get(&entity_id).unwrap();
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
        builder.append(Bytes::from_static(b"e1"), 0).unwrap();

        let snap2 = StateSnapshot::new(
            entity_id.clone(),
            *builder.head(),
            Bytes::from_static(b"state-v2"),
            ObservedHorizon::new(),
        );
        store.store(snap2);

        assert_eq!(store.count(), 1);
        let retrieved = store.get(&entity_id).unwrap();
        assert_eq!(retrieved.state, Bytes::from_static(b"state-v2"));
        assert_eq!(retrieved.through_seq, 1);
    }

    #[test]
    fn test_from_bytes_too_short() {
        assert!(StateSnapshot::from_bytes(&[0u8; 10]).is_none());
    }

    // ---- Regression tests for Cubic AI findings ----

    #[test]
    fn test_regression_from_bytes_rejects_sequence_mismatch() {
        // Regression: from_bytes accepted snapshots where
        // chain_link.sequence != through_seq.
        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());
        builder.append(Bytes::from_static(b"e1"), 0).unwrap();

        let snapshot = StateSnapshot::new(
            entity_id,
            *builder.head(),
            Bytes::from_static(b"state"),
            ObservedHorizon::new(),
        );
        let mut bytes = snapshot.to_bytes();

        // v1 layout: 4 magic + 1 version + 32 entity_id = 37 bytes
        // before through_seq starts.
        bytes[37] = 0xFF;

        assert!(
            StateSnapshot::from_bytes(&bytes).is_none(),
            "from_bytes must reject snapshot with sequence mismatch"
        );
    }

    // ---- v1 wire format tests ----

    #[test]
    fn v1_roundtrip_preserves_bindings_and_envelope() {
        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());
        builder.append(Bytes::from_static(b"e"), 0).unwrap();

        let env = IdentityEnvelope {
            target_static_pub: [0x11; 32],
            sealed_seed: [0x22; 80],
            signer_pub: [0x33; 32],
            signature: [0x44; 64],
        };

        let mut snapshot = StateSnapshot::new(
            entity_id,
            *builder.head(),
            Bytes::from_static(b"state"),
            ObservedHorizon::new(),
        );
        snapshot.bindings_bytes = vec![0x55; 42];
        snapshot.identity_envelope = Some(env.clone());

        let bytes = snapshot.to_bytes();
        // Writers always emit v1 — the first 4 bytes are the magic.
        assert_eq!(&bytes[..4], b"CDS1");
        assert_eq!(bytes[4], SNAPSHOT_VERSION);

        let parsed = StateSnapshot::from_bytes(&bytes).expect("v1 round-trip");
        assert_eq!(parsed.version, SNAPSHOT_VERSION);
        assert_eq!(parsed.bindings_bytes, vec![0x55; 42]);
        assert_eq!(parsed.identity_envelope, Some(env));
        assert_eq!(parsed.state, Bytes::from_static(b"state"));
    }

    #[test]
    fn v0_bytes_decode_as_v1_with_empty_extras() {
        // Construct the exact v0 wire layout from first principles:
        // pre-magic writers just concatenated the header + state.
        // A v1 reader must still decode these successfully so a
        // rolling upgrade between pre- and post-v1 nodes doesn't
        // stall.
        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());
        builder.append(Bytes::from_static(b"e"), 0).unwrap();

        let head = *builder.head();
        let state = Bytes::from_static(b"legacy-state");

        let mut v0 = Vec::new();
        v0.extend_from_slice(entity_id.as_bytes());
        v0.extend_from_slice(&head.sequence.to_le_bytes());
        v0.extend_from_slice(&head.to_bytes());
        v0.extend_from_slice(&0u64.to_le_bytes()); // created_at
        v0.extend_from_slice(&(state.len() as u32).to_le_bytes());
        v0.extend_from_slice(&state);

        let parsed = StateSnapshot::from_bytes(&v0)
            .expect("v1 reader must accept v0 bytes for rolling upgrade");
        assert_eq!(parsed.version, SNAPSHOT_VERSION);
        assert_eq!(parsed.entity_id, entity_id);
        assert_eq!(parsed.state, state);
        assert!(parsed.bindings_bytes.is_empty());
        assert!(parsed.identity_envelope.is_none());
    }

    #[test]
    fn v1_rejects_trailing_garbage() {
        let kp = EntityKeypair::generate();
        let snapshot = StateSnapshot::new(
            kp.entity_id().clone(),
            CausalLink::genesis(kp.origin_hash(), 0),
            Bytes::from_static(b"s"),
            ObservedHorizon::new(),
        );
        let mut bytes = snapshot.to_bytes();
        bytes.push(0xFF);
        assert!(
            StateSnapshot::from_bytes(&bytes).is_none(),
            "trailing byte after a v1 snapshot must be rejected — a short \
             snapshot plus junk is indistinguishable from a framing bug",
        );
    }

    #[test]
    fn v1_rejects_unknown_version_byte() {
        let kp = EntityKeypair::generate();
        let snapshot = StateSnapshot::new(
            kp.entity_id().clone(),
            CausalLink::genesis(kp.origin_hash(), 0),
            Bytes::from_static(b"s"),
            ObservedHorizon::new(),
        );
        let mut bytes = snapshot.to_bytes();
        // 4 magic bytes then the version. Flip to an unknown future
        // version — decoder must refuse rather than mis-parse.
        bytes[4] = 0xFE;
        assert!(StateSnapshot::from_bytes(&bytes).is_none());
    }

    #[test]
    fn v1_without_envelope_uses_single_zero_byte_trailer() {
        // Regression: a `None` envelope must occupy exactly one byte
        // on the wire — a stray extra byte would shift every trailing
        // field and poison the round-trip in ways the strict
        // length-match catches only coincidentally.
        let kp = EntityKeypair::generate();
        let snapshot = StateSnapshot::new(
            kp.entity_id().clone(),
            CausalLink::genesis(kp.origin_hash(), 0),
            Bytes::from_static(b"s"),
            ObservedHorizon::new(),
        );
        let bytes = snapshot.to_bytes();
        assert_eq!(
            *bytes.last().expect("at least one byte"),
            0,
            "envelope_flag must be zero when None",
        );
    }
}
