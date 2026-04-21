//! DaemonRegistry — local daemon management.
//!
//! Tracks all daemons running on this node, routes events to the
//! correct daemon by origin_hash, and provides snapshot/lifecycle APIs.

use dashmap::DashMap;
use parking_lot::Mutex;

use super::daemon::{DaemonError, DaemonStats};
use super::host::DaemonHost;
use crate::adapter::net::state::causal::CausalEvent;
use crate::adapter::net::state::snapshot::StateSnapshot;

/// Registry of local daemon hosts.
///
/// Each daemon is keyed by `origin_hash` (derived from its EntityKeypair).
/// The `Mutex` per daemon serializes event processing for a single daemon
/// while allowing different daemons to process concurrently.
pub struct DaemonRegistry {
    daemons: DashMap<u32, Mutex<DaemonHost>>,
}

impl DaemonRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            daemons: DashMap::new(),
        }
    }

    /// Register a daemon host.
    ///
    /// Returns an error if a daemon with the same origin_hash is already registered.
    pub fn register(&self, host: DaemonHost) -> Result<(), DaemonError> {
        let origin_hash = host.origin_hash();
        match self.daemons.entry(origin_hash) {
            dashmap::mapref::entry::Entry::Occupied(_) => Err(DaemonError::ProcessFailed(format!(
                "daemon {:#x} already registered",
                origin_hash
            ))),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(Mutex::new(host));
                Ok(())
            }
        }
    }

    /// Unregister a daemon, returning the host.
    pub fn unregister(&self, origin_hash: u32) -> Result<DaemonHost, DaemonError> {
        self.daemons
            .remove(&origin_hash)
            .map(|(_, mutex)| mutex.into_inner())
            .ok_or(DaemonError::NotFound(origin_hash))
    }

    /// Deliver an event to the daemon identified by `origin_hash`.
    pub fn deliver(
        &self,
        origin_hash: u32,
        event: &CausalEvent,
    ) -> Result<Vec<CausalEvent>, DaemonError> {
        let entry = self
            .daemons
            .get(&origin_hash)
            .ok_or(DaemonError::NotFound(origin_hash))?;
        let mut host = entry.lock();
        host.deliver(event)
    }

    /// Take a snapshot of a specific daemon.
    pub fn snapshot(&self, origin_hash: u32) -> Result<Option<StateSnapshot>, DaemonError> {
        let entry = self
            .daemons
            .get(&origin_hash)
            .ok_or(DaemonError::NotFound(origin_hash))?;
        let host = entry.lock();
        Ok(host.take_snapshot())
    }

    /// Get stats for a specific daemon.
    pub fn stats(&self, origin_hash: u32) -> Result<DaemonStats, DaemonError> {
        let entry = self
            .daemons
            .get(&origin_hash)
            .ok_or(DaemonError::NotFound(origin_hash))?;
        let host = entry.lock();
        Ok(host.stats().clone())
    }

    /// List all registered daemon origin hashes and names.
    pub fn list(&self) -> Vec<(u32, String)> {
        self.daemons
            .iter()
            .map(|entry| {
                let host = entry.lock();
                (*entry.key(), host.name().to_string())
            })
            .collect()
    }

    /// Number of registered daemons.
    pub fn count(&self) -> usize {
        self.daemons.len()
    }

    /// Check if a daemon is registered.
    pub fn contains(&self, origin_hash: u32) -> bool {
        self.daemons.contains_key(&origin_hash)
    }

    /// Clone the signing keypair of a locally-registered daemon.
    ///
    /// Used by the migration-source path to seal the daemon's
    /// identity into an [`IdentityEnvelope`] before shipping its
    /// snapshot. Returns `None` when the daemon isn't registered
    /// (already unregistered / never spawned here). The clone is
    /// deliberate — the caller gets its own keypair instance that
    /// outlives the host's internal lock guard.
    pub fn daemon_keypair(
        &self,
        origin_hash: u32,
    ) -> Option<crate::adapter::net::identity::EntityKeypair> {
        let entry = self.daemons.get(&origin_hash)?;
        let host = entry.lock();
        Some(host.keypair().clone())
    }
}

impl Default for DaemonRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DaemonRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DaemonRegistry")
            .field("daemons", &self.daemons.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::net::behavior::capability::CapabilityFilter;
    use crate::adapter::net::compute::daemon::{DaemonHostConfig, MeshDaemon};
    use crate::adapter::net::identity::EntityKeypair;
    use crate::adapter::net::state::causal::CausalLink;
    use bytes::Bytes;

    struct NoopDaemon;

    impl MeshDaemon for NoopDaemon {
        fn name(&self) -> &str {
            "noop"
        }
        fn requirements(&self) -> CapabilityFilter {
            CapabilityFilter::default()
        }
        fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, DaemonError> {
            Ok(vec![])
        }
    }

    fn make_host(name_daemon: impl MeshDaemon + 'static) -> DaemonHost {
        let kp = EntityKeypair::generate();
        DaemonHost::new(Box::new(name_daemon), kp, DaemonHostConfig::default())
    }

    fn make_event() -> CausalEvent {
        CausalEvent {
            link: CausalLink {
                origin_hash: 0xAAAA,
                horizon_encoded: 0,
                sequence: 1,
                parent_hash: 0,
            },
            payload: Bytes::from_static(b"test"),
            received_at: 0,
        }
    }

    #[test]
    fn test_register_and_deliver() {
        let reg = DaemonRegistry::new();
        let host = make_host(NoopDaemon);
        let origin = host.origin_hash();

        reg.register(host).unwrap();
        assert_eq!(reg.count(), 1);
        assert!(reg.contains(origin));

        let outputs = reg.deliver(origin, &make_event()).unwrap();
        assert!(outputs.is_empty()); // noop produces no output
    }

    #[test]
    fn test_deliver_not_found() {
        let reg = DaemonRegistry::new();
        let result = reg.deliver(0xDEAD, &make_event());
        assert_eq!(result.unwrap_err(), DaemonError::NotFound(0xDEAD));
    }

    #[test]
    fn test_unregister() {
        let reg = DaemonRegistry::new();
        let host = make_host(NoopDaemon);
        let origin = host.origin_hash();

        reg.register(host).unwrap();
        assert_eq!(reg.count(), 1);

        let recovered = reg.unregister(origin).unwrap();
        assert_eq!(recovered.origin_hash(), origin);
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_duplicate_register_rejected() {
        let reg = DaemonRegistry::new();
        let kp = EntityKeypair::generate();
        let host1 = DaemonHost::new(
            Box::new(NoopDaemon),
            kp.clone(),
            DaemonHostConfig::default(),
        );
        let host2 = DaemonHost::new(Box::new(NoopDaemon), kp, DaemonHostConfig::default());

        reg.register(host1).unwrap();
        assert!(reg.register(host2).is_err());
    }

    #[test]
    fn test_list() {
        let reg = DaemonRegistry::new();
        reg.register(make_host(NoopDaemon)).unwrap();
        reg.register(make_host(NoopDaemon)).unwrap();

        let list = reg.list();
        assert_eq!(list.len(), 2);
        for (_, name) in &list {
            assert_eq!(name, "noop");
        }
    }

    #[test]
    fn test_snapshot_stateless() {
        let reg = DaemonRegistry::new();
        let host = make_host(NoopDaemon);
        let origin = host.origin_hash();
        reg.register(host).unwrap();

        let snap = reg.snapshot(origin).unwrap();
        assert!(snap.is_none()); // noop is stateless
    }
}
