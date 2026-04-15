//! Layer 6: Subprotocols for Net.
//!
//! Formalizes the `subprotocol_id: u16` field in the Net header. Provides
//! a registry mapping IDs to handler metadata, version negotiation between
//! peers, and capability-aware routing via tags.
//!
//! Opaque forwarding is the default — nodes that don't understand a
//! subprotocol forward packets without inspecting the payload.

mod descriptor;
mod negotiation;
mod registry;

/// Subprotocol ID for negotiation messages.
pub const SUBPROTOCOL_NEGOTIATION: u16 = 0x0600;

pub use descriptor::{SubprotocolDescriptor, SubprotocolVersion, MANIFEST_ENTRY_SIZE};
pub use negotiation::{negotiate, ManifestEntry, NegotiatedSet, SubprotocolManifest};
pub use registry::SubprotocolRegistry;
