//! Compute surface — `MeshDaemon` + `DaemonRuntime`.
//!
//! Users implement [`MeshDaemon`] and hand it to a [`DaemonRuntime`]
//! tied to a [`Mesh`](crate::Mesh) node. The runtime holds the
//! kind-keyed factory table, the per-daemon host registry, and the
//! lifecycle gate that decides when inbound migrations may land.
//!
//! This file is Stage 1 of
//! [`SDK_COMPUTE_SURFACE_PLAN.md`](../../../docs/SDK_COMPUTE_SURFACE_PLAN.md)
//! plus the lifecycle half of
//! [`DAEMON_RUNTIME_READINESS_PLAN.md`](../../../docs/DAEMON_RUNTIME_READINESS_PLAN.md):
//! local spawn / snapshot / stop, with an explicit
//! `Registering → Ready → ShuttingDown` fence. Migration is Stage 2;
//! the wire-level half of the readiness plan
//! (`MigrationFailureReason`) ships alongside it.
//!
//! # Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use bytes::Bytes;
//! use net_sdk::{Identity, Mesh};
//! use net_sdk::compute::{
//!     CausalEvent, DaemonHostConfig, DaemonRuntime, MeshDaemon,
//! };
//! use net_sdk::capabilities::CapabilityFilter;
//! use net::adapter::net::compute::DaemonError as CoreDaemonError;
//!
//! struct EchoDaemon;
//! impl MeshDaemon for EchoDaemon {
//!     fn name(&self) -> &str { "echo" }
//!     fn requirements(&self) -> CapabilityFilter { CapabilityFilter::default() }
//!     fn process(&mut self, event: &CausalEvent) -> Result<Vec<Bytes>, CoreDaemonError> {
//!         Ok(vec![event.payload.clone()])
//!     }
//! }
//!
//! # async fn example(mesh: Arc<Mesh>) -> Result<(), Box<dyn std::error::Error>> {
//! let rt = DaemonRuntime::new(mesh);
//! rt.register_factory("echo", || Box::new(EchoDaemon))?;
//! rt.start().await?;
//! let handle = rt.spawn("echo", Identity::generate(), DaemonHostConfig::default()).await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use thiserror::Error;

pub use ::net::adapter::net::compute::{
    DaemonError as CoreDaemonError, DaemonHostConfig, DaemonStats, MeshDaemon,
    PlacementDecision, SchedulerError,
};
pub use ::net::adapter::net::state::causal::{CausalEvent, CausalLink};
pub use ::net::adapter::net::state::snapshot::StateSnapshot;

use ::net::adapter::net::compute::{DaemonFactoryRegistry, DaemonHost, DaemonRegistry};
use ::net::adapter::net::identity::EntityId;

use crate::identity::Identity;
use crate::mesh::Mesh;

/// Arc-wrapped factory closure. Kind-keyed at the SDK layer; cloned
/// into the core `DaemonFactoryRegistry` at `spawn` time so a future
/// migration target can reconstruct the daemon by `origin_hash`.
type FactoryFn = Arc<dyn Fn() -> Box<dyn MeshDaemon> + Send + Sync>;

/// Errors from the SDK daemon runtime.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// `start()` has not been called yet; the runtime is still in
    /// `Registering` and will not accept spawns or migrations.
    #[error("daemon runtime is not ready — call DaemonRuntime::start() first")]
    NotReady,
    /// `shutdown()` has been called; the runtime is permanently
    /// non-functional.
    #[error("daemon runtime has been shut down")]
    ShuttingDown,
    /// Two `register_factory` calls used the same `kind` string.
    #[error("factory for kind '{0}' is already registered")]
    FactoryAlreadyRegistered(String),
    /// `spawn` / `spawn_from_snapshot` referenced an unregistered kind.
    #[error("no factory registered for kind '{0}'")]
    FactoryNotFound(String),
    /// The snapshot's `entity_id.origin_hash` does not match the
    /// identity handed to `spawn_from_snapshot`.
    #[error(
        "snapshot/identity mismatch: snapshot origin {snapshot:#x} != identity origin {identity:#x}"
    )]
    SnapshotIdentityMismatch { snapshot: u32, identity: u32 },
    /// Pass-through for errors surfaced by the core compute layer.
    #[error(transparent)]
    Core(#[from] CoreDaemonError),
}

// Runtime state machine. Encoded as `u8` so it rides in an
// `AtomicU8` without an extra layer of indirection. Values are
// stable across the lifetime of a `DaemonRuntime` and must not be
// reordered (release / acquire cmpxchg compares by value).
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State {
    Registering = 0,
    Ready = 1,
    ShuttingDown = 2,
}

impl State {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => State::Registering,
            1 => State::Ready,
            2 => State::ShuttingDown,
            // `AtomicU8` is only written through `State::*` variants,
            // so any other value means memory corruption. Panic
            // loudly rather than silently misinterpret.
            other => panic!("daemon runtime: corrupt state byte {other}"),
        }
    }
}

/// Per-mesh compute runtime.
///
/// Holds the kind-keyed factory table, the per-daemon host registry,
/// and the `Registering → Ready → ShuttingDown` lifecycle gate. One
/// `DaemonRuntime` per [`Mesh`]; clone the handle freely — the inner
/// state is `Arc`-shared.
#[derive(Clone)]
pub struct DaemonRuntime {
    inner: Arc<Inner>,
}

struct Inner {
    mesh: Arc<Mesh>,
    state: AtomicU8,
    /// SDK-side kind → factory map. Stage 2 readers (migration
    /// target) reach into the core `factory_registry` below by
    /// `origin_hash`; `kind` is SDK sugar so the *caller* can spawn
    /// without knowing the underlying keypair.
    factories: RwLock<HashMap<String, FactoryFn>>,
    /// Core registry — lives inside `MeshNode` machinery in Stage 2
    /// once the migration subprotocol is wired; today it sits behind
    /// the SDK surface only.
    registry: Arc<DaemonRegistry>,
    /// Core factory registry, keyed by `origin_hash`. `spawn` mirrors
    /// each SDK-side kind registration into this map with the
    /// concrete keypair attached, so Stage 2 migrations restore
    /// through the existing `DaemonFactoryRegistry::construct` path
    /// without a second lookup layer.
    factory_registry: Arc<DaemonFactoryRegistry>,
}

impl DaemonRuntime {
    /// Attach a runtime to an existing [`Mesh`]. Stage 1 does not
    /// consume the `Mesh` — users keep their `Arc<Mesh>` for channel
    /// registration, subscription, and the rest of the non-compute
    /// surface. Stage 2 will install the migration subprotocol
    /// handler when [`Self::start`] runs; until then inbound
    /// migration messages (if any) are silently dropped by the core,
    /// same as today.
    pub fn new(mesh: Arc<Mesh>) -> Self {
        Self {
            inner: Arc::new(Inner {
                mesh,
                state: AtomicU8::new(State::Registering as u8),
                factories: RwLock::new(HashMap::new()),
                registry: Arc::new(DaemonRegistry::new()),
                factory_registry: Arc::new(DaemonFactoryRegistry::new()),
            }),
        }
    }

    /// Register a factory for a daemon type. `kind` is a user-chosen
    /// string shared across every node that may host this daemon —
    /// Stage 2 migrations will look up the same `kind` on the
    /// target. Second registrations of the same `kind` return
    /// [`DaemonError::FactoryAlreadyRegistered`].
    ///
    /// Valid in both `Registering` and `Ready` states; the runtime
    /// permits new kinds to appear at runtime. Only `ShuttingDown`
    /// rejects.
    pub fn register_factory<F>(&self, kind: &str, factory: F) -> Result<(), DaemonError>
    where
        F: Fn() -> Box<dyn MeshDaemon> + Send + Sync + 'static,
    {
        if self.state() == State::ShuttingDown {
            return Err(DaemonError::ShuttingDown);
        }
        let mut map = self.inner.factories.write().expect("factory map poisoned");
        if map.contains_key(kind) {
            return Err(DaemonError::FactoryAlreadyRegistered(kind.to_string()));
        }
        map.insert(kind.to_string(), Arc::new(factory));
        Ok(())
    }

    /// Promote to `Ready`. Idempotent — a second call on an already-
    /// `Ready` runtime is a no-op; a call on a `ShuttingDown` runtime
    /// returns [`DaemonError::ShuttingDown`]. Stage 2 will also wire
    /// the migration subprotocol handler here.
    pub async fn start(&self) -> Result<(), DaemonError> {
        loop {
            match self.state() {
                State::Registering => {
                    let swap = self.inner.state.compare_exchange(
                        State::Registering as u8,
                        State::Ready as u8,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    );
                    if swap.is_ok() {
                        return Ok(());
                    }
                    // Lost the CAS — another caller flipped the state.
                    // Loop and re-classify.
                }
                State::Ready => return Ok(()),
                State::ShuttingDown => return Err(DaemonError::ShuttingDown),
            }
        }
    }

    /// Tear down the runtime. Unregisters every local daemon host,
    /// clears the factory registry, and transitions state to
    /// `ShuttingDown`. Subsequent calls on this runtime fail with
    /// [`DaemonError::ShuttingDown`]. A second `shutdown` is a no-op.
    pub async fn shutdown(&self) -> Result<(), DaemonError> {
        // Mark ShuttingDown first so new spawns / registrations
        // immediately short-circuit. The store isn't a CAS — a
        // `ShuttingDown → ShuttingDown` re-store is cheap and
        // benign.
        self.inner
            .state
            .store(State::ShuttingDown as u8, Ordering::Release);

        // Drain the registry. `list()` snapshots origin_hashes under
        // its internal read guard; iterating is safe because we own
        // the unregister path.
        let origins: Vec<u32> = self
            .inner
            .registry
            .list()
            .into_iter()
            .map(|(origin, _)| origin)
            .collect();
        for origin in origins {
            let _ = self.inner.registry.unregister(origin);
            self.inner.factory_registry.remove(origin);
        }
        Ok(())
    }

    /// Readiness accessor for tests + operators. `true` iff the
    /// runtime has transitioned to `Ready` and has not yet begun
    /// shutting down.
    pub fn is_ready(&self) -> bool {
        self.state() == State::Ready
    }

    /// Spawn a daemon of `kind` under the caller-provided
    /// [`Identity`]. The identity's keypair seeds the daemon's
    /// `origin_hash` + `entity_id`; the runtime registers both the
    /// live host and a kind-keyed factory in the core registry so a
    /// future migration target can reconstruct the daemon through
    /// the existing `DaemonFactoryRegistry::construct` path.
    ///
    /// The returned [`DaemonHandle`] is clone-safe; dropping it does
    /// not stop the daemon. Call [`Self::stop`] explicitly.
    pub async fn spawn(
        &self,
        kind: &str,
        identity: Identity,
        config: DaemonHostConfig,
    ) -> Result<DaemonHandle, DaemonError> {
        self.require_ready()?;
        let factory = self.factory_for_kind(kind)?;
        let daemon = (factory)();
        let keypair = identity.keypair().as_ref().clone();
        let origin_hash = keypair.origin_hash();
        let entity_id = keypair.entity_id().clone();

        // Mirror the factory into the core registry BEFORE registering
        // the host, so a migration-target handler that catches up on
        // this origin mid-spawn always sees a consistent view.
        let factory_for_core = factory.clone();
        self.inner.factory_registry.register(
            keypair.clone(),
            config.clone(),
            move || (factory_for_core)(),
        );

        let host = DaemonHost::new(daemon, keypair, config);
        // `DaemonRegistry::register` errors on origin_hash collisions
        // — two daemons can't share the same identity. Roll back the
        // factory_registry insert so a retry with a fresh identity
        // doesn't pick up a stale factory.
        if let Err(e) = self.inner.registry.register(host) {
            self.inner.factory_registry.remove(origin_hash);
            return Err(DaemonError::Core(e));
        }

        Ok(DaemonHandle {
            origin_hash,
            entity_id,
            inner: self.inner.clone(),
        })
    }

    /// Spawn a daemon of `kind` and restore its state from `snapshot`.
    /// The snapshot's `entity_id` must match the caller's
    /// [`Identity`]; mismatch returns
    /// [`DaemonError::SnapshotIdentityMismatch`] before any side
    /// effects land.
    pub async fn spawn_from_snapshot(
        &self,
        kind: &str,
        identity: Identity,
        snapshot: StateSnapshot,
        config: DaemonHostConfig,
    ) -> Result<DaemonHandle, DaemonError> {
        self.require_ready()?;
        let factory = self.factory_for_kind(kind)?;
        let keypair = identity.keypair().as_ref().clone();
        let origin_hash = keypair.origin_hash();
        let entity_id = keypair.entity_id().clone();

        if snapshot.entity_id.origin_hash() != origin_hash {
            return Err(DaemonError::SnapshotIdentityMismatch {
                snapshot: snapshot.entity_id.origin_hash(),
                identity: origin_hash,
            });
        }

        let daemon = (factory)();
        let factory_for_core = factory.clone();
        self.inner.factory_registry.register(
            keypair.clone(),
            config.clone(),
            move || (factory_for_core)(),
        );

        let host = match DaemonHost::from_snapshot(daemon, keypair, &snapshot, config) {
            Ok(h) => h,
            Err(e) => {
                self.inner.factory_registry.remove(origin_hash);
                return Err(DaemonError::Core(e));
            }
        };

        if let Err(e) = self.inner.registry.register(host) {
            self.inner.factory_registry.remove(origin_hash);
            return Err(DaemonError::Core(e));
        }

        Ok(DaemonHandle {
            origin_hash,
            entity_id,
            inner: self.inner.clone(),
        })
    }

    /// Stop a daemon, removing it from the runtime's registry. Valid
    /// while `Ready` and (idempotently) during `ShuttingDown` —
    /// `ShuttingDown` paths through here are a no-op because the
    /// shutdown sweep has already drained the registry.
    pub async fn stop(&self, origin_hash: u32) -> Result<(), DaemonError> {
        if self.state() == State::Registering {
            return Err(DaemonError::NotReady);
        }
        // Treat a missing daemon as success during ShuttingDown —
        // the shutdown sweep drained it.
        match self.inner.registry.unregister(origin_hash) {
            Ok(_) => {
                self.inner.factory_registry.remove(origin_hash);
                Ok(())
            }
            Err(CoreDaemonError::NotFound(_)) if self.state() == State::ShuttingDown => Ok(()),
            Err(e) => Err(DaemonError::Core(e)),
        }
    }

    /// Take a snapshot of a running daemon by `origin_hash`. Returns
    /// `Ok(None)` when the daemon is stateless.
    pub async fn snapshot(&self, origin_hash: u32) -> Result<Option<StateSnapshot>, DaemonError> {
        self.require_ready()?;
        self.inner
            .registry
            .snapshot(origin_hash)
            .map_err(DaemonError::Core)
    }

    /// Deliver one causal event to the daemon identified by
    /// `origin_hash`, returning the daemon's outputs wrapped in the
    /// host's causal chain.
    ///
    /// Stage 1 convenience — Stage 2 adds mesh-dispatched delivery
    /// via the causal subprotocol, and this direct path becomes
    /// testing sugar rather than the primary ingress.
    pub fn deliver(
        &self,
        origin_hash: u32,
        event: &CausalEvent,
    ) -> Result<Vec<CausalEvent>, DaemonError> {
        self.require_ready()?;
        self.inner
            .registry
            .deliver(origin_hash, event)
            .map_err(DaemonError::Core)
    }

    /// Number of daemons currently registered.
    pub fn daemon_count(&self) -> usize {
        self.inner.registry.count()
    }

    /// Underlying mesh. Exposed read-only so the caller can still
    /// reach the channel / subscribe / publish surface without
    /// reaching around the runtime.
    pub fn mesh(&self) -> &Arc<Mesh> {
        &self.inner.mesh
    }

    // ------------ internal helpers ------------

    fn state(&self) -> State {
        State::from_u8(self.inner.state.load(Ordering::Acquire))
    }

    fn require_ready(&self) -> Result<(), DaemonError> {
        match self.state() {
            State::Ready => Ok(()),
            State::Registering => Err(DaemonError::NotReady),
            State::ShuttingDown => Err(DaemonError::ShuttingDown),
        }
    }

    fn factory_for_kind(&self, kind: &str) -> Result<FactoryFn, DaemonError> {
        self.inner
            .factories
            .read()
            .expect("factory map poisoned")
            .get(kind)
            .cloned()
            .ok_or_else(|| DaemonError::FactoryNotFound(kind.to_string()))
    }
}

/// Handle to a running daemon. Clone-safe; dropping does not stop
/// the daemon — call [`DaemonRuntime::stop`] explicitly.
#[derive(Clone)]
pub struct DaemonHandle {
    /// Daemon's 32-bit origin hash. Stable for the daemon's lifetime
    /// and across migrations.
    pub origin_hash: u32,
    /// Daemon's full 32-byte entity id.
    pub entity_id: EntityId,
    inner: Arc<Inner>,
}

impl DaemonHandle {
    /// Read the daemon's current stats.
    pub fn stats(&self) -> Result<DaemonStats, DaemonError> {
        self.inner
            .registry
            .stats(self.origin_hash)
            .map_err(DaemonError::Core)
    }

    /// Take a snapshot of the daemon's current state. `Ok(None)` for
    /// stateless daemons.
    pub async fn snapshot(&self) -> Result<Option<StateSnapshot>, DaemonError> {
        self.inner
            .registry
            .snapshot(self.origin_hash)
            .map_err(DaemonError::Core)
    }
}

impl std::fmt::Debug for DaemonRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let factory_count = self
            .inner
            .factories
            .read()
            .map(|m| m.len())
            .unwrap_or_default();
        f.debug_struct("DaemonRuntime")
            .field("state", &self.state())
            .field("factories", &factory_count)
            .field("daemons", &self.inner.registry.count())
            .finish()
    }
}

impl std::fmt::Debug for DaemonHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DaemonHandle")
            .field("origin_hash", &format_args!("{:#x}", self.origin_hash))
            .field("entity_id", &self.entity_id)
            .finish()
    }
}
