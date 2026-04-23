"""Minimal sanity check for net-sdk.

Run: python hello.py

What it proves: the SDK installs, a node starts, you can publish to a named
channel, a subscriber receives it, and shutdown is clean.
"""

import threading
import time
from dataclasses import dataclass

from net_sdk import NetNode


@dataclass
class Hello:
    msg: str


def main() -> None:
    with NetNode(shards=1) as node:
        ch = node.channel("hello/world", Hello)
        received: list[Hello] = []

        # Subscribe FIRST, in a background thread. Subscriptions are hot.
        def consume() -> None:
            for ev in ch.subscribe():
                received.append(ev)
                return

        t = threading.Thread(target=consume, daemon=True)
        t.start()

        time.sleep(0.05)  # let the subscriber's first poll land
        ch.publish(Hello(msg="hello, mesh"))

        t.join(timeout=2.0)
        assert received, "subscriber did not receive event"
        print("received:", received[0])


if __name__ == "__main__":
    main()
