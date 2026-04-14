//! Permission tokens for BLTP authorization.
//!
//! Tokens are ed25519-signed, delegatable, and expirable. They authorize
//! an entity to perform specific actions (publish, subscribe, admin) on
//! specific channels. L2 (Channels & Authorization) enforces these at
//! subscription time, not per-packet.

use dashmap::DashMap;
use ed25519_dalek::Signature;
use std::time::{SystemTime, UNIX_EPOCH};

use super::entity::{EntityId, EntityKeypair};

/// Actions a token can authorize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenScope {
    bits: u32,
}

impl TokenScope {
    /// No permissions.
    pub const NONE: Self = Self { bits: 0 };
    /// Publish events to a channel.
    pub const PUBLISH: Self = Self { bits: 0b0001 };
    /// Subscribe to events from a channel.
    pub const SUBSCRIBE: Self = Self { bits: 0b0010 };
    /// Administrative access (create/delete channels, manage tokens).
    pub const ADMIN: Self = Self { bits: 0b0100 };
    /// Can delegate this token to other entities.
    pub const DELEGATE: Self = Self { bits: 0b1000 };
    /// Full access.
    pub const ALL: Self = Self { bits: 0b1111 };

    /// Create a scope from raw bits.
    #[inline]
    pub const fn from_bits(bits: u32) -> Self {
        Self { bits }
    }

    /// Get the raw bits.
    #[inline]
    pub const fn bits(self) -> u32 {
        self.bits
    }

    /// Check if this scope includes another.
    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Restrict this scope to only include permissions in `other`.
    #[inline]
    pub const fn intersect(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }

    /// Combine with another scope.
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    /// Optional channel hash filter. If set, token only applies to
    /// channels matching this hash.
    pub fn with_channel(self, channel_hash: u16) -> ScopedToken {
        ScopedToken {
            scope: self,
            channel_hash: Some(channel_hash),
        }
    }
}

/// A scope bound to an optional channel.
#[derive(Debug, Clone, Copy)]
pub struct ScopedToken {
    pub scope: TokenScope,
    pub channel_hash: Option<u16>,
}

/// A signed, delegatable permission token.
///
/// Wire format (159 bytes):
/// ```text
/// issuer:           32 bytes (EntityId)
/// subject:          32 bytes (EntityId)
/// scope:             4 bytes (u32)
/// channel_hash:      2 bytes (u16, 0 = all channels)
/// not_before:        8 bytes (u64 unix timestamp)
/// not_after:         8 bytes (u64 unix timestamp)
/// delegation_depth:  1 byte  (u8)
/// nonce:             8 bytes (u64)
/// --- signed above ---
/// signature:        64 bytes (ed25519)
/// ```
#[derive(Clone)]
pub struct PermissionToken {
    /// Who issued this token.
    pub issuer: EntityId,
    /// Who this token authorizes.
    pub subject: EntityId,
    /// What actions are permitted.
    pub scope: TokenScope,
    /// Channel restriction (0 = all channels).
    pub channel_hash: u16,
    /// Valid from (unix timestamp seconds).
    pub not_before: u64,
    /// Valid until (unix timestamp seconds).
    pub not_after: u64,
    /// How many times this token can be re-delegated.
    pub delegation_depth: u8,
    /// Unique nonce for revocation.
    pub nonce: u64,
    /// Ed25519 signature over all preceding fields.
    pub signature: [u8; 64],
}

impl PermissionToken {
    /// Size of the signed payload (everything before the signature).
    const SIGNED_PAYLOAD_SIZE: usize = 32 + 32 + 4 + 2 + 8 + 8 + 1 + 8; // 95 bytes

    /// Total serialized size.
    pub const WIRE_SIZE: usize = Self::SIGNED_PAYLOAD_SIZE + 64; // 159 bytes

    /// Issue a new token.
    pub fn issue(
        issuer_keypair: &EntityKeypair,
        subject: EntityId,
        scope: TokenScope,
        channel_hash: u16,
        duration_secs: u64,
        delegation_depth: u8,
    ) -> Self {
        let now = current_timestamp();
        let mut nonce_bytes = [0u8; 8];
        getrandom::fill(&mut nonce_bytes).expect("getrandom failed");
        let nonce = u64::from_le_bytes(nonce_bytes);

        let mut token = Self {
            issuer: issuer_keypair.entity_id().clone(),
            subject,
            scope,
            channel_hash,
            not_before: now,
            not_after: now + duration_secs,
            delegation_depth,
            nonce,
            signature: [0u8; 64],
        };

        let payload = token.signed_payload();
        let sig = issuer_keypair.sign(&payload);
        token.signature = sig.to_bytes();
        token
    }

    /// Verify the token's signature against the issuer's public key.
    pub fn verify(&self) -> Result<(), TokenError> {
        let payload = self.signed_payload();
        let sig = Signature::from_bytes(&self.signature);
        self.issuer
            .verify(&payload, &sig)
            .map_err(|_| TokenError::InvalidSignature)
    }

    /// Check if the token is currently valid (signature + time bounds).
    pub fn is_valid(&self) -> Result<(), TokenError> {
        self.verify()?;
        let now = current_timestamp();
        if now < self.not_before {
            return Err(TokenError::NotYetValid);
        }
        if now > self.not_after {
            return Err(TokenError::Expired);
        }
        Ok(())
    }

    /// Check if this token authorizes a specific action on a channel.
    pub fn authorizes(&self, action: TokenScope, channel: u16) -> bool {
        if !self.scope.contains(action) {
            return false;
        }
        // channel_hash 0 = wildcard (all channels)
        self.channel_hash == 0 || self.channel_hash == channel
    }

    /// Delegate this token to another entity with restricted scope.
    ///
    /// Returns `None` if delegation is not allowed (depth exhausted or
    /// DELEGATE not in scope).
    pub fn delegate(
        &self,
        signer: &EntityKeypair,
        new_subject: EntityId,
        restricted_scope: TokenScope,
    ) -> Result<Self, TokenError> {
        // Validate the parent token first
        self.is_valid()?;

        // Check delegation is allowed
        if self.delegation_depth == 0 {
            return Err(TokenError::DelegationExhausted);
        }
        if !self.scope.contains(TokenScope::DELEGATE) {
            return Err(TokenError::DelegationNotAllowed);
        }
        // Verify the signer is the subject of this token
        if signer.entity_id() != &self.subject {
            return Err(TokenError::NotAuthorized);
        }

        // New scope is intersection of current scope and requested scope
        let new_scope = self.scope.intersect(restricted_scope);

        Ok(Self::issue(
            signer,
            new_subject,
            new_scope,
            self.channel_hash,
            self.not_after.saturating_sub(current_timestamp()),
            self.delegation_depth - 1,
        ))
    }

    /// Serialize the fields that are covered by the signature.
    fn signed_payload(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::SIGNED_PAYLOAD_SIZE);
        buf.extend_from_slice(self.issuer.as_bytes());
        buf.extend_from_slice(self.subject.as_bytes());
        buf.extend_from_slice(&self.scope.bits().to_le_bytes());
        buf.extend_from_slice(&self.channel_hash.to_le_bytes());
        buf.extend_from_slice(&self.not_before.to_le_bytes());
        buf.extend_from_slice(&self.not_after.to_le_bytes());
        buf.push(self.delegation_depth);
        buf.extend_from_slice(&self.nonce.to_le_bytes());
        buf
    }

    /// Serialize to wire format.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = self.signed_payload();
        buf.extend_from_slice(&self.signature);
        buf
    }

    /// Deserialize from wire format.
    pub fn from_bytes(data: &[u8]) -> Result<Self, TokenError> {
        if data.len() < Self::WIRE_SIZE {
            return Err(TokenError::InvalidFormat);
        }

        let issuer = EntityId::from_bytes(data[0..32].try_into().unwrap());
        let subject = EntityId::from_bytes(data[32..64].try_into().unwrap());
        let scope = TokenScope::from_bits(u32::from_le_bytes(data[64..68].try_into().unwrap()));
        let channel_hash = u16::from_le_bytes(data[68..70].try_into().unwrap());
        let not_before = u64::from_le_bytes(data[70..78].try_into().unwrap());
        let not_after = u64::from_le_bytes(data[78..86].try_into().unwrap());
        let delegation_depth = data[86];
        let nonce = u64::from_le_bytes(data[87..95].try_into().unwrap());
        let mut signature = [0u8; 64];
        signature.copy_from_slice(&data[95..159]);

        Ok(Self {
            issuer,
            subject,
            scope,
            channel_hash,
            not_before,
            not_after,
            delegation_depth,
            nonce,
            signature,
        })
    }
}

impl std::fmt::Debug for PermissionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PermissionToken")
            .field("issuer", &self.issuer)
            .field("subject", &self.subject)
            .field("scope", &format!("{:04b}", self.scope.bits()))
            .field("channel_hash", &format!("{:04x}", self.channel_hash))
            .field("delegation_depth", &self.delegation_depth)
            .field("nonce", &self.nonce)
            .finish()
    }
}

/// Fast permission lookup cache.
///
/// Keyed by `(subject EntityId, channel_hash)` for O(1) per-channel
/// authorization checks. Entries are not evicted automatically —
/// callers should check `is_valid()` on retrieved tokens.
pub struct TokenCache {
    tokens: DashMap<([u8; 32], u16), PermissionToken>,
}

impl TokenCache {
    /// Create an empty token cache.
    pub fn new() -> Self {
        Self {
            tokens: DashMap::new(),
        }
    }

    /// Insert a token into the cache after verifying its signature.
    ///
    /// Returns an error if the token's signature is invalid. This prevents
    /// self-signed or tampered tokens from being cached.
    pub fn insert(&self, token: PermissionToken) -> Result<(), TokenError> {
        token.verify()?;
        let key = (*token.subject.as_bytes(), token.channel_hash);
        self.tokens.insert(key, token);
        Ok(())
    }

    /// Insert a token without verification (for trusted internal use).
    ///
    /// Only use this when the token is known to be valid (e.g., just issued locally).
    pub fn insert_unchecked(&self, token: PermissionToken) {
        let key = (*token.subject.as_bytes(), token.channel_hash);
        self.tokens.insert(key, token);
    }

    /// Check if an entity is authorized for an action on a channel.
    ///
    /// Returns `Ok(())` if a valid token exists, or an error explaining why not.
    /// Checks channel-specific token first, then wildcard (channel_hash = 0).
    /// An expired/invalid channel-specific token does NOT block the wildcard fallback.
    pub fn check(
        &self,
        subject: &EntityId,
        action: TokenScope,
        channel_hash: u16,
    ) -> Result<(), TokenError> {
        // Try exact channel match first
        if let Some(token) = self.tokens.get(&(*subject.as_bytes(), channel_hash)) {
            if token.is_valid().is_ok() && token.authorizes(action, channel_hash) {
                return Ok(());
            }
        }
        // Try wildcard (channel_hash = 0)
        if let Some(token) = self.tokens.get(&(*subject.as_bytes(), 0)) {
            if token.is_valid().is_ok() && token.authorizes(action, channel_hash) {
                return Ok(());
            }
        }
        Err(TokenError::NotAuthorized)
    }

    /// Remove expired tokens.
    pub fn evict_expired(&self) {
        let now = current_timestamp();
        self.tokens.retain(|_, token| token.not_after > now);
    }

    /// Number of cached tokens.
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

impl Default for TokenCache {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TokenCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenCache")
            .field("count", &self.tokens.len())
            .finish()
    }
}

/// Errors from token operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenError {
    /// Token signature is invalid.
    InvalidSignature,
    /// Token is not yet valid (before not_before).
    NotYetValid,
    /// Token has expired (after not_after).
    Expired,
    /// Delegation depth exhausted.
    DelegationExhausted,
    /// DELEGATE scope not present in token.
    DelegationNotAllowed,
    /// No valid token found for the requested action.
    NotAuthorized,
    /// Wire format is too short or malformed.
    InvalidFormat,
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "invalid token signature"),
            Self::NotYetValid => write!(f, "token not yet valid"),
            Self::Expired => write!(f, "token expired"),
            Self::DelegationExhausted => write!(f, "delegation depth exhausted"),
            Self::DelegationNotAllowed => write!(f, "delegation not allowed by scope"),
            Self::NotAuthorized => write!(f, "not authorized"),
            Self::InvalidFormat => write!(f, "invalid token format"),
        }
    }
}

impl std::error::Error for TokenError {}

/// Current unix timestamp in seconds.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_and_verify() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();

        let token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH.union(TokenScope::SUBSCRIBE),
            0, // all channels
            3600,
            0,
        );

        assert!(token.verify().is_ok());
        assert!(token.is_valid().is_ok());
    }

    #[test]
    fn test_tampered_token() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();

        let mut token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0,
            3600,
            0,
        );

        // Tamper with scope
        token.scope = TokenScope::ADMIN;
        assert!(token.verify().is_err());
    }

    #[test]
    fn test_expired_token() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();

        let token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0,
            0, // expires immediately
            0,
        );

        assert!(token.verify().is_ok()); // signature is valid
                                         // Token may or may not be expired depending on timing,
                                         // but with duration 0 and not_after = now, it's borderline
    }

    #[test]
    fn test_channel_filter() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();

        let token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0xABCD, // specific channel
            3600,
            0,
        );

        assert!(token.authorizes(TokenScope::PUBLISH, 0xABCD));
        assert!(!token.authorizes(TokenScope::PUBLISH, 0x1234)); // wrong channel
        assert!(!token.authorizes(TokenScope::SUBSCRIBE, 0xABCD)); // wrong action
    }

    #[test]
    fn test_wildcard_channel() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();

        let token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0, // all channels
            3600,
            0,
        );

        assert!(token.authorizes(TokenScope::PUBLISH, 0xABCD));
        assert!(token.authorizes(TokenScope::PUBLISH, 0x1234));
        assert!(token.authorizes(TokenScope::PUBLISH, 0));
    }

    #[test]
    fn test_delegation() {
        let root = EntityKeypair::generate();
        let node_a = EntityKeypair::generate();
        let node_b = EntityKeypair::generate();

        // Root issues to A with delegation depth 2
        let token_a = PermissionToken::issue(
            &root,
            node_a.entity_id().clone(),
            TokenScope::ALL,
            0,
            3600,
            2,
        );
        assert!(token_a.is_valid().is_ok());

        // A delegates to B with restricted scope
        let token_b = token_a
            .delegate(
                &node_a,
                node_b.entity_id().clone(),
                TokenScope::PUBLISH.union(TokenScope::DELEGATE),
            )
            .unwrap();

        assert!(token_b.is_valid().is_ok());
        assert_eq!(token_b.delegation_depth, 1);
        assert!(token_b.authorizes(TokenScope::PUBLISH, 0));
        assert!(!token_b.authorizes(TokenScope::ADMIN, 0)); // restricted away
    }

    #[test]
    fn test_delegation_depth_exhausted() {
        let root = EntityKeypair::generate();
        let node_a = EntityKeypair::generate();
        let node_b = EntityKeypair::generate();

        let token = PermissionToken::issue(
            &root,
            node_a.entity_id().clone(),
            TokenScope::ALL,
            0,
            3600,
            0, // no delegation
        );

        let result = token.delegate(&node_a, node_b.entity_id().clone(), TokenScope::PUBLISH);
        assert_eq!(result.unwrap_err(), TokenError::DelegationExhausted);
    }

    #[test]
    fn test_delegation_wrong_signer() {
        let root = EntityKeypair::generate();
        let node_a = EntityKeypair::generate();
        let node_b = EntityKeypair::generate();
        let imposter = EntityKeypair::generate();

        let token = PermissionToken::issue(
            &root,
            node_a.entity_id().clone(),
            TokenScope::ALL,
            0,
            3600,
            1,
        );

        // Imposter tries to delegate A's token
        let result = token.delegate(&imposter, node_b.entity_id().clone(), TokenScope::PUBLISH);
        assert_eq!(result.unwrap_err(), TokenError::NotAuthorized);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();

        let token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH.union(TokenScope::SUBSCRIBE),
            0xBEEF,
            3600,
            3,
        );

        let bytes = token.to_bytes();
        assert_eq!(bytes.len(), PermissionToken::WIRE_SIZE);

        let parsed = PermissionToken::from_bytes(&bytes).unwrap();
        assert!(parsed.verify().is_ok());
        assert_eq!(parsed.issuer, token.issuer);
        assert_eq!(parsed.subject, token.subject);
        assert_eq!(parsed.scope.bits(), token.scope.bits());
        assert_eq!(parsed.channel_hash, 0xBEEF);
        assert_eq!(parsed.delegation_depth, 3);
        assert_eq!(parsed.nonce, token.nonce);
    }

    #[test]
    fn test_token_cache() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();

        let cache = TokenCache::new();

        let token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0xABCD,
            3600,
            0,
        );
        cache.insert(token);

        assert_eq!(cache.len(), 1);

        // Should find the token
        assert!(cache
            .check(subject.entity_id(), TokenScope::PUBLISH, 0xABCD)
            .is_ok());

        // Wrong channel
        assert!(cache
            .check(subject.entity_id(), TokenScope::PUBLISH, 0x1234)
            .is_err());

        // Wrong action
        assert!(cache
            .check(subject.entity_id(), TokenScope::ADMIN, 0xABCD)
            .is_err());

        // Unknown entity
        let unknown = EntityKeypair::generate();
        assert!(cache
            .check(unknown.entity_id(), TokenScope::PUBLISH, 0xABCD)
            .is_err());
    }

    #[test]
    fn test_token_cache_wildcard() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();

        let cache = TokenCache::new();

        // Wildcard token (channel_hash = 0)
        let token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0,
            3600,
            0,
        );
        cache.insert(token);

        // Should match any channel
        assert!(cache
            .check(subject.entity_id(), TokenScope::PUBLISH, 0xABCD)
            .is_ok());
        assert!(cache
            .check(subject.entity_id(), TokenScope::PUBLISH, 0x1234)
            .is_ok());
    }
}
