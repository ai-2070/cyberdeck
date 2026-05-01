//! Subprotocol registry — maps protocol IDs to descriptors.
//!
//! The registry tracks which subprotocols this node supports. When a
//! subprotocol is registered, a capability tag is added so that other
//! nodes can discover support via the existing CapabilityIndex.

use dashmap::DashMap;

use super::descriptor::{SubprotocolDescriptor, SubprotocolVersion};
use super::stream_window::SUBPROTOCOL_STREAM_WINDOW;
use crate::adapter::net::behavior::capability::{CapabilityFilter, CapabilitySet};
use crate::adapter::net::compute::SUBPROTOCOL_MIGRATION;
use crate::adapter::net::state::causal::{SUBPROTOCOL_CAUSAL, SUBPROTOCOL_SNAPSHOT};

/// Registry of subprotocols supported by this node.
///
/// Keyed by `subprotocol_id: u16`. Does NOT store the daemon itself
/// (that stays in `DaemonRegistry`). This maps protocol IDs to metadata
/// and provides query methods.
pub struct SubprotocolRegistry {
    /// id -> descriptor
    entries: DashMap<u16, SubprotocolDescriptor>,
}

impl SubprotocolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
        }
    }

    /// Create a registry pre-populated with core subprotocols.
    pub fn with_defaults() -> Self {
        let reg = Self::new();
        reg.register(SubprotocolDescriptor::new(
            SUBPROTOCOL_CAUSAL,
            "causal",
            SubprotocolVersion::new(1, 0),
        ));
        reg.register(SubprotocolDescriptor::new(
            SUBPROTOCOL_SNAPSHOT,
            "snapshot",
            SubprotocolVersion::new(1, 0),
        ));
        reg.register(SubprotocolDescriptor::new(
            SUBPROTOCOL_MIGRATION,
            "migration",
            SubprotocolVersion::new(1, 0),
        ));
        reg.register(SubprotocolDescriptor::new(
            super::SUBPROTOCOL_NEGOTIATION,
            "negotiation",
            SubprotocolVersion::new(1, 0),
        ));
        reg.register(SubprotocolDescriptor::new(
            SUBPROTOCOL_STREAM_WINDOW,
            "stream-window",
            SubprotocolVersion::new(1, 0),
        ));
        reg
    }

    /// Register a subprotocol.
    ///
    /// If a subprotocol with the same ID is already registered, it is
    /// replaced (version upgrade). Returns the previous descriptor if any.
    pub fn register(&self, descriptor: SubprotocolDescriptor) -> Option<SubprotocolDescriptor> {
        self.entries.insert(descriptor.id, descriptor)
    }

    /// Unregister a subprotocol by ID.
    pub fn unregister(&self, id: u16) -> Option<SubprotocolDescriptor> {
        self.entries.remove(&id).map(|(_, d)| d)
    }

    /// Look up a subprotocol by ID.
    pub fn get(
        &self,
        id: u16,
    ) -> Option<dashmap::mapref::one::Ref<'_, u16, SubprotocolDescriptor>> {
        self.entries.get(&id)
    }

    /// Check if a handler is present for this subprotocol (not just forwarding).
    pub fn is_handled(&self, id: u16) -> bool {
        self.entries.get(&id).is_some_and(|d| d.handler_present)
    }

    /// Check if a subprotocol ID is registered (handler or forwarding-only).
    pub fn is_registered(&self, id: u16) -> bool {
        self.entries.contains_key(&id)
    }

    /// List all registered subprotocols.
    pub fn list(&self) -> Vec<SubprotocolDescriptor> {
        self.entries.iter().map(|e| e.value().clone()).collect()
    }

    /// Number of registered subprotocols.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Build a `CapabilityFilter` that finds nodes supporting a given subprotocol.
    ///
    /// Returns a filter requiring the tag `subprotocol:0x{id:04x}`.
    pub fn capability_filter_for(id: u16) -> CapabilityFilter {
        CapabilityFilter::new().require_tag(format!("subprotocol:{:#06x}", id))
    }

    /// Get all capability tags for registered subprotocols.
    ///
    /// Add these to the local node's `CapabilitySet` and re-broadcast
    /// via `CapabilityAd` so other nodes can discover support.
    ///
    /// Returned in ascending subprotocol-id order, matching
    /// `SubprotocolManifest::from_registry`'s sort. Without this,
    /// the two channels' byte streams diverge — fine today, but a
    /// future digest-dedup optimisation that hashes either output
    /// would silently disagree with the other side.
    pub fn capability_tags(&self) -> Vec<String> {
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.handler_present)
            .map(|e| (e.id, e.capability_tag()))
            .collect();
        entries.sort_by_key(|(id, _)| *id);
        entries.into_iter().map(|(_, tag)| tag).collect()
    }

    /// Enrich a `CapabilitySet` with tags for all handled subprotocols.
    ///
    /// Call this when building the local node's capability set before
    /// broadcasting via `CapabilityAnnouncement` or `CapabilityAd`.
    /// This ensures other nodes can discover which subprotocols this
    /// node supports (including migration) through the capability graph.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let caps = CapabilitySet::new();
    /// let caps = subprotocol_registry.enrich_capabilities(caps);
    /// // caps now has tags like "subprotocol:0x0400", "subprotocol:0x0500", etc.
    /// ```
    pub fn enrich_capabilities(&self, mut caps: CapabilitySet) -> CapabilitySet {
        for tag in self.capability_tags() {
            caps = caps.add_tag(tag);
        }
        caps
    }
}

impl Default for SubprotocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SubprotocolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubprotocolRegistry")
            .field("count", &self.entries.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::behavior::capability::CapabilitySet;

    #[test]
    fn test_empty_registry() {
        let reg = SubprotocolRegistry::new();
        assert_eq!(reg.count(), 0);
        assert!(!reg.is_handled(0x0400));
        assert!(!reg.is_registered(0x0400));
    }

    #[test]
    fn test_with_defaults() {
        let reg = SubprotocolRegistry::with_defaults();
        assert!(reg.is_handled(SUBPROTOCOL_CAUSAL));
        assert!(reg.is_handled(SUBPROTOCOL_SNAPSHOT));
        assert!(reg.is_handled(SUBPROTOCOL_MIGRATION));
        assert!(reg.count() >= 4); // causal, snapshot, migration, negotiation
    }

    #[test]
    fn test_register_and_lookup() {
        let reg = SubprotocolRegistry::new();
        let desc = SubprotocolDescriptor::new(0x1000, "vendor-test", SubprotocolVersion::new(1, 0));

        assert!(reg.register(desc).is_none()); // first registration
        assert!(reg.is_handled(0x1000));
        assert!(reg.is_registered(0x1000));

        let retrieved = reg.get(0x1000).unwrap();
        assert_eq!(retrieved.name, "vendor-test");
    }

    #[test]
    fn test_register_upgrade() {
        let reg = SubprotocolRegistry::new();

        let v1 = SubprotocolDescriptor::new(0x1000, "test", SubprotocolVersion::new(1, 0));
        reg.register(v1);

        let v2 = SubprotocolDescriptor::new(0x1000, "test", SubprotocolVersion::new(2, 0));
        let old = reg.register(v2);

        assert!(old.is_some());
        assert_eq!(old.unwrap().version, SubprotocolVersion::new(1, 0));

        let current = reg.get(0x1000).unwrap();
        assert_eq!(current.version, SubprotocolVersion::new(2, 0));
    }

    #[test]
    fn test_unregister() {
        let reg = SubprotocolRegistry::new();
        reg.register(SubprotocolDescriptor::new(
            0x1000,
            "test",
            SubprotocolVersion::new(1, 0),
        ));

        let removed = reg.unregister(0x1000);
        assert!(removed.is_some());
        assert!(!reg.is_registered(0x1000));
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_forwarding_only() {
        let reg = SubprotocolRegistry::new();
        let desc = SubprotocolDescriptor::new(0x2000, "remote-only", SubprotocolVersion::new(1, 0))
            .forwarding_only();

        reg.register(desc);
        assert!(reg.is_registered(0x2000));
        assert!(!reg.is_handled(0x2000)); // no local handler
    }

    #[test]
    fn test_capability_tags() {
        let reg = SubprotocolRegistry::new();
        reg.register(SubprotocolDescriptor::new(
            0x1000,
            "handled",
            SubprotocolVersion::new(1, 0),
        ));
        reg.register(
            SubprotocolDescriptor::new(0x2000, "forwarded", SubprotocolVersion::new(1, 0))
                .forwarding_only(),
        );

        let tags = reg.capability_tags();
        assert_eq!(tags.len(), 1); // only handled protocols get tags
        assert!(tags[0].contains("0x1000"));
    }

    #[test]
    fn test_capability_filter_for() {
        let filter = SubprotocolRegistry::capability_filter_for(0x0400);
        // The filter should require the subprotocol tag
        assert!(!filter.require_tags.is_empty());
        assert_eq!(filter.require_tags[0], "subprotocol:0x0400");
    }

    #[test]
    fn test_list() {
        let reg = SubprotocolRegistry::new();
        reg.register(SubprotocolDescriptor::new(
            0x1000,
            "a",
            SubprotocolVersion::new(1, 0),
        ));
        reg.register(SubprotocolDescriptor::new(
            0x2000,
            "b",
            SubprotocolVersion::new(2, 0),
        ));

        let list = reg.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_enrich_capabilities() {
        let reg = SubprotocolRegistry::new();
        reg.register(SubprotocolDescriptor::new(
            0x0500,
            "migration",
            SubprotocolVersion::new(1, 0),
        ));
        reg.register(
            SubprotocolDescriptor::new(0x2000, "forwarded", SubprotocolVersion::new(1, 0))
                .forwarding_only(),
        );

        let caps = CapabilitySet::new();
        let enriched = reg.enrich_capabilities(caps);

        // Only handled subprotocols get tags
        assert!(enriched.has_tag("subprotocol:0x0500"));
        assert!(!enriched.has_tag("subprotocol:0x2000")); // forwarding-only excluded
    }

    #[test]
    fn test_enrich_capabilities_with_defaults() {
        let reg = SubprotocolRegistry::with_defaults();
        let caps = reg.enrich_capabilities(CapabilitySet::new());

        // All default subprotocols should have tags
        assert!(caps.has_tag("subprotocol:0x0400")); // causal
        assert!(caps.has_tag("subprotocol:0x0401")); // snapshot
        assert!(caps.has_tag("subprotocol:0x0500")); // migration
        assert!(caps.has_tag("subprotocol:0x0600")); // negotiation
    }

    #[test]
    fn test_enrich_preserves_existing_tags() {
        let reg = SubprotocolRegistry::new();
        reg.register(SubprotocolDescriptor::new(
            0x0500,
            "migration",
            SubprotocolVersion::new(1, 0),
        ));

        let caps = CapabilitySet::new().add_tag("custom:my-tag");
        let enriched = reg.enrich_capabilities(caps);

        assert!(enriched.has_tag("custom:my-tag"));
        assert!(enriched.has_tag("subprotocol:0x0500"));
    }
}
