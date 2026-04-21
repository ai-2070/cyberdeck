//! Compute surface — `MeshDaemon` + `DaemonRuntime`.
//!
//! Users implement [`MeshDaemon`] and hand it to a [`DaemonRuntime`]
//! tied to a [`Mesh`] node. The runtime holds the
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
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use thiserror::Error;

pub use ::net::adapter::net::compute::{
    DaemonBindings, DaemonError as CoreDaemonError, DaemonHostConfig, DaemonStats, MeshDaemon,
    MigrationError, MigrationFailureReason, MigrationPhase, PlacementDecision, SchedulerError,
    SubscriptionBinding, SUBPROTOCOL_MIGRATION,
};
pub use ::net::adapter::net::state::causal::{CausalEvent, CausalLink};
pub use ::net::adapter::net::state::snapshot::StateSnapshot;

use ::net::adapter::net::channel::ChannelName;
use ::net::adapter::net::identity::PermissionToken;

use ::net::adapter::net::compute::{
    orchestrator::wire as migration_wire, DaemonFactoryRegistry, DaemonHost, DaemonRegistry,
    MigrationMessage, MigrationOrchestrator, MigrationSourceHandler, MigrationTargetHandler,
};
use ::net::adapter::net::identity::EntityId;
use ::net::adapter::net::subprotocol::{
    FailureCallback, MigrationHandlerHooks, MigrationSubprotocolHandler, PostRestoreCallback,
    PreCleanupCallback, ReadinessCallback,
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
    /// Structured failure reason surfaced by the migration
    /// dispatcher on the source side. Use
    /// [`MigrationFailureReason::is_retriable`] to decide whether
    /// the caller should back off and retry rather than propagating.
    #[error("migration failed: {0}")]
    MigrationFailed(MigrationFailureReason),
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
    /// Most recent `MigrationFailureReason` observed on the source
    /// side for each migration, keyed by `daemon_origin`. Populated
    /// by the dispatcher's failure callback; consumed by
    /// `MigrationHandle::wait` to surface the reason to the caller.
    /// Mutex because the SDK's dependency set doesn't include
    /// `dashmap` directly, and this map sees low write frequency
    /// (one entry per failed migration).
    recent_failures: Mutex<HashMap<u32, MigrationFailureReason>>,
    /// Test-only knob: when set to `true`, the readiness callback
    /// reports "not ready" even when the runtime is in `Ready`.
    /// Lets integration tests drive the `NotReady` retry path
    /// without racing against runtime startup. Defaults to `false`;
    /// production code should not touch it.
    simulate_not_ready: AtomicBool,
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
                recent_failures: Mutex::new(HashMap::new()),
                simulate_not_ready: AtomicBool::new(false),
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
                    // Install the migration subprotocol handler
                    // **before** publishing `Ready`. Other threads
                    // that observe `Ready` must be able to rely on
                    // the handler being live: the previous ordering
                    // (CAS → install) left a window where a
                    // concurrent caller read `Ready`, began a
                    // migration, and sent `SnapshotReady` onto a
                    // mesh whose handler slot was still empty —
                    // the dispatcher's no-handler fallback would
                    // synthesise `ComputeNotSupported`, aborting
                    // the migration nondeterministically during
                    // startup.
                    //
                    // Double-install is safe: `set_migration_handler`
                    // is an `ArcSwap` store, so if two concurrent
                    // `start()`s both reach this point the later
                    // store just wins and the CAS picks one caller
                    // to return first. Both built handlers are
                    // functionally equivalent (same registry,
                    // orchestrator, hooks).
                    let handler = Arc::new(self.build_migration_handler());
                    self.inner.mesh.inner().set_migration_handler(handler);

                    let swap = self.inner.state.compare_exchange(
                        State::Registering as u8,
                        State::Ready as u8,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    );
                    if swap.is_ok() {
                        return Ok(());
                    }
                    // Lost the CAS — another caller flipped the state
                    // and also installed a handler. Our own install
                    // may have overwritten theirs with an equivalent
                    // one; either way the mesh now holds a live
                    // handler. Loop to re-classify and return Ok on
                    // the `Ready` arm.
                }
                State::Ready => return Ok(()),
                State::ShuttingDown => return Err(DaemonError::ShuttingDown),
            }
        }
    }

    /// Construct the migration subprotocol handler with every
    /// hook wired — identity context, channel-rebind replay,
    /// unsubscribe teardown, readiness predicate, failure
    /// observer. Extracted so [`Self::start`] can install it
    /// before the atomic state flip (race fix) without burying
    /// the construction inline.
    fn build_migration_handler(&self) -> MigrationSubprotocolHandler {
        let local_node_id = self.inner.mesh.inner().node_id();
        // Ask the core crate to build the context. The Noise static
        // private key is captured inside closures the core owns and
        // never crosses this boundary as raw bytes — see
        // `MeshNode::migration_identity_context`.
        let ctx = self.inner.mesh.inner().migration_identity_context();
        let inner_for_rebind = self.inner.clone();
        let post_restore: PostRestoreCallback = Arc::new(move |origin_hash: u32| {
            let inner = inner_for_rebind.clone();
            tokio::spawn(async move {
                replay_subscriptions(inner, origin_hash).await;
            });
        });
        let inner_for_teardown = self.inner.clone();
        let pre_cleanup: PreCleanupCallback = Arc::new(move |origin_hash: u32| {
            // Snapshot the ledger BEFORE cleanup drops the host —
            // after that, the ledger is gone. Spawn async
            // unsubscribes so the dispatcher thread returns
            // immediately.
            let bindings = inner_for_teardown
                .registry
                .with_host(origin_hash, |host| host.bindings_snapshot().subscriptions)
                .unwrap_or_default();
            if bindings.is_empty() {
                return;
            }
            let inner = inner_for_teardown.clone();
            tokio::spawn(async move {
                teardown_subscriptions(inner, bindings).await;
            });
        });
        let inner_for_readiness = self.inner.clone();
        let readiness: ReadinessCallback = Arc::new(move || {
            // Test-only: `simulate_not_ready` flips the predicate
            // to false regardless of the underlying lifecycle
            // state. Honour it first so integration tests can
            // drive the NotReady retry path.
            if inner_for_readiness
                .simulate_not_ready
                .load(Ordering::Acquire)
            {
                return false;
            }
            inner_for_readiness.state.load(Ordering::Acquire) == State::Ready as u8
        });
        let inner_for_failure = self.inner.clone();
        let failure: FailureCallback =
            Arc::new(move |origin_hash: u32, reason: MigrationFailureReason| {
                if let Ok(mut map) = inner_for_failure.recent_failures.lock() {
                    map.insert(origin_hash, reason);
                }
            });
        MigrationSubprotocolHandler::with_hooks(
            self.inner.orchestrator.clone(),
            self.inner.source_handler.clone(),
            self.inner.target_handler.clone(),
            local_node_id,
            MigrationHandlerHooks {
                identity: Some(ctx),
                post_restore: Some(post_restore),
                pre_cleanup: Some(pre_cleanup),
                readiness: Some(readiness),
                failure: Some(failure),
            },
        )
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
        // Drop any leftover migration-failure entries so they
        // don't count against the process's memory footprint after
        // shutdown. The runtime is permanently non-functional at
        // this point, so no one will consume them.
        if let Ok(mut map) = self.inner.recent_failures.lock() {
            map.clear();
        }
        Ok(())
    }

    /// **Test-only.** Force the readiness predicate seen by the
    /// migration dispatcher to return `false` regardless of
    /// lifecycle state — simulates a target that's still in
    /// `Registering` even after `start()` has run. Lets
    /// integration tests exercise the `NotReady` retry path
    /// without racing against runtime startup.
    ///
    /// No effect on `is_ready()` or `spawn` / `stop` — those use
    /// the underlying `state` directly. Only the dispatcher's
    /// readiness predicate is affected.
    pub fn simulate_not_ready(&self, flag: bool) {
        self.inner.simulate_not_ready.store(flag, Ordering::Release);
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
        // this origin mid-spawn always sees a consistent view. Atomic
        // on collision: if another daemon already claims this
        // `origin_hash`, bail without mutating either registry —
        // otherwise the rollback below would strip the factory entry
        // for the *existing* daemon and silently break its future
        // migratability.
        let factory_for_core = factory.clone();
        self.inner
            .factory_registry
            .register(keypair.clone(), config.clone(), move || {
                (factory_for_core)()
            })
            .map_err(DaemonError::Core)?;

        let host = DaemonHost::new(daemon, keypair, config);
        // `DaemonRegistry::register` errors on origin_hash collisions
        // — two daemons can't share the same identity. Roll back the
        // factory_registry insert we just made (we own that slot
        // because `register` above succeeded atomically) so a retry
        // with a fresh identity doesn't pick up a stale factory.
        if let Err(e) = self.inner.registry.register(host) {
            self.inner.factory_registry.remove(origin_hash);
            return Err(DaemonError::Core(e));
        }

        // TEMP revert: no post-insert fence.
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

        // Compare the **full** 32-byte `entity_id`, not just the
        // 32-bit `origin_hash` projection. `origin_hash` is a
        // birthday-bounded 32-bit hash of the ed25519 public key;
        // two legitimately-different identities can collide on
        // `origin_hash` with probability ~2^-16 after ~65k daemons.
        // A collision would let the *wrong* identity restore the
        // snapshot, producing a daemon signed under one pubkey
        // but claiming to be another — downstream signatures then
        // verify against the wrong key and the daemon silently
        // produces outputs no peer accepts.
        if snapshot.entity_id != entity_id {
            return Err(DaemonError::SnapshotIdentityMismatch {
                snapshot: snapshot.entity_id.origin_hash(),
                identity: origin_hash,
            });
        }

        let daemon = (factory)();
        // Atomic register: collision here means some other daemon
        // (live or a previous spawn_from_snapshot in-flight) already
        // owns this `origin_hash`. Bail without touching the other
        // daemon's state — the later rollback would otherwise remove
        // the victim's factory entry and silently break its future
        // migratability.
        let factory_for_core = factory.clone();
        self.inner
            .factory_registry
            .register(keypair.clone(), config.clone(), move || {
                (factory_for_core)()
            })
            .map_err(DaemonError::Core)?;

        let host = match DaemonHost::from_snapshot(daemon, keypair, &snapshot, config) {
            Ok(h) => h,
            Err(e) => {
                // We own the factory slot (register above succeeded
                // atomically). Safe to remove on rollback.
                self.inner.factory_registry.remove(origin_hash);
                return Err(DaemonError::Core(e));
            }
        };

        if let Err(e) = self.inner.registry.register(host) {
            self.inner.factory_registry.remove(origin_hash);
            return Err(DaemonError::Core(e));
        }

        // Post-insert fence against a concurrent `shutdown` — see
        // the matching comment in `spawn`.
        if self.state() == State::ShuttingDown {
            let _ = self.inner.registry.unregister(origin_hash);
            self.inner.factory_registry.remove(origin_hash);
            return Err(DaemonError::ShuttingDown);
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

    /// Current orchestrator-side migration phase for `origin_hash`,
    /// or `None` when no migration record exists (either never
    /// started here or already reached its terminal state and was
    /// removed). Useful for tests that assert the migration reached
    /// true completion (record gone via `ActivateAck`) rather than
    /// simply advancing to the `Complete` phase.
    pub fn migration_phase(&self, origin_hash: u32) -> Option<MigrationPhase> {
        self.inner.orchestrator.status(origin_hash)
    }

    /// **Test-only.** Peek at the cached failure reason for
    /// `origin_hash` without consuming it.
    ///
    /// Normal `MigrationHandle::wait` code path pops the entry when
    /// it hits status=None; this accessor exists so regression tests
    /// can observe the cache's lifecycle directly (e.g. assert the
    /// cache is cleared by `start_migration_with`).
    ///
    /// Exposed publicly because SDK integration tests live in a
    /// separate crate; not part of the stable surface.
    #[doc(hidden)]
    pub fn peek_migration_failure(&self, origin_hash: u32) -> Option<MigrationFailureReason> {
        self.inner
            .recent_failures
            .lock()
            .ok()
            .and_then(|m| m.get(&origin_hash).cloned())
    }

    /// **Test-only.** Inject a failure reason into the cache for
    /// `origin_hash`. Lets tests stage a "stale entry from a prior
    /// attempt" scenario deterministically, without having to run
    /// a whole losing migration to populate it.
    ///
    /// Same caveat as [`Self::peek_migration_failure`] — visible
    /// because SDK integration tests live out-of-crate; not part of
    /// the stable surface.
    #[doc(hidden)]
    pub fn inject_migration_failure(&self, origin_hash: u32, reason: MigrationFailureReason) {
        if let Ok(mut m) = self.inner.recent_failures.lock() {
            m.insert(origin_hash, reason);
        }
    }

    /// Snapshot the daemon's subscription ledger — a cloned view of
    /// every `(publisher, channel)` pair the daemon has subscribed
    /// to via [`Self::subscribe_channel`]. Used by the migration
    /// target path to drive replay and by tests / operators to
    /// observe what a daemon is subscribed to.
    pub fn subscriptions(&self, origin_hash: u32) -> Result<Vec<SubscriptionBinding>, DaemonError> {
        self.inner
            .registry
            .with_host(origin_hash, |host| host.bindings_snapshot().subscriptions)
            .map_err(DaemonError::Core)
    }

    /// Subscribe a specific daemon to a channel on a remote
    /// publisher. Routes through the mesh's membership subprotocol
    /// and **records the subscription in the daemon's ledger** so
    /// a migration target can replay it after cutover. Users should
    /// use this method (rather than reaching through
    /// `rt.mesh().inner().subscribe_channel_*`) for daemon-owned
    /// subscriptions; otherwise the subscription travels with the
    /// node, not the daemon, and silently drops on migration.
    ///
    /// Flow:
    /// 1. Hit the publisher's membership endpoint via
    ///    `Mesh::subscribe_channel_with_token` (or the
    ///    no-token variant).
    /// 2. On success, record
    ///    `(publisher, channel) → SubscriptionBinding` in the
    ///    host's ledger.
    /// 3. On wire failure, no ledger mutation.
    ///
    /// `token` is the caller-owned [`PermissionToken`] for
    /// token-gated channels; `None` for open channels.
    pub async fn subscribe_channel(
        &self,
        origin_hash: u32,
        publisher: u64,
        channel: ChannelName,
        token: Option<PermissionToken>,
    ) -> Result<(), DaemonError> {
        self.require_ready()?;
        if !self.inner.registry.contains(origin_hash) {
            return Err(DaemonError::Core(CoreDaemonError::NotFound(origin_hash)));
        }
        // Capture serialized token bytes for the ledger BEFORE
        // handing ownership to the mesh. The mesh call consumes the
        // token by value on the token-path.
        let token_bytes = token.as_ref().map(|t| t.to_bytes().to_vec());
        let result = match token {
            Some(tok) => {
                self.inner
                    .mesh
                    .inner()
                    .subscribe_channel_with_token(publisher, channel.clone(), tok)
                    .await
            }
            None => {
                self.inner
                    .mesh
                    .inner()
                    .subscribe_channel(publisher, channel.clone())
                    .await
            }
        };
        result.map_err(|e| {
            DaemonError::Core(CoreDaemonError::ProcessFailed(format!(
                "subscribe_channel failed: {e}"
            )))
        })?;

        // Mesh accepted the subscribe — record in the ledger.
        // `with_host` reaches into the registry's per-daemon mutex;
        // since this is SDK-level code we do the minimum work
        // under the lock (a single DashMap insert).
        if let Err(e) = self.inner.registry.with_host(origin_hash, |host| {
            host.record_subscription(publisher, channel, token_bytes);
        }) {
            return Err(DaemonError::Core(e));
        }
        Ok(())
    }

    /// Unsubscribe a specific daemon from a channel. Symmetric to
    /// [`Self::subscribe_channel`]: mesh wire call first, then
    /// ledger update.
    pub async fn unsubscribe_channel(
        &self,
        origin_hash: u32,
        publisher: u64,
        channel: ChannelName,
    ) -> Result<(), DaemonError> {
        self.require_ready()?;
        if !self.inner.registry.contains(origin_hash) {
            return Err(DaemonError::Core(CoreDaemonError::NotFound(origin_hash)));
        }
        self.inner
            .mesh
            .inner()
            .unsubscribe_channel(publisher, channel.clone())
            .await
            .map_err(|e| {
                DaemonError::Core(CoreDaemonError::ProcessFailed(format!(
                    "unsubscribe_channel failed: {e}"
                )))
            })?;
        let _ = self.inner.registry.with_host(origin_hash, |host| {
            host.forget_subscription(publisher, &channel);
        });
        Ok(())
    }

    /// Pre-register a factory on the target node keyed by the
    /// daemon's `origin_hash`, using the caller-supplied `Identity`
    /// as the **fallback** keypair.
    ///
    /// Use this when:
    /// - The caller genuinely has the daemon's keypair on hand
    ///   (typical: test harnesses that share the same
    ///   `Identity` between source and target runtimes).
    /// - Migration runs with `transport_identity = false`, so the
    ///   snapshot carries no envelope and the target needs a
    ///   matching keypair pre-provisioned.
    ///
    /// For the common envelope-transport case where the target
    /// doesn't know the daemon's private key ahead of time,
    /// prefer [`Self::expect_migration`] — it registers a
    /// placeholder factory keyed only on `origin_hash`, and the
    /// envelope in the migration snapshot supplies the real
    /// keypair at restore time.
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
        let origin_hash = keypair.origin_hash();
        let factory_clone = factory.clone();
        self.inner
            .factory_registry
            .register(keypair, config, move || (factory_clone)())
            .map_err(DaemonError::Core)?;
        // Post-insert fence — a concurrent `shutdown` may have
        // raced past the initial state check. Roll back so no
        // factory entry outlives the torn-down runtime.
        if self.state() == State::ShuttingDown {
            self.inner.factory_registry.remove(origin_hash);
            return Err(DaemonError::ShuttingDown);
        }
        Ok(())
    }

    /// Declare on the target that this node expects a migration
    /// for `origin_hash` of the given `kind`. Registers a
    /// **placeholder** factory in the core registry — no matching
    /// keypair required, because the migration snapshot's
    /// [`IdentityEnvelope`](::net::adapter::net::identity::IdentityEnvelope)
    /// carries the real keypair and the dispatcher overrides the
    /// placeholder at restore time.
    ///
    /// Fails cleanly if the source migrates without an envelope
    /// (e.g., `MigrationOpts { transport_identity: false }`) —
    /// the target's factory has no keypair and the dispatcher
    /// emits `IdentityTransportFailed`. Use
    /// [`Self::register_migration_target_identity`] with a shared
    /// identity for the explicit public-identity-migration case.
    ///
    /// Landing this method closes the seam documented in the
    /// `envelope_overrides_target_placeholder_keypair` test of
    /// Stage 5b of the identity-migration plan — targets can now
    /// pre-register for a migration by `origin_hash` alone.
    pub fn expect_migration(
        &self,
        kind: &str,
        origin_hash: u32,
        config: DaemonHostConfig,
    ) -> Result<(), DaemonError> {
        if self.state() == State::ShuttingDown {
            return Err(DaemonError::ShuttingDown);
        }
        let factory = self.factory_for_kind(kind)?;
        let factory_clone = factory.clone();
        self.inner
            .factory_registry
            .register_placeholder(origin_hash, config, move || (factory_clone)())
            .map_err(DaemonError::Core)?;
        // Post-insert fence against a concurrent `shutdown` — see
        // the matching comment in `register_migration_target_identity`.
        if self.state() == State::ShuttingDown {
            self.inner.factory_registry.remove(origin_hash);
            return Err(DaemonError::ShuttingDown);
        }
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
        self.start_migration_with(
            origin_hash,
            source_node,
            target_node,
            MigrationOpts::default(),
        )
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
        // Clear any stale failure reason from a prior migration
        // attempt for this same `origin_hash`. Without this, if the
        // previous attempt's `MigrationHandle` was dropped before
        // `wait()` ran (or never `wait()`ed at all), the dispatcher's
        // failure callback left an entry in `recent_failures` that
        // would leak into THIS attempt's `wait()` — a successful
        // new migration would incorrectly surface the old reason
        // when `wait_one_attempt` hits its None-status branch and
        // pops `recent_failures`.
        if let Ok(mut map) = self.inner.recent_failures.lock() {
            map.remove(&origin_hash);
        }
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
                // `?` on the seal's `Result` propagates hard failures
                // (prerequisites met but crypto bug) and leaves the
                // caller's orchestrator record to abort cleanly. A
                // quiet `Ok(None)` means "prerequisites missing;
                // proceed unsealed" — the documented fallback.
                match self.maybe_seal_local_snapshot(
                    origin_hash,
                    target_node,
                    snapshot_bytes,
                ) {
                    Ok(Some(sealed)) => *snapshot_bytes = sealed,
                    Ok(None) => {}
                    Err(e) => {
                        // The orchestrator record was created by the
                        // preceding `start_migration` call — roll it
                        // back so a retry starts from phase 0.
                        let _ = self.inner.orchestrator.abort_migration(
                            origin_hash,
                            format!("envelope seal failed: {e}"),
                        );
                        return Err(e);
                    }
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
                return Err(DaemonError::Migration(MigrationError::StateFailed(
                    format!(
                        "orchestrator returned unexpected initial migration message: {:?}",
                        other
                    ),
                )));
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
            opts,
        })
    }

    /// Decode `snapshot_bytes`, attempt to seal an identity envelope
    /// using the local daemon's keypair + target's X25519 static
    /// pubkey, and re-encode.
    ///
    /// Resolution:
    /// - `Ok(None)`: the snapshot already has an envelope, or a
    ///   transport prerequisite (peer static not known, daemon
    ///   keypair absent) is missing. The caller proceeds with the
    ///   original unsealed bytes — this matches the documented
    ///   NKpsk0-responder / public-identity fallback.
    /// - `Ok(Some(new_bytes))`: sealed successfully; caller should
    ///   replace the snapshot payload.
    /// - `Err(_)`: prerequisites were met but the seal operation
    ///   itself failed. The caller **must** abort — silently
    ///   proceeding with the unsealed snapshot would break the
    ///   identity-transport guarantee the caller opted into via
    ///   `MigrationOpts { transport_identity: true }`, and the
    ///   target would restore under whatever fallback keypair the
    ///   factory registry carries (possibly a stale or absent one).
    fn maybe_seal_local_snapshot(
        &self,
        daemon_origin: u32,
        target_node: u64,
        snapshot_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, DaemonError> {
        let snapshot = StateSnapshot::from_bytes(snapshot_bytes).ok_or_else(|| {
            DaemonError::Migration(MigrationError::StateFailed(
                "failed to decode local snapshot for envelope sealing".into(),
            ))
        })?;
        if snapshot.identity_envelope.is_some() {
            return Ok(None);
        }
        let Some(target_pub) = self.inner.mesh.inner().peer_static_x25519(target_node) else {
            return Ok(None);
        };
        let Some(kp) = self.inner.registry.daemon_keypair(daemon_origin) else {
            return Ok(None);
        };
        snapshot
            .with_identity_envelope(&kp, target_pub)
            .map(|sealed| Some(sealed.to_bytes()))
            .map_err(|e| {
                DaemonError::Migration(MigrationError::StateFailed(format!(
                    "identity envelope seal failed for daemon {daemon_origin:#x}: {e}"
                )))
            })
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
            .ok_or(DaemonError::Migration(MigrationError::TargetUnavailable(
                dest_node,
            )))?;
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

/// Stage 3 of `DAEMON_CHANNEL_REBIND_PLAN.md` — after a migration
/// target restores a daemon, walk its subscription ledger and
/// re-send each `subscribe_channel` to the matching publisher so
/// messages flow to the target without waiting for the source's
/// entry to age out of the publisher's roster. Errors are
/// per-subscription; one publisher being offline doesn't fail the
/// rest.
///
/// Runs in a tokio task spawned by the post-restore callback
/// installed on `MigrationSubprotocolHandler`.
async fn replay_subscriptions(inner: Arc<Inner>, origin_hash: u32) {
    let bindings = match inner
        .registry
        .with_host(origin_hash, |host| host.bindings_snapshot().subscriptions)
    {
        Ok(list) => list,
        Err(_) => return,
    };
    for sub in bindings {
        let token = sub
            .token_bytes
            .as_deref()
            .and_then(|bytes| PermissionToken::from_bytes(bytes).ok());
        let result = match token {
            Some(tok) => {
                inner
                    .mesh
                    .inner()
                    .subscribe_channel_with_token(sub.publisher, sub.channel.clone(), tok)
                    .await
            }
            None => {
                inner
                    .mesh
                    .inner()
                    .subscribe_channel(sub.publisher, sub.channel.clone())
                    .await
            }
        };
        if let Err(e) = result {
            // Non-fatal: one subscription failing (publisher
            // offline, token expired, etc.) must not take down the
            // rest of the ledger's replay. The SDK doesn't depend
            // on `tracing`; drop the failure to stderr via
            // `eprintln!` so operators running `RUST_LOG=warn`
            // still see it. Future work can add a `ReplayPartial`
            // event on the migration phase stream (plan §
            // *Error surface*) to surface failures programmatically.
            eprintln!(
                "channel re-bind replay failed: daemon={:#x} channel={} publisher={:#x} error={}",
                origin_hash, sub.channel, sub.publisher, e,
            );
        }
    }
}

/// Stage 4 of `DAEMON_CHANNEL_REBIND_PLAN.md` — fires at `Cutover`
/// on the source node (just before daemon cleanup). Walks the
/// daemon's ledger and sends `unsubscribe_channel` to each
/// publisher so rosters drop the source without waiting for
/// session-timeout (~30 s). Fire-and-forget: we don't block the
/// cutover dispatch on acks.
async fn teardown_subscriptions(inner: Arc<Inner>, bindings: Vec<SubscriptionBinding>) {
    for sub in bindings {
        if let Err(e) = inner
            .mesh
            .inner()
            .unsubscribe_channel(sub.publisher, sub.channel.clone())
            .await
        {
            // Non-fatal; the publisher's session timeout will
            // eventually clean up our stale roster entry even
            // without an explicit Unsubscribe.
            eprintln!(
                "channel re-bind teardown failed: channel={} publisher={:#x} error={}",
                sub.channel, sub.publisher, e,
            );
        }
    }
}

/// Options for [`DaemonRuntime::start_migration_with`].
///
/// - Stage 6 of
///   [`DAEMON_IDENTITY_MIGRATION_PLAN.md`](../../../docs/DAEMON_IDENTITY_MIGRATION_PLAN.md):
///   the `transport_identity` flag. Default `true`.
/// - Stages 3 + 4 of
///   [`DAEMON_RUNTIME_READINESS_PLAN.md`](../../../docs/DAEMON_RUNTIME_READINESS_PLAN.md):
///   the `retry_not_ready` budget. When the migration target
///   responds `NotReady` (runtime still in `Registering`), the
///   source backs off + re-initiates up to this total elapsed
///   time. `None` disables retry; the first `NotReady` surfaces
///   immediately.
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

    /// Retry budget for [`MigrationFailureReason::NotReady`].
    ///
    /// `Some(d)` (default 30 s): on `NotReady`, the source backs
    /// off (500 ms → 1 s → 2 s → 4 s → 8 s, capped at 16 s) and
    /// re-initiates the migration. The total retry clock is
    /// capped at `d`; after that, the caller sees
    /// [`MigrationFailureReason::NotReadyTimeout`].
    ///
    /// `None`: no retry. The first `NotReady` surfaces as a
    /// terminal failure to the caller.
    pub retry_not_ready: Option<std::time::Duration>,
}

impl Default for MigrationOpts {
    fn default() -> Self {
        Self {
            transport_identity: true,
            retry_not_ready: Some(std::time::Duration::from_secs(30)),
        }
    }
}

/// Exponential backoff for the i-th `NotReady` retry attempt.
/// First retry waits 500 ms, subsequent retries double up to a
/// 16 s cap — total budget is controlled separately by
/// [`MigrationOpts::retry_not_ready`]. Matching the schedule in
/// `DAEMON_RUNTIME_READINESS_PLAN.md` § *Source-side retry*.
fn not_ready_backoff(attempt: u8) -> std::time::Duration {
    use std::time::Duration;
    let ms = 500u64 << (attempt.saturating_sub(1).min(5));
    Duration::from_millis(ms.min(16_000))
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
    /// Options the migration was initiated with. Drives retry
    /// policy on `NotReady` + the identity-transport flag for
    /// re-initiated attempts.
    opts: MigrationOpts,
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
    /// This method does **not** enforce any wall-clock timeout. A
    /// migration that stalls waiting on an unresponsive peer will
    /// block indefinitely; callers that want a bound should use
    /// [`Self::wait_with_timeout`] instead.
    ///
    /// Polls every 50 ms. The implementation is deliberately
    /// simple — Stage 2 of `DAEMON_IDENTITY_MIGRATION_PLAN.md` and
    /// the V2 iteration of this plan will swap this for a
    /// broadcast-channel push, but 50 ms polling is plenty for the
    /// use cases a migration API sees today.
    pub async fn wait(self) -> Result<(), DaemonError> {
        self.wait_until(None).await
    }

    /// Like [`Self::wait`] with a caller-controlled timeout. A
    /// timeout aborts the orchestrator-side record and returns
    /// `Err(MigrationError::StateFailed)`; a graceful `Complete`
    /// returns `Ok`.
    pub async fn wait_with_timeout(self, timeout: std::time::Duration) -> Result<(), DaemonError> {
        let deadline = tokio::time::Instant::now() + timeout;
        self.wait_until(Some(deadline)).await
    }

    /// Inner wait loop with an optional deadline. `None` = block
    /// forever until the migration reaches a terminal state;
    /// `Some(d)` = give up and abort at `d`.
    ///
    /// Centralised here so `wait` and `wait_with_timeout` can't
    /// drift on the retry/backoff semantics, and so the
    /// "hidden 60 s ceiling" cannot reappear under `wait`.
    async fn wait_until(
        self,
        overall_deadline: Option<tokio::time::Instant>,
    ) -> Result<(), DaemonError> {
        let start = tokio::time::Instant::now();
        let retry_deadline = self.opts.retry_not_ready.map(|b| start + b);
        let mut attempts: u8 = 1; // first attempt initiated by start_migration_with
        loop {
            match self.wait_one_attempt(overall_deadline).await {
                Ok(()) => return Ok(()),
                Err(DaemonError::MigrationFailed(reason)) if reason.is_retriable() => {
                    // Retry decision.
                    let Some(retry_d) = retry_deadline else {
                        // Opts explicitly disable retry — surface
                        // the NotReady verbatim.
                        return Err(DaemonError::MigrationFailed(reason));
                    };
                    let now = tokio::time::Instant::now();
                    let overall_exhausted = overall_deadline.map(|d| now >= d).unwrap_or(false);
                    if now >= retry_d || overall_exhausted {
                        // Budget exhausted. `NotReadyTimeout` carries
                        // the attempt count for operator diagnosis.
                        return Err(DaemonError::MigrationFailed(
                            MigrationFailureReason::NotReadyTimeout { attempts },
                        ));
                    }
                    // Back off and re-initiate.
                    let backoff = not_ready_backoff(attempts);
                    tokio::time::sleep(backoff).await;
                    attempts = attempts.saturating_add(1);
                    // Re-initiate the migration by calling the
                    // orchestrator fresh. The previous record has
                    // been cleaned up by the dispatcher's
                    // MigrationFailed handler, so this starts a new
                    // attempt from phase 0.
                    self.reinitiate_attempt().await?;
                    // Loop; poll this new attempt's outcome.
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Poll a single migration attempt's outcome. Returns:
    /// - `Ok(())` on Complete.
    /// - `Err(MigrationFailed(reason))` when the dispatcher
    ///   observed a structured failure.
    /// - `Err(Migration(_))` on overall-timeout or unknown abort.
    ///
    /// `overall_deadline = None` disables the deadline check — the
    /// poll loop only returns via a terminal status transition.
    async fn wait_one_attempt(
        &self,
        overall_deadline: Option<tokio::time::Instant>,
    ) -> Result<(), DaemonError> {
        loop {
            let current_phase = self.runtime.inner.orchestrator.status(self.origin_hash);
            match current_phase {
                Some(phase) => {
                    if phase == MigrationPhase::Complete {
                        // Give the dispatcher a beat to finish
                        // cleanup, then surface success.
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        return Ok(());
                    }
                }
                None => {
                    // A recorded failure is authoritative — the
                    // dispatcher populates `recent_failures` before
                    // the orchestrator removes the record, so
                    // status=None + recorded-reason unambiguously
                    // means abort.
                    if let Some(reason) = self.take_recent_failure() {
                        return Err(DaemonError::MigrationFailed(reason));
                    }
                    // No recorded failure. The orchestrator removes
                    // records via two paths:
                    //   1. `on_activate_ack` — success.
                    //   2. `abort_migration_with_reason` — failure,
                    //      which rides through the dispatcher's
                    //      `MigrationFailed` handler *before* the
                    //      record is dropped, so `recent_failures`
                    //      is populated first.
                    // Therefore status=None + no-recorded-failure is
                    // unambiguously success, regardless of what
                    // phase we last observed. This matters when the
                    // dispatcher runs the tail of the migration
                    // (Cutover → Complete → ActivateAck) entirely
                    // between two 50 ms polls — we may never observe
                    // `Complete` explicitly.
                    //
                    // Synchronous abort paths that bypass the
                    // dispatcher (`wait_one_attempt`'s own timeout,
                    // `start_migration_with`'s send-failure path)
                    // return `Err` to the caller *before* the wait
                    // loop can observe the None status, so they
                    // don't trip this branch.
                    return Ok(());
                }
            }
            if let Some(d) = overall_deadline {
                if tokio::time::Instant::now() >= d {
                    let _ = self
                        .runtime
                        .inner
                        .orchestrator
                        .abort_migration(self.origin_hash, "timeout".into());
                    return Err(DaemonError::Migration(MigrationError::StateFailed(
                        format!("migration timed out in phase {:?}", current_phase),
                    )));
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    /// Re-initiate a migration attempt for the same daemon after a
    /// retriable failure. Calls the orchestrator fresh + sends the
    /// first wire message; mirrors the tail of
    /// [`DaemonRuntime::start_migration_with`] but without building
    /// a new handle (we keep the existing one).
    async fn reinitiate_attempt(&self) -> Result<(), DaemonError> {
        let mut first_msg = self
            .runtime
            .inner
            .orchestrator
            .start_migration(self.origin_hash, self.source_node, self.target_node)
            .map_err(DaemonError::Migration)?;

        if self.opts.transport_identity {
            if let MigrationMessage::SnapshotReady {
                ref mut snapshot_bytes,
                ..
            } = first_msg
            {
                match self.runtime.maybe_seal_local_snapshot(
                    self.origin_hash,
                    self.target_node,
                    snapshot_bytes,
                ) {
                    Ok(Some(sealed)) => *snapshot_bytes = sealed,
                    Ok(None) => {}
                    Err(e) => {
                        let _ = self.runtime.inner.orchestrator.abort_migration(
                            self.origin_hash,
                            format!("retry envelope seal failed: {e}"),
                        );
                        return Err(e);
                    }
                }
            }
        }

        let (dest_node, payload) = match &first_msg {
            MigrationMessage::TakeSnapshot { .. } => (self.source_node, &first_msg),
            MigrationMessage::SnapshotReady { .. } => (self.target_node, &first_msg),
            other => {
                let _ = self
                    .runtime
                    .inner
                    .orchestrator
                    .abort_migration(self.origin_hash, "unexpected retry message".into());
                return Err(DaemonError::Migration(MigrationError::StateFailed(
                    format!("unexpected retry initial message: {other:?}"),
                )));
            }
        };

        self.runtime
            .send_migration_message(dest_node, payload)
            .await
    }

    /// Pop the most recent `MigrationFailureReason` the dispatcher
    /// observed for this migration (if any). Consumed: subsequent
    /// calls see `None` until the next failure arrives.
    fn take_recent_failure(&self) -> Option<MigrationFailureReason> {
        self.runtime
            .inner
            .recent_failures
            .lock()
            .ok()?
            .remove(&self.origin_hash)
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
