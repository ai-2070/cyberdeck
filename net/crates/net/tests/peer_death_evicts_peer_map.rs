//! Regression test for BUGS.md MEDIUM — "failed peers are never
//! evicted from the `peers` map."
//!
//! The failure-detector `on_failure` callback used to clean up
//! `reroute_policy`, `roster`, `peer_subnets`, `peer_entity_ids`,
//! and `capability_index` — but never `self.peers` or
//! `self.addr_to_node`. A peer that went silent permanently still
//! had its `PeerInfo` (including its session) in `peers`, so
//! `send_to_peer` calls continued routing through a dead session
//! and UDP silently dropped packets until an application-layer
//! timeout fired.
//!
//! The fix adds the eviction to the `on_failure` closure. This
//! test pins it: after the failure detector marks a peer Failed,
//! that peer's `node_id` and `addr` must both be gone from the
//! live maps.
//!
//! Run: `cargo test --features net --test peer_death_evicts_peer_map`

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
        .with_heartbeat_interval(Duration::from_millis(100))
        .with_session_timeout(Duration::from_millis(500))
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn peers_map_evicts_failed_peer() {
    let a = build_node().await;
    let b = build_node().await;
    connect_pair(&a, &b).await;
    a.start();
    b.start();

    let b_id = b.node_id();
    let b_bind = b.local_addr();

    // Pre-condition: A has B registered in its peer map.
    assert_eq!(a.peer_count(), 1, "A must have exactly one peer (B)");
    assert_eq!(
        a.peer_addr(b_id),
        Some(b_bind),
        "A must know B's address via peer_addr()"
    );

    // Simulate B going silent from A's perspective.
    a.block_peer(b_bind);

    // Wait past 3 × session_timeout, then explicitly drive
    // `check_all()` to trigger the `on_failure` callback.
    tokio::time::sleep(Duration::from_millis(2_000)).await;
    let _ = a.failure_detector().check_all();

    // After failure, B must be gone from A's `peers` map.
    let evicted = wait_for(Duration::from_secs(2), || a.peer_addr(b_id).is_none()).await;
    assert!(
        evicted,
        "A's `peers` map must evict B after the failure detector \
         fires. peer_addr(B) = {:?}, peer_count = {}",
        a.peer_addr(b_id),
        a.peer_count(),
    );

    // `peer_count` should drop to zero — A no longer has any live
    // peers.
    assert_eq!(
        a.peer_count(),
        0,
        "peer_count must reflect the eviction; got {}",
        a.peer_count()
    );
}
