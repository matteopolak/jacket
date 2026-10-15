#!/usr/bin/env python3
"""Run reviewed stdlib contracts, NEW module tests and independent regressions.

No inherited compatibility skip/coverage baseline is consulted. Native-only SDK
and graphics cases are reported unavailable in WASM modes, not counted passed.
Producer-owned specialized suites remain required separate integration gates.
"""
import argparse
import threading
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time
from stdlib_test_catalog import validated_entries, unavailable_reason
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODES = ["interp", "native", "wasm-interp", "wasm-native"]
COVERAGE_LOCK = threading.Lock()


def host_platform():
    os_name = {"darwin": "macos", "win32": "windows"}.get(sys.platform, "linux")
    machine = platform.machine().lower()
    arch = "arm64" if machine in ("arm64", "aarch64") else "x64"
    return f"{os_name}-{arch}"


def cases():
    for case in validated_entries():
        yield case["id"], case["path"]


def contract_result(case, code, output):
    if case.get("mode") == "check-fail":
        diagnostic = case.get("diagnostic_contains")
        if not isinstance(diagnostic, str) or not diagnostic.strip():
            raise ValueError("check-fail requires a precise diagnostic")
        return code is not None and code != 0 and diagnostic in output
    if "error_contains" in case:
        return code == case["exit_code"] and case["error_contains"] in output
    if "stdout" in case:
        return code == 0 and output == case["stdout"]
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
    """(exit code, output); exit code None on a timeout. Output goes through a file: a program
    the test started may outlive a killed one and keep a pipe open."""
    with tempfile.TemporaryFile() as out:
        proc = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL, stdout=out,
                                stderr=subprocess.STDOUT)
        try:
            code = proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            code = None
        out.seek(0)
        return code, out.read().decode("utf-8", "replace").replace("\r\n", "\n")


def run_mode(args, test, path, mode, scratch, metadata=None):
    """(passed, detail) of one test in one mode; passed is None when the build wrote nothing to
    run (the program did its work at compile time)."""
    env = dict(os.environ)
    env.setdefault("JAIC_MEMORY_LIMIT", "3G")
    # Tests may read the mode (docs/tools/stdlib-runtime-tests.md), for example to avoid what the
    # interpreter cannot do yet.
    env["JAIC_STDLIB_TEST_MODE"] = mode
    wasm = mode.startswith("wasm")
    target = ["-os", "wasm"] if wasm else []
    if mode in ("interp", "wasm-interp"):
        record = Path(scratch) / f"{test.replace('/', '-')}-{mode}.coverage"
        if args.coverage:
            env["JAIC_COVERAGE"] = str(record)
        code, out = run_command([args.jaic, str(path), "-interp", *target], path.parent, args.timeout, env)
        if args.coverage and record.exists():
            # One file per run, gathered under a lock: concurrent appends could interleave.
            with COVERAGE_LOCK, open(args.coverage, "a", encoding="utf-8") as into:
                into.write(record.read_text(encoding="utf-8", errors="replace"))
            record.unlink()
        return contract_result(metadata or {}, code, out), out if code is not None else f"timed out after {args.timeout:.0f}s\n{out}"
    exe = Path(tempfile.mkdtemp(dir=scratch)) / test.replace("/", "-")
    try:
        code, out = run_command([args.jaic, str(path), *target, "-" + args.opt, "-exe", str(exe)], path.parent,
                                args.timeout, env)
        # A program without `main` did all its work at compile time.
        if code != 0 and ("no exported 'main'" in out or "has no `main` procedure" in out):
            return None, "no main: compile-time only"
        if code != 0:
            return False, f"build failed ({'timeout' if code is None else f'exit {code}'}):\n{out}"
        built = [p for p in exe.parent.iterdir() if p.name in (exe.name, exe.name + ".exe", exe.name + ".wasm")]
        if not built:
            return None, "no executable: compile-time only"
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
        code, out = run_command(command, path.parent, args.timeout, env)
        return contract_result(metadata or {}, code, out), out if code is not None else f"timed out after {args.timeout:.0f}s\n{out}"
    finally:
        shutil.rmtree(exe.parent, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    target = os.environ.get("CARGO_TARGET_DIR") or str(ROOT / "target")
    ap.add_argument("--jaic", default=os.path.join(target, "debug", "jaic"))
    ap.add_argument("--modes", default="interp,native", help="comma-separated: " + ", ".join(MODES))
    ap.add_argument("--opt", choices=("O0", "O2"), default="O0", help="native optimization level")
    ap.add_argument("--platform", default=host_platform(), help="actual host/target platform")
    ap.add_argument("--filter", default="", help="only tests whose id contains this")
    ap.add_argument("--coverage", type=Path, help="append the procedures the interpreter runs to this file")
    ap.add_argument("--strict", action="store_true", help="retained CLI spelling; selected supported failures are always fatal")
    ap.add_argument("--timeout", type=float, default=180)
    ap.add_argument("--jobs", "-j", type=int, default=0, help="tests at once (default: CPU count, at most 8)")
    ap.add_argument("--dlls", type=Path,
                    help="copy this directory's DLLs next to each native executable (Mesa's opengl32.dll in CI)")
    ap.add_argument("--verbose", "-v", action="store_true", help="print the output of every failure in full")
    args = ap.parse_args()
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
    todo = [case for case in catalogue if args.filter in case["id"]]
    if not todo:
        ap.error("test selection is empty")
    scratch = tempfile.mkdtemp(prefix="jaic-stdlib-runtime-")
    jobs = args.jobs or min(8, os.cpu_count() or 2)

    def run_test(case):
        test, path = case["id"], case["path"]
        results = []
        for mode in modes:
            reason = unavailable_reason(case, mode)
            started = time.monotonic()
            if reason:
                passed, detail = None, reason
            elif case.get("mode") == "check-fail" or "error_contains" in case:
                target = ["-os", "wasm"] if mode.startswith("wasm") else []
                check = ["-check"] if case.get("mode") == "check-fail" else ["-interp", "-no_workspace_output"]
                code, detail = run_command([args.jaic, str(path), *target, *check], path.parent, args.timeout, dict(os.environ))
                passed = contract_result(case, code, detail)
                if case.get("positive_baseline"):
                    baseline = next(c for c in catalogue if c["id"] == "contracts/" + case["positive_baseline"])
                    baseline_code, baseline_output = run_command([args.jaic, str(baseline["path"]), *target, "-check"], baseline["path"].parent, args.timeout, dict(os.environ))
                    if baseline_code != 0:
                        passed = False
                        detail += "\npositive public baseline failed:\n" + baseline_output
            else:
                passed, detail = run_mode(args, test, path, mode, scratch, case)
            results.append((mode, passed, detail, reason, "error_contains" in case or case.get("mode") == "check-fail", time.monotonic() - started))
        return test, results

    failures, unavailable, counts = [], [], {m: [0, 0, 0, 0, 0] for m in modes}  # passed, failed, skipped, built only
    try:
        with ThreadPoolExecutor(max_workers=jobs) as pool:
            for test, results in pool.map(run_test, todo):
                for mode, passed, detail, reason, negative, _ in results:
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
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
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
        print(f"  {mode:<12} {p} passed, {f} failed, {u} native-only unavailable, {b} compile-time only, {n} expected negative checks")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
