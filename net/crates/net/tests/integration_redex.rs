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
    OrderedAppender, Redex, RedexError, RedexEvent, RedexFile, RedexFileConfig, RedexFold,
    TypedRedexFile,
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
    let f = r
        .open_file(&cn("inline/only"), RedexFileConfig::default())
        .unwrap();

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
    let f = r
        .open_file(&cn("gapless"), RedexFileConfig::default())
        .unwrap();

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
    let cfg = RedexFileConfig::default().with_retention_max_events(100);
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
async fn test_ordered_appender_sequential_seqs() {
    // Single-threaded use of OrderedAppender: seqs are strictly
    // 0, 1, 2, ... in the order we call append.
    let r = Redex::new();
    let f = r
        .open_file(&cn("ordered/seq"), RedexFileConfig::default())
        .unwrap();
    let appender = OrderedAppender::new(f.clone());

    let mut seqs = Vec::new();
    for i in 0..100u64 {
        seqs.push(appender.append(format!("e{}", i).as_bytes()).unwrap());
    }
    for (i, &seq) in seqs.iter().enumerate() {
        assert_eq!(seq, i as u64);
    }

    // read_range returns the events in seq order — which also equals
    // insertion order for ordered appends.
    let events = f.read_range(0, 100);
    assert_eq!(events.len(), 100);
    for (i, ev) in events.iter().enumerate() {
        assert_eq!(ev.entry.seq, i as u64);
        assert_eq!(ev.payload.as_ref(), format!("e{}", i).as_bytes());
    }
}

#[tokio::test]
async fn test_ordered_appender_concurrent_writers_stay_in_seq_order() {
    // N concurrent tasks each call append via one OrderedAppender.
    // After all join, the index must be in strict seq order — no
    // seq gaps, no out-of-order insertions.
    let r = Redex::new();
    let f = r
        .open_file(&cn("ordered/concurrent"), RedexFileConfig::default())
        .unwrap();
    let appender = OrderedAppender::new(f.clone());

    const THREADS: usize = 8;
    const PER_THREAD: usize = 200;
    let mut handles = Vec::new();
    for t in 0..THREADS {
        let a = appender.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..PER_THREAD {
                let payload = format!("t{}-e{}", t, i);
                let _ = a.append(payload.as_bytes()).unwrap();
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let total = THREADS * PER_THREAD;
    let events = f.read_range(0, total as u64);
    assert_eq!(events.len(), total);
    // Index order matches seq order strictly.
    for (i, ev) in events.iter().enumerate() {
        assert_eq!(ev.entry.seq, i as u64, "out-of-order at position {}", i);
    }
}

#[tokio::test]
async fn test_ordered_appender_inline_and_batch() {
    let r = Redex::new();
    let f = r
        .open_file(&cn("ordered/variants"), RedexFileConfig::default())
        .unwrap();
    let a = OrderedAppender::new(f.clone());

    // Inline appends.
    let s0 = a.append_inline(b"abcdefgh").unwrap();
    let s1 = a.append_inline(b"12345678").unwrap();
    assert_eq!(s0, 0);
    assert_eq!(s1, 1);

    // Batch append — contiguous seqs.
    let first = a
        .append_batch(&[
            Bytes::from_static(b"one"),
            Bytes::from_static(b"two"),
            Bytes::from_static(b"three"),
        ])
        .unwrap();
    assert_eq!(first, 2);
    assert_eq!(f.next_seq(), 5);

    let events = f.read_range(0, 5);
    assert!(events[0].entry.is_inline());
    assert_eq!(events[0].payload.as_ref(), b"abcdefgh");
    assert_eq!(events[4].payload.as_ref(), b"three");
}

#[tokio::test]
async fn test_typed_redex_file_roundtrip() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Metric {
        name: String,
        value: u64,
    }

    let r = Redex::new();
    let f = r
        .open_file(&cn("typed/metrics"), RedexFileConfig::default())
        .unwrap();
    let typed: TypedRedexFile<Metric> = TypedRedexFile::new(f.clone());

    for i in 0..50u64 {
        typed
            .append(&Metric {
                name: format!("m-{}", i),
                value: i * 10,
            })
            .unwrap();
    }

    let results = typed.read_range(0, 50);
    assert_eq!(results.len(), 50);
    for (i, result) in results.iter().enumerate() {
        let (seq, m) = result.as_ref().unwrap();
        assert_eq!(*seq, i as u64);
        assert_eq!(m.name, format!("m-{}", i));
        assert_eq!(m.value, (i as u64) * 10);
    }
}

#[tokio::test]
async fn test_typed_redex_file_tail_decode() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Event {
        kind: String,
        payload: Vec<u8>,
    }

    let r = Redex::new();
    let f = r
        .open_file(&cn("typed/tail"), RedexFileConfig::default())
        .unwrap();
    let typed: TypedRedexFile<Event> = TypedRedexFile::new(f.clone());

    let mut stream = Box::pin(typed.tail(0));

    let writer = {
        let typed2 = typed.clone();
        tokio::spawn(async move {
            for i in 0..20u64 {
                typed2
                    .append(&Event {
                        kind: if i % 2 == 0 {
                            "even".into()
                        } else {
                            "odd".into()
                        },
                        payload: vec![i as u8; 4],
                    })
                    .unwrap();
            }
        })
    };

    let mut collected: Vec<(u64, Event)> = Vec::new();
    for _ in 0..20 {
        let (seq, ev) = stream.next().await.unwrap().unwrap();
        collected.push((seq, ev));
    }
    writer.await.unwrap();

    for (i, (seq, ev)) in collected.iter().enumerate() {
        assert_eq!(*seq, i as u64);
        let expected_kind = if i % 2 == 0 { "even" } else { "odd" };
        assert_eq!(ev.kind, expected_kind);
        assert_eq!(ev.payload, vec![i as u8; 4]);
    }
}

#[tokio::test]
async fn test_append_and_fold() {
    // Append an event AND update an in-memory count in one call.
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Inc {
        by: u64,
    }

    let r = Redex::new();
    let f = r
        .open_file(&cn("typed/fold"), RedexFileConfig::default())
        .unwrap();

    let mut total: u64 = 0;
    for i in 1..=5u64 {
        f.append_and_fold(&Inc { by: i }, &mut total, |inc, state| {
            *state += inc.by;
        })
        .unwrap();
    }
    assert_eq!(total, 1 + 2 + 3 + 4 + 5);
    assert_eq!(f.len(), 5);

    // Verify appended events deserialize back correctly.
    let typed: TypedRedexFile<Inc> = TypedRedexFile::new(f.clone());
    let results = typed.read_range(0, 5);
    for (i, result) in results.iter().enumerate() {
        let (_seq, inc) = result.as_ref().unwrap();
        assert_eq!(inc.by, (i + 1) as u64);
    }
}

#[tokio::test]
async fn test_redex_age_based_retention() {
    // Append events, wait past the max_age window, sweep — all
    // expire. Append fresh events afterward — they're retained.
    use std::thread::sleep;
    use std::time::Duration;

    let cfg = RedexFileConfig::default().with_retention_max_age(Duration::from_millis(50));
    let r = Redex::new();
    let f = r.open_file(&cn("retention/age"), cfg).unwrap();

    // Old batch.
    for i in 0..10u64 {
        f.append(format!("old-{}", i).as_bytes()).unwrap();
    }
    assert_eq!(f.len(), 10);

    // Wait past the age window, then append a fresh batch.
    sleep(Duration::from_millis(80));
    for i in 10..15u64 {
        f.append(format!("fresh-{}", i).as_bytes()).unwrap();
    }

    // Sweep: the 10 old entries are past cutoff; the 5 fresh are
    // within the 50 ms window.
    f.sweep_retention();
    assert_eq!(f.len(), 5);

    // Surviving entries are the fresh ones.
    let events = f.read_range(10, 15);
    assert_eq!(events.len(), 5);
    for (i, ev) in events.iter().enumerate() {
        assert_eq!(ev.payload.as_ref(), format!("fresh-{}", 10 + i).as_bytes());
    }
}

#[tokio::test]
async fn test_redex_age_retention_combined_with_count() {
    // Both policies active; the stricter one wins.
    use std::thread::sleep;
    use std::time::Duration;

    let cfg = RedexFileConfig::default()
        .with_retention_max_events(3)
        .with_retention_max_age(Duration::from_millis(50));
    let r = Redex::new();
    let f = r.open_file(&cn("retention/age_count"), cfg).unwrap();

    // Append 5 events in quick succession — all young, but count
    // policy caps at 3.
    for i in 0..5u64 {
        f.append(format!("e{}", i).as_bytes()).unwrap();
    }
    f.sweep_retention();
    assert_eq!(f.len(), 3, "count policy should cap at 3");

    // Wait past age window; sweep again; everything expires.
    sleep(Duration::from_millis(80));
    f.sweep_retention();
    assert_eq!(f.len(), 0, "age policy should drain everything old");
}

#[tokio::test]
async fn test_redex_close_signals_tail() {
    let r = Redex::new();
    let f = r
        .open_file(&cn("closing"), RedexFileConfig::default())
        .unwrap();
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
    let f = r
        .open_file(&cn("fold/smoke"), RedexFileConfig::default())
        .unwrap();
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

#[cfg(feature = "redex-disk")]
mod persistent {
    use super::*;
    use std::path::PathBuf;

    fn tmpdir(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "redex_int_{}_{}_{}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[tokio::test]
    async fn test_persistent_append_close_reopen() {
        let base = tmpdir("reopen");
        let name = cn("durable/basic");
        let cfg = RedexFileConfig::default().with_persistent(true);

        {
            let r = Redex::new().with_persistent_dir(&base);
            let f = r.open_file(&name, cfg).unwrap();
            for i in 0..50u64 {
                f.append(format!("d-{}", i).as_bytes()).unwrap();
            }
            // Close flushes to disk.
            r.close_file(&name).unwrap();
        }

        // New manager, reopen — the index and payloads replay from disk.
        let r2 = Redex::new().with_persistent_dir(&base);
        let f2 = r2.open_file(&name, cfg).unwrap();
        assert_eq!(f2.len(), 50);
        assert_eq!(f2.next_seq(), 50);

        let events = f2.read_range(0, 50);
        assert_eq!(events.len(), 50);
        for (i, ev) in events.iter().enumerate() {
            assert_eq!(ev.entry.seq, i as u64);
            assert_eq!(ev.payload.as_ref(), format!("d-{}", i).as_bytes());
        }

        let _ = std::fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn test_persistent_inline_roundtrip_across_reopen() {
        let base = tmpdir("inline");
        let name = cn("durable/inline");
        let cfg = RedexFileConfig::default().with_persistent(true);

        {
            let r = Redex::new().with_persistent_dir(&base);
            let f = r.open_file(&name, cfg).unwrap();
            for i in 0..100u64 {
                f.append_inline(&i.to_le_bytes()).unwrap();
            }
            r.close_file(&name).unwrap();
        }

        let r2 = Redex::new().with_persistent_dir(&base);
        let f2 = r2.open_file(&name, cfg).unwrap();
        assert_eq!(f2.len(), 100);
        let events = f2.read_range(0, 100);
        for (i, ev) in events.iter().enumerate() {
            assert!(ev.entry.is_inline());
            assert_eq!(ev.payload.as_ref(), &(i as u64).to_le_bytes());
        }

        let _ = std::fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn test_persistent_crash_recovery_drops_without_close() {
        // Crash simulation: drop the handle without close(). Because
        // writes go through the OS page cache (not per-append fsync),
        // an OS-level crash could lose the tail. A plain drop in this
        // process keeps writes in the page cache, which survives a
        // handle drop — so we expect full recovery here.
        let base = tmpdir("crash");
        let name = cn("durable/crash");
        let cfg = RedexFileConfig::default().with_persistent(true);

        {
            let r = Redex::new().with_persistent_dir(&base);
            let f = r.open_file(&name, cfg).unwrap();
            for i in 0..25u64 {
                f.append(format!("c-{}", i).as_bytes()).unwrap();
            }
            // Force fsync before "crash" so we have a durable anchor.
            f.sync().unwrap();
            drop(f);
            drop(r); // no close_file call — simulates crash
        }

        let r2 = Redex::new().with_persistent_dir(&base);
        let f2 = r2.open_file(&name, cfg).unwrap();
        assert_eq!(f2.len(), 25);
        for (i, ev) in f2.read_range(0, 25).iter().enumerate() {
            assert_eq!(ev.entry.seq, i as u64);
            assert_eq!(ev.payload.as_ref(), format!("c-{}", i).as_bytes());
        }

        let _ = std::fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn test_persistent_tail_works_after_reopen() {
        // Tail after reopen should backfill from the persisted index
        // and then pick up new live appends.
        let base = tmpdir("tail");
        let name = cn("durable/tail");
        let cfg = RedexFileConfig::default().with_persistent(true);

        {
            let r = Redex::new().with_persistent_dir(&base);
            let f = r.open_file(&name, cfg).unwrap();
            for i in 0..5u64 {
                f.append(format!("pre-{}", i).as_bytes()).unwrap();
            }
            r.close_file(&name).unwrap();
        }

        let r2 = Redex::new().with_persistent_dir(&base);
        let f2 = r2.open_file(&name, cfg).unwrap();
        let mut stream = Box::pin(f2.tail(0));
        for i in 0..5u64 {
            let ev = stream.next().await.unwrap().unwrap();
            assert_eq!(ev.entry.seq, i);
            assert_eq!(ev.payload.as_ref(), format!("pre-{}", i).as_bytes());
        }

        // Live append after reopen.
        f2.append(b"post").unwrap();
        let ev = stream.next().await.unwrap().unwrap();
        assert_eq!(ev.entry.seq, 5);
        assert_eq!(ev.payload.as_ref(), b"post");

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_persistent_without_base_dir_errors() {
        // Asking for persistent: true on a manager without a base dir
        // must fail at open_file time with a helpful message.
        let r = Redex::new();
        let cfg = RedexFileConfig::default().with_persistent(true);
        let err = r.open_file(&cn("no/basedir"), cfg).unwrap_err();
        assert!(matches!(err, RedexError::Channel(_)));
    }

    #[tokio::test]
    async fn test_persistent_mixed_inline_and_heap_recovery() {
        // Verify inline + heap entries interleaved round-trip correctly
        // through disk persistence and recovery.
        let base = tmpdir("mixed");
        let name = cn("durable/mixed");
        let cfg = RedexFileConfig::default().with_persistent(true);

        {
            let r = Redex::new().with_persistent_dir(&base);
            let f = r.open_file(&name, cfg).unwrap();
            f.append_inline(b"inline01").unwrap();
            f.append(b"this-is-heap-1").unwrap();
            f.append_inline(b"inline02").unwrap();
            f.append(b"this-is-heap-2").unwrap();
            r.close_file(&name).unwrap();
        }

        let r2 = Redex::new().with_persistent_dir(&base);
        let f2 = r2.open_file(&name, cfg).unwrap();
        assert_eq!(f2.len(), 4);
        let events = f2.read_range(0, 4);
        assert!(events[0].entry.is_inline());
        assert_eq!(events[0].payload.as_ref(), b"inline01");
        assert!(!events[1].entry.is_inline());
        assert_eq!(events[1].payload.as_ref(), b"this-is-heap-1");
        assert!(events[2].entry.is_inline());
        assert_eq!(events[2].payload.as_ref(), b"inline02");
        assert!(!events[3].entry.is_inline());
        assert_eq!(events[3].payload.as_ref(), b"this-is-heap-2");

        let _ = std::fs::remove_dir_all(&base);
    }
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
