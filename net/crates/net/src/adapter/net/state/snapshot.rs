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
use crate::adapter::net::identity::{
    EntityId, EntityKeypair, IdentityEnvelope, IDENTITY_ENVELOPE_SIZE,
};

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
    /// Runtime-only: payload bytes of the event at
    /// `chain_link.sequence`. Required by
    /// [`super::log::EntityLog::from_snapshot`] to validate the
    /// next event's `parent_hash` after restore (the chain validator
    /// computes `xxh3(prev_link_bytes ++ prev_payload)`).
    ///
    /// **Not serialized** — `to_bytes` / `from_bytes` skip this
    /// field, so the wire format is unchanged. Callers reconstructing
    /// a snapshot from the log have the head event in hand and
    /// populate this via `with_head_payload` before passing the
    /// snapshot to restore. Cross-node migration carries the head
    /// event through the migration message itself, paired with the
    /// snapshot bytes.
    ///
    /// Default for snapshots deserialized from wire bytes is
    /// `Bytes::new()`; callers must populate it from the head event
    /// before `EntityLog::from_snapshot` can validate subsequent
    /// events.
    pub head_payload: Bytes,
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
            head_payload: Bytes::new(),
        }
    }

    /// Attach the head event's payload bytes — needed by
    /// `EntityLog::from_snapshot` to validate the next event's
    /// chain link after restore. Genesis snapshots
    /// (`chain_link.sequence == 0`) carry empty bytes; subsequent
    /// snapshots carry the payload of the event at
    /// `chain_link.sequence`.
    pub fn with_head_payload(mut self, head_payload: Bytes) -> Self {
        self.head_payload = head_payload;
        self
    }

    /// Attach an identity envelope sealed to `target_static_pub`,
    /// returning `self` by value so the call chains cleanly off
    /// [`Self::new`] / the source's snapshot-build path.
    ///
    /// Fails with
    /// [`EnvelopeError::SourceReadOnly`](crate::adapter::net::identity::EnvelopeError::SourceReadOnly)
    /// when `source_kp` is public-only — a public-only caller can't
    /// produce the attestation signature the target needs to verify.
    /// The attestation transcript binds to `self.chain_link`, so the
    /// resulting envelope is non-replayable at a different migration
    /// point.
    pub fn with_identity_envelope(
        mut self,
        source_kp: &EntityKeypair,
        target_static_pub: [u8; 32],
    ) -> Result<Self, crate::adapter::net::identity::EnvelopeError> {
        let env = IdentityEnvelope::new(source_kp, target_static_pub, &self.chain_link)?;
        self.identity_envelope = Some(env);
        Ok(self)
    }

    /// Open the attached identity envelope (if any) using the
    /// target's X25519 static private key. Returns the daemon's
    /// fully-keyed [`EntityKeypair`], which the target-side
    /// restore path uses instead of the caller-supplied fallback.
    ///
    /// Returns `Ok(None)` when the snapshot has no envelope —
    /// callers interpret this as "public-identity migration, target
    /// gets a read-only keypair." Returns `Err` if the envelope is
    /// present but fails to verify / unseal; callers must treat
    /// that as a terminal error, not a fallback trigger, or an
    /// attacker could downgrade identity transport by tampering.
    pub fn open_identity_envelope(
        &self,
        target_x25519_priv: &x25519_dalek::StaticSecret,
    ) -> Result<Option<EntityKeypair>, crate::adapter::net::identity::EnvelopeError> {
        match &self.identity_envelope {
            None => Ok(None),
            Some(env) => {
                let kp = env.open(target_x25519_priv, &self.chain_link)?;
                // Belt-and-braces: the decrypted keypair's
                // `origin_hash` must match the snapshot's
                // `entity_id`. `IdentityEnvelope::open` already
                // checks that the decrypted pub matches the
                // envelope's `signer_pub`, but the snapshot's
                // `entity_id` is a second, separately-signed
                // commitment that the decrypted identity must
                // also match.
                if kp.entity_id() != &self.entity_id {
                    return Err(crate::adapter::net::identity::EnvelopeError::OriginHashMismatch);
                }
                Ok(Some(kp))
            }
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
    /// Horizon and `head_payload` are not serialized in the compact
    /// format — `head_payload` is a runtime-only field populated by
    /// the caller from the head event before invoking restore (see
    /// the field's doc).
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
    ///
    /// `head_payload` is runtime-only and always defaults to empty
    /// after deserialize; callers must populate it from the head
    /// event before passing the snapshot to
    /// [`super::log::EntityLog::from_snapshot`].
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
            // Runtime-only: not on the wire. Caller populates from
            // the head event before invoking `EntityLog::from_snapshot`.
            head_payload: Bytes::new(),
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
        // Enforce strict length. The v1 path already does this; v0
        // silently accepted trailing bytes, which made the parser
        // tolerant to corrupted / partial input that should be
        // rejected.
        if cursor.len() != state_len {
            return None;
        }

        // Validate internal consistency
        if chain_link.sequence != through_seq {
            return None;
        }
        if chain_link.origin_hash != entity_id.origin_hash() {
            return None;
        }

        Some(Self {
            // Surface the snapshot as the current version for
            // downstream uniformity; writers always emit current,
            // so a fresh round-trip is a v0 → current operation
            // after this read.
            version: SNAPSHOT_VERSION,
            entity_id,
            through_seq,
            chain_link,
            state,
            horizon: ObservedHorizon::new(), // reconstructed separately
            created_at,
            bindings_bytes: Vec::new(),
            identity_envelope: None,
            head_payload: Bytes::new(),
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

    /// Store a snapshot if it is newer than the existing entry.
    ///
    /// Returns `true` when the snapshot was stored, `false` when an
    /// existing snapshot with a strictly higher (or equal)
    /// `through_seq` blocked the write — i.e. an older / replayed
    /// snapshot tried to overwrite a fresher one.
    ///
    /// BUG #122: pre-fix this method called
    /// `self.snapshots.insert(key, snapshot)` unconditionally, with
    /// no comparison against the existing entry's `through_seq`. A
    /// reordered or replayed snapshot delivery silently rewrote
    /// state at sequence N over an existing one at N+M — and
    /// concurrent stores raced (whichever DashMap insert landed
    /// last won regardless of freshness). Now uses
    /// `DashMap::entry` to make the read-compare-write atomic per
    /// shard. Equal `through_seq` is also rejected so a re-
    /// emission of the *same* snapshot from a stale producer doesn't
    /// thrash the entry (refresh-with-equal must explicitly
    /// `remove` first if intentional).
    pub fn store(&self, snapshot: StateSnapshot) -> bool {
        use dashmap::mapref::entry::Entry;
        let key = *snapshot.entity_id.as_bytes();
        match self.snapshots.entry(key) {
            Entry::Vacant(slot) => {
                slot.insert(snapshot);
                true
            }
            Entry::Occupied(mut slot) => {
                if snapshot.through_seq > slot.get().through_seq {
                    slot.insert(snapshot);
                    true
                } else {
                    false
                }
            }
        }
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

        let stored = store.store(snapshot);
        assert!(stored, "first store of an entity must succeed");
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
        assert!(store.store(snap1));

        let mut builder = CausalChainBuilder::new(origin_hash);
        builder.append(Bytes::from_static(b"e1"), 0).unwrap();

        let snap2 = StateSnapshot::new(
            entity_id.clone(),
            *builder.head(),
            Bytes::from_static(b"state-v2"),
            ObservedHorizon::new(),
        );
        assert!(store.store(snap2));

        assert_eq!(store.count(), 1);
        let retrieved = store.get(&entity_id).unwrap();
        assert_eq!(retrieved.state, Bytes::from_static(b"state-v2"));
        assert_eq!(retrieved.through_seq, 1);
    }

    #[test]
    fn test_from_bytes_too_short() {
        assert!(StateSnapshot::from_bytes(&[0u8; 10]).is_none());
    }

    // ========================================================================
    // BUG #122: store() must reject older snapshots (no rewind)
    // ========================================================================

    /// Building snapshots via the chain helper makes the
    /// `chain_link.sequence` actually-match `through_seq`, which is
    /// the wire-level invariant `from_bytes` enforces. Tests below
    /// drive the real public API rather than poking through_seq
    /// directly so the regression resembles the production failure
    /// mode (signed snapshots arriving in non-monotonic order).
    fn snap_at(
        entity_id: EntityId,
        builder: &mut CausalChainBuilder,
        state_bytes: &'static [u8],
    ) -> StateSnapshot {
        StateSnapshot::new(
            entity_id,
            *builder.head(),
            Bytes::from_static(state_bytes),
            ObservedHorizon::new(),
        )
    }

    /// An older snapshot (lower `through_seq`) arriving after a
    /// newer one must NOT overwrite the newer entry. Pre-fix
    /// `store` unconditionally inserted, so a replayed or reordered
    /// older snapshot silently rolled state back. Now `store`
    /// returns `false` and the existing entry is preserved.
    #[test]
    fn store_rejects_older_snapshot_against_newer_existing_entry() {
        let store = SnapshotStore::new();
        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());

        // newer snapshot at seq 5
        for _ in 0..5 {
            builder.append(Bytes::from_static(b"e"), 0).unwrap();
        }
        let newer = snap_at(entity_id.clone(), &mut builder, b"v5");
        assert_eq!(newer.through_seq, 5);
        assert!(store.store(newer), "first store must succeed");

        // older snapshot at seq 2 (rebuild a fresh chain)
        let mut older_builder = CausalChainBuilder::new(kp.origin_hash());
        for _ in 0..2 {
            older_builder.append(Bytes::from_static(b"e"), 0).unwrap();
        }
        let older = snap_at(entity_id.clone(), &mut older_builder, b"v2");
        assert_eq!(older.through_seq, 2);
        let stored = store.store(older);

        assert!(!stored, "older snapshot must be rejected");
        let retrieved = store.get(&entity_id).unwrap();
        assert_eq!(
            retrieved.state,
            Bytes::from_static(b"v5"),
            "newer snapshot must be preserved despite older arrival",
        );
        assert_eq!(retrieved.through_seq, 5);
    }

    /// Equal `through_seq` is rejected too — a re-emission from a
    /// stale producer shouldn't churn the entry. Callers that
    /// genuinely need to refresh-at-same-seq (e.g. legitimate
    /// rebind) must `remove` first.
    #[test]
    fn store_rejects_equal_through_seq_against_existing_entry() {
        let store = SnapshotStore::new();
        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());
        for _ in 0..3 {
            builder.append(Bytes::from_static(b"e"), 0).unwrap();
        }
        let first = snap_at(entity_id.clone(), &mut builder, b"first");
        assert_eq!(first.through_seq, 3);
        assert!(store.store(first));

        let mut other = CausalChainBuilder::new(kp.origin_hash());
        for _ in 0..3 {
            other.append(Bytes::from_static(b"e"), 0).unwrap();
        }
        let second = snap_at(entity_id.clone(), &mut other, b"second");
        assert_eq!(second.through_seq, 3);
        let stored = store.store(second);

        assert!(!stored, "equal through_seq must be rejected");
        let retrieved = store.get(&entity_id).unwrap();
        assert_eq!(
            retrieved.state,
            Bytes::from_static(b"first"),
            "first-stored snapshot must remain authoritative on equal through_seq",
        );
    }

    /// Strictly newer `through_seq` is accepted — pins the success
    /// path so a future tightening that flips `>` to `>=` can't
    /// silently break legitimate progressive snapshots.
    #[test]
    fn store_accepts_strictly_newer_snapshot() {
        let store = SnapshotStore::new();
        let kp = EntityKeypair::generate();
        let entity_id = kp.entity_id().clone();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());
        for _ in 0..2 {
            builder.append(Bytes::from_static(b"e"), 0).unwrap();
        }
        let earlier = snap_at(entity_id.clone(), &mut builder, b"v2");
        assert!(store.store(earlier));

        for _ in 0..3 {
            builder.append(Bytes::from_static(b"e"), 0).unwrap();
        }
        let later = snap_at(entity_id.clone(), &mut builder, b"v5");
        assert_eq!(later.through_seq, 5);
        assert!(store.store(later), "newer snapshot must be accepted");

        let retrieved = store.get(&entity_id).unwrap();
        assert_eq!(retrieved.through_seq, 5);
        assert_eq!(retrieved.state, Bytes::from_static(b"v5"));
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

    // ---- Identity-envelope end-to-end (Stage 5) ----

    fn fresh_x25519() -> (x25519_dalek::StaticSecret, [u8; 32]) {
        let mut seed = [0u8; 32];
        getrandom::fill(&mut seed).unwrap();
        let sk = x25519_dalek::StaticSecret::from(seed);
        let pk = x25519_dalek::PublicKey::from(&sk);
        (sk, *pk.as_bytes())
    }

    #[test]
    fn envelope_roundtrip_seals_and_opens_through_wire() {
        // Full migration-primitive slice: source builds a snapshot,
        // seals its daemon keypair to the target's X25519 pubkey,
        // serializes, ships bytes, target deserializes, opens the
        // envelope with its X25519 private key, recovers the same
        // daemon keypair (including the ability to sign).
        let daemon_kp = EntityKeypair::generate();
        let entity_id = daemon_kp.entity_id().clone();
        let mut builder = CausalChainBuilder::new(daemon_kp.origin_hash());
        builder.append(Bytes::from_static(b"event"), 0).unwrap();

        let (target_priv, target_pub) = fresh_x25519();

        let snapshot = StateSnapshot::new(
            entity_id.clone(),
            *builder.head(),
            Bytes::from_static(b"daemon state"),
            ObservedHorizon::new(),
        )
        .with_identity_envelope(&daemon_kp, target_pub)
        .expect("seal");

        // Round-trip through bytes (simulating the wire).
        let bytes = snapshot.to_bytes();
        let received = StateSnapshot::from_bytes(&bytes).expect("decode");
        assert!(received.identity_envelope.is_some());

        // Target opens with its X25519 private key.
        let recovered = received
            .open_identity_envelope(&target_priv)
            .expect("open")
            .expect("envelope present");

        // Full round-trip: the recovered keypair has the same
        // identity AND a working signing half.
        assert_eq!(recovered.entity_id(), &entity_id);
        assert_eq!(recovered.origin_hash(), daemon_kp.origin_hash());
        assert!(!recovered.is_read_only());
        let sig = recovered.sign(b"post-migration");
        assert!(entity_id.verify(b"post-migration", &sig).is_ok());
    }

    #[test]
    fn envelope_open_on_snapshot_without_envelope_returns_none() {
        let kp = EntityKeypair::generate();
        let snapshot = StateSnapshot::new(
            kp.entity_id().clone(),
            CausalLink::genesis(kp.origin_hash(), 0),
            Bytes::from_static(b"s"),
            ObservedHorizon::new(),
        );

        let (target_priv, _) = fresh_x25519();
        let opened = snapshot
            .open_identity_envelope(&target_priv)
            .expect("no envelope is not an error");
        assert!(
            opened.is_none(),
            "public-identity migration: target gets None"
        );
    }

    #[test]
    fn envelope_open_rejects_wrong_entity_id() {
        // Belt-and-braces: the snapshot commits to a specific
        // entity_id independently of the envelope's attestation. If
        // the envelope's attested `signer_pub` doesn't match the
        // snapshot's `entity_id`, `open_identity_envelope` must
        // reject — otherwise an attacker who compromises the
        // envelope-sealing path could still be caught by the
        // snapshot-level identity commitment.
        let real_daemon = EntityKeypair::generate();
        let impostor = EntityKeypair::generate();
        let (target_priv, target_pub) = fresh_x25519();

        let mut builder = CausalChainBuilder::new(real_daemon.origin_hash());
        builder.append(Bytes::from_static(b"e"), 0).unwrap();

        // Snapshot commits to `real_daemon`'s entity_id…
        let mut snapshot = StateSnapshot::new(
            real_daemon.entity_id().clone(),
            *builder.head(),
            Bytes::from_static(b"s"),
            ObservedHorizon::new(),
        );

        // …but an envelope is built from the impostor's keypair.
        // Can't happen through `with_identity_envelope` (which uses
        // the snapshot's own daemon keypair), so we construct
        // manually to simulate a tampered wire payload.
        let env = IdentityEnvelope::new(&impostor, target_pub, &snapshot.chain_link)
            .expect("impostor can still seal their own keypair");
        snapshot.identity_envelope = Some(env);

        // Fix up chain_link's origin_hash so the snapshot's own
        // consistency check (origin_hash == entity_id.origin_hash)
        // still passes — the point of this test is the
        // envelope-vs-entity_id mismatch, not the chain check.
        assert_eq!(
            snapshot.chain_link.origin_hash,
            snapshot.entity_id.origin_hash()
        );

        let err = snapshot
            .open_identity_envelope(&target_priv)
            .expect_err("impostor envelope must be rejected");
        use crate::adapter::net::identity::EnvelopeError;
        assert_eq!(err, EnvelopeError::OriginHashMismatch);
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

    /// Regression: BUG_REPORT.md #40 — `from_bytes_v0` previously
    /// accepted trailing bytes past the declared `state_len`. The
    /// v1 path enforces strict length; v0 silently accepted junk
    /// after the state, masking corrupt or partial input that
    /// should be rejected. This test pins v0 to the same
    /// strict-length contract.
    #[test]
    fn from_bytes_v0_rejects_trailing_garbage() {
        // Build a v0 snapshot via the same chain builder used by
        // other tests so `chain_link.sequence == through_seq` and
        // `chain_link.origin_hash == entity_id.origin_hash()` (the
        // parser enforces both invariants and we want them both to
        // pass — we're isolating the trailing-bytes check).
        let kp = EntityKeypair::generate();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());
        builder.append(Bytes::from_static(b"e1"), 0).unwrap();
        let head = *builder.head();
        let through_seq = head.sequence;
        let state = b"state-bytes";

        let mut buf = bytes::BytesMut::new();
        use bytes::BufMut;
        buf.put_slice(kp.entity_id().as_bytes());
        buf.put_u64_le(through_seq);
        buf.put_slice(&head.to_bytes());
        buf.put_u64_le(12345);
        buf.put_u32_le(state.len() as u32);
        buf.put_slice(state);

        // Sanity: the round-trip works as-is.
        assert!(
            StateSnapshot::from_bytes_v0(&buf).is_some(),
            "valid v0 snapshot must parse"
        );

        // Append trailing junk. The strict-length check rejects.
        buf.put_slice(b"trailing-junk");
        assert!(
            StateSnapshot::from_bytes_v0(&buf).is_none(),
            "v0 must reject trailing bytes past state_len (#40)"
        );
    }

    /// Regression: `EntityLog::from_snapshot` requires the head
    /// event's payload bytes to validate the next event's
    /// `parent_hash`. The snapshot now carries `head_payload` as a
    /// runtime-only field (not on the wire) that callers populate
    /// from the head event before invoking restore. This test pins:
    ///
    /// 1. The default constructor leaves `head_payload` empty.
    /// 2. `with_head_payload` stores the bytes.
    /// 3. The wire format is unchanged — `head_payload` round-trips
    ///    as empty regardless of what was set in-process (since the
    ///    field isn't serialized).
    /// 4. After deserialize, the caller can populate `head_payload`
    ///    out-of-band and use the snapshot for restore.
    #[test]
    fn head_payload_is_runtime_only_not_on_wire() {
        let kp = EntityKeypair::generate();
        let mut builder = CausalChainBuilder::new(kp.origin_hash());
        let head_event_payload = Bytes::from_static(b"head-event-payload");
        builder.append(head_event_payload.clone(), 0).unwrap();

        // Default constructor: head_payload is empty.
        let mut snap = StateSnapshot::new(
            kp.entity_id().clone(),
            *builder.head(),
            Bytes::from_static(b"daemon-state-bytes"),
            ObservedHorizon::new(),
        );
        assert!(
            snap.head_payload.is_empty(),
            "default constructor leaves head_payload empty"
        );

        // Pin `created_at` so the wire-byte comparison below is
        // deterministic — the field is sampled from the system
        // clock at construction.
        snap.created_at = 0;
        // with_head_payload stores the bytes.
        let snap = snap.with_head_payload(head_event_payload.clone());
        assert_eq!(snap.head_payload, head_event_payload);

        // Wire format is unchanged: head_payload is NOT serialized.
        // We pin this two ways:
        //   (a) the round-trip yields head_payload = empty
        //       regardless of what was set in-process
        //   (b) the byte length is identical to a snapshot with
        //       empty head_payload (proves no length-prefix sneaked
        //       into the wire format)
        let bytes_with = snap.to_bytes();
        let mut snap_empty = StateSnapshot::new(
            kp.entity_id().clone(),
            *builder.head(),
            Bytes::from_static(b"daemon-state-bytes"),
            ObservedHorizon::new(),
        );
        snap_empty.created_at = 0;
        let bytes_without = snap_empty.to_bytes();
        assert_eq!(
            bytes_with.len(),
            bytes_without.len(),
            "head_payload must not appear in the wire format"
        );
        assert_eq!(
            bytes_with, bytes_without,
            "wire bytes must be identical regardless of head_payload"
        );

        // Round-trip: head_payload defaults to empty after parse.
        let parsed = StateSnapshot::from_bytes(&bytes_with).unwrap();
        assert!(
            parsed.head_payload.is_empty(),
            "head_payload after round-trip must be empty (runtime-only field)"
        );

        // Caller populates head_payload from the head event they
        // already have, then restore can succeed.
        let parsed = parsed.with_head_payload(head_event_payload.clone());
        assert_eq!(parsed.head_payload, head_event_payload);
    }
}
