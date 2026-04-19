//! Integration tests for the CortEX memories model.
//!
//! Covers the typed `MemoriesAdapter` surface end-to-end: full
//! lifecycle, tag-based queries through the live adapter, unknown-id
//! no-ops, replay after close, and coexistence with the tasks model
//! on the same Redex manager.

#![cfg(feature = "cortex")]

use futures::StreamExt;
use net::adapter::net::cortex::memories::{MemoriesAdapter, OrderBy};
use net::adapter::net::redex::Redex;

const ORIGIN: u32 = 0x0BAD_F00D;

#[tokio::test]
async fn test_full_memory_lifecycle() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    memories
        .store(
            1,
            "notes from the standup",
            vec!["work".into(), "notes".into()],
            "alice",
            100,
        )
        .unwrap();
    memories
        .store(
            2,
            "grocery list for the week",
            vec!["personal".into(), "todo".into()],
            "alice",
            200,
        )
        .unwrap();
    memories
        .retag(2, vec!["personal".into(), "shopping".into()], 250)
        .unwrap();
    memories.pin(1, 260).unwrap();
    let seq = memories.pin(2, 270).unwrap();
    memories.wait_for_seq(seq).await;

    let state = memories.state();
    let guard = state.read();
    assert_eq!(guard.len(), 2);

    let m1 = guard.get(1).unwrap();
    assert!(m1.pinned);
    assert_eq!(m1.tags, vec!["work".to_string(), "notes".to_string()]);
    assert_eq!(m1.created_ns, 100);
    assert_eq!(m1.updated_ns, 260);

    let m2 = guard.get(2).unwrap();
    assert!(m2.pinned);
    assert_eq!(
        m2.tags,
        vec!["personal".to_string(), "shopping".to_string()]
    );
    assert_eq!(m2.updated_ns, 270);
}

#[tokio::test]
async fn test_pin_and_unpin_toggle() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    memories
        .store(1, "toggle me", Vec::<String>::new(), "tester", 100)
        .unwrap();
    memories.pin(1, 110).unwrap();
    let seq = memories.pin(1, 120).unwrap();
    memories.wait_for_seq(seq).await;
    assert!(memories.state().read().get(1).unwrap().pinned);

    let seq = memories.unpin(1, 130).unwrap();
    memories.wait_for_seq(seq).await;
    assert!(!memories.state().read().get(1).unwrap().pinned);
}

#[tokio::test]
async fn test_delete_removes_memory() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    memories
        .store(1, "temp", Vec::<String>::new(), "alice", 100)
        .unwrap();
    let seq = memories.delete(1).unwrap();
    memories.wait_for_seq(seq).await;

    let state = memories.state();
    let guard = state.read();
    assert!(guard.is_empty());
}

#[tokio::test]
async fn test_retag_on_unknown_id_is_noop() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    let seq = memories.retag(999, vec!["ghost".into()], 100).unwrap();
    memories.wait_for_seq(seq).await;

    let state = memories.state();
    let guard = state.read();
    assert!(guard.is_empty());
}

#[tokio::test]
async fn test_tag_queries_through_live_adapter() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    memories
        .store(
            1,
            "morning standup",
            vec!["work".into(), "meetings".into()],
            "alice",
            100,
        )
        .unwrap();
    memories
        .store(
            2,
            "reading list",
            vec!["personal".into(), "reading".into()],
            "alice",
            200,
        )
        .unwrap();
    memories
        .store(
            3,
            "api design session",
            vec!["work".into(), "design".into()],
            "bob",
            300,
        )
        .unwrap();
    let seq = memories
        .store(
            4,
            "book recommendations",
            vec!["personal".into(), "reading".into(), "books".into()],
            "bob",
            400,
        )
        .unwrap();
    memories.wait_for_seq(seq).await;

    let state = memories.state();
    let guard = state.read();

    // where_tag("work") → 1, 3.
    let mut work_ids: Vec<_> = guard
        .query()
        .where_tag("work")
        .collect()
        .iter()
        .map(|m| m.id)
        .collect();
    work_ids.sort();
    assert_eq!(work_ids, vec![1, 3]);

    // where_any_tag({books, design}) → 3 (design), 4 (books).
    let mut any_ids: Vec<_> = guard
        .query()
        .where_any_tag(["books".into(), "design".into()])
        .collect()
        .iter()
        .map(|m| m.id)
        .collect();
    any_ids.sort();
    assert_eq!(any_ids, vec![3, 4]);

    // where_all_tags({personal, reading}) → 2, 4.
    let mut all_ids: Vec<_> = guard
        .query()
        .where_all_tags(["personal".into(), "reading".into()])
        .collect()
        .iter()
        .map(|m| m.id)
        .collect();
    all_ids.sort();
    assert_eq!(all_ids, vec![2, 4]);

    // where_source("bob") → 3, 4.
    let mut bob_ids: Vec<_> = guard
        .query()
        .where_source("bob")
        .collect()
        .iter()
        .map(|m| m.id)
        .collect();
    bob_ids.sort();
    assert_eq!(bob_ids, vec![3, 4]);

    // content_contains("api") → 3.
    let ids: Vec<_> = guard
        .query()
        .content_contains("API")
        .collect()
        .iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(ids, vec![3]);

    // Composed: where_source=bob AND where_tag=reading → only 4.
    let ids: Vec<_> = guard
        .query()
        .where_source("bob")
        .where_tag("reading")
        .collect()
        .iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(ids, vec![4]);

    // Order by CreatedDesc, limit 2 → 4, 3.
    let ids: Vec<_> = guard
        .query()
        .order_by(OrderBy::CreatedDesc)
        .limit(2)
        .collect()
        .iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(ids, vec![4, 3]);
}

#[tokio::test]
async fn test_replay_after_close_reconstructs_state() {
    let redex = Redex::new();
    {
        let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();
        memories
            .store(1, "alpha", vec!["x".into()], "alice", 100)
            .unwrap();
        memories.pin(1, 110).unwrap();
        memories
            .store(2, "beta", vec!["y".into()], "alice", 200)
            .unwrap();
        let seq = memories
            .retag(2, vec!["y".into(), "z".into()], 210)
            .unwrap();
        memories.wait_for_seq(seq).await;
        memories.close().unwrap();
    }

    // Fresh adapter, same file — state replays from log.
    let memories2 = MemoriesAdapter::open(&redex, ORIGIN).unwrap();
    memories2.wait_for_seq(3).await;

    let state = memories2.state();
    let guard = state.read();
    assert_eq!(guard.len(), 2);

    let m1 = guard.get(1).unwrap();
    assert!(m1.pinned);
    assert_eq!(m1.tags, vec!["x".to_string()]);

    let m2 = guard.get(2).unwrap();
    assert!(!m2.pinned);
    assert_eq!(m2.tags, vec!["y".to_string(), "z".to_string()]);
}

#[tokio::test]
async fn test_watch_initial_emission() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    // Pre-populate with one pinned + one unpinned.
    memories
        .store(1, "pinned content", vec!["urgent".into()], "alice", 100)
        .unwrap();
    memories
        .store(2, "other content", vec!["later".into()], "alice", 200)
        .unwrap();
    let seq = memories.pin(1, 210).unwrap();
    memories.wait_for_seq(seq).await;

    let mut stream = Box::pin(
        memories
            .watch()
            .where_pinned(true)
            .order_by(OrderBy::IdAsc)
            .stream(),
    );

    let initial = stream.next().await.unwrap();
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0].id, 1);
}

#[tokio::test]
async fn test_watch_emits_on_tag_change() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    // Watch memories tagged "urgent".
    let mut stream = Box::pin(
        memories
            .watch()
            .where_tag("urgent")
            .order_by(OrderBy::IdAsc)
            .stream(),
    );
    let initial = stream.next().await.unwrap();
    assert!(initial.is_empty());

    // Store a memory without the "urgent" tag → no emission expected
    // (the next tagged store should produce the next emission, not
    // this irrelevant one).
    memories
        .store(1, "routine", vec!["later".into()], "alice", 100)
        .unwrap();

    // Store a matching memory → emission [1] where id=2 and has tag.
    memories
        .store(
            2,
            "fire in the datacenter",
            vec!["urgent".into()],
            "alice",
            200,
        )
        .unwrap();
    let next = stream.next().await.unwrap();
    assert_eq!(next.len(), 1);
    assert_eq!(next[0].id, 2);

    // Retag #1 to include "urgent" → now it matches; emission [1, 2].
    memories
        .retag(1, vec!["later".into(), "urgent".into()], 300)
        .unwrap();
    let next = stream.next().await.unwrap();
    let mut ids: Vec<_> = next.iter().map(|m| m.id).collect();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);

    // Retag #2 to drop "urgent" → drops out of filter; emission [1].
    memories.retag(2, vec!["resolved".into()], 400).unwrap();
    let next = stream.next().await.unwrap();
    assert_eq!(next.len(), 1);
    assert_eq!(next[0].id, 1);
}

#[tokio::test]
async fn test_watch_dedupes_unchanged_results() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    // Seed one memory tagged "work" + one tagged "home".
    memories
        .store(1, "work note", vec!["work".into()], "alice", 100)
        .unwrap();
    memories
        .store(2, "home note", vec!["home".into()], "alice", 200)
        .unwrap();
    let seq = memories.pin(2, 210).unwrap();
    memories.wait_for_seq(seq).await;

    let mut stream = Box::pin(memories.watch().where_tag("work").stream());
    let initial = stream.next().await.unwrap();
    assert_eq!(initial.len(), 1);

    // Changes that DON'T affect the "work"-tagged set:
    //   - pin/unpin the home memory
    //   - retag the home memory (still tagged home)
    memories.unpin(2, 300).unwrap();
    memories.pin(2, 310).unwrap();
    memories
        .retag(2, vec!["home".into(), "archive".into()], 320)
        .unwrap();

    // Now make a change that DOES affect the "work" set: store a new
    // work-tagged memory.
    memories
        .store(3, "another work note", vec!["work".into()], "alice", 400)
        .unwrap();

    let next = stream.next().await.unwrap();
    let mut ids: Vec<_> = next.iter().map(|m| m.id).collect();
    ids.sort();
    assert_eq!(ids, vec![1, 3]);
}

#[tokio::test]
async fn test_watch_multiple_subscribers_independent() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    let mut pinned_stream = Box::pin(memories.watch().where_pinned(true).stream());
    let mut tagged_stream = Box::pin(memories.watch().where_tag("flagged").stream());

    // Both emit empty initial.
    assert!(pinned_stream.next().await.unwrap().is_empty());
    assert!(tagged_stream.next().await.unwrap().is_empty());

    // Store a memory with tag "flagged" but not pinned.
    memories
        .store(1, "flagged mem", vec!["flagged".into()], "alice", 100)
        .unwrap();

    // tagged_stream yields [1]; pinned_stream stays empty (no emit).
    let t = tagged_stream.next().await.unwrap();
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].id, 1);

    // Pin it. Now pinned_stream yields [1]; tagged_stream stays
    // unchanged (still tagged flagged, no re-emission needed).
    memories.pin(1, 200).unwrap();
    let p = pinned_stream.next().await.unwrap();
    assert_eq!(p.len(), 1);
}

#[tokio::test]
async fn test_watch_with_limit_and_order() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    let mut stream = Box::pin(
        memories
            .watch()
            .where_pinned(true)
            .order_by(OrderBy::CreatedDesc)
            .limit(2)
            .stream(),
    );
    assert!(stream.next().await.unwrap().is_empty());

    for id in 1..=5u64 {
        memories
            .store(
                id,
                format!("m-{}", id),
                Vec::<String>::new(),
                "alice",
                100 * id,
            )
            .unwrap();
        memories.pin(id, 100 * id + 1).unwrap();
    }

    // Drain until we see the top-2 newest pinned: ids 5, 4.
    let mut last: Vec<_> = Vec::new();
    for _ in 0..20 {
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
async fn test_snapshot_and_restore_round_trip() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();

    memories
        .store(1, "alpha", vec!["x".into()], "alice", 100)
        .unwrap();
    memories.pin(1, 110).unwrap();
    memories
        .store(2, "beta", vec!["y".into()], "alice", 200)
        .unwrap();
    let seq = memories
        .retag(2, vec!["y".into(), "z".into()], 210)
        .unwrap();
    memories.wait_for_seq(seq).await;

    let (bytes, last_seq) = memories.snapshot().unwrap();
    assert_eq!(last_seq, Some(3));
    memories.close().unwrap();

    // Restore on a fresh Redex.
    let redex2 = Redex::new();
    let memories2 = MemoriesAdapter::open_from_snapshot(&redex2, ORIGIN, &bytes, last_seq).unwrap();
    let state = memories2.state();
    let guard = state.read();
    assert_eq!(guard.len(), 2);
    let m1 = guard.get(1).unwrap();
    assert!(m1.pinned);
    let m2 = guard.get(2).unwrap();
    assert_eq!(m2.tags, vec!["y".to_string(), "z".to_string()]);
}

#[tokio::test]
async fn test_ingest_after_close_errors() {
    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();
    memories
        .store(1, "before close", Vec::<String>::new(), "alice", 100)
        .unwrap();
    memories.close().unwrap();

    assert!(memories
        .store(2, "after close", Vec::<String>::new(), "alice", 200)
        .is_err());
}

#[cfg(feature = "cortex")]
#[tokio::test]
async fn test_memories_and_tasks_coexist_on_same_redex() {
    // Two CortEX models sharing one Redex manager, each with its own
    // file. Events on either channel must not leak into the other's
    // state.
    use net::adapter::net::cortex::tasks::{TaskStatus, TasksAdapter};

    let redex = Redex::new();
    let memories = MemoriesAdapter::open(&redex, ORIGIN).unwrap();
    let tasks = TasksAdapter::open(&redex, ORIGIN).unwrap();

    // Drive both in parallel.
    memories
        .store(1, "mem-1", vec!["m".into()], "alice", 100)
        .unwrap();
    tasks.create(1, "task-1", 100).unwrap();
    memories
        .store(2, "mem-2", vec!["m".into()], "alice", 200)
        .unwrap();
    let task_seq = tasks.complete(1, 210).unwrap();
    let mem_seq = memories.pin(1, 220).unwrap();

    memories.wait_for_seq(mem_seq).await;
    tasks.wait_for_seq(task_seq).await;

    // Memories state has 2 memories, one pinned.
    let mstate = memories.state();
    let mg = mstate.read();
    assert_eq!(mg.len(), 2);
    assert_eq!(mg.query().where_pinned(true).count(), 1);

    // Tasks state has 1 task, completed.
    let tstate = tasks.state();
    let tg = tstate.read();
    assert_eq!(tg.len(), 1);
    assert_eq!(tg.get(1).unwrap().status, TaskStatus::Completed);
}
