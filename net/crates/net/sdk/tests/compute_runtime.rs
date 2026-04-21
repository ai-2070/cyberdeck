//! Rust SDK smoke + surface tests for the compute runtime.
//!
//! Stage 1 of `SDK_COMPUTE_SURFACE_PLAN.md` (local spawn / snapshot /
//! stop) plus the lifecycle fence from
//! `DAEMON_RUNTIME_READINESS_PLAN.md`. Migration paths are exercised
//! by Stage 2 once the subprotocol wiring lands — this file only
//! covers the local surface.

#![cfg(feature = "compute")]

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use bytes::Bytes;

use net::adapter::net::compute::DaemonError as CoreDaemonError;
use net::adapter::net::state::causal::{CausalEvent, CausalLink};
use net_sdk::capabilities::CapabilityFilter;
use net_sdk::compute::{DaemonError, DaemonHostConfig, DaemonRuntime, MeshDaemon};
use net_sdk::mesh::MeshBuilder;
use net_sdk::Identity;

const PSK: [u8; 32] = [0x42u8; 32];

// ---- Fixtures ---------------------------------------------------------

/// Stateless echo: returns each inbound payload verbatim as one output.
struct EchoDaemon;

impl MeshDaemon for EchoDaemon {
    fn name(&self) -> &str {
        "echo"
    }
    fn requirements(&self) -> CapabilityFilter {
        CapabilityFilter::default()
    }
    fn process(&mut self, event: &CausalEvent) -> Result<Vec<Bytes>, CoreDaemonError> {
        Ok(vec![event.payload.clone()])
    }
}

/// Stateful counter: increments on every event and snapshots / restores
/// the running total as a little-endian u64.
struct CounterDaemon {
    count: u64,
}

impl MeshDaemon for CounterDaemon {
    fn name(&self) -> &str {
        "counter"
    }
    fn requirements(&self) -> CapabilityFilter {
        CapabilityFilter::default()
    }
    fn process(&mut self, _event: &CausalEvent) -> Result<Vec<Bytes>, CoreDaemonError> {
        self.count += 1;
        Ok(vec![Bytes::copy_from_slice(&self.count.to_le_bytes())])
    }
    fn snapshot(&self) -> Option<Bytes> {
        Some(Bytes::copy_from_slice(&self.count.to_le_bytes()))
    }
    fn restore(&mut self, state: Bytes) -> Result<(), CoreDaemonError> {
        if state.len() != 8 {
            return Err(CoreDaemonError::RestoreFailed(format!(
                "counter state must be 8 bytes, got {}",
                state.len()
            )));
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&state);
        self.count = u64::from_le_bytes(arr);
        Ok(())
    }
}

async fn runtime() -> DaemonRuntime {
    let mesh = MeshBuilder::new("127.0.0.1:0", &PSK)
        .unwrap()
        .build()
        .await
        .expect("build mesh");
    DaemonRuntime::new(Arc::new(mesh))
}

fn event(origin_hash: u32, seq: u64, payload: &'static [u8]) -> CausalEvent {
    CausalEvent {
        link: CausalLink {
            origin_hash,
            horizon_encoded: 0,
            sequence: seq,
            parent_hash: 0,
        },
        payload: Bytes::from_static(payload),
        received_at: 0,
    }
}

// ---- Lifecycle --------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registering_rejects_spawn_with_not_ready() {
    let rt = runtime().await;
    rt.register_factory("echo", || Box::new(EchoDaemon))
        .expect("register");

    let err = rt
        .spawn("echo", Identity::generate(), DaemonHostConfig::default())
        .await
        .expect_err("spawn before start must fail");
    assert!(
        matches!(err, DaemonError::NotReady),
        "expected NotReady, got {err:?}",
    );
    assert!(!rt.is_ready());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_is_idempotent() {
    let rt = runtime().await;
    rt.start().await.expect("first start");
    rt.start().await.expect("second start is a no-op");
    assert!(rt.is_ready());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_rejects_subsequent_spawn_and_register() {
    let rt = runtime().await;
    rt.start().await.expect("start");
    rt.shutdown().await.expect("shutdown");

    let spawn_err = rt
        .spawn("echo", Identity::generate(), DaemonHostConfig::default())
        .await
        .expect_err("spawn after shutdown must fail");
    assert!(
        matches!(spawn_err, DaemonError::ShuttingDown),
        "expected ShuttingDown, got {spawn_err:?}",
    );

    let reg_err = rt
        .register_factory("echo", || Box::new(EchoDaemon))
        .expect_err("register after shutdown must fail");
    assert!(
        matches!(reg_err, DaemonError::ShuttingDown),
        "expected ShuttingDown, got {reg_err:?}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_factory_rejects_duplicate_kind() {
    let rt = runtime().await;
    rt.register_factory("echo", || Box::new(EchoDaemon))
        .expect("first register");
    let err = rt
        .register_factory("echo", || Box::new(EchoDaemon))
        .expect_err("duplicate kind must fail");
    match err {
        DaemonError::FactoryAlreadyRegistered(ref k) => assert_eq!(k, "echo"),
        other => panic!("expected FactoryAlreadyRegistered, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_new_kind_after_ready_is_allowed() {
    // The runtime permits runtime-discovered kinds — `Ready` does not
    // freeze the factory table. Only `ShuttingDown` rejects.
    let rt = runtime().await;
    rt.start().await.expect("start");
    rt.register_factory("late", || Box::new(EchoDaemon))
        .expect("register after start");
    let _ = rt
        .spawn("late", Identity::generate(), DaemonHostConfig::default())
        .await
        .expect("spawn late-registered kind");
}

// ---- Local spawn / deliver / stop -----------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn echo_daemon_roundtrip() {
    let rt = runtime().await;
    rt.register_factory("echo", || Box::new(EchoDaemon))
        .expect("register");
    rt.start().await.expect("start");

    let identity = Identity::generate();
    let origin_hash = identity.keypair().origin_hash();
    let handle = rt
        .spawn("echo", identity, DaemonHostConfig::default())
        .await
        .expect("spawn");
    assert_eq!(handle.origin_hash, origin_hash);

    let outputs = rt
        .deliver(handle.origin_hash, &event(origin_hash, 1, b"ping"))
        .expect("deliver");
    assert_eq!(outputs.len(), 1);
    assert_eq!(&outputs[0].payload[..], b"ping");

    let stats = handle.stats().expect("stats");
    assert_eq!(stats.events_processed, 1);
    assert_eq!(stats.events_emitted, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn counter_snapshot_round_trip_through_runtime() {
    let rt = runtime().await;
    rt.register_factory("counter", || Box::new(CounterDaemon { count: 0 }))
        .expect("register");
    rt.start().await.expect("start");

    let identity = Identity::generate();
    let origin_hash = identity.keypair().origin_hash();
    let handle = rt
        .spawn("counter", identity.clone(), DaemonHostConfig::default())
        .await
        .expect("spawn");

    for i in 1..=5u64 {
        let outputs = rt
            .deliver(handle.origin_hash, &event(origin_hash, i, b"tick"))
            .expect("deliver");
        assert_eq!(outputs.len(), 1);
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&outputs[0].payload);
        assert_eq!(u64::from_le_bytes(bytes), i);
    }

    let snapshot = handle
        .snapshot()
        .await
        .expect("snapshot")
        .expect("counter is stateful");
    assert_eq!(snapshot.through_seq, 5);

    // Stop + re-spawn from snapshot under the SAME identity. The
    // runtime must accept the snapshot because its origin_hash
    // matches the identity's origin_hash.
    rt.stop(handle.origin_hash).await.expect("stop");
    let rehydrated = rt
        .spawn_from_snapshot("counter", identity, snapshot, DaemonHostConfig::default())
        .await
        .expect("spawn_from_snapshot");

    // Counter survived the round-trip: the next event should report 6,
    // not 1.
    let outputs = rt
        .deliver(
            rehydrated.origin_hash,
            &event(origin_hash, 6, b"resumed"),
        )
        .expect("deliver after restore");
    assert_eq!(outputs.len(), 1);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&outputs[0].payload);
    assert_eq!(u64::from_le_bytes(bytes), 6);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_from_snapshot_rejects_identity_mismatch() {
    let rt = runtime().await;
    rt.register_factory("counter", || Box::new(CounterDaemon { count: 0 }))
        .expect("register");
    rt.start().await.expect("start");

    let ident_a = Identity::generate();
    let handle = rt
        .spawn("counter", ident_a.clone(), DaemonHostConfig::default())
        .await
        .expect("spawn");
    let snapshot = handle
        .snapshot()
        .await
        .expect("snapshot")
        .expect("counter is stateful");
    rt.stop(handle.origin_hash).await.expect("stop");

    // A different identity must be rejected — the snapshot is tied
    // to the daemon's original entity_id.
    let ident_b = Identity::generate();
    assert_ne!(
        ident_a.keypair().origin_hash(),
        ident_b.keypair().origin_hash(),
        "fixture: fresh identity must differ",
    );
    let err = rt
        .spawn_from_snapshot("counter", ident_b, snapshot, DaemonHostConfig::default())
        .await
        .expect_err("identity mismatch must be rejected");
    assert!(
        matches!(err, DaemonError::SnapshotIdentityMismatch { .. }),
        "expected SnapshotIdentityMismatch, got {err:?}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_unknown_kind_errors() {
    let rt = runtime().await;
    rt.start().await.expect("start");
    let err = rt
        .spawn(
            "never-registered",
            Identity::generate(),
            DaemonHostConfig::default(),
        )
        .await
        .expect_err("unknown kind must fail");
    match err {
        DaemonError::FactoryNotFound(ref k) => assert_eq!(k, "never-registered"),
        other => panic!("expected FactoryNotFound, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_same_identity_twice_is_rejected() {
    // Two daemons can't share the same origin_hash. The runtime
    // surfaces the core's `ProcessFailed` with a "already registered"
    // message as a `DaemonError::Core(_)`.
    let rt = runtime().await;
    rt.register_factory("echo", || Box::new(EchoDaemon))
        .expect("register");
    rt.start().await.expect("start");

    let identity = Identity::generate();
    let _handle = rt
        .spawn("echo", identity.clone(), DaemonHostConfig::default())
        .await
        .expect("first spawn");
    let err = rt
        .spawn("echo", identity, DaemonHostConfig::default())
        .await
        .expect_err("second spawn with same identity must fail");
    assert!(matches!(err, DaemonError::Core(_)), "got {err:?}");
    // And the runtime still reports exactly one daemon.
    assert_eq!(rt.daemon_count(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_drops_daemon_from_registry() {
    let rt = runtime().await;
    rt.register_factory("echo", || Box::new(EchoDaemon))
        .expect("register");
    rt.start().await.expect("start");

    let handle = rt
        .spawn("echo", Identity::generate(), DaemonHostConfig::default())
        .await
        .expect("spawn");
    assert_eq!(rt.daemon_count(), 1);

    rt.stop(handle.origin_hash).await.expect("stop");
    assert_eq!(rt.daemon_count(), 0);

    // Deliver to a now-gone daemon surfaces NotFound.
    let err = rt
        .deliver(handle.origin_hash, &event(handle.origin_hash, 1, b"drop"))
        .expect_err("deliver to gone daemon must fail");
    match err {
        DaemonError::Core(CoreDaemonError::NotFound(o)) => assert_eq!(o, handle.origin_hash),
        other => panic!("expected Core(NotFound), got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_drains_all_daemons() {
    let rt = runtime().await;
    rt.register_factory("echo", || Box::new(EchoDaemon))
        .expect("register");
    rt.start().await.expect("start");

    // Spawn five daemons with distinct identities.
    for _ in 0..5 {
        rt.spawn("echo", Identity::generate(), DaemonHostConfig::default())
            .await
            .expect("spawn");
    }
    assert_eq!(rt.daemon_count(), 5);

    rt.shutdown().await.expect("shutdown");
    assert_eq!(rt.daemon_count(), 0);
}

// ---- Factory closure sharing -----------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn factory_is_invoked_once_per_spawn() {
    // The closure lives behind an `Arc<dyn Fn>` inside the runtime; we
    // observe via a shared counter that exactly one `Fn()` call lands
    // per successful spawn.
    let counter = Arc::new(AtomicU32::new(0));
    let counter_for_factory = counter.clone();

    let rt = runtime().await;
    rt.register_factory("echo", move || {
        counter_for_factory.fetch_add(1, Ordering::SeqCst);
        Box::new(EchoDaemon)
    })
    .expect("register");
    rt.start().await.expect("start");

    for _ in 0..3 {
        rt.spawn("echo", Identity::generate(), DaemonHostConfig::default())
            .await
            .expect("spawn");
    }
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}
