//! Regression test for a cubic-flagged P1 timing bug in the
//! keep-alive train scheduled by `schedule_punch`.
//!
//! # The bug
//!
//! `schedule_punch` computes three absolute offsets from the
//! task's spawn instant — `base_lead`, `base_lead + 100ms`,
//! `base_lead + 250ms` — and iterates them. An earlier revision
//! used `sleep(offset)` per iteration, which cumulatively summed:
//!
//! - Packet 1: waited `base_lead`              → fires at `base_lead`
//! - Packet 2: waited `base_lead + 100`        → fires at `2·base_lead + 100`
//! - Packet 3: waited `base_lead + 250`        → fires at `3·base_lead + 350`
//!
//! At the default `punch_fire_lead = 500 ms` this produced
//! 500 / 1100 / 1850 ms instead of the documented 500 / 600 /
//! 750 ms — packets 2 and 3 missed the peer's punch window
//! entirely.
//!
//! # The fix
//!
//! Anchor the schedule against a single `Instant` and
//! `sleep_until(start + offset)`. Scheduler jitter on packet N
//! doesn't delay packet N+1.
//!
//! # This test
//!
//! Builds a four-node rendezvous topology but overrides B's
//! reflex to point at a test-controlled UDP listener. After A
//! fires a punch, A's `schedule_punch` sends three keep-alives
//! to the listener. We measure their inter-arrival times.
//!
//! The documented spacing puts all three packets within
//! `base_lead + 250 ms` of start. At the default fire_lead
//! (500 ms) that's a ~250 ms total spread. We assert
//! `arrival[last] - arrival[first] < 500 ms` — clears the
//! correct schedule by 2×, rejects the buggy schedule (which
//! spread packets over >1300 ms) by a wide margin.
//!
//! Run: `cargo test --features net,nat-traversal --test punch_keepalive_timing`

#![cfg(all(feature = "net", feature = "nat-traversal"))]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use net::adapter::net::behavior::capability::CapabilitySet;
use net::adapter::net::traversal::rendezvous::{decode_keepalive, KEEPALIVE_LEN};
use net::adapter::net::{EntityKeypair, MeshNode, MeshNodeConfig, SocketBufferConfig};
use tokio::net::UdpSocket;

const TEST_BUFFER_SIZE: usize = 256 * 1024;
const PSK: [u8; 32] = [0x42u8; 32];

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

async fn build_node_with_reflex(override_addr: SocketAddr) -> Arc<MeshNode> {
    let cfg = test_config().with_reflex_override(override_addr);
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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn keepalive_train_spacing_stays_within_punch_window() {
    // Bind a UDP listener to stand in for B's reflex target.
    // A's `schedule_punch` will send three keep-alives here; we
    // record their arrival instants to verify the schedule
    // spacing.
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let listener_addr: SocketAddr = listener.local_addr().unwrap();

    // Four-node topology: A + R mesh normally; B is a real mesh
    // node but with `reflex_override = listener_addr`, so its
    // capability announcement advertises the listener as its
    // public reflex. X is the second probe target A + B need
    // for classification (so reclassify_nat produces a stable
    // reflex). B doesn't actually need to classify — its
    // reflex is overridden — but the topology shape is easier
    // to reason about when every node has ≥2 peers.
    let a = build_node().await;
    let r = build_node().await;
    let b = build_node_with_reflex(listener_addr).await;
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

    // A classifies + announces so R gets A's reflex too. B's
    // reflex is the override, published via its own announce.
    a.reclassify_nat().await;
    a.announce_capabilities(CapabilitySet::new())
        .await
        .expect("A announce");
    b.announce_capabilities(CapabilitySet::new())
        .await
        .expect("B announce");

    // Wait for R to see B's announced reflex (the override).
    let b_id = b.node_id();
    let r_for_poll = r.clone();
    let mut ready = false;
    for _ in 0..30 {
        if r_for_poll.peer_reflex_addr(b_id) == Some(listener_addr) {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        ready,
        "R should see B's override reflex; got {:?}",
        r.peer_reflex_addr(b_id),
    );

    // Start the clock + fire the punch. A's `schedule_punch`
    // task will start sending keep-alives to `listener_addr` at
    // offsets base_lead / base_lead+100 / base_lead+250 from
    // task-spawn.
    let t0 = tokio::time::Instant::now();
    let _intro = a
        .request_punch(r.node_id(), b_id, a.local_addr())
        .await
        .expect("request_punch should mediate");

    // Read up to 3 keep-alive packets from the listener, each
    // with its arrival instant. Filter by `decode_keepalive` so
    // stray packets (ack forwards, handshake traffic) don't
    // pollute the measurement.
    let mut arrivals: Vec<Duration> = Vec::with_capacity(3);
    let mut buf = [0u8; 64];
    let collection_deadline = Duration::from_secs(3);
    while arrivals.len() < 3 {
        let remaining = collection_deadline.saturating_sub(t0.elapsed());
        if remaining == Duration::ZERO {
            break;
        }
        match tokio::time::timeout(remaining, listener.recv_from(&mut buf)).await {
            Ok(Ok((n, _))) => {
                if n == KEEPALIVE_LEN && decode_keepalive(&buf[..n]).is_some() {
                    arrivals.push(t0.elapsed());
                }
            }
            Ok(Err(_)) | Err(_) => break,
        }
    }

    assert_eq!(
        arrivals.len(),
        3,
        "expected 3 keep-alives at listener; got {} (arrivals: {:?})",
        arrivals.len(),
        arrivals,
    );

    // Documented spacing: base_lead, base_lead+100ms,
    // base_lead+250ms from spawn → spread of 250 ms total.
    // The bug produced ~1350 ms spread (at default fire_lead).
    // Cap at 500 ms: 2× margin on the correct schedule, 2.7×
    // rejection margin on the buggy one.
    let spread = arrivals[2] - arrivals[0];
    assert!(
        spread < Duration::from_millis(500),
        "keep-alive train spread too wide — scheduler is drifting. \
         Expected packet3 - packet1 ≈ 250 ms; got {spread:?}. \
         Arrivals: {arrivals:?}",
    );

    // And packets should be roughly ordered (not bunched all at
    // t0, not all at the end). Assert packet 2 comes after
    // packet 1 + at least a few ms.
    assert!(
        arrivals[1] > arrivals[0] + Duration::from_millis(30),
        "packet 2 arrived too close to packet 1 ({:?} vs {:?}) — \
         schedule may be collapsed",
        arrivals[0],
        arrivals[1],
    );
    assert!(
        arrivals[2] > arrivals[1] + Duration::from_millis(30),
        "packet 3 arrived too close to packet 2 ({:?} vs {:?}) — \
         schedule may be collapsed",
        arrivals[1],
        arrivals[2],
    );
}
