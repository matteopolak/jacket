#!/usr/bin/env python3
"""Run jaic over test programs and summarize results.

Usage: tools/jaic-sweep.py [--jaic PATH] [--filter TEXT] [--verbose] SET...
Sets: corpus (tests/corpus/positive with expected runtime), negative (tests/corpus/negative:
`jaic check` must fail and report the recorded text), stdlib (tests/stdlib,
run must succeed), modules (the stdlib's own tests: stdlib/tests and stdlib/<Module>/tests,
run must succeed; a test directory's `modules/` folder holds its mock modules), examples (tests/examples.json:
example programs such as examples/tour, whose stdout must contain the listed lines), upstream
(tools/upstream-cases.json: upstream project entry points that must pass), or file paths.
"""
import argparse, json, os, subprocess, sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

def cases(name):
    if name == "corpus":
        m = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
        for c in m["cases"]:
            if "runtime" in c:
                yield c["id"], ROOT / "tests/corpus" / c["source"], "run", c["runtime"], []
    elif name == "negative":
        m = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
        for c in m["cases"]:
            if c.get("kind") == "negative":
                yield c["id"], ROOT / "tests/corpus" / c["source"], "check", {"negative": next(iter(c["negative"].values()))}, []
    elif name == "stdlib":
        for p in sorted((ROOT / "tests/stdlib").glob("*.jai")):
            yield p.stem, p, "run", None, []
    elif name == "modules":
        found = list((ROOT / "stdlib").glob("*/tests/*.jai")) + list((ROOT / "stdlib/tests").glob("**/*.jai"))
        for p in sorted(set(found)):
            if "modules" not in p.relative_to(ROOT / "stdlib").parts:
                yield str(p.relative_to(ROOT / "stdlib").with_suffix("")), p, "run", None, []
    elif name == "examples":
        for c in json.loads((ROOT / "tests/examples.json").read_text())["cases"]:
            expect = {"contains": c["stdout_contains"], "excludes": c.get("stdout_excludes", [])}
            yield c["id"], ROOT / c["directory"] / c["main"], "run", expect, c.get("args", [])
    elif name == "upstream":
        for c in json.loads((ROOT / "tools/upstream-cases.json").read_text()):
            yield c["id"], ROOT / "corpus/upstream" / c["path"], c["mode"], None, c.get("args", [])
    else:
        p = Path(name)
        yield p.stem, p, "run", None, []

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("sets", nargs="+")
    target = os.environ.get("CARGO_TARGET_DIR") or str(ROOT / "target")
    ap.add_argument("--jaic", default=os.path.join(target, "debug", "jaic"))
    ap.add_argument("--filter", default="")
    ap.add_argument("--verbose", "-v", action="store_true")
    ap.add_argument("--timeout", type=float, default=60)
    ap.add_argument("--jobs", "-j", type=int, default=os.cpu_count() or 1,
                    help="cases run at once (default: CPU count)")
    a = ap.parse_args()
    todo = [c for s in a.sets for c in cases(s) if a.filter in c[0]]
    # Upstream cases may name setup commands (building a C library the program loads); they run
    # once, serially, in the case's directory before any case starts.
    if "upstream" in a.sets:
        for c in json.loads((ROOT / "tools/upstream-cases.json").read_text()):
            if a.filter in c["id"]:
                for command in c.get("setup", []):
                    subprocess.run(command, cwd=(ROOT / "corpus/upstream" / c["path"]).parent,
                                   capture_output=True, stdin=subprocess.DEVNULL)

    def run(case):
        cid, path, mode, expect, extra = case
        try:
            r = subprocess.run([a.jaic, mode, str(path), *extra], capture_output=True, timeout=a.timeout,
                               cwd=path.parent, stdin=subprocess.DEVNULL)
            out, err, code = r.stdout.decode(errors="replace"), r.stderr.decode(errors="replace"), r.returncode
        except subprocess.TimeoutExpired:
            out, err, code = "", "timeout", -1
        if expect is None:
            ok = code == 0
        elif "negative" in expect:
            ok = code != 0 and code != -1 and expect["negative"] in err
        elif "contains" in expect:
            missing = [line for line in expect["contains"] if line not in out]
            present = [text for text in expect["excludes"] if text in out]
            ok = code == 0 and not missing and not present
            if code == 0 and not ok:
                err += "".join(f"\nerror: stdout lacks {line!r}" for line in missing)
                err += "".join(f"\nerror: stdout has {text!r}" for text in present)
        else:
            ok = code == expect.get("exit_code", 0) and out == expect.get("stdout", "")
        return cid, ok, out, err, code

    passed, failed = 0, []
    with ThreadPoolExecutor(max_workers=max(1, a.jobs)) as pool:
        for cid, ok, out, err, code in pool.map(run, todo):
            if ok:
                passed += 1
                continue
            first = next((l for l in err.splitlines() if "error" in l), err.strip().splitlines()[0] if err.strip() else f"exit {code}, stdout {out[:80]!r}")
            failed.append((cid, first))
            if a.verbose:
                print(f"--- {cid}\n{err[:1500]}{out[:500]}")
    for cid, msg in failed:
        print(f"FAIL {cid}: {msg[:220]}")
    print(f"\n{passed} passed, {len(failed)} failed")

if __name__ == "__main__":
    main()
