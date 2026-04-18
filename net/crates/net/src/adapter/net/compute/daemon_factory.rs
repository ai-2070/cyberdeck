//! Registry mapping daemon `origin_hash` to the pieces needed to restore
//! the daemon from a snapshot: a constructor closure, the matching
//! `EntityKeypair`, and a `DaemonHostConfig`.
//!
//! `MigrationTargetHandler::restore_snapshot` takes a
//! `daemon_factory: FnOnce() -> Box<dyn MeshDaemon>` plus a keypair and
//! config. These cannot be serialized across the wire, so the subprotocol
//! handler has to resolve them locally when a snapshot arrives. This
//! registry is that local resolver.
//!
//! Populate the registry at node startup with one entry per daemon type
//! the node may be asked to host.
//!
//! # Keypair provisioning (out of scope here)
//!
//! Secure transfer of a daemon's `EntityKeypair` from source to target is a
//! separate security problem. For now, callers provision the keypair in the
//! factory registry out-of-band (same shape the existing integration tests
//! use).

use std::sync::Arc;

use dashmap::DashMap;

use super::daemon::{DaemonHostConfig, MeshDaemon};
use crate::adapter::net::identity::EntityKeypair;

/// Bundle required to reconstruct a daemon on the target.
pub struct FactoryEntry {
    /// Constructor for a fresh, unrestored daemon instance.
    pub factory: Box<dyn Fn() -> Box<dyn MeshDaemon> + Send + Sync>,
    /// The daemon's signing keypair (must match `snapshot.entity_id`).
    pub keypair: EntityKeypair,
    /// Host configuration to apply to the restored daemon.
    pub config: DaemonHostConfig,
}

/// Thread-safe registry of daemon factories keyed by `origin_hash`.
#[derive(Default)]
pub struct DaemonFactoryRegistry {
    entries: DashMap<u32, FactoryEntry>,
}

impl DaemonFactoryRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a factory for a daemon type.
    ///
    /// The `origin_hash` must equal `keypair.origin_hash()`; this is checked
    /// at registration time to catch mismatches early.
    pub fn register<F>(
        &self,
        origin_hash: u32,
        keypair: EntityKeypair,
        config: DaemonHostConfig,
        factory: F,
    ) where
        F: Fn() -> Box<dyn MeshDaemon> + Send + Sync + 'static,
    {
        debug_assert_eq!(
            origin_hash,
            keypair.origin_hash(),
            "origin_hash must match keypair"
        );
        self.entries.insert(
            origin_hash,
            FactoryEntry {
                factory: Box::new(factory),
                keypair,
                config,
            },
        );
    }

    /// Consume the factory entry for `origin_hash`, if any. Returns `None`
    /// when no factory has been registered.
    ///
    /// Consumption (rather than cloning) surfaces double-restore bugs: if a
    /// migration arrives a second time without the caller re-registering,
    /// this returns `None` and the subprotocol handler fails the migration
    /// cleanly instead of silently restoring twice.
    pub fn take(&self, origin_hash: u32) -> Option<FactoryEntry> {
        self.entries.remove(&origin_hash).map(|(_, entry)| entry)
    }

    /// Whether a factory is currently registered for `origin_hash`.
    pub fn contains(&self, origin_hash: u32) -> bool {
        self.entries.contains_key(&origin_hash)
    }

    /// An empty shared registry, for handlers that are never expected to
    /// restore daemons (e.g., source-only nodes).
    pub fn empty() -> Arc<Self> {
        Arc::new(Self::default())
    }
}

impl std::fmt::Debug for DaemonFactoryRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DaemonFactoryRegistry")
            .field("entries", &self.entries.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::behavior::capability::CapabilityFilter;
    use crate::adapter::net::compute::DaemonError;
    use crate::adapter::net::state::causal::CausalEvent;
    use bytes::Bytes;

    struct Stub;
    impl MeshDaemon for Stub {
        fn name(&self) -> &str {
            "stub"
        }
        fn requirements(&self) -> CapabilityFilter {
            CapabilityFilter::default()
        }
        fn process(&mut self, _: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
            Ok(vec![])
        }
    }

    #[test]
    fn register_and_take_returns_entry_once() {
        let reg = DaemonFactoryRegistry::new();
        let kp = EntityKeypair::generate();
        let origin = kp.origin_hash();

        reg.register(origin, kp, DaemonHostConfig::default(), || Box::new(Stub));
        assert!(reg.contains(origin));

        let entry = reg.take(origin).expect("factory should be present");
        let _daemon = (entry.factory)();
        assert!(!reg.contains(origin), "take must consume the entry");
        assert!(reg.take(origin).is_none());
    }

    #[test]
    fn take_missing_returns_none() {
        let reg = DaemonFactoryRegistry::new();
        assert!(reg.take(0xDEADBEEF).is_none());
    }
}
