"""NetDB smoke tests — unified db handle bundling tasks + memories."""

import pytest

from net._net import CortexError, NetDb, NetDbError

ORIGIN = 0xABCDEF01


# =========================================================================
# build + inspect
# =========================================================================


def test_open_with_both_models() -> None:
    db = NetDb.open(origin_hash=ORIGIN, with_tasks=True, with_memories=True)
    assert db.tasks is not None
    assert db.memories is not None


def test_open_with_only_tasks() -> None:
    db = NetDb.open(origin_hash=ORIGIN, with_tasks=True)
    assert db.tasks is not None
    assert db.memories is None


def test_open_with_neither_model() -> None:
    db = NetDb.open(origin_hash=ORIGIN)
    assert db.tasks is None
    assert db.memories is None


# =========================================================================
# CRUD through bundled adapters
# =========================================================================


def test_tasks_crud_via_db_tasks() -> None:
    db = NetDb.open(origin_hash=ORIGIN, with_tasks=True)
    tasks = db.tasks
    assert tasks is not None
    tasks.create(1, "write plan", 100)
    tasks.create(2, "ship netdb", 200)
    seq = tasks.complete(1, 150)
    tasks.wait_for_seq(seq)

    all_tasks = tasks.list_tasks()
    assert len(all_tasks) == 2
    by_id = {t.id: t for t in all_tasks}
    assert by_id[1].status == "completed"


def test_memories_crud_via_db_memories() -> None:
    db = NetDb.open(origin_hash=ORIGIN, with_memories=True)
    memories = db.memories
    assert memories is not None
    memories.store(1, "hello", ["x"], "alice", 100)
    seq = memories.pin(1, 110)
    memories.wait_for_seq(seq)

    all_m = memories.list_memories()
    assert len(all_m) == 1
    assert all_m[0].pinned is True


def test_both_models_coexist() -> None:
    db = NetDb.open(origin_hash=ORIGIN, with_tasks=True, with_memories=True)
    db.tasks.create(1, "task", 100)
    db.memories.store(1, "mem", ["x"], "alice", 100)
    ts = db.tasks.complete(1, 150)
    ms = db.memories.pin(1, 150)
    db.tasks.wait_for_seq(ts)
    db.memories.wait_for_seq(ms)

    assert db.tasks.count() == 1
    assert db.memories.count() == 1


# =========================================================================
# Filters through bundled adapters
# =========================================================================


def test_filter_queries() -> None:
    db = NetDb.open(origin_hash=ORIGIN, with_tasks=True, with_memories=True)

    for i in range(1, 6):
        db.tasks.create(i, f"t-{i}", 100 * i)
    last = db.tasks.complete(2, 999)
    db.tasks.wait_for_seq(last)

    db.memories.store(1, "work note", ["work"], "alice", 100)
    db.memories.store(2, "home note", ["home"], "alice", 200)
    m_seq = db.memories.pin(1, 210)
    db.memories.wait_for_seq(m_seq)

    pending = db.tasks.list_tasks(status="pending", order_by="id_asc")
    assert [t.id for t in pending] == [1, 3, 4, 5]

    pinned = db.memories.list_memories(pinned=True, order_by="id_asc")
    assert [m.id for m in pinned] == [1]


# =========================================================================
# Whole-db snapshot / restore
# =========================================================================


def test_snapshot_and_restore_round_trip() -> None:
    db = NetDb.open(origin_hash=ORIGIN, with_tasks=True, with_memories=True)

    db.tasks.create(1, "alpha", 100)
    t_seq = db.tasks.complete(1, 150)
    db.memories.store(1, "hello", ["x"], "alice", 100)
    m_seq = db.memories.pin(1, 110)
    db.tasks.wait_for_seq(t_seq)
    db.memories.wait_for_seq(m_seq)

    bundle = db.snapshot()
    assert isinstance(bundle, bytes)
    assert len(bundle) > 0
    db.close()

    # Restore on a fresh NetDb.
    db2 = NetDb.open_from_snapshot(
        bundle,
        origin_hash=ORIGIN,
        with_tasks=True,
        with_memories=True,
    )
    all_tasks = db2.tasks.list_tasks()
    assert len(all_tasks) == 1
    assert all_tasks[0].status == "completed"

    all_memories = db2.memories.list_memories()
    assert len(all_memories) == 1
    assert all_memories[0].pinned is True


def test_restore_opens_missing_model_fresh() -> None:
    # Build a snapshot that only covers tasks.
    db = NetDb.open(origin_hash=ORIGIN, with_tasks=True)
    db.tasks.create(1, "just tasks", 100)
    seq = db.tasks.complete(1, 150)
    db.tasks.wait_for_seq(seq)
    bundle = db.snapshot()
    db.close()

    # Restore with both models — memories has no snapshot entry, so
    # it opens fresh (empty).
    db2 = NetDb.open_from_snapshot(
        bundle,
        origin_hash=ORIGIN,
        with_tasks=True,
        with_memories=True,
    )
    assert db2.tasks.count() == 1
    assert db2.memories.count() == 0


def test_close_is_idempotent() -> None:
    db = NetDb.open(origin_hash=ORIGIN, with_tasks=True, with_memories=True)
    db.close()
    db.close()  # no throw


def test_persistent_without_dir_errors() -> None:
    # Underlying adapter open failure surfaces as CortexError, not
    # NetDbError — NetDbError is reserved for handle-level errors
    # (snapshot encode/decode, model-not-included).
    with pytest.raises(CortexError, match="persistent"):
        NetDb.open(origin_hash=ORIGIN, with_tasks=True, persistent=True)


def test_netdb_snapshot_decode_raises_netdb_error() -> None:
    # Garbage bytes → bincode decode error → NetDbError.
    with pytest.raises(NetDbError, match="decode bundle"):
        NetDb.open_from_snapshot(
            b"\xff\xff\xff garbage not bincode \xff\xff",
            origin_hash=ORIGIN,
            with_tasks=True,
        )


def test_error_classes_are_exception_subclasses() -> None:
    assert issubclass(CortexError, Exception)
    assert issubclass(NetDbError, Exception)
    # They're independent — NetDbError is not a CortexError subclass.
    assert not issubclass(NetDbError, CortexError)
    assert not issubclass(CortexError, NetDbError)
