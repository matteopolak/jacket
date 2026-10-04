#!/usr/bin/env python3
"""Run jaic over test programs and summarize results.

Usage: tools/jaic-sweep.py [--jaic PATH] [--filter TEXT] [--verbose] SET...
Sets: corpus (tests/corpus/positive with expected runtime), stdlib (tests/stdlib,
run must succeed), upstream
(tools/upstream-cases.json: upstream project entry points that must pass), or file paths.
"""
import argparse, json, os, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

def cases(name):
    if name == "corpus":
        m = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
        for c in m["cases"]:
            if "runtime" in c:
                yield c["id"], ROOT / "tests/corpus" / c["source"], "run", c["runtime"], []
    elif name == "stdlib":
        for p in sorted((ROOT / "tests/stdlib").glob("*.jai")):
            yield p.stem, p, "run", None, []
    elif name == "upstream":
        for c in json.loads((ROOT / "tools/upstream-cases.json").read_text()):
            yield c["id"], ROOT / "corpus/upstream" / c["path"], c["mode"], None, c.get("args", [])
    else:
        p = Path(name)
        yield p.stem, p, "run", None, []

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("sets", nargs="+")
    ap.add_argument("--jaic", default="/Volumes/CodexBuilds/targets/jai-dev/debug/jaic")
    ap.add_argument("--filter", default="")
    ap.add_argument("--verbose", "-v", action="store_true")
    ap.add_argument("--timeout", type=float, default=60)
    a = ap.parse_args()
    passed, failed = 0, []
    for s in a.sets:
        for cid, path, mode, expect, extra in cases(s):
            if a.filter not in cid:
                continue
            try:
                r = subprocess.run([a.jaic, mode, str(path), *extra], capture_output=True, timeout=a.timeout, cwd=path.parent)
                out, err, code = r.stdout.decode(errors="replace"), r.stderr.decode(errors="replace"), r.returncode
            except subprocess.TimeoutExpired:
                out, err, code = "", "timeout", -1
            ok = code == (expect or {}).get("exit_code", 0) and (expect is None or out == expect.get("stdout", ""))
            if expect is None:
                ok = code == 0
            if ok:
                passed += 1
            else:
                first = next((l for l in err.splitlines() if "error" in l), err.strip().splitlines()[0] if err.strip() else f"exit {code}, stdout {out[:80]!r}")
                failed.append((cid, first))
                if a.verbose:
                    print(f"--- {cid}\n{err[:1500]}{out[:500]}")
    for cid, msg in failed:
        print(f"FAIL {cid}: {msg[:220]}")
    print(f"\n{passed} passed, {len(failed)} failed")

if __name__ == "__main__":
    main()
