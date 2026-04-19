//! Integration tests for the CortEX tasks model.
//!
//! Covers the typed `TasksAdapter` surface end-to-end: CRUD through
//! the adapter, queries over materialized state, unknown-id no-ops,
//! multi-producer origin_hash separation, replay after close, and
//! durability with `redex-disk`.

#![cfg(feature = "cortex")]

use futures::StreamExt;
use net::adapter::net::channel::ChannelName;
use net::adapter::net::cortex::tasks::{OrderBy, TaskStatus, TasksAdapter, TASKS_CHANNEL};
use net::adapter::net::cortex::{compute_checksum, EventMeta, EVENT_META_SIZE};
use net::adapter::net::redex::Redex;
#[cfg(feature = "redex-disk")]
use net::adapter::net::redex::RedexFileConfig;

const ORIGIN: u32 = 0xABCD_EF01;

fn now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[tokio::test]
async fn test_full_task_lifecycle() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    let t0 = now_ns();
    let _ = tasks.create(1, "write docs", t0).unwrap();
    let _ = tasks.create(2, "ship adapter", t0 + 1).unwrap();
    let _ = tasks.rename(1, "write better docs", t0 + 2).unwrap();
    let seq = tasks.complete(2, t0 + 3).unwrap();
    tasks.wait_for_seq(seq).await;

    let state = tasks.state();
    let guard = state.read();
    assert_eq!(guard.len(), 2);

    let t1 = guard.get(1).unwrap();
    assert_eq!(t1.title, "write better docs");
    assert_eq!(t1.status, TaskStatus::Pending);
    assert_eq!(t1.created_ns, t0);
    assert_eq!(t1.updated_ns, t0 + 2);

    let t2 = guard.get(2).unwrap();
    assert_eq!(t2.title, "ship adapter");
    assert_eq!(t2.status, TaskStatus::Completed);
    assert_eq!(t2.updated_ns, t0 + 3);

    assert_eq!(guard.pending().count(), 1);
    assert_eq!(guard.completed().count(), 1);
}

#[tokio::test]
async fn test_delete_removes_task() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    tasks.create(1, "temp", 100).unwrap();
    let seq = tasks.delete(1).unwrap();
    tasks.wait_for_seq(seq).await;

    let state = tasks.state();
    let guard = state.read();
    assert!(guard.is_empty());
    assert!(guard.get(1).is_none());
}

#[tokio::test]
async fn test_rename_on_unknown_id_is_noop() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    // Rename before create — fold silently drops (log is the truth).
    let seq = tasks.rename(42, "ghost", 100).unwrap();
    tasks.wait_for_seq(seq).await;

    let state = tasks.state();
    let guard = state.read();
    assert!(guard.is_empty());
}

#[tokio::test]
async fn test_complete_on_unknown_id_is_noop() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    let seq = tasks.complete(99, 100).unwrap();
    tasks.wait_for_seq(seq).await;

    let state = tasks.state();
    let guard = state.read();
    assert!(guard.is_empty());
}

#[tokio::test]
async fn test_replay_after_close_reconstructs_state() {
    // Open → drive CRUD → close → reopen fresh, state replays from log.
    let redex = Redex::new();

    {
        let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();
        tasks.create(1, "a", 100).unwrap();
        tasks.create(2, "b", 101).unwrap();
        tasks.complete(1, 102).unwrap();
        let seq = tasks.rename(2, "b-renamed", 103).unwrap();
        tasks.wait_for_seq(seq).await;
        tasks.close().unwrap();
    }

    // Fresh handle; the Redex manager still owns the file (close on
    // the adapter doesn't drop the file), so reopen replays.
    let tasks2 = TasksAdapter::open(&redex, ORIGIN).unwrap();
    // 4 events were appended → wait for fold to catch up.
    tasks2.wait_for_seq(3).await;

    let state = tasks2.state();
    let guard = state.read();
    assert_eq!(guard.len(), 2);
    assert_eq!(guard.get(1).unwrap().status, TaskStatus::Completed);
    assert_eq!(guard.get(2).unwrap().title, "b-renamed");
    assert_eq!(guard.get(2).unwrap().status, TaskStatus::Pending);
}

#[tokio::test]
async fn test_multi_producer_same_file_different_origins() {
    // Two TasksAdapters against the same RedEX channel, each with its
    // own origin_hash and its own app_seq counter. Both see the same
    // materialized state because they share the underlying file.
    let redex = Redex::new();

    let a = TasksAdapter::open(&redex, 0x0000_0001).unwrap();
    let b = TasksAdapter::open(&redex, 0x0000_0002).unwrap();

    a.create(1, "from-a", 100).unwrap();
    let seq = b.create(2, "from-b", 101).unwrap();
    a.wait_for_seq(seq).await;
    b.wait_for_seq(seq).await;

    let state_a = a.state();
    let state_b = b.state();
    let ga = state_a.read();
    let gb = state_b.read();
    assert_eq!(ga.len(), 2);
    assert_eq!(gb.len(), 2);
}

#[tokio::test]
async fn test_pending_and_completed_queries() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    for i in 1..=10u64 {
        tasks.create(i, format!("task-{}", i), 100 + i).unwrap();
    }
    // Complete the even ids.
    for i in (2..=10u64).step_by(2) {
        tasks.complete(i, 200 + i).unwrap();
    }
    let last = tasks.complete(10, 9999).unwrap(); // idempotent-ish; refreshes updated_ns
    tasks.wait_for_seq(last).await;

    let state = tasks.state();
    let guard = state.read();
    assert_eq!(guard.len(), 10);

    let mut pending_ids: Vec<_> = guard.pending().map(|t| t.id).collect();
    pending_ids.sort();
    assert_eq!(pending_ids, vec![1, 3, 5, 7, 9]);

    let mut completed_ids: Vec<_> = guard.completed().map(|t| t.id).collect();
    completed_ids.sort();
    assert_eq!(completed_ids, vec![2, 4, 6, 8, 10]);
}

#[tokio::test]
async fn test_query_through_live_adapter() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    // Build a mixed corpus.
    for (id, title, now) in [
        (1u64, "alpha", 1000u64),
        (2, "beta", 2000),
        (3, "gamma", 3000),
        (4, "delta", 4000),
        (5, "epsilon", 5000),
    ] {
        tasks.create(id, title, now).unwrap();
    }
    tasks.complete(2, 2500).unwrap();
    tasks.complete(4, 4500).unwrap();
    let last = tasks.rename(5, "EPSILON", 5500).unwrap();
    tasks.wait_for_seq(last).await;

    let state = tasks.state();
    let guard = state.read();

    // Pending only → ids 1, 3, 5.
    let mut pending_ids: Vec<_> = guard
        .query()
        .where_status(TaskStatus::Pending)
        .collect()
        .iter()
        .map(|t| t.id)
        .collect();
    pending_ids.sort();
    assert_eq!(pending_ids, vec![1, 3, 5]);

    // Completed, ordered by updated desc, limit 1 → id 4 (updated_ns 4500).
    let top = guard
        .query()
        .where_status(TaskStatus::Completed)
        .order_by(OrderBy::UpdatedDesc)
        .first()
        .unwrap();
    assert_eq!(top.id, 4);

    // Title contains "psi" (case-insensitive) → id 5 (EPSILON).
    let match_title: Vec<_> = guard
        .query()
        .title_contains("PSI")
        .collect()
        .iter()
        .map(|t| t.id)
        .collect();
    assert_eq!(match_title, vec![5]);

    // created_after(2500) AND pending → id 3, 5.
    let mut recent_pending: Vec<_> = guard
        .query()
        .created_after(2500)
        .where_status(TaskStatus::Pending)
        .collect()
        .iter()
        .map(|t| t.id)
        .collect();
    recent_pending.sort();
    assert_eq!(recent_pending, vec![3, 5]);

    // exists with no match.
    assert!(!guard.query().title_contains("does-not-exist").exists());
    assert!(guard.query().where_status(TaskStatus::Pending).exists());
}

#[tokio::test]
async fn test_watch_initial_emission() {
    // A watcher opened against a non-empty state should yield the
    // current filter result on the first .next().await.
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    // Pre-populate.
    tasks.create(1, "a", 100).unwrap();
    tasks.create(2, "b", 200).unwrap();
    let seq = tasks.complete(2, 250).unwrap();
    tasks.wait_for_seq(seq).await;

    let mut stream = Box::pin(
        tasks
            .watch()
            .where_status(TaskStatus::Pending)
            .order_by(OrderBy::IdAsc)
            .stream(),
    );

    let initial = stream.next().await.unwrap();
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0].id, 1);
}

#[tokio::test]
async fn test_watch_emits_on_relevant_change() {
    // After the initial emission, the stream should yield again when
    // a new event changes the filter result.
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    let mut stream = Box::pin(
        tasks
            .watch()
            .where_status(TaskStatus::Pending)
            .order_by(OrderBy::IdAsc)
            .stream(),
    );

    // Initial: empty.
    let initial = stream.next().await.unwrap();
    assert!(initial.is_empty());

    // Create one pending task → stream should yield [task-1].
    tasks.create(1, "first", 100).unwrap();
    let next = stream.next().await.unwrap();
    assert_eq!(next.len(), 1);
    assert_eq!(next[0].id, 1);

    // Create another pending → [1, 2].
    tasks.create(2, "second", 200).unwrap();
    let next = stream.next().await.unwrap();
    assert_eq!(next.len(), 2);
    assert_eq!(next[0].id, 1);
    assert_eq!(next[1].id, 2);

    // Complete task 1 → no longer matches Pending; result becomes [2].
    tasks.complete(1, 300).unwrap();
    let next = stream.next().await.unwrap();
    assert_eq!(next.len(), 1);
    assert_eq!(next[0].id, 2);
}

#[tokio::test]
async fn test_watch_dedupes_unchanged_results() {
    // Events that advance the log but don't change the filter result
    // must NOT cause a duplicate emission.
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    // Seed one pending + one completed.
    tasks.create(1, "p", 100).unwrap();
    tasks.create(2, "c", 200).unwrap();
    let seq = tasks.complete(2, 250).unwrap();
    tasks.wait_for_seq(seq).await;

    let mut stream = Box::pin(tasks.watch().where_status(TaskStatus::Pending).stream());
    let initial = stream.next().await.unwrap();
    assert_eq!(initial.len(), 1);

    // Append events that DON'T change the pending filter:
    //   - complete on already-completed id 2 (refresh updated_ns, still completed)
    //   - rename on completed id 2 (still completed, filter unaffected)
    tasks.complete(2, 9999).unwrap();
    let seq = tasks.rename(2, "c-renamed", 9999).unwrap();
    tasks.wait_for_seq(seq).await;

    // No duplicate should have fired. Assert the next emission only
    // comes after we do something that DOES change Pending set.
    tasks.create(3, "p2", 300).unwrap();
    let next = stream.next().await.unwrap();
    let ids: Vec<_> = next.iter().map(|t| t.id).collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&3));
    assert_eq!(ids.len(), 2);
}

#[tokio::test]
async fn test_watch_multiple_subscribers_independent() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    let mut pending_stream = Box::pin(tasks.watch().where_status(TaskStatus::Pending).stream());
    let mut completed_stream = Box::pin(tasks.watch().where_status(TaskStatus::Completed).stream());

    // Both get an empty initial emission.
    assert!(pending_stream.next().await.unwrap().is_empty());
    assert!(completed_stream.next().await.unwrap().is_empty());

    // Create → pending gets [1], completed stays empty (no emit).
    tasks.create(1, "x", 100).unwrap();
    let p = pending_stream.next().await.unwrap();
    assert_eq!(p.len(), 1);

    // Complete → pending becomes empty, completed becomes [1].
    tasks.complete(1, 200).unwrap();
    let p = pending_stream.next().await.unwrap();
    assert!(p.is_empty());
    let c = completed_stream.next().await.unwrap();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].id, 1);
}

#[tokio::test]
async fn test_watch_with_limit_and_order() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    let mut stream = Box::pin(
        tasks
            .watch()
            .where_status(TaskStatus::Pending)
            .order_by(OrderBy::CreatedDesc)
            .limit(2)
            .stream(),
    );

    // Initial empty.
    assert!(stream.next().await.unwrap().is_empty());

    for id in 1..=5u64 {
        tasks.create(id, format!("t-{}", id), 100 * id).unwrap();
    }

    // Drain until the result stabilizes at [5, 4] (newest two).
    let mut last: Vec<_> = Vec::new();
    for _ in 0..5 {
        last = stream.next().await.unwrap();
        if last.len() == 2 && last[0].id == 5 && last[1].id == 4 {
            break;
        }
    }
    assert_eq!(last.len(), 2);
    assert_eq!(last[0].id, 5);
    assert_eq!(last[1].id, 4);
}

#[tokio::test]
async fn test_regression_open_from_snapshot_rejects_u64_max_last_seq() {
    // Regression: `open_from_snapshot` used to compute `last_seq + 1`
    // unchecked. A corrupted or malicious snapshot with
    // `last_seq = u64::MAX` would panic in debug, wraparound to 0 in
    // release, and silently resume tailing from seq 0 — replaying
    // the entire log as "new". The fix uses `checked_add` and returns
    // `CortexAdapterError::Redex(Encode)` on overflow.
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();
    let (state_bytes, _) = tasks.snapshot().unwrap();
    tasks.close().unwrap();

    // Fresh Redex to avoid channel re-use interference.
    let redex2 = Redex::new();
    let result = TasksAdapter::open_from_snapshot(&redex2, ORIGIN, &state_bytes, Some(u64::MAX));
    assert!(result.is_err(), "u64::MAX last_seq must be rejected");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("u64::MAX"),
        "error should mention u64::MAX overflow; got: {}",
        msg
    );
}

#[tokio::test]
async fn test_regression_checksum_is_computed_not_zero() {
    // Regression: `EventMeta::checksum` used to be hardcoded to 0 in
    // the tasks adapter's `ingest_typed`. The documented contract
    // (see `EventMeta` struct doc in meta.rs) is xxh3 truncation of
    // the payload tail. Verify the on-disk event's meta.checksum
    // matches `compute_checksum(tail)`.
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    let seq = tasks.create(42, "distinctive title", 12345).unwrap();
    tasks.wait_for_seq(seq).await;

    // Read the raw RedEX event for this append.
    let file = redex
        .open_file(
            &ChannelName::new(TASKS_CHANNEL).unwrap(),
            Default::default(),
        )
        .unwrap();
    let events = file.read_range(0, 1);
    assert_eq!(events.len(), 1, "tasks channel should have one event");
    let payload = &events[0].payload;
    let meta = EventMeta::from_bytes(&payload[..EVENT_META_SIZE]).expect("decode meta");
    let tail = &payload[EVENT_META_SIZE..];

    assert_ne!(meta.checksum, 0, "checksum must not be hardcoded to 0");
    assert_eq!(
        meta.checksum,
        compute_checksum(tail),
        "meta.checksum must match xxh3 truncation of the payload tail"
    );
}

#[tokio::test]
async fn test_regression_watch_without_order_by_is_stable() {
    // Regression for the HashMap-iteration-order false-positive in
    // the watcher's Vec-equality dedup. Before the fix, a watcher
    // opened without `order_by` could emit Vecs whose element order
    // depended on HashMap rehash timing, so a mutation that didn't
    // change the filter output could still trigger a spurious
    // re-emission (element reorder breaks Vec equality). The fix
    // defaults the watcher's `order_by` to `IdAsc` when unset, so
    // the emitted Vec is deterministic and dedup is correct.
    //
    // Seed enough pending tasks that hash iteration order is
    // demonstrably non-ascending, then assert the watch output is
    // IdAsc.
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();
    const N: u64 = 64;
    let mut last = 0;
    for id in 1..=N {
        last = tasks.create(id, format!("t-{}", id), id * 100).unwrap();
    }
    tasks.wait_for_seq(last).await;

    // Open watch *without* order_by. The fix makes this default to
    // IdAsc under the hood.
    let mut stream = Box::pin(tasks.watch().where_status(TaskStatus::Pending).stream());
    let initial = stream.next().await.unwrap();
    assert_eq!(initial.len(), N as usize);
    let ids: Vec<u64> = initial.iter().map(|t| t.id).collect();
    let sorted: Vec<u64> = (1..=N).collect();
    assert_eq!(ids, sorted, "watcher without order_by must emit IdAsc");
}

#[tokio::test]
async fn test_snapshot_and_restore_skips_replay() {
    // Open, do CRUD, snapshot, close. Reopen from snapshot on the
    // SAME redex — state matches without the fold replaying events
    // 0..=last_seq (the adapter tails at FromSeq(last_seq+1)).
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    tasks.create(1, "alpha", 100).unwrap();
    tasks.create(2, "beta", 200).unwrap();
    tasks.complete(1, 150).unwrap();
    let seq = tasks.rename(2, "beta-v2", 250).unwrap();
    tasks.wait_for_seq(seq).await;

    let (bytes, last_seq) = tasks.snapshot().unwrap();
    assert_eq!(last_seq, Some(3)); // 4 events → seq 0..=3
    tasks.close().unwrap();

    // Reopen on the same redex — the file still holds seqs 0..=3,
    // but the restored adapter's fold starts at seq 4 (last_seq+1),
    // so those old events are NOT replayed. State comes from bytes.
    let tasks2 = TasksAdapter::open_from_snapshot(&redex, ORIGIN, &bytes, last_seq).unwrap();

    {
        let state = tasks2.state();
        let guard = state.read();
        assert_eq!(guard.len(), 2);
        let t1 = guard.get(1).unwrap();
        assert_eq!(t1.status, TaskStatus::Completed);
        let t2 = guard.get(2).unwrap();
        assert_eq!(t2.title, "beta-v2");
        assert_eq!(t2.status, TaskStatus::Pending);
    } // guard dropped here before the await below

    // New ingest flows through normally. The underlying file's
    // next_seq is 4 (persisted across close), so this create
    // appends at seq 4, which the fold task picks up since it
    // tails FromSeq(4).
    let seq = tasks2.create(3, "gamma", 300).unwrap();
    assert_eq!(seq, 4);
    tasks2.wait_for_seq(seq).await;
    assert_eq!(tasks2.state().read().len(), 3);
}

#[tokio::test]
async fn test_snapshot_empty_state_has_no_last_seq() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();
    let (bytes, last_seq) = tasks.snapshot().unwrap();
    assert_eq!(last_seq, None);
    assert!(!bytes.is_empty()); // even empty state serializes to >0 bytes.
}

#[tokio::test]
async fn test_ingest_after_close_errors() {
    let redex = Redex::new();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();
    tasks.create(1, "a", 100).unwrap();
    tasks.close().unwrap();
    assert!(tasks.create(2, "b", 101).is_err());
}

#[cfg(feature = "redex-disk")]
#[tokio::test]
async fn test_persistent_tasks_recover_across_processes() {
    use std::path::PathBuf;

    let mut base: PathBuf = std::env::temp_dir();
    base.push(format!(
        "cortex_tasks_persist_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&base).unwrap();

    let cfg = RedexFileConfig::default().with_persistent(true);

    {
        let redex = Redex::new().with_persistent_dir(&base);
        let tasks = TasksAdapter::open_with_config(&redex, ORIGIN, cfg).unwrap();
        tasks.create(1, "durable", 100).unwrap();
        tasks.create(2, "also durable", 101).unwrap();
        let seq = tasks.complete(1, 102).unwrap();
        tasks.wait_for_seq(seq).await;
        tasks.close().unwrap();
    }

    // Fresh Redex manager, same base_dir — state replays from disk.
    let redex2 = Redex::new().with_persistent_dir(&base);
    let tasks2 = TasksAdapter::open_with_config(&redex2, ORIGIN, cfg).unwrap();
    tasks2.wait_for_seq(2).await;

    let state = tasks2.state();
    let guard = state.read();
    assert_eq!(guard.len(), 2);
    assert_eq!(guard.get(1).unwrap().status, TaskStatus::Completed);
    assert_eq!(guard.get(2).unwrap().status, TaskStatus::Pending);

    let _ = std::fs::remove_dir_all(&base);
}
