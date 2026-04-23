"""ABI-stability regression tests for the Python binding's error
surface.

The Rust side raises messages of the shape
``daemon: migration: <kind>[: <detail>]`` and
``daemon: group: <kind>[: <detail>]``. SDK users match on the
``<kind>`` token via the ``migration_error_kind`` /
``group_error_kind`` helpers, so:

- Renaming any kind is a **silent breaking change** for callers
  whose ``if kind == "not-ready"`` branches suddenly never match.
- Changing the ``daemon: migration:`` prefix breaks the parser
  itself, returning ``None`` for every caught exception.

These tests pin both surfaces. They operate on the pure-Python
parser path (no mesh, no runtime, no network) — the helper takes
``str(exc)``, so any exception-like object with the right
stringification is enough.

Corresponds to TEST_COVERAGE_PLAN §P3-15.
"""

from __future__ import annotations

import re

import pytest

from net import migration_error_kind

try:
    from net import group_error_kind
except ImportError:
    group_error_kind = None  # type: ignore[assignment]


# Stable kind vocabulary emitted by the Rust layer. This list is
# the cross-binding contract — Go / Node / Python all agree on
# these exact strings. A rename anywhere in the stack flags here.
MIGRATION_KINDS = (
    "not-ready",
    "factory-not-found",
    "compute-not-supported",
    "state-failed",
    "already-migrating",
    "identity-transport-failed",
    "not-ready-timeout",
    "daemon-not-found",
    "target-unavailable",
    "wrong-phase",
    "snapshot-too-large",
)


class _FakeExc(Exception):
    """A stand-in that lets us drive ``str(exc)`` with controlled
    content without depending on the PyO3 `MigrationError` class
    being constructible from pure Python (it may or may not be,
    depending on whether the SDK wants the class opaque).
    """


@pytest.mark.parametrize("kind", MIGRATION_KINDS)
def test_migration_error_kind_parses_every_known_kind(kind: str) -> None:
    # Tag-only form — no trailing detail.
    tag_only = _FakeExc(f"daemon: migration: {kind}")
    assert migration_error_kind(tag_only) == kind, (
        f"migration_error_kind must recover {kind!r} from the tag-only envelope"
    )

    # Tag + detail form — the parser returns just the kind, not
    # the detail. Pinning that so callers don't see "wrong-phase:
    # expected Snapshot" and need to re-split.
    with_detail = _FakeExc(f"daemon: migration: {kind}: detail body here")
    assert migration_error_kind(with_detail) == kind


def test_migration_error_kind_handles_whitespace_around_kind() -> None:
    # The parser ``strip``s the body, so extra whitespace around
    # the kind must not break classification.
    exc = _FakeExc("daemon: migration:   not-ready   ")
    assert migration_error_kind(exc) == "not-ready"


def test_migration_error_kind_returns_none_for_unprefixed_message() -> None:
    # Messages that don't carry the envelope return None — SDK
    # callers rely on this to distinguish "not a migration error"
    # from a known kind. Changing it to raise or return "" would
    # break the `if kind is None:` branch.
    assert migration_error_kind(_FakeExc("factory not found")) is None
    assert migration_error_kind(_FakeExc("daemon: plain daemon error")) is None
    assert migration_error_kind(_FakeExc("")) is None


def test_migration_error_kind_unknown_future_kind_is_returned_verbatim() -> None:
    # Forward compatibility: an older binding running against a
    # newer Rust layer will see kinds it doesn't recognize. The
    # helper must still return the kind string (not raise, not
    # collapse to None) so callers can log it and dispatch on a
    # pinned set in their own code.
    exc = _FakeExc("daemon: migration: brand-new-kind-2030")
    assert migration_error_kind(exc) == "brand-new-kind-2030"


def test_migration_error_prefix_is_pinned_literal() -> None:
    # Hard-code the prefix the parser recognizes. If this fails,
    # both the Rust wire format AND the helper in __init__.py
    # must be updated together — this test is the alarm bell.
    exc = _FakeExc("daemon: migration: not-ready")
    assert migration_error_kind(exc) == "not-ready"

    # Case-sensitive: the parser must not lowercase or normalize.
    exc = _FakeExc("DAEMON: MIGRATION: not-ready")
    assert migration_error_kind(exc) is None, (
        "parser must be case-sensitive; uppercase prefix is not a known envelope"
    )


MIGRATION_ENVELOPE_RE = re.compile(r"^daemon: migration: [a-z][a-z0-9-]*(: .+)?$")


@pytest.mark.parametrize("kind", MIGRATION_KINDS)
def test_migration_kind_shape_is_kebab_case_lowercase(kind: str) -> None:
    # Pin the shape of each kind token: lowercase, kebab-case,
    # starts with a letter. Matches the Go + Node vocabulary.
    full = f"daemon: migration: {kind}"
    assert MIGRATION_ENVELOPE_RE.match(full), (
        f"envelope {full!r} violates the `daemon: migration: <kebab-case>` shape"
    )
    assert kind == kind.lower(), (
        f"kind {kind!r} must be all-lowercase for cross-binding consistency"
    )
    assert "_" not in kind, f"kind {kind!r} must be kebab-case, not snake_case"


def test_group_error_kind_symmetric_when_available() -> None:
    # The `group` feature is optional — if the binding wasn't
    # built with it, there's no helper to test. Under the feature,
    # the parser must share shape with `migration_error_kind`.
    if group_error_kind is None:
        pytest.skip("net._net built without `groups` feature")

    exc = _FakeExc("daemon: group: split-brain")
    assert group_error_kind(exc) == "split-brain"

    exc = _FakeExc("daemon: group: split-brain: detail")
    assert group_error_kind(exc) == "split-brain"

    assert group_error_kind(_FakeExc("not a group error")) is None
