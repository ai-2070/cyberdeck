"""Tests for the compute surface — Stage 5 of SDK_COMPUTE_SURFACE_PLAN.md.

Mirrors `sdk-ts/test/compute.test.ts`. Sub-step 1 covers lifecycle only:
a Python caller can build a `DaemonRuntime` against a `NetMesh`, register
a factory (stored but not yet invoked), start the runtime, and shut it
down. Event delivery, migration, snapshot/restore land in sub-steps 2-5.
"""

from __future__ import annotations

import itertools

import pytest

from net import CausalEvent, DaemonHandle, DaemonRuntime, Identity, NetMesh

PSK = "42" * 32

# Per-test unique ports so repeated runs don't collide on localhost.
_port_counter = itertools.count(29_400)


def _next_port() -> str:
    return f"127.0.0.1:{next(_port_counter)}"


def _mesh() -> NetMesh:
    return NetMesh(bind_addr=_next_port(), psk=PSK)


# -------------------------------------------------------------------------
# Stage 5 sub-step 1: skeleton + lifecycle
# -------------------------------------------------------------------------


def test_builds_against_mesh_and_reports_not_ready_before_start() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        assert rt.is_ready() is False
        assert rt.daemon_count() == 0
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_start_flips_to_ready_shutdown_flips_back() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.start()
        assert rt.is_ready() is True
        rt.shutdown()
        assert rt.is_ready() is False
    finally:
        mesh.shutdown()


def test_register_factory_accepts_a_python_callable() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("echo", lambda: {"name": "echo"})
        # Sub-step 1 stores the factory but doesn't invoke it.
        # Correctness proves itself via the no-exception path here
        # and the duplicate-registration check below.
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_register_factory_second_registration_of_same_kind_fails() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("echo", lambda: {})
        with pytest.raises(RuntimeError) as exc_info:
            rt.register_factory("echo", lambda: {})
        assert "already registered" in str(exc_info.value)
        assert str(exc_info.value).startswith("daemon:")
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_register_factory_different_kinds_coexist() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("echo", lambda: {})
        rt.register_factory("counter", lambda: {})
        rt.register_factory("router", lambda: {})
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_shutdown_is_idempotent() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.start()
        rt.shutdown()
        # Second shutdown is a no-op.
        rt.shutdown()
    finally:
        mesh.shutdown()


def test_daemon_runtime_does_not_shut_down_underlying_mesh() -> None:
    # Shutting down the runtime tears down daemons + migration handler
    # but leaves the NetMesh alive. Caller owns the mesh lifecycle.
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.start()
        rt.shutdown()
        # Mesh should still be usable after runtime shutdown.
        assert mesh.node_id != 0
    finally:
        mesh.shutdown()


# -------------------------------------------------------------------------
# Stage 5 sub-step 2: spawn + stop + event dispatch
# -------------------------------------------------------------------------


class EchoDaemon:
    """Trivial stateless daemon — echoes the event payload."""

    def process(self, event: CausalEvent) -> list[bytes]:
        return [event.payload]


class CounterDaemon:
    """Stateful daemon — increments on every event, exposes state via
    snapshot/restore. Factory closure captures a fresh `count = 0` per
    instance."""

    def __init__(self) -> None:
        self._count = 0

    def process(self, event: CausalEvent) -> list[bytes]:
        self._count += 1
        return [self._count.to_bytes(4, "little")]

    def snapshot(self) -> bytes:
        return self._count.to_bytes(4, "little")

    def restore(self, state: bytes) -> None:
        self._count = int.from_bytes(state, "little")


def test_spawn_returns_handle_with_origin_hash_and_entity_id() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("echo", EchoDaemon)
        rt.start()
        ident = Identity.generate()
        handle = rt.spawn("echo", ident)
        assert isinstance(handle, DaemonHandle)
        assert handle.origin_hash == ident.origin_hash
        assert handle.entity_id == ident.entity_id
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_spawn_unregistered_kind_raises() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.start()
        ident = Identity.generate()
        with pytest.raises(RuntimeError) as exc_info:
            rt.spawn("never-registered", ident)
        assert "no factory registered" in str(exc_info.value)
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_spawn_stop_reduces_daemon_count() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("echo", EchoDaemon)
        rt.start()
        ident = Identity.generate()
        handle = rt.spawn("echo", ident)
        assert rt.daemon_count() == 1
        rt.stop(handle.origin_hash)
        assert rt.daemon_count() == 0
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_echo_daemon_round_trip_via_deliver() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("echo", EchoDaemon)
        rt.start()
        ident = Identity.generate()
        handle = rt.spawn("echo", ident)

        payload = b"hello from python"
        event = CausalEvent(ident.origin_hash, 1, payload)
        outputs = rt.deliver(handle.origin_hash, event)
        assert len(outputs) == 1
        assert outputs[0] == payload
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_counter_daemon_accumulates_state_across_deliveries() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("counter", CounterDaemon)
        rt.start()
        ident = Identity.generate()
        handle = rt.spawn("counter", ident)

        for i in range(1, 6):
            event = CausalEvent(ident.origin_hash, i, b"")
            outputs = rt.deliver(handle.origin_hash, event)
            assert len(outputs) == 1
            assert int.from_bytes(outputs[0], "little") == i
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_process_returning_multiple_buffers_is_fanout() -> None:
    class Fanout:
        def process(self, _event: CausalEvent) -> list[bytes]:
            return [b"a", b"bb", b"ccc"]

    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("fanout", Fanout)
        rt.start()
        ident = Identity.generate()
        handle = rt.spawn("fanout", ident)
        outputs = rt.deliver(handle.origin_hash, CausalEvent(ident.origin_hash, 1, b""))
        assert outputs == [b"a", b"bb", b"ccc"]
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_process_raising_surfaces_as_daemon_error() -> None:
    class Buggy:
        def process(self, _event: CausalEvent) -> list[bytes]:
            raise ValueError("deliberate process failure")

    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("buggy", Buggy)
        rt.start()
        ident = Identity.generate()
        handle = rt.spawn("buggy", ident)
        with pytest.raises(RuntimeError) as exc_info:
            rt.deliver(handle.origin_hash, CausalEvent(ident.origin_hash, 1, b""))
        assert str(exc_info.value).startswith("daemon:")
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_deliver_to_unknown_origin_raises() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("echo", EchoDaemon)
        rt.start()
        with pytest.raises(RuntimeError) as exc_info:
            rt.deliver(0xDEADBEEF, CausalEvent(0xDEADBEEF, 1, b"x"))
        assert str(exc_info.value).startswith("daemon:")
    finally:
        rt.shutdown()
        mesh.shutdown()


# -------------------------------------------------------------------------
# Stage 5 sub-step 3: snapshot + restore round-trip
# -------------------------------------------------------------------------


def test_counter_snapshot_then_spawn_from_snapshot_restores_state() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("counter", CounterDaemon)
        rt.start()
        ident = Identity.generate()
        handle = rt.spawn("counter", ident)

        # Drive the counter to 3.
        for i in range(1, 4):
            rt.deliver(handle.origin_hash, CausalEvent(ident.origin_hash, i, b""))

        snap = rt.snapshot(handle.origin_hash)
        assert snap is not None
        assert isinstance(snap, bytes)
        assert len(snap) > 0

        # Tear the original daemon down — the restored instance
        # must pick up purely from the snapshot, not from live
        # state.
        rt.stop(handle.origin_hash)
        assert rt.daemon_count() == 0

        restored = rt.spawn_from_snapshot("counter", ident, snap)
        assert rt.daemon_count() == 1
        assert restored.origin_hash == handle.origin_hash

        # One more delivery — counter steps from 3 to 4, proving
        # the snapshot's state survived the round-trip.
        out = rt.deliver(restored.origin_hash, CausalEvent(ident.origin_hash, 4, b""))
        assert int.from_bytes(out[0], "little") == 4
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_snapshot_of_stateless_daemon_returns_none() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("echo", EchoDaemon)
        rt.start()
        ident = Identity.generate()
        handle = rt.spawn("echo", ident)
        assert rt.snapshot(handle.origin_hash) is None
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_snapshot_of_unknown_origin_raises() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("counter", CounterDaemon)
        rt.start()
        with pytest.raises(RuntimeError) as exc_info:
            rt.snapshot(0xDEADBEEF)
        assert str(exc_info.value).startswith("daemon:")
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_spawn_from_snapshot_with_corrupted_bytes_raises() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("counter", CounterDaemon)
        rt.start()
        ident = Identity.generate()
        with pytest.raises(RuntimeError) as exc_info:
            rt.spawn_from_snapshot("counter", ident, b"not a real snapshot")
        assert "snapshot decode failed" in str(exc_info.value)
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_spawn_from_snapshot_with_wrong_identity_raises() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("counter", CounterDaemon)
        rt.start()
        original = Identity.generate()
        handle = rt.spawn("counter", original)
        rt.deliver(handle.origin_hash, CausalEvent(original.origin_hash, 1, b""))
        snap = rt.snapshot(handle.origin_hash)
        assert snap is not None
        rt.stop(handle.origin_hash)

        # Different identity — snapshot's entity_id doesn't match.
        other = Identity.generate()
        with pytest.raises(RuntimeError) as exc_info:
            rt.spawn_from_snapshot("counter", other, snap)
        assert str(exc_info.value).startswith("daemon:")
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_snapshot_modify_snapshot_captures_newer_state() -> None:
    # Restoring an earlier vs later snapshot yields different
    # counter values. Proves snapshot captures the state at the
    # moment it was taken.
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("counter", CounterDaemon)
        rt.start()
        ident = Identity.generate()
        handle = rt.spawn("counter", ident)

        for i in range(1, 3):
            rt.deliver(handle.origin_hash, CausalEvent(ident.origin_hash, i, b""))
        snap_at_2 = rt.snapshot(handle.origin_hash)
        for i in range(3, 6):
            rt.deliver(handle.origin_hash, CausalEvent(ident.origin_hash, i, b""))
        snap_at_5 = rt.snapshot(handle.origin_hash)

        rt.stop(handle.origin_hash)

        # Restore earlier snapshot; next event steps to 3.
        h2 = rt.spawn_from_snapshot("counter", ident, snap_at_2)
        out = rt.deliver(h2.origin_hash, CausalEvent(ident.origin_hash, 6, b""))
        assert int.from_bytes(out[0], "little") == 3
        rt.stop(h2.origin_hash)

        # Restore later snapshot; next event steps to 6.
        h5 = rt.spawn_from_snapshot("counter", ident, snap_at_5)
        out = rt.deliver(h5.origin_hash, CausalEvent(ident.origin_hash, 7, b""))
        assert int.from_bytes(out[0], "little") == 6
    finally:
        rt.shutdown()
        mesh.shutdown()


def test_two_daemons_keep_independent_counter_state() -> None:
    mesh = _mesh()
    rt = DaemonRuntime(mesh)
    try:
        rt.register_factory("counter", CounterDaemon)
        rt.start()
        id_a = Identity.generate()
        id_b = Identity.generate()
        h_a = rt.spawn("counter", id_a)
        h_b = rt.spawn("counter", id_b)

        for i in range(1, 4):
            out = rt.deliver(h_a.origin_hash, CausalEvent(id_a.origin_hash, i, b""))
            assert int.from_bytes(out[0], "little") == i

        out = rt.deliver(h_b.origin_hash, CausalEvent(id_b.origin_hash, 1, b""))
        assert int.from_bytes(out[0], "little") == 1

        # A advances one more; B advances one more. Independent.
        out_a = rt.deliver(h_a.origin_hash, CausalEvent(id_a.origin_hash, 4, b""))
        out_b = rt.deliver(h_b.origin_hash, CausalEvent(id_b.origin_hash, 2, b""))
        assert int.from_bytes(out_a[0], "little") == 4
        assert int.from_bytes(out_b[0], "little") == 2
    finally:
        rt.shutdown()
        mesh.shutdown()
