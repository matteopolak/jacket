#!/usr/bin/env python3
"""Time Python NFC and optional regex graphemes on bench.jai's identical corpus."""
import time
import unicodedata

text = "Café Καλημέρα 한글 👩‍👩‍👧‍👦\n" * 32768
size = len(text.encode())


def measure(name, work):
    best = float("inf")
    for _ in range(5):
        start = time.perf_counter()
        work()
        best = min(best, time.perf_counter() - start)
    print(f"{name}: {size / best / 1e6:.3f} MB/s ({size} bytes, {best:.6f} s best of 5)")


print(f"Python unicodedata UCD {unicodedata.unidata_version}")
measure("NFC", lambda: unicodedata.normalize("NFC", text))
try:
    import regex
except ImportError:
    print("regex not installed; grapheme comparison skipped")
else:
    pattern = regex.compile(r"\X")
    measure("graphemes", lambda: sum(1 for _ in pattern.finditer(text)))
