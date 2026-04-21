//! PyO3 surface for the compute runtime — `MeshDaemon` + migration.
//!
//! Stage 5 of `SDK_COMPUTE_SURFACE_PLAN.md`.
//!
//! # Dispatcher pattern
//!
//! Unlike the NAPI side — where TSFN callbacks marshal to the Node
//! main thread via `call_with_return_value` + mpsc — PyO3 lets us
//! call into Python from any tokio worker by acquiring the GIL
//! with `Python::attach`. That yields a simpler bridge: hold
//! `Py<PyAny>` for each callback; every `process` / `snapshot` /
//! `restore` invocation does `Python::attach(|py| ...)` inline.
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
//! convention as `identity:` / `cortex:` / `token:`).

use std::sync::Arc;

use bytes::Bytes;
use dashmap::DashMap;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyTuple};

use net::adapter::net::behavior::capability::CapabilityFilter;
use net::adapter::net::compute::{DaemonError as CoreDaemonError, DaemonHostConfig, MeshDaemon};
use net::adapter::net::state::causal::{CausalEvent, CausalLink};
use net_sdk::compute::{
    DaemonHandle as SdkDaemonHandle, DaemonRuntime as SdkDaemonRuntime, StateSnapshot,
};
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
// PyCausalEvent — the value delivered to a daemon's `process`
// =========================================================================

/// A causal event handed to a daemon's `process(event)` method.
///
/// Field shape matches `net::adapter::net::state::causal::CausalEvent`.
/// The 64-bit `sequence` is exposed as a Python `int` — Python
/// integers are unbounded so no precision concerns.
#[pyclass(name = "CausalEvent", module = "net._net")]
#[derive(Clone)]
pub struct PyCausalEvent {
    /// 32-bit hash of the emitting entity.
    #[pyo3(get)]
    pub origin_hash: u32,
    /// Sequence number in the emitter's causal chain.
    #[pyo3(get)]
    pub sequence: u64,
    /// Opaque payload bytes — identical to `event.payload` on the
    /// Rust side.
    pub payload: Vec<u8>,
}

#[pymethods]
impl PyCausalEvent {
    /// Construct manually — mainly used by tests that call
    /// `DaemonRuntime.deliver` directly.
    #[new]
    #[pyo3(signature = (origin_hash, sequence, payload))]
    fn new(origin_hash: u32, sequence: u64, payload: Vec<u8>) -> Self {
        Self {
            origin_hash,
            sequence,
            payload,
        }
    }

    /// Opaque payload bytes.
    #[getter]
    fn payload<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.payload)
    }

    fn __repr__(&self) -> String {
        format!(
            "CausalEvent(origin_hash={:#x}, sequence={}, payload_len={})",
            self.origin_hash,
            self.sequence,
            self.payload.len()
        )
    }
}

impl PyCausalEvent {
    fn from_core(event: &CausalEvent) -> Self {
        Self {
            origin_hash: event.link.origin_hash,
            sequence: event.link.sequence,
            payload: event.payload.as_ref().to_vec(),
        }
    }
}

// =========================================================================
// DaemonHostConfig — Python dict → core config
// =========================================================================

/// Parse an optional Python dict into a core `DaemonHostConfig`.
/// Keys: `auto_snapshot_interval` (int), `max_log_entries` (int).
/// Unknown keys are ignored; missing keys take core defaults.
fn daemon_host_config_from_dict(config: Option<&Bound<'_, PyDict>>) -> PyResult<DaemonHostConfig> {
    let mut cfg = DaemonHostConfig::default();
    let Some(d) = config else {
        return Ok(cfg);
    };
    if let Some(v) = d.get_item("auto_snapshot_interval")? {
        cfg.auto_snapshot_interval = v
            .extract::<u64>()
            .map_err(|e| daemon_err(format!("auto_snapshot_interval must be int: {e}")))?;
    }
    if let Some(v) = d.get_item("max_log_entries")? {
        cfg.max_log_entries = v
            .extract::<u32>()
            .map_err(|e| daemon_err(format!("max_log_entries must be int: {e}")))?;
    }
    Ok(cfg)
}

// =========================================================================
// PyDaemonHandle — returned by `spawn` / `spawn_from_snapshot`
// =========================================================================

/// Handle to a running daemon. Identifies a specific daemon by
/// its `origin_hash`; cloning the Python object shares the same
/// underlying daemon. Dropping the handle does NOT stop the
/// daemon — callers must call `stop(handle.origin_hash)`.
#[pyclass(name = "DaemonHandle", module = "net._net")]
pub struct PyDaemonHandle {
    origin_hash: u32,
    entity_id: [u8; 32],
    #[allow(dead_code)]
    inner: SdkDaemonHandle,
}

impl PyDaemonHandle {
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

#[pymethods]
impl PyDaemonHandle {
    /// 32-bit hash of the daemon's identity.
    #[getter]
    fn origin_hash(&self) -> u32 {
        self.origin_hash
    }

    /// Full 32-byte `EntityId` (ed25519 public key). Returned as
    /// `bytes` to match the convention used by `Identity.entity_id`.
    #[getter]
    fn entity_id<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.entity_id)
    }

    fn __repr__(&self) -> String {
        format!("DaemonHandle(origin_hash={:#x})", self.origin_hash)
    }
}

// =========================================================================
// PyDaemonRuntime — main surface
// =========================================================================

/// Python surface for the compute runtime. One instance per
/// `NetMesh`. Construct via `DaemonRuntime(mesh)`.
#[pyclass(name = "DaemonRuntime", module = "net._net")]
pub struct PyDaemonRuntime {
    inner: Arc<SdkDaemonRuntime>,
    runtime: Arc<Runtime>,
    /// Registered factory callables, keyed by `kind`. Values are
    /// `Py<PyAny>` holding the user's Python factory callable.
    /// The DashMap is the single source of truth; the SDK side
    /// mirrors a closure that reaches back into this map to call
    /// the Python factory on demand (on spawn, on migration-
    /// target reconstruction).
    factories: Arc<DashMap<String, Py<PyAny>>>,
}

#[pymethods]
impl PyDaemonRuntime {
    /// Build a compute runtime against an existing `NetMesh`.
    #[new]
    fn new(mesh: &crate::mesh_bindings::NetMesh) -> PyResult<Self> {
        let node = mesh.node_arc_clone()?;
        let channel_configs = mesh.channel_configs_arc();
        let runtime = mesh.runtime_arc();
        let sdk_mesh = SdkMesh::from_node_arc(node, channel_configs, None);
        let sdk_rt = SdkDaemonRuntime::new(Arc::new(sdk_mesh));
        Ok(PyDaemonRuntime {
            inner: Arc::new(sdk_rt),
            runtime,
            factories: Arc::new(DashMap::new()),
        })
    }

    /// Transition to `Ready`. Installs the migration subprotocol
    /// handler. Idempotent — a second `Ready` call is a no-op; a
    /// call on a `ShuttingDown` runtime raises.
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

    /// `True` iff the runtime is `Ready`.
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    /// Number of daemons currently registered.
    fn daemon_count(&self) -> u32 {
        self.inner.daemon_count() as u32
    }

    /// Register a factory callable under `kind`. The callable
    /// must return a `MeshDaemon`-shaped object with a `process`
    /// method and optional `snapshot` / `restore` methods.
    ///
    /// Second registration of the same `kind` raises
    /// `daemon: factory for kind '<kind>' is already registered`.
    fn register_factory(&self, kind: String, factory: Py<PyAny>) -> PyResult<()> {
        use dashmap::mapref::entry::Entry;
        match self.factories.entry(kind.clone()) {
            Entry::Occupied(_) => {
                return Err(daemon_err(format!(
                    "factory for kind '{kind}' is already registered"
                )));
            }
            Entry::Vacant(slot) => {
                slot.insert(factory);
            }
        }

        // Mirror into the SDK factory map. The SDK closure reaches
        // back into `self.factories` every time the core registry
        // needs a fresh daemon (spawn + migration-target
        // reconstruction). We clone `factories` (the Arc) into the
        // closure so the closure's Fn bound is satisfied without
        // borrowing `self`.
        //
        // On the failure path below (NoopBridge) we log and
        // fall back — same safety argument as the NAPI layer.
        let factories_for_closure = self.factories.clone();
        let kind_for_closure = kind.clone();
        if let Err(e) = self.inner.register_factory(&kind, move || {
            build_bridge_from_factory(&factories_for_closure, &kind_for_closure)
        }) {
            self.factories.remove(&kind);
            return Err(daemon_err(e.to_string()));
        }
        Ok(())
    }

    /// Spawn a daemon of `kind` under the given identity.
    ///
    /// Invokes the registered factory callable (in the current
    /// thread's GIL context) to build the per-instance daemon
    /// object, then extracts its `process` / `snapshot` /
    /// `restore` methods and wraps them in a `PyDaemonBridge`
    /// that the core registry drives.
    ///
    /// `config` accepts an optional dict with keys
    /// `auto_snapshot_interval` (int) and `max_log_entries`
    /// (int); missing keys take runtime defaults.
    #[pyo3(signature = (kind, identity, config=None))]
    fn spawn(
        &self,
        py: Python<'_>,
        kind: String,
        identity: &crate::identity::Identity,
        config: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyDaemonHandle> {
        if !self.factories.contains_key(&kind) {
            return Err(daemon_err(format!(
                "no factory registered for kind '{kind}'"
            )));
        }
        let cfg = daemon_host_config_from_dict(config)?;
        let sdk_identity = identity.to_sdk_identity();
        // Build the bridge by invoking the Python factory now.
        // This makes `spawn` fail fast if the factory throws
        // rather than surfacing the failure later during event
        // dispatch.
        let bridge = build_bridge_inline(py, &self.factories, &kind)?;

        // Kind-factory closure for migration-target
        // reconstruction — fresh bridge per invocation via the
        // same `factories` map.
        let factories_for_closure = self.factories.clone();
        let kind_for_closure = kind.clone();
        let kind_factory = move || -> Box<dyn MeshDaemon> {
            build_bridge_from_factory(&factories_for_closure, &kind_for_closure)
        };

        let runtime = self.runtime.clone();
        let inner = self.inner.clone();
        let handle = py.detach(move || {
            runtime
                .block_on(async move {
                    inner
                        .spawn_with_daemon(sdk_identity, cfg, bridge, kind_factory)
                        .await
                })
                .map_err(|e| daemon_err(e.to_string()))
        })?;
        Ok(PyDaemonHandle::from_sdk(handle))
    }

    /// Stop a daemon, removing it from the runtime's registry.
    /// Idempotent during `ShuttingDown`; raises for other
    /// failures with the `daemon:` prefix.
    fn stop(&self, py: Python<'_>, origin_hash: u32) -> PyResult<()> {
        let runtime = self.runtime.clone();
        let inner = self.inner.clone();
        py.detach(move || {
            runtime
                .block_on(async move { inner.stop(origin_hash).await })
                .map_err(|e| daemon_err(e.to_string()))
        })
    }

    /// Take a snapshot of a running daemon by `origin_hash`.
    ///
    /// Returns the daemon's serialized state as `bytes`, or
    /// `None` when the daemon is stateless (no `snapshot`
    /// method, or its `snapshot` returned `None`). The wire
    /// format is the core's `StateSnapshot::to_bytes` encoding —
    /// opaque to Python callers, but round-trippable via
    /// `spawn_from_snapshot`.
    fn snapshot<'py>(
        &self,
        py: Python<'py>,
        origin_hash: u32,
    ) -> PyResult<Option<Bound<'py, PyBytes>>> {
        let runtime = self.runtime.clone();
        let inner = self.inner.clone();
        let snap = py.detach(move || {
            runtime
                .block_on(async move { inner.snapshot(origin_hash).await })
                .map_err(|e| daemon_err(e.to_string()))
        })?;
        Ok(snap.map(|s| PyBytes::new(py, &s.to_bytes())))
    }

    /// Spawn a daemon of `kind` from a previously-taken snapshot.
    /// Parallel to `spawn`, but the daemon's initial state is
    /// seeded from `snapshot_bytes` by calling its `restore`
    /// method before any events land.
    ///
    /// `snapshot_bytes` must be the exact `bytes` returned by a
    /// prior call to `snapshot`; mismatched or corrupted bytes
    /// surface as `daemon: snapshot decode failed`.
    ///
    /// Identity check: the snapshot's `entity_id` must match the
    /// caller's `identity` — mismatch raises `daemon: snapshot
    /// identity mismatch`.
    #[pyo3(signature = (kind, identity, snapshot_bytes, config=None))]
    fn spawn_from_snapshot(
        &self,
        py: Python<'_>,
        kind: String,
        identity: &crate::identity::Identity,
        snapshot_bytes: &[u8],
        config: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyDaemonHandle> {
        if !self.factories.contains_key(&kind) {
            return Err(daemon_err(format!(
                "no factory registered for kind '{kind}'"
            )));
        }
        // Decode the snapshot synchronously — cheap, and a clean
        // `daemon: snapshot decode failed` is friendlier than the
        // downstream SDK error.
        let snapshot_decoded = StateSnapshot::from_bytes(snapshot_bytes)
            .ok_or_else(|| daemon_err("snapshot decode failed"))?;

        let cfg = daemon_host_config_from_dict(config)?;
        let sdk_identity = identity.to_sdk_identity();
        let bridge = build_bridge_inline(py, &self.factories, &kind)?;

        let factories_for_closure = self.factories.clone();
        let kind_for_closure = kind.clone();
        let kind_factory = move || -> Box<dyn MeshDaemon> {
            build_bridge_from_factory(&factories_for_closure, &kind_for_closure)
        };

        let runtime = self.runtime.clone();
        let inner = self.inner.clone();
        let handle = py.detach(move || {
            runtime
                .block_on(async move {
                    inner
                        .spawn_from_snapshot_with_daemon(
                            sdk_identity,
                            snapshot_decoded,
                            cfg,
                            bridge,
                            kind_factory,
                        )
                        .await
                })
                .map_err(|e| daemon_err(e.to_string()))
        })?;
        Ok(PyDaemonHandle::from_sdk(handle))
    }

    /// Deliver one causal event to the daemon identified by
    /// `origin_hash`. Invokes the daemon's `process(event)`
    /// method (in this runtime's GIL context via the bridge)
    /// and returns the list of output `bytes` payloads.
    ///
    /// Direct ingress — Stage 1 convenience. Mesh-dispatched
    /// delivery lands in a later stage; this method stays as
    /// test sugar + a manual-trigger surface.
    fn deliver(
        &self,
        py: Python<'_>,
        origin_hash: u32,
        event: &PyCausalEvent,
    ) -> PyResult<Vec<Py<PyBytes>>> {
        let core_event = CausalEvent {
            link: CausalLink {
                origin_hash: event.origin_hash,
                horizon_encoded: 0,
                sequence: event.sequence,
                parent_hash: 0,
            },
            payload: Bytes::copy_from_slice(&event.payload),
            received_at: 0,
        };

        // Run the SDK deliver on a tokio worker. Keep the GIL
        // while doing so because `MeshDaemon::process` reaches
        // back into Python via `Python::attach` — if we held
        // it here it would deadlock. Detaching releases the
        // GIL; the worker acquires it via `attach` during
        // dispatch and releases it when process returns.
        let runtime = self.runtime.clone();
        let inner = self.inner.clone();
        let outputs = py.detach(move || {
            runtime.block_on(async move {
                inner
                    .deliver(origin_hash, &core_event)
                    .map_err(|e| daemon_err(e.to_string()))
            })
        })?;

        let out: Vec<Py<PyBytes>> = outputs
            .into_iter()
            .map(|ev| PyBytes::new(py, ev.payload.as_ref()).unbind())
            .collect();
        Ok(out)
    }

    fn __repr__(&self) -> String {
        format!(
            "DaemonRuntime(ready={}, daemons={})",
            self.inner.is_ready(),
            self.inner.daemon_count()
        )
    }
}

// =========================================================================
// PyDaemonBridge — MeshDaemon impl driven by Py<PyAny> callbacks
// =========================================================================

/// Daemon bridge wrapping three Python callables (`process`,
/// `snapshot?`, `restore?`) extracted from a factory invocation.
/// Every `MeshDaemon` method call does a single
/// `Python::attach` to reach the underlying Python callable.
struct PyDaemonBridge {
    name: String,
    process: Py<PyAny>,
    snapshot: Option<Py<PyAny>>,
    restore: Option<Py<PyAny>>,
}

impl MeshDaemon for PyDaemonBridge {
    fn name(&self) -> &str {
        &self.name
    }

    fn requirements(&self) -> CapabilityFilter {
        CapabilityFilter::default()
    }

    /// Dispatch the event to the Python `process` callable. The
    /// callable must return an iterable of `bytes`; anything else
    /// is surfaced as `CoreDaemonError::ProcessFailed`.
    fn process(
        &mut self,
        event: &CausalEvent,
    ) -> std::result::Result<Vec<Bytes>, CoreDaemonError> {
        let py_event = PyCausalEvent::from_core(event);
        Python::attach(|py| -> std::result::Result<Vec<Bytes>, CoreDaemonError> {
            let event_obj = Py::new(py, py_event).map_err(|e| {
                CoreDaemonError::ProcessFailed(format!("failed to wrap event: {e}"))
            })?;
            let args = PyTuple::new(py, [event_obj.into_any()]).map_err(|e| {
                CoreDaemonError::ProcessFailed(format!("failed to build args: {e}"))
            })?;
            let result = self.process.call1(py, args).map_err(|e| {
                CoreDaemonError::ProcessFailed(format!("process raised: {e}"))
            })?;
            let list: Bound<'_, PyAny> = result.into_bound(py);
            parse_output_list(&list)
        })
    }

    /// Ask the Python `snapshot()` callable for the daemon's
    /// current state. Returns `None` if no snapshot callable
    /// was registered, or if the callable returned `None`.
    fn snapshot(&self) -> Option<Bytes> {
        let snapshot = self.snapshot.as_ref()?;
        Python::attach(|py| -> Option<Bytes> {
            match snapshot.call0(py) {
                Ok(ret) => {
                    let ret_any = ret.into_bound(py);
                    if ret_any.is_none() {
                        None
                    } else {
                        match ret_any.extract::<Vec<u8>>() {
                            Ok(v) => Some(Bytes::from(v)),
                            Err(e) => {
                                eprintln!(
                                    "PyDaemonBridge::snapshot: return value is not bytes: {e}; treating as None"
                                );
                                None
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("PyDaemonBridge::snapshot: callable raised: {e}; treating as None");
                    None
                }
            }
        })
    }

    /// Invoke the Python `restore(state)` callable. Errors
    /// propagate as `CoreDaemonError::RestoreFailed`.
    fn restore(
        &mut self,
        state: Bytes,
    ) -> std::result::Result<(), CoreDaemonError> {
        let Some(restore) = self.restore.as_ref() else {
            return Ok(());
        };
        Python::attach(|py| -> std::result::Result<(), CoreDaemonError> {
            let state_bytes = PyBytes::new(py, state.as_ref());
            let args = PyTuple::new(py, [state_bytes.into_any()]).map_err(|e| {
                CoreDaemonError::RestoreFailed(format!("failed to build args: {e}"))
            })?;
            restore.call1(py, args).map_err(|e| {
                CoreDaemonError::RestoreFailed(format!("restore raised: {e}"))
            })?;
            Ok(())
        })
    }
}

/// Parse the Python return value of `process` into `Vec<Bytes>`.
/// Accepts any iterable whose elements convert to `bytes`.
fn parse_output_list(obj: &Bound<'_, PyAny>) -> std::result::Result<Vec<Bytes>, CoreDaemonError> {
    // Common case: list of bytes.
    if let Ok(list) = obj.downcast::<PyList>() {
        let mut out = Vec::with_capacity(list.len());
        for item in list.iter() {
            let v: Vec<u8> = item.extract().map_err(|e| {
                CoreDaemonError::ProcessFailed(format!(
                    "process output element is not bytes: {e}"
                ))
            })?;
            out.push(Bytes::from(v));
        }
        return Ok(out);
    }
    // Fallback: any iterable of bytes.
    match obj.try_iter() {
        Ok(iter) => {
            let mut out = Vec::new();
            for item in iter {
                let item = item.map_err(|e| {
                    CoreDaemonError::ProcessFailed(format!(
                        "iterating process output: {e}"
                    ))
                })?;
                let v: Vec<u8> = item.extract().map_err(|e| {
                    CoreDaemonError::ProcessFailed(format!(
                        "process output element is not bytes: {e}"
                    ))
                })?;
                out.push(Bytes::from(v));
            }
            Ok(out)
        }
        Err(e) => Err(CoreDaemonError::ProcessFailed(format!(
            "process must return a list/iterable of bytes; got {e}"
        ))),
    }
}

// =========================================================================
// Bridge construction helpers
// =========================================================================

/// Build a `PyDaemonBridge` by invoking the Python factory for
/// `kind` under the current GIL context. Used by `spawn` —
/// propagates factory exceptions directly to the caller.
fn build_bridge_inline(
    py: Python<'_>,
    factories: &DashMap<String, Py<PyAny>>,
    kind: &str,
) -> PyResult<Box<dyn MeshDaemon>> {
    let Some(entry) = factories.get(kind) else {
        return Err(daemon_err(format!(
            "no factory registered for kind '{kind}'"
        )));
    };
    let factory_obj = entry.clone_ref(py);
    drop(entry); // release DashMap guard before doing Python work

    let instance = factory_obj
        .call0(py)
        .map_err(|e| daemon_err(format!("factory for kind '{kind}' raised: {e}")))?;

    let process = instance.getattr(py, "process").map_err(|e| {
        daemon_err(format!(
            "factory return for kind '{kind}' has no `process` method: {e}"
        ))
    })?;

    let snapshot = match instance.getattr(py, "snapshot") {
        Ok(v) => {
            if v.is_none(py) {
                None
            } else {
                Some(v)
            }
        }
        Err(_) => None,
    };
    let restore = match instance.getattr(py, "restore") {
        Ok(v) => {
            if v.is_none(py) {
                None
            } else {
                Some(v)
            }
        }
        Err(_) => None,
    };

    Ok(Box::new(PyDaemonBridge {
        name: kind.to_string(),
        process,
        snapshot,
        restore,
    }))
}

/// Build a `PyDaemonBridge` from a tokio worker (no GIL held on
/// entry). Used by the SDK kind-factory closure for
/// migration-target reconstruction. Acquires the GIL via
/// `Python::attach`, calls the Python factory, extracts methods,
/// and returns a bridge — or a no-op fallback on any failure
/// (migrated-in daemons can't reliably panic the whole runtime).
fn build_bridge_from_factory(
    factories: &DashMap<String, Py<PyAny>>,
    kind: &str,
) -> Box<dyn MeshDaemon> {
    let factory_obj = match factories.get(kind) {
        Some(entry) => Python::attach(|py| entry.clone_ref(py)),
        None => {
            eprintln!(
                "build_bridge_from_factory: no factory for kind '{kind}'; returning NoopBridge"
            );
            return Box::new(NoopBridge::new(kind.to_string()));
        }
    };

    Python::attach(|py| -> Box<dyn MeshDaemon> {
        let instance = match factory_obj.call0(py) {
            Ok(i) => i,
            Err(e) => {
                eprintln!(
                    "build_bridge_from_factory: factory for kind '{kind}' raised: {e}; falling back to NoopBridge"
                );
                return Box::new(NoopBridge::new(kind.to_string()));
            }
        };
        let process = match instance.getattr(py, "process") {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "build_bridge_from_factory: kind '{kind}' daemon has no `process`: {e}; falling back to NoopBridge"
                );
                return Box::new(NoopBridge::new(kind.to_string()));
            }
        };
        let snapshot = match instance.getattr(py, "snapshot") {
            Ok(v) if !v.is_none(py) => Some(v),
            _ => None,
        };
        let restore = match instance.getattr(py, "restore") {
            Ok(v) if !v.is_none(py) => Some(v),
            _ => None,
        };
        Box::new(PyDaemonBridge {
            name: kind.to_string(),
            process,
            snapshot,
            restore,
        })
    })
}

// =========================================================================
// NoopBridge — migration-target fallback when the Python factory
// can't be reached or its return value is malformed.
// =========================================================================

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
