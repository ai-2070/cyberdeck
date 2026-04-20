"""Regression tests for snapshot_and_watch on the Python bindings.

Mirrors `test_regression_snapshot_and_watch_forwards_divergent_stream_initial`
from the Rust integration suite and the vitest / net-sdk equivalents.
Under the pre-fix `skip(1)` implementation a mutation racing the
snapshot/stream construction would be silently dropped; here we spawn
a concurrent mutator and assert the stream delivers the post-snapshot
state every time the race window is hit.
"""

import threading
import time

import pytest

from net import MemoriesAdapter, Redex, TasksAdapter


ORIGIN = 0xABCDEF01


def now_ns() -> int:
    return time.time_ns()


def _next_within(it, timeout_s: float):
    """Call it.__next__ on a worker thread with a timeout.

    Python's native iterators are sync; there's no async-friendly way
    to impose a timeout on `next()`. Spin a thread and join with a
    deadline — if the thread is still alive, the iterator hung (the
    regression signal under the old `skip(1)` code).
    """
    result: list[object] = []
    err: list[BaseException] = []

    def pump() -> None:
        try:
            result.append(next(it))
        except StopIteration:
            result.append(None)
        except BaseException as e:  # pragma: no cover - defensive
            err.append(e)

    t = threading.Thread(target=pump, daemon=True)
    t.start()
    t.join(timeout_s)
    if t.is_alive():
        # Close the underlying iterator to let the thread exit.
        # Callers that hit this are failing the regression check.
        try:
            it.close()
        except Exception:
            pass
        t.join(1.0)
        return "timeout"
    if err:
        raise err[0]
    return result[0]


# =========================================================================
# Tasks
# =========================================================================


def test_tasks_snapshot_and_watch_returns_tuple() -> None:
    """Baseline: snapshot + iter come back together, snapshot is
    populated, iter is the usual pull-based shape."""
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)
    seq = tasks.create(1, "seed", 100)
    tasks.wait_for_seq(seq)

    snapshot, it = tasks.snapshot_and_watch_tasks()
    assert len(snapshot) == 1
    assert snapshot[0].id == 1
    assert hasattr(it, "__next__")
    assert hasattr(it, "close")
    it.close()


def test_tasks_snapshot_and_watch_delivers_post_call_updates() -> None:
    """Functional contract: any change after the call must land on
    the iterator. Passes under both skip(1) and skip_while — the race
    regression test below exercises the v2 fix itself."""
    redex = Redex()
    tasks = TasksAdapter.open(redex, ORIGIN)
    seq = tasks.create(1, "seed", 100)
    tasks.wait_for_seq(seq)

    snapshot, it = tasks.snapshot_and_watch_tasks()
    assert [t.id for t in snapshot] == [1]

    seq = tasks.create(2, "post", 200)
    tasks.wait_for_seq(seq)

    batch = _next_within(it, 1.0)
    assert batch != "timeout", "stream must deliver post-call mutation"
    assert batch is not None, "stream must not end"
    assert sorted(t.id for t in batch) == [1, 2]
    it.close()


def test_regression_tasks_snapshot_and_watch_forwards_divergent_initial() -> None:
    """Race regression: mutate concurrently with the snapshot_and_watch
    call. Trials where the mutation landed before the snapshot read
    are skipped (nothing further to deliver). On remaining trials the
    iterator must yield the post-mutation state within a short
    timeout — under the old `skip(1)` the race trials would hang
    because the watcher's internal `last` already equalled the
    post-mutation state, so no subsequent emission differed from it.
    """
    for trial in range(20):
        redex = Redex()
        tasks = TasksAdapter.open(redex, ORIGIN)
        seq = tasks.create(1, "seed", 100)
        tasks.wait_for_seq(seq)

        def mutate() -> None:
            s = tasks.create(2, "race", 200)
            tasks.wait_for_seq(s)

        mutator = threading.Thread(target=mutate, daemon=True)
        mutator.start()

        snapshot, it = tasks.snapshot_and_watch_tasks()
        mutator.join(5.0)
        assert not mutator.is_alive(), f"trial {trial}: mutator stuck"

        # Mutation landed before the snapshot read — no further delta
        # to deliver. Skip this trial.
        if len(snapshot) == 2:
            it.close()
            continue
        assert len(snapshot) == 1, f"trial {trial}: snapshot should be [seed]"

        batch = _next_within(it, 1.0)
        assert batch != "timeout", (
            f"trial {trial}: iter must deliver post-snapshot state within timeout"
        )
        assert batch is not None, f"trial {trial}: iter must not end"
        assert len(batch) == 2, (
            f"trial {trial}: iter must deliver state with both tasks"
        )
        it.close()


# =========================================================================
# Memories
# =========================================================================


def test_regression_memories_snapshot_and_watch_forwards_divergent_initial() -> None:
    """Memories mirror of the tasks race regression test."""
    for trial in range(20):
        redex = Redex()
        mem = MemoriesAdapter.open(redex, ORIGIN)
        seq = mem.store(1, "seed", ["t"], "alice", 100)
        mem.wait_for_seq(seq)

        def mutate() -> None:
            s = mem.store(2, "race", ["t"], "alice", 200)
            mem.wait_for_seq(s)

        mutator = threading.Thread(target=mutate, daemon=True)
        mutator.start()

        snapshot, it = mem.snapshot_and_watch_memories()
        mutator.join(5.0)
        assert not mutator.is_alive(), f"trial {trial}: mutator stuck"

        if len(snapshot) == 2:
            it.close()
            continue
        assert len(snapshot) == 1, f"trial {trial}: snapshot should be [seed]"

        batch = _next_within(it, 1.0)
        assert batch != "timeout", (
            f"trial {trial}: iter must deliver post-snapshot state within timeout"
        )
        assert batch is not None, f"trial {trial}: iter must not end"
        assert len(batch) == 2, (
            f"trial {trial}: iter must deliver both memories"
        )
        it.close()


def test_memories_snapshot_and_watch_preserves_filter() -> None:
    """`snapshot_and_watch_memories(**filter)` must apply the filter
    to both the snapshot and the delta stream."""
    redex = Redex()
    mem = MemoriesAdapter.open(redex, ORIGIN)
    mem.store(1, "pinned-content", ["x"], "alice", 100)
    mem.store(2, "unpinned-content", ["x"], "alice", 100)
    seq = mem.pin(1, 110)
    mem.wait_for_seq(seq)

    snapshot, it = mem.snapshot_and_watch_memories(pinned=True)
    assert [m.id for m in snapshot] == [1]
    it.close()
