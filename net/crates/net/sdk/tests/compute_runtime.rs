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
        .deliver(rehydrated.origin_hash, &event(origin_hash, 6, b"resumed"))
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

/// Regression (Cubic-AI P1): `spawn_from_snapshot` used to compare
/// only `origin_hash` (a 32-bit BLAKE2s projection of the entity
/// pubkey), not the full 32-byte `entity_id`. On a birthday-bounded
/// collision (~2^16 daemons in a pool, ~2^32 across a runtime's
/// lifetime), two legitimately-different identities can share the
/// same `origin_hash`. Pre-fix, the SDK would accept the mismatched
/// identity at its check layer, create a factory entry, and only
/// fail much later when `DaemonHost::from_snapshot` did its own
/// full-bytes check — surfacing as `DaemonError::Core(RestoreFailed)`
/// rather than the semantically correct `SnapshotIdentityMismatch`.
/// Callers relying on the typed error (the docstring advertises it)
/// would never see it.
///
/// This test brute-force searches for a real origin_hash collision
/// between two ed25519 keypairs, then feeds the snapshot of one
/// through `spawn_from_snapshot` with the other's identity.
///
/// Runtime: ~1–3 seconds on modern hardware (birthday-bound ~2^16
/// keygens against a 32-bit hash). Bounded at 300 000 attempts to
/// prevent pathological CI hangs.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_from_snapshot_checks_full_entity_id_not_just_origin_hash() {
    use std::collections::HashMap;

    // Deterministic-seed brute force: Identity::from_seed takes any
    // 32-byte value, so iterate a counter → seed for reproducibility.
    let mut seen: HashMap<u32, Identity> = HashMap::new();
    let mut collision: Option<(Identity, Identity)> = None;
    for i in 0u64..300_000 {
        let mut seed = [0u8; 32];
        seed[..8].copy_from_slice(&i.to_le_bytes());
        let id = Identity::from_seed(seed);
        let h = id.keypair().origin_hash();
        if let Some(prior) = seen.remove(&h) {
            if prior.entity_id() != id.entity_id() {
                collision = Some((prior, id));
                break;
            }
        }
        seen.insert(h, id);
    }
    let (ident_a, ident_b) = collision
        .expect("no origin_hash collision found within the attempt budget — try raising the bound");
    assert_eq!(
        ident_a.keypair().origin_hash(),
        ident_b.keypair().origin_hash(),
        "fixture: pair must collide on origin_hash",
    );
    assert_ne!(
        ident_a.entity_id(),
        ident_b.entity_id(),
        "fixture: pair must have different entity_ids",
    );

    let rt = runtime().await;
    rt.register_factory("counter", || Box::new(CounterDaemon { count: 0 }))
        .expect("register");
    rt.start().await.expect("start");

    // Spawn with A, take a real snapshot, stop.
    let handle = rt
        .spawn("counter", ident_a.clone(), DaemonHostConfig::default())
        .await
        .expect("spawn A");
    let snapshot = handle
        .snapshot()
        .await
        .expect("snapshot")
        .expect("counter is stateful");
    rt.stop(handle.origin_hash).await.expect("stop");

    // Attempt restore with B, whose origin_hash collides with A's
    // but whose full entity_id differs. Must be rejected at the
    // SDK check layer with the typed `SnapshotIdentityMismatch`
    // variant — NOT `DaemonError::Core(RestoreFailed)`, which
    // would indicate the check slipped past the SDK and was only
    // caught by the deeper `DaemonHost::from_snapshot` backstop.
    let err = rt
        .spawn_from_snapshot("counter", ident_b, snapshot, DaemonHostConfig::default())
        .await
        .expect_err("collision but distinct entity_id must reject");
    match err {
        DaemonError::SnapshotIdentityMismatch { .. } => {}
        DaemonError::Core(inner) => panic!(
            "origin_hash collision slipped past the SDK check and was only caught by the \
             core backstop ({inner:?}); the SDK must do its own full-entity_id check",
        ),
        other => panic!("expected SnapshotIdentityMismatch, got {other:?}"),
    }
}

/// Regression (Cubic-AI P1): `start()` used to flip the runtime
/// state to `Ready` **before** calling `set_migration_handler`.
/// Any thread observing `is_ready() == true` in that window would
/// try to migrate against a handler-less mesh — the dispatcher's
/// fallback synthesises `ComputeNotSupported`, aborting the
/// migration nondeterministically during startup.
///
/// The fix installs the handler first, then CAS-flips state. This
/// test races a background observer against `start()`: the watcher
/// spin-reads `is_ready()` and records whether `has_migration_handler()`
/// was false at the moment `is_ready()` first became true. Under
/// the pre-fix ordering, the watcher occasionally catches the gap.
/// Under the fix, the handler is always live by the time state
/// publishes `Ready`.
///
/// The test repeats the observation many times to make a single
/// flaky run unlikely to mask a regression. `std::thread` (not
/// `tokio::spawn`) is used because the observer needs to tight-
/// loop across the runtime's state transitions; tokio cooperative
/// scheduling could starve it long past the race window.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn start_installs_handler_before_publishing_ready() {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering as AOrd};
    use std::sync::Arc as StdArc;

    // 64 trials total. A single observable race is enough to fail
    // the test — the assertion trips on the first gap witnessed.
    for trial in 0..64u32 {
        let rt = runtime().await;
        let mesh = rt.mesh().clone();

        // Shared signals between observer thread and the main
        // task. `gap_witnessed` records the bug; `started` asks
        // the observer to stop after start() completes so we can
        // join it.
        let gap_witnessed = StdArc::new(AtomicBool::new(false));
        let observer_done = StdArc::new(AtomicBool::new(false));
        let first_ready_tick = StdArc::new(AtomicU32::new(0));

        let rt_w = rt.clone();
        let mesh_w = mesh.clone();
        let gap_w = gap_witnessed.clone();
        let done_w = observer_done.clone();
        let first_w = first_ready_tick.clone();

        let observer = std::thread::spawn(move || {
            let mut ticks = 0u32;
            // Spin until is_ready() flips. At the moment of the
            // flip, check whether the handler is installed. Under
            // the pre-fix ordering, the flip can precede the
            // install → `has_migration_handler == false`.
            loop {
                if rt_w.is_ready() {
                    if !mesh_w.inner().has_migration_handler() {
                        gap_w.store(true, AOrd::Release);
                    }
                    first_w.store(ticks, AOrd::Release);
                    break;
                }
                ticks = ticks.saturating_add(1);
                // Yield rarely; tight loop keeps us in the window.
                if ticks & 0xFFFF == 0 {
                    std::thread::yield_now();
                }
                if done_w.load(AOrd::Acquire) {
                    return;
                }
            }
        });

        // Small jitter so the observer has a chance to hit the
        // spin loop before start() executes.
        std::thread::sleep(std::time::Duration::from_micros(50));
        rt.start().await.expect("start");
        observer_done.store(true, AOrd::Release);
        observer.join().expect("observer panicked");

        assert!(
            !gap_witnessed.load(AOrd::Acquire),
            "trial {trial}: observed Ready-without-handler gap — start() flipped state \
             to Ready before set_migration_handler completed",
        );

        let _ = first_ready_tick.load(AOrd::Acquire); // keep for debugging
    }
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
    //
    // Regression (Cubic AI P1): the duplicate spawn used to fail at
    // the `DaemonRegistry::register` step *after* the factory_registry
    // had already been silently clobbered by the second insert, so
    // the rollback then removed the slot the *first* daemon was
    // relying on. The first daemon would stay live in the registry
    // but lose its factory entry — future migrations of that daemon
    // would then fail to construct on the source. `DaemonFactoryRegistry::register`
    // is now atomic-on-collision: we fail fast at the factory
    // registration step, before the incumbent's slot is touched.
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
        .spawn("echo", identity.clone(), DaemonHostConfig::default())
        .await
        .expect_err("second spawn with same identity must fail");
    // Core(ProcessFailed("... already registered")) is what the
    // atomic factory_registry now returns. Anything else means the
    // collision was caught at a later stage — i.e., the factory slot
    // was clobbered first, which is exactly the regressed behavior.
    match err {
        DaemonError::Core(CoreDaemonError::ProcessFailed(ref m)) => {
            assert!(
                m.contains("already registered"),
                "expected 'already registered' in message, got {m:?}",
            );
        }
        other => panic!(
            "expected Core(ProcessFailed(already registered)) from atomic factory_registry; \
             got {other:?} — collision caught too late may mean the incumbent's slot was clobbered",
        ),
    }
    // Runtime still reports exactly one daemon — the incumbent is
    // untouched.
    assert_eq!(rt.daemon_count(), 1);

    // Prove the incumbent's factory is still usable: stop it and
    // re-spawn from snapshot via the same kind. Under the pre-fix
    // behavior, the rollback had stripped the factory_registry entry,
    // so on-mesh migration of this daemon would have failed with
    // FactoryNotFound at restore time. spawn_from_snapshot uses the
    // SDK-level kind map (not factory_registry), so this succeeds
    // regardless — the real migration-side regression is covered by
    // compute_migration::duplicate_spawn_preserves_migratability.
    let snapshot = rt
        .snapshot(identity.keypair().origin_hash())
        .await
        .expect("snapshot");
    assert!(
        snapshot.is_none(),
        "EchoDaemon is stateless, so snapshot returns Ok(None)",
    );
}

/// Regression: duplicate `spawn_from_snapshot` must not corrupt the
/// incumbent's factory entry. Same root cause as
/// [`spawn_same_identity_twice_is_rejected`] but exercised through
/// the snapshot-rehydration path.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn duplicate_spawn_from_snapshot_does_not_corrupt_first_daemon() {
    let rt = runtime().await;
    rt.register_factory("counter", || Box::new(CounterDaemon { count: 0 }))
        .expect("register");
    rt.start().await.expect("start");

    let identity = Identity::generate();
    let handle = rt
        .spawn("counter", identity.clone(), DaemonHostConfig::default())
        .await
        .expect("first spawn");
    // Drive a few events so the snapshot is non-trivial.
    for i in 1..=2u64 {
        rt.deliver(handle.origin_hash, &event(handle.origin_hash, i, b"tick"))
            .expect("deliver");
    }
    let snapshot = handle
        .snapshot()
        .await
        .expect("snapshot")
        .expect("counter is stateful");

    // Duplicate spawn_from_snapshot with the same identity + the
    // living daemon's snapshot. Fails at the atomic factory_registry
    // step, before touching the incumbent's slot.
    let err = rt
        .spawn_from_snapshot(
            "counter",
            identity.clone(),
            snapshot,
            DaemonHostConfig::default(),
        )
        .await
        .expect_err("duplicate spawn_from_snapshot must fail");
    match err {
        DaemonError::Core(CoreDaemonError::ProcessFailed(ref m)) => {
            assert!(
                m.contains("already registered"),
                "expected 'already registered' in message, got {m:?}",
            );
        }
        other => panic!("expected Core(ProcessFailed), got {other:?}"),
    }
    assert_eq!(rt.daemon_count(), 1);

    // Incumbent still processes events — its state wasn't disturbed.
    let outputs = rt
        .deliver(
            handle.origin_hash,
            &event(handle.origin_hash, 3, b"post-dupe"),
        )
        .expect("deliver after failed duplicate");
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&outputs[0].payload);
    assert_eq!(
        u64::from_le_bytes(bytes),
        3,
        "incumbent counter must continue at 3, not reset",
    );
}

/// Regression: `expect_migration` (placeholder register) must fail
/// cleanly on collision, not replace the incumbent's keypair-bearing
/// entry with a placeholder. Before the fix, a target that
/// accidentally double-called `expect_migration` for the same
/// `origin_hash` would silently overwrite, and a subsequent migration
/// restore would fail at `resolve_restore_keypair` when the envelope
/// path was opted out of.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expect_migration_collision_is_rejected() {
    let rt = runtime().await;
    rt.register_factory("echo", || Box::new(EchoDaemon))
        .expect("register");
    rt.start().await.expect("start");

    let origin_hash = 0xDEAD_BEEFu32;
    rt.expect_migration("echo", origin_hash, DaemonHostConfig::default())
        .expect("first expect_migration");
    let err = rt
        .expect_migration("echo", origin_hash, DaemonHostConfig::default())
        .expect_err("duplicate expect_migration must fail");
    match err {
        DaemonError::Core(CoreDaemonError::ProcessFailed(ref m)) => {
            assert!(m.contains("already registered"), "got {m:?}");
        }
        other => panic!("expected Core(ProcessFailed), got {other:?}"),
    }
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
