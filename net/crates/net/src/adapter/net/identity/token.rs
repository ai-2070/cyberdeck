//! Permission tokens for Net authorization.
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
    ///
    /// `duration_secs` is clamped: a value that would overflow
    /// `now + duration_secs` saturates `not_after` at `u64::MAX`,
    /// producing a functionally-never-expiring token rather than
    /// wrapping the timestamp or panicking. Callers who want to
    /// reject pathological TTLs should range-check at the SDK
    /// layer.
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
            not_after: now.saturating_add(duration_secs),
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

    /// Pure time-bound check: `true` iff the host wall-clock is
    /// past `not_after`. Deliberately **does not** touch the
    /// signature — callers wanting end-to-end validity use
    /// [`Self::is_valid`], and signature integrity alone is
    /// [`Self::verify`]. This separation matters because a
    /// tampered-but-expired token is still expired, and every
    /// binding's `token_is_expired` helper documents itself as a
    /// pure time check.
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.not_after
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
    ///
    /// The child's `not_after` is copied from the parent verbatim,
    /// NOT derived from `parent.not_after - now`. The subtract-then-
    /// re-read-clock approach lost multiple seconds of validity
    /// when the parent was near expiry — the child's `issue()` call
    /// re-reads `current_timestamp()` and computes
    /// `now + (parent.not_after - previous_now)`, which rounds down
    /// by the wall-clock delta between the two reads. Copying
    /// `not_after` avoids the double-read and guarantees the
    /// child's lifetime is `parent.not_after - child.not_before`
    /// exactly.
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

        // Issue a child whose `not_after` matches the parent's.
        // `issue()` stamps `not_before = now`, so the child's
        // effective lifetime is `parent.not_after - now` — the
        // same quantity as before, but computed against a single
        // clock read instead of two. Avoids the near-zero-lifetime
        // bug when the parent is near expiry.
        let now = current_timestamp();
        let mut nonce_bytes = [0u8; 8];
        getrandom::fill(&mut nonce_bytes).expect("getrandom failed");
        let nonce = u64::from_le_bytes(nonce_bytes);

        let mut child = Self {
            issuer: signer.entity_id().clone(),
            subject: new_subject,
            scope: new_scope,
            channel_hash: self.channel_hash,
            not_before: now,
            not_after: self.not_after,
            delegation_depth: self.delegation_depth - 1,
            nonce,
            signature: [0u8; 64],
        };
        let payload = child.signed_payload();
        let sig = signer.sign(&payload);
        child.signature = sig.to_bytes();
        Ok(child)
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
    ///
    /// Rejects buffers whose length is anything other than exactly
    /// [`Self::WIRE_SIZE`]. Previously this method only guarded the
    /// lower bound, silently accepting concatenated or trailing-
    /// garbage payloads — which weakened the wire-format contract
    /// and let malformed blobs parse as valid tokens. Callers
    /// framing tokens inside a larger message must slice to exactly
    /// `WIRE_SIZE` before calling this.
    pub fn from_bytes(data: &[u8]) -> Result<Self, TokenError> {
        if data.len() != Self::WIRE_SIZE {
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
/// Keyed by `(subject EntityId, channel_hash)`. Each slot holds a
/// **list** of tokens — previous versions kept a single token per
/// slot, which silently dropped tokens when the same subject needed
/// multiple distinct scopes on the same channel (e.g. one PUBLISH
/// token and one SUBSCRIBE token). On insert the incoming token
/// replaces any existing entry with an **identical scope bitfield**
/// so a refresh doesn't stack duplicates, but tokens with different
/// scopes coexist.
///
/// Entries are not evicted automatically — callers should check
/// `is_valid()` on retrieved tokens, or call [`Self::evict_expired`]
/// on a cadence.
pub struct TokenCache {
    tokens: DashMap<([u8; 32], u16), Vec<PermissionToken>>,
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
    ///
    /// Tokens with distinct scope bitfields for the same
    /// `(subject, channel_hash)` are stored side-by-side.
    /// A new token with the same scope as an existing entry
    /// **replaces** the existing one — latest-issued wins so
    /// refreshing via re-issue doesn't leak growth.
    pub fn insert(&self, token: PermissionToken) -> Result<(), TokenError> {
        token.verify()?;
        self.insert_unchecked(token);
        Ok(())
    }

    /// Insert a token without verification (for trusted internal use).
    ///
    /// Only use this when the token is known to be valid (e.g., just issued locally).
    pub fn insert_unchecked(&self, token: PermissionToken) {
        let key = (*token.subject.as_bytes(), token.channel_hash);
        let mut entry = self.tokens.entry(key).or_default();
        // Replace any existing token with exactly the same scope;
        // otherwise push so distinct-scope tokens coexist.
        if let Some(slot) = entry.iter_mut().find(|t| t.scope == token.scope) {
            *slot = token;
        } else {
            entry.push(token);
        }
    }

    /// Check if an entity is authorized for an action on a channel.
    ///
    /// Returns `Ok(())` if any cached token for this subject grants
    /// `action`, else an error. Walks the exact-channel slot first,
    /// then the wildcard (`channel_hash = 0`) slot. Within a slot,
    /// any valid token that authorizes the requested action wins —
    /// an expired or otherwise-invalid token in the same slot is
    /// ignored, not blocking.
    pub fn check(
        &self,
        subject: &EntityId,
        action: TokenScope,
        channel_hash: u16,
    ) -> Result<(), TokenError> {
        // Try exact channel match first
        if let Some(slot) = self.tokens.get(&(*subject.as_bytes(), channel_hash)) {
            if slot
                .value()
                .iter()
                .any(|t| t.is_valid().is_ok() && t.authorizes(action, channel_hash))
            {
                return Ok(());
            }
        }
        // Try wildcard (channel_hash = 0)
        if let Some(slot) = self.tokens.get(&(*subject.as_bytes(), 0)) {
            if slot
                .value()
                .iter()
                .any(|t| t.is_valid().is_ok() && t.authorizes(action, channel_hash))
            {
                return Ok(());
            }
        }
        Err(TokenError::NotAuthorized)
    }

    /// Fetch any cached token for `(subject, channel_hash)`. Exact
    /// match only — the wildcard (`channel_hash = 0`) entry is a
    /// separate key. Returns the first valid token in the slot; if
    /// none are valid, returns any entry (so callers can still
    /// inspect for debugging). Callers that need a specific scope
    /// should use [`Self::check`] instead.
    pub fn get(&self, subject: &EntityId, channel_hash: u16) -> Option<PermissionToken> {
        let slot = self.tokens.get(&(*subject.as_bytes(), channel_hash))?;
        let tokens = slot.value();
        // Prefer a currently-valid token; otherwise fall back to
        // the first entry so callers like `net_identity_lookup_token`
        // can still inspect it.
        tokens
            .iter()
            .find(|t| t.is_valid().is_ok())
            .or_else(|| tokens.first())
            .cloned()
    }

    /// Remove expired tokens.
    pub fn evict_expired(&self) {
        let now = current_timestamp();
        self.tokens.retain(|_, slot| {
            slot.retain(|t| t.not_after > now);
            !slot.is_empty()
        });
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

    /// Regression for a cubic-flagged bug that hit every FFI binding:
    /// `token_is_expired` used to call `is_valid()` and match on
    /// `Err(Expired)`, which short-circuited on signature failure.
    /// A tampered + expired token therefore returned `false` ("not
    /// expired") even though the wall-clock was past `not_after`.
    /// `is_expired()` must be a pure time check, independent of the
    /// signature.
    #[test]
    fn is_expired_ignores_signature_tampering() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();

        // Fresh token — not expired.
        let mut token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0,
            3600,
            0,
        );
        assert!(!token.is_expired(), "fresh token is not expired");

        // Construct the bug scenario: backdate `not_after` into the
        // past AND flip a byte in the signature. In practice a
        // tampered packet would arrive over the wire; here we
        // mutate in place so the test doesn't depend on sleeps.
        // Both mutations land outside what `verify()` recomputes —
        // not_after is part of the signed payload, so verify() is
        // already going to fail; the point is that `is_expired()`
        // doesn't care.
        token.not_after = 0;
        token.signature[0] ^= 0xFF;

        // Signature fails (expected).
        assert!(
            token.verify().is_err(),
            "mutated payload / signature must fail verify",
        );

        // Pre-fix pattern: `matches!(is_valid(), Err(Expired))`.
        // `is_valid()` short-circuits on the signature failure and
        // returns `Err(InvalidSignature)`, so the match returns
        // false — this is exactly the bug Cubic flagged.
        assert!(
            !matches!(token.is_valid(), Err(TokenError::Expired)),
            "captures the pre-fix pattern: is_valid() short-circuits \
             on signature, never reaches the time check",
        );

        // Post-fix: `is_expired()` compares time directly and
        // reports `true` regardless of signature state.
        assert!(
            token.is_expired(),
            "is_expired() must be a pure time check, independent \
             of signature validity",
        );
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
        let _ = cache.insert(token);

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
        let _ = cache.insert(token);

        // Should match any channel
        assert!(cache
            .check(subject.entity_id(), TokenScope::PUBLISH, 0xABCD)
            .is_ok());
        assert!(cache
            .check(subject.entity_id(), TokenScope::PUBLISH, 0x1234)
            .is_ok());
    }

    // ---- Regression tests for Cubic AI findings ----

    #[test]
    fn test_regression_wildcard_fallback_not_blocked_by_expired_channel_token() {
        // Regression: token.is_valid()? short-circuited on an expired
        // channel-specific token, preventing the wildcard fallback from
        // being reached.
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();
        let cache = TokenCache::new();

        // Insert an expired channel-specific token
        let mut expired_token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0xABCD,
            0, // expires immediately
            0,
        );
        // Force expiry by setting not_after to the past
        expired_token.not_after = 0;
        // Re-sign with the modified field
        let payload = expired_token.signed_payload();
        expired_token.signature = issuer.sign(&payload).to_bytes();
        cache.insert_unchecked(expired_token);

        // Insert a valid wildcard token
        let wildcard_token = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0, // wildcard
            3600,
            0,
        );
        cache.insert_unchecked(wildcard_token);

        // The wildcard should be reached despite the expired channel token
        assert!(
            cache
                .check(subject.entity_id(), TokenScope::PUBLISH, 0xABCD)
                .is_ok(),
            "wildcard fallback must not be blocked by expired channel-specific token"
        );
    }

    #[test]
    fn test_regression_delegate_rejects_expired_parent() {
        // Regression: delegate() minted child tokens from an invalid parent
        // because it never called is_valid() on the parent.
        let root = EntityKeypair::generate();
        let node_a = EntityKeypair::generate();
        let node_b = EntityKeypair::generate();

        let mut token = PermissionToken::issue(
            &root,
            node_a.entity_id().clone(),
            TokenScope::ALL,
            0,
            3600,
            2,
        );
        // Force expiry
        token.not_after = 0;
        let payload = token.signed_payload();
        token.signature = root.sign(&payload).to_bytes();

        let result = token.delegate(&node_a, node_b.entity_id().clone(), TokenScope::PUBLISH);
        assert_eq!(
            result.unwrap_err(),
            TokenError::Expired,
            "delegation from expired parent must be rejected"
        );
    }

    #[test]
    fn test_regression_insert_rejects_tampered_token() {
        // Regression: insert() accepted self-signed/tampered tokens
        // because it did not verify the signature.
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
        // Tamper: change scope after signing
        token.scope = TokenScope::ADMIN;

        let cache = TokenCache::new();
        assert!(
            cache.insert(token).is_err(),
            "insert must reject tampered token"
        );
        assert_eq!(cache.len(), 0, "tampered token must not be cached");
    }

    // ========================================================================
    // Cubic-flagged P1/P2 regressions
    // ========================================================================

    /// Regression for a cubic-flagged P1: TokenCache used to key on
    /// `(subject, channel_hash)` and store a single token per slot,
    /// so inserting a SUBSCRIBE token after a PUBLISH token
    /// silently overwrote the earlier one. Both must coexist.
    #[test]
    fn cache_coexists_tokens_of_different_scopes_for_same_channel() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();
        let channel = 0xABCD;

        let publish_tok = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            channel,
            3600,
            0,
        );
        let subscribe_tok = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::SUBSCRIBE,
            channel,
            3600,
            0,
        );

        let cache = TokenCache::new();
        cache.insert(publish_tok).expect("insert publish");
        cache.insert(subscribe_tok).expect("insert subscribe");

        // Both authorizations must pass — the second insert used to
        // clobber the first because the cache was keyed without
        // considering scope.
        assert!(
            cache.check(subject.entity_id(), TokenScope::PUBLISH, channel).is_ok(),
            "publish auth lost after subscribe insert",
        );
        assert!(
            cache.check(subject.entity_id(), TokenScope::SUBSCRIBE, channel).is_ok(),
            "subscribe auth lost",
        );
    }

    /// Regression for the other half of the cache semantic: issuing
    /// a SECOND token with the same scope as an existing one
    /// should **replace** it, not stack. Otherwise repeated refreshes
    /// leak linear memory.
    #[test]
    fn cache_same_scope_reinsert_replaces_not_stacks() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();
        let channel = 0xABCD;

        let cache = TokenCache::new();
        for _ in 0..10 {
            let tok = PermissionToken::issue(
                &issuer,
                subject.entity_id().clone(),
                TokenScope::SUBSCRIBE,
                channel,
                3600,
                0,
            );
            cache.insert(tok).expect("insert");
        }
        // All ten had scope=SUBSCRIBE. The cache should hold one
        // entry total (the most recent), not ten.
        assert_eq!(
            cache.len(),
            1,
            "repeated inserts with the same scope must replace, not stack",
        );
    }

    /// Regression for a cubic-flagged P2: `from_bytes` used to
    /// accept any buffer ≥ WIRE_SIZE, silently ignoring trailing
    /// bytes. Concatenated / corrupted payloads must fail cleanly.
    #[test]
    fn from_bytes_rejects_trailing_garbage() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();
        let tok = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0,
            3600,
            0,
        );
        let mut bytes = tok.to_bytes();
        assert_eq!(bytes.len(), PermissionToken::WIRE_SIZE);
        // Fresh bytes parse fine.
        assert!(PermissionToken::from_bytes(&bytes).is_ok());

        // Append trailing garbage — parser must now refuse.
        bytes.push(0xFF);
        assert!(
            matches!(
                PermissionToken::from_bytes(&bytes),
                Err(TokenError::InvalidFormat)
            ),
            "trailing byte must reject as InvalidFormat",
        );

        // Truncate by one — also refused (already was, but lock in).
        let truncated = &tok.to_bytes()[..PermissionToken::WIRE_SIZE - 1];
        assert!(matches!(
            PermissionToken::from_bytes(truncated),
            Err(TokenError::InvalidFormat)
        ));
    }

    /// Regression for a cubic-flagged P1: `issue()` used unchecked
    /// `now + duration_secs`, which panics in debug builds on
    /// large TTL. Saturating add yields a never-expiring token
    /// instead of crashing.
    #[test]
    fn issue_with_huge_ttl_saturates_rather_than_panics() {
        let issuer = EntityKeypair::generate();
        let subject = EntityKeypair::generate();
        let tok = PermissionToken::issue(
            &issuer,
            subject.entity_id().clone(),
            TokenScope::PUBLISH,
            0,
            u64::MAX,
            0,
        );
        assert_eq!(
            tok.not_after,
            u64::MAX,
            "TTL=u64::MAX must saturate, not wrap or panic",
        );
        assert!(!tok.is_expired());
        // Signature is still valid.
        assert!(tok.verify().is_ok());
    }

    /// Regression for a cubic-flagged P2: `delegate()` computed the
    /// child's TTL as `parent.not_after - current_timestamp()` and
    /// then passed that duration back through `issue()`, which
    /// re-reads `current_timestamp()`. When the parent was close
    /// to expiry the double-read shaved meaningful lifetime off
    /// the child — in the worst case, a child token born already
    /// expired. The fix copies `parent.not_after` directly.
    #[test]
    fn delegate_preserves_parent_not_after() {
        let a = EntityKeypair::generate();
        let b = EntityKeypair::generate();
        let c = EntityKeypair::generate();

        let parent = PermissionToken::issue(
            &a,
            b.entity_id().clone(),
            TokenScope::PUBLISH.union(TokenScope::DELEGATE),
            0,
            3600,
            2,
        );

        let child = parent
            .delegate(&b, c.entity_id().clone(), TokenScope::PUBLISH)
            .expect("delegate");

        assert_eq!(
            child.not_after, parent.not_after,
            "child's not_after must equal parent's, not some smaller value \
             derived from a second clock read",
        );
        // child.not_before was stamped by the child's own clock
        // read, so it's ≥ parent.not_before — which is correct.
        assert!(child.not_before >= parent.not_before);
        assert!(child.verify().is_ok());
    }
}
