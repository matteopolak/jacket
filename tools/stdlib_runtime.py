#!/usr/bin/env python3
"""Run every stdlib runtime test under the interpreter and as a compiled program.

    stdlib_runtime.py [--jaic PATH] [--modes interp,native,wasm-interp,wasm-native]
                      [--platform KEY] [--filter TEXT] [--coverage FILE] [--strict] [-j N]
                      [--dlls DIR]

The tests are `tests/stdlib/*.jai` and the stdlib's own module tests (`stdlib/<Module>/tests/*.jai`,
`stdlib/tests/**/*.jai`, minus their `modules/` folders of mock modules). Each one checks its own
results with `assert` and must exit 0 in every mode:

    interp       `jaic run file.jai`
    native       `jaic build file.jai` for the host, then the executable
    wasm-interp  `jaic run file.jai -os wasm` (the sandboxed host the browser engine also uses)
    wasm-native  `jaic build file.jai -os wasm`, then the module under node's WASI (tools/wasi_run.mjs)

A program without `main` does its work while it compiles; its build passing is its result in the
`native` and `wasm-native` modes. Programs run in their own directory, one mode after another,
so a test may use fixed scratch paths.

Platforms (`--platform`, default the host's): `macos-arm64`, `macos-x64`, `linux-x64`,
`linux-arm64`, `windows-x64`, `windows-arm64`, and the MinGW cross builds `tools/windows_cross.py`
makes, `windows-x64-mingw` and `windows-arm64-mingw`. `tests/stdlib-runtime-skips.txt` lists the
tests that cannot pass on a platform and mode, with the reason. A skipped test still runs; one that
passes is reported, and fails the run with `--strict` (CI), so the list cannot go stale.

`--coverage FILE` records the procedures the interpreter runs (JAIC_COVERAGE) for
`tools/stdlib_coverage.py`. See docs/tools/stdlib-runtime-tests.md.
"""
import argparse
import fnmatch
import threading
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKIPS = ROOT / "tests/stdlib-runtime-skips.txt"
MODES = ["interp", "native", "wasm-interp", "wasm-native"]
# Skip lines may also name `playground`: the browser engine, run by tools/check_playground_stdlib.mjs
# (platform `browser`) on tests/stdlib/*.jai.
SKIP_MODES = MODES + ["playground"]
COVERAGE_LOCK = threading.Lock()


def host_platform():
    os_name = {"darwin": "macos", "win32": "windows"}.get(sys.platform, "linux")
    machine = platform.machine().lower()
    arch = "arm64" if machine in ("arm64", "aarch64") else "x64"
    return f"{os_name}-{arch}"


def cases():
    """(id, path) of every runtime test: tests/stdlib by file name, module tests by their path
    below stdlib/ without `.jai` (as tools/jaic-sweep.py names them)."""
    for path in sorted((ROOT / "tests/stdlib").glob("*.jai")):
        yield path.stem, path
    stdlib = ROOT / "stdlib"
    found = set(stdlib.glob("*/tests/*.jai")) | set((stdlib / "tests").glob("**/*.jai"))
    for path in sorted(found):
        rel = path.relative_to(stdlib)
        if "modules" not in rel.parts:
            yield rel.with_suffix("").as_posix(), path


def load_skips(path=SKIPS):
    """[(test id, platform patterns, modes, reason)] from the skip list. Each line is
    `id platforms modes reason`: platforms and modes are comma-separated, platforms are
    shell-style patterns (`linux-*`), `*` means every mode; `#` starts a comment."""
    skips = []
    for number, line in enumerate(path.read_text().splitlines(), 1):
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        parts = line.split(None, 3)
        if len(parts) < 4:
            raise SystemExit(f"{path.name}:{number}: expected `test platforms modes reason`")
        test, platforms, modes, reason = parts
        modes = SKIP_MODES if modes == "*" else modes.split(",")
        unknown = [m for m in modes if m not in SKIP_MODES]
        if unknown:
            raise SystemExit(f"{path.name}:{number}: unknown mode {unknown[0]!r}")
        skips.append((test, platforms.split(","), modes, reason))
    return skips


def skip_reason(skips, test, plat, mode):
    """Why `test` is not expected to pass on `plat` in `mode`, or None."""
    for skip_test, platforms, modes, reason in skips:
        if skip_test == test and mode in modes and any(fnmatch.fnmatchcase(plat, p) for p in platforms):
            return reason
    return None


def use_native_libs(plat):
    """Point jaic at the third-party C libraries some tests link (stb_vorbis, rpmalloc, FreeType
    on Windows...), building any that are missing for `plat`, as tools/jaic-sweep.py does."""
    if os.environ.get("JAIC_NATIVE_LIBS") or sys.platform not in ("darwin", "linux", "win32"):
        return
    if plat.endswith("-mingw"):
        return
    # On Windows the skip-list platform names the CPU (`python` may be an x64 build on arm64).
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


def run_mode(args, test, path, mode, scratch):
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
        code, out = run_command([args.jaic, "run", str(path), *target], path.parent, args.timeout, env)
        if args.coverage and record.exists():
            # One file per run, gathered under a lock: concurrent appends could interleave.
            with COVERAGE_LOCK, open(args.coverage, "a", encoding="utf-8") as into:
                into.write(record.read_text(encoding="utf-8", errors="replace"))
            record.unlink()
        return code == 0, out if code is not None else f"timed out after {args.timeout:.0f}s\n{out}"
    exe = Path(tempfile.mkdtemp(dir=scratch)) / test.replace("/", "-")
    try:
        code, out = run_command([args.jaic, "build", str(path), *target, "-o", str(exe)], path.parent,
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
        command = ["node", "--no-warnings", str(ROOT / "tools/wasi_run.mjs"), str(built[0])] if wasm else [str(built[0])]
        code, out = run_command(command, path.parent, args.timeout, env)
        return code == 0, out if code is not None else f"timed out after {args.timeout:.0f}s\n{out}"
    finally:
        shutil.rmtree(exe.parent, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    target = os.environ.get("CARGO_TARGET_DIR") or str(ROOT / "target")
    ap.add_argument("--jaic", default=os.path.join(target, "debug", "jaic"))
    ap.add_argument("--modes", default="interp,native", help="comma-separated: " + ", ".join(MODES))
    ap.add_argument("--platform", default=host_platform(), help="skip-list platform (default: the host's)")
    ap.add_argument("--filter", default="", help="only tests whose id contains this")
    ap.add_argument("--coverage", type=Path, help="append the procedures the interpreter runs to this file")
    ap.add_argument("--strict", action="store_true", help="a skipped test that passes fails the run")
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
    skips = load_skips()
    if "native" in modes:
        use_native_libs(args.platform)
    todo = [(t, p) for t, p in cases() if args.filter in t]
    known = {t for t, _ in cases()}
    for test, *_ in skips:
        if test not in known:
            sys.exit(f"stdlib_runtime: {SKIPS.name} names {test!r}, which is not a test")
    scratch = tempfile.mkdtemp(prefix="jaic-stdlib-runtime-")
    jobs = args.jobs or min(8, os.cpu_count() or 2)

    def run_test(case):
        test, path = case
        results = []
        for mode in modes:
            reason = skip_reason(skips, test, args.platform, mode)
            started = time.monotonic()
            passed, detail = run_mode(args, test, path, mode, scratch)
            results.append((mode, passed, detail, reason, time.monotonic() - started))
        return test, results

    failures, stale, counts = [], [], {m: [0, 0, 0, 0] for m in modes}  # passed, failed, skipped, built only
    try:
        with ThreadPoolExecutor(max_workers=jobs) as pool:
            for test, results in pool.map(run_test, todo):
                for mode, passed, detail, reason, _ in results:
                    c = counts[mode]
                    if reason is not None:
                        c[2] += 1
                        if passed:
                            stale.append(f"{test} [{mode}] passes on {args.platform} but is listed: {reason}")
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
    for line in stale:
        print(f"{'STALE' if args.strict else 'note'}: {line}")
    print(f"\nstdlib runtime tests on {args.platform} ({len(todo)} tests):")
    for mode in modes:
        p, f, s, b = counts[mode]
        print(f"  {mode:<12} {p} passed, {f} failed, {s} skipped, {b} compile-time only")
    return 1 if failures or (args.strict and stale) else 0


if __name__ == "__main__":
    sys.exit(main())
