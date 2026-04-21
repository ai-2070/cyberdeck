//! PyO3 surface for the compute runtime — `MeshDaemon` + migration.
//!
//! Stage 5 of `SDK_COMPUTE_SURFACE_PLAN.md`.
//!
//! **Sub-step 1** (this file): lifecycle skeleton. A Python caller
//! can build a `DaemonRuntime` against an existing `NetMesh`,
//! register a factory (stored but not yet invoked), start the
//! runtime, and shut it down. Event delivery, migration, and
//! snapshot/restore land in subsequent sub-steps.
//!
//! # Dispatcher pattern (future sub-steps)
//!
//! Unlike the NAPI side — where TSFN callbacks marshal to the Node
//! main thread via `call_with_return_value` + mpsc — PyO3 lets us
//! call into Python from any tokio worker by acquiring the GIL
//! with `Python::with_gil`. That yields a simpler bridge: hold
//! `Py<PyAny>` for each callback; every `process`/`snapshot`/
//! `restore` invocation does `Python::with_gil(|py| ...)` inline.
//! No cross-thread channel dance required.
//!
//! The GIL-acquisition latency *does* carry the same caveat as the
//! NAPI layer: Python-implemented daemons don't inherit the
//! microsecond-latency contract of `MeshDaemon::process`. Hot
//! loops should stay in Rust.
//!
//! # Error prefix
//!
//! Every `PyErr` produced here uses the `daemon:` prefix (same
//! convention as `identity:` / `cortex:` / `token:`). Migration-
//! layer errors use the `daemon: migration: <kind>[: detail]`
//! shape so the Python side can dispatch on kind without parsing
//! free-form messages.

use std::sync::Arc;

use dashmap::DashMap;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use net_sdk::compute::DaemonRuntime as SdkDaemonRuntime;
use net_sdk::mesh::Mesh as SdkMesh;
use tokio::runtime::Runtime;

// =========================================================================
// Error prefix — stable string convention
// =========================================================================

const ERR_DAEMON_PREFIX: &str = "daemon:";

/// Build a `PyRuntimeError` with the `daemon:` prefix. Matches the
/// identity / cortex / token conventions so Python-side
/// classifiers can route on the prefix cheaply.
pub(crate) fn daemon_err(msg: impl Into<String>) -> PyErr {
    PyRuntimeError::new_err(format!("{} {}", ERR_DAEMON_PREFIX, msg.into()))
}

// =========================================================================
// PyDaemonRuntime — lifecycle skeleton
// =========================================================================

/// Python surface for the compute runtime. One instance per
/// `NetMesh`; share by `Arc` internally.
///
/// Construct via `DaemonRuntime(mesh)`. The runtime shares the
/// given `NetMesh`'s live `MeshNode` — no second socket. Shutting
/// down the runtime does NOT shut down the mesh; callers own the
/// `NetMesh` lifecycle separately.
#[pyclass(name = "DaemonRuntime", module = "net._net")]
pub struct PyDaemonRuntime {
    /// SDK-level runtime. Owns the daemon registry, factory
    /// registry, migration orchestrator, and lifecycle state.
    /// The PyO3 layer is a thin envelope.
    inner: Arc<SdkDaemonRuntime>,
    /// Shared tokio runtime (cloned from the source `NetMesh`).
    /// Every async SDK call goes through `runtime.block_on` with
    /// the GIL released via `py.detach`.
    runtime: Arc<Runtime>,
    /// Tracked set of registered kinds. Used by `spawn` /
    /// `spawn_from_snapshot` in later sub-steps to produce a
    /// friendlier error than the SDK's `FactoryNotFound` on an
    /// unregistered kind. The authoritative factory storage
    /// lives on the SDK side once sub-step 2 wires it up.
    factories: Arc<DashMap<String, ()>>,
}

#[pymethods]
impl PyDaemonRuntime {
    /// Build a compute runtime against an existing `NetMesh`.
    ///
    /// Args:
    ///     mesh: A live `NetMesh` instance; its underlying
    ///         `MeshNode` is shared (not copied) into the runtime.
    ///
    /// Raises:
    ///     RuntimeError: if the mesh has been shut down.
    #[new]
    fn new(mesh: &crate::mesh_bindings::NetMesh) -> PyResult<Self> {
        let node = mesh.node_arc_clone()?;
        let channel_configs = mesh.channel_configs_arc();
        let runtime = mesh.runtime_arc();
        // Build an SDK-level `Mesh` sharing the live `MeshNode`.
        // Identity is `None` — the Python binding manages identity
        // separately (via the `Identity` class); the daemon runtime
        // only needs the mesh for peer lookup + subprotocol handler
        // install.
        let sdk_mesh = SdkMesh::from_node_arc(node, channel_configs, None);
        let sdk_rt = SdkDaemonRuntime::new(Arc::new(sdk_mesh));
        Ok(PyDaemonRuntime {
            inner: Arc::new(sdk_rt),
            runtime,
            factories: Arc::new(DashMap::new()),
        })
    }

    /// Transition to `Ready`. Installs the migration subprotocol
    /// handler on the underlying mesh. Idempotent — a second call
    /// on a runtime that's already `Ready` is a no-op; on a
    /// shut-down runtime raises `daemon: shutting down`.
    fn start(&self, py: Python<'_>) -> PyResult<()> {
        let runtime = self.runtime.clone();
        let inner = self.inner.clone();
        py.detach(|| {
            runtime
                .block_on(async move { inner.start().await })
                .map_err(|e| daemon_err(e.to_string()))
        })
    }

    /// Tear down the runtime. Drains daemons, clears factory
    /// registrations, uninstalls the migration handler. The
    /// underlying `NetMesh` is untouched.
    fn shutdown(&self, py: Python<'_>) -> PyResult<()> {
        let runtime = self.runtime.clone();
        let inner = self.inner.clone();
        let factories = self.factories.clone();
        py.detach(move || {
            runtime
                .block_on(async move { inner.shutdown().await })
                .map_err(|e| daemon_err(e.to_string()))?;
            factories.clear();
            Ok(())
        })
    }

    /// `True` iff the runtime has transitioned to `Ready` and
    /// has not yet begun shutting down.
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    /// Number of daemons currently registered with the runtime.
    fn daemon_count(&self) -> u32 {
        // Cast: SDK returns `usize`. Daemon counts are realistically
        // << 2^32; Python has no narrow integer type.
        self.inner.daemon_count() as u32
    }

    /// Register a factory closure under `kind`. The factory is a
    /// Python callable returning a `MeshDaemon`-shaped object
    /// (with `process` / `snapshot` / `restore` methods).
    ///
    /// **Sub-step 1** (this file): the factory is stored but NOT
    /// invoked. Sub-step 2 adds daemon construction on spawn.
    /// Second registration of the same `kind` raises
    /// `daemon: factory for kind '<kind>' is already registered`.
    fn register_factory(&self, kind: String, _factory: Py<PyAny>) -> PyResult<()> {
        use dashmap::mapref::entry::Entry;
        match self.factories.entry(kind.clone()) {
            Entry::Occupied(_) => Err(daemon_err(format!(
                "factory for kind '{kind}' is already registered"
            ))),
            Entry::Vacant(slot) => {
                slot.insert(());
                // Sub-step 2 will mirror into
                // `self.inner.register_factory(&kind, ...)` with a
                // GIL-acquiring closure that invokes the Python
                // factory. For now we just record the kind so the
                // test suite can assert registration uniqueness.
                Ok(())
            }
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "DaemonRuntime(ready={}, daemons={})",
            self.inner.is_ready(),
            self.inner.daemon_count()
        )
    }
}
