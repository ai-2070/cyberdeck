//! Capability-broadcast subprotocol identifiers.
//!
//! The wire payload is a [`super::capability::CapabilityAnnouncement`]
//! serialized via its `to_bytes` / `from_bytes` codec. Dispatch +
//! index integration live on `MeshNode` (see `mesh.rs`); this module
//! is intentionally small so the subprotocol id + its payload type
//! have one shared home.

/// Subprotocol id for `CapabilityAnnouncement` packets. Adjacent to
/// the channel-membership id (0x0A00) and stream-window id (0x0B00)
/// to keep the allocated range contiguous.
pub const SUBPROTOCOL_CAPABILITY_ANN: u16 = 0x0C00;
