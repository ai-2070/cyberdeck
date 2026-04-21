// `#[napi]` exports functions to JS but leaves them "unused" from
// Rust's POV, so clippy's dead-code analysis doesn't apply to this
// module. Suppress at file scope.
#![allow(dead_code)]

//! NAPI surface for the compute runtime — `MeshDaemon` + migration.
//!
//! Stage 3 of `SDK_COMPUTE_SURFACE_PLAN.md`.
//!
//! **Sub-step 2a** (this file): a TS caller can `spawn` and `stop`
//! daemons; the spawned daemon is a `NoopBridge` that implements
//! `MeshDaemon` with no-op methods. Event delivery is not yet
//! wired — sub-step 2b will invoke the factory TSFN, extract JS
//! `process` / `snapshot` / `restore` methods, and replace the
//! `NoopBridge` with an `EventDispatchBridge`.
//!
//! # Error prefix
//!
//! Every `Error` produced here is prefixed with `daemon:` so the TS
//! side's `classifyError` can route to a dedicated `DaemonError`
//! class. Mirrors the `identity:` / `cortex:` / `token:` convention
//! used by other modules.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use bytes::Bytes;
use dashmap::DashMap;
use std::sync::Arc;

use net::adapter::net::behavior::capability::CapabilityFilter;
use net::adapter::net::compute::{DaemonError as CoreDaemonError, DaemonHostConfig, MeshDaemon};
use net::adapter::net::state::causal::CausalEvent;
use net_sdk::compute::{
    DaemonHandle as SdkDaemonHandle, DaemonRuntime as SdkDaemonRuntime, StateSnapshot,
};
use net_sdk::mesh::Mesh as SdkMesh;

use crate::NetMesh;

// =========================================================================
// Error prefix — stable string the TS layer dispatches on
// =========================================================================

const ERR_DAEMON_PREFIX: &str = "daemon:";

fn daemon_err(msg: impl Into<String>) -> Error {
    Error::from_reason(format!("{} {}", ERR_DAEMON_PREFIX, msg.into()))
}

// =========================================================================
// NAPI class — DaemonRuntime
// =========================================================================

/// Factory closure handle: a `ThreadsafeFunction` built from the
/// JS function passed to `register_factory`. Once built, the TSFN
/// is `Send + Sync + Clone` — it can be called from any tokio task
/// without being pinned to the Node main thread.
///
/// Return type is `UnknownReturnValue` — napi-rs's `'static`
/// placeholder for "we don't inspect the return value here." The
/// TSFN alias would otherwise inherit the call-site `Unknown<'_>`
/// lifetime from the `Function` param, which isn't `'static`-
/// storable. Sub-step 3 will read the concrete object shape off
/// the returned JS value inside the `call` callback, so holding
/// the typed return here adds nothing.
///
/// `CalleeHandled` left at the builder's default (`false`) —
/// we'll wire callee-side error handling when sub-step 3 brings
/// up the `process` dispatch path.
type FactoryTsfn = napi::threadsafe_function::ThreadsafeFunction<
    (),
    napi::threadsafe_function::UnknownReturnValue,
    (),
    napi::Status,
    false,
>;

/// `process` TSFN — invoked by [`EventDispatchBridge::process`]
/// on every inbound causal event. Takes a [`CausalEventJs`] by
/// value, returns `Buffer[]` which NAPI marshals into
/// `Vec<Buffer>` here.
///
/// `CalleeHandled = false`: we deal with JS-thrown errors by
/// routing them through the `Result<Return>` callback in
/// `call_with_return_value` (wrapped as `DaemonError::ProcessFailed`
/// on the way back into the SDK). No need for napi-rs's
/// callee-side error-propagation plumbing on top of that.
type ProcessTsfn = napi::threadsafe_function::ThreadsafeFunction<
    CausalEventJs,
    Vec<Buffer>,
    CausalEventJs,
    napi::Status,
    false,
>;

/// `snapshot` TSFN — invoked by the SDK's `take_snapshot` path.
/// Takes no args; returns `Buffer | null` which napi-rs marshals
/// into `Option<Buffer>` via `FromNapiValue`. A stateless daemon
/// returns `null`, which propagates as `Option<Bytes>::None` to
/// the core.
type SnapshotTsfn =
    napi::threadsafe_function::ThreadsafeFunction<(), Option<Buffer>, (), napi::Status, false>;

/// `restore` TSFN — invoked by the SDK on
/// `DaemonHost::from_snapshot`. Takes one `Buffer` (the daemon's
/// serialized state) and returns nothing useful — we only care
/// whether JS throws. `UnknownReturnValue` is napi-rs's `'static`
/// placeholder for "discard the return value."
type RestoreTsfn = napi::threadsafe_function::ThreadsafeFunction<
    Buffer,
    napi::threadsafe_function::UnknownReturnValue,
    Buffer,
    napi::Status,
    false,
>;

/// Per-runtime compute surface. One instance per `NetMesh`; clone
/// handles are `Arc`-shared internally.
#[napi]
pub struct DaemonRuntime {
    /// SDK-level runtime. Owns the daemon registry, factory
    /// registry, migration orchestrator, and lifecycle state. The
    /// NAPI layer is a thin envelope — behavior lives in the SDK.
    inner: Arc<SdkDaemonRuntime>,
    /// JS-side factory table. Keyed by the `kind` string; value is
    /// the `ThreadsafeFunction` returned by NAPI when the caller
    /// passed a JS function. Sub-step 2 consumes these at `spawn`
    /// time to build each daemon's `process` / `snapshot` /
    /// `restore` TSFNs. Sub-step 1 just stores and drops.
    factories: Arc<DashMap<String, FactoryTsfn>>,
}

#[napi]
impl DaemonRuntime {
    /// Build a compute runtime against an existing `NetMesh`.
    ///
    /// Shares the mesh's live `MeshNode` — no new socket, no new
    /// handshake table. The caller keeps ownership of their
    /// `NetMesh`; shutting down the `DaemonRuntime` does **not**
    /// shut down the underlying mesh.
    #[napi(factory)]
    pub fn create(mesh: &NetMesh) -> Result<DaemonRuntime> {
        let node = mesh.node_arc_clone()?;
        let channel_configs = mesh.channel_configs_arc();
        // Build an SDK-level `Mesh` that shares the same live
        // `MeshNode` as the caller's `NetMesh`. Identity is `None`
        // here because the NAPI layer manages identity separately
        // (via the `Identity` class); the daemon runtime only
        // needs the mesh for node_id, peer lookup, and subprotocol
        // handler install.
        let sdk_mesh = SdkMesh::from_node_arc(node, channel_configs, None);
        let sdk_rt = SdkDaemonRuntime::new(Arc::new(sdk_mesh));
        Ok(DaemonRuntime {
            inner: Arc::new(sdk_rt),
            factories: Arc::new(DashMap::new()),
        })
    }

    /// Transition to `Ready`. Installs the migration subprotocol
    /// handler on the underlying mesh. Idempotent — a second call
    /// on a runtime that's already `Ready` is a no-op; a call on
    /// a `ShuttingDown` runtime returns `daemon: shutting down`.
    #[napi]
    pub async fn start(&self) -> Result<()> {
        self.inner
            .start()
            .await
            .map_err(|e| daemon_err(e.to_string()))
    }

    /// Tear down the runtime. Drains daemons, clears factory
    /// registrations, uninstalls the migration handler. The
    /// underlying `NetMesh` is untouched.
    ///
    /// Factory TSFNs held by this runtime are dropped here, which
    /// releases their JS-side refs so the Node process can exit
    /// cleanly. Sub-step 1 only stores TSFNs — no per-daemon
    /// bridge TSFNs to clean up yet.
    #[napi]
    pub async fn shutdown(&self) -> Result<()> {
        self.inner
            .shutdown()
            .await
            .map_err(|e| daemon_err(e.to_string()))?;
        self.factories.clear();
        Ok(())
    }

    /// `true` iff the runtime has transitioned to `Ready` and
    /// has not yet begun shutting down.
    #[napi]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    /// Number of daemons currently registered with the runtime.
    #[napi]
    pub fn daemon_count(&self) -> u32 {
        // Cast: the SDK returns `usize`. Daemon counts are
        // realistically << 2^32; NAPI needs a number type.
        self.inner.daemon_count() as u32
    }

    /// Register a factory closure under `kind`. The factory is a
    /// JS function that returns a `MeshDaemon`-shaped object
    /// (with `process` / `snapshot` / `restore` methods).
    ///
    /// Sub-step 1 stores the factory but does **not invoke it**
    /// — event dispatch + daemon construction land in sub-step 2.
    /// Second registration of the same `kind` returns
    /// `daemon: factory for kind '<kind>' is already registered`.
    #[napi]
    pub fn register_factory(
        &self,
        kind: String,
        factory: Function<'_, (), napi::threadsafe_function::UnknownReturnValue>,
    ) -> Result<()> {
        // Build a threadsafe handle so the factory can be called
        // from any tokio task (sub-step 2 will do this at spawn
        // time). The TSFN is `Send + Sync + Clone` — once built we
        // can invoke it off the Node main thread.
        let tsfn: FactoryTsfn = factory.build_threadsafe_function().build()?;

        // Atomic insert-or-error via DashMap's entry API. Matches
        // the SDK's `register_factory` contract — "second
        // registration fails."
        use dashmap::mapref::entry::Entry;
        match self.factories.entry(kind.clone()) {
            Entry::Occupied(_) => Err(daemon_err(format!(
                "factory for kind '{kind}' is already registered"
            ))),
            Entry::Vacant(slot) => {
                slot.insert(tsfn);
                Ok(())
            }
        }
    }

    /// Spawn a daemon of `kind` under the given identity with
    /// pre-bound `process` / `snapshot` / `restore` callbacks.
    ///
    /// The TS wrapper invokes the user-supplied factory in JS
    /// land, extracts the returned daemon object's methods, and
    /// passes them here as three separate `Function`s. NAPI wraps
    /// each in a `ThreadsafeFunction` so the eventual event
    /// dispatch (sub-step 3) can call them from any tokio task
    /// without being pinned to the Node main thread.
    ///
    /// **Sub-step 2b** (this file): the `EventDispatchBridge`
    /// stores the three TSFNs but its `MeshDaemon::process`
    /// implementation returns an empty output. Sub-step 3 wires
    /// that method to call `process_tsfn` synchronously and
    /// marshal the result.
    ///
    /// `snapshot` / `restore` are optional — stateless daemons
    /// omit them. If `snapshot` is present, the stored TSFN will
    /// be called by the host on `take_snapshot`; if absent, the
    /// host reports the daemon as stateless.
    ///
    /// # Method is sync but returns a `PromiseRaw`
    ///
    /// napi-rs `Function` values are `!Send`, so an `async fn`
    /// taking them would produce a non-`Send` future that tokio's
    /// worker pool can't schedule. The two-stage shape here
    /// (sync consumes the `Function`s to build `TSFN`s → then
    /// hands all-`Send` state to `env.spawn_future`) is the
    /// idiomatic napi-rs pattern for "sync setup, async
    /// continuation."
    #[napi]
    pub fn spawn<'env>(
        &'env self,
        env: &'env Env,
        kind: String,
        identity: &crate::identity::Identity,
        process: Function<'_, CausalEventJs, Vec<Buffer>>,
        snapshot: Option<Function<'_, (), Option<Buffer>>>,
        restore: Option<Function<'_, Buffer, napi::threadsafe_function::UnknownReturnValue>>,
        config: Option<DaemonHostConfigJs>,
    ) -> Result<napi::bindgen_prelude::PromiseRaw<'env, DaemonHandle>> {
        // Guard: kind must have been registered. Registration is
        // the TS API's contract for "the target knows about this
        // daemon type"; skipping it would hide mis-configured TS
        // callers until a downstream operation fails cryptically.
        if !self.factories.contains_key(&kind) {
            return Err(daemon_err(format!(
                "no factory registered for kind '{kind}'"
            )));
        }

        // Build TSFNs synchronously — the napi-rs `Function`
        // values are `!Send`, so we can't carry them into the
        // async future. Each TSFN is `Send + Sync + Clone`; the
        // future below holds only TSFNs.
        let process_tsfn: ProcessTsfn = process.build_threadsafe_function().build()?;
        let snapshot_tsfn: Option<SnapshotTsfn> = match snapshot {
            Some(f) => Some(f.build_threadsafe_function().build()?),
            None => None,
        };
        let restore_tsfn: Option<RestoreTsfn> = match restore {
            Some(f) => Some(f.build_threadsafe_function().build()?),
            None => None,
        };

        let sdk_identity = identity.to_sdk_identity();
        let sdk_config = config.map(DaemonHostConfig::from).unwrap_or_default();
        let runtime = self.inner.clone();
        let kind_label = kind.clone();

        let bridge = Box::new(EventDispatchBridge {
            name: kind,
            process: process_tsfn,
            snapshot: snapshot_tsfn,
            restore: restore_tsfn,
        });

        // Kind-factory closure for migration-target reconstruction.
        // The TS node can't yet serve migrations targeting it
        // because the factory TSFN's return value (a JS daemon
        // object with method closures) can't be reconstructed from
        // Rust without re-running the TS factory. Sub-step 4+ will
        // address this; for now hand out NoopBridges so the core
        // registry has *some* factory — migrations into this node
        // will fail downstream until the full path lands.
        let kind_factory =
            move || -> Box<dyn MeshDaemon> { Box::new(NoopBridge::new(kind_label.clone())) };

        env.spawn_future(async move {
            runtime
                .spawn_with_daemon(sdk_identity, sdk_config, bridge, kind_factory)
                .await
                .map(DaemonHandle::from_sdk)
                .map_err(|e| daemon_err(e.to_string()))
        })
    }

    /// Stop a daemon, removing it from the runtime's registry.
    ///
    /// `origin_hash` is the 32-bit identifier carried on
    /// [`DaemonHandle`]. A second call for the same origin is a
    /// no-op during `ShuttingDown` and an error otherwise; the
    /// SDK's error is surfaced verbatim with the `daemon:` prefix.
    #[napi]
    pub async fn stop(&self, origin_hash: u32) -> Result<()> {
        self.inner
            .stop(origin_hash)
            .await
            .map_err(|e| daemon_err(e.to_string()))
    }

    /// Take a snapshot of a running daemon by `origin_hash`.
    ///
    /// Returns the daemon's serialized state as a `Buffer`, or
    /// `null` when the daemon is stateless (no `snapshot` method,
    /// or its `snapshot` returned null). The wire format is the
    /// core's `StateSnapshot::to_bytes` encoding — opaque to JS
    /// callers, but round-trippable via
    /// [`Self::spawn_from_snapshot`].
    ///
    /// Calls into `MeshDaemon::snapshot` on the bridge, which in
    /// turn fires the JS `snapshot` TSFN stored at spawn time.
    /// Same TSFN-blocking pattern as `deliver`.
    #[napi]
    pub async fn snapshot(&self, origin_hash: u32) -> Result<Option<Buffer>> {
        let snap = self
            .inner
            .snapshot(origin_hash)
            .await
            .map_err(|e| daemon_err(e.to_string()))?;
        Ok(snap.map(|s| Buffer::from(s.to_bytes())))
    }

    /// Spawn a daemon from a previously-taken `snapshot_bytes`
    /// payload. The daemon instance is built from the
    /// caller-supplied `process` / `snapshot` / `restore` functions
    /// (same shape as [`Self::spawn`]); its state is seeded from
    /// the snapshot via the `restore` TSFN.
    ///
    /// `snapshot_bytes` must be the exact `Buffer` returned by a
    /// prior call to [`Self::snapshot`]; the core validates the
    /// wire magic/version and rejects mismatched bytes as
    /// `daemon: snapshot decode failed`.
    ///
    /// Identity check: the snapshot's `entity_id` must match the
    /// caller's `identity`; mismatch surfaces as
    /// `daemon: snapshot identity mismatch`.
    ///
    /// Same sync-with-PromiseRaw shape as `spawn` — `Function`
    /// values are `!Send`, so we build TSFNs synchronously before
    /// handing off to the async continuation.
    #[napi]
    pub fn spawn_from_snapshot<'env>(
        &'env self,
        env: &'env Env,
        kind: String,
        identity: &crate::identity::Identity,
        snapshot_bytes: Buffer,
        process: Function<'_, CausalEventJs, Vec<Buffer>>,
        snapshot: Option<Function<'_, (), Option<Buffer>>>,
        restore: Option<Function<'_, Buffer, napi::threadsafe_function::UnknownReturnValue>>,
        config: Option<DaemonHostConfigJs>,
    ) -> Result<napi::bindgen_prelude::PromiseRaw<'env, DaemonHandle>> {
        if !self.factories.contains_key(&kind) {
            return Err(daemon_err(format!(
                "no factory registered for kind '{kind}'"
            )));
        }

        // Decode the snapshot synchronously — cheap, and it lets
        // us surface a clean error before spinning up any TSFNs.
        // `from_bytes` returns `Option` (no error detail); the
        // core would reject mis-framed bytes anyway, but failing
        // fast here saves allocating the TSFNs and the bridge.
        let snapshot_decoded = StateSnapshot::from_bytes(snapshot_bytes.as_ref())
            .ok_or_else(|| daemon_err("snapshot decode failed"))?;

        let process_tsfn: ProcessTsfn = process.build_threadsafe_function().build()?;
        let snapshot_tsfn: Option<SnapshotTsfn> = match snapshot {
            Some(f) => Some(f.build_threadsafe_function().build()?),
            None => None,
        };
        let restore_tsfn: Option<RestoreTsfn> = match restore {
            Some(f) => Some(f.build_threadsafe_function().build()?),
            None => None,
        };

        let sdk_identity = identity.to_sdk_identity();
        let sdk_config = config.map(DaemonHostConfig::from).unwrap_or_default();
        let runtime = self.inner.clone();
        let kind_label = kind.clone();

        let bridge = Box::new(EventDispatchBridge {
            name: kind,
            process: process_tsfn,
            snapshot: snapshot_tsfn,
            restore: restore_tsfn,
        });

        // Same NoopBridge fallback for migration-target reconstruction
        // as `spawn` — see comment there for why.
        let kind_factory =
            move || -> Box<dyn MeshDaemon> { Box::new(NoopBridge::new(kind_label.clone())) };

        env.spawn_future(async move {
            runtime
                .spawn_from_snapshot_with_daemon(
                    sdk_identity,
                    snapshot_decoded,
                    sdk_config,
                    bridge,
                    kind_factory,
                )
                .await
                .map(DaemonHandle::from_sdk)
                .map_err(|e| daemon_err(e.to_string()))
        })
    }

    /// Deliver a single causal event to the daemon identified by
    /// `origin_hash`. Invokes the daemon's JS `process(event)`
    /// callback via the `ThreadsafeFunction` stored at `spawn`
    /// time, waits for the `Buffer[]` return, and surfaces each
    /// output back to JS as a `Buffer`.
    ///
    /// Direct ingress — Stage 1 convenience. Mesh-dispatched
    /// delivery (inbound via the causal subprotocol) lands in a
    /// later stage, at which point this method becomes test
    /// sugar rather than the primary entry point.
    #[napi]
    pub async fn deliver(&self, origin_hash: u32, event: CausalEventJs) -> Result<Vec<Buffer>> {
        use bytes::Bytes as BytesType;
        use net::adapter::net::state::causal::CausalLink;

        let (_sign, sequence, _lossless) = event.sequence.get_u64();
        let core_event = CausalEvent {
            link: CausalLink {
                origin_hash: event.origin_hash,
                horizon_encoded: 0,
                sequence,
                parent_hash: 0,
            },
            payload: BytesType::copy_from_slice(&event.payload),
            received_at: 0,
        };

        // SDK's `deliver` routes through `DaemonRegistry::deliver`
        // → `DaemonHost::deliver` → `MeshDaemon::process` on the
        // bridge, which is where the TSFN round-trip to JS
        // happens. The outputs come back as `Vec<Bytes>` wrapped
        // in causal events; we discard the chain wrapping and
        // return just the payload buffers.
        let outputs = self
            .inner
            .deliver(origin_hash, &core_event)
            .map_err(|e| daemon_err(e.to_string()))?;

        Ok(outputs
            .into_iter()
            .map(|ev| Buffer::from(ev.payload.as_ref()))
            .collect())
    }
}

// =========================================================================
// CausalEvent POJO — marshalled across NAPI into the JS daemon's `process`.
// =========================================================================

/// The causal event handed to a daemon's `process(event)` method.
///
/// Field shape matches
/// [`net::adapter::net::state::causal::CausalEvent`] with the
/// 64-bit `sequence` exposed as `BigInt` so JS doesn't silently
/// truncate.
#[napi(object)]
pub struct CausalEventJs {
    /// 32-bit hash of the emitting entity.
    pub origin_hash: u32,
    /// Sequence number in the emitter's causal chain.
    pub sequence: BigInt,
    /// Opaque payload bytes — identical to `event.payload` on the
    /// Rust side.
    pub payload: Buffer,
}

impl From<&CausalEvent> for CausalEventJs {
    fn from(event: &CausalEvent) -> Self {
        Self {
            origin_hash: event.link.origin_hash,
            sequence: BigInt::from(event.link.sequence),
            payload: Buffer::from(event.payload.as_ref()),
        }
    }
}

// =========================================================================
// DaemonHostConfig POJO — maps to core's struct
// =========================================================================

/// Host configuration for a daemon. Omitted fields fall back to
/// the core defaults (`auto_snapshot_interval: 0`,
/// `max_log_entries: 10_000`).
#[napi(object)]
pub struct DaemonHostConfigJs {
    /// Auto-snapshot cadence in events processed. `0` or absent =
    /// manual snapshots only.
    pub auto_snapshot_interval: Option<BigInt>,
    /// Maximum events to buffer before forcing a snapshot.
    pub max_log_entries: Option<u32>,
}

impl From<DaemonHostConfigJs> for DaemonHostConfig {
    fn from(js: DaemonHostConfigJs) -> Self {
        let mut cfg = DaemonHostConfig::default();
        if let Some(interval) = js.auto_snapshot_interval {
            let (_sign, as_u64, _lossless) = interval.get_u64();
            cfg.auto_snapshot_interval = as_u64;
        }
        if let Some(max) = js.max_log_entries {
            cfg.max_log_entries = max;
        }
        cfg
    }
}

// =========================================================================
// DaemonHandle — thin NAPI class over the SDK handle
// =========================================================================

/// Handle returned by [`DaemonRuntime::spawn`]. Identifies a
/// specific daemon by its `origin_hash`; cloning the JS object
/// shares the same underlying daemon. Dropping the handle does
/// **not** stop the daemon — callers must explicitly
/// [`DaemonRuntime::stop`] the origin.
#[napi]
pub struct DaemonHandle {
    origin_hash: u32,
    entity_id: [u8; 32],
    inner: SdkDaemonHandle,
}

impl DaemonHandle {
    fn from_sdk(handle: SdkDaemonHandle) -> Self {
        let origin_hash = handle.origin_hash;
        let entity_id = *handle.entity_id.as_bytes();
        Self {
            origin_hash,
            entity_id,
            inner: handle,
        }
    }
}

#[napi]
impl DaemonHandle {
    /// 32-bit hash of the daemon's identity — the key used by the
    /// registry, factory registry, and migration dispatcher.
    #[napi(getter)]
    pub fn origin_hash(&self) -> u32 {
        self.origin_hash
    }

    /// Full 32-byte `EntityId` (ed25519 public key) of the
    /// daemon's identity. Returned as a `Buffer` to match the
    /// convention used by `Identity.entityId`.
    #[napi(getter)]
    pub fn entity_id(&self) -> Buffer {
        Buffer::from(self.entity_id.to_vec())
    }

    /// Current runtime statistics for the daemon — event counters
    /// and snapshot count. Reads a live atomic snapshot from the
    /// registry; no TSFN round-trip, so the call is cheap enough
    /// to poll.
    ///
    /// Rejects with `daemon: not found` if the daemon has been
    /// stopped (or never successfully registered).
    #[napi]
    pub fn stats(&self) -> Result<DaemonStatsJs> {
        let stats = self
            .inner
            .stats()
            .map_err(|e| daemon_err(e.to_string()))?;
        Ok(DaemonStatsJs {
            events_processed: BigInt::from(stats.events_processed),
            events_emitted: BigInt::from(stats.events_emitted),
            errors: BigInt::from(stats.errors),
            snapshots_taken: BigInt::from(stats.snapshots_taken),
        })
    }
}

// =========================================================================
// DaemonStats POJO — mirrors the core struct, u64 fields as BigInt.
// =========================================================================

/// Runtime statistics for a single daemon.
///
/// All counters are monotonically increasing for the daemon's
/// lifetime and reset to zero when the daemon is stopped + respawned
/// (including via `spawnFromSnapshot`, because the core's registry
/// replaces the host). Field shape mirrors
/// [`net::adapter::net::compute::DaemonStats`] with `u64` → `BigInt`
/// so JS doesn't silently lose precision past 2^53.
#[napi(object)]
pub struct DaemonStatsJs {
    /// Total events processed since spawn.
    pub events_processed: BigInt,
    /// Total output events emitted since spawn.
    pub events_emitted: BigInt,
    /// Total processing errors surfaced from `process`.
    pub errors: BigInt,
    /// Number of snapshots taken (manual + auto combined).
    pub snapshots_taken: BigInt,
}

// =========================================================================
// EventDispatchBridge — real daemon bridge holding method TSFNs.
// =========================================================================

/// Daemon bridge built at `spawn` time from three TSFNs extracted
/// by the TS wrapper from the user's factory return value.
///
/// **Sub-step 2b** (this file): the TSFNs are stored but not yet
/// invoked — `process` returns an empty output, `snapshot` /
/// `restore` are ignored. The storage + lifecycle paths work
/// end-to-end; sub-step 3 will wire the method implementations to
/// call the TSFNs and marshal arguments / return values.
struct EventDispatchBridge {
    name: String,
    process: ProcessTsfn,
    snapshot: Option<SnapshotTsfn>,
    restore: Option<RestoreTsfn>,
}

impl MeshDaemon for EventDispatchBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn requirements(&self) -> CapabilityFilter {
        CapabilityFilter::default()
    }

    /// Synchronously dispatch the event to the JS `process`
    /// callback and wait for the returned `Buffer[]`.
    ///
    /// The TSFN call itself is asynchronous w.r.t. the Node
    /// event loop — napi-rs queues the JS callback and fires it
    /// when Node gets around to it. We use
    /// `call_with_return_value` so napi-rs invokes our Rust
    /// callback with the parsed `Result<Vec<Buffer>>` once JS
    /// returns; that callback sends through an `std::sync::mpsc`
    /// channel which this tokio task blocks on.
    ///
    /// Deadlock avoidance: `process` runs on a tokio worker, not
    /// the Node main thread. Blocking on the channel is safe
    /// because the Node event loop — where the JS `process`
    /// function actually executes — is a separate thread. The
    /// caller that reached us (`DaemonRuntime::deliver`) returns
    /// a `Promise` to JS, so the JS caller's `await` yields the
    /// event loop long enough for the TSFN callback to fire.
    fn process(&mut self, event: &CausalEvent) -> std::result::Result<Vec<Bytes>, CoreDaemonError> {
        let event_js = CausalEventJs::from(event);
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<Vec<Buffer>>>(1);

        let status = self.process.call_with_return_value(
            event_js,
            napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
            move |ret: Result<Vec<Buffer>>, _env| {
                // `send` only fails if the receiver was dropped —
                // that means the Rust caller gave up before we
                // got a chance to reply. Nothing productive to
                // do here; swallow to avoid napi-rs escalating
                // to a fatal process exit.
                let _ = tx.send(ret);
                Ok(())
            },
        );
        if status != napi::Status::Ok {
            return Err(CoreDaemonError::ProcessFailed(format!(
                "threadsafe_function enqueue failed: {status:?}"
            )));
        }

        let result = rx.recv().map_err(|e| {
            CoreDaemonError::ProcessFailed(format!("JS `process` callback did not respond: {e}"))
        })?;

        match result {
            Ok(buffers) => {
                // Convert `Vec<Buffer>` → `Vec<Bytes>` without
                // extra copies. `Buffer` derefs to `&[u8]`, and
                // `Bytes::copy_from_slice` allocates an
                // Arc-tracked payload. The daemon's contract
                // says outputs may be held indefinitely by the
                // causal chain, so we must not alias the
                // `Buffer`'s V8-managed memory — the copy is
                // load-bearing.
                Ok(buffers
                    .into_iter()
                    .map(|b| Bytes::copy_from_slice(b.as_ref()))
                    .collect())
            }
            Err(e) => Err(CoreDaemonError::ProcessFailed(format!(
                "JS `process` threw: {e}"
            ))),
        }
    }

    /// Synchronously ask the JS `snapshot()` callback for the
    /// daemon's current state. Same channel-and-block pattern as
    /// [`Self::process`].
    ///
    /// `MeshDaemon::snapshot` returns `Option<Bytes>` (no `Result`),
    /// so there's no way to surface an error from here. If the JS
    /// `snapshot` throws or the TSFN enqueue fails, we log a
    /// warning via `eprintln!` and return `None` — the core
    /// interprets that as "stateless at this moment" rather than
    /// "snapshot attempted but failed." Callers who need strict
    /// error propagation should rely on [`Self::restore`] round-
    /// trips to catch corrupted state.
    fn snapshot(&self) -> Option<Bytes> {
        let tsfn = self.snapshot.as_ref()?;
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<Option<Buffer>>>(1);
        let status = tsfn.call_with_return_value(
            (),
            napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
            move |ret: Result<Option<Buffer>>, _env| {
                let _ = tx.send(ret);
                Ok(())
            },
        );
        if status != napi::Status::Ok {
            eprintln!("EventDispatchBridge::snapshot enqueue failed: {status:?}");
            return None;
        }
        match rx.recv() {
            Ok(Ok(Some(buf))) => Some(Bytes::copy_from_slice(buf.as_ref())),
            Ok(Ok(None)) => None,
            Ok(Err(e)) => {
                eprintln!("EventDispatchBridge::snapshot JS callback threw: {e}");
                None
            }
            Err(e) => {
                eprintln!("EventDispatchBridge::snapshot channel recv failed: {e}");
                None
            }
        }
    }

    /// Synchronously invoke the JS `restore(state)` callback.
    /// Errors propagate back as `CoreDaemonError::RestoreFailed` so
    /// the core's `DaemonHost::from_snapshot` can reject a bad
    /// snapshot before any events are processed.
    ///
    /// If no `restore` TSFN is installed (user's daemon didn't
    /// provide one), the state is silently ignored — matches the
    /// default `MeshDaemon::restore` behaviour in core.
    fn restore(&mut self, state: Bytes) -> std::result::Result<(), CoreDaemonError> {
        let tsfn = match self.restore.as_ref() {
            Some(t) => t,
            None => return Ok(()),
        };
        let buf = Buffer::from(state.as_ref());
        let (tx, rx) = std::sync::mpsc::sync_channel::<
            Result<napi::threadsafe_function::UnknownReturnValue>,
        >(1);
        let status = tsfn.call_with_return_value(
            buf,
            napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
            move |ret, _env| {
                let _ = tx.send(ret);
                Ok(())
            },
        );
        if status != napi::Status::Ok {
            return Err(CoreDaemonError::RestoreFailed(format!(
                "threadsafe_function enqueue failed: {status:?}"
            )));
        }
        let result = rx.recv().map_err(|e| {
            CoreDaemonError::RestoreFailed(format!("JS `restore` callback did not respond: {e}"))
        })?;
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(CoreDaemonError::RestoreFailed(format!(
                "JS `restore` threw: {e}"
            ))),
        }
    }
}

// =========================================================================
// NoopBridge — fallback `MeshDaemon` for migration-target reconstruction.
// =========================================================================

/// Placeholder bridge used as the core factory-registry's
/// "kind-factory" for NAPI-spawned daemons. The core registry
/// requires a `Fn() -> Box<dyn MeshDaemon>` factory so a migration
/// target can reconstruct the daemon from `origin_hash` alone;
/// faithfully implementing that for NAPI requires re-running the
/// TS factory on the Rust side (sub-step 4+ work). Until then
/// `NoopBridge` stands in — migrations into a TS node's daemon
/// will fail downstream at the first event delivery, but the
/// lifecycle / registry machinery stays consistent.
struct NoopBridge {
    name: String,
}

impl NoopBridge {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl MeshDaemon for NoopBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn requirements(&self) -> CapabilityFilter {
        CapabilityFilter::default()
    }

    fn process(
        &mut self,
        _event: &CausalEvent,
    ) -> std::result::Result<Vec<Bytes>, CoreDaemonError> {
        Ok(Vec::new())
    }
}
