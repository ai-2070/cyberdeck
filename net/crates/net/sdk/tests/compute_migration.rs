//! Rust SDK migration surface tests.
//!
//! Stage 2 of `SDK_COMPUTE_SURFACE_PLAN.md` — exercises
//! `DaemonRuntime::start_migration` end-to-end over an encrypted UDP
//! mesh, plus `MigrationHandle::wait` / `cancel` / `phase` on both
//! the happy path and failure modes.
//!
//! Scope: local-source case (source node == the orchestrator). The
//! remote-source case is covered by `three_node_integration.rs` at
//! the core layer.

#![cfg(feature = "compute")]

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use tokio::time::sleep;

use net::adapter::net::compute::DaemonError as CoreDaemonError;
use net::adapter::net::state::causal::CausalEvent;
use net_sdk::capabilities::CapabilityFilter;
use net_sdk::compute::{DaemonError, DaemonHostConfig, DaemonRuntime, MeshDaemon};
use net_sdk::mesh::{Mesh, MeshBuilder};
use net_sdk::Identity;

const PSK: [u8; 32] = [0x42u8; 32];

// ---- Fixture daemon: stateful counter ---------------------------------

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
                "counter needs 8 bytes, got {}",
                state.len()
            )));
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&state);
        self.count = u64::from_le_bytes(arr);
        Ok(())
    }
}

// ---- Harness: two meshes + runtimes + handshake ------------------------

struct Pair {
    source_rt: DaemonRuntime,
    target_rt: DaemonRuntime,
}

async fn build_pair() -> Pair {
    let a = MeshBuilder::new("127.0.0.1:0", &PSK)
        .unwrap()
        .build()
        .await
        .expect("build a");
    let b = MeshBuilder::new("127.0.0.1:0", &PSK)
        .unwrap()
        .build()
        .await
        .expect("build b");
    handshake(&a, &b).await;
    let source_rt = DaemonRuntime::new(Arc::new(a));
    let target_rt = DaemonRuntime::new(Arc::new(b));
    Pair { source_rt, target_rt }
}

async fn handshake(a: &Mesh, b: &Mesh) {
    let addr_b = b.inner().local_addr();
    let pub_b = *b.inner().public_key();
    let nid_b = b.inner().node_id();
    let nid_a = a.inner().node_id();
    let (r1, r2) = tokio::join!(b.inner().accept(nid_a), async {
        sleep(Duration::from_millis(50)).await;
        a.inner().connect(addr_b, &pub_b, nid_b).await
    });
    r1.expect("accept");
    r2.expect("connect");
}

fn counter_factory() -> impl Fn() -> Box<dyn MeshDaemon> + Send + Sync + 'static {
    || Box::new(CounterDaemon { count: 0 })
}

// ---- Tests -------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn start_migration_requires_ready_runtime() {
    let pair = build_pair().await;
    // Source is Registering — spawn / migrate rejected.
    let err = pair
        .source_rt
        .start_migration(0xDEAD_BEEF, 0, 0)
        .await
        .expect_err("start_migration must fail while Registering");
    assert!(matches!(err, DaemonError::NotReady), "got {err:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn local_source_migration_reaches_complete_and_transfers_state() {
    let pair = build_pair().await;
    let Pair {
        source_rt,
        target_rt,
    } = &pair;

    // Register the same factory on both nodes. Target uses it to
    // reconstruct the daemon after the snapshot arrives.
    source_rt
        .register_factory("counter", counter_factory())
        .unwrap();
    target_rt
        .register_factory("counter", counter_factory())
        .unwrap();

    source_rt.start().await.unwrap();
    target_rt.start().await.unwrap();

    // Start both meshes' receive loops.
    source_rt.mesh().inner().start();
    target_rt.mesh().inner().start();
    sleep(Duration::from_millis(100)).await;

    // Daemon identity — shared by the test harness between source
    // and target. In production Stages 5b / 6 of the identity-
    // migration plan transport the keypair across the wire; today
    // we hand it to both runtimes out-of-band.
    let identity = Identity::generate();
    let origin_hash = identity.keypair().origin_hash();
    let handle = source_rt
        .spawn("counter", identity.clone(), DaemonHostConfig::default())
        .await
        .expect("spawn on source");
    for i in 1..=3u64 {
        source_rt
            .deliver(
                handle.origin_hash,
                &make_event(origin_hash, i, b"tick"),
            )
            .expect("deliver");
    }

    // Pre-register the target's factory with the shared identity —
    // the stopgap until `register_migration_target_identity` is
    // replaced by the identity envelope that travels with the
    // snapshot.
    target_rt
        .register_migration_target_identity(
            "counter",
            identity,
            DaemonHostConfig::default(),
        )
        .expect("pre-register target identity");

    // Kick off the migration: source → target.
    let mig = source_rt
        .start_migration(
            handle.origin_hash,
            source_rt.mesh().inner().node_id(),
            target_rt.mesh().inner().node_id(),
        )
        .await
        .expect("start_migration");
    assert_eq!(mig.origin_hash, handle.origin_hash);

    // Wait for Complete. Migration should finish within a couple
    // seconds on a localhost UDP loop.
    let result = mig.wait_with_timeout(Duration::from_secs(5)).await;
    result.expect("migration reached Complete");

    // Target now holds the daemon. Deliver one more event and
    // assert the counter continued from 3 (not reset to 0).
    let outputs = target_rt
        .deliver(origin_hash, &make_event(origin_hash, 4, b"post-migration"))
        .expect("deliver on target");
    assert_eq!(outputs.len(), 1);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&outputs[0].payload);
    assert_eq!(
        u64::from_le_bytes(bytes),
        4,
        "counter must continue from the pre-migration state, not reset",
    );

    // Source should no longer host the daemon.
    assert_eq!(source_rt.daemon_count(), 0);
    assert_eq!(target_rt.daemon_count(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn migration_to_unconnected_peer_fails_target_unavailable() {
    let pair = build_pair().await;
    pair.source_rt
        .register_factory("counter", counter_factory())
        .unwrap();
    pair.source_rt.start().await.unwrap();
    pair.source_rt.mesh().inner().start();

    let identity = Identity::generate();
    let origin_hash = identity.keypair().origin_hash();
    let _handle = pair
        .source_rt
        .spawn("counter", identity, DaemonHostConfig::default())
        .await
        .expect("spawn");

    let err = pair
        .source_rt
        .start_migration(
            origin_hash,
            pair.source_rt.mesh().inner().node_id(),
            0xDEAD_BEEF_CAFE_F00D, // no session with this node_id
        )
        .await
        .expect_err("unconnected target must fail");
    match err {
        DaemonError::Migration(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains("unavailable") || msg.contains("send"),
                "expected TargetUnavailable-flavored error, got: {msg}",
            );
        }
        other => panic!("expected DaemonError::Migration, got {other:?}"),
    }

    // Orchestrator must have rolled back — no stale migration record.
    let mig = pair.source_rt.start_migration(
        origin_hash,
        pair.source_rt.mesh().inner().node_id(),
        0xBEEF_CAFE_FEED_F00D,
    );
    // A second migration attempt should get the same outcome, not
    // AlreadyMigrating — confirming the rollback.
    let err2 = mig.await.expect_err("second attempt still fails");
    assert!(
        !matches!(
            err2,
            DaemonError::Migration(net_sdk::compute::MigrationError::AlreadyMigrating(_))
        ),
        "orchestrator should have cleaned up after the first failure, not held a \
         stale migration record — got {err2:?}",
    );
}

// ---- Helpers -----------------------------------------------------------

fn make_event(origin_hash: u32, seq: u64, payload: &'static [u8]) -> CausalEvent {
    use net::adapter::net::state::causal::CausalLink;
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

