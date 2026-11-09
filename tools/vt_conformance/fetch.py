#!/usr/bin/env python3
"""Fetch the vte parser expectations used by Extensions/Vt/tests/conformance.jai.

PYTHONPATH=tools python3 tools/vt_conformance/fetch.py
"""
from toolkit.download import pinned

REVISION = "89c9c22f5b8fcc3feba0a9d9f50a020801e74acc"


def fetch():
    """Return the hash-verified source containing the public parser test cases."""
    return pinned(
        f"https://raw.githubusercontent.com/alacritty/vte/{REVISION}/src/lib.rs",
        "1c5312f33f985114bf461f81d71c63ae81a3613712c1be0f78c3b65d9bc2750c",
        "vt-conformance/vte-tests.rs",
    )


if __name__ == "__main__":
    print(fetch())
