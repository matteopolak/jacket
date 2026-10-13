#!/usr/bin/env python3
"""Run the Io and Http tests many copies at a time, to shake out bugs that depend on timing.

    io_stress.py [--jaic PATH] [--variants default,o2,asan] [--copies N] [--rounds M]
                 [--timeout SECONDS] [-j N] [--keep] [TEST.jai ...]

The scheduler's invariants (a fiber is woken once, and only while it is parked) fail only when
two things land in a particular order, which a quiet machine rarely produces; with many copies
fighting for the cores it happens. Each test is built once per variant with `jaic build FILE -o OUT`
from the test's own directory:

    default   no flags
    o2        -O2
    asan      --sanitize address,undefined (Linux only; skipped elsewhere)

and each round then starts `--copies` copies of every program at once (default three per CPU),
each in its own scratch working directory (the tests write `.scratch/` below it). A copy fails
if it exits non-zero or runs longer than `--timeout`. The first lines of each failure's output are
printed, and the exit status is 1 if anything failed (a build that fails counts).

Without test files it runs stdlib/Extensions/Io/tests/{backends,tcp,timeouts,files,pipe,streams}.jai
and stdlib/Extensions/Http/tests/{client,server_connections,testing}.jai.
"""
import argparse
import os
import shutil
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_TESTS = [
    "stdlib/Extensions/Io/tests/backends.jai",
    "stdlib/Extensions/Io/tests/tcp.jai",
    "stdlib/Extensions/Io/tests/timeouts.jai",
    "stdlib/Extensions/Io/tests/files.jai",
    "stdlib/Extensions/Io/tests/pipe.jai",
    "stdlib/Extensions/Io/tests/streams.jai",
    "stdlib/Extensions/Http/tests/client.jai",
    "stdlib/Extensions/Http/tests/server_connections.jai",
    "stdlib/Extensions/Http/tests/testing.jai",
]
VARIANTS = {
    "default": [],
    "o2": ["-O2"],
    "asan": ["--sanitize", "address,undefined"],
}
# Lines of a failing copy's output that are shown, and failures shown in full.
OUTPUT_LINES = 15
FAILURES_SHOWN = 10
EXE = ".exe" if sys.platform == "win32" else ""


def build(jaic, test, variant, out):
    """Compiles `test` for `variant`. None on success, else the compiler's output."""
    command = [str(jaic), "build", test.name, *VARIANTS[variant], "-o", str(out)]
    try:
        result = subprocess.run(command, cwd=test.parent, capture_output=True, text=True, timeout=1800)
    except subprocess.TimeoutExpired:
        return "the build took longer than 30 minutes"
    if result.returncode != 0:
        return result.stdout + result.stderr
    return None


def run_copy(program, directory, timeout):
    """Runs one copy in `directory`. None if it passed, else a description of the failure."""
    directory.mkdir(parents=True)
    try:
        result = subprocess.run([str(program)], cwd=directory, capture_output=True, text=True,
                                errors="replace", timeout=timeout)
    except subprocess.TimeoutExpired as error:
        output = error.stdout or b""
        if isinstance(output, bytes):
            output = output.decode(errors="replace")
        return f"timed out after {timeout:g} s\n{output}"
    if result.returncode == 0:
        return None
    status = f"signal {-result.returncode}" if result.returncode < 0 else f"exit status {result.returncode}"
    return f"{status}\n{result.stdout}{result.stderr}"


def short(text):
    lines = text.rstrip().splitlines()
    shown = "\n".join("    " + line for line in lines[:OUTPUT_LINES])
    if len(lines) > OUTPUT_LINES:
        shown += f"\n    ... ({len(lines) - OUTPUT_LINES} more lines)"
    return shown


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("tests", nargs="*", help="test programs (default: the Io and Http tests)")
    parser.add_argument("--jaic", default=str(ROOT / "target/debug/jaic"), help="the compiler")
    parser.add_argument("--variants", default="default,o2,asan", help="comma-separated: default, o2, asan")
    parser.add_argument("--copies", type=int, default=3 * (os.cpu_count() or 1),
                        help="copies of a program running at once (default: 3 per CPU)")
    parser.add_argument("--rounds", type=int, default=5, help="rounds per program and variant")
    parser.add_argument("--timeout", type=float, default=180, help="seconds a copy may run")
    parser.add_argument("-j", "--jobs", type=int, default=os.cpu_count() or 1, help="parallel builds")
    parser.add_argument("--keep", action="store_true", help="keep the build and scratch folders")
    args = parser.parse_args()

    jaic = Path(args.jaic).resolve()
    tests = [Path(t).resolve() for t in (args.tests or [str(ROOT / t) for t in DEFAULT_TESTS])]
    variants = [v for v in args.variants.split(",") if v]
    for variant in variants:
        if variant not in VARIANTS:
            parser.error(f"unknown variant {variant!r}; choose from {', '.join(VARIANTS)}")
    for test in tests:
        if not test.is_file():
            parser.error(f"no such test: {test}")
    if "asan" in variants and not sys.platform.startswith("linux"):
        print("skipping the asan variant: AddressSanitizer is Linux-only here")
        variants.remove("asan")

    work = Path(tempfile.mkdtemp(prefix="io-stress-"))
    failures = []
    runs = 0

    try:
        # Build every program once per variant.
        jobs = [(test, variant) for variant in variants for test in tests]
        programs = {}
        started = time.time()

        def build_job(job):
            test, variant = job
            out = work / "bin" / variant / (test.stem + EXE)
            out.parent.mkdir(parents=True, exist_ok=True)
            return job, out, build(jaic, test, variant, out)

        with ThreadPoolExecutor(max_workers=max(args.jobs, 1)) as pool:
            for (test, variant), out, problem in pool.map(build_job, jobs):
                if problem is None:
                    programs[(test, variant)] = out
                else:
                    failures.append((f"{test.stem} [{variant}] build", problem))
        print(f"built {len(programs)} of {len(jobs)} programs in {time.time() - started:.0f} s")

        for number in range(1, args.rounds + 1):
            for variant in variants:
                for test in tests:
                    program = programs.get((test, variant))
                    if program is None:
                        continue
                    started = time.time()
                    base = work / "run" / f"{variant}-{test.stem}-{number}"
                    with ThreadPoolExecutor(max_workers=args.copies) as pool:
                        results = list(pool.map(
                            lambda copy: run_copy(program, base / str(copy), args.timeout),
                            range(args.copies)))
                    bad = [(copy, r) for copy, r in enumerate(results) if r is not None]
                    runs += len(results)
                    for copy, problem in bad:
                        failures.append((f"{test.stem} [{variant}] round {number} copy {copy}", problem))
                    shutil.rmtree(base, ignore_errors=True)
                    print(f"round {number} {variant:7} {test.stem:20} {args.copies} copies, "
                          f"{len(bad)} failed, {time.time() - started:.1f} s", flush=True)
    finally:
        if args.keep:
            print(f"kept {work}")
        else:
            shutil.rmtree(work, ignore_errors=True)

    print(f"\n{runs} runs, {len(failures)} failed")
    for name, problem in failures[:FAILURES_SHOWN]:
        print(f"\nFAIL {name}\n{short(problem)}")
    if len(failures) > FAILURES_SHOWN:
        print(f"\n... and {len(failures) - FAILURES_SHOWN} more failures")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
