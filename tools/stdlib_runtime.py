#!/usr/bin/env python3
"""Run every stdlib module test and compiler regression through one catalogue.

No inherited compatibility skip/coverage baseline is consulted. Native-only SDK
and graphics cases are reported unavailable in WASM modes, not counted passed.
Tool-driven integration suites (native SDK probes, generator workflows) are separate gates.

`--modes check --platform windows-arm64` type-checks every case that applies to that target
(`-check -no_dce -os windows -cpu arm64`; `--platform wasm-wasm64` for WASI) without running
anything: the cross-target source check. The other modes run on this host only.
"""

import argparse
import hashlib
import json
import threading
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time
from stdlib_test_catalog import (
    validated_entries,
    unavailable_reason,
    reference_rejected_reason,
    case_for_mode,
)
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODES = ["interp", "native", "wasm-interp", "wasm-native", "check"]
COVERAGE_LOCK = threading.Lock()
CLIPBOARD_LOCK = threading.Lock()
# Only one application at a time is active on a Mac, and a program's windows are key (and its
# focus events arrive) only while it is: GUI programs would otherwise take focus from each other.
WINDOW_LEASE = threading.Lock()


def host_platform():
    os_name = {"darwin": "macos", "win32": "windows"}.get(sys.platform, "linux")
    machine = platform.machine().lower()
    arch = "arm64" if machine in ("arm64", "aarch64") else "x64"
    return f"{os_name}-{arch}"


def cases():
    for case in validated_entries():
        yield case["id"], case["path"]


def contract_result(case, code, output, error="", native=False):
    if case.get("mode") == "check-fail":
        diagnostic = case.get("diagnostic_contains")
        if not isinstance(diagnostic, str) or not diagnostic.strip():
            raise ValueError("check-fail requires a precise diagnostic")
        return code is not None and code != 0 and diagnostic in output + error
    if "error_contains" in case:
        # A native executable that fails a check aborts the process, so its status is a signal
        # status rather than the interpreter's `exit_code`; any nonzero end counts.
        if native and case.get("mode") == "fault":
            exit_ok = code is not None and code != 0
        else:
            exit_ok = code == case["exit_code"]
        return (
            exit_ok
            and case["error_contains"] in output + error
            and output == case.get("stdout", "")
        )
    if "stdout" in case:
        return (
            code == case.get("exit_code", 0) and output == case["stdout"] and error == case.get("stderr", "")
        )
    if "stdout_one_of" in case:
        return (
            code == case.get("exit_code", 0) and output in case["stdout_one_of"] and error == case.get("stderr", "")
        )
    return code == 0


def use_native_libs(plat):
    """Point jaic at the third-party C libraries some tests link (stb_vorbis, rpmalloc, FreeType
    on Windows...), building any that are missing for `plat`, as tools/jaic-sweep.py does."""
    if os.environ.get("JAIC_NATIVE_LIBS") or sys.platform not in (
        "darwin",
        "linux",
        "win32",
    ):
        return
    if plat.endswith("-mingw"):
        return
    # On Windows the selected platform names the CPU (`python` may be an x64 build on arm64).
    plat = plat if sys.platform == "win32" else None
    sys.path.insert(0, str(ROOT / "tools"))
    import build_native_libs

    if build_native_libs.missing(plat):
        extra = ["--platform", plat] if plat else []
        subprocess.run(
            [sys.executable, str(ROOT / "tools/build_native_libs.py"), *extra],
            check=True,
        )
    os.environ["JAIC_NATIVE_LIBS"] = str(build_native_libs.output_dir(plat))


def run_command(command, cwd, timeout, env):
    """(exit code, stdout, stderr); code None on timeout. Streams use separate files: a program
    the test started may outlive a killed one and keep a pipe open."""
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as error:
        proc = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=out,
            stderr=error,
        )
        try:
            code = proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            code = None
        out.seek(0)
        error.seek(0)
        return (
            code,
            out.read().decode("utf-8", "replace").replace("\r\n", "\n"),
            error.read().decode("utf-8", "replace").replace("\r\n", "\n"),
        )


def program_arguments(case, scratch, env):
    if "arguments" in case:
        arguments = case["arguments"]
        if (
            "input" in case
            or not isinstance(arguments, list)
            or not all(isinstance(a, str) for a in arguments)
        ):
            raise ValueError(
                "literal fixture arguments must be a string list without a separate input provider"
            )
        return arguments.copy()
    kind = case.get("input")
    if kind is None:
        return []
    if kind == "font":
        font = env.get("JAIC_TEST_FONT_PATH")
        if not font or not Path(font).is_file():
            raise ValueError(
                "fixture requires JAIC_TEST_FONT_PATH naming a real caller-owned font"
            )
        return [str(Path(font).resolve())]
    if kind == "windows-resource-directory":
        directory = env.get("JAIC_TEST_WINDOWS_RESOURCE_DIR")
        if not directory or not all(
            (Path(directory) / name).is_file()
            for name in ("resource-only.dll", "resource.ico")
        ):
            raise ValueError(
                "Windows resource fixture requires JAIC_TEST_WINDOWS_RESOURCE_DIR with the real source-built resource-only.dll and resource.ico"
            )
        return [str(Path(directory).resolve())]
    if kind not in {"scratch-file", "scratch-directory"}:
        raise ValueError("unknown fixture input: " + kind)
    directory = Path(scratch) / "inputs" / case["id"].replace("/", "-")
    directory.mkdir(parents=True, exist_ok=True)
    return [str(directory / "data" if kind == "scratch-file" else directory)]


def initialize_coverage(path):
    """An empty run has zero recorded hits, even when every compilation fails."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("")


def prepare_program(case, source, scratch):
    """Bind a caller-supplied native probe without modifying its fixture body."""
    libraries = case.get("fixture_libraries")
    if not libraries:
        return source
    if not isinstance(libraries, dict) or not all(
        isinstance(name, str) and name.isidentifier() and isinstance(library, str)
        for name, library in libraries.items()
    ):
        raise ValueError(
            "fixture library bindings must map identifiers to native library names"
        )
    directory = Path(scratch) / "fixture-programs" / case["id"].replace("/", "-")
    directory.mkdir(parents=True, exist_ok=True)
    wrapper = directory / "main.jai"
    text = "".join(
        name + " :: #library " + json.dumps(library) + ";\n"
        for name, library in libraries.items()
    )
    wrapper.write_text(text + "#load " + json.dumps(str(source.resolve())) + ";\n")
    return wrapper


def runtime_prefix(case, env, host=sys.platform):
    if not case.get("clipboard_guard") or host != "darwin":
        return []
    guard = env.get("JAIC_CLIPBOARD_GUARD")
    if not guard or not Path(guard).is_file() or not os.access(guard, os.X_OK):
        raise ValueError(
            "Mac clipboard fixture requires source-built JAIC_CLIPBOARD_GUARD for native restoration"
        )
    return [str(Path(guard).resolve())]


def run_program(command, cwd, timeout, env, case):
    """Run one test program; macOS GUI programs one at a time (see WINDOW_LEASE)."""
    if case.get("window_lease") and sys.platform == "darwin":
        with WINDOW_LEASE:
            return run_unleased(command, cwd, timeout, env, case)
    return run_unleased(command, cwd, timeout, env, case)


def run_unleased(command, cwd, timeout, env, case):
    prefix = runtime_prefix(case, env)
    if not prefix:
        return run_command(command, cwd, timeout, env)
    guarded_env = dict(
        env, JAIC_TEST_CHILD_TIMEOUT_MS=str(max(1, round(timeout * 1000)))
    )
    # The native guard enforces the child deadline, reaps it and restores before
    # returning. Give that cleanup time; serialize clipboard fixtures in this run.
    with CLIPBOARD_LOCK:
        return run_command([*prefix, *command], cwd, timeout + 30, guarded_env)


def windows_library_dirs(directories):
    result = []
    for directory in directories:
        directory = Path(directory)
        imports = directory / "dynamic-imports"
        if imports.is_dir():
            result.append(imports)
        result.append(directory)
    return list(dict.fromkeys(result))


def deploy_windows_dlls(directories, destination):
    """Stage actual package DLLs; reject basename collisions before any copy."""
    planned = {}
    for directory in directories:
        for source in Path(directory).glob("*.dll"):
            key = source.name.casefold()
            digest = hashlib.sha256(source.read_bytes()).digest()
            if key in planned and planned[key][1] != digest:
                raise ValueError("conflicting Windows DLL providers: " + source.name)
            planned.setdefault(key, (source, digest))
    for source, digest in planned.values():
        target = Path(destination) / source.name
        if target.exists() and hashlib.sha256(target.read_bytes()).digest() != digest:
            raise ValueError("conflicting Windows DLL destination: " + source.name)
    for source, _ in planned.values():
        target = Path(destination) / source.name
        if not target.exists():
            shutil.copy2(source, target)


def target_flags(platform_name):
    """`jaic` flags that select the `os-cpu` target of a cross-target check."""
    target_os, _, cpu = platform_name.removesuffix("-mingw").partition("-")
    if target_os == "wasm":
        return ["-target", "wasm64-unknown-wasi"]
    return ["-os", target_os, "-cpu", cpu]


def run_mode(args, test, path, mode, scratch, metadata=None):
    """(passed, detail) of one test in one mode; passed is None when the build wrote nothing to
    run (the program did its work at compile time)."""
    env = dict(os.environ)
    if sys.platform == "win32" and env.get("JAIC_NATIVE_LIBS"):
        env["JAIC_NATIVE_LIBS"] = os.pathsep.join(map(str, windows_library_dirs(
            filter(None, env["JAIC_NATIVE_LIBS"].split(os.pathsep))
        )))
    env.setdefault("JAIC_MEMORY_LIMIT", "3G")
    # Tests may read the mode (docs/tools/testing.md), for example to avoid what the
    # interpreter cannot do yet.
    env["JAIC_STDLIB_TEST_MODE"] = mode
    try:
        path = prepare_program(metadata or {}, path, scratch)
        arguments = program_arguments(metadata or {}, scratch, env)
        runtime_prefix(metadata or {}, env)
    except ValueError as error:
        return False, str(error)
    if mode == "check":
        code, stdout, stderr = run_command(
            [args.jaic, str(path), "-check", "-no_dce", *target_flags(args.platform)],
            path.parent,
            args.timeout,
            env,
        )
        detail = stdout + stderr
        if code is None:
            detail = f"timed out after {args.timeout:.0f}s\n{detail}"
        if metadata and metadata.get("mode") == "check-fail":
            return contract_result(metadata, code, stdout, stderr), detail
        if metadata and "error_contains" in metadata and metadata.get("mode") != "fault":
            return None, "expects an error while interpreting; nothing to check statically"
        if metadata and metadata.get("source_check_only") is True:
            return (None if code == 0 else False), detail
        return code == 0, detail
    wasm = mode.startswith("wasm")
    target = ["-os", "wasm"] if wasm else []
    if metadata and metadata.get("source_check_only") is True:
        code, stdout, stderr = run_command(
            [args.jaic, str(path), *target, "-check"], path.parent, args.timeout, env
        )
        detail = stdout + stderr
        if code is None:
            detail = f"timed out after {args.timeout:.0f}s\n{detail}"
        return (None if code == 0 else False), detail
    if mode in ("interp", "wasm-interp"):
        record = Path(scratch) / f"{test.replace('/', '-')}-{mode}.coverage"
        if args.coverage:
            env["JAIC_COVERAGE"] = str(record)
        code, stdout, stderr = run_program(
            [
                args.jaic,
                str(path),
                "-interp",
                *target,
                *(["--", *arguments] if arguments else []),
            ],
            path.parent,
            args.timeout,
            env,
            metadata or {},
        )
        if args.coverage and record.exists():
            # One file per run, gathered under a lock: concurrent appends could interleave.
            with COVERAGE_LOCK, open(args.coverage, "a", encoding="utf-8") as into:
                into.write(record.read_text(encoding="utf-8", errors="replace"))
            record.unlink()
        out = stdout + stderr
        return contract_result(
            metadata or {}, code, stdout, stderr
        ), out if code is not None else f"timed out after {args.timeout:.0f}s\n{out}"
    exe = Path(tempfile.mkdtemp(dir=scratch)) / test.replace("/", "-")
    try:
        code, stdout, stderr = run_command(
            [args.jaic, str(path), *target, "-" + args.opt, "-exe", str(exe)],
            path.parent,
            args.timeout,
            env,
        )
        out = stdout + stderr
        # A program without `main` did all its work at compile time.
        if (
            code is not None
            and code != 0
            and ("no exported 'main'" in out or "has no `main` procedure" in out)
        ):
            checked, checked_out, checked_error = run_command(
                [args.jaic, str(path), *target, "-check"],
                path.parent,
                args.timeout,
                env,
            )
            if checked == 0:
                return None, "no main: separately checked compile-time only"
            return (
                False,
                "missing-main build also fails strict source checking:\n"
                + out
                + checked_out
                + checked_error,
            )
        if code != 0:
            return (
                False,
                f"build failed ({'timeout' if code is None else f'exit {code}'}):\n{out}",
            )
        built = [
            p
            for p in exe.parent.iterdir()
            if p.name in (exe.name, exe.name + ".exe", exe.name + ".wasm")
        ]
        if not built:
            checked, checked_out, checked_error = run_command(
                [args.jaic, str(path), *target, "-check"],
                path.parent,
                args.timeout,
                env,
            )
            if checked == 0:
                return None, "no executable: separately checked compile-time only"
            return (
                False,
                "missing executable also fails strict source checking:\n"
                + checked_out
                + checked_error,
            )
        if args.dlls and not wasm:
            # Windows loads a DLL from the executable's directory before the system's.
            for dll in args.dlls.glob("*.dll"):
                shutil.copy2(dll, exe.parent / dll.name)
        if sys.platform == "win32" and not wasm and env.get("JAIC_NATIVE_LIBS"):
            try:
                deploy_windows_dlls(
                    filter(None, env["JAIC_NATIVE_LIBS"].split(os.pathsep)), exe.parent
                )
            except (OSError, ValueError) as error:
                return False, "Windows native DLL deployment failed: " + str(error)
        command = (
            ["node", "--no-warnings", str(ROOT / "tools/wasi_run.mjs"), str(built[0])]
            if wasm
            else [str(built[0])]
        )
        code, stdout, stderr = run_program(
            [*command, *arguments], path.parent, args.timeout, env, metadata or {}
        )
        out = stdout + stderr
        return contract_result(
            metadata or {}, code, stdout, stderr, native=not wasm
        ), out if code is not None else f"timed out after {args.timeout:.0f}s\n{out}"
    finally:
        shutil.rmtree(exe.parent)


def modes_given_cross(args):
    """Whether this run executes programs, which only the host can do."""
    return any(m.strip() not in ("check", "") for m in args.modes.split(","))


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    target = os.environ.get("CARGO_TARGET_DIR") or str(ROOT / "target")
    ap.add_argument("--jaic", default=os.path.join(target, "debug", "jaic"))
    ap.add_argument(
        "--modes", default="interp,native", help="comma-separated: " + ", ".join(MODES)
    )
    ap.add_argument(
        "--opt", choices=("O0", "O2"), default="O0", help="native optimization level"
    )
    ap.add_argument(
        "--platform", default=host_platform(), help="actual host/target platform"
    )
    ap.add_argument("--filter", default="", help="only tests whose id contains this")
    ap.add_argument(
        "--case",
        action="append",
        default=[],
        help="select an exact catalogue ID (repeatable)",
    )
    ap.add_argument(
        "--coverage",
        type=Path,
        help="append the procedures the interpreter runs to this file",
    )
    ap.add_argument(
        "--report",
        type=Path,
        help="write completed case results and measured elapsed times as JSON",
    )
    ap.add_argument(
        "--strict",
        action="store_true",
        help="retained CLI spelling; selected supported failures are always fatal",
    )
    ap.add_argument("--timeout", type=float, default=180)
    ap.add_argument(
        "--jobs",
        "-j",
        type=int,
        default=0,
        help="tests at once (default: CPU count, at most 8)",
    )
    ap.add_argument(
        "--dlls",
        type=Path,
        help="copy this directory's DLLs next to each native executable (Mesa's opengl32.dll in CI)",
    )
    ap.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="print the output of every failure in full",
    )
    args = ap.parse_args()
    if modes_given_cross(args) and args.platform.removesuffix("-mingw") != host_platform():
        ap.error(
            f"runtime platform must match actual host {host_platform()}; use --modes check for cross-target coverage"
        )
    args.jaic = str(Path(args.jaic).resolve())
    if not Path(args.jaic).is_file() and not Path(args.jaic + ".exe").is_file():
        sys.exit(f"stdlib_runtime: no jaic at {args.jaic}")
    modes = [m.strip() for m in args.modes.split(",") if m.strip()]
    if any(m not in MODES for m in modes):
        sys.exit(f"stdlib_runtime: modes are {', '.join(MODES)}")
    if args.coverage:
        args.coverage = args.coverage.resolve()
        initialize_coverage(args.coverage)
    if "native" in modes:
        use_native_libs(args.platform)
    catalogue = validated_entries()
    unknown = set(args.case) - {case["id"] for case in catalogue}
    if unknown:
        ap.error("unknown catalogue IDs: " + ", ".join(sorted(unknown)))
    todo = [
        case
        for case in catalogue
        if args.filter in case["id"] and (not args.case or case["id"] in args.case)
    ]
    if not todo:
        ap.error("test selection is empty")
    scratch = tempfile.mkdtemp(prefix="jaic-stdlib-runtime-")
    jobs = args.jobs or min(8, os.cpu_count() or 2)

    def run_test(case):
        test, path = case["id"], case["path"]
        results = []
        source_case = case
        for mode in modes:
            case = case_for_mode(source_case, mode)
            rejected = reference_rejected_reason(case)
            # A cross-target check applies to the cases that could run on that target.
            reason = unavailable_reason(
                case,
                (
                    ("wasm-native" if args.platform.startswith("wasm") else "native")
                    if mode == "check"
                    else mode
                ),
                args.platform,
            )
            started = time.monotonic()
            if rejected:
                passed, detail, reason = None, rejected, None
            elif reason:
                passed, detail = None, reason
            elif mode != "check" and (
                case.get("mode") == "check-fail"
                or ("error_contains" in case and case.get("mode") != "fault")
            ):
                target = ["-os", "wasm"] if mode.startswith("wasm") else []
                check = (
                    ["-check"]
                    if case.get("mode") == "check-fail"
                    else ["-interp", "-no_workspace_output"]
                )
                code, stdout, stderr = run_command(
                    [args.jaic, str(path), *target, *check],
                    path.parent,
                    args.timeout,
                    dict(os.environ),
                )
                detail = stdout + stderr
                if code is None:
                    detail = f"timed out after {args.timeout:.0f}s\n{detail}"
                passed = contract_result(case, code, stdout, stderr)
                if case.get("positive_baseline"):
                    baseline = next(
                        c
                        for c in catalogue
                        if c["id"] == case["positive_baseline_id"]
                    )
                    baseline_code, baseline_out, baseline_error = run_command(
                        [args.jaic, str(baseline["path"]), *target, "-check"],
                        baseline["path"].parent,
                        args.timeout,
                        dict(os.environ),
                    )
                    if baseline_code != 0:
                        passed = False
                        detail += (
                            "\npositive public baseline failed:\n"
                            + baseline_out
                            + baseline_error
                        )
            else:
                passed, detail = run_mode(args, test, path, mode, scratch, case)
            results.append(
                (
                    mode,
                    passed,
                    detail,
                    reason,
                    "error_contains" in case or case.get("mode") == "check-fail",
                    time.monotonic() - started,
                    rejected,
                    case.get("source_check_only") is True,
                )
            )
        return test, results

    failures, unavailable, reference_rejected, counts = (
        [],
        [],
        [],
        {m: [0, 0, 0, 0, 0, 0, 0] for m in modes},
    )
    records = []

    def write_report(status):
        if args.report:
            args.report.parent.mkdir(parents=True, exist_ok=True)
            args.report.write_text(
                json.dumps(
                    {
                        "schema": 1,
                        "status": status,
                        "platform": args.platform,
                        "compiler": args.jaic,
                        "selected_cases": len(todo),
                        "timeout_seconds": args.timeout,
                        "results": records,
                        "counts": {
                            mode: dict(
                                zip(
                                    (
                                        "passed",
                                        "failed",
                                        "platform_unavailable",
                                        "compile_time_only",
                                        "expected_negative_checks",
                                        "reference_rejected",
                                        "source_check_only",
                                    ),
                                    count,
                                )
                            )
                            for mode, count in counts.items()
                        },
                    },
                    indent=2,
                )
                + "\n"
            )

    write_report("running")
    try:
        with ThreadPoolExecutor(max_workers=jobs) as pool:
            futures = [pool.submit(run_test, case) for case in todo]
            for future in as_completed(futures):
                test, results = future.result()
                for (
                    mode,
                    passed,
                    detail,
                    reason,
                    negative,
                    elapsed,
                    rejected,
                    source_only,
                ) in results:
                    c = counts[mode]
                    if rejected is not None:
                        c[5] += 1
                        reference_rejected.append(f"{test} [{mode}]: {rejected}")
                    elif reason is not None:
                        c[2] += 1
                        unavailable.append(f"{test} [{mode}]: {reason}")
                    elif negative and passed:
                        c[4] += 1
                    elif passed is None:
                        c[6 if source_only else 3] += 1
                    elif passed:
                        c[0] += 1
                    else:
                        c[1] += 1
                        failures.append((test, mode, detail))
                    records.append(
                        {
                            "name": test,
                            "mode": mode,
                            "passed": passed,
                            "platform_unavailable": reason,
                            "expected_negative": negative,
                            "reference_rejected": rejected,
                            "source_check_only": source_only,
                            "elapsed_seconds": elapsed,
                            "timed_out": detail.startswith("timed out after "),
                            "failure_detail": detail if passed is False else None,
                        }
                    )
                write_report("running")
    finally:
        shutil.rmtree(scratch)
    write_report("finished")
    for test, mode, detail in failures:
        lines = detail.strip().splitlines()
        shown = lines if args.verbose else lines[-25:]
        print(f"FAIL {test} [{mode}]")
        for line in shown:
            print(f"    {line}")
    for line in unavailable:
        print("UNAVAILABLE " + line)
    for line in reference_rejected:
        print("REFERENCE_REJECTED " + line)
    print(f"\nstdlib runtime tests on {args.platform} ({len(todo)} tests):")
    for mode in modes:
        p, f, u, b, n, r, s = counts[mode]
        print(
            f"  {mode:<12} {p} passed, {f} failed, {u} platform unavailable, {b} compile-time only, {n} expected negative checks, {r} reference-rejected fixtures, {s} source-check only"
        )
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
