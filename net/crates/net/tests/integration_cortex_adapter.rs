//! Integration tests for the CortEX adapter core.
//!
//! Covers the adapter boundary end-to-end: project envelope → RedEX
//! append → tail → fold → state. Focus areas:
//!
//! - read-after-write (`wait_for_seq` semantics).
//! - Replay via `StartPosition::FromBeginning` after a state reset.
//! - `FromSeq(k)` checkpoint resume.
//! - `LiveOnly` skips pre-open events.
//! - Mixed dispatches routed by fold impl.
//! - Ingest after close rejected; state remains readable.
//! - Ordering preserved across large bursts.

#![cfg(feature = "cortex-adapter")]

use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use net::adapter::net::channel::ChannelName;
use net::adapter::net::cortex::{
    CortexAdapter, CortexAdapterConfig, EventEnvelope, EventMeta, StartPosition,
};
use net::adapter::net::redex::{Redex, RedexError, RedexEvent, RedexFileConfig, RedexFold};

fn cn(s: &str) -> ChannelName {
    ChannelName::new(s).unwrap()
}

/// Fold that records every event's (dispatch, seq_or_ts) pair.
#[derive(Default)]
struct RecorderState {
    seen: Vec<(u8, u64)>,
}

struct RecorderFold;

impl RedexFold<RecorderState> for RecorderFold {
    fn apply(&mut self, ev: &RedexEvent, state: &mut RecorderState) -> Result<(), RedexError> {
        let meta = EventMeta::from_bytes(&ev.payload[..20])
            .ok_or_else(|| RedexError::Encode("bad EventMeta".into()))?;
        state.seen.push((meta.dispatch, meta.seq_or_ts));
        Ok(())
    }
}

fn mk_env(dispatch: u8, seq_or_ts: u64, tail: &[u8]) -> EventEnvelope {
    let meta = EventMeta::new(dispatch, 0, 0xAB, seq_or_ts, 0);
    EventEnvelope::new(meta, Bytes::copy_from_slice(tail))
}

#[tokio::test]
async fn test_read_after_write() {
    let redex = Redex::new();
    let adapter = CortexAdapter::<RecorderState>::open(
        &redex,
        &cn("cortex/raw"),
        RedexFileConfig::default(),
        CortexAdapterConfig::default(),
        RecorderFold,
        RecorderState::default(),
    )
    .unwrap();

    for i in 0..50u64 {
        let seq = adapter.ingest(mk_env(1, i, b"")).unwrap();
        adapter.wait_for_seq(seq).await;
    }

    let state_handle = adapter.state();
    let guard = state_handle.read();
    assert_eq!(guard.seen.len(), 50);
    for (i, (dispatch, app_seq)) in guard.seen.iter().enumerate() {
        assert_eq!(*dispatch, 1);
        assert_eq!(*app_seq, i as u64);
    }
}

#[tokio::test]
async fn test_replay_from_beginning_rebuilds_state() {
    let redex = Redex::new();

    // Fill the file via a throwaway adapter.
    {
        let a = CortexAdapter::<RecorderState>::open(
            &redex,
            &cn("cortex/replay"),
            RedexFileConfig::default(),
            CortexAdapterConfig::default(),
            RecorderFold,
            RecorderState::default(),
        )
        .unwrap();
        for i in 0..25u64 {
            let seq = a.ingest(mk_env(2, i, b"")).unwrap();
            a.wait_for_seq(seq).await;
        }
        a.close().unwrap();
    }

    // Fresh adapter reopens the same file with empty initial state;
    // fold replays from seq 0.
    let a2 = CortexAdapter::<RecorderState>::open(
        &redex,
        &cn("cortex/replay"),
        RedexFileConfig::default(),
        CortexAdapterConfig::default(),
        RecorderFold,
        RecorderState::default(),
    )
    .unwrap();
    // Wait for the fold to catch up to the last event.
    a2.wait_for_seq(24).await;
    assert_eq!(a2.state().read().seen.len(), 25);
}

#[tokio::test]
async fn test_from_seq_checkpoint_resume() {
    let redex = Redex::new();
    let a = CortexAdapter::<RecorderState>::open(
        &redex,
        &cn("cortex/checkpoint"),
        RedexFileConfig::default(),
        CortexAdapterConfig::default(),
        RecorderFold,
        RecorderState::default(),
    )
    .unwrap();
    for i in 0..10u64 {
        let seq = a.ingest(mk_env(3, i, b"")).unwrap();
        a.wait_for_seq(seq).await;
    }
    a.close().unwrap();

    // Simulate a caller that has persisted state through seq 4 and
    // rehydrates from an external snapshot. Resume at seq 5.
    let mut rehydrated = RecorderState::default();
    for i in 0..5u64 {
        rehydrated.seen.push((3, i));
    }

    let a2 = CortexAdapter::<RecorderState>::open(
        &redex,
        &cn("cortex/checkpoint"),
        RedexFileConfig::default(),
        CortexAdapterConfig::new().with_start(StartPosition::FromSeq(5)),
        RecorderFold,
        rehydrated,
    )
    .unwrap();
    a2.wait_for_seq(9).await;

    let state = a2.state();
    let guard = state.read();
    assert_eq!(guard.seen.len(), 10);
    // 0..5 from rehydration, 5..10 from replay.
    for (i, (_d, app_seq)) in guard.seen.iter().enumerate() {
        assert_eq!(*app_seq, i as u64);
    }
}

#[tokio::test]
async fn test_live_only_skips_pre_open_events() {
    let redex = Redex::new();
    // Pre-populate the file.
    {
        let a = CortexAdapter::<RecorderState>::open(
            &redex,
            &cn("cortex/liveonly"),
            RedexFileConfig::default(),
            CortexAdapterConfig::default(),
            RecorderFold,
            RecorderState::default(),
        )
        .unwrap();
        for i in 0..10u64 {
            let seq = a.ingest(mk_env(5, i, b"")).unwrap();
            a.wait_for_seq(seq).await;
        }
        a.close().unwrap();
    }

    // Reopen with LiveOnly; fresh state should stay empty until new
    // ingests arrive.
    let a2 = CortexAdapter::<RecorderState>::open(
        &redex,
        &cn("cortex/liveonly"),
        RedexFileConfig::default(),
        CortexAdapterConfig::new().with_start(StartPosition::LiveOnly),
        RecorderFold,
        RecorderState::default(),
    )
    .unwrap();

    // Give the fold task a moment to observe that there's nothing to
    // backfill (in practice, open() already set folded_through_seq to
    // next_seq - 1). Read state: empty.
    assert_eq!(a2.state().read().seen.len(), 0);

    // Append 3 more — these arrive live.
    for i in 100..103u64 {
        let seq = a2.ingest(mk_env(5, i, b"")).unwrap();
        a2.wait_for_seq(seq).await;
    }

    let state = a2.state();
    let guard = state.read();
    assert_eq!(guard.seen.len(), 3);
    assert_eq!(guard.seen[0].1, 100);
    assert_eq!(guard.seen[2].1, 102);
}

/// Tasks-style fold — toy but demonstrates dispatch routing.
#[derive(Default)]
struct TaskState {
    tasks: HashMap<u64, String>,
}

const DISPATCH_TASK_CREATED: u8 = 0x80;
const DISPATCH_TASK_RENAMED: u8 = 0x81;
const DISPATCH_TASK_DELETED: u8 = 0x82;

struct TaskFold;

impl RedexFold<TaskState> for TaskFold {
    fn apply(&mut self, ev: &RedexEvent, state: &mut TaskState) -> Result<(), RedexError> {
        let meta = EventMeta::from_bytes(&ev.payload[..20])
            .ok_or_else(|| RedexError::Encode("bad EventMeta".into()))?;
        let tail = &ev.payload[20..];
        let id = meta.seq_or_ts;
        match meta.dispatch {
            DISPATCH_TASK_CREATED | DISPATCH_TASK_RENAMED => {
                state.tasks.insert(id, String::from_utf8_lossy(tail).into_owned());
            }
            DISPATCH_TASK_DELETED => {
                state.tasks.remove(&id);
            }
            _ => {}
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_mixed_dispatch_routing() {
    let redex = Redex::new();
    let adapter = CortexAdapter::<TaskState>::open(
        &redex,
        &cn("cortex/tasks"),
        RedexFileConfig::default(),
        CortexAdapterConfig::default(),
        TaskFold,
        TaskState::default(),
    )
    .unwrap();

    adapter.ingest(mk_env(DISPATCH_TASK_CREATED, 1, b"first")).unwrap();
    adapter.ingest(mk_env(DISPATCH_TASK_CREATED, 2, b"second")).unwrap();
    adapter.ingest(mk_env(DISPATCH_TASK_RENAMED, 1, b"first-renamed")).unwrap();
    let seq = adapter.ingest(mk_env(DISPATCH_TASK_DELETED, 2, b"")).unwrap();
    adapter.wait_for_seq(seq).await;

    let state = adapter.state();
    let guard = state.read();
    assert_eq!(guard.tasks.len(), 1);
    assert_eq!(guard.tasks.get(&1).map(|s| s.as_str()), Some("first-renamed"));
    assert!(!guard.tasks.contains_key(&2));
}

#[tokio::test]
async fn test_ingest_after_close_errors() {
    let redex = Redex::new();
    let adapter = CortexAdapter::<RecorderState>::open(
        &redex,
        &cn("cortex/close-ingest"),
        RedexFileConfig::default(),
        CortexAdapterConfig::default(),
        RecorderFold,
        RecorderState::default(),
    )
    .unwrap();

    let seq = adapter.ingest(mk_env(0, 0, b"")).unwrap();
    adapter.wait_for_seq(seq).await;
    adapter.close().unwrap();

    // State handle survives close.
    assert_eq!(adapter.state().read().seen.len(), 1);

    // Ingest after close returns Closed.
    assert!(adapter.ingest(mk_env(0, 1, b"")).is_err());
}

#[tokio::test]
async fn test_burst_ordering_preserved() {
    let redex = Redex::new();
    let adapter = CortexAdapter::<RecorderState>::open(
        &redex,
        &cn("cortex/burst"),
        RedexFileConfig::default(),
        CortexAdapterConfig::default(),
        RecorderFold,
        RecorderState::default(),
    )
    .unwrap();

    // Rapid-fire 500 ingests without waiting between each.
    let mut last_seq = 0;
    for i in 0..500u64 {
        last_seq = adapter.ingest(mk_env(7, i, b"")).unwrap();
    }
    adapter.wait_for_seq(last_seq).await;

    let state = adapter.state();
    let guard = state.read();
    assert_eq!(guard.seen.len(), 500);
    for (i, (_d, app_seq)) in guard.seen.iter().enumerate() {
        assert_eq!(
            *app_seq, i as u64,
            "event {} arrived out of order or missing",
            i
        );
    }
}

#[tokio::test]
async fn test_state_handle_is_shared_arc() {
    // Verify that the Arc returned by state() is the same one the
    // fold task writes to.
    let redex = Redex::new();
    let adapter = CortexAdapter::<RecorderState>::open(
        &redex,
        &cn("cortex/shared"),
        RedexFileConfig::default(),
        CortexAdapterConfig::default(),
        RecorderFold,
        RecorderState::default(),
    )
    .unwrap();

    let handle_a: Arc<_> = adapter.state();
    let handle_b: Arc<_> = adapter.state();
    assert!(Arc::ptr_eq(&handle_a, &handle_b));

    let seq = adapter.ingest(mk_env(0, 42, b"")).unwrap();
    adapter.wait_for_seq(seq).await;

    // Writing through the fold is visible on both handles.
    assert_eq!(handle_a.read().seen.len(), 1);
    assert_eq!(handle_b.read().seen.len(), 1);
}
