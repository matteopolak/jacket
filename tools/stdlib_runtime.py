#!/usr/bin/env python3
"""Run reviewed stdlib contracts, NEW module tests and independent regressions.

No inherited compatibility skip/coverage baseline is consulted. Native-only SDK
and graphics cases are reported unavailable in WASM modes, not counted passed.
Producer-owned specialized suites remain required separate integration gates.
"""
import argparse
import json
import threading
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time
from stdlib_test_catalog import validated_entries, unavailable_reason
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODES = ["interp", "native", "wasm-interp", "wasm-native"]
COVERAGE_LOCK = threading.Lock()
CLIPBOARD_LOCK = threading.Lock()


def host_platform():
    os_name = {"darwin": "macos", "win32": "windows"}.get(sys.platform, "linux")
    machine = platform.machine().lower()
    arch = "arm64" if machine in ("arm64", "aarch64") else "x64"
    return f"{os_name}-{arch}"


def cases():
    for case in validated_entries():
        yield case["id"], case["path"]


def contract_result(case, code, output, error=""):
    if case.get("mode") == "check-fail":
        diagnostic = case.get("diagnostic_contains")
        if not isinstance(diagnostic, str) or not diagnostic.strip():
            raise ValueError("check-fail requires a precise diagnostic")
        return code is not None and code != 0 and diagnostic in output + error
    if "error_contains" in case:
        return (code == case["exit_code"] and case["error_contains"] in output + error
                and output == case.get("stdout", ""))
    if "stdout" in case:
        return code == 0 and output == case["stdout"] and error == case.get("stderr", "")
    return code == 0


def use_native_libs(plat):
    """Point jaic at the third-party C libraries some tests link (stb_vorbis, rpmalloc, FreeType
    on Windows...), building any that are missing for `plat`, as tools/jaic-sweep.py does."""
    if os.environ.get("JAIC_NATIVE_LIBS") or sys.platform not in ("darwin", "linux", "win32"):
        return
    if plat.endswith("-mingw"):
        return
    # On Windows the selected platform names the CPU (`python` may be an x64 build on arm64).
    plat = plat if sys.platform == "win32" else None
    sys.path.insert(0, str(ROOT / "tools"))
    import build_native_libs
    if build_native_libs.missing(plat):
        extra = ["--platform", plat] if plat else []
        subprocess.run([sys.executable, str(ROOT / "tools/build_native_libs.py"), *extra], check=True)
    os.environ["JAIC_NATIVE_LIBS"] = str(build_native_libs.output_dir(plat))


def run_command(command, cwd, timeout, env):
    """(exit code, stdout, stderr); code None on timeout. Streams use separate files: a program
    the test started may outlive a killed one and keep a pipe open."""
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as error:
        proc = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL, stdout=out,
                                stderr=error)
        try:
            code = proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            code = None
        out.seek(0)
        error.seek(0)
        return (code, out.read().decode("utf-8", "replace").replace("\r\n", "\n"),
                error.read().decode("utf-8", "replace").replace("\r\n", "\n"))


def program_arguments(case, scratch, env):
    if 'arguments' in case:
        arguments = case['arguments']
        if 'input' in case or not isinstance(arguments, list) or not all(isinstance(a, str) for a in arguments):
            raise ValueError('literal fixture arguments must be a string list without a separate input provider')
        return arguments.copy()
    kind = case.get("input")
    if kind is None:
        return []
    if kind == "font":
        font = env.get("JAIC_TEST_FONT_PATH")
        if not font or not Path(font).is_file():
            raise ValueError("fixture requires JAIC_TEST_FONT_PATH naming a real caller-owned font")
        return [str(Path(font).resolve())]
    if kind not in {"scratch-file", "scratch-directory"}:
        raise ValueError("unknown fixture input: " + kind)
    directory = Path(scratch) / "inputs" / case["id"].replace("/", "-")
    directory.mkdir(parents=True, exist_ok=True)
    return [str(directory / "data" if kind == "scratch-file" else directory)]


def runtime_prefix(case, env, host=sys.platform):
    if not case.get("clipboard_guard") or host != "darwin":
        return []
    guard = env.get("JAIC_CLIPBOARD_GUARD")
    if not guard or not Path(guard).is_file() or not os.access(guard, os.X_OK):
        raise ValueError("Mac clipboard fixture requires source-built JAIC_CLIPBOARD_GUARD for native restoration")
    return [str(Path(guard).resolve())]


def run_program(command, cwd, timeout, env, case):
    prefix = runtime_prefix(case, env)
    if not prefix:
        return run_command(command, cwd, timeout, env)
    guarded_env = dict(env, JAIC_TEST_CHILD_TIMEOUT_MS=str(max(1, round(timeout * 1000))))
    # The native guard enforces the child deadline, reaps it and restores before
    # returning. Give that cleanup time; serialize clipboard fixtures in this run.
    with CLIPBOARD_LOCK:
        return run_command([*prefix, *command], cwd, timeout + 30, guarded_env)


def run_mode(args, test, path, mode, scratch, metadata=None):
    """(passed, detail) of one test in one mode; passed is None when the build wrote nothing to
    run (the program did its work at compile time)."""
    env = dict(os.environ)
    env.setdefault("JAIC_MEMORY_LIMIT", "3G")
    # Tests may read the mode (docs/tools/stdlib-runtime-tests.md), for example to avoid what the
    # interpreter cannot do yet.
    env["JAIC_STDLIB_TEST_MODE"] = mode
    try:
        arguments = program_arguments(metadata or {}, scratch, env)
        runtime_prefix(metadata or {}, env)
    except ValueError as error:
        return False, str(error)
    wasm = mode.startswith("wasm")
    target = ["-os", "wasm"] if wasm else []
    if mode in ("interp", "wasm-interp"):
        record = Path(scratch) / f"{test.replace('/', '-')}-{mode}.coverage"
        if args.coverage:
            env["JAIC_COVERAGE"] = str(record)
        code, stdout, stderr = run_program([args.jaic, str(path), "-interp", *target,
                                           *(["--", *arguments] if arguments else [])], path.parent, args.timeout, env, metadata or {})
        if args.coverage and record.exists():
            # One file per run, gathered under a lock: concurrent appends could interleave.
            with COVERAGE_LOCK, open(args.coverage, "a", encoding="utf-8") as into:
                into.write(record.read_text(encoding="utf-8", errors="replace"))
            record.unlink()
        out = stdout + stderr
        return contract_result(metadata or {}, code, stdout, stderr), out if code is not None else f"timed out after {args.timeout:.0f}s\n{out}"
    exe = Path(tempfile.mkdtemp(dir=scratch)) / test.replace("/", "-")
    try:
        code, stdout, stderr = run_command([args.jaic, str(path), *target, "-" + args.opt, "-exe", str(exe)], path.parent,
                                args.timeout, env)
        out = stdout + stderr
        # A program without `main` did all its work at compile time.
        if code is not None and code != 0 and ("no exported 'main'" in out or "has no `main` procedure" in out):
            checked, checked_out, checked_error = run_command(
                [args.jaic, str(path), *target, "-check"], path.parent, args.timeout, env)
            if checked == 0:
                return None, "no main: separately checked compile-time only"
            return False, "missing-main build also fails strict source checking:\n" + out + checked_out + checked_error
        if code != 0:
            return False, f"build failed ({'timeout' if code is None else f'exit {code}'}):\n{out}"
        built = [p for p in exe.parent.iterdir() if p.name in (exe.name, exe.name + ".exe", exe.name + ".wasm")]
        if not built:
            checked, checked_out, checked_error = run_command(
                [args.jaic, str(path), *target, "-check"], path.parent, args.timeout, env)
            if checked == 0:
                return None, "no executable: separately checked compile-time only"
            return False, "missing executable also fails strict source checking:\n" + checked_out + checked_error
        if args.dlls and not wasm:
            # Windows loads a DLL from the executable's directory before the system's.
            for dll in args.dlls.glob("*.dll"):
                shutil.copy2(dll, exe.parent / dll.name)
        if sys.platform == "win32" and not wasm and os.environ.get("JAIC_NATIVE_LIBS"):
            # SDL2 ships as an import library plus its DLL (tools/native-libs.json), which the
            # executable needs beside it; the other native libraries are linked statically.
            for dll in Path(os.environ["JAIC_NATIVE_LIBS"]).glob("SDL2.dll"):
                shutil.copy2(dll, exe.parent / dll.name)
        command = ["node", "--no-warnings", str(ROOT / "tools/wasi_run.mjs"), str(built[0])] if wasm else [str(built[0])]
        code, stdout, stderr = run_program([*command, *arguments], path.parent, args.timeout, env, metadata or {})
        out = stdout + stderr
        return contract_result(metadata or {}, code, stdout, stderr), out if code is not None else f"timed out after {args.timeout:.0f}s\n{out}"
    finally:
        shutil.rmtree(exe.parent)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    target = os.environ.get("CARGO_TARGET_DIR") or str(ROOT / "target")
    ap.add_argument("--jaic", default=os.path.join(target, "debug", "jaic"))
    ap.add_argument("--modes", default="interp,native", help="comma-separated: " + ", ".join(MODES))
    ap.add_argument("--opt", choices=("O0", "O2"), default="O0", help="native optimization level")
    ap.add_argument("--platform", default=host_platform(), help="actual host/target platform")
    ap.add_argument("--filter", default="", help="only tests whose id contains this")
    ap.add_argument("--case", action="append", default=[], help="select an exact catalogue ID (repeatable)")
    ap.add_argument("--coverage", type=Path, help="append the procedures the interpreter runs to this file")
    ap.add_argument("--report", type=Path, help="write completed case results and measured elapsed times as JSON")
    ap.add_argument("--strict", action="store_true", help="retained CLI spelling; selected supported failures are always fatal")
    ap.add_argument("--timeout", type=float, default=180)
    ap.add_argument("--jobs", "-j", type=int, default=0, help="tests at once (default: CPU count, at most 8)")
    ap.add_argument("--dlls", type=Path,
                    help="copy this directory's DLLs next to each native executable (Mesa's opengl32.dll in CI)")
    ap.add_argument("--verbose", "-v", action="store_true", help="print the output of every failure in full")
    args = ap.parse_args()
    if args.platform.removesuffix("-mingw") != host_platform():
        ap.error(f"runtime platform must match actual host {host_platform()}; use target source checks for cross-target coverage")
    args.jaic = str(Path(args.jaic).resolve())
    if not Path(args.jaic).is_file() and not Path(args.jaic + ".exe").is_file():
        sys.exit(f"stdlib_runtime: no jaic at {args.jaic}")
    modes = [m.strip() for m in args.modes.split(",") if m.strip()]
    if any(m not in MODES for m in modes):
        sys.exit(f"stdlib_runtime: modes are {', '.join(MODES)}")
    if args.coverage:
        args.coverage = args.coverage.resolve()
        args.coverage.unlink(missing_ok=True)
    if "native" in modes:
        use_native_libs(args.platform)
    catalogue = validated_entries()
    unknown = set(args.case) - {case["id"] for case in catalogue}
    if unknown:
        ap.error("unknown catalogue IDs: " + ", ".join(sorted(unknown)))
    todo = [case for case in catalogue if args.filter in case["id"] and (not args.case or case["id"] in args.case)]
    if not todo:
        ap.error("test selection is empty")
    scratch = tempfile.mkdtemp(prefix="jaic-stdlib-runtime-")
    jobs = args.jobs or min(8, os.cpu_count() or 2)

    def run_test(case):
        test, path = case["id"], case["path"]
        results = []
        for mode in modes:
            reason = unavailable_reason(case, mode, args.platform)
            started = time.monotonic()
            if reason:
                passed, detail = None, reason
            elif case.get("mode") == "check-fail" or "error_contains" in case:
                target = ["-os", "wasm"] if mode.startswith("wasm") else []
                check = ["-check"] if case.get("mode") == "check-fail" else ["-interp", "-no_workspace_output"]
                code, stdout, stderr = run_command([args.jaic, str(path), *target, *check], path.parent, args.timeout, dict(os.environ))
                detail = stdout + stderr
                if code is None:
                    detail = f"timed out after {args.timeout:.0f}s\n{detail}"
                passed = contract_result(case, code, stdout, stderr)
                if case.get("positive_baseline"):
                    baseline = next(c for c in catalogue if c["id"] == "contracts/" + case["positive_baseline"])
                    baseline_code, baseline_out, baseline_error = run_command([args.jaic, str(baseline["path"]), *target, "-check"], baseline["path"].parent, args.timeout, dict(os.environ))
                    if baseline_code != 0:
                        passed = False
                        detail += "\npositive public baseline failed:\n" + baseline_out + baseline_error
            else:
                passed, detail = run_mode(args, test, path, mode, scratch, case)
            results.append((mode, passed, detail, reason, "error_contains" in case or case.get("mode") == "check-fail", time.monotonic() - started))
        return test, results

    failures, unavailable, counts = [], [], {m: [0, 0, 0, 0, 0] for m in modes}  # passed, failed, skipped, built only
    records = []

    def write_report(status):
        if args.report:
            args.report.parent.mkdir(parents=True, exist_ok=True)
            args.report.write_text(json.dumps({
                "schema": 1, "status": status, "platform": args.platform,
                "compiler": args.jaic, "selected_cases": len(todo),
                "timeout_seconds": args.timeout, "results": records,
                "counts": {mode: dict(zip(
                    ("passed", "failed", "platform_unavailable", "compile_time_only", "expected_negative_checks"), count))
                    for mode, count in counts.items()},
            }, indent=2) + "\n")

    write_report("running")
    try:
        with ThreadPoolExecutor(max_workers=jobs) as pool:
            futures = [pool.submit(run_test, case) for case in todo]
            for future in as_completed(futures):
                test, results = future.result()
                for mode, passed, detail, reason, negative, elapsed in results:
                    c = counts[mode]
                    if reason is not None:
                        c[2] += 1
                        unavailable.append(f"{test} [{mode}]: {reason}")
                    elif negative and passed:
                        c[4] += 1
                    elif passed is None:
                        c[3] += 1
                    elif passed:
                        c[0] += 1
                    else:
                        c[1] += 1
                        failures.append((test, mode, detail))
                    records.append({
                        "name": test, "mode": mode, "passed": passed,
                        "platform_unavailable": reason, "expected_negative": negative,
                        "elapsed_seconds": elapsed,
                        "timed_out": detail.startswith("timed out after "),
                        "failure_detail": detail if passed is False else None,
                    })
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
    print(f"\nstdlib runtime tests on {args.platform} ({len(todo)} tests):")
    for mode in modes:
        p, f, u, b, n = counts[mode]
        print(f"  {mode:<12} {p} passed, {f} failed, {u} platform unavailable, {b} compile-time only, {n} expected negative checks")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
