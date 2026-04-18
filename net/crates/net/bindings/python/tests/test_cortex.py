"""Smoke tests for the CortEX Python bindings (tasks + memories).

Mirrors the Node vitest suite. Exercises CRUD, filter queries, and
the sync Python iterator protocol for watchers.
"""

import threading
import time

import pytest

from net._net import (
    MemoriesAdapter,
    Redex,
    TasksAdapter,
)

ORIGIN = 0xABCDEF01


def now_ns() -> int:
    return time.time_ns()


# =========================================================================
# Tasks
# =========================================================================


def test_tasks_full_lifecycle() -> None:
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)

    t0 = now_ns()
    tasks.create(1, "write plan", t0)
    tasks.create(2, "ship adapter", t0 + 1)
    tasks.rename(1, "write better plan", t0 + 2)
    seq = tasks.complete(2, t0 + 3)
    tasks.wait_for_seq(seq)

    all_tasks = tasks.list_tasks()
    assert len(all_tasks) == 2

    by_id = {t.id: t for t in all_tasks}
    assert by_id[1].title == "write better plan"
    assert by_id[1].status == "pending"
    assert by_id[2].title == "ship adapter"
    assert by_id[2].status == "completed"


def test_tasks_filter_and_order() -> None:
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)

    for i in range(1, 6):
        tasks.create(i, f"t-{i}", 100 * i)
    seq = tasks.complete(1, 999)
    tasks.wait_for_seq(seq)

    # Status filter.
    pending = tasks.list_tasks(status="pending")
    assert sorted(t.id for t in pending) == [2, 3, 4, 5]

    # Order + limit.
    newest = tasks.list_tasks(order_by="created_desc", limit=2)
    assert [t.id for t in newest] == [5, 4]


def test_tasks_delete_removes_task() -> None:
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)
    tasks.create(1, "ephemeral", 100)
    seq = tasks.delete(1)
    tasks.wait_for_seq(seq)
    assert tasks.count() == 0


def test_tasks_ingest_after_close_errors() -> None:
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)
    tasks.create(1, "before", 100)
    tasks.close()
    with pytest.raises(RuntimeError):
        tasks.create(2, "after", 200)


def test_tasks_invalid_status_raises() -> None:
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)
    with pytest.raises(ValueError):
        tasks.list_tasks(status="nonsense")


# =========================================================================
# Memories
# =========================================================================


def test_memories_full_lifecycle() -> None:
    redex = Redex()
    memories = MemoriesAdapter.open(redex, ORIGIN)

    memories.store(1, "meeting notes", ["work", "notes"], "alice", 100)
    memories.store(2, "grocery list", ["personal", "todo"], "alice", 200)
    memories.store(3, "api design", ["work", "design"], "bob", 300)
    memories.retag(1, ["work", "meetings"], 310)
    memories.pin(3, 320)
    seq = memories.pin(1, 330)
    memories.wait_for_seq(seq)

    assert memories.count() == 3

    work = memories.list_memories(tag="work")
    assert sorted(m.id for m in work) == [1, 3]

    any_match = memories.list_memories(any_tag=["design", "todo"])
    assert sorted(m.id for m in any_match) == [2, 3]

    all_match = memories.list_memories(all_tags=["work", "meetings"])
    assert [m.id for m in all_match] == [1]

    pinned = memories.list_memories(pinned=True)
    assert sorted(m.id for m in pinned) == [1, 3]

    bob = memories.list_memories(source="bob")
    assert [m.id for m in bob] == [3]


def test_memories_content_search_case_insensitive() -> None:
    redex = Redex()
    memories = MemoriesAdapter.open(redex, ORIGIN)
    seq = memories.store(1, "Fire in the datacenter", [], "alice", 100)
    memories.wait_for_seq(seq)

    hit = memories.list_memories(content_contains="DATACENTER")
    assert [m.id for m in hit] == [1]

    miss = memories.list_memories(content_contains="unicorn")
    assert miss == []


# =========================================================================
# Watch (sync iterator protocol)
# =========================================================================


def test_watch_tasks_initial_emission() -> None:
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)

    tasks.create(1, "alpha", 100)
    seq = tasks.create(2, "beta", 200)
    tasks.wait_for_seq(seq)

    it = tasks.watch_tasks(status="pending", order_by="id_asc")
    try:
        initial = next(it)
        assert [t.id for t in initial] == [1, 2]
    finally:
        it.close()


def test_watch_tasks_close_stops_iteration() -> None:
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)

    it = tasks.watch_tasks()
    # Drain initial (empty).
    initial = next(it)
    assert initial == []

    # Close, then subsequent next raises StopIteration.
    it.close()
    with pytest.raises(StopIteration):
        next(it)


def test_watch_tasks_for_loop_exits_on_close() -> None:
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)

    emissions = []
    # Race condition: same as the Node test — fast fires can coalesce.
    # Check final state, not exact emission count.
    seen_states: set = set()

    def state_key(lst):
        return tuple(sorted(t.id for t in lst))

    it = tasks.watch_tasks(status="pending")

    def consume():
        for batch in it:
            emissions.append(batch)
            seen_states.add(state_key(batch))
            if state_key(batch) == (1, 2):
                it.close()

    reader = threading.Thread(target=consume)
    reader.start()

    tasks.create(1, "a", 100)
    tasks.create(2, "b", 200)

    reader.join(timeout=5)
    assert not reader.is_alive(), "watcher thread did not exit after close()"

    # Initial empty AND final (1, 2) state must both have been observed.
    assert () in seen_states
    assert (1, 2) in seen_states


def test_watch_memories_tag_filter() -> None:
    redex = Redex()
    memories = MemoriesAdapter.open(redex, ORIGIN)

    it = memories.watch_memories(tag="urgent")
    try:
        initial = next(it)
        assert initial == []

        # Store non-matching memory → no new emission yet.
        memories.store(1, "routine", ["later"], "alice", 100)

        # Store matching memory → emission [2].
        memories.store(2, "fire", ["urgent"], "alice", 200)
        next_batch = next(it)
        assert [m.id for m in next_batch] == [2]

        # Retag #1 to include urgent → emission grows.
        memories.retag(1, ["urgent", "later"], 300)
        next_batch = next(it)
        assert sorted(m.id for m in next_batch) == [1, 2]
    finally:
        it.close()


def test_persistent_tasks_round_trip(tmp_path) -> None:
    dir = str(tmp_path / "tasks")
    # First process: create + persist.
    redex1 = Redex(persistent_dir=dir)
    tasks1 = TasksAdapter.open(redex1, ORIGIN, persistent=True)
    tasks1.create(1, "durable", 100)
    tasks1.create(2, "also durable", 101)
    seq = tasks1.complete(1, 102)
    tasks1.wait_for_seq(seq)
    tasks1.close()
    del redex1, tasks1

    # Second process: reopen same dir, state replays from disk.
    redex2 = Redex(persistent_dir=dir)
    tasks2 = TasksAdapter.open(redex2, ORIGIN, persistent=True)
    tasks2.wait_for_seq(2)
    all_tasks = tasks2.list_tasks()
    assert len(all_tasks) == 2
    by_id = {t.id: t for t in all_tasks}
    assert by_id[1].status == "completed"
    assert by_id[2].status == "pending"
    assert by_id[2].title == "also durable"


def test_persistent_memories_round_trip(tmp_path) -> None:
    dir = str(tmp_path / "mem")
    redex1 = Redex(persistent_dir=dir)
    memories1 = MemoriesAdapter.open(redex1, ORIGIN, persistent=True)
    memories1.store(1, "alpha", ["x"], "alice", 100)
    memories1.pin(1, 110)
    memories1.store(2, "beta", ["y"], "alice", 200)
    seq = memories1.retag(2, ["y", "z"], 210)
    memories1.wait_for_seq(seq)
    memories1.close()
    del redex1, memories1

    redex2 = Redex(persistent_dir=dir)
    memories2 = MemoriesAdapter.open(redex2, ORIGIN, persistent=True)
    memories2.wait_for_seq(3)
    all_m = memories2.list_memories()
    assert len(all_m) == 2
    by_id = {m.id: m for m in all_m}
    assert by_id[1].pinned is True
    assert sorted(by_id[2].tags) == ["y", "z"]


def test_persistent_without_dir_errors() -> None:
    redex = Redex()  # heap-only, no persistent_dir
    with pytest.raises(RuntimeError, match="persistent"):
        TasksAdapter.open(redex, ORIGIN, persistent=True)


def test_multi_model_coexistence() -> None:
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)
    memories = MemoriesAdapter.open(redex, ORIGIN)

    tasks.create(1, "task-1", 100)
    memories.store(1, "mem-1", ["x"], "alice", 100)
    memories.store(2, "mem-2", ["x"], "alice", 200)
    ts = tasks.complete(1, 150)
    ms = memories.pin(1, 250)
    tasks.wait_for_seq(ts)
    memories.wait_for_seq(ms)

    assert tasks.count() == 1
    assert memories.count() == 2
    assert [m.id for m in memories.list_memories(pinned=True)] == [1]
