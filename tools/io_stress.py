#!/usr/bin/env python3
"""Run the Io and Http tests many copies at a time, to shake out bugs that depend on timing.

    io_stress.py [--jaic PATH] [--variants default,o2,asan] [--copies N] [--rounds M]
                 [--timeout SECONDS] [-j N] [--keep] [TEST.jai ...]

The scheduler's invariants (a fiber is woken once, and only while it is parked) fail only when
two things land in a particular order, which a quiet machine rarely produces; with many copies
fighting for the cores it happens. Each test is built once per variant with `jaic FILE -exe OUT`
from the test's own directory:

    default   no flags
    o2        -O2
    asan      -sanitize address,undefined (Linux only; skipped elsewhere)

and each round then starts `--copies` copies of every program at once (default three per CPU),
each in its own scratch working directory (the tests write `.scratch/` below it). A copy fails
if it exits non-zero or runs longer than `--timeout`. The first lines of each failure's output are
printed, and the exit status is 1 if anything failed (a build that fails counts).

Without test files it runs stdlib/Extensions/Io/tests/{backends,tcp,timeouts,files,pipe,streams}.jai,
stdlib/Extensions/Http/tests/{client,server_connections,testing,https}.jai and
stdlib/Extensions/Tls/tests/tls.jai (it carries its certificates, so it runs from the scratch
directories). interop.jai starts openssl processes on ports and is not part of it.
"""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from threading import Thread
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from http_native import ensure_native_libraries
from check_io_burst_diagnostics import observe_process
from io_failure_observer import FailureGroup, source_identity

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
    "stdlib/Extensions/Http/tests/https.jai",
    "stdlib/Extensions/Tls/tests/tls.jai",
]
VARIANTS = {
    "default": [],
    "o2": ["-O2"],
    "asan": ["-sanitize", "address,undefined"],
}
# Lines of a failing copy's output that are shown, and failures shown in full.
OUTPUT_LINES = 15
FAILURES_SHOWN = 10
EXE = ".exe" if sys.platform == "win32" else ""


def build(jaic, test, variant, out):
    """Compiles `test` for `variant`. None on success, else the compiler's output."""
    command = [str(jaic), test.name, *VARIANTS[variant], "-exe", str(out)]
    try:
        result = subprocess.run(
            command, cwd=test.parent, capture_output=True, text=True, timeout=1800
        )
    except subprocess.TimeoutExpired:
        return "the build took longer than 30 minutes"
    if result.returncode != 0:
        return result.stdout + result.stderr
    return None


def observed_run(program, directory, timeout, report):
    """Observe the original child; the original supervisor owns its timeout."""
    report.mkdir(parents=True, exist_ok=False)
    receipt = {
        "program": str(program),
        "program_sha256": hashlib.sha256(program.read_bytes()).hexdigest(),
        "cwd": str(directory),
        "supervisor_timeout_seconds": timeout,
        "samples": [],
    }

    def save():
        (report / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")

    with subprocess.Popen(
        [str(program)],
        cwd=directory,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        errors="replace",
    ) as process:
        receipt["pid"] = process.pid

        def observe():
            try:
                observe_process(process, receipt, save)
            except Exception as error:
                receipt["observer_error"] = repr(error)

        observer = Thread(target=observe)
        observer.start()
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            # This is the same timeout/kill policy used by subprocess.run for
            # every other copy. The observer never owns termination or retries.
            receipt["supervisor_timeout"] = True
            process.kill()
            stdout, stderr = process.communicate()
            (report / "program.stdout").write_text(stdout)
            (report / "program.stderr").write_text(stderr)
            raise
        finally:
            observer.join()
            receipt["exit"] = process.returncode
            save()
        (report / "program.stdout").write_text(stdout)
        (report / "program.stderr").write_text(stderr)
        if "observer_error" in receipt:
            raise RuntimeError(
                "required observation failed: " + receipt["observer_error"]
            )
        return subprocess.CompletedProcess(
            [str(program)], process.returncode, stdout, stderr
        )


def run_copy(
    program, directory, timeout, observation=None, failure_group=None, copy_index=None
):
    """Runs one copy in `directory`. None if it passed, else a description of the failure."""
    directory.mkdir(parents=True)
    try:
        if failure_group is not None:
            result = failure_group.run(copy_index, program, directory, timeout)
        elif observation is not None:
            result = observed_run(program, directory, timeout, observation)
        else:
            result = subprocess.run(
                [str(program)],
                cwd=directory,
                capture_output=True,
                text=True,
                errors="replace",
                timeout=timeout,
            )
    except subprocess.TimeoutExpired as error:
        output = error.stdout or b""
        if isinstance(output, bytes):
            output = output.decode(errors="replace")
        stderr = error.stderr or b""
        if isinstance(stderr, bytes):
            stderr = stderr.decode(errors="replace")
        return f"timed out after {timeout:g} s\n{output}{stderr}"
    except RuntimeError as error:
        return f"observation failed: {error}"
    if result.returncode == 0:
        return None
    status = (
        f"signal {-result.returncode}"
        if result.returncode < 0
        else f"exit status {result.returncode}"
    )
    return f"{status}\n{result.stdout}{result.stderr}"


def short(text):
    lines = text.rstrip().splitlines()
    shown = "\n".join("    " + line for line in lines[:OUTPUT_LINES])
    if len(lines) > OUTPUT_LINES:
        shown += f"\n    ... ({len(lines) - OUTPUT_LINES} more lines)"
    return shown


def report_burst_diagnostics(name, problem):
    """Preserve failure-stage stderr from the actual loaded copy, without a rerun."""
    for line in problem.splitlines():
        if line.startswith("Io burst "):
            print(f"{name}: {line}", file=sys.stderr, flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "tests", nargs="*", help="test programs (default: the Io and Http tests)"
    )
    parser.add_argument(
        "--jaic", default=str(ROOT / "target/debug/jaic"), help="the compiler"
    )
    parser.add_argument(
        "--variants",
        default="default,o2,asan",
        help="comma-separated: default, o2, asan",
    )
    parser.add_argument(
        "--copies",
        type=int,
        default=3 * (os.cpu_count() or 1),
        help="copies of a program running at once (default: 3 per CPU)",
    )
    parser.add_argument(
        "--rounds", type=int, default=5, help="rounds per program and variant"
    )
    parser.add_argument(
        "--timeout", type=float, default=180, help="seconds a copy may run"
    )
    parser.add_argument(
        "-j", "--jobs", type=int, default=os.cpu_count() or 1, help="parallel builds"
    )
    parser.add_argument(
        "--keep", action="store_true", help="keep the build and scratch folders"
    )
    parser.add_argument(
        "--observe-copy", help="Linux only: one existing test:variant:round:copy"
    )
    parser.add_argument(
        "--observe-report", type=Path, help="fresh private metadata output directory"
    )
    parser.add_argument(
        "--observe-failures-report",
        type=Path,
        help="Linux only: bounded numeric evidence for actual failed backends/tcp copies",
    )
    args = parser.parse_args()

    jaic = Path(args.jaic).resolve()
    tests = [
        Path(t).resolve()
        for t in (args.tests or [str(ROOT / t) for t in DEFAULT_TESTS])
    ]
    variants = [v for v in args.variants.split(",") if v]
    for variant in variants:
        if variant not in VARIANTS:
            parser.error(
                f"unknown variant {variant!r}; choose from {', '.join(VARIANTS)}"
            )
    for test in tests:
        if not test.is_file():
            parser.error(f"no such test: {test}")
    observed = None
    if args.observe_failures_report:
        if (
            not sys.platform.startswith("linux")
            or args.observe_copy
            or args.observe_report
        ):
            parser.error(
                "failure observation requires Linux and cannot combine with --observe-copy/report"
            )
        if args.observe_failures_report.exists():
            parser.error("--observe-failures-report must be a fresh directory")
    if args.observe_copy or args.observe_report:
        if (
            not sys.platform.startswith("linux")
            or not args.observe_copy
            or not args.observe_report
        ):
            parser.error(
                "observation requires Linux, --observe-copy and --observe-report"
            )
        try:
            stem, variant, number, copy = args.observe_copy.split(":")
            observed = (stem, variant, int(number), int(copy))
        except ValueError:
            parser.error("--observe-copy must be test:variant:round:copy")
        if (
            sum(t.stem == stem for t in tests) != 1
            or variant not in variants
            or not 1 <= observed[2] <= args.rounds
            or not 0 <= observed[3] < args.copies
        ):
            parser.error(
                "--observe-copy must select exactly one existing workload copy"
            )
        if args.observe_report.exists():
            parser.error("--observe-report must be a fresh directory")
    if "asan" in variants and not sys.platform.startswith("linux"):
        print("skipping the asan variant: AddressSanitizer is Linux-only here")
        variants.remove("asan")

    ensure_native_libraries()
    failure_source = (
        source_identity(ROOT, jaic) if args.observe_failures_report else None
    )
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
        print(
            f"built {len(programs)} of {len(jobs)} programs in {time.time() - started:.0f} s"
        )

        for number in range(1, args.rounds + 1):
            for variant in variants:
                for test in tests:
                    program = programs.get((test, variant))
                    if program is None:
                        continue
                    started = time.time()
                    base = work / "run" / f"{variant}-{test.stem}-{number}"
                    group = None
                    if failure_source is not None and test.stem in {"backends", "tcp"}:
                        group = FailureGroup(
                            args.observe_failures_report,
                            {
                                **failure_source,
                                "test": test.stem,
                                "variant": variant,
                                "round": number,
                                "copies": args.copies,
                                "test_source_sha256": hashlib.sha256(
                                    test.read_bytes()
                                ).hexdigest(),
                                "program_sha256": hashlib.sha256(
                                    program.read_bytes()
                                ).hexdigest(),
                            },
                        )
                    try:
                        with ThreadPoolExecutor(max_workers=args.copies) as pool:
                            results = list(
                                pool.map(
                                    lambda copy: run_copy(
                                        program,
                                        base / str(copy),
                                        args.timeout,
                                        args.observe_report
                                        if observed
                                        == (test.stem, variant, number, copy)
                                        else None,
                                        group,
                                        copy,
                                    ),
                                    range(args.copies),
                                )
                            )
                    finally:
                        if group is not None:
                            group.finish(
                                sorted(
                                    i
                                    for i, row in group.records.items()
                                    if row.get("exit_phase") != "running"
                                    and row.get("exit") != 0
                                )
                            )
                    bad = [(copy, r) for copy, r in enumerate(results) if r is not None]
                    runs += len(results)
                    for copy, problem in bad:
                        name = f"{test.stem} [{variant}] round {number} copy {copy}"
                        failures.append((name, problem))
                        report_burst_diagnostics(name, problem)
                    shutil.rmtree(base, ignore_errors=True)
                    print(
                        f"round {number} {variant:7} {test.stem:20} {args.copies} copies, "
                        f"{len(bad)} failed, {time.time() - started:.1f} s",
                        flush=True,
                    )
    finally:
        if failure_source is not None:
            after = source_identity(ROOT, jaic)
            directory = args.observe_failures_report
            directory.mkdir(parents=True, exist_ok=True)
            path = directory / "receipt.json"
            data = (
                json.loads(path.read_text())
                if path.exists()
                else {"version": 1, "groups": []}
            )
            data["source_guard"] = {
                "before": failure_source,
                "after": after,
                "matches": failure_source == after,
            }
            path.write_text(json.dumps(data, indent=2) + "\n")
            if failure_source != after:
                failures.append(
                    (
                        "observer source guard",
                        "compiler/library source inputs changed during original workload",
                    )
                )
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
