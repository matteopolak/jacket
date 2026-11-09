#!/usr/bin/env python3
"""Measure pyte's byte-stream parser on the recorded Vt benchmark input.

python3 stdlib/Extensions/Vt/bench/reference.py /tmp/vt-mixed.bin
Requires pyte; uses a counting listener, with no terminal grid.
"""
import argparse
import importlib.metadata
import time

import pyte


class Counter:
    def __init__(self):
        self.actions = 0

    def draw(self, text):
        self.actions += len(text)

    def command(self, *args, **kwargs):
        self.actions += 1

    def __getattr__(self, name):
        return self.command


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("recording")
    args = parser.parse_args()
    with open(args.recording, "rb") as source:
        recording = source.read()
    times = []
    for _ in range(5):
        listener = Counter()
        stream = pyte.ByteStream(listener)
        start = time.perf_counter()
        for _ in range(8):
            stream.feed(recording)
        times.append(time.perf_counter() - start)
    print(f"pyte {importlib.metadata.version('pyte')}: "
          f"{len(recording) * 8 / min(times) / 1e6:.2f} MB/s, "
          f"{len(recording)} bytes, {listener.actions // 8} actions/pass "
          "(best of 5, 8 passes)")


if __name__ == "__main__":
    main()
