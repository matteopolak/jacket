#!/usr/bin/env python3
"""Shrink a program on which jaic's backends disagree, keeping the disagreement.

Usage: tools/jaic-reduce.py PROGRAM.jai [--out SMALL.jai] [backend options as for tools/jaic-diff.py]

A candidate is kept when it still compiles and the backends still split into the same groups
(say interp+wasm against native+native-O2). The reduction is line based, which suits tools/jaigen.py
output (one statement per line, blocks opened with `{` at the end of a line):
  1. delete chunks of lines, halving the chunk size down to one line (ddmin-style);
  2. delete whole blocks: a line ending in `{` through its matching `}`;
  3. unwrap blocks: keep a block's body but drop its `if ... {` / `}` lines;
repeated until nothing changes. The result is written next to the input as NAME.reduced.jai.
"""
import argparse, importlib.util, shutil, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location("jaic_diff", ROOT / "tools/jaic-diff.py")
diff = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(diff)


def signature(results):
    """Which backends agree with which: a set of frozensets of backend names (None: no disagreement)."""
    live = {b: r for b, r in results.items() if r.status != "unsupported"}
    if any(r.status == "compile error" for r in live.values()):
        return None
    with_stderr = all(r.status.startswith("exit") for r in live.values())
    groups = {}
    for b, r in live.items():
        groups.setdefault(r.key(with_stderr), set()).add(b)
    if len(groups) < 2:
        return None
    return frozenset(frozenset(g) for g in groups.values())


def blocks(lines):
    """(start, end) line ranges of brace blocks, innermost last."""
    stack, found = [], []
    for i, line in enumerate(lines):
        text = line.strip()
        if text.startswith("}") and stack:
            found.append((stack.pop(), i))
        if text.endswith("{"):
            stack.append(i)
    return found


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("program")
    ap.add_argument("--out")
    diff.add_backend_arguments(ap)
    # Deleting a loop's counter update makes it spin: give up on such candidates quickly.
    ap.set_defaults(timeout=10)
    a = ap.parse_args()
    backends = diff.chosen_backends(a)
    source = Path(a.program).resolve()
    work = Path(tempfile.mkdtemp(prefix="jaic-reduce-"))
    runner = diff.Runner(a.jaic, backends, work, a.wasm, a.timeout, a.memory_limit)
    candidate = work / "case" / source.name
    candidate.parent.mkdir()
    tries = 0

    def check(lines):
        nonlocal tries
        tries += 1
        candidate.write_text("\n".join(lines) + "\n")
        _, results = runner.run(diff.Case("reduce", candidate))
        return signature(results)

    lines = source.read_text().splitlines()
    want = check(lines)
    if want is None:
        sys.exit("jaic-reduce: the backends agree on this program (or it does not compile); nothing to reduce")
    print(f"disagreement: {' vs '.join('+'.join(sorted(g)) for g in want)}; {len(lines)} lines", flush=True)

    def attempt(new):
        nonlocal lines
        if len(new) < len(lines) and check(new) == want:
            lines = new
            return True
        return False

    changed = True
    while changed:
        changed = False
        chunk = max(1, len(lines) // 2)
        while chunk >= 1:
            i = 0
            while i < len(lines):
                if attempt(lines[:i] + lines[i + chunk:]):
                    changed = True
                else:
                    i += chunk
            chunk //= 2
        for start, end in sorted(blocks(lines), key=lambda r: r[0] - r[1]):
            if end >= len(lines) or not lines[start].rstrip().endswith("{"):
                continue
            if attempt(lines[:start] + lines[end + 1:]) or \
                    attempt(lines[:start] + lines[start + 1:end] + lines[end + 1:]):
                changed = True
                break
        print(f"  {len(lines)} lines after {tries} runs", flush=True)
    out = Path(a.out) if a.out else source.with_suffix(".reduced.jai")
    out.write_text("\n".join(lines) + "\n")
    shutil.rmtree(work, ignore_errors=True)
    print(f"wrote {out} ({len(lines)} lines, {tries} runs)")


if __name__ == "__main__":
    main()
