// `#[napi]` exports functions to JS but leaves them "unused" from
// Rust's POV, so clippy's dead-code analysis doesn't apply to this
// module. Suppress at file scope.
#![allow(dead_code)]

//! NAPI surface for the compute runtime — `MeshDaemon` + migration.
//!
//! Stage 3 of [`SDK_COMPUTE_SURFACE_PLAN.md`]. This sub-step (1 of 5)
//! lands only the **skeleton + lifecycle**: a TS caller can build a
//! `DaemonRuntime` against an existing `NetMesh`, register a factory
//! (the JS function is held as a `ThreadsafeFunction` but not yet
//! invoked), start the runtime, and shut it down. Event delivery,
//! migration, and snapshot/restore land in subsequent sub-steps.
//!
//! # Error prefix
//!
//! Every `Error` produced here is prefixed with `daemon:` so the TS
//! side's `classifyError` can route to a dedicated `DaemonError`
//! class. Mirrors the `identity:` / `cortex:` / `token:` convention
//! used by other modules.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use dashmap::DashMap;
use std::sync::Arc;

use net_sdk::compute::DaemonRuntime as SdkDaemonRuntime;
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
        self.inner.start().await.map_err(|e| daemon_err(e.to_string()))
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
}
