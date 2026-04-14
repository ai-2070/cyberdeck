//! Subprotocol descriptor and version types.
//!
//! A subprotocol is identified by a `u16` ID in the BLTP header and
//! described by a `SubprotocolDescriptor` carrying name, version, and
//! compatibility metadata.

use bytes::{Buf, BufMut};

/// Semantic version for subprotocol negotiation.
///
/// Two peers are compatible if both peers' version >= the other's
/// `min_compatible`. Wire format: 2 bytes (major, minor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubprotocolVersion {
    /// Major version — breaking changes increment this.
    pub major: u8,
    /// Minor version — backward-compatible changes.
    pub minor: u8,
}

impl SubprotocolVersion {
    /// Create a new version.
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Check if this version is compatible with a peer's minimum requirement.
    #[inline]
    pub fn satisfies(self, min_required: Self) -> bool {
        self >= min_required
    }

    /// Serialize to 2 bytes.
    #[inline]
    pub fn to_bytes(self) -> [u8; 2] {
        [self.major, self.minor]
    }

    /// Deserialize from 2 bytes.
    #[inline]
    pub fn from_bytes(data: &[u8; 2]) -> Self {
        Self {
            major: data[0],
            minor: data[1],
        }
    }
}

impl std::fmt::Display for SubprotocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Metadata describing a registered subprotocol.
#[derive(Debug, Clone)]
pub struct SubprotocolDescriptor {
    /// Unique protocol ID (from BLTP header `subprotocol_id` field).
    pub id: u16,
    /// Human-readable name (e.g., "causal", "migration", "vendor-x-inference").
    pub name: String,
    /// Version of this handler.
    pub version: SubprotocolVersion,
    /// Minimum compatible version accepted from peers.
    pub min_compatible: SubprotocolVersion,
    /// Whether this node can process packets for this subprotocol
    /// (false = opaque forwarding only, no local handler).
    pub handler_present: bool,
}

impl SubprotocolDescriptor {
    /// Create a new descriptor.
    pub fn new(id: u16, name: impl Into<String>, version: SubprotocolVersion) -> Self {
        Self {
            id,
            name: name.into(),
            version,
            min_compatible: version,
            handler_present: true,
        }
    }

    /// Set the minimum compatible version.
    pub fn with_min_compatible(mut self, min: SubprotocolVersion) -> Self {
        self.min_compatible = min;
        self
    }

    /// Mark as forwarding-only (no local handler).
    pub fn forwarding_only(mut self) -> Self {
        self.handler_present = false;
        self
    }

    /// Check if two descriptors are version-compatible.
    ///
    /// Both sides must satisfy the other's minimum requirement.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.id == other.id
            && self.version.satisfies(other.min_compatible)
            && other.version.satisfies(self.min_compatible)
    }

    /// Capability tag for this subprotocol (e.g., "subprotocol:0x0400").
    pub fn capability_tag(&self) -> String {
        format!("subprotocol:{:#06x}", self.id)
    }
}

impl std::fmt::Display for SubprotocolDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:#06x}) v{}", self.name, self.id, self.version)
    }
}

/// Wire format entry for manifest exchange (6 bytes per entry).
///
/// ```text
/// id:             2 bytes (u16)
/// version:        2 bytes (major, minor)
/// min_compatible: 2 bytes (major, minor)
/// ```
pub const MANIFEST_ENTRY_SIZE: usize = 6;

/// Serialize a descriptor to a manifest entry (6 bytes).
pub fn write_manifest_entry(desc: &SubprotocolDescriptor, buf: &mut impl BufMut) {
    buf.put_u16_le(desc.id);
    buf.put_slice(&desc.version.to_bytes());
    buf.put_slice(&desc.min_compatible.to_bytes());
}

/// Deserialize a manifest entry from bytes.
pub fn read_manifest_entry(
    buf: &mut impl Buf,
) -> Option<(u16, SubprotocolVersion, SubprotocolVersion)> {
    if buf.remaining() < MANIFEST_ENTRY_SIZE {
        return None;
    }
    let id = buf.get_u16_le();
    let version = SubprotocolVersion::new(buf.get_u8(), buf.get_u8());
    let min_compat = SubprotocolVersion::new(buf.get_u8(), buf.get_u8());
    Some((id, version, min_compat))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_ordering() {
        let v1_0 = SubprotocolVersion::new(1, 0);
        let v1_1 = SubprotocolVersion::new(1, 1);
        let v2_0 = SubprotocolVersion::new(2, 0);

        assert!(v1_0 < v1_1);
        assert!(v1_1 < v2_0);
        assert!(v1_0 < v2_0);
    }

    #[test]
    fn test_version_satisfies() {
        let v1_0 = SubprotocolVersion::new(1, 0);
        let v1_1 = SubprotocolVersion::new(1, 1);
        let v2_0 = SubprotocolVersion::new(2, 0);

        assert!(v1_1.satisfies(v1_0)); // 1.1 >= 1.0
        assert!(v2_0.satisfies(v1_0)); // 2.0 >= 1.0
        assert!(!v1_0.satisfies(v1_1)); // 1.0 < 1.1
    }

    #[test]
    fn test_descriptor_compatibility() {
        let a = SubprotocolDescriptor::new(0x0400, "causal", SubprotocolVersion::new(1, 1))
            .with_min_compatible(SubprotocolVersion::new(1, 0));
        let b = SubprotocolDescriptor::new(0x0400, "causal", SubprotocolVersion::new(1, 0));

        assert!(a.is_compatible_with(&b)); // a(1.1) >= b.min(1.0) AND b(1.0) >= a.min(1.0)
    }

    #[test]
    fn test_descriptor_incompatible() {
        let a = SubprotocolDescriptor::new(0x0400, "causal", SubprotocolVersion::new(2, 0))
            .with_min_compatible(SubprotocolVersion::new(2, 0));
        let b = SubprotocolDescriptor::new(0x0400, "causal", SubprotocolVersion::new(1, 0));

        assert!(!a.is_compatible_with(&b)); // b(1.0) < a.min(2.0)
    }

    #[test]
    fn test_descriptor_different_id() {
        let a = SubprotocolDescriptor::new(0x0400, "causal", SubprotocolVersion::new(1, 0));
        let b = SubprotocolDescriptor::new(0x0500, "migration", SubprotocolVersion::new(1, 0));

        assert!(!a.is_compatible_with(&b));
    }

    #[test]
    fn test_capability_tag() {
        let d = SubprotocolDescriptor::new(0x0400, "causal", SubprotocolVersion::new(1, 0));
        assert_eq!(d.capability_tag(), "subprotocol:0x0400");
    }

    #[test]
    fn test_manifest_entry_roundtrip() {
        let desc = SubprotocolDescriptor::new(0x1234, "test", SubprotocolVersion::new(3, 7))
            .with_min_compatible(SubprotocolVersion::new(2, 0));

        let mut buf = Vec::new();
        write_manifest_entry(&desc, &mut buf);
        assert_eq!(buf.len(), MANIFEST_ENTRY_SIZE);

        let mut cursor = &buf[..];
        let (id, version, min_compat) = read_manifest_entry(&mut cursor).unwrap();
        assert_eq!(id, 0x1234);
        assert_eq!(version, SubprotocolVersion::new(3, 7));
        assert_eq!(min_compat, SubprotocolVersion::new(2, 0));
    }

    #[test]
    fn test_version_display() {
        assert_eq!(format!("{}", SubprotocolVersion::new(1, 2)), "1.2");
    }

    #[test]
    fn test_descriptor_display() {
        let d = SubprotocolDescriptor::new(0x0400, "causal", SubprotocolVersion::new(1, 0));
        assert_eq!(format!("{}", d), "causal(0x0400) v1.0");
    }
}
