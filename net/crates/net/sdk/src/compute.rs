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
    DaemonError as CoreDaemonError, DaemonHostConfig, DaemonStats, MeshDaemon, MigrationError,
    MigrationPhase, PlacementDecision, SchedulerError, SUBPROTOCOL_MIGRATION,
};
pub use ::net::adapter::net::state::causal::{CausalEvent, CausalLink};
pub use ::net::adapter::net::state::snapshot::StateSnapshot;

use ::net::adapter::net::compute::{
    orchestrator::wire as migration_wire, DaemonFactoryRegistry, DaemonHost, DaemonRegistry,
    MigrationMessage, MigrationOrchestrator, MigrationSourceHandler, MigrationTargetHandler,
};
use ::net::adapter::net::identity::EntityId;
use ::net::adapter::net::subprotocol::{
    MigrationIdentityContext, MigrationSubprotocolHandler,
};

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
    /// Pass-through for migration-layer errors.
    #[error("migration failed: {0}")]
    Migration(#[from] MigrationError),
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
    /// SDK-side kind → factory map. The migration target path reaches
    /// into the core `factory_registry` below by `origin_hash`;
    /// `kind` is SDK sugar so the *caller* can spawn without knowing
    /// the underlying keypair.
    factories: RwLock<HashMap<String, FactoryFn>>,
    /// Core registry — shared with the migration target handler so
    /// daemons restored from an inbound snapshot land in the same
    /// map a local `spawn` uses.
    registry: Arc<DaemonRegistry>,
    /// Core factory registry, keyed by `origin_hash`. `spawn` mirrors
    /// each SDK-side kind registration into this map with the
    /// concrete keypair attached so the migration target restores
    /// through the existing `DaemonFactoryRegistry::construct` path.
    factory_registry: Arc<DaemonFactoryRegistry>,
    /// Migration orchestrator, owned by this node. Orchestrates the
    /// 6-phase state machine when this node initiates a migration.
    orchestrator: Arc<MigrationOrchestrator>,
    /// Migration source handler — drives the source side when THIS
    /// node is the source of a migration.
    source_handler: Arc<MigrationSourceHandler>,
    /// Migration target handler — drives the target side when THIS
    /// node is the target of a migration.
    target_handler: Arc<MigrationTargetHandler>,
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
        let local_node_id = mesh.inner().node_id();
        let registry = Arc::new(DaemonRegistry::new());
        let factory_registry = Arc::new(DaemonFactoryRegistry::new());
        let orchestrator = Arc::new(MigrationOrchestrator::new(registry.clone(), local_node_id));
        let source_handler = Arc::new(MigrationSourceHandler::new(registry.clone()));
        let target_handler = Arc::new(MigrationTargetHandler::new_with_factories(
            registry.clone(),
            factory_registry.clone(),
        ));
        Self {
            inner: Arc::new(Inner {
                mesh,
                state: AtomicU8::new(State::Registering as u8),
                factories: RwLock::new(HashMap::new()),
                registry,
                factory_registry,
                orchestrator,
                source_handler,
                target_handler,
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
    /// returns [`DaemonError::ShuttingDown`].
    ///
    /// Wires the migration subprotocol (`0x0500`) handler into the
    /// mesh so inbound `TakeSnapshot` / `SnapshotReady` / etc.
    /// messages reach the orchestrator / source / target handlers
    /// owned by this runtime. Installing is idempotent w.r.t.
    /// multiple `start` calls — the `ArcSwapOption` on the mesh
    /// swaps the same handler in on each call.
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
                        // Install the migration subprotocol handler —
                        // this is the runtime's one-time side effect
                        // on the underlying `Mesh`. Constructed with
                        // an `IdentityContext` derived from the
                        // mesh's own Noise static key + the
                        // `peer_static_x25519` accessor so the
                        // dispatcher can auto-seal envelopes on the
                        // source path and auto-open them on the
                        // target path.
                        let local_node_id = self.inner.mesh.inner().node_id();
                        let mesh_for_lookup = self.inner.mesh.clone();
                        let ctx = MigrationIdentityContext {
                            local_x25519_priv: self.inner.mesh.inner().static_x25519_priv(),
                            peer_static_lookup: Arc::new(move |node_id: u64| {
                                mesh_for_lookup.inner().peer_static_x25519(node_id)
                            }),
                        };
                        let handler = Arc::new(MigrationSubprotocolHandler::new_with_identity(
                            self.inner.orchestrator.clone(),
                            self.inner.source_handler.clone(),
                            self.inner.target_handler.clone(),
                            local_node_id,
                            Some(ctx),
                        ));
                        self.inner.mesh.inner().set_migration_handler(handler);
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

    /// Pre-register a factory on the target node keyed by the
    /// daemon's `origin_hash`, using the caller-supplied `Identity`
    /// as the keypair. Required today on any node that may receive
    /// an inbound migration for a specific daemon — the target's
    /// `DaemonFactoryRegistry` needs both the constructor closure
    /// *and* the matching keypair to reconstruct the daemon.
    ///
    /// **Stopgap.** `DAEMON_IDENTITY_MIGRATION_PLAN.md` Stages 5b /
    /// 6 replace this with an `IdentityEnvelope` that carries the
    /// keypair across the wire, sealed to the target's X25519
    /// pubkey. Once those land, the target no longer needs to know
    /// the daemon's keypair out of band and this method becomes
    /// unnecessary — keep it as a low-level escape hatch or remove
    /// it entirely.
    ///
    /// `kind` still matters: it's looked up in the SDK-side
    /// `factories` map so later `spawn_from_snapshot` calls on the
    /// same node can find the same constructor. The `Identity`
    /// keypair must match the one the *source* used to spawn the
    /// daemon, otherwise `origin_hash` mismatches at restore and
    /// the migration fails.
    pub fn register_migration_target_identity(
        &self,
        kind: &str,
        identity: Identity,
        config: DaemonHostConfig,
    ) -> Result<(), DaemonError> {
        if self.state() == State::ShuttingDown {
            return Err(DaemonError::ShuttingDown);
        }
        let factory = self.factory_for_kind(kind)?;
        let keypair = identity.keypair().as_ref().clone();
        let factory_clone = factory.clone();
        self.inner.factory_registry.register(
            keypair,
            config,
            move || (factory_clone)(),
        );
        Ok(())
    }

    /// Start migrating a daemon from `source_node` to `target_node`.
    /// The orchestrator runs on this node regardless of who owns
    /// the daemon — call this on whichever node wants to drive the
    /// migration state machine.
    ///
    /// Returns a [`MigrationHandle`] whose [`MigrationHandle::wait`]
    /// resolves when the migration reaches a terminal state
    /// (`Complete` on success, `MigrationError` on abort / failure).
    ///
    /// For the common local-source case (`source_node ==
    /// mesh.node_id()`), the snapshot is taken synchronously inside
    /// this call and `SnapshotReady` is shipped to the target. For
    /// a remote source, the orchestrator sends `TakeSnapshot` to
    /// the source and drives the rest of the state machine from
    /// inbound wire messages.
    pub async fn start_migration(
        &self,
        origin_hash: u32,
        source_node: u64,
        target_node: u64,
    ) -> Result<MigrationHandle, DaemonError> {
        self.start_migration_with(origin_hash, source_node, target_node, MigrationOpts::default())
            .await
    }

    /// `start_migration` with caller-supplied options. Stage 6 of
    /// [`DAEMON_IDENTITY_MIGRATION_PLAN.md`](../../../docs/DAEMON_IDENTITY_MIGRATION_PLAN.md):
    /// lets the caller opt out of identity transport when the daemon
    /// doesn't need to sign anything on the target.
    pub async fn start_migration_with(
        &self,
        origin_hash: u32,
        source_node: u64,
        target_node: u64,
        opts: MigrationOpts,
    ) -> Result<MigrationHandle, DaemonError> {
        self.require_ready()?;
        let mut first_msg = self
            .inner
            .orchestrator
            .start_migration(origin_hash, source_node, target_node)
            .map_err(DaemonError::Migration)?;

        // Local-source path: `start_migration` builds `SnapshotReady`
        // synchronously from the local registry. If the caller asked
        // for identity transport, seal the envelope HERE — the
        // dispatcher's source-side seal only fires on the TakeSnapshot
        // path (remote source). Decoding + re-encoding isolates the
        // payload cleanly.
        if opts.transport_identity {
            if let MigrationMessage::SnapshotReady {
                ref mut snapshot_bytes,
                ..
            } = first_msg
            {
                if let Some(sealed) =
                    self.maybe_seal_local_snapshot(origin_hash, target_node, snapshot_bytes)
                {
                    *snapshot_bytes = sealed;
                }
            }
        }

        // Send the first message to the appropriate node. `start_migration`
        // returns `TakeSnapshot` when source is remote, `SnapshotReady`
        // when source is local.
        let (dest_node, payload) = match &first_msg {
            MigrationMessage::TakeSnapshot { target_node: _, .. } => (source_node, &first_msg),
            MigrationMessage::SnapshotReady { .. } => (target_node, &first_msg),
            other => {
                // `start_migration` on the core orchestrator is
                // documented to return one of these two variants;
                // anything else is a protocol bug. Abort + surface.
                let _ = self
                    .inner
                    .orchestrator
                    .abort_migration(origin_hash, "unexpected initial message".into());
                return Err(DaemonError::Migration(MigrationError::StateFailed(format!(
                    "orchestrator returned unexpected initial migration message: {:?}",
                    other
                ))));
            }
        };

        if let Err(e) = self.send_migration_message(dest_node, payload).await {
            let _ = self
                .inner
                .orchestrator
                .abort_migration(origin_hash, format!("initial send failed: {e}"));
            return Err(e);
        }

        Ok(MigrationHandle {
            origin_hash,
            source_node,
            target_node,
            runtime: self.clone(),
        })
    }

    /// Decode `snapshot_bytes`, attempt to seal an identity envelope
    /// using the local daemon's keypair + target's X25519 static
    /// pubkey, and re-encode. Returns `Some(new_bytes)` on success,
    /// `None` if any prerequisite is missing — the caller treats
    /// `None` as "proceed without envelope," matching the remote-
    /// source dispatcher's public-identity fallback.
    fn maybe_seal_local_snapshot(
        &self,
        daemon_origin: u32,
        target_node: u64,
        snapshot_bytes: &[u8],
    ) -> Option<Vec<u8>> {
        let snapshot = StateSnapshot::from_bytes(snapshot_bytes)?;
        if snapshot.identity_envelope.is_some() {
            return None;
        }
        let target_pub = self.inner.mesh.inner().peer_static_x25519(target_node)?;
        let kp = self.inner.registry.daemon_keypair(daemon_origin)?;
        match snapshot.with_identity_envelope(&kp, target_pub) {
            Ok(sealed) => Some(sealed.to_bytes()),
            Err(_) => None,
        }
    }

    async fn send_migration_message(
        &self,
        dest_node: u64,
        msg: &MigrationMessage,
    ) -> Result<(), DaemonError> {
        let addr = self
            .inner
            .mesh
            .inner()
            .peer_addr(dest_node)
            .ok_or_else(|| {
                DaemonError::Migration(MigrationError::TargetUnavailable(dest_node))
            })?;
        let bytes = migration_wire::encode(msg).map_err(DaemonError::Migration)?;
        self.inner
            .mesh
            .inner()
            .send_subprotocol(addr, SUBPROTOCOL_MIGRATION, &bytes)
            .await
            .map_err(|e| {
                DaemonError::Migration(MigrationError::StateFailed(format!(
                    "send_subprotocol failed: {e}"
                )))
            })
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

/// Options for [`DaemonRuntime::start_migration_with`].
///
/// Stage 6 of
/// [`DAEMON_IDENTITY_MIGRATION_PLAN.md`](../../../docs/DAEMON_IDENTITY_MIGRATION_PLAN.md) —
/// lets the caller opt out of identity transport for workloads that
/// don't need post-migration signing capability (pure compute
/// daemons). Default: identity IS transported.
#[derive(Debug, Clone)]
pub struct MigrationOpts {
    /// If `true` (default), the source node seals its daemon's
    /// ed25519 seed into the outbound snapshot using the target's
    /// X25519 static pubkey. The target unseals on arrival and the
    /// migrated daemon keeps its full signing capability.
    ///
    /// If `false`, the envelope is omitted and the target
    /// reconstructs the daemon with a `public_only` keypair —
    /// identity queries (`entity_id`, `origin_hash`) work, but
    /// `sign` calls fail with `EntityError::ReadOnly`. Appropriate
    /// for pure compute daemons that only consume events and emit
    /// payloads, and do NOT need to mint capability announcements
    /// or issue permission tokens from the target.
    pub transport_identity: bool,
}

impl Default for MigrationOpts {
    fn default() -> Self {
        Self {
            transport_identity: true,
        }
    }
}

/// Handle to an in-flight migration. Drop the handle and the
/// orchestrator continues driving the migration to completion in the
/// background; keep it to observe phase transitions or request abort.
///
/// Cheap to clone — the backing state is shared with the
/// [`DaemonRuntime`] that produced it.
#[derive(Clone)]
pub struct MigrationHandle {
    /// Daemon being migrated.
    pub origin_hash: u32,
    /// Source node that currently hosts the daemon.
    pub source_node: u64,
    /// Target node that will host the daemon after cutover.
    pub target_node: u64,
    /// Runtime the orchestrator lives on. Used by [`Self::phase`]
    /// and [`Self::wait`] to poll migration state.
    runtime: DaemonRuntime,
}

impl MigrationHandle {
    /// Current migration phase, or `None` once the migration has
    /// left the orchestrator's records (either via `Complete` → auto
    /// cleanup, or via explicit abort). Callers distinguish the two
    /// by remembering the last non-None phase.
    pub fn phase(&self) -> Option<MigrationPhase> {
        self.runtime.inner.orchestrator.status(self.origin_hash)
    }

    /// Block until the migration reaches a terminal state.
    ///
    /// Returns `Ok(())` on normal completion (saw `Complete`, then
    /// the orchestrator cleaned up). Returns `Err(MigrationError)`
    /// if the orchestrator's record disappeared without the caller
    /// ever having seen `Complete` — either an explicit abort or a
    /// failure at some upstream stage.
    ///
    /// Polls every 50 ms. The implementation is deliberately
    /// simple — Stage 2 of `DAEMON_IDENTITY_MIGRATION_PLAN.md` and
    /// the V2 iteration of this plan will swap this for a
    /// broadcast-channel push, but 50 ms polling is plenty for the
    /// use cases a migration API sees today.
    pub async fn wait(self) -> Result<(), DaemonError> {
        self.wait_with_timeout(std::time::Duration::from_secs(60))
            .await
    }

    /// Like [`Self::wait`] with a caller-controlled timeout. A
    /// timeout aborts the orchestrator-side record and returns
    /// `Err(MigrationError::StateFailed)`; a graceful `Complete`
    /// returns `Ok`.
    pub async fn wait_with_timeout(
        self,
        timeout: std::time::Duration,
    ) -> Result<(), DaemonError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_phase: Option<MigrationPhase> = None;
        loop {
            match self.runtime.inner.orchestrator.status(self.origin_hash) {
                Some(phase) => {
                    last_phase = Some(phase);
                    if phase == MigrationPhase::Complete {
                        // Complete phase is reached and the orchestrator
                        // will clean up its record momentarily. Give it
                        // one more tick to settle, then return Ok.
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        return Ok(());
                    }
                }
                None => {
                    // Record gone. If we ever observed Complete, this
                    // is the normal cleanup path — success. Otherwise
                    // someone aborted.
                    if last_phase == Some(MigrationPhase::Complete) {
                        return Ok(());
                    }
                    return Err(DaemonError::Migration(MigrationError::StateFailed(
                        "migration aborted before reaching Complete".into(),
                    )));
                }
            }
            if tokio::time::Instant::now() >= deadline {
                let _ = self
                    .runtime
                    .inner
                    .orchestrator
                    .abort_migration(self.origin_hash, "timeout".into());
                return Err(DaemonError::Migration(MigrationError::StateFailed(
                    format!("migration timed out in phase {:?}", last_phase),
                )));
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    /// Request abort. The orchestrator emits a `MigrationFailed`
    /// message to involved nodes and clears its record; the target
    /// rolls back via its own handler. Best-effort — a migration
    /// past `Cutover` cannot be undone cleanly because routing has
    /// already flipped.
    pub async fn cancel(&self) -> Result<(), DaemonError> {
        let msg = self
            .runtime
            .inner
            .orchestrator
            .abort_migration(self.origin_hash, "cancel requested".into())
            .map_err(DaemonError::Migration)?;
        // Best-effort notify — ignore send errors, the orchestrator
        // record is already gone on our side.
        let _ = self
            .runtime
            .send_migration_message(self.source_node, &msg)
            .await;
        let _ = self
            .runtime
            .send_migration_message(self.target_node, &msg)
            .await;
        Ok(())
    }
}

impl std::fmt::Debug for MigrationHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationHandle")
            .field("origin_hash", &format_args!("{:#x}", self.origin_hash))
            .field("source_node", &format_args!("{:#x}", self.source_node))
            .field("target_node", &format_args!("{:#x}", self.target_node))
            .field("phase", &self.phase())
            .finish()
    }
}
