"""Typed channels — strongly typed pub/sub over named channels."""

from __future__ import annotations

import json
from typing import Callable, Generic, Iterator, Optional, TypeVar

from net import Net

from net_sdk.stream import EventStream, SubscribeOpts, TypedEventStream

T = TypeVar("T")


class TypedChannel(Generic[T]):
    """
    A strongly typed channel for publishing and subscribing to events.

    Example:
        >>> temps = node.channel('sensors/temperature', TemperatureReading)
        >>> temps.publish(TemperatureReading(sensor_id='A1', celsius=22.5))
        >>> for reading in temps.subscribe():
        ...     print(f'{reading.sensor_id}: {reading.celsius}°C')
    """

    def __init__(
        self,
        bus: Net,
        name: str,
        model: Optional[type] = None,
        parse: Optional[Callable[[str], T]] = None,
    ) -> None:
        self._bus = bus
        self._name = name
        self._model = model
        self._parse = parse

    @property
    def name(self) -> str:
        """The channel name."""
        return self._name

    def publish(self, event: T) -> None:
        """Publish a typed event to this channel."""
        if hasattr(event, "model_dump"):
            # Pydantic model
            data = event.model_dump()  # type: ignore[union-attr]
        elif hasattr(event, "__dict__"):
            data = event.__dict__
        else:
            data = event  # type: ignore[assignment]

        data["_channel"] = self._name  # type: ignore[index]
        self._bus.ingest_raw(json.dumps(data))

    def publish_batch(self, events: list[T]) -> int:
        """Publish a batch of typed events. Returns number ingested."""
        payloads = []
        for event in events:
            if hasattr(event, "model_dump"):
                data = event.model_dump()  # type: ignore[union-attr]
            elif hasattr(event, "__dict__"):
                data = event.__dict__
            else:
                data = event  # type: ignore[assignment]

            data["_channel"] = self._name  # type: ignore[index]
            payloads.append(json.dumps(data))
        return self._bus.ingest_raw_batch(payloads)

    def subscribe(self, opts: Optional[SubscribeOpts] = None) -> TypedEventStream[T]:
        """Subscribe to typed events on this channel."""
        filter_str = json.dumps({"path": "_channel", "value": self._name})
        merged = opts or SubscribeOpts()
        if merged.filter is None:
            merged.filter = filter_str

        if self._parse is not None:
            parse_fn = self._parse
        elif self._model is not None:
            model = self._model

            def parse_fn(raw: str) -> T:
                data = json.loads(raw)
                return model(**data)  # type: ignore[return-value]
        else:

            def parse_fn(raw: str) -> T:
                return json.loads(raw)  # type: ignore[return-value]

        return TypedEventStream(self._bus, parse_fn, merged)

    def subscribe_raw(self, opts: Optional[SubscribeOpts] = None) -> EventStream:
        """Subscribe to raw events on this channel."""
        filter_str = json.dumps({"path": "_channel", "value": self._name})
        merged = opts or SubscribeOpts()
        if merged.filter is None:
            merged.filter = filter_str
        return EventStream(self._bus, merged)
