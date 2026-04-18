//! Integration tests for RedEX v1.
//!
//! Covers the single-node full pipeline: open a file through the
//! `Redex` manager, append many events, tail from seq 0 on a spawned
//! task, drive more appends, assert every event arrives in order with
//! matching payload and checksum.

#![cfg(feature = "redex")]

use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;
use net::adapter::net::channel::{AuthGuard, ChannelName};
use net::adapter::net::redex::{
    Redex, RedexError, RedexEvent, RedexFile, RedexFileConfig, RedexFold,
};

fn cn(s: &str) -> ChannelName {
    ChannelName::new(s).unwrap()
}

/// Open a file through the manager, append 10 000 events, tail from
/// seq 0, assert every event arrives in order.
#[tokio::test]
async fn test_redex_10k_roundtrip() {
    let r = Redex::new();
    let f = r
        .open_file(&cn("throughput/10k"), RedexFileConfig::default())
        .unwrap();

    const N: u64 = 10_000;
    let mut stream = Box::pin(f.tail(0));

    let f2 = f.clone();
    let writer = tokio::spawn(async move {
        for i in 0..N {
            f2.append(format!("event-{}", i).as_bytes()).unwrap();
            if i % 1024 == 0 {
                tokio::task::yield_now().await;
            }
        }
    });

    for i in 0..N {
        let ev = stream.next().await.unwrap().expect("event");
        assert_eq!(ev.entry.seq, i, "event {} arrived out of order", i);
        assert_eq!(ev.payload.as_ref(), format!("event-{}", i).as_bytes());
    }

    writer.await.unwrap();
}

#[tokio::test]
async fn test_redex_inline_only_roundtrip() {
    // Exercise the zero-segment-allocation inline path end-to-end.
    let r = Redex::new();
    let f = r.open_file(&cn("inline/only"), RedexFileConfig::default()).unwrap();

    for i in 0..1000u64 {
        f.append_inline(&i.to_le_bytes()).unwrap();
    }

    let events = f.read_range(0, 1000);
    assert_eq!(events.len(), 1000);
    for (i, ev) in events.iter().enumerate() {
        assert!(ev.entry.is_inline(), "event {} should be inline", i);
        let expected = (i as u64).to_le_bytes();
        assert_eq!(ev.payload.as_ref(), &expected);
    }
}

#[tokio::test]
async fn test_redex_tail_backfill_plus_live_is_gapless() {
    // Two writers: one that fills before tail opens, one that writes
    // while tail is live. Every event must arrive exactly once.
    let r = Redex::new();
    let f = r.open_file(&cn("gapless"), RedexFileConfig::default()).unwrap();

    for i in 0..500u64 {
        f.append(format!("pre-{}", i).as_bytes()).unwrap();
    }

    let mut stream = Box::pin(f.tail(0));

    let f2 = f.clone();
    let writer = tokio::spawn(async move {
        for i in 500..1000u64 {
            f2.append(format!("post-{}", i).as_bytes()).unwrap();
            if i % 64 == 0 {
                tokio::task::yield_now().await;
            }
        }
    });

    let mut seen_seqs = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let ev = stream.next().await.unwrap().unwrap();
        seen_seqs.push(ev.entry.seq);
    }
    writer.await.unwrap();

    for (i, &seq) in seen_seqs.iter().enumerate() {
        assert_eq!(seq, i as u64, "gap or dup at position {}", i);
    }
}

#[tokio::test]
async fn test_redex_retention_with_continued_tail() {
    // Retention evicts while a tail is live on the surviving tail.
    let cfg = RedexFileConfig::default()
        .with_retention_max_events(100);
    let r = Redex::new();
    let f = r.open_file(&cn("retention/ev"), cfg).unwrap();

    // Append 200 events.
    for i in 0..200u64 {
        f.append(format!("r-{}", i).as_bytes()).unwrap();
    }

    // Sweep — oldest 100 are dropped.
    f.sweep_retention();
    assert_eq!(f.len(), 100);

    // read_range of evicted region: empty.
    assert_eq!(f.read_range(0, 50).len(), 0);

    // Surviving tail: seq 100..200, in order.
    let events = f.read_range(100, 200);
    assert_eq!(events.len(), 100);
    assert_eq!(events[0].entry.seq, 100);
    assert_eq!(events[99].entry.seq, 199);
}

#[tokio::test]
async fn test_redex_close_signals_tail() {
    let r = Redex::new();
    let f = r.open_file(&cn("closing"), RedexFileConfig::default()).unwrap();
    f.append(b"hello").unwrap();

    let mut stream = Box::pin(f.tail(0));
    let _first = stream.next().await.unwrap().unwrap();

    r.close_file(&cn("closing")).unwrap();

    let err = stream.next().await.unwrap().unwrap_err();
    assert!(matches!(err, RedexError::Closed));

    // Stream ends after the Closed signal.
    assert!(stream.next().await.is_none());
}

#[test]
fn test_redex_auth_enforcement() {
    // Unauthorized origin is rejected at open_file even on a local
    // in-process manager.
    let guard = Arc::new(AuthGuard::new());
    let name = cn("locked");
    let r = Redex::with_auth(guard.clone(), 0xDEAD_BEEF);
    assert!(matches!(
        r.open_file(&name, RedexFileConfig::default()),
        Err(RedexError::Unauthorized)
    ));

    // Authorize and retry.
    guard.authorize(0xDEAD_BEEF, name.hash());
    assert!(r.open_file(&name, RedexFileConfig::default()).is_ok());
}

#[tokio::test]
async fn test_redex_fold_smoke() {
    // A toy fold that sums payload lengths — verifies the trait is
    // expressive enough to drive state from a tail stream.
    struct LenSum;
    impl RedexFold<u64> for LenSum {
        fn apply(&mut self, ev: &RedexEvent, state: &mut u64) -> Result<(), RedexError> {
            *state += ev.payload.len() as u64;
            Ok(())
        }
    }

    let r = Redex::new();
    let f = r.open_file(&cn("fold/smoke"), RedexFileConfig::default()).unwrap();
    for payload in ["a", "bb", "ccc", "dddd"] {
        f.append(payload.as_bytes()).unwrap();
    }

    let mut state = 0u64;
    let mut folder = LenSum;
    for ev in f.read_range(0, 4) {
        folder.apply(&ev, &mut state).unwrap();
    }
    assert_eq!(state, 1 + 2 + 3 + 4);
}

#[tokio::test]
async fn test_redex_append_batch_atomic_seq() {
    let r = Redex::new();
    let f: RedexFile = r
        .open_file(&cn("batches"), RedexFileConfig::default())
        .unwrap();

    let first = f
        .append_batch(&[
            Bytes::from_static(b"a"),
            Bytes::from_static(b"b"),
            Bytes::from_static(b"c"),
        ])
        .unwrap();
    assert_eq!(first, 0);
    assert_eq!(f.next_seq(), 3);

    // Interleave with a plain append then another batch.
    f.append(b"x").unwrap();
    let next = f
        .append_batch(&[Bytes::from_static(b"y"), Bytes::from_static(b"z")])
        .unwrap();
    assert_eq!(next, 4);

    let events = f.read_range(0, 6);
    let payloads: Vec<&[u8]> = events.iter().map(|e| e.payload.as_ref()).collect();
    assert_eq!(payloads, vec![&b"a"[..], b"b", b"c", b"x", b"y", b"z"]);
}
