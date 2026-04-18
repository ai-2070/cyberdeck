//! Integration tests for the CortEX tasks model.
//!
//! Covers the typed `TasksAdapter` surface end-to-end: CRUD through
//! the adapter, queries over materialized state, unknown-id no-ops,
//! multi-producer origin_hash separation, replay after close, and
//! durability with `redex-disk`.

#![cfg(feature = "cortex-tasks")]

use net::adapter::net::cortex::tasks::{OrderBy, TaskStatus, TasksAdapter};
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
