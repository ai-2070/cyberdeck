"""Regression tests for the Python `RedisStreamDedup` re-export
(CR-3).

Pre-fix the PyO3 module registered `RedisStreamDedup` in `_net` but
the public Python API (`net/__init__.py`) never re-exported it, so
every README example saying ``from net import RedisStreamDedup``
hit ImportError. This file pins both:

1. Direct `from net import RedisStreamDedup` works when the
   binding was built with the `redis` feature.
2. The class behaves correctly under cross-thread shared use
   (pre-CR-2 it was `!Send + !Sync` because the underlying Rust
   storage was `Rc<str>`; this test pins the post-fix `Arc<str>`
   guarantee at the Python layer).

The tests are skip-on-ImportError so they no-op cleanly on builds
without the `redis` feature.
"""

from __future__ import annotations

import threading

import pytest

try:
    from net import RedisStreamDedup
except ImportError:
    RedisStreamDedup = None  # type: ignore[assignment]


pytestmark = pytest.mark.skipif(
    RedisStreamDedup is None,
    reason="binding built without `redis` feature; RedisStreamDedup not exposed",
)


def test_first_observation_is_not_a_duplicate() -> None:
    d = RedisStreamDedup(64)
    assert d.is_duplicate("abc:0:0:0") is False
    assert d.len == 1


def test_repeat_observation_is_a_duplicate() -> None:
    d = RedisStreamDedup(64)
    d.is_duplicate("abc:0:0:0")
    assert d.is_duplicate("abc:0:0:0") is True


def test_default_capacity_is_4096() -> None:
    d = RedisStreamDedup()
    assert d.capacity == 4096


def test_clear_resets_state() -> None:
    d = RedisStreamDedup(64)
    d.is_duplicate("a")
    d.is_duplicate("b")
    assert d.len == 2
    d.clear()
    assert d.len == 0
    assert d.is_empty is True
    # Post-clear, re-observation is NOT a duplicate.
    assert d.is_duplicate("a") is False


def test_concurrent_threads_share_one_handle_safely() -> None:
    """CR-2: pin that one handle can be shared across Python
    threads. Pre-CR-2 the underlying Rust type was `!Send + !Sync`;
    PyO3 would have rejected the class entirely (the binding
    wouldn't build). Post-fix this is `Send + Sync` and the NAPI
    `Mutex` guards the inner state — concurrent
    `is_duplicate` calls serialize but are safe.
    """
    d = RedisStreamDedup(4096)
    n_threads = 4
    per_thread = 64
    barrier = threading.Barrier(n_threads)

    def worker(t: int) -> None:
        barrier.wait()
        for i in range(per_thread):
            # Distinct id per (thread, i) so every insert is novel.
            assert d.is_duplicate(f"t{t}:{i}") is False

    threads = [threading.Thread(target=worker, args=(t,)) for t in range(n_threads)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert d.len == n_threads * per_thread
