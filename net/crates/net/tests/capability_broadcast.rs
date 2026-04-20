//! Integration tests for the capability-broadcast subprotocol
//! (`SUBPROTOCOL_CAPABILITY_ANN = 0x0C00`).
//!
//! Covers the four load-bearing guarantees of Stage C-1:
//! - Two-node announce → find round-trip
//! - TTL expiry: post-TTL queries no longer return the peer
//! - Late joiner: session-open push catches new peers up
//! - Version skip: older announcements from the same peer are ignored
//!
//! Run: `cargo test --features net --test capability_broadcast`

#![cfg(feature = "net")]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::behavior::capability::{
    CapabilityAnnouncement, CapabilityFilter, CapabilitySet,
};
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
        .with_handshake(3, Duration::from_secs(2))
        .with_capability_gc_interval(Duration::from_millis(250));
    cfg.socket_buffers = SocketBufferConfig {
        send_buffer_size: TEST_BUFFER_SIZE,
        recv_buffer_size: TEST_BUFFER_SIZE,
    };
    cfg
}

/// Build an unstarted MeshNode and return it alongside its node_id.
async fn build_node(port: u16) -> Arc<MeshNode> {
    build_node_with(port, |cfg| cfg).await
}

/// Build a MeshNode with a caller-supplied tweak to the test config.
async fn build_node_with<F>(port: u16, tweak: F) -> Arc<MeshNode>
where
    F: FnOnce(MeshNodeConfig) -> MeshNodeConfig,
{
    let cfg = tweak(test_config(port));
    let keypair = EntityKeypair::generate();
    Arc::new(MeshNode::new(keypair, cfg).await.expect("MeshNode::new"))
}

/// Handshake two nodes (A initiator, B responder) and `start()` both.
async fn handshake(a: &Arc<MeshNode>, b: &Arc<MeshNode>) {
    let a_id = a.node_id();
    let b_id = b.node_id();
    let b_pub = *b.public_key();
    let b_addr = b.local_addr();

    let b_clone = b.clone();
    let accept = tokio::spawn(async move { b_clone.accept(a_id).await });
    // No pre-connect sleep — `handshake_initiator` and
    // `handshake_responder` each have internal retry-with-backoff.
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

/// Poll `cond` on `node` every 25ms for up to 2s; returns `true` on
/// match. Callers use this instead of a fixed `sleep` so slow CI
/// boxes don't flake.
async fn wait_until<F>(node: &Arc<MeshNode>, mut cond: F) -> bool
where
    F: FnMut(&MeshNode) -> bool,
{
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < deadline {
        if cond(node) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    cond(node)
}

#[tokio::test]
async fn two_node_announce_is_visible() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    let caps = CapabilitySet::new().add_tag("gpu").add_tag("inference");
    a.announce_capabilities(caps)
        .await
        .expect("announce failed");

    let filter = CapabilityFilter::new().require_tag("gpu");
    let a_id = a.node_id();
    let arrived = wait_until(&b, |node| {
        node.find_peers_by_filter(&filter).contains(&a_id)
    })
    .await;
    assert!(arrived, "B did not observe A's capability announcement");
}

#[tokio::test]
async fn announcement_expires_after_ttl() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    let caps = CapabilitySet::new().add_tag("ephemeral");
    // TTL = 1s; GC tick from `test_config` is 250ms, so two or three
    // sweeps land before we re-query at 1.5s.
    a.announce_capabilities_with(caps, Duration::from_secs(1), false)
        .await
        .expect("announce failed");

    let filter = CapabilityFilter::new().require_tag("ephemeral");
    let a_id = a.node_id();
    assert!(
        wait_until(&b, |n| n.find_peers_by_filter(&filter).contains(&a_id)).await,
        "B never indexed A's announcement in the first place"
    );

    // Wait beyond TTL; GC should evict.
    tokio::time::sleep(Duration::from_millis(1_500)).await;
    let still_present = b.find_peers_by_filter(&filter).contains(&a_id);
    assert!(
        !still_present,
        "B still returns A after TTL expiry (GC not running?)"
    );
}

#[tokio::test]
async fn late_joiner_receives_session_open_push() {
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;

    // A announces *before* B exists.
    let caps = CapabilitySet::new().add_tag("preannounced");
    a.announce_capabilities(caps)
        .await
        .expect("announce failed");

    // B joins the party.
    let b = build_node(ports[1]).await;
    handshake(&a, &b).await;

    let filter = CapabilityFilter::new().require_tag("preannounced");
    let a_id = a.node_id();
    let arrived = wait_until(&b, |n| n.find_peers_by_filter(&filter).contains(&a_id)).await;
    assert!(
        arrived,
        "session-open push did not deliver the pre-announcement"
    );
}

#[tokio::test]
async fn require_signed_capabilities_drops_unsigned_announcements() {
    // Post-E-1, `announce_capabilities` signs by default. Test the
    // policy knob by explicitly calling `announce_capabilities_with`
    // with `sign = false` — receiver B's flag must drop those.
    // Receiver B has the flag on; sender A announces unsigned.
    // A self-indexes its own announcement (local path bypasses
    // receive), so a self-query on A still matches — only B's view
    // should be blank.
    let ports = find_ports(2).await;
    let a = build_node(ports[0]).await;
    let b = build_node_with(ports[1], |cfg| cfg.with_require_signed_capabilities(true)).await;
    handshake(&a, &b).await;

    a.announce_capabilities_with(
        CapabilitySet::new().add_tag("classified"),
        Duration::from_secs(60),
        false, // unsigned
    )
    .await
    .expect("announce failed");

    // A sees itself (local self-index isn't subject to the flag).
    let filter = CapabilityFilter::new().require_tag("classified");
    assert!(
        a.find_peers_by_filter(&filter).contains(&a.node_id()),
        "sender lost its own self-index"
    );

    // Give the receive path a few ms to process (or drop).
    tokio::time::sleep(Duration::from_millis(100)).await;

    // B must not have indexed A's unsigned announcement.
    assert!(
        !b.find_peers_by_filter(&filter).contains(&a.node_id()),
        "receiver accepted an unsigned announcement despite require_signed_capabilities=true"
    );
}

#[tokio::test]
async fn stale_versions_are_ignored_by_index() {
    // Dodges the mesh entirely — the version-skip invariant is a
    // property of CapabilityIndex itself, not the subprotocol.
    // Keeping the test here so the whole "capability pipeline" suite
    // lives together and this regression catches anyone who alters
    // index semantics.
    use net::adapter::net::behavior::CapabilityIndex;
    use net::adapter::net::EntityId;

    let index = CapabilityIndex::new();
    let caps_v1 = CapabilitySet::new().add_tag("v1");
    let caps_v2 = CapabilitySet::new().add_tag("v2");

    // Direct index test — no mesh, no signature verification.
    // A zero-byte EntityId is a valid data-structure input even
    // though it's not a valid ed25519 public key.
    let eid = EntityId::from_bytes([0u8; 32]);
    let v1 = CapabilityAnnouncement::new(/* node_id */ 0xAA, eid.clone(), 1, caps_v1);
    let v2 = CapabilityAnnouncement::new(0xAA, eid, 2, caps_v2);

    index.index(v2);
    index.index(v1); // older — must be a no-op

    let v2_filter = CapabilityFilter::new().require_tag("v2");
    assert_eq!(index.query(&v2_filter), vec![0xAA]);

    let v1_filter = CapabilityFilter::new().require_tag("v1");
    assert!(
        index.query(&v1_filter).is_empty(),
        "older version overwrote the newer one"
    );
}
