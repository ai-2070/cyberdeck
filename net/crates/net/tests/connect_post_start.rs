//! Regression for BUG_AUDIT_2026_04_30_CORE.md #86:
//! `try_handshake_initiator` and `try_handshake_responder`
//! previously polled `socket_arc.recv_from` directly. After
//! `start()` had spawned `spawn_receive_loop`, both consumers
//! raced each datagram on the same `Arc<UdpSocket>` — tokio
//! dispatches a UDP datagram to exactly one waiter, so the
//! handshake response could be swallowed by the dispatch loop.
//!
//! The fix routes direct-handshake responses through a registry:
//! when `started` is true, the initiator registers an oneshot
//! keyed by the peer's `SocketAddr`, sends msg1, and awaits the
//! oneshot. The dispatcher's direct-handshake branch forwards
//! the parsed payload bytes through the matching oneshot.
//! Pre-`start()` the dispatcher isn't running, so the initiator
//! falls back to the original `recv_from` path (no race exists
//! pre-start).
//!
//! These tests pin both the post-`start()` path and the
//! concurrent-connect-on-same-node path.

#![cfg(feature = "net")]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::{EntityKeypair, MeshNode, MeshNodeConfig, SocketBufferConfig};

const TEST_BUFFER_SIZE: usize = 256 * 1024;
const PSK: [u8; 32] = [0x42u8; 32];

fn test_config() -> MeshNodeConfig {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut cfg = MeshNodeConfig::new(addr, PSK)
        .with_heartbeat_interval(Duration::from_millis(500))
        .with_session_timeout(Duration::from_secs(5))
        .with_handshake(3, Duration::from_secs(3));
    cfg.socket_buffers = SocketBufferConfig {
        send_buffer_size: TEST_BUFFER_SIZE,
        recv_buffer_size: TEST_BUFFER_SIZE,
    };
    cfg
}

async fn build_node() -> Arc<MeshNode> {
    Arc::new(
        MeshNode::new(EntityKeypair::generate(), test_config())
            .await
            .expect("MeshNode::new"),
    )
}

/// Initiator-side of #86: `connect()` after `start()` must
/// complete the handshake. Pre-fix the initiator's `recv_from`
/// raced the dispatch loop and the handshake response was
/// sometimes swallowed by the dispatcher; post-fix the
/// initiator registers a oneshot in `pending_direct_initiators`
/// and the dispatcher forwards the response through it.
///
/// We pin the documented contract for the responder side
/// (`accept` is called before `start`, where `recv_from` works
/// without a race), and exercise the post-start initiator path
/// that the audit flagged as hot.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn initiator_connect_after_start_completes_handshake() {
    let a = build_node().await;
    let b = build_node().await;

    let b_pub = *b.public_key();
    let b_addr = b.local_addr();
    let b_id = b.node_id();
    let a_id = a.node_id();

    // Per the documented contract on `MeshNode::start`:
    // "Must be called after `connect()` / `accept()` to begin
    // processing inbound packets." So we set up the responder
    // side BEFORE either node's dispatcher starts. The
    // initiator-side race the audit flagged is what we exercise
    // post-start.
    let b_clone = b.clone();
    let accept = tokio::spawn(async move { b_clone.accept(a_id).await });

    // Give B's `accept` a moment to register its `recv_from`
    // before A starts emitting packets.
    tokio::time::sleep(Duration::from_millis(20)).await;

    // BUG #86: A's dispatcher is now running. Without the fix,
    // A's `recv_from` inside `try_handshake_initiator` races the
    // dispatch loop and either may swallow B's msg2.
    a.start();

    a.connect(b_addr, &b_pub, b_id)
        .await
        .expect("connect after start must complete the handshake");

    accept
        .await
        .expect("accept task panicked")
        .expect("accept must complete");

    assert!(a.peer_count() > 0, "A must have registered the peer");
}

/// Multiple direct connects on the same node must each
/// receive their OWN handshake response — the registry is
/// keyed per-`SocketAddr`, so concurrent initiators don't
/// steal each other's responses.
///
/// Pre-fix all initiators raced one shared `recv_from`, so the
/// FIRST datagram to arrive could resolve any of them.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_connects_to_distinct_peers_dont_steal_responses() {
    let a = build_node().await;
    let b = build_node().await;
    let c = build_node().await;

    let b_pub = *b.public_key();
    let b_addr = b.local_addr();
    let b_id = b.node_id();
    let c_pub = *c.public_key();
    let c_addr = c.local_addr();
    let c_id = c.node_id();
    let a_id = a.node_id();

    // Spin up both responders BEFORE starting any dispatcher
    // (see #86 doc contract: `accept` must run before `start`).
    let b_clone = b.clone();
    let accept_b = tokio::spawn(async move { b_clone.accept(a_id).await });
    let c_clone = c.clone();
    let accept_c = tokio::spawn(async move { c_clone.accept(a_id).await });
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Now start A's dispatcher. The two concurrent initiators
    // below will both go through the registry path (started=true).
    a.start();
    b.start();
    c.start();

    // Concurrent initiators on the same node A targeting B and C.
    let a_clone1 = a.clone();
    let connect_b = tokio::spawn(async move {
        a_clone1.connect(b_addr, &b_pub, b_id).await
    });
    let a_clone2 = a.clone();
    let connect_c = tokio::spawn(async move {
        a_clone2.connect(c_addr, &c_pub, c_id).await
    });

    connect_b
        .await
        .expect("connect_b task panicked")
        .expect("A→B connect must succeed despite concurrent A→C connect");
    connect_c
        .await
        .expect("connect_c task panicked")
        .expect("A→C connect must succeed despite concurrent A→B connect");
    accept_b
        .await
        .expect("accept_b panicked")
        .expect("B accept must succeed");
    accept_c
        .await
        .expect("accept_c panicked")
        .expect("C accept must succeed");

    // Sanity: both peers registered.
    assert!(
        a.peer_count() >= 2,
        "A must have registered both B and C, got {}",
        a.peer_count()
    );
}
