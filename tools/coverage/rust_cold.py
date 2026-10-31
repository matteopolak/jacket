#!/usr/bin/env python3
"""Summarize a Rust lcov file (`cargo llvm-cov report --lcov`) as the coldest code, for choosing what to test next.

Usage: tools/coverage/rust_cold.py LCOV [--root crates/jaic/src] [--top 25] [--markdown] [--functions]

Prints coverage per area (the directories one level under each `--root`, so `sema`, `interp`, `parser`, ...)
and the files with the most uncovered lines. `--functions` lists the functions that never ran, largest source
spans first, which is where a new test pays most (docs/tools/rust-coverage.md).
"""

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path


def parse(text: str):
    """{file: {"lines": {line: hits}, "functions": {name: (line, hits)}}} from lcov text (a file may repeat; counts add)."""
    files = defaultdict(lambda: {"lines": defaultdict(int), "functions": {}})
    current = None
    starts = {}
    for raw in text.splitlines():
        if raw.startswith("SF:"):
            current = files[raw[3:]]
            starts = {}
        elif current is None:
            continue
        elif raw.startswith("DA:"):
            line, hits = raw[3:].split(",")[:2]
            current["lines"][int(line)] += int(hits)
        elif raw.startswith("FN:"):
            line, name = raw[3:].split(",", 1)
            starts[name] = int(line)
            current["functions"].setdefault(name, (int(line), 0))
        elif raw.startswith("FNDA:"):
            hits, name = raw[5:].split(",", 1)
            line = starts.get(name, 0)
            current["functions"][name] = (
                line,
                current["functions"].get(name, (line, 0))[1] + int(hits),
            )
    return files


def relative(path: str, roots):
    for root in roots:
        marker = root.rstrip("/") + "/"
        if marker in path:
            return root, path[path.index(marker) + len(marker) :]
    return None, path


def area_table(files, roots):
    areas = defaultdict(lambda: [0, 0])
    for path, data in files.items():
        root, rest = relative(path, roots)
        if root is None:
            continue
        area = f"{root}/{rest.split('/')[0]}" if "/" in rest else f"{root} (top level)"
        areas[area][0] += sum(1 for h in data["lines"].values() if h)
        areas[area][1] += len(data["lines"])
    return sorted(
        ((a, hit, total) for a, (hit, total) in areas.items()),
        key=lambda r: (r[1] / max(r[2], 1), r[0]),
    )


def cold_files(files, roots, top):
    rows = []
    for path, data in files.items():
        if relative(path, roots)[0] is None:
            continue
        missed = sum(1 for h in data["lines"].values() if not h)
        rows.append((missed, len(data["lines"]), path))
    return sorted(rows, reverse=True)[:top]


def cold_functions(files, roots, top):
    rows = []
    for path, data in files.items():
        if relative(path, roots)[0] is None:
            continue
        lines = sorted(data["lines"])
        for name, (line, hits) in data["functions"].items():
            if hits == 0 and line:
                # Length: lines from the function's start to the next function's start.
                later = [
                    start for start, _ in data["functions"].values() if start > line
                ]
                end = min(later) if later else (lines[-1] + 1 if lines else line)
                rows.append((end - line, path, line, name))
    return sorted(rows, reverse=True)[:top]


def demangle(name: str) -> str:
    return re.sub(r"17h[0-9a-f]{16}E$", "", name)


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("lcov", type=Path)
    parser.add_argument(
        "--root",
        action="append",
        default=None,
        help="source directory to report on (default: crates/*/src as given; repeatable)",
    )
    parser.add_argument("--top", type=int, default=25)
    parser.add_argument("--markdown", action="store_true")
    parser.add_argument("--functions", action="store_true")
    args = parser.parse_args()
    roots = args.root or [
        "crates/jaic/src",
        "crates/jaic-syntax/src",
        "crates/jaic-cli/src",
        "crates/jaic-llvm/src",
        "crates/jai-language-server/src",
        "crates/jailint/src",
    ]
    files = parse(args.lcov.read_text())
    md = args.markdown
    print(
        "| area | lines hit | total | % |\n|---|---:|---:|---:|"
        if md
        else f"{'area':<44}{'hit':>9}{'total':>9}{'%':>7}"
    )
    for area, hit, total in area_table(files, roots):
        pct = 100 * hit / total if total else 0
        print(
            f"| `{area}` | {hit} | {total} | {pct:.1f} |"
            if md
            else f"{area:<44}{hit:>9}{total:>9}{pct:>7.1f}"
        )
    print()
    print(
        f"Files with the most uncovered lines (top {args.top}):\n"
        if md
        else f"files with the most uncovered lines (top {args.top}):"
    )
    if md:
        print("| file | uncovered | lines |\n|---|---:|---:|")
    for missed, total, path in cold_files(files, roots, args.top):
        print(
            f"| `{path}` | {missed} | {total} |"
            if md
            else f"{missed:>7} of {total:<6} {path}"
        )
    if args.functions:
        print()
        print(
            "Functions that never ran, longest first:"
            if not md
            else "Functions that never ran, longest first:\n\n| lines | function | file |\n|---:|---|---|"
        )
        for length, path, line, name in cold_functions(files, roots, args.top):
            print(
                f"| {length} | `{demangle(name)}` | `{path}:{line}` |"
                if md
                else f"{length:>6} lines  {demangle(name)}  {path}:{line}"
            )
    return 0


if __name__ == "__main__":
    sys.exit(main())
