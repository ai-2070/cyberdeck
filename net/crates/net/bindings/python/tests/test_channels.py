"""Tests for the channel (distributed pub/sub) surface on NetMesh.

Exercises the standalone register/error-classification paths plus the
`ChannelAuthError` subclass relationship. End-to-end two-node
handshake tests are deferred — they require a dedicated fixture that
doesn't race port allocation, and the Python SDK doesn't yet surface
`local_addr()` to the caller (same gap flagged for the TS SDK).
"""

import pytest

from net import ChannelAuthError, ChannelError, NetMesh


PSK = "42" * 32


def _port(seed: int) -> str:
    return f"127.0.0.1:{27000 + seed}"


def test_register_channel_accepts_valid_config() -> None:
    m = NetMesh(_port(1), PSK)
    try:
        m.register_channel(
            "sensors/temp",
            visibility="global",
            reliable=True,
            require_token=False,
            priority=3,
            max_rate_pps=1000,
        )
    finally:
        m.shutdown()


def test_register_channel_accepts_all_visibility_variants() -> None:
    m = NetMesh(_port(2), PSK)
    try:
        for visibility in (
            "subnet-local",
            "parent-visible",
            "exported",
            "global",
        ):
            m.register_channel(f"v/{visibility}", visibility=visibility)
    finally:
        m.shutdown()


def test_invalid_visibility_raises_channel_error() -> None:
    m = NetMesh(_port(3), PSK)
    try:
        with pytest.raises(ChannelError):
            m.register_channel("bad", visibility="nonsense")
    finally:
        m.shutdown()


def test_invalid_channel_name_raises_channel_error() -> None:
    m = NetMesh(_port(4), PSK)
    try:
        with pytest.raises(ChannelError):
            m.register_channel("has spaces which are invalid")
    finally:
        m.shutdown()


def test_channel_auth_error_subclass_of_channel_error() -> None:
    # `except ChannelError:` should catch both classes — this is the
    # user-facing contract.
    assert issubclass(ChannelAuthError, ChannelError)
    err = ChannelAuthError("unauthorized")
    assert isinstance(err, ChannelError)
    assert isinstance(err, Exception)


def test_publish_on_unregistered_channel_with_no_subscribers() -> None:
    # Publishing when no one is subscribed still succeeds — the
    # report reflects an empty roster.
    m = NetMesh(_port(5), PSK)
    try:
        m.register_channel("chan/quiet", visibility="global")
        report = m.publish(
            "chan/quiet",
            b"hello",
            reliability="reliable",
            on_failure="best_effort",
            max_inflight=8,
        )
        assert report["attempted"] == 0
        assert report["delivered"] == 0
        assert report["errors"] == []
    finally:
        m.shutdown()


def test_publish_rejects_invalid_reliability() -> None:
    m = NetMesh(_port(6), PSK)
    try:
        m.register_channel("chan/x")
        with pytest.raises(ChannelError):
            m.publish("chan/x", b"p", reliability="whatever")
    finally:
        m.shutdown()
