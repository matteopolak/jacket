#!/usr/bin/env python3
"""Fail when the compiler allocates or executes noticeably more than the committed baseline says it should.

Usage: tools/bench/gate.py [--jaic PATH] [--baseline FILE] [--update] [--threshold PCT] [--no-instructions]

Wall-clock time is too noisy on shared runners to gate on (docs/tools/benchmarks.md). These counts are not:
for a fixed workload, flags and binary, the allocation counts `jaic -timings` reports (`jaic-alloc:` lines) and
the instructions valgrind's cachegrind executes are the same on every run. Each workload (a generated program
and a hello world, `-check` and a native build) is run single-threaded; the metrics are compared with
`tools/bench/gate-baseline.json`, kept per platform (`linux-x86_64`), and a metric that grew by more than its
threshold fails. A platform without a baseline, or without valgrind for the instruction counts, skips what it
cannot measure. Explained in docs/tools/perf-gate.md, including how to update the baseline.
"""

import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import benchgen
from toolkit.cli import add_jaic_argument
from toolkit.repo import ROOT, stdlib_env

BASELINE = ROOT / "tools/bench/gate-baseline.json"
# Single-threaded and without the parse-ahead crawl: the same work in the same order every run.
FLAGS = ["-serial_parse", "-threads", "1", "-timings", "-no_color"]
# Allowed growth, in percent, per metric. Allocation counts do not move with the machine, only with the code;
# instructions also follow the libc and valgrind versions of the runner image.
THRESHOLDS = {"alloc_blocks": 2.0, "alloc_bytes": 3.0, "instructions": 4.0}
# Shrinking by more than this asks for the baseline to be updated, so the gate keeps its bite.
IMPROVED = 5.0

HELLO = '#import "Basic";\nmain :: () { print("hello\\n"); }\n'
# A build metaprogram that reads every TYPECHECKED message: counts what it holds and walks the nodes, so the
# export of the code trees shows up in the counts (a metaprogram that never names the message skips it).
READER = '''#import "Basic";
#import "Compiler";

build :: () {
    set_build_options_dc(.{ do_output = false });
    w := compiler_create_workspace("gate");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_file("%/main.jai", w);

    declarations, nodes: s64;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .TYPECHECKED {
            typechecked := cast(*Message_Typechecked) message;
            declarations += typechecked.declarations.count;
            for typechecked.all nodes += it.subexpressions.count;
        }
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
    print("typechecked: % declarations, % nodes\\n", declarations, nodes);
}

#run build();
'''
ALLOC = re.compile(r"^jaic-alloc: total (\d+) (\d+)$", re.M)
IREFS = re.compile(r"I\s+refs:\s+([\d,]+)")


def platform_key() -> str:
    return f"{platform.system().lower()}-{platform.machine().lower()}"


def workloads(scratch: Path):
    """(name, directory of the program, extra jaic arguments)."""
    hello = scratch / "hello"
    hello.mkdir()
    (hello / "main.jai").write_text(HELLO, encoding='utf-8')
    gens = {}
    for lines in (2000, 10000):
        directory = scratch / f"gen-{lines // 1000}k"
        directory.mkdir()
        for name, text in benchgen.generate(lines, 1, 1).items():
            (directory / name).write_text(text, encoding='utf-8')
        gens[lines] = directory
    meta = scratch / "gen-2k-meta"
    (meta / "program").mkdir(parents=True)
    for name, text in benchgen.generate(2000, 1, 1).items():
        (meta / "program" / name).write_text(text, encoding='utf-8')
    (meta / "main.jai").write_text(READER.replace("%/main.jai", str(meta / "program") + "/main.jai"), encoding='utf-8')
    return [
        ("hello/check", hello, ["-check"]),
        ("hello/build", hello, ["-exe", str(scratch / "hello-out")]),
        ("gen-2k/check", gens[2000], ["-check"]),
        ("gen-2k/build", gens[2000], ["-exe", str(scratch / "gen2-out")]),
        ("gen-10k/check", gens[10000], ["-check"]),
        ("gen-2k/meta", meta, []),
    ]


def parse_allocations(stderr: str):
    match = ALLOC.search(stderr)
    return (int(match.group(1)), int(match.group(2))) if match else None


def parse_instructions(stderr: str):
    match = IREFS.search(stderr)
    return int(match.group(1).replace(",", "")) if match else None


def measure(jaic: Path, directory: Path, args, instructions: bool):
    command = [str(jaic), "main.jai", *args, *FLAGS]
    run = subprocess.run(command, cwd=directory, capture_output=True, text=True, env=stdlib_env(), timeout=600)
    if run.returncode:
        raise SystemExit(f"{' '.join(command)} failed ({run.returncode}):\n{run.stderr[-2000:]}")
    counts = parse_allocations(run.stderr)
    if counts is None:
        raise SystemExit("jaic printed no `jaic-alloc: total` line; is -timings built in?")
    result = {"alloc_blocks": counts[0], "alloc_bytes": counts[1]}
    if instructions:
        valgrind = subprocess.run(
            ["valgrind", "--tool=cachegrind", "--cache-sim=no", "--cachegrind-out-file=/dev/null", *command],
            cwd=directory, capture_output=True, text=True, env=stdlib_env(), timeout=3600,
        )
        found = parse_instructions(valgrind.stderr)
        if valgrind.returncode or found is None:
            raise SystemExit(f"valgrind failed ({valgrind.returncode}):\n{valgrind.stderr[-2000:]}")
        result["instructions"] = found
    return result


def compare(baseline: dict, current: dict, threshold_scale: float = 1.0):
    """(regressions, improvements, rows): each a list of text; rows are (workload, metric, base, now, change %)."""
    regressions, improvements, rows = [], [], []
    for name, metrics in sorted(current.items()):
        for metric, now in sorted(metrics.items()):
            base = baseline.get(name, {}).get(metric)
            if base is None:
                regressions.append(f"{name} {metric}: no baseline entry; run with --update")
                continue
            change = 100 * (now - base) / base if base else 0.0
            rows.append((name, metric, base, now, change))
            limit = THRESHOLDS[metric] * threshold_scale
            if change > limit:
                regressions.append(f"{name} {metric}: {base:,} -> {now:,} ({change:+.1f}%, limit +{limit:.1f}%)")
            elif change < -IMPROVED:
                improvements.append(f"{name} {metric}: {base:,} -> {now:,} ({change:+.1f}%)")
    for name in sorted(set(baseline) - set(current)):
        regressions.append(f"{name}: in the baseline but not measured; run with --update")
    return regressions, improvements, rows


def table(rows) -> str:
    out = ["| workload | metric | baseline | now | change |", "|---|---|---:|---:|---:|"]
    out += [f"| {n} | {m} | {b:,} | {c:,} | {d:+.1f}% |" for n, m, b, c, d in rows]
    return "\n".join(out)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    add_jaic_argument(parser, profile="release")
    parser.add_argument("--baseline", type=Path, default=BASELINE)
    parser.add_argument("--update", action="store_true", help="write this platform's measurements into the baseline")
    parser.add_argument("--threshold", type=float, default=1.0, help="multiply every threshold by this")
    parser.add_argument("--no-instructions", action="store_true", help="skip the valgrind runs")
    args = parser.parse_args()
    instructions = not args.no_instructions and shutil.which("valgrind") is not None
    if not instructions:
        print("instruction counts skipped (--no-instructions or no valgrind)", file=sys.stderr)
    key = platform_key()
    with tempfile.TemporaryDirectory(prefix="perf-gate-") as scratch:
        current = {name: measure(args.jaic.resolve(), directory, extra, instructions) for name, directory, extra in workloads(Path(scratch))}
    data = json.loads(args.baseline.read_text(encoding='utf-8')) if args.baseline.exists() else {"format": 1, "platforms": {}}
    if args.update:
        data["platforms"][key] = current
        args.baseline.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding='utf-8')
        print(f"wrote {key} to {args.baseline}")
        print(json.dumps({key: current}, indent=2, sort_keys=True))
        return 0
    baseline = data["platforms"].get(key)
    if baseline is None:
        print(f"no baseline for {key} in {args.baseline}; measured only:\n{json.dumps(current, indent=2, sort_keys=True)}")
        return 0
    if not instructions:
        baseline = {name: {m: v for m, v in metrics.items() if m != "instructions"} for name, metrics in baseline.items()}
    regressions, improvements, rows = compare(baseline, current, args.threshold)
    report = table(rows)
    print(report)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as summary:
            summary.write(f"## Performance gate ({key})\n\n{report}\n")
    for line in improvements:
        print(f"improved, consider updating the baseline: {line}", file=sys.stderr)
    for line in regressions:
        print(f"REGRESSION {line}", file=sys.stderr)
    if regressions:
        print("If the growth is intended, update the baseline (docs/tools/perf-gate.md).", file=sys.stderr)
    return 1 if regressions else 0


if __name__ == "__main__":
    sys.exit(main())
