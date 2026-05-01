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

Cubic P2: skip on whether the underlying NAPI/PyO3 binding was
built with the `redis` feature, NOT on the public re-export
itself. The earlier shape `try: from net import RedisStreamDedup`
would silently pass if the re-export was missing, defeating the
test. Now we probe the lower-level `_net` module (which always
exists when the binding is loadable) to decide whether to skip.
"""

from __future__ import annotations

import threading

import pytest

# Skip-decision probe: does the underlying binding have the
# RedisStreamDedup symbol? This is independent of the public
# `from net import RedisStreamDedup` we're trying to verify.
_BINDING_HAS_REDIS_DEDUP = False
try:
    from net import _net  # type: ignore[attr-defined]
    _BINDING_HAS_REDIS_DEDUP = hasattr(_net, "RedisStreamDedup")
except ImportError:
    # Whole `net` package missing — treat as skip.
    _BINDING_HAS_REDIS_DEDUP = False


pytestmark = pytest.mark.skipif(
    not _BINDING_HAS_REDIS_DEDUP,
    reason="binding built without `redis` feature; RedisStreamDedup not in _net",
)


# CR-3 / Cubic P2: the load-bearing import is THIS one — public
# `from net import RedisStreamDedup`. If the re-export is broken
# this raises ImportError and every test below errors out
# (not skipped). That's the contract we want to pin.
from net import RedisStreamDedup  # noqa: E402


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
