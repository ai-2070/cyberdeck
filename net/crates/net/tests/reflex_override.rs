//! Integration tests for stage 4a of `docs/NAT_TRAVERSAL_PLAN.md`:
//! `MeshNodeConfig::reflex_override`.
//!
//! A reflex override is for operators who already know their
//! node's public `SocketAddr` — port-forwarded servers, manually
//! configured VPN endpoints, or (stage 4b) a successful UPnP /
//! NAT-PMP mapping. Setting one short-circuits the classifier's
//! multi-peer sweep and starts the node in `NatClass::Open` with
//! the supplied address advertised on capability announcements.
//!
//! **Framing reminder** (plan §4): the override is an
//! optimization surface — a node with no override still reaches
//! every peer through the routed-handshake path. Tests assert
//! classifier + announcement semantics; they do NOT assert that
//! the mesh is otherwise broken without the override.
//!
//! # Properties under test
//!
//! - **Construction honors the override.** A mesh built with
//!   `with_reflex_override(addr)` reports `NatClass::Open` and
//!   `reflex_addr() == Some(addr)` immediately — no probes fired,
//!   no peers required.
//! - **Override propagates via capability announcement.** After
//!   A announces its capabilities, B's index sees A's reflex as
//!   the override value (not A's bind address).
//! - **`reclassify_nat` is a no-op when override is set.**
//!   Calling reclassify doesn't replace the overridden values
//!   with observed reflexes — the override is load-bearing even
//!   when the node has plenty of peers.
//! - **No override → normal classifier path.** A mesh without
//!   an override behaves exactly as before — Unknown until
//!   classified, then Open via observation on localhost.
//!
//! Run: `cargo test --features net,nat-traversal --test reflex_override`

#![cfg(all(feature = "net", feature = "nat-traversal"))]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::behavior::capability::CapabilitySet;
use net::adapter::net::traversal::classify::NatClass;
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

async fn build_mesh_with_override(
    port: u16,
    external: SocketAddr,
) -> Arc<MeshNode> {
    let cfg = test_config(port).with_reflex_override(external);
    Arc::new(
        MeshNode::new(EntityKeypair::generate(), cfg)
            .await
            .expect("MeshNode::new"),
    )
}

async fn build_mesh_plain(port: u16) -> Arc<MeshNode> {
    let cfg = test_config(port);
    Arc::new(
        MeshNode::new(EntityKeypair::generate(), cfg)
            .await
            .expect("MeshNode::new"),
    )
}

async fn connect_pair(a: &Arc<MeshNode>, b: &Arc<MeshNode>) {
    let a_id = a.node_id();
    let b_pub = *b.public_key();
    let b_addr = b.local_addr();
    let b_id = b.node_id();
    let b_clone = b.clone();
    let accept = tokio::spawn(async move { b_clone.accept(a_id).await });
    a.connect(b_addr, &b_pub, b_id)
        .await
        .expect("connect failed");
    accept
        .await
        .expect("accept task panicked")
        .expect("accept failed");
}

/// A freshly-built mesh with `reflex_override` set reports
/// `Open` + the override as its reflex_addr *before* any peers
/// are connected and *before* start() is called. No classification
/// probes happened — the override is load-bearing at init time.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_forces_open_at_construction() {
    let ports = find_ports(1).await;
    let external: SocketAddr = "203.0.113.42:9001".parse().unwrap();
    let node = build_mesh_with_override(ports[0], external).await;

    assert_eq!(
        node.nat_class(),
        NatClass::Open,
        "override should force Open at construction",
    );
    assert_eq!(
        node.reflex_addr(),
        Some(external),
        "reflex_addr should reflect the override",
    );
}

/// `reclassify_nat` is a no-op when the override is set. Even
/// with enough peers to run the sweep, the override is preserved.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reclassify_is_noop_when_override_set() {
    let ports = find_ports(3).await;
    let external: SocketAddr = "203.0.113.42:9001".parse().unwrap();
    let a = build_mesh_with_override(ports[0], external).await;
    let b = build_mesh_plain(ports[1]).await;
    let c = build_mesh_plain(ports[2]).await;

    connect_pair(&a, &b).await;
    connect_pair(&a, &c).await;
    a.start();
    b.start();
    c.start();

    // A has ≥2 peers; normally reclassify_nat would fire probes
    // and (on localhost) land on Open with reflex == bind. The
    // override must preempt that result.
    a.reclassify_nat().await;

    assert_eq!(a.nat_class(), NatClass::Open);
    assert_eq!(
        a.reflex_addr(),
        Some(external),
        "reflex_addr must stay at the override; reclassify must not overwrite it",
    );
}

/// Override propagates through the capability broadcast: B's
/// index, after receiving A's announcement, sees the override as
/// A's reflex, not A's bind address.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_propagates_through_capability_broadcast() {
    let ports = find_ports(2).await;
    let external: SocketAddr = "198.51.100.7:54321".parse().unwrap();
    let a = build_mesh_with_override(ports[0], external).await;
    let b = build_mesh_plain(ports[1]).await;

    connect_pair(&a, &b).await;
    a.start();
    b.start();

    a.announce_capabilities(CapabilitySet::new())
        .await
        .expect("A announce");

    // Allow the announcement to reach B and land in its index.
    let a_id = a.node_id();
    let mut ready = false;
    for _ in 0..30 {
        if b.peer_reflex_addr(a_id) == Some(external) {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        ready,
        "B should see A's override reflex (not bind); got {:?}",
        b.peer_reflex_addr(a_id),
    );

    // B should also see A's nat:open tag (override implies Open).
    let peers = b.find_peers_by_filter(
        &net::adapter::net::behavior::capability::CapabilityFilter::new()
            .require_tag("nat:open"),
    );
    assert!(
        peers.contains(&a_id),
        "B's index should tag A as nat:open; got peers = {peers:?}",
    );
}

/// A plain mesh (no override) still uses the classifier path
/// unchanged — Unknown until sweep, then Open via real probes.
/// Regression guard: adding the override should not affect the
/// default path.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn no_override_uses_classifier_path() {
    let ports = find_ports(3).await;
    let a = build_mesh_plain(ports[0]).await;
    let b = build_mesh_plain(ports[1]).await;
    let c = build_mesh_plain(ports[2]).await;

    connect_pair(&a, &b).await;
    connect_pair(&a, &c).await;
    a.start();
    b.start();
    c.start();

    // Pre-sweep: Unknown / None.
    assert_eq!(a.nat_class(), NatClass::Unknown);
    assert!(a.reflex_addr().is_none());

    a.reclassify_nat().await;

    // Post-sweep on localhost: Open, reflex == bind.
    assert_eq!(a.nat_class(), NatClass::Open);
    assert_eq!(a.reflex_addr(), Some(a.local_addr()));
}
