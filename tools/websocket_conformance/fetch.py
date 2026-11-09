#!/usr/bin/env python3
"""Fetch the pinned pure-Python WebSocket interop peer into the download cache.

Example: PYTHONPATH=tools python3 tools/websocket_conformance/fetch.py
The wheel can be used directly on PYTHONPATH; no installation is needed.
"""
from ci.pins import load
from toolkit.download import pinned


def fetch():
    """Return the SHA-256 verified peer wheel, e.g. for an interop subprocess's PYTHONPATH."""
    pins = load()
    name = f"websockets-{pins['websockets']}-py3-none-any.whl"
    return pinned(
        "https://files.pythonhosted.org/packages/fa/a8/5b41e0da817d64113292ab1f8247140aac61cbf6cfd085d6a0fa77f4984f/" + name,
        pins["websockets_sha256"], "websocket/" + name,
    )


if __name__ == "__main__":
    print(fetch())
