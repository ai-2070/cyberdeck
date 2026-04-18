"""
net-sdk — Ergonomic Python SDK for the Net mesh network.

Example:
    >>> from net_sdk import NetNode
    >>> node = NetNode(shards=4)
    >>> node.emit({'token': 'hello', 'index': 0})
    >>> for event in node.subscribe():
    ...     print(event.raw)
    >>> node.shutdown()
"""

from net_sdk.node import NetNode
from net_sdk.stream import EventStream, TypedEventStream
from net_sdk.channel import TypedChannel
from net_sdk.mesh import (
    MeshNode,
    MeshStream,
    StreamStats,
    Reliability,
    BackpressureError,
    NotConnectedError,
)

__all__ = [
    "NetNode",
    "EventStream",
    "TypedEventStream",
    "TypedChannel",
    "MeshNode",
    "MeshStream",
    "StreamStats",
    "Reliability",
    "BackpressureError",
    "NotConnectedError",
]

__version__ = "0.1.0"
