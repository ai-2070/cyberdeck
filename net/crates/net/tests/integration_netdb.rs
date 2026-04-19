//! Integration tests for NetDB.
//!
//! Covers the unified `NetDb` handle end-to-end: build with multiple
//! models, CRUD through `db.tasks()` / `db.memories()`, filter-based
//! `find_many` / `count_where` / `exists_where`, whole-db snapshot
//! and restore.

#![cfg(feature = "netdb")]

use net::adapter::net::cortex::memories::OrderBy as MemoriesOrderBy;
use net::adapter::net::cortex::tasks::{OrderBy as TasksOrderBy, TaskStatus};
use net::adapter::net::netdb::{MemoriesFilter, NetDb, NetDbSnapshot, TasksFilter};
use net::adapter::net::redex::Redex;

const ORIGIN: u32 = 0xABCD_EF01;

#[tokio::test]
async fn test_netdb_build_with_both_models() {
    let redex = Redex::new();
    let db = NetDb::builder(redex)
        .origin(ORIGIN)
        .with_tasks()
        .with_memories()
        .build()
        .unwrap();

    assert!(db.try_tasks().is_some());
    assert!(db.try_memories().is_some());
}

#[tokio::test]
async fn test_netdb_build_with_only_tasks() {
    let redex = Redex::new();
    let db = NetDb::builder(redex)
        .origin(ORIGIN)
        .with_tasks()
        .build()
        .unwrap();

    assert!(db.try_tasks().is_some());
    assert!(db.try_memories().is_none());
}

#[tokio::test]
async fn test_netdb_crud_through_tasks_handle() {
    let redex = Redex::new();
    let db = NetDb::builder(redex)
        .origin(ORIGIN)
        .with_tasks()
        .build()
        .unwrap();

    let tasks = db.tasks();
    tasks.create(1, "write plan", 100).unwrap();
    tasks.create(2, "ship adapter", 200).unwrap();
    let seq = tasks.complete(1, 150).unwrap();
    tasks.wait_for_seq(seq).await;

    let state = tasks.state();
    let guard = state.read();
    assert_eq!(guard.len(), 2);
    assert_eq!(guard.find_unique(1).unwrap().status, TaskStatus::Completed);
}

#[tokio::test]
async fn test_netdb_find_many_on_tasks_state() {
    let redex = Redex::new();
    let db = NetDb::builder(redex)
        .origin(ORIGIN)
        .with_tasks()
        .build()
        .unwrap();

    for i in 1..=10u64 {
        db.tasks().create(i, format!("t-{}", i), 100 * i).unwrap();
    }
    // Complete the even ids.
    for i in (2..=10u64).step_by(2) {
        db.tasks().complete(i, 1000 + i).unwrap();
    }
    let last = db.tasks().complete(10, 9999).unwrap();
    db.tasks().wait_for_seq(last).await;

    let state = db.tasks().state();
    let guard = state.read();

    // Pending tasks, ordered by id.
    let filter = TasksFilter {
        status: Some(TaskStatus::Pending),
        order_by: Some(TasksOrderBy::IdAsc),
        ..Default::default()
    };
    let pending = guard.find_many(&filter);
    let ids: Vec<_> = pending.iter().map(|t| t.id).collect();
    assert_eq!(ids, vec![1, 3, 5, 7, 9]);

    // Count via filter.
    assert_eq!(guard.count_where(&filter), 5);

    // Exists.
    assert!(guard.exists_where(&filter));

    // Completed tasks, limit 2, ordered by updated desc.
    let completed_filter = TasksFilter {
        status: Some(TaskStatus::Completed),
        order_by: Some(TasksOrderBy::UpdatedDesc),
        limit: Some(2),
        ..Default::default()
    };
    let completed = guard.find_many(&completed_filter);
    assert_eq!(completed.len(), 2);
    // id=10 had its updated_ns bumped twice (latest = 9999).
    assert_eq!(completed[0].id, 10);
}

#[tokio::test]
async fn test_netdb_find_many_on_memories_state() {
    let redex = Redex::new();
    let db = NetDb::builder(redex)
        .origin(ORIGIN)
        .with_memories()
        .build()
        .unwrap();

    db.memories()
        .store(
            1,
            "meeting notes",
            vec!["work".into(), "notes".into()],
            "alice",
            100,
        )
        .unwrap();
    db.memories()
        .store(2, "grocery list", vec!["personal".into()], "alice", 200)
        .unwrap();
    db.memories()
        .store(
            3,
            "api design",
            vec!["work".into(), "design".into()],
            "bob",
            300,
        )
        .unwrap();
    let seq = db.memories().pin(1, 310).unwrap();
    db.memories().wait_for_seq(seq).await;

    let state = db.memories().state();
    let guard = state.read();

    // Tag filter → any memory with "work".
    let work_filter = MemoriesFilter {
        tag: Some("work".into()),
        order_by: Some(MemoriesOrderBy::IdAsc),
        ..Default::default()
    };
    let work_memories = guard.find_many(&work_filter);
    let ids: Vec<_> = work_memories.iter().map(|m| m.id).collect();
    assert_eq!(ids, vec![1, 3]);

    // Pinned filter.
    let pinned_filter = MemoriesFilter {
        pinned: Some(true),
        ..Default::default()
    };
    assert_eq!(guard.count_where(&pinned_filter), 1);
    assert!(guard.exists_where(&pinned_filter));

    // Source filter + content search combined.
    let combo = MemoriesFilter {
        source: Some("bob".into()),
        content_contains: Some("API".into()),
        ..Default::default()
    };
    let results = guard.find_many(&combo);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, 3);
}

#[tokio::test]
async fn test_netdb_whole_snapshot_and_restore() {
    // Fill both models in one NetDb, snapshot, encode, decode,
    // rebuild a FRESH NetDb from the decoded snapshot, and verify
    // both models' state matches.
    //
    // Each NetDb owns its Redex so they don't share underlying
    // files — state comes entirely from the snapshot blob. Continued
    // ingest on the restored db is tested separately at the tasks/
    // memories layer (same-file continuation is already covered by
    // integration_cortex_{tasks,memories}).
    let snapshot_blob: Vec<u8>;

    {
        let db = NetDb::builder(Redex::new())
            .origin(ORIGIN)
            .with_tasks()
            .with_memories()
            .build()
            .unwrap();

        db.tasks().create(1, "alpha", 100).unwrap();
        db.tasks().create(2, "beta", 200).unwrap();
        let t_seq = db.tasks().complete(1, 150).unwrap();

        db.memories()
            .store(1, "hello", vec!["x".into()], "alice", 100)
            .unwrap();
        let m_seq = db.memories().pin(1, 110).unwrap();

        db.tasks().wait_for_seq(t_seq).await;
        db.memories().wait_for_seq(m_seq).await;

        let snapshot = db.snapshot().unwrap();
        assert!(snapshot.tasks.is_some());
        assert!(snapshot.memories.is_some());
        snapshot_blob = snapshot.encode().unwrap();
        db.close().unwrap();
    }

    // Decode the blob and rebuild against a fresh Redex.
    let restored = NetDbSnapshot::decode(&snapshot_blob).unwrap();
    let db2 = NetDb::builder(Redex::new())
        .origin(ORIGIN)
        .with_tasks()
        .with_memories()
        .build_from_snapshot(&restored)
        .unwrap();

    // Tasks state restored.
    let t_state = db2.tasks().state();
    let t_guard = t_state.read();
    assert_eq!(t_guard.len(), 2);
    assert_eq!(
        t_guard.find_unique(1).unwrap().status,
        TaskStatus::Completed
    );
    assert_eq!(t_guard.find_unique(2).unwrap().title, "beta");
    drop(t_guard);

    // Memories state restored.
    let m_state = db2.memories().state();
    let m_guard = m_state.read();
    assert_eq!(m_guard.len(), 1);
    assert!(m_guard.find_unique(1).unwrap().pinned);
    assert_eq!(m_guard.find_unique(1).unwrap().content, "hello");
}

#[tokio::test]
async fn test_netdb_build_from_empty_snapshot_is_fresh_open() {
    // A model listed in `with_*()` but with None in the snapshot is
    // opened from scratch — equivalent to build() for that model.
    let empty = NetDbSnapshot {
        tasks: None,
        memories: None,
    };
    let db = NetDb::builder(Redex::new())
        .origin(ORIGIN)
        .with_tasks()
        .with_memories()
        .build_from_snapshot(&empty)
        .unwrap();
    assert_eq!(db.tasks().count(), 0);
    assert_eq!(db.memories().count(), 0);
}

#[tokio::test]
async fn test_netdb_close_is_idempotent() {
    let redex = Redex::new();
    let db = NetDb::builder(redex)
        .origin(ORIGIN)
        .with_tasks()
        .with_memories()
        .build()
        .unwrap();

    db.close().unwrap();
    db.close().unwrap(); // idempotent
}

#[tokio::test]
#[should_panic(expected = "tasks not enabled")]
async fn test_netdb_tasks_without_with_tasks_panics() {
    let redex = Redex::new();
    let db = NetDb::builder(redex)
        .origin(ORIGIN)
        .with_memories()
        .build()
        .unwrap();
    // Should panic — tasks weren't enabled.
    let _ = db.tasks();
}

#[tokio::test]
async fn test_regression_build_from_snapshot_error_path_is_clean() {
    // Regression: `build_from_snapshot` used to open the tasks
    // adapter, then open memories — if memories failed (e.g. corrupt
    // snapshot bytes), the tasks adapter's fold task would outlive
    // the failed build as an orphan. The runtime fix closes the
    // first adapter before propagating the error (see
    // [`NetDbBuilder::build_from_snapshot`] for the code-level
    // guarantee).
    //
    // This test exercises the error path — corrupt memories bytes
    // must surface as `Err` and a fresh NetDb built afterward must
    // ingest cleanly. It does NOT directly observe the closed
    // first-adapter's fold task on the failing Redex, because
    // `build_from_snapshot` consumes the Redex by value and drops
    // it on the error path — without an `Arc`-backed `Redex`
    // handle the failed manager is unreachable from outside. The
    // atomicity guarantee itself is kept honest by the six-line
    // close-on-error block in the builder; this test protects
    // against regressions in the observable surface only.
    let redex = Redex::new();

    let corrupt_bundle = NetDbSnapshot {
        tasks: None,
        memories: Some((vec![0xFFu8; 32], Some(0))),
    };

    let first = NetDb::builder(redex)
        .origin(ORIGIN)
        .with_tasks()
        .with_memories()
        .build_from_snapshot(&corrupt_bundle);
    assert!(
        first.is_err(),
        "corrupt memories snapshot must cause build to fail"
    );

    let redex2 = Redex::new();
    let db = NetDb::builder(redex2)
        .origin(ORIGIN)
        .with_tasks()
        .with_memories()
        .build()
        .unwrap();
    assert!(db.try_tasks().is_some());
    assert!(db.try_memories().is_some());
    // Smoke ingest to prove the fresh handle is functional after a
    // prior failed build in the same test scope.
    let seq = db.tasks().create(1, "t", 100).unwrap();
    db.tasks().wait_for_seq(seq).await;
    assert_eq!(db.tasks().state().read().len(), 1);
    db.close().unwrap();
}
