//! Subprotocol descriptor and version types.
//!
//! A subprotocol is identified by a `u16` ID in the Net header and
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
    /// Unique protocol ID (from Net header `subprotocol_id` field).
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
    ///
    /// BUG #132: enforces the wire-format invariant
    /// `min_compatible <= version` — pre-fix, a descriptor could be
    /// built with `min_compatible > version`, which would break
    /// `is_compatible_with`'s contract (every honest peer
    /// computes `local.version.satisfies(other.min_compatible)`,
    /// which silently fails for any version of `local` once
    /// `other.min_compatible > other.version`). On the wire-format
    /// side this enabled a phantom-incompatibility DoS where a
    /// peer advertised `min_compatible=255.255` against
    /// `version=1.0` and unilaterally evicted the subprotocol
    /// from negotiation. The constructor (`new`) initializes
    /// `min_compatible = version` so the invariant holds by
    /// default; this setter clamps `min` to `self.version` if a
    /// caller passes a higher value.
    pub fn with_min_compatible(mut self, min: SubprotocolVersion) -> Self {
        self.min_compatible = if min > self.version {
            self.version
        } else {
            min
        };
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
///
/// BUG #132: now rejects entries that violate the wire-format
/// invariant `min_compatible <= version`. Pre-fix, a peer could
/// advertise `version=1.0, min_compatible=255.255` and every
/// honest peer's `negotiate()` would mark the subprotocol
/// `incompatible` (because `local.version.satisfies(remote.min)`
/// fails for any local), unilaterally evicting that subprotocol
/// from negotiation between the victim and its peers — a
/// phantom-incompatibility DoS that requires no actual presence
/// on the channel. Returning `None` for such entries makes them
/// surface as a parse error to the caller (the manifest is
/// already structured to skip parse failures gracefully).
pub fn read_manifest_entry(
    buf: &mut impl Buf,
) -> Option<(u16, SubprotocolVersion, SubprotocolVersion)> {
    if buf.remaining() < MANIFEST_ENTRY_SIZE {
        return None;
    }
    let id = buf.get_u16_le();
    let version = SubprotocolVersion::new(buf.get_u8(), buf.get_u8());
    let min_compat = SubprotocolVersion::new(buf.get_u8(), buf.get_u8());
    if min_compat > version {
        return None;
    }
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

    // ========================================================================
    // BUG #132: read_manifest_entry / with_min_compatible must reject
    // min_compatible > version (phantom-incompatibility DoS)
    // ========================================================================

    /// A manifest entry advertising `version=1.0, min_compat=255.255`
    /// is rejected by `read_manifest_entry`. Pre-fix, every honest
    /// peer would mark the subprotocol `incompatible` (because
    /// `local.version.satisfies(remote.min_compat)` fails for any
    /// local), letting an attacker unilaterally evict subprotocols.
    #[test]
    fn read_manifest_entry_rejects_min_compatible_above_version() {
        // Hand-craft a manifest entry where min_compat > version.
        // Wire layout: id(2) | version(2) | min_compat(2)
        let mut buf = Vec::new();
        buf.extend_from_slice(&0x1234u16.to_le_bytes()); // id
        buf.extend_from_slice(&[1, 0]); // version 1.0
        buf.extend_from_slice(&[255, 255]); // min_compat 255.255

        let mut cursor = &buf[..];
        let parsed = read_manifest_entry(&mut cursor);
        assert!(
            parsed.is_none(),
            "read_manifest_entry must reject min_compat > version (BUG #132)",
        );
    }

    /// `min_compatible == version` is accepted (it's the default
    /// produced by `SubprotocolDescriptor::new`). Pins the
    /// inclusive boundary so a future tightening that flips the
    /// `>` to `>=` doesn't reject legitimate descriptors that
    /// haven't bumped past their floor yet.
    #[test]
    fn read_manifest_entry_accepts_min_compatible_equal_to_version() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&0x4242u16.to_le_bytes());
        buf.extend_from_slice(&[3, 7]);
        buf.extend_from_slice(&[3, 7]);

        let mut cursor = &buf[..];
        let (id, version, min_compat) =
            read_manifest_entry(&mut cursor).expect("equal min_compat must be accepted");
        assert_eq!(id, 0x4242);
        assert_eq!(version, SubprotocolVersion::new(3, 7));
        assert_eq!(min_compat, SubprotocolVersion::new(3, 7));
    }

    /// `with_min_compatible` clamps to `self.version` instead of
    /// allowing a higher floor than the descriptor's own version.
    /// Without the clamp, a local builder could produce a
    /// descriptor that violates `is_compatible_with`'s wire-format
    /// contract (no peer at any version could satisfy
    /// `min > version`).
    #[test]
    fn with_min_compatible_clamps_to_version() {
        let desc = SubprotocolDescriptor::new(0x1000, "x", SubprotocolVersion::new(1, 0))
            .with_min_compatible(SubprotocolVersion::new(2, 5));
        assert_eq!(
            desc.min_compatible,
            SubprotocolVersion::new(1, 0),
            "with_min_compatible must clamp to self.version",
        );
    }

    /// A legitimate downward-floor `min_compat <= version` is
    /// preserved by the clamp.
    #[test]
    fn with_min_compatible_preserves_lower_floor() {
        let desc = SubprotocolDescriptor::new(0x1000, "x", SubprotocolVersion::new(2, 5))
            .with_min_compatible(SubprotocolVersion::new(1, 0));
        assert_eq!(desc.min_compatible, SubprotocolVersion::new(1, 0));
    }
}
