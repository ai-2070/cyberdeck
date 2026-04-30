//! DaemonRegistry — local daemon management.
//!
//! Tracks all daemons running on this node, routes events to the
//! correct daemon by origin_hash, and provides snapshot/lifecycle APIs.

use std::sync::Arc;

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
///
/// # Shard-lock hygiene
///
/// The inner `Arc<Mutex<DaemonHost>>` matters: DashMap's per-shard
/// read/write locks are held only for the duration of each `get` /
/// `remove` call. All accessors here clone the `Arc` out of the
/// map, drop the `Ref` (releasing the shard read lock), and only
/// *then* take the inner `Mutex`. A closure passed to
/// [`Self::with_host`] therefore cannot re-enter the map on the
/// same shard and deadlock, and arbitrary user work inside the
/// closure can't stall other daemons that happen to hash to the
/// same shard.
pub struct DaemonRegistry {
    daemons: DashMap<u32, Arc<Mutex<DaemonHost>>>,
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
                entry.insert(Arc::new(Mutex::new(host)));
                Ok(())
            }
        }
    }

    /// Atomically replace the daemon at `host.origin_hash()`, or
    /// insert if absent. Used by group lifecycles
    /// (replica/fork/standby) where the new daemon's origin_hash is
    /// deterministic and matches whatever is currently there — the
    /// old `register`-after-`unregister` pattern had a small window
    /// where placement could succeed but the re-register failed
    /// (concurrent registration race) and left the slot orphaned.
    /// `replace` collapses the swap into a single map operation so
    /// the slot is never empty between callers.
    pub fn replace(&self, host: DaemonHost) {
        let origin_hash = host.origin_hash();
        self.daemons.insert(origin_hash, Arc::new(Mutex::new(host)));
    }

    /// Unregister a daemon. Drops the registry's ownership of the
    /// host; any in-flight `Arc` clones (e.g. a
    /// [`Self::with_host`] closure currently running on another
    /// thread) keep the host alive until they release their
    /// reference, then the host is dropped naturally.
    ///
    /// Returns `DaemonError::NotFound` if no daemon with this
    /// `origin_hash` is registered. This method used to return
    /// `Result<DaemonHost, _>` (consuming the host back to the
    /// caller), but no production caller actually needed the
    /// returned host — they all used `let _ =`. Dropping the
    /// return value instead of trying to `Arc::try_unwrap` avoids
    /// a subtle race where the caller could get back a transient
    /// `Err` if any other thread happened to be reading the same
    /// daemon via `with_host` at the moment of unregister.
    pub fn unregister(&self, origin_hash: u32) -> Result<(), DaemonError> {
        self.daemons
            .remove(&origin_hash)
            .map(|_| ())
            .ok_or(DaemonError::NotFound(origin_hash))
    }

    /// Clone the `Arc<Mutex<DaemonHost>>` out of the map, if
    /// present. Holding only the shard read lock long enough to
    /// clone the Arc — never across user work — is the whole
    /// point of wrapping the host in an `Arc`. Internal helper;
    /// the public accessors below build on it.
    fn get_arc(&self, origin_hash: u32) -> Option<Arc<Mutex<DaemonHost>>> {
        self.daemons.get(&origin_hash).map(|e| e.value().clone())
    }

    /// Deliver an event to the daemon identified by `origin_hash`.
    pub fn deliver(
        &self,
        origin_hash: u32,
        event: &CausalEvent,
    ) -> Result<Vec<CausalEvent>, DaemonError> {
        let arc = self
            .get_arc(origin_hash)
            .ok_or(DaemonError::NotFound(origin_hash))?;
        let mut host = arc.lock();
        host.deliver(event)
    }

    /// Take a snapshot of a specific daemon.
    pub fn snapshot(&self, origin_hash: u32) -> Result<Option<StateSnapshot>, DaemonError> {
        let arc = self
            .get_arc(origin_hash)
            .ok_or(DaemonError::NotFound(origin_hash))?;
        let host = arc.lock();
        Ok(host.take_snapshot())
    }

    /// Restore a daemon's state from a snapshot taken on another
    /// daemon (typically the active member of a standby group).
    /// Mutates the existing host in place — keypair and registry
    /// entry stay put; only daemon-state bytes, chain head, and
    /// horizon are replaced.
    ///
    /// Used by `StandbyGroup::sync_standbys` to push the active's
    /// state onto each standby so a promoted standby has the same
    /// state the active had at snapshot time.
    pub fn restore_from_snapshot(
        &self,
        origin_hash: u32,
        snapshot: &StateSnapshot,
    ) -> Result<(), DaemonError> {
        let arc = self
            .get_arc(origin_hash)
            .ok_or(DaemonError::NotFound(origin_hash))?;
        let mut host = arc.lock();
        host.restore_from_snapshot(snapshot)
    }

    /// Get stats for a specific daemon.
    pub fn stats(&self, origin_hash: u32) -> Result<DaemonStats, DaemonError> {
        let arc = self
            .get_arc(origin_hash)
            .ok_or(DaemonError::NotFound(origin_hash))?;
        let host = arc.lock();
        Ok(host.stats().clone())
    }

    /// List all registered daemon origin hashes and names.
    pub fn list(&self) -> Vec<(u32, String)> {
        // Snapshot `(origin_hash, Arc<Mutex<Host>>)` under each
        // shard read lock, then drop the Ref before locking the
        // inner Mutex for `name()`. Keeps shard lock hold time
        // bounded to a Vec push per entry.
        let arcs: Vec<(u32, Arc<Mutex<DaemonHost>>)> = self
            .daemons
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();
        arcs.into_iter()
            .map(|(origin, arc)| {
                let host = arc.lock();
                (origin, host.name().to_string())
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

    /// Invoke `f` with a reference to the host identified by
    /// `origin_hash`, holding the per-daemon `Mutex` (but **not**
    /// the DashMap shard lock) for the duration. Returns the
    /// closure's result, or `DaemonError::NotFound` if no daemon
    /// matches.
    ///
    /// Keeps the lock scoped so callers don't accidentally leak
    /// `MutexGuard` out; used by the SDK's subscribe / unsubscribe
    /// path to update the subscription ledger after a successful
    /// mesh call.
    ///
    /// # Re-entrancy
    ///
    /// The closure is free to call back into `self` (including
    /// `register` / `unregister` / `with_host` for any origin,
    /// same shard or not) without deadlocking. The only thing it
    /// cannot do is re-enter `with_host` for the **same**
    /// `origin_hash` from the same thread — that would try to
    /// re-lock the per-daemon `Mutex`. `parking_lot::Mutex` is
    /// not re-entrant; keeping the outer shard lock out of the
    /// way is what fixes the broader deadlock class.
    pub fn with_host<F, R>(&self, origin_hash: u32, f: F) -> Result<R, DaemonError>
    where
        F: FnOnce(&DaemonHost) -> R,
    {
        let arc = self
            .get_arc(origin_hash)
            .ok_or(DaemonError::NotFound(origin_hash))?;
        let host = arc.lock();
        Ok(f(&host))
    }

    /// Clone the signing keypair of a locally-registered daemon.
    ///
    /// Used by the migration-source path to seal the daemon's
    /// identity into an
    /// [`IdentityEnvelope`](crate::adapter::net::identity::IdentityEnvelope)
    /// before shipping its snapshot. Returns `None` when the daemon
    /// isn't registered
    /// (already unregistered / never spawned here). The clone is
    /// deliberate — the caller gets its own keypair instance that
    /// outlives the host's internal lock guard.
    pub fn daemon_keypair(
        &self,
        origin_hash: u32,
    ) -> Option<crate::adapter::net::identity::EntityKeypair> {
        let arc = self.get_arc(origin_hash)?;
        let host = arc.lock();
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

        reg.unregister(origin).unwrap();
        assert_eq!(reg.count(), 0);
        assert!(!reg.contains(origin));
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

    /// Regression (Cubic-AI P2): `with_host` used to hold the
    /// DashMap shard read lock across the entire closure — any
    /// re-entrant mutation of the same shard (e.g. `register` /
    /// `unregister` for an origin that hashes to the same shard)
    /// would deadlock, since DashMap's per-shard `RwLock` can't
    /// upgrade from read to write.
    ///
    /// This test spawns `with_host` onto a background thread and
    /// calls `unregister(same_origin)` from inside the closure —
    /// which *always* writes to the same shard. Pre-fix, the
    /// closure hangs; post-fix, it returns promptly because the
    /// shard lock was released before the closure ran.
    ///
    /// A watchdog thread kicks in after 1 s and panics if the
    /// worker hasn't finished — otherwise a regressed fix would
    /// hang the test runner indefinitely.
    #[test]
    fn with_host_closure_can_mutate_same_shard_without_deadlock() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc as StdArc;
        use std::thread;
        use std::time::Duration;

        let reg = StdArc::new(DaemonRegistry::new());
        let host = make_host(NoopDaemon);
        let origin = host.origin_hash();
        reg.register(host).unwrap();

        let done = StdArc::new(AtomicBool::new(false));
        let reg_worker = reg.clone();
        let done_worker = done.clone();
        let worker = thread::spawn(move || {
            let reg_inner = reg_worker.clone();
            reg_worker
                .with_host(origin, move |_host| {
                    // Reach back into the same DashMap shard. Pre-fix
                    // this acquires a write lock while the read lock
                    // is held by the enclosing `with_host` → deadlock.
                    let _ = reg_inner.unregister(origin);
                })
                .expect("with_host should not error");
            done_worker.store(true, Ordering::Release);
        });

        // Watchdog: give the worker 1 second (generous; real work
        // here is microseconds). A pre-fix build hangs forever
        // without this, which wrecks CI.
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while std::time::Instant::now() < deadline {
            if done.load(Ordering::Acquire) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            done.load(Ordering::Acquire),
            "with_host closure deadlocked when calling back into the registry — \
             shard lock is being held across the closure body (Cubic-AI P2)",
        );
        worker.join().expect("worker panicked");

        // Post-fix, the unregister inside the closure succeeded;
        // the registry is now empty.
        assert!(!reg.contains(origin));
        assert_eq!(reg.count(), 0);
    }

    /// Secondary check: `with_host` must not block *other*
    /// daemons' access while its closure is running. With
    /// `Arc<Mutex<Host>>` plus shard-lock release, a slow closure
    /// on daemon A cannot stall daemon B even when A and B hash
    /// to the same DashMap shard.
    #[test]
    fn with_host_does_not_block_other_daemons() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc as StdArc;
        use std::thread;
        use std::time::Duration;

        let reg = StdArc::new(DaemonRegistry::new());
        let host_a = make_host(NoopDaemon);
        let origin_a = host_a.origin_hash();
        let host_b = make_host(NoopDaemon);
        let origin_b = host_b.origin_hash();
        reg.register(host_a).unwrap();
        reg.register(host_b).unwrap();

        // Thread 1: hold a long `with_host` on A.
        let release = StdArc::new(AtomicBool::new(false));
        let reg_a = reg.clone();
        let release_a = release.clone();
        let long_reader = thread::spawn(move || {
            reg_a
                .with_host(origin_a, |_host| {
                    // Spin until released. Pre-fix, this would
                    // hold the DashMap shard lock; post-fix it
                    // only holds daemon A's Mutex.
                    while !release_a.load(Ordering::Acquire) {
                        std::hint::spin_loop();
                    }
                })
                .unwrap();
        });

        // Thread 2: hit daemon B. Must complete without waiting
        // for thread 1 to release, even if A and B share a shard.
        thread::sleep(Duration::from_millis(20)); // let thread 1 start
        let reg_b = reg.clone();
        let b_done = StdArc::new(AtomicBool::new(false));
        let b_done_worker = b_done.clone();
        let short_reader = thread::spawn(move || {
            reg_b.with_host(origin_b, |_host| {}).unwrap();
            b_done_worker.store(true, Ordering::Release);
        });

        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while std::time::Instant::now() < deadline {
            if b_done.load(Ordering::Acquire) {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            b_done.load(Ordering::Acquire),
            "with_host on daemon B stalled behind a long-running closure on \
             daemon A — the shard lock is still being held across closure \
             execution",
        );

        // Let thread 1 finish and clean up.
        release.store(true, Ordering::Release);
        long_reader.join().unwrap();
        short_reader.join().unwrap();
    }
}
