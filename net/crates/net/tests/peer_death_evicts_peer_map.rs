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
//! The fix adds a sweep to the heartbeat loop that evicts peers
//! which have been in the failure detector's `Failed` state with
//! session-idle for longer than `10 × session_timeout` — long
//! enough that transient-partition recovery still works (the
//! `on_recovery` callback runs off incoming heartbeats that are
//! matched against the retained session), but short enough that
//! permanently-gone peers are eventually dropped.
//!
//! Refactored onto the Stage-3 chaos harness (`tests/common/`)
//! so the polling + config + setup boilerplate — previously 80
//! lines of local helpers — reduces to two lines of imports.
//! The original behavior this regression pins is unchanged;
//! only the plumbing is shared.
//!
//! Run: `cargo test --features net --test peer_death_evicts_peer_map`

#![cfg(feature = "net")]

use std::time::Duration;

use net::adapter::net::{MeshNodeConfig, SocketBufferConfig};

mod common;
use common::*;

/// Config with a shorter session_timeout (300 ms) than the
/// harness default (500 ms) — the peer-eviction sweep fires at
/// 30 × session_timeout, so this keeps the wall-clock under 10 s
/// rather than ~15 s with the default.
fn config() -> MeshNodeConfig {
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut cfg = MeshNodeConfig::new(addr, CHAOS_PSK)
        .with_heartbeat_interval(Duration::from_millis(100))
        .with_session_timeout(Duration::from_millis(300))
        .with_handshake(3, Duration::from_secs(2));
    cfg.socket_buffers = SocketBufferConfig {
        send_buffer_size: CHAOS_BUFFER_SIZE,
        recv_buffer_size: CHAOS_BUFFER_SIZE,
    };
    cfg
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn peers_map_evicts_permanently_failed_peer() {
    // session_timeout = 300 ms → dead_peer_timeout = 9 s
    // (30 × session_timeout). Block B from A's perspective and
    // wait past that threshold so the heartbeat-loop sweep runs
    // and evicts. The 30× multiplier leaves plenty of room for
    // transient-partition recovery; this test only exercises the
    // permanently-gone path by never unblocking.
    let a = build_node_with(config()).await;
    let b = build_node_with(config()).await;
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

    // Simulate B going permanently silent from A's perspective.
    chaos_one_sided_block(&a, &b);

    // Drive FD through Failed, then wait past 30 × session_timeout
    // = 9 s for the heartbeat-loop sweep to run and evict B.
    await_peer_failed(&a, b_id, Duration::from_secs(3)).await;
    await_peer_count(&a, 0, Duration::from_secs(12)).await;

    // Post-condition: both the peer-by-id lookup and the
    // peer_count invariant agree on B's eviction.
    assert!(
        a.peer_addr(b_id).is_none(),
        "A's peer_addr(B) must be None after eviction; got {:?}",
        a.peer_addr(b_id),
    );
}
