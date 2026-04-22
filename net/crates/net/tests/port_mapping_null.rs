//! Stage 4b integration tests: port-mapping task wiring on the
//! `MeshNode` surface.
//!
//! The task-lifecycle details (probe / install / renew / revoke)
//! are covered by unit tests in `traversal::portmap::tests` with
//! a deterministic `MockPortMapperClient`. This file covers the
//! integration boundary:
//!
//! - Flag gating — `try_port_mapping(false)` spawns no task.
//! - Explicit client injection via
//!   [`MeshNode::spawn_port_mapping_loop`] — the path tests +
//!   future SDK consumers use to drive the lifecycle against a
//!   known client (mock or real).
//!
//! The `start()` auto-spawn path, which now uses the real
//! [`SequentialMapper`] (NAT-PMP + UPnP), is covered by the
//! `#[ignore]`d real-router test in `tests/port_mapping_real_router.rs`
//! — we don't test it from CI because it would either produce
//! non-deterministic results (on developer machines with a live
//! router) or contact-external-network behavior we don't want
//! in the integration suite.
//!
//! Run: `cargo test --features port-mapping --test port_mapping_null`

#![cfg(all(feature = "net", feature = "port-mapping"))]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::traversal::classify::NatClass;
use net::adapter::net::traversal::portmap::{MockPortMapperClient, PortMapping, Protocol};
use net::adapter::net::{EntityKeypair, MeshNode, MeshNodeConfig, SocketBufferConfig};

const TEST_BUFFER_SIZE: usize = 256 * 1024;
const PSK: [u8; 32] = [0x42u8; 32];

/// Bind via `127.0.0.1:0` so the OS picks a free port — no
/// pre-bind reservation, no TOCTOU race with parallel tests.
fn base_config() -> MeshNodeConfig {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut cfg = MeshNodeConfig::new(addr, PSK)
        .with_heartbeat_interval(Duration::from_millis(200))
        .with_session_timeout(Duration::from_secs(5))
        .with_handshake(3, Duration::from_secs(2));
    cfg.socket_buffers = SocketBufferConfig {
        send_buffer_size: TEST_BUFFER_SIZE,
        recv_buffer_size: TEST_BUFFER_SIZE,
    };
    cfg
}

async fn build_node(cfg: MeshNodeConfig) -> Arc<MeshNode> {
    Arc::new(
        MeshNode::new(EntityKeypair::generate(), cfg)
            .await
            .expect("MeshNode::new"),
    )
}

/// Default `try_port_mapping(false)` — no port-mapping task
/// spawned. Stats show inactive; reflex untouched.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn no_port_mapping_when_flag_unset() {
    let node = build_node(base_config()).await;
    node.start();

    // No time to spawn anything — the flag is unset.
    let stats = node.traversal_stats();
    assert!(!stats.port_mapping_active);
    assert_eq!(stats.port_mapping_external, None);
    assert_eq!(stats.port_mapping_renewals, 0);

    // reflex_addr stays None (no override), nat_class stays
    // Unknown (no classifier run against anyone).
    assert!(node.reflex_addr().is_none());
    assert_eq!(node.nat_class(), NatClass::Unknown);
}

/// Explicit spawn via `spawn_port_mapping_loop` with a mock
/// client that returns a successful install. Verify the
/// install flips `port_mapping_active = true`,
/// `port_mapping_external = Some(mapped)`, and `reflex_addr`
/// returns the mapped address (override is active).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn explicit_spawn_install_pins_reflex_override() {
    let mut cfg = base_config();
    // Short renewal so the mock queue turns over fast in this test.
    cfg.heartbeat_interval = Duration::from_millis(100);
    let node = build_node(cfg).await;
    node.start();

    let external: SocketAddr = "198.51.100.11:54321".parse().unwrap();
    let mapping = PortMapping {
        external,
        internal_port: node.local_addr().port(),
        ttl: Duration::from_secs(3600),
        protocol: Protocol::Upnp,
    };

    let mock = Arc::new(MockPortMapperClient::new());
    mock.queue_probe(Ok(()));
    mock.queue_install(Ok(mapping));
    // Endless renewal successes so only our shutdown ends the
    // task — we want to observe the pinned state without the
    // 3-strike revoke firing.
    for _ in 0..100 {
        mock.queue_renew(Ok(mapping));
    }

    let handle = node.spawn_port_mapping_loop(Box::new(Arc::clone(&mock)));

    // Wait for install to land.
    let mut installed = false;
    for _ in 0..50 {
        if node.traversal_stats().port_mapping_active {
            installed = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(installed, "install should flip the active flag");

    let stats = node.traversal_stats();
    assert_eq!(stats.port_mapping_external, Some(external));
    assert_eq!(
        node.reflex_addr(),
        Some(external),
        "reflex_addr should return the mapped external — override is active",
    );
    assert_eq!(
        node.nat_class(),
        NatClass::Open,
        "install should force nat_class to Open",
    );

    // Let the task exit by aborting — we don't want to wait
    // through the 100-tick renewal queue.
    handle.abort();
}
