//! Rust SDK smoke test: stream open → concurrent sends hit the
//! window → `SdkError::Backpressure` surfaces through the SDK layer,
//! `send_with_retry` absorbs the pressure and eventually succeeds.
//!
//! This exercises the full chain `MeshNode` → core `StreamError` →
//! `SdkError::Backpressure` conversion in `sdk/src/error.rs`. If the
//! `From<StreamError>` impl regresses, this test surfaces it
//! immediately rather than at daemon runtime.

#![cfg(feature = "net")]

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use net_sdk::error::SdkError;
use net_sdk::mesh::MeshBuilder;
use net_sdk::{Reliability, StreamConfig};
use tokio::net::UdpSocket;

async fn find_two_ports() -> (u16, u16) {
    let s1 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let s2 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let p = (
        s1.local_addr().unwrap().port(),
        s2.local_addr().unwrap().port(),
    );
    drop(s1);
    drop(s2);
    tokio::time::sleep(Duration::from_millis(10)).await;
    p
}

/// Open a stream with window=1, spawn concurrent sends, assert that at
/// least one observes `SdkError::Backpressure` (i.e. the core variant
/// crosses the SDK boundary cleanly).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_sdk_surfaces_backpressure_variant() {
    let (port_a, port_b) = find_two_ports().await;
    let addr_a = format!("127.0.0.1:{}", port_a);
    let addr_b = format!("127.0.0.1:{}", port_b);
    let psk = [0x42u8; 32];

    let a = MeshBuilder::new(&addr_a, &psk)
        .unwrap()
        .build()
        .await
        .unwrap();
    let b = MeshBuilder::new(&addr_b, &psk)
        .unwrap()
        .build()
        .await
        .unwrap();

    // Handshake (A connects to B as initiator, B accepts).
    let pub_b = *b.inner().public_key();
    let nid_b = b.inner().node_id();
    let nid_a = a.inner().node_id();

    let (r1, r2) = tokio::join!(b.inner().accept(nid_a), async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        a.inner()
            .connect(addr_b.parse().unwrap(), &pub_b, nid_b)
            .await
    });
    r1.expect("accept");
    r2.expect("connect");
    a.inner().start();
    b.inner().start();

    let a = Arc::new(a);
    let stream = a
        .open_stream(
            nid_b,
            0x1337,
            StreamConfig::new()
                .with_reliability(Reliability::FireAndForget)
                .with_window_bytes(1),
        )
        .expect("open_stream");

    // 16 concurrent sends on a window-1 stream. At least one must
    // cross the SDK boundary as `SdkError::Backpressure`.
    let mut handles = Vec::new();
    for _ in 0..16 {
        let mesh = Arc::clone(&a);
        let stream = stream.clone();
        handles.push(tokio::spawn(async move {
            mesh.send_on_stream(&stream, &[Bytes::from_static(b"{}")])
                .await
        }));
    }
    let mut bp = 0usize;
    let mut ok = 0usize;
    for h in handles {
        match h.await.unwrap() {
            Ok(()) => ok += 1,
            Err(SdkError::Backpressure) => bp += 1,
            Err(e) => panic!("unexpected error kind: {:?}", e),
        }
    }
    assert!(
        bp > 0,
        "expected SdkError::Backpressure from concurrent sends; ok={}, bp={}",
        ok,
        bp
    );
    assert!(ok > 0);

    let stats = a.stream_stats(nid_b, 0x1337).expect("stats");
    assert_eq!(stats.tx_window, 1);
    assert!(stats.backpressure_events >= bp as u64);

    // Shutdown — consume the Arc.
    Arc::try_unwrap(a).ok().unwrap().shutdown().await.unwrap();
    b.shutdown().await.unwrap();
}

/// `send_with_retry` should absorb Backpressure and eventually succeed
/// on a 4-slot window under 32 concurrent senders.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_sdk_send_with_retry_succeeds_through_backpressure() {
    let (port_a, port_b) = find_two_ports().await;
    let addr_a = format!("127.0.0.1:{}", port_a);
    let addr_b = format!("127.0.0.1:{}", port_b);
    let psk = [0x42u8; 32];

    let a = MeshBuilder::new(&addr_a, &psk)
        .unwrap()
        .build()
        .await
        .unwrap();
    let b = MeshBuilder::new(&addr_b, &psk)
        .unwrap()
        .build()
        .await
        .unwrap();

    let pub_b = *b.inner().public_key();
    let nid_b = b.inner().node_id();
    let nid_a = a.inner().node_id();

    let (r1, r2) = tokio::join!(b.inner().accept(nid_a), async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        a.inner()
            .connect(addr_b.parse().unwrap(), &pub_b, nid_b)
            .await
    });
    r1.unwrap();
    r2.unwrap();
    a.inner().start();
    b.inner().start();

    let a = Arc::new(a);
    let stream = a
        .open_stream(nid_b, 0x2468, StreamConfig::new().with_window_bytes(4))
        .unwrap();

    let mut handles = Vec::new();
    for i in 0..32 {
        let mesh = Arc::clone(&a);
        let stream = stream.clone();
        let payload = Bytes::from(format!(r#"{{"i":{}}}"#, i));
        handles.push(tokio::spawn(async move {
            mesh.send_with_retry(&stream, &[payload], 64).await
        }));
    }
    for h in handles {
        h.await.unwrap().expect("send_with_retry must succeed");
    }

    let stats = a.stream_stats(nid_b, 0x2468).unwrap();
    assert_eq!(stats.tx_inflight, 0);

    Arc::try_unwrap(a).ok().unwrap().shutdown().await.unwrap();
    b.shutdown().await.unwrap();
}
