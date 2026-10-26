#!/usr/bin/env python3
"""Run jaic over test programs and summarize results.

Usage: tools/differential/jaic_sweep.py [--jaic PATH] [--filter TEXT] [--verbose] SET...
Sets: corpus (tests/corpus/positive with expected runtime), negative (tests/corpus/negative:
`jaic -check` must fail and report the recorded text), regressions (tests/compiler/regressions), modules (the stdlib's own
tests: every stdlib/**/tests directory, described by its manifest.json rows; run must succeed), examples (tests/corpus/examples.json:
example workspaces (examples/<name>), whose stdout must contain the listed lines), upstream
(tools/upstream-cases.json: upstream project entry points that must pass, with the output the
project documents where a case has an `expect` record), or file paths.

--native builds each `run` case with `jaic` and runs the executable instead, with the
same expectations; --sanitize address,undefined (implies --native) instruments those builds and
fails a case on any sanitizer report (docs/native/sanitizers.md).
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from toolkit.host import brew_prefix
from toolkit.repo import ROOT
from toolkit.cli import add_jaic_argument
from corpus import select_upstream_cases
from corpus import upstream_sandbox as usb
import threading

UPSTREAM = ROOT / "corpus/upstream"
# Upstream cases with `copy` run in a scratch copy of those project directories (`link` names
# read-only siblings to symlink next to them), so a build that writes generated files never
# changes the pinned corpus. Case id -> scratch root, filled before the cases start.
WORKDIRS = {}
CASE_METADATA = {}
# Upstream cases that name the platforms they can run on (`only`, with `only_reason`), by id.
UPSTREAM_PLATFORMS = {}
# The upstream case list; `JAIC_UPSTREAM_CASES` points a run at another copy (for trying entry points).
UPSTREAM_CASES = Path(os.environ.get("JAIC_UPSTREAM_CASES") or ROOT / "tools/upstream-cases.json")
# Upstream cases with `"review": "pending"`: unreviewed code that only runs inside a sandbox.
UPSTREAM_PENDING = set()

# Upstream cases are confined (docs/tools/upstream-corpus.md, "Sandbox"): UPSTREAM_IDS are their
# ids, SANDBOX the run's settings (None when unconfined), and `_case.box` the (sandbox, scratch)
# of the case the current thread is running, which run_limited wraps every command in.
UPSTREAM_IDS = set()
SANDBOX = None
CASE_SCRATCH = []
_case = threading.local()

# Cases switched off with a `disabled` reason, by id: upstream cases (tools/upstream-cases.json) and negative corpus rows.
DISABLED_REASONS = {}


def upstream_root(c):
    return WORKDIRS.get(c["id"], UPSTREAM)


def make_workdir(c):
    root = Path(tempfile.mkdtemp(prefix=f"jaic_sweep-{c['id']}-"))
    copy, link = c.get("copy", []), c.get("link", [])
    if "copy" not in c and (c["mode"] == "build" or c.get("run_after") or c.get("setup")):
        # A confined build writes only inside a scratch copy of its project; its siblings are links.
        copy = [c["path"].split("/")[0]]
        link = [p.name for p in UPSTREAM.iterdir() if p.name != copy[0]]
    for name in copy:
        shutil.copytree(UPSTREAM / name, root / name, symlinks=True)
    for name in link:
        (root / name).symlink_to(UPSTREAM / name, target_is_directory=True)
    # Files of this repository placed into the copy (a driver for a project whose own build
    # script cannot run here); destination relative to the scratch root -> repository path.
    for dest, source in c.get("files", {}).items():
        shutil.copyfile(ROOT / source, root / dest)
    return root


def case_box(cid):
    """(sandbox, scratch) confining the commands of upstream case `cid`, or None."""
    if not SANDBOX or cid not in UPSTREAM_IDS:
        return None
    scratch = usb.scratch_dir(cid)
    CASE_SCRATCH.append(scratch)
    root = WORKDIRS.get(cid)
    writes = [scratch, *([root] if root else [])]
    return usb.make_sandbox([*SANDBOX["reads"], *writes], writes), scratch


def upstream_expect(c):
    """An upstream case's `expect` record (output the project itself documents), in the shape
    run() compares: exit code, exact stdout, or lines that must appear (in order or anywhere)."""
    e = c.get("expect")
    if not e and not c.get("run_after"):
        return None
    e = e or {}
    return {
        "upstream": True,
        "exit_code": e.get("exit_code", 0),
        "stdout": e.get("stdout"),
        "ordered": e.get("stdout_ordered", []),
        "contains": e.get("stdout_contains", []),
        "excludes": [],
        "run_after": c.get("run_after"),
    }


def missing_in_order(lines, out):
    """The first of `lines` not found after the previous one's match in `out`, or None."""
    at = 0
    for line in lines:
        found = out.find(line, at)
        if found < 0:
            return line
        at = found + len(line)
    return None


def cases(name):
    if name == "corpus":
        m = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
        for c in m["cases"]:
            if "runtime" in c:
                yield (
                    c["id"],
                    ROOT / "tests/corpus" / c["source"],
                    "run",
                    c["runtime"],
                    [],
                )
    elif name == "negative":
        m = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
        for c in m["cases"]:
            if c.get("kind") == "negative":
                if "disabled" in c:
                    DISABLED_REASONS[c["id"]] = c["disabled"]
                yield (
                    c["id"],
                    ROOT / "tests/corpus" / c["source"],
                    "check",
                    {"negative": next(iter(c["negative"].values()))},
                    [],
                )
    elif name in ("regressions", "modules"):
        from stdlib.stdlib_test_catalog import validated_entries

        wanted = "compiler" if name == "regressions" else "modules"
        for c in validated_entries():
            if c["family"] != wanted:
                continue
            CASE_METADATA[c["id"]] = c
            expect = (
                c
                if (
                    "stdout" in c
                    or "stdout_one_of" in c
                    or "error_contains" in c
                    or c.get("mode") == "check-fail"
                )
                else None
            )
            yield (
                c["id"],
                c["path"],
                "check" if c.get("mode") == "check-fail" else "run",
                expect,
                [],
            )
    elif name == "examples":
        cases = json.loads((ROOT / "tests/corpus/examples.json").read_text())["cases"]
        folders = {d.name for d in (ROOT / "examples").iterdir() if d.is_dir()}
        missing = folders - {Path(c["directory"]).name for c in cases}
        if missing:
            sys.exit(f"jaic_sweep: examples/{sorted(missing)[0]} has no case in tests/corpus/examples.json")
        for c in cases:
            mode = c.get("mode", "run")
            expect = {
                "contains": c["stdout_contains"],
                "excludes": c.get("stdout_excludes", []),
                "exit_code": c.get("exit_code", 0),
            }
            yield (
                c["id"],
                ROOT / c["directory"] / c["main"],
                mode,
                expect if mode == "run" else None,
                c.get("args", []),
            )
    elif name == "upstream":
        for c in json.loads(UPSTREAM_CASES.read_text()):
            UPSTREAM_IDS.add(c["id"])
            if "only" in c:
                UPSTREAM_PLATFORMS[c["id"]] = c
            if select_upstream_cases.pending(c):
                UPSTREAM_PENDING.add(c["id"])
            if "disabled" in c:
                DISABLED_REASONS[c["id"]] = c["disabled"]
            yield (
                c["id"],
                upstream_root(c) / c["path"],
                c["mode"],
                upstream_expect(c),
                c.get("args", []),
            )
    else:
        p = Path(name)
        yield p.stem, p, "run", None, []


def use_native_libs():
    """Point jaic at the third-party C libraries (rpmalloc, stb_*) that some cases call or link,
    building any that are missing. Worktrees share the main checkout's target/."""
    if os.environ.get("JAIC_NATIVE_LIBS") or sys.platform not in ("darwin", "linux"):
        return
    from native import build_native_libs

    if build_native_libs.missing():
        subprocess.run(
            [sys.executable, str(ROOT / "tools/native/build_native_libs.py")], check=True
        )
    # The Debug module's native helper is built into the stdlib tree of this checkout.
    suffix = "dylib" if sys.platform == "darwin" else "so"
    helper = ROOT / f"stdlib/internal/developer-tools/debug-native/libdeveloper_tools_debug.{suffix}"
    if not helper.exists():
        subprocess.run([sys.executable, str(ROOT / "tools/developer_checks/build_developer_debug.py")], check=True)
    libs = [str(build_native_libs.output_dir())]
    # The source-built SDK providers (tools/native/build_ci_native_bindings.py): the stdlib's rpmalloc
    # allocator and the other bound SDKs load them when they exist.
    from repo_output import shared_output_dir

    providers = shared_output_dir("production-bindings", "native")
    if providers.is_dir():
        libs.insert(0, str(providers))
    os.environ["JAIC_NATIVE_LIBS"] = os.pathsep.join(libs)


def physical_memory_gib():
    try:
        return os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES") / 2**30
    except (ValueError, OSError, AttributeError):
        return 16


def stale_sources(jaic):
    """A compiler source newer than the jaic binary: a stale build can lack limits that keep
    negative cases (unbounded recursion and the like) from exhausting memory."""
    try:
        built = jaic.stat().st_mtime
    except OSError:
        return None
    for crate in ("jaic", "jaic-cli", "jaic-llvm"):
        for f in (ROOT / "crates" / crate / "src").rglob("*.rs"):
            if f.stat().st_mtime > built:
                return f.relative_to(ROOT)
    return None


# jaic's exit status when JAIC_MEMORY_LIMIT stops it (`jaic::memory_limit::EXIT_CODE`).
MEMORY_LIMIT_EXIT = 120
# `jaic`'s exit status for a program without `main` (NO_MAIN_STATUS in jaic-cli).
NO_MAIN_EXIT = 3


def run_limited(command, cwd, timeout, limit_bytes, jaic=True, environment=None):
    """Runs `command` with jaic's exact allocation limit armed, killing it on timeout. Returns
    (stdout, stderr, code), with code -1 and a one-line verdict as stderr when a limit stopped it.
    `jaic` says whether the command is jaic, whose exit status MEMORY_LIMIT_EXIT means the limit
    (another program may exit with any status)."""
    env = dict(os.environ, JAIC_MEMORY_LIMIT=str(limit_bytes))
    if environment:
        env.update(environment)
    box = getattr(_case, "box", None)
    if box:
        command, env = usb.confine(box[0], box[1], command, env)
    # Output goes to files rather than pipes: a program the case launched may outlive a killed
    # jaic and would keep a pipe open.
    with tempfile.TemporaryFile() as out_file, tempfile.TemporaryFile() as err_file:
        proc = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=out_file,
            stderr=err_file,
        )
        try:
            code = proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            return "", "timeout", -1
        out_file.seek(0)
        err_file.seek(0)
        out = out_file.read().decode(errors="replace")
        err = err_file.read().decode(errors="replace")
    if jaic and code == MEMORY_LIMIT_EXIT:
        return (
            "",
            next(
                (line for line in err.splitlines() if line.strip()),
                "memory limit exceeded",
            ),
            -1,
        )
    return out, err, code


def run_fixture(command, cwd, timeout, limit_bytes, metadata, jaic=True, extra_env=None):
    from stdlib.stdlib_runtime import runtime_prefix, CLIPBOARD_LOCK

    try:
        prefix = runtime_prefix(metadata or {}, os.environ)
    except ValueError as error:
        return "", "error: fixture setup: " + str(error), 1
    if not prefix:
        return run_limited(
            command, cwd, timeout, limit_bytes, jaic=jaic, environment=extra_env
        )
    environment = {
        "JAIC_TEST_CHILD_TIMEOUT_MS": str(max(1, round(timeout * 1000))),
        **(extra_env or {}),
    }
    with CLIPBOARD_LOCK:
        return run_limited(
            [*prefix, *command],
            cwd,
            timeout + 30,
            limit_bytes,
            jaic=jaic,
            environment=environment,
        )


# Runtime options for sanitized executables. Leaks are not reported: programs routinely leave
# their memory to the OS at exit, and a leak is not the codegen or memory-safety bug the
# sanitizer run looks for. Every other report stops the program with a failing status.
SANITIZER_ENV = {
    "ASAN_OPTIONS": "detect_leaks=0:halt_on_error=1:abort_on_error=0:detect_stack_use_after_return=1:strict_string_checks=1:check_initialization_order=1",
    "UBSAN_OPTIONS": "print_stacktrace=1:halt_on_error=1",
}


def sanitizer_report(err):
    """The first line of a sanitizer report in `err`, or None."""
    for line in err.splitlines():
        if ("Sanitizer:" in line and "ERROR" in line) or ": runtime error:" in line:
            return line.strip()
    return None


def symbolizer():
    """llvm-symbolizer from the LLVM jaic links against, so reports show source lines."""
    prefix = os.environ.get("LLVM_SYS_231_PREFIX")
    candidates = [Path(prefix) / "bin/llvm-symbolizer"] if prefix else []
    candidates += [
        Path(p)
        for p in (shutil.which("llvm-symbolizer-23"), shutil.which("llvm-symbolizer"))
        if p
    ]
    return next((str(p) for p in candidates if p.is_file()), None)


# `--coverage DIR`: where each native run's ingested coverage goes (docs/tools/coverage.md).
COVERAGE_DIR = None


def ingest_coverage(exe, source):
    """Fold the coverage files `exe` wrote into `COVERAGE_DIR` before its debug info is deleted."""
    runs = [str(p) for p in exe.parent.glob(f"{exe.name}.*.cov")]
    if not runs:
        return
    name = re.sub(r"[^A-Za-z0-9_.-]", "_", f"{source.parent.name}_{source.stem}")
    subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools/coverage/report.py"),
            "ingest",
            "--exe",
            str(exe),
            "--out",
            str(COVERAGE_DIR / f"{name}.json"),
            *runs,
        ],
        check=False,
    )


# `run_native`'s status for a program whose build writes no executable.
NO_EXECUTABLE = "no executable"
PLATFORM_UNAVAILABLE = "platform unavailable"
DISABLED = "disabled"
HEADLESS_BUILD = "headless build only"


def opens_windows(path):
    """Whether a case opens windows: it imports Window_Creation. Machines without a display or
    GPU (CI runners) can build such programs but not run them."""
    try:
        return '#import "Window_Creation"' in path.read_text(errors="replace")
    except OSError:
        return False


def run_native(
    jaic,
    path,
    extra,
    build_flags,
    scratch,
    timeout,
    limit_bytes,
    run=True,
    metadata=None,
):
    """Build `path` natively with `build_flags` and run the executable in the source's directory.
    Returns (stdout, stderr, code) like `run_limited`; a failed build or a sanitizer report puts
    an `error:` line first in stderr."""
    # A case's extra arguments are `jaic -interp` options; the program's own come after `--`.
    split = extra.index("--") if "--" in extra else len(extra)
    jaic_args, program_args = extra[:split], extra[split + 1 :]
    if any(a in jaic_args for a in ("-os", "-cpu", "-target")):
        return "", "", NO_EXECUTABLE  # built for another platform
    exe = Path(tempfile.mkdtemp(dir=scratch)) / path.stem
    try:
        _, err, code = run_fixture(
            [jaic, str(path), "-exe", str(exe), *build_flags, *jaic_args],
            path.parent,
            timeout,
            limit_bytes,
            metadata,
        )
        # A program without `main` (its checks are `#run` directives), or whose metaprogram asks
        # for no output, did all its work at compile time.
        if code == NO_MAIN_EXIT:
            return "", "", NO_EXECUTABLE
        if code != 0:
            cause = next(
                (line.strip() for line in err.splitlines() if "error" in line.lower()),
                "",
            )
            return "", f"error: native build failed (exit {code}): {cause}\n{err}", code
        if not exe.exists():
            return "", "", NO_EXECUTABLE
        if not run:
            return "", "", 0
        out, err, code = run_fixture(
            [str(exe), *program_args],
            path.parent,
            timeout,
            limit_bytes,
            metadata,
            jaic=False,
            extra_env={"JAIC_COVERAGE_DIR": str(exe.parent)} if COVERAGE_DIR else None,
        )
        if COVERAGE_DIR:
            ingest_coverage(exe, path)
        report = sanitizer_report(err)
        if report:
            return out, f"error: sanitizer: {report}\n{err}", code if code != 0 else 1
        return out, err, code
    finally:
        shutil.rmtree(exe.parent, ignore_errors=True)


def main():
    from stdlib.stdlib_runtime import shard, take_shard

    ap = argparse.ArgumentParser()
    ap.add_argument("sets", nargs="+")
    add_jaic_argument(ap, type=str)
    ap.add_argument("--filter", default="")
    ap.add_argument(
        "--only-platform",
        metavar="PLATFORM",
        help="upstream: only the enabled cases restricted to PLATFORM (`only` in tools/upstream-cases.json)",
    )
    ap.add_argument(
        "--no-sandbox",
        action="store_true",
        help="run upstream cases without the sandbox (otherwise refused where none exists, except on CI)",
    )
    ap.add_argument("--verbose", "-v", action="store_true")
    ap.add_argument("--timeout", type=float, default=60)
    ap.add_argument(
        "--memory-limit",
        type=float,
        default=3,
        help="GiB a case may allocate before jaic stops it (JAIC_MEMORY_LIMIT; default: 3)",
    )
    ap.add_argument(
        "--jobs",
        "-j",
        type=int,
        default=0,
        help="cases run at once (default: CPU count, capped so jobs x memory limit fits in RAM)",
    )
    ap.add_argument(
        "--allow-stale",
        action="store_true",
        help="run even if the jaic binary is older than the compiler sources",
    )
    ap.add_argument(
        "--headless",
        action="store_true",
        help="build (or check) programs that open windows instead of running them",
    )
    ap.add_argument(
        "--native",
        action="store_true",
        help="build `run` cases with `jaic` and run the executables",
    )
    ap.add_argument(
        "--sanitize",
        default="",
        help="sanitizers for native builds (address, undefined or both, comma-separated); implies --native",
    )
    ap.add_argument(
        "--opt",
        default="",
        choices=["", "O0", "O1", "O2", "O3"],
        help="optimization level of native builds (default: what the program asks for)",
    )
    ap.add_argument(
        "--shard",
        type=shard,
        default=(1, 1),
        metavar="K/N",
        help="run only the K-th of N equal parts of the selected cases (CI splits sweeps this way)",
    )
    ap.add_argument(
        "--coverage",
        type=Path,
        metavar="DIR",
        help="build native cases with -coverage and write each run's line and function hits to DIR "
        "(report with tools/coverage/report.py); implies --native",
    )
    a = ap.parse_args()
    # Cases run in their own directory, so a relative path would not resolve there.
    a.jaic = str(Path(a.jaic).resolve())
    if a.sanitize or a.opt or a.coverage:
        a.native = True
    if a.coverage:
        global COVERAGE_DIR
        COVERAGE_DIR = a.coverage.resolve()
        COVERAGE_DIR.mkdir(parents=True, exist_ok=True)
    build_flags = (
        (["-sanitize", a.sanitize] if a.sanitize else [])
        + (["-coverage"] if a.coverage else [])
        + ([f"-{a.opt}"] if a.opt else [])
    )
    if a.sanitize:
        os.environ.update(
            {k: v for k, v in SANITIZER_ENV.items() if k not in os.environ}
        )
        if "ASAN_SYMBOLIZER_PATH" not in os.environ and symbolizer():
            os.environ["ASAN_SYMBOLIZER_PATH"] = symbolizer()
        if sys.platform == "darwin" and "DYLD_FALLBACK_LIBRARY_PATH" not in os.environ:
            # ASan intercepts dlopen, so a library that loads another by `@loader_path` or its own
            # rpaths resolves from the ASan runtime instead (Homebrew's SDL2 is sdl2-compat, which
            # loads SDL3 that way and shows a modal dialog when it can't). Search Homebrew's lib
            # directory after dyld's defaults.
            brew = brew_prefix()
            os.environ["DYLD_FALLBACK_LIBRARY_PATH"] = ":".join(
                [f"{Path.home()}/lib", "/usr/local/lib", "/usr/lib", *([f"{brew}/lib"] if brew else [])]
            )
    scratch = tempfile.mkdtemp(prefix="jaic_sweep-native-") if a.native else None
    limit_bytes = int(a.memory_limit * 2**30)
    if a.jobs <= 0:
        a.jobs = max(
            1, min(os.cpu_count() or 1, int(physical_memory_gib() // a.memory_limit))
        )
    if not a.allow_stale:
        stale = stale_sources(Path(a.jaic))
        if stale:
            sys.exit(
                f"jaic_sweep: {a.jaic} is older than {stale}; rebuild it (or pass --allow-stale)"
            )
    use_native_libs()
    chosen = None
    if a.only_platform:
        chosen = {c["id"] for c in select_upstream_cases.select(select_upstream_cases.load(), a.only_platform)}

    def wanted(cid):
        return chosen is None or cid in chosen

    global SANDBOX
    if "upstream" in a.sets:
        try:
            confined = usb.policy(a.no_sandbox)
        except usb.Unsandboxed as error:
            sys.exit(f"jaic_sweep: {error}")
        if not confined:
            print("jaic_sweep: upstream cases run WITHOUT a sandbox", file=sys.stderr)
        else:
            # Absolute: the default lookup goes through `crates/jaic/..`, which a sandbox does not mount.
            os.environ.setdefault("JAIC_STDLIB", str(ROOT / "stdlib"))
            SANDBOX = {"reads": usb.jaic_reads(a.jaic, ROOT / "stdlib", UPSTREAM) + [Path(tempfile.gettempdir())]}
        for c in json.loads(UPSTREAM_CASES.read_text()):
            UPSTREAM_IDS.add(c["id"])
    # Upstream cases may name setup commands (building a C library the program loads); they run
    # once, serially, in the case's directory before any case starts.
    if "upstream" in a.sets:
        for c in json.loads(UPSTREAM_CASES.read_text()):
            if a.filter in c["id"] and wanted(c["id"]) and not (select_upstream_cases.pending(c) and not SANDBOX):
                if "copy" in c or (SANDBOX and (c["mode"] == "build" or c.get("run_after") or c.get("setup"))):
                    WORKDIRS[c["id"]] = make_workdir(c)
                for command in c.get("setup", []):
                    box = case_box(c["id"])
                    env = None
                    if box:
                        command, env = usb.confine(box[0], box[1], command, dict(os.environ))
                    subprocess.run(
                        command,
                        cwd=(upstream_root(c) / c["path"]).parent,
                        env=env,
                        capture_output=True,
                        stdin=subprocess.DEVNULL,
                    )
    if "modules" in a.sets:
        from stdlib.stdlib_runtime import build_data_fixtures

        build_data_fixtures()
    todo = take_shard(
        [c for s in a.sets for c in cases(s) if a.filter in c[0] and wanted(c[0])],
        a.shard
    )
    if not todo:
        ap.error("test selection is empty")

    def run(case):
        _case.box = case_box(case[0])
        try:
            return run_case(case)
        finally:
            _case.box = None

    def run_case(case):
        cid, path, mode, expect, extra = case
        if cid in DISABLED_REASONS:
            return cid, None, "", DISABLED_REASONS[cid], DISABLED
        if cid in UPSTREAM_PENDING and not SANDBOX:
            # Even `--no-sandbox` and CI runners do not excuse this: nobody has read the code.
            return cid, None, "", "review pending: unreviewed code does not run without a sandbox", PLATFORM_UNAVAILABLE
        if cid in UPSTREAM_PLATFORMS:
            from stdlib.stdlib_runtime import host_platform

            only = UPSTREAM_PLATFORMS[cid]
            if host_platform() not in only["only"]:
                reason = f"runs only on {', '.join(only['only'])}: {only.get('only_reason', '')}"
                return cid, None, "", reason, PLATFORM_UNAVAILABLE
        metadata = CASE_METADATA.get(cid, {})
        if metadata:
            from stdlib.stdlib_test_catalog import (
                unavailable_reason,
                disabled_reason,
            )
            from stdlib.stdlib_runtime import host_platform, program_arguments, prepare_program

            rejected = disabled_reason(metadata)
            if rejected:
                return cid, None, "", rejected, DISABLED
            reason = unavailable_reason(
                metadata, "native" if a.native else "interp", host_platform()
            )
            if reason:
                return cid, None, "", reason, PLATFORM_UNAVAILABLE
            try:
                arguments = program_arguments(
                    metadata, scratch or tempfile.gettempdir(), os.environ
                )
                path = prepare_program(metadata, path, scratch or tempfile.gettempdir())
            except ValueError as error:
                return cid, False, "", "error: fixture setup: " + str(error), 1
            if arguments:
                if "--" in extra:
                    return (
                        cid,
                        False,
                        "",
                        "error: conflicting fixture argument providers",
                        1,
                    )
                extra = [*extra, "--", *arguments]
        # Source-check-only programs must never run (an uninitialized ring, say): they only build.
        windowed = mode == "run" and (
            metadata.get("source_check_only") is True
            or (
                a.headless
                and (metadata.get("requires_display") or metadata.get("window_lease") or opens_windows(path))
            )
        )
        if windowed and expect is not None:
            expect = None  # Only whether it builds is checked.
        if a.native and mode == "run":
            out, err, code = run_native(
                a.jaic,
                path,
                extra,
                build_flags,
                scratch,
                a.timeout,
                limit_bytes,
                run=not windowed,
                metadata=metadata,
            )
            if code == NO_EXECUTABLE:
                return cid, None, out, err, code
        else:
            # A sweep only checks what metaprograms' workspaces produce; it writes nothing.
            quiet = ["-no_workspace_output"] if mode == "run" and not windowed else []
            # `run` interprets, `check` only compiles, `build` is jaic's default.
            mode_flag = {"run": ["-interp"], "check": ["-check"], "build": []}[mode]
            if windowed:
                mode_flag = ["-check"]
            out, err, code = run_fixture(
                [a.jaic, str(path), *mode_flag, *quiet, *extra],
                path.parent,
                a.timeout,
                limit_bytes,
                metadata,
            )
        # `run_after`: the program the build produced is run, and the expectations apply to it.
        if windowed and code == 0:
            return cid, None, out, err, HEADLESS_BUILD
        if code == 0 and expect and expect.get("run_after"):
            program = expect["run_after"]
            out, err, code = run_limited(
                [str(path.parent / program[0]), *program[1:]],
                path.parent,
                a.timeout,
                limit_bytes,
                jaic=False,
            )
        if expect and expect.get("mode") == "check-fail":
            from stdlib.stdlib_runtime import contract_result

            ok = contract_result(expect, None if code == -1 else code, out + err)
            if expect.get("positive_baseline"):
                from stdlib.stdlib_test_catalog import validated_entries

                baseline = next(
                    c
                    for c in validated_entries()
                    if c["id"] == expect["positive_baseline_id"]
                )
                _, baseline_err, baseline_code = run_limited(
                    [a.jaic, str(baseline["path"]), "-check"],
                    baseline["path"].parent,
                    a.timeout,
                    limit_bytes,
                )
                if baseline_code != 0:
                    ok = False
                    err += "\npositive baseline failed:\n" + baseline_err
        elif expect and "error_contains" in expect:
            from stdlib.stdlib_runtime import contract_result

            ok = contract_result(expect, code, out, err, native=a.native and mode == "run")
        elif expect and "stdout_one_of" in expect:
            ok = code == expect.get("exit_code", 0) and out in expect["stdout_one_of"]
        elif expect is None:
            ok = code == 0
        elif "upstream" in expect:
            problems = (
                [f"exit code {code}, expected {expect['exit_code']}"]
                if code != expect["exit_code"]
                else []
            )
            if expect["stdout"] is not None and out != expect["stdout"]:
                problems.append("stdout differs from the recorded output")
            problems += [
                f"stdout lacks {line!r}"
                for line in expect["contains"]
                if line not in out
            ]
            gap = missing_in_order(expect["ordered"], out)
            if gap is not None:
                problems.append(f"stdout lacks {gap!r} (in order)")
            ok = not problems
            err = "".join(f"error: {p}\n" for p in problems) + err
        elif "negative" in expect:
            ok = code != 0 and code != -1 and expect["negative"] in err
        elif "contains" in expect:
            missing = [line for line in expect["contains"] if line not in out]
            present = [text for text in expect["excludes"] if text in out]
            ok = code == expect.get("exit_code", 0) and not missing and not present
            if code == expect.get("exit_code", 0) and not ok:
                err += "".join(f"\nerror: stdout lacks {line!r}" for line in missing)
                err += "".join(f"\nerror: stdout has {text!r}" for text in present)
        elif "error" in expect:
            # A runtime error: any failing status (a native build traps), with the message.
            ok = (
                code not in (0, -1)
                and expect["error"] in err
                and out == expect.get("stdout", "")
            )
        else:
            ok = code == expect.get("exit_code", 0) and out == expect.get("stdout", "")
        return cid, ok, out, err, code

    passed, failed, no_exe, unavailable, built_only, rejected = 0, [], [], [], [], []
    with ThreadPoolExecutor(max_workers=max(1, a.jobs)) as pool:
        for cid, ok, out, err, code in pool.map(run, todo):
            if ok is None:
                if code == DISABLED:
                    rejected.append((cid, err))
                    continue
                if code == PLATFORM_UNAVAILABLE:
                    unavailable.append((cid, err))
                elif code == HEADLESS_BUILD:
                    built_only.append(cid)
                else:
                    no_exe.append(cid)
                continue
            if ok:
                passed += 1
                continue
            first = next(
                (line for line in err.splitlines() if "error" in line),
                err.strip().splitlines()[0]
                if err.strip()
                else f"exit {code}, stdout {out[:80]!r}",
            )
            failed.append((cid, first))
            if a.verbose:
                print(f"--- {cid}\n{err[:1500]}{out[:500]}")
    for root in [*WORKDIRS.values(), *CASE_SCRATCH]:
        shutil.rmtree(root, ignore_errors=True)
    if scratch:
        shutil.rmtree(scratch, ignore_errors=True)
    for cid, msg in failed:
        print(f"FAIL {cid}: {msg[:220]}")
    if no_exe:
        print(
            f"\nnot run natively (compile-time only, interpreter-only or another platform): {', '.join(no_exe)}"
        )
    for cid, reason in unavailable:
        print(f"UNAVAILABLE {cid}: {reason}")
    for cid, reason in rejected:
        print(f"DISABLED {cid}: {reason}")
    if built_only:
        print(
            "Built only under --headless (runtime not passed): " + ", ".join(built_only)
        )
    print(
        f"\n{passed} passed, {len(failed)} failed, {len(unavailable)} platform unavailable, {len(built_only)} built only, {len(rejected)} disabled"
        + (f", {len(no_exe)} not run natively" if no_exe else "")
    )
    # A failing status lets CI use the sweep directly.
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
