"""Tests for the compute surface — Stage 5 of SDK_COMPUTE_SURFACE_PLAN.md.

Mirrors `sdk-ts/test/compute.test.ts`. Sub-step 1 covers lifecycle only:
a Python caller can build a `DaemonRuntime` against a `NetMesh`, register
a factory (stored but not yet invoked), start the runtime, and shut it
down. Event delivery, migration, snapshot/restore land in sub-steps 2-5.
"""

from __future__ import annotations

import itertools

import pytest

from net import DaemonRuntime, NetMesh

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
