#!/usr/bin/env python3
"""Time jaic on the micro-benchmarks in benchmarks/ and on the larger corpus projects.

    python3 tools/bench.py [--jaic PATH] [--repeat 3] [--only NAME] [--out bench.json] [--compare old.json]

Each workload reports the median wall time over --repeat runs and the interpreter's instruction
count (one extra run with JAIC_PROFILE=1). Instruction counts are nearly deterministic, so they are the
better signal for small changes; wall time catches native compiler costs the profile cannot see.
Corpus workloads are skipped when corpus/upstream is missing (python3 tools/fetch_upstreams.py).
"""
import argparse
import json
import os
import re
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UPSTREAM = ROOT / "corpus" / "upstream"

# (name, working directory, jaic arguments). Only workloads that succeed and leave the corpus
# untouched: Vk-Engine's Build.jai regenerates bindings into its own tree, so it is not here.
CORPUS = [
    ("focus-check", UPSTREAM / "focus-editor--focus", ["check", "first.jai"]),
    ("jaison-tests", UPSTREAM / "rluba--jaison", ["run", "tests.jai"]),
    ("jails-check", UPSTREAM / "SogoCZE--Jails", ["check", "build.jai"]),
    ("sgpu-examples-check", UPSTREAM / "roeyb1--sgpu" / "examples", ["check", "build.jai"]),
]


def default_jaic():
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    return target / "release" / "jaic"


def workloads():
    for path in sorted((ROOT / "benchmarks").glob("*.jai")):
        yield path.stem, ROOT, ["run", str(path)]
    for name, cwd, args in CORPUS:
        if cwd.is_dir():
            yield name, cwd, args


def run(jaic, cwd, args, profile=False):
    env = dict(os.environ)
    if profile:
        env["JAIC_PROFILE"] = "1"
    else:
        env.pop("JAIC_PROFILE", None)
    start = time.perf_counter()
    proc = subprocess.run([str(jaic), *args], cwd=cwd, env=env, capture_output=True, text=True)
    elapsed = time.perf_counter() - start
    if proc.returncode != 0:
        tail = (proc.stderr or proc.stdout).strip().splitlines()[-3:]
        raise RuntimeError(f"exit {proc.returncode}: " + " | ".join(tail))
    return elapsed, proc.stderr


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--jaic", type=Path, default=default_jaic())
    ap.add_argument("--repeat", type=int, default=3)
    ap.add_argument("--only", default="", help="substring of the workload names to run")
    ap.add_argument("--out", type=Path, help="write the results as JSON")
    ap.add_argument("--compare", type=Path, help="JSON from an earlier --out to diff against")
    a = ap.parse_args()
    jaic = a.jaic.resolve()
    if not jaic.exists():
        sys.exit(f"{jaic} not found; build it with cargo build --release -p jaic-cli or pass --jaic")
    baseline = json.loads(a.compare.read_text()) if a.compare else {}
    results, failed = {}, False
    print(f"{'workload':24} {'median s':>9} {'instructions':>15}  vs baseline")
    for name, cwd, args in workloads():
        if a.only not in name:
            continue
        try:
            times = [run(jaic, cwd, args)[0] for _ in range(a.repeat)]
            _, stderr = run(jaic, cwd, args, profile=True)
        except RuntimeError as error:
            print(f"{name:24} FAILED {error}")
            failed = True
            continue
        match = re.search(r"interpreter profile: (\d+) instructions", stderr)
        row = {"seconds": statistics.median(times), "instructions": int(match[1]) if match else 0}
        results[name] = row
        delta = ""
        if name in baseline:
            old = baseline[name]
            delta = f"{row['seconds'] / old['seconds']:.2f}x time"
            if old["instructions"]:
                delta += f", {row['instructions'] / old['instructions']:.2f}x instructions"
        print(f"{name:24} {row['seconds']:9.3f} {row['instructions']:15,}  {delta}")
    if a.out:
        a.out.write_text(json.dumps(results, indent=2) + "\n")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
