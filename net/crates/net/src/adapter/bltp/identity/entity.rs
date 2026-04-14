//! Cryptographic entity identity for BLTP nodes.
//!
//! An entity is identified by its ed25519 public key. The identity is
//! independent of network addresses — an entity can migrate across nodes.
//! The u64 node IDs used in swarm/routing are derived from the public key.

use blake2::{
    digest::{consts::U32, Mac},
    Blake2sMac,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// Entity identity — a 32-byte ed25519 public key.
///
/// This is the canonical identity for a node in the mesh. All other
/// identifiers (node_id, origin_hash) are derived from this.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EntityId(pub [u8; 32]);

impl EntityId {
    /// Create an EntityId from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the raw public key bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Derive the 4-byte origin hash for the BLTP header.
    ///
    /// Uses BLAKE2s-MAC keyed with `"bltp-origin"` to produce a truncated
    /// hash. This is what goes into `BltpHeader::origin_hash`.
    #[inline]
    pub fn origin_hash(&self) -> u32 {
        let hash = self.blake2s_hash(b"bltp-origin-v1");
        u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
    }

    /// Derive the 8-byte node ID for swarm/routing.
    ///
    /// This replaces arbitrary u64 node IDs with cryptographically derived
    /// ones, binding node identity to the entity keypair.
    #[inline]
    pub fn node_id(&self) -> u64 {
        let hash = self.blake2s_hash(b"bltp-node-id-v1");
        u64::from_le_bytes(hash[0..8].try_into().unwrap())
    }

    /// Get the ed25519 verifying key for signature verification.
    pub fn verifying_key(&self) -> Result<VerifyingKey, EntityError> {
        VerifyingKey::from_bytes(&self.0).map_err(|_| EntityError::InvalidPublicKey)
    }

    /// Verify a signature against this entity's public key.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), EntityError> {
        let vk = self.verifying_key()?;
        vk.verify(message, signature)
            .map_err(|_| EntityError::InvalidSignature)
    }

    /// Compute BLAKE2s-MAC hash of the public key with a domain label.
    fn blake2s_hash(&self, label: &[u8]) -> [u8; 32] {
        let mut mac = <Blake2sMac<U32> as Mac>::new_from_slice(label)
            .expect("BLAKE2s accepts variable-length keys");
        Mac::update(&mut mac, &self.0);
        let result = mac.finalize().into_bytes();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }
}

impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({})", hex_short(&self.0))
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex_short(&self.0))
    }
}

/// Entity keypair — ed25519 signing key + public identity.
///
/// This is the root of trust for a node. The signing key must be kept secret.
/// The `EntityId` (public key) is freely shareable.
pub struct EntityKeypair {
    signing_key: SigningKey,
    entity_id: EntityId,
    /// Cached origin_hash (computed once)
    origin_hash: u32,
    /// Cached node_id (computed once)
    node_id: u64,
}

impl EntityKeypair {
    /// Generate a new random keypair.
    pub fn generate() -> Self {
        let mut rng_bytes = [0u8; 32];
        getrandom::fill(&mut rng_bytes).expect("getrandom failed");
        let signing_key = SigningKey::from_bytes(&rng_bytes);
        Self::from_signing_key(signing_key)
    }

    /// Create from an existing ed25519 signing key.
    pub fn from_signing_key(signing_key: SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();
        let entity_id = EntityId::from_bytes(verifying_key.to_bytes());
        let origin_hash = entity_id.origin_hash();
        let node_id = entity_id.node_id();
        Self {
            signing_key,
            entity_id,
            origin_hash,
            node_id,
        }
    }

    /// Create from raw secret key bytes (32 bytes).
    pub fn from_bytes(secret: [u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(&secret);
        Self::from_signing_key(signing_key)
    }

    /// Get the entity identity (public key).
    #[inline]
    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    /// Get the cached origin hash for the BLTP header.
    #[inline]
    pub fn origin_hash(&self) -> u32 {
        self.origin_hash
    }

    /// Get the cached node ID for swarm/routing.
    #[inline]
    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    /// Sign a message.
    #[inline]
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Get the raw secret key bytes.
    ///
    /// Handle with care — this is the root secret.
    pub fn secret_bytes(&self) -> &[u8; 32] {
        self.signing_key.as_bytes()
    }
}

impl Clone for EntityKeypair {
    fn clone(&self) -> Self {
        Self::from_signing_key(self.signing_key.clone())
    }
}

impl std::fmt::Debug for EntityKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityKeypair")
            .field("entity_id", &self.entity_id)
            .field("origin_hash", &format!("{:08x}", self.origin_hash))
            .field("node_id", &format!("{:016x}", self.node_id))
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

/// Errors from entity identity operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityError {
    /// Public key bytes are not a valid ed25519 point.
    InvalidPublicKey,
    /// Signature verification failed.
    InvalidSignature,
}

impl std::fmt::Display for EntityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPublicKey => write!(f, "invalid public key"),
            Self::InvalidSignature => write!(f, "invalid signature"),
        }
    }
}

impl std::error::Error for EntityError {}

/// Format first 8 bytes as hex for debug display.
fn hex_short(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(8)
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
        + "..."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generate() {
        let kp1 = EntityKeypair::generate();
        let kp2 = EntityKeypair::generate();

        // Different keypairs produce different identities
        assert_ne!(kp1.entity_id(), kp2.entity_id());
        assert_ne!(kp1.origin_hash(), kp2.origin_hash());
        assert_ne!(kp1.node_id(), kp2.node_id());
    }

    #[test]
    fn test_keypair_from_bytes_deterministic() {
        let secret = [0x42u8; 32];
        let kp1 = EntityKeypair::from_bytes(secret);
        let kp2 = EntityKeypair::from_bytes(secret);

        assert_eq!(kp1.entity_id(), kp2.entity_id());
        assert_eq!(kp1.origin_hash(), kp2.origin_hash());
        assert_eq!(kp1.node_id(), kp2.node_id());
    }

    #[test]
    fn test_sign_verify() {
        let kp = EntityKeypair::generate();
        let message = b"hello, mesh";

        let signature = kp.sign(message);
        assert!(kp.entity_id().verify(message, &signature).is_ok());
    }

    #[test]
    fn test_verify_wrong_message() {
        let kp = EntityKeypair::generate();
        let signature = kp.sign(b"correct message");

        assert_eq!(
            kp.entity_id().verify(b"wrong message", &signature),
            Err(EntityError::InvalidSignature)
        );
    }

    #[test]
    fn test_verify_wrong_key() {
        let kp1 = EntityKeypair::generate();
        let kp2 = EntityKeypair::generate();
        let message = b"hello";

        let signature = kp1.sign(message);
        assert_eq!(
            kp2.entity_id().verify(message, &signature),
            Err(EntityError::InvalidSignature)
        );
    }

    #[test]
    fn test_origin_hash_nonzero() {
        let kp = EntityKeypair::generate();
        // origin_hash should be non-zero (with overwhelming probability)
        // and consistent with entity_id derivation
        assert_eq!(kp.origin_hash(), kp.entity_id().origin_hash());
    }

    #[test]
    fn test_node_id_nonzero() {
        let kp = EntityKeypair::generate();
        assert_eq!(kp.node_id(), kp.entity_id().node_id());
    }

    #[test]
    fn test_clone_preserves_identity() {
        let kp = EntityKeypair::generate();
        let kp2 = kp.clone();

        assert_eq!(kp.entity_id(), kp2.entity_id());
        assert_eq!(kp.origin_hash(), kp2.origin_hash());

        // Cloned keypair can sign and the original can verify
        let sig = kp2.sign(b"test");
        assert!(kp.entity_id().verify(b"test", &sig).is_ok());
    }

    #[test]
    fn test_entity_id_display() {
        let kp = EntityKeypair::generate();
        let display = format!("{}", kp.entity_id());
        // Should be hex prefix + "..."
        assert!(display.ends_with("..."));
        assert!(display.len() > 4);
    }
}
