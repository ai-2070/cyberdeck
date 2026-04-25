"""Regression: TypedChannel.subscribe() must not mutate caller's SubscribeOpts.

The previous implementation aliased the user's `SubscribeOpts` instance
(`merged = opts or SubscribeOpts()`) and then assigned `merged.filter =
self._filter` in place. Because `SubscribeOpts` is a mutable dataclass,
the caller's object then carried the first channel's filter, so passing
the same opts to a second channel silently delivered the wrong channel's
events.

The TS SDK got this right via spread-copy (`{ ...opts, filter: ... }`);
the Python side regressed and these tests pin the fix.
"""

from __future__ import annotations

from net_sdk.channel import TypedChannel
from net_sdk.stream import SubscribeOpts


class _FakeBus:
    """Minimal stand-in for `net.Net`. `TypedChannel` only stores the
    bus during construction and delegates to it when the stream is
    iterated; the subscribe-time behavior we test here never touches
    the bus."""


def _make_channel(name: str) -> TypedChannel:
    return TypedChannel(_FakeBus(), name)


def test_subscribe_does_not_mutate_caller_opts() -> None:
    opts = SubscribeOpts(limit=50)
    assert opts.filter is None  # precondition

    channel = _make_channel("sensors/temperature")
    channel.subscribe(opts)

    assert opts.filter is None, (
        "subscribe() mutated the caller's SubscribeOpts.filter — passing "
        "the same opts to a second channel will now deliver the wrong "
        "channel's events"
    )


def test_subscribe_raw_does_not_mutate_caller_opts() -> None:
    opts = SubscribeOpts(limit=50)
    assert opts.filter is None  # precondition

    channel = _make_channel("sensors/temperature")
    channel.subscribe_raw(opts)

    assert opts.filter is None, (
        "subscribe_raw() mutated the caller's SubscribeOpts.filter"
    )


def test_subscribe_reuses_opts_across_channels_without_crosstalk() -> None:
    """End-to-end repro of the reported failure mode.

    Calling `subscribe(opts)` on channel A and then `subscribe(opts)` on
    channel B must result in two streams whose effective filters point
    at A and B respectively. With the bug, B's stream observed A's
    filter because A had mutated `opts.filter`.
    """
    opts = SubscribeOpts(limit=50)

    temps = _make_channel("sensors/temperature")
    humidity = _make_channel("sensors/humidity")

    temps_stream = temps.subscribe(opts)
    humidity_stream = humidity.subscribe(opts)

    # The streams expose `_opts.filter`; cross-check both ended up with
    # the channel-specific filter, not whichever was set first.
    assert "temperature" in (temps_stream._inner._opts.filter or "")
    assert "humidity" in (humidity_stream._inner._opts.filter or ""), (
        "second channel inherited the first channel's filter — "
        "TypedChannel.subscribe() leaked state through the shared opts"
    )


def test_subscribe_preserves_caller_supplied_filter() -> None:
    """If the caller already set `opts.filter`, the channel must NOT
    overwrite it. The fix copies opts before mutating, but the
    "fill in only when None" behavior must survive that copy."""
    custom_filter = '{"path":"custom","value":"override"}'
    opts = SubscribeOpts(limit=50, filter=custom_filter)

    channel = _make_channel("sensors/temperature")
    stream = channel.subscribe(opts)

    assert stream._inner._opts.filter == custom_filter, (
        "channel overrode a caller-supplied filter"
    )
    assert opts.filter == custom_filter, "caller's opts.filter was mutated"


def test_subscribe_with_none_opts_uses_default() -> None:
    """Sanity: passing None still yields a stream with the channel's
    filter installed (no regression on the default path)."""
    channel = _make_channel("sensors/temperature")
    stream = channel.subscribe(None)

    assert stream._inner._opts.filter is not None
    assert "temperature" in stream._inner._opts.filter
