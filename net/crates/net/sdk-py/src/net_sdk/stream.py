"""Streaming event consumption — generators and async generators."""

from __future__ import annotations

import json
import time
from dataclasses import dataclass
from typing import Any, Callable, Generic, Iterator, Optional, TypeVar

from net import Net, StoredEvent

T = TypeVar("T")

DEFAULT_LIMIT = 100
DEFAULT_POLL_INTERVAL = 0.001  # 1ms
DEFAULT_MAX_BACKOFF = 0.1  # 100ms


@dataclass
class SubscribeOpts:
    """Options for subscribing to events."""

    limit: int = DEFAULT_LIMIT
    filter: Optional[str] = None
    ordering: Optional[str] = None
    poll_interval: float = DEFAULT_POLL_INTERVAL
    max_backoff: float = DEFAULT_MAX_BACKOFF
    timeout: Optional[float] = None


class EventStream:
    """
    A synchronous generator-based event stream.

    Polls the bus with adaptive backoff — tight loop when events flow,
    exponential backoff when idle.

    Example:
        >>> for event in node.subscribe(limit=100):
        ...     print(event.raw)
    """

    def __init__(self, bus: Net, opts: Optional[SubscribeOpts] = None) -> None:
        self._bus = bus
        self._opts = opts or SubscribeOpts()
        self._cursor: Optional[str] = None
        self._stopped = False

    def stop(self) -> None:
        """Stop the stream."""
        self._stopped = True

    def __iter__(self) -> Iterator[StoredEvent]:
        backoff = self._opts.poll_interval
        start = time.monotonic()

        while not self._stopped:
            if self._opts.timeout is not None:
                elapsed = time.monotonic() - start
                if elapsed >= self._opts.timeout:
                    return

            response = self._bus.poll(
                limit=self._opts.limit,
                cursor=self._cursor,
                filter=self._opts.filter,
                ordering=self._opts.ordering,
            )

            if len(response) > 0:
                backoff = self._opts.poll_interval
                self._cursor = response.next_id

                for event in response:
                    yield event
            else:
                time.sleep(backoff)
                backoff = min(backoff * 2, self._opts.max_backoff)


class TypedEventStream(Generic[T]):
    """
    A typed synchronous event stream that deserializes events.

    Example:
        >>> for reading in node.subscribe_typed(model=TemperatureReading):
        ...     print(f'{reading.sensor_id}: {reading.celsius}°C')
    """

    def __init__(
        self,
        bus: Net,
        parse: Callable[[str], T],
        opts: Optional[SubscribeOpts] = None,
    ) -> None:
        self._inner = EventStream(bus, opts)
        self._parse = parse

    def stop(self) -> None:
        """Stop the stream."""
        self._inner.stop()

    def __iter__(self) -> Iterator[T]:
        for event in self._inner:
            yield self._parse(event.raw)
