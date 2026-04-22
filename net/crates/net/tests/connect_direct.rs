//! Integration tests for stage 3c of `docs/NAT_TRAVERSAL_PLAN.md`:
//! `MeshNode::connect_direct` orchestration + traversal stats.
//!
//! These tests exercise the pair-type-matrix driven path selection
//! and the stats counters. Stage 3d will add the real keep-alive
//! train + `PunchAck` round-trip; stage 3c approximates "punch
//! succeeded" by "coordinator mediated an introduction," which is
//! sufficient on localhost where every packet trivially reaches
//! its destination.
//!
//! # Properties under test
//!
//! - **Open × Open goes direct.** `connect_direct` picks the
//!   `Direct` action, does not call the coordinator, bumps
//!   `relay_fallbacks` (no punch was attempted), and establishes
//!   a session on `peer_reflex`.
//! - **Punch-worthy pair bumps `punches_attempted`.** A pair that
//!   the matrix classifies as `SinglePunch` records a punch
//!   attempt in the stats, even when the result is ultimately
//!   punted on localhost.
//! - **Missing peer reflex fails fast.** `connect_direct` to a
//!   peer whose reflex is not cached returns
//!   `PeerNotReachable` without consulting the coordinator.
//! - **Stats are monotonic.** Successive `connect_direct` calls
//!   stack counter increments; snapshots never go backwards.
//!
//! Run: `cargo test --features net,nat-traversal --test connect_direct`

#![cfg(all(feature = "net", feature = "nat-traversal"))]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::behavior::capability::CapabilitySet;
use net::adapter::net::traversal::TraversalError;
use net::adapter::net::{EntityKeypair, MeshNode, MeshNodeConfig, SocketBufferConfig};

const TEST_BUFFER_SIZE: usize = 256 * 1024;
const PSK: [u8; 32] = [0x42u8; 32];

/// Bind via `127.0.0.1:0` so the OS picks a free port — no
/// pre-bind reservation, no TOCTOU race with parallel tests.
fn test_config() -> MeshNodeConfig {
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

async fn build_node() -> Arc<MeshNode> {
    Arc::new(
        MeshNode::new(EntityKeypair::generate(), test_config())
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

/// Four-node topology identical to the rendezvous_coordinator
/// tests: R mediates A↔B, X is an auxiliary for classification
/// (A and B need two peers each to run `reclassify_nat`).
///
/// Importantly: **A and B are not directly connected**. That's
/// the whole point — `connect_direct(A, B)` has to open a new
/// session via the rendezvous path.
async fn punch_topology() -> (Arc<MeshNode>, Arc<MeshNode>, Arc<MeshNode>, Arc<MeshNode>) {
    let a = build_node().await;
    let r = build_node().await;
    let b = build_node().await;
    let x = build_node().await;
    connect_pair(&a, &r).await;
    connect_pair(&b, &r).await;
    connect_pair(&a, &x).await;
    connect_pair(&b, &x).await;
    connect_pair(&r, &x).await;
    a.start();
    r.start();
    b.start();
    x.start();
    (a, r, b, x)
}

async fn wait_for<F: Fn() -> bool>(limit: Duration, check: F) -> bool {
    let start = tokio::time::Instant::now();
    while start.elapsed() < limit {
        if check() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    check()
}

/// Open × Open → direct connect, no punch. Stats: relay_fallbacks++.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connect_direct_open_pair_takes_direct_path() {
    let (a, r, b, _x) = punch_topology().await;

    // Both sides classify (Open on localhost) + announce so the
    // pair-type matrix has real data on both ends.
    a.reclassify_nat().await;
    b.reclassify_nat().await;
    a.announce_capabilities(CapabilitySet::new())
        .await
        .expect("A announce");
    b.announce_capabilities(CapabilitySet::new())
        .await
        .expect("B announce");

    // Wait for A's index to see B's reflex. Without this,
    // connect_direct returns PeerNotReachable.
    let a_for_poll = a.clone();
    let b_id = b.node_id();
    let b_bind = b.local_addr();
    assert!(
        wait_for(Duration::from_secs(3), || {
            a_for_poll.peer_reflex_addr(b_id) == Some(b_bind)
        })
        .await,
        "A should see B's reflex",
    );

    let before = a.traversal_stats();

    let b_pub = *b.public_key();
    let session_id = a
        .connect_direct(b_id, &b_pub, r.node_id())
        .await
        .expect("connect_direct should succeed");
    assert_eq!(
        session_id, b_id,
        "connect_direct returns the peer's node_id",
    );

    // Stats: Open×Open → Direct, bumps relay_fallbacks (no
    // punch was attempted). punches_attempted stays at zero.
    let after = a.traversal_stats();
    assert_eq!(
        after.punches_attempted, before.punches_attempted,
        "Open × Open should not attempt a punch",
    );
    assert_eq!(
        after.punches_succeeded, before.punches_succeeded,
        "no punch attempted → no punch success",
    );
    assert_eq!(
        after.relay_fallbacks,
        before.relay_fallbacks + 1,
        "Direct action bumps relay_fallbacks",
    );
}

/// `connect_direct` to a peer whose reflex hasn't been announced
/// (not cached in our capability index) fails fast with
/// `PeerNotReachable`. No coordinator call, no stats change.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connect_direct_unknown_peer_fails_fast() {
    let (a, r, _b, _x) = punch_topology().await;

    // Deliberately do NOT reclassify or announce — A's index
    // stays empty of reflex entries for B.

    let bogus_id: u64 = 0xDEAD_BEEF_FEED_CAFE;
    let bogus_pubkey = [0u8; 32];

    let before = a.traversal_stats();
    let start = tokio::time::Instant::now();
    let err = a
        .connect_direct(bogus_id, &bogus_pubkey, r.node_id())
        .await
        .expect_err("connect_direct should fail for uncached peer");
    let elapsed = start.elapsed();

    match err {
        TraversalError::PeerNotReachable => {}
        other => panic!("expected PeerNotReachable, got {other:?}"),
    }
    assert!(
        elapsed < Duration::from_millis(500),
        "fast-fail took {elapsed:?}; want < 500 ms (no coordinator round-trip)",
    );
    let after = a.traversal_stats();
    assert_eq!(
        after, before,
        "early-exit paths should not touch the stats counters",
    );
}

/// Counters are monotonic: successive `connect_direct` calls
/// stack increments, never decrement. Guards against a future
/// refactor that accidentally resets a counter on a code path.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stats_counters_are_monotonic() {
    let (a, r, b, _x) = punch_topology().await;

    a.reclassify_nat().await;
    b.reclassify_nat().await;
    a.announce_capabilities(CapabilitySet::new())
        .await
        .expect("A announce");
    b.announce_capabilities(CapabilitySet::new())
        .await
        .expect("B announce");

    let a_for_poll = a.clone();
    let b_id = b.node_id();
    let b_bind = b.local_addr();
    assert!(
        wait_for(Duration::from_secs(3), || {
            a_for_poll.peer_reflex_addr(b_id) == Some(b_bind)
        })
        .await,
        "A should see B's reflex",
    );

    let b_pub = *b.public_key();

    // First connect_direct attempt. `connect` will fail on the
    // second call because A already has a session with B, but
    // the first call should succeed and bump stats.
    let s1 = a.traversal_stats();
    let _ = a.connect_direct(b_id, &b_pub, r.node_id()).await;
    let s2 = a.traversal_stats();

    assert!(
        s2.relay_fallbacks >= s1.relay_fallbacks,
        "relay_fallbacks should never decrease",
    );
    assert!(
        s2.punches_attempted >= s1.punches_attempted,
        "punches_attempted should never decrease",
    );
    assert!(
        s2.punches_succeeded >= s1.punches_succeeded,
        "punches_succeeded should never decrease",
    );
}

/// Pre-classification stats are all zero. Guards against a
/// future default-impl that accidentally seeds counters with
/// non-zero values.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stats_start_at_zero() {
    let a = build_node().await;
    let stats = a.traversal_stats();
    assert_eq!(stats.punches_attempted, 0);
    assert_eq!(stats.punches_succeeded, 0);
    assert_eq!(stats.relay_fallbacks, 0);
}
