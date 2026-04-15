//! Layer 1: Trust & Identity for Net.
//!
//! This module provides cryptographic identity, origin binding, and
//! permission tokens for the mesh. All identifiers (node_id, origin_hash)
//! are derived from ed25519 public keys.

mod entity;
mod origin;
mod token;

pub use entity::{EntityError, EntityId, EntityKeypair};
pub use origin::OriginStamp;
pub use token::{PermissionToken, TokenCache, TokenError, TokenScope};
