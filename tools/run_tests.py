#!/usr/bin/env python3
"""Run the unittest suites under tools/: one `unittest discover` per folder and pattern, in parallel.

tools/ has no __init__.py, so a plain `discover -s tools` never descends into the subfolders. Every folder, at
any depth, holding a matching file is discovered on its own (unittest puts that folder on sys.path, so a test
imports its siblings by bare name), with tools/ on PYTHONPATH so cross-folder imports are folder-qualified
(`from toolkit.download import fetch`), exactly as they are when a script runs directly.

    python3 tools/run_tests.py                                  # every test_*.py
    python3 tools/run_tests.py -p 'test_sdl*.py' -v             # only the matching files, verbosely
    python3 tools/run_tests.py -p test_a.py -p test_b.py        # several patterns, one run
    python3 tools/run_tests.py ci native_bindings -j 1         # only these folders, one at a time
    python3 tools/run_tests.py -p tools/ci/test_pins.py         # a pattern may be a path: that folder's file

A folder runs in its own process and is killed after `--timeout` seconds (FOLDER_TIMEOUTS raises the limit of
the slow ones). Folders that need prepared inputs have a fixture (FIXTURES below): it runs once, in the worker
that needs it, and only when a selected file in that folder uses it; a fixture that cannot be satisfied skips
the folder with a message instead of failing the run.
"""

from __future__ import annotations

import argparse
import fnmatch
import os
import re
import signal
import subprocess
import sys
import tempfile
import threading
import time
from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path

from toolkit.repo import jaifmt_path

TOOLS = Path(__file__).resolve().parent
NO_TESTS_RAN = 5  # unittest's exit status when a folder has no file matching the pattern
DEFAULT_TIMEOUT = 900  # seconds one folder's tests may take
FOLDER_TIMEOUTS = {"native_bindings": 1800, "stdlib": 1800, "ci": 1800}


class Skip(Exception):
    """A folder's fixture cannot be satisfied here; the message says why."""


def sdl_environment() -> dict[str, str]:
    """The SDL tests need hash-pinned header inputs (tools/bindings/prepare_sdl_python_tests.py); CI prepares
    them first. Do it here when the caller has not, caching the download under target/."""
    if os.environ.get("SDL_TEST_TMPDIR"):
        return {}
    prepare = TOOLS / "bindings" / "prepare_sdl_python_tests.py"
    scratch = str(Path(tempfile.mkdtemp(prefix="sdl-python-")).resolve())  # /var is a symlink on macOS
    cache = TOOLS.parent / "target" / "sdl-python-cache"
    result = subprocess.run(
        [sys.executable, str(prepare), "--scratch", scratch, "--cache", str(cache)],
        text=True,
        capture_output=True,
    )
    if result.returncode:
        sys.stderr.write(
            "run_tests.py: could not prepare the SDL test inputs (no network?); "
            "the tools/bindings SDL tests will fail:\n" + result.stderr[-400:]
        )
        return {}
    return dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)


def formatter() -> dict[str, str]:
    """The generator tests format with the repository jaifmt (tools/generators/jai_codegen.py). Build it when it
    is missing, as `tools/dev/dev.py fmt` does."""
    wanted = jaifmt_path()
    if wanted.is_file():
        return {}
    try:
        from dev.dev import ensure_jaifmt

        ensure_jaifmt(str(os.cpu_count() or 1))
    except (Exception, SystemExit) as error:
        raise Skip(f"{wanted} is missing and could not be built ({error})") from error
    if not wanted.is_file():
        raise Skip(f"{wanted} is missing")
    return {}


@dataclass(frozen=True)
class Fixture:
    """Prepares a folder's tests: `prepare()` returns environment variables for them, or raises Skip. It runs
    when a selected file matching one of `files` is in the folder."""

    files: tuple[str, ...]
    prepare: Callable[[], dict[str, str]]


FIXTURES = {
    "bindings": Fixture(("test_sdl*.py",), sdl_environment),
    "generators": Fixture(("test_*.py",), formatter),
}


def split_path(pattern: str) -> tuple[str | None, str]:
    """A pattern given as a path (`tools/ci/test_a.py`, `ci/test_a.py`) as the folder it names, relative to tools/,
    and the file pattern; a bare file pattern names no folder."""
    if "/" not in pattern:
        return None, pattern
    directory, _, name = pattern.rpartition("/")
    parts = [part for part in directory.split("/") if part not in ("", ".")]
    if parts[:1] == ["tools"]:
        parts = parts[1:]
    return ("/".join(parts) or "tools"), name


def folders(pattern: str = "test_*.py") -> list[Path]:
    """tools/ itself and every subfolder (at any depth) holding a file that matches `pattern`."""
    found = []
    for directory, names, files in os.walk(TOOLS):
        names[:] = sorted(n for n in names if not n.startswith(("_", ".")))
        if any(fnmatch.fnmatch(f, pattern) for f in files):
            found.append(Path(directory))
    return found


def label(folder: Path) -> str:
    if folder == TOOLS:
        return "tools"
    return folder.relative_to(TOOLS).as_posix() if folder.is_relative_to(TOOLS) else folder.name


@dataclass
class Outcome:
    name: str
    tests: int = 0
    seconds: float = 0.0
    status: str = "ok"  # ok | failed | timeout | skipped
    output: str = ""


class Fixtures:
    """Each folder's fixture, run at most once however many tasks need it."""

    def __init__(self, table: dict[str, Fixture]):
        self.table = table
        self.lock = threading.Lock()
        self.results: dict[str, dict[str, str] | Skip] = {}

    def environment(self, folder: Path, patterns: list[str]) -> dict[str, str]:
        fixture = self.table.get(folder.name)
        if fixture is None or folder == TOOLS:
            return {}
        selected = [f.name for f in folder.glob("*.py") if any(fnmatch.fnmatch(f.name, p) for p in patterns)]
        if not any(fnmatch.fnmatch(name, wanted) for name in selected for wanted in fixture.files):
            return {}
        with self.lock:  # one at a time: a fixture may download or build
            if folder.name not in self.results:
                try:
                    self.results[folder.name] = fixture.prepare()
                except Skip as skip:
                    self.results[folder.name] = skip
            result = self.results[folder.name]
        if isinstance(result, Skip):
            raise result
        return result


def run_process(command: list[str], env: dict[str, str], timeout: float) -> tuple[int | None, str, str]:
    """(status, stdout, stderr); status None when the process had to be killed after `timeout` seconds. The
    whole process group goes, so a test's own children do not keep the pipes open."""
    process = subprocess.Popen(
        command,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=os.name == "posix",
    )
    try:
        out, err = process.communicate(timeout=timeout)
        return process.returncode, out, err
    except subprocess.TimeoutExpired:
        if os.name == "posix":
            os.killpg(process.pid, signal.SIGKILL)
        else:
            process.kill()
        out, err = process.communicate()
        return None, out or "", err or ""


def run_task(
    folder: Path,
    pattern: str,
    verbose: bool,
    timeout: float | None,
    fixtures: Fixtures,
    patterns: list[str],
    env: dict[str, str],
) -> Outcome:
    name = label(folder) + (f" ({pattern})" if len(patterns) > 1 else "")
    started = time.monotonic()
    try:
        env = dict(env, **fixtures.environment(folder, patterns))
    except Skip as skip:
        return Outcome(name, status="skipped", output=f"run_tests.py: skipping {label(folder)}: {skip}\n")
    limit = timeout or FOLDER_TIMEOUTS.get(folder.name, DEFAULT_TIMEOUT)
    command = [sys.executable, "-m", "unittest", "discover", "-s", str(folder), "-p", pattern]
    if verbose:
        command.append("-v")
    code, out, err = run_process(command, env, limit)
    outcome = Outcome(name, seconds=time.monotonic() - started, output=out + err)
    ran = re.search(r"^Ran (\d+) tests?", err, re.M)
    outcome.tests = int(ran.group(1)) if ran else 0
    if code is None:
        outcome.status = "timeout"
        outcome.output += f"\nrun_tests.py: {name} killed after {limit:.0f} s\n"
    elif code not in (0, NO_TESTS_RAN):
        outcome.status = "failed"
    return outcome


def run_all(
    patterns: list[str],
    only: list[str],
    jobs: int,
    verbose: bool,
    timeout: float | None,
    table: dict[str, Fixture] | None = None,
    base_env: dict[str, str] | None = None,
) -> list[Outcome]:
    base = dict(os.environ if base_env is None else base_env)
    base["PYTHONPATH"] = os.pathsep.join(filter(None, [str(TOOLS), base.get("PYTHONPATH", "")]))
    fixtures = Fixtures(FIXTURES if table is None else table)
    plain = [split_path(given)[1] for given in patterns]
    tasks = []
    for given in patterns:
        directory, pattern = split_path(given)
        for folder in folders(pattern):
            if directory is not None and label(folder) != directory:
                continue
            if not only or label(folder) in only or folder.name in only:
                tasks.append((folder, pattern))
    # The slowest folders start first so they overlap the rest instead of ending the run alone.
    tasks.sort(key=lambda task: -FOLDER_TIMEOUTS.get(task[0].name, 0))
    outcomes = []
    with ThreadPoolExecutor(max_workers=max(1, jobs)) as pool:
        futures = [pool.submit(run_task, folder, pattern, verbose, timeout, fixtures, plain, base) for folder, pattern in tasks]
        for future in futures:
            outcome = future.result()
            outcomes.append(outcome)
            report(outcome, verbose)
    return outcomes


def report(outcome: Outcome, verbose: bool) -> None:
    """A folder's output goes out whole, when it finishes: always for a failure, otherwise with -v."""
    if outcome.status != "ok" or verbose:
        sys.stdout.write(f"=== {outcome.name}: {outcome.status} ===\n{outcome.output}")
        if outcome.output and not outcome.output.endswith("\n"):
            sys.stdout.write("\n")
    else:
        print(f"ok {outcome.name}: {outcome.tests} tests in {outcome.seconds:.1f} s")
    sys.stdout.flush()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("folders", nargs="*", help="only these folders (as tools/<name>; default all)")
    parser.add_argument("-p", "--pattern", action="append", help="test file pattern (repeatable; default test_*.py)")
    parser.add_argument("-v", "--verbose", action="store_true", help="print every folder's output, not only failures'")
    parser.add_argument("-j", "--jobs", type=int, default=min(4, os.cpu_count() or 1), help="folders to run at once (default %(default)s)")
    parser.add_argument("--timeout", type=float, help="seconds before a folder's process is killed (default %d, longer for the slow folders)" % DEFAULT_TIMEOUT)
    args = parser.parse_args(argv)
    outcomes = run_all(args.pattern or ["test_*.py"], args.folders, args.jobs, args.verbose, args.timeout)
    total = sum(o.tests for o in outcomes)
    failed = [o.name for o in outcomes if o.status in ("failed", "timeout")]
    skipped = [o.name for o in outcomes if o.status == "skipped"]
    print(
        f"tools/run_tests.py: {total} tests; failed folders: {', '.join(failed) or 'none'}"
        + (f"; skipped: {', '.join(skipped)}" if skipped else "")
    )
    return 1 if failed or total == 0 else 0


if __name__ == "__main__":
    sys.exit(main())
