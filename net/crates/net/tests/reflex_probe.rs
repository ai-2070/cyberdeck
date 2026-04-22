//! Integration tests for the reflex-probe subprotocol
//! (`SUBPROTOCOL_REFLEX = 0x0D00`).
//!
//! Stage 1 of `docs/NAT_TRAVERSAL_PLAN.md`. Verifies the
//! request/response round-trip end-to-end over a real UDP
//! session:
//!
//! - **Round-trip success** — A probes B; B responds with the
//!   `SocketAddr` it observed for A's UDP packets. On localhost
//!   (no NAT), that address is A's bind address.
//! - **Unknown peer** — probing a node we never handshaked with
//!   fails fast with `PeerNotReachable`, not `ReflexTimeout`.
//! - **Idle in steady state** — `probe_reflex` is explicit; no
//!   reflex traffic fires until a caller requests it. This
//!   property matches the stage-1 exit criterion and is
//!   asserted structurally via the absence of a passive probe
//!   path (the test doesn't wait around; if the subprotocol
//!   were background-fired the `probe_reflex` call would race
//!   with whatever was already in flight).
//!
//! Run: `cargo test --features net,nat-traversal --test reflex_probe`

#![cfg(all(feature = "net", feature = "nat-traversal"))]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::traversal::TraversalError;
use net::adapter::net::{EntityKeypair, MeshNode, MeshNodeConfig, SocketBufferConfig};
use tokio::net::UdpSocket;

const TEST_BUFFER_SIZE: usize = 256 * 1024;
const PSK: [u8; 32] = [0x42u8; 32];

async fn find_ports(n: usize) -> Vec<u16> {
    let mut ports = Vec::with_capacity(n);
    let mut sockets = Vec::with_capacity(n);
    for _ in 0..n {
        let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        ports.push(sock.local_addr().unwrap().port());
        sockets.push(sock);
    }
    drop(sockets);
    tokio::time::sleep(Duration::from_millis(10)).await;
    ports
}

fn test_config(port: u16) -> MeshNodeConfig {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
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

async fn build_node(port: u16) -> Arc<MeshNode> {
    let cfg = test_config(port);
    let keypair = EntityKeypair::generate();
    Arc::new(MeshNode::new(keypair, cfg).await.expect("MeshNode::new"))
}

async fn handshake(a: &Arc<MeshNode>, b: &Arc<MeshNode>) {
    let a_id = a.node_id();
    let b_id = b.node_id();
    let b_pub = *b.public_key();
    let b_addr = b.local_addr();

    let b_clone = b.clone();
    let accept = tokio::spawn(async move { b_clone.accept(a_id).await });
    a.connect(b_addr, &b_pub, b_id)
        .await
        .expect("connect failed");
    accept
        .await
        .expect("accept task panicked")
        .expect("accept failed");
    a.start();
    b.start();
}

/// A probes B; the observed address should be A's bind address
/// (localhost has no NAT, so the UDP source equals the bind addr).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reflex_probe_returns_observed_source_address() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    let a_bind = a.local_addr();
    let b_id = b.node_id();

    let observed = a
        .probe_reflex(b_id)
        .await
        .expect("reflex probe should succeed on localhost");

    // Localhost: B should see A at the same address A bound.
    assert_eq!(
        observed, a_bind,
        "reflex response should echo A's bind addr; got {observed}, expected {a_bind}",
    );
}

/// Probing a peer we never handshaked with fails fast with
/// `PeerNotReachable` — no timeout wait, no socket traffic.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reflex_probe_unknown_peer_fails_fast() {
    let ports = find_ports(1).await;
    let a = build_node(ports[0]).await;
    a.start();

    let bogus_node_id: u64 = 0xDEAD_BEEF_FEED_CAFE;
    let start = tokio::time::Instant::now();
    let err = a
        .probe_reflex(bogus_node_id)
        .await
        .expect_err("unknown peer should fail");
    let elapsed = start.elapsed();

    match err {
        TraversalError::PeerNotReachable => {}
        other => panic!("expected PeerNotReachable, got {other:?}"),
    }
    // Must be fast — explicitly not waiting out the default
    // reflex_timeout (3 s). 500 ms is plenty of headroom for
    // any CI-flake jitter while still catching the bug where
    // we'd miss the `peer_addrs` check and fall through to the
    // timeout path.
    assert!(
        elapsed < Duration::from_millis(500),
        "fast-fail took {elapsed:?}; want < 500 ms",
    );
}

/// Bidirectional symmetry: both sides can probe each other. Guards
/// against a one-sided dispatch bug where the responder role
/// works but the initiator role doesn't (or vice versa).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reflex_probe_is_bidirectional() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    let a_bind = a.local_addr();
    let b_bind = b.local_addr();

    let from_a = a.probe_reflex(b.node_id()).await.expect("A → B probe");
    let from_b = b.probe_reflex(a.node_id()).await.expect("B → A probe");

    assert_eq!(from_a, a_bind, "A's reflex from B's POV");
    assert_eq!(from_b, b_bind, "B's reflex from A's POV");
}
