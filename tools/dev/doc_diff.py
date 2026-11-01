#!/usr/bin/env python3
"""Compare built documentation sites byte for byte, grouping differences by page kind."""

import argparse
from collections import Counter
import os
from pathlib import Path
import re
import sys

KINDS = ("index", "module", "item", "guide", "other")


def inventory(root):
    """Use relative POSIX paths for stable reports without opening every file twice."""
    return {
        (Path(folder) / name).relative_to(root).as_posix()
        for folder, _, names in os.walk(root)
        for name in names
    }


def page_kind(path):
    if path == "index.html":
        return "index"
    if path.startswith("api/") and path.endswith("/index.html"):
        return "module"
    if path.startswith("api/") and path.endswith(".html"):
        return "item"
    if path.endswith(".html"):
        return "guide"
    return "other"


def classify(path, data):
    kind = page_kind(path)
    if kind == "module":
        # Both module and item URLs end in /index.html. The titlebar is stable even
        # for an empty module; prose can also contain the word "module".
        titlebar = re.search(rb'<div class="titlebar">[^\n]*', data)
        if titlebar and b'<span class="badge">module</span>' in titlebar[0]:
            return "module"
        return "item"
    if kind == "guide" and not (b'<div class="prose guide">' in data or b'<h1 class="page-title">' in data):
        return "other"
    return kind


def first_difference(old, new):
    """Return a bounded window around the first differing byte, with line numbers."""
    common = min(len(old), len(new))
    at = 0
    while at < common and old[at] == new[at]:
        at += 1
    def context(data):
        start = max(data.rfind(b"\n", 0, at) + 1, at - 100)
        end = data.find(b"\n", at)
        if end < 0:
            end = len(data)
        end = min(end, at + 101)
        return data.count(b"\n", 0, at) + 1, data[start:end].decode("utf-8", "replace")
    return context(old), context(new)


def compare(old_root, new_root, limit=3, ignores=()):
    old_paths, new_paths = inventory(old_root), inventory(new_root)
    missing = False
    for label, root, paths in (("only OLD", old_root, old_paths - new_paths),
                                ("only NEW", new_root, new_paths - old_paths)):
        grouped = {kind: [] for kind in KINDS}
        for path in sorted(paths):
            grouped[classify(path, (root / path).read_bytes())].append(path)
        for kind in KINDS:
            for path in grouped[kind]:
                print(f"{label} [{kind}]: {path}")
                missing = True
    counts = Counter()
    for path in sorted(old_paths & new_paths):
        old = (old_root / path).read_bytes()
        new = (new_root / path).read_bytes()
        kind = classify(path, old)
        if old == new:
            continue
        if ignores:
            # Decode only differing files; equality before normalization is sufficient.
            a, b = old.decode("utf-8", "surrogateescape"), new.decode("utf-8", "surrogateescape")
            for pattern in ignores:
                a, b = pattern.sub("", a), pattern.sub("", b)
            old, new = a.encode("utf-8", "surrogateescape"), b.encode("utf-8", "surrogateescape")
            if old == new:
                continue
        counts[kind] += 1
        if counts[kind] <= limit:
            print(f"different [{kind}]: {path}")
            for label, (line, snippet) in zip(("OLD", "NEW"), first_difference(old, new)):
                print(f"  {label} line {line}: {snippet}")
    print("Differences: " + ", ".join(f"{kind}={counts[kind]}" for kind in KINDS))
    if not missing and not counts:
        print(f"Identical: {len(old_paths)} files")
    return int(missing or bool(counts))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("old", type=Path)
    parser.add_argument("new", type=Path)
    parser.add_argument("--limit", type=int, default=3)
    parser.add_argument("--ignore", nargs="+", action="extend", default=[], metavar="REGEX")
    args = parser.parse_args()
    if args.limit < 0:
        parser.error("--limit must be nonnegative")
    if not args.old.is_dir() or not args.new.is_dir():
        parser.error("OLD and NEW must both be directories")
    try:
        patterns = [re.compile(pattern) for pattern in args.ignore]
    except re.error as error:
        parser.error(str(error))
    return compare(args.old, args.new, args.limit, patterns)


if __name__ == "__main__":
    sys.exit(main())
