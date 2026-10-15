#!/usr/bin/env python3
"""Cross-build Windows test programs with `jaic -os windows`, and run them on Windows.

    windows_cross.py build --jaic path/to/jaic --out dir   # any host with MinGW-w64
    windows_cross.py build --cpu arm64 --jaic jaic --out dir   # with llvm-mingw
    windows_cross.py build --host --jaic jaic.exe --out dir # on Windows, its own toolchain
    windows_cross.py run --dir dir                          # on Windows

`build` compiles every corpus case with a runtime expectation (tests/corpus/manifest.json),
the self-checking programs in WINDOWS_PROGRAMS, the C struct fixture and, with `--stdlib`,
the reviewed contracts and independent regression catalogue (tools/stdlib_test_catalog.py) to `dir/<id>.exe` and writes
`dir/expected.json`. `run` executes them and compares exit code and stdout (for a case that
expects a runtime error, a failing status and the error's message on stderr). See
docs/native/windows.md.
"""
import argparse
import concurrent.futures
import json
import os
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
import stdlib_runtime  # noqa: E402

# Self-checking programs (each prints "ok") exercised on Windows besides the corpus.
WINDOWS_PROGRAMS = [
    "tests/native/windows/runtime.jai",
    "tests/native/windows/variadic.jai",
    "tests/compiler-regressions/struct-literal-overrides-default-string.jai",
    "tests/compiler-regressions/array-literal-view-lifetime.jai",
    "tests/compiler-regressions/c-variadic-foreign-calls.jai",
    "tests/compiler-regressions/member-align-lowers-alignment.jai",
    "tests/compiler-regressions/over-aligned-allocation.jai",
]

def target_platform(args):
    """Actual build target; no legacy expected-failure baseline."""
    if args.host:
        return stdlib_runtime.host_platform()
    return f"windows-{args.cpu}-mingw"


def cases(stdlib, platform=None):
    """(id, source, exit code or the runtime error message (a str) the program must stop with,
    stdout or None for "not checked", must build)."""
    manifest = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
    for case in manifest["cases"]:
        runtime = case.get("runtime")
        if runtime is None:
            continue
        yield (
            case["id"],
            ROOT / "tests/corpus" / case["source"],
            runtime.get("error", runtime.get("exit_code", 0)),
            runtime.get("stdout", ""),
            True,
        )
    listed = set()
    for program in WINDOWS_PROGRAMS:
        path = ROOT / program
        listed.add(path)
        yield path.stem, path, 0, "ok\n", True
    if not stdlib:
        return
    # Expected compile-time failures are validated separately; they are not executables.
    for case in stdlib_runtime.validated_entries():
        if case.get("mode") == "check-fail" or "error_contains" in case:
            continue
        if case["path"] not in listed:
            yield "stdlib-" + case["id"].replace("/", "-"), case["path"], 0, case.get("stdout"), True


# tests/native/c-structs-by-value: C structs by value both ways across the C ABI, against C
# compiled by a Windows toolchain (same expectations as crates/jaic-cli/tests/native.rs).
C_STRUCTS = {
    "foreign_calls": "{11, 22} {2, 4, 6} {5, 6, 7, 8} 10 {-7, 9} {99, 2.5} {11, 22, 33} {11, 22}\n832\n8940414\n",
    "callbacks": "{111, 47} {10, 20, 30, 40} {8, 4}\n832\n{12, 10.25} {7.5, 5.25}\n8940414\n",
}


def mingw_prefix(args):
    """The MinGW-w64 tool prefix for the target CPU (llvm-mingw's `aarch64-w64-mingw32-gcc`
    is its Clang under GCC's name)."""
    return "aarch64-w64-mingw32" if args.cpu == "arm64" else "x86_64-w64-mingw32"


def c_structs_library(args, work):
    """Build the fixture's C library in `work`: a static `libstructs.lib` with Clang on a
    Windows host, a `libstructs.dll` with the MinGW-w64 compiler when cross-building (linked
    directly; it must sit next to the executables at run time)."""
    if args.host:
        steps = [
            ["clang", "-c", "-O1", "structs.c", "-o", "structs.o"],
            ["llvm-ar", "rcs", "libstructs.lib", "structs.o"],
        ]
    else:
        steps = [[f"{mingw_prefix(args)}-gcc", "-shared", "-O1", "-o", "libstructs.dll", "structs.c"]]
    for step in steps:
        result = subprocess.run(step, cwd=work, capture_output=True, text=True)
        if result.returncode != 0:
            return f"{' '.join(step)}: {result.stderr.strip()}"
    return None


def build_one(args, source, output):
    target = [] if args.host else ["-os", "windows", "-cpu", args.cpu]
    try:
        result = subprocess.run(
            [str(pathlib.Path(args.jaic).resolve()), str(source), *target, "-exe", str(output)],
            cwd=source.parent,
            capture_output=True,
            text=True,
            timeout=300,
        )
    except subprocess.TimeoutExpired:
        return "build timed out"
    # A program without `main`, or whose metaprogram writes no output, did its work at compile time.
    if result.returncode != 0 and ("no exported 'main'" in result.stderr or "has no `main` procedure" in result.stderr):
        return COMPILE_TIME_ONLY
    if result.returncode != 0:
        return f"exit {result.returncode}: {result.stderr.strip()}"
    if not output.with_name(output.name + ".exe").exists():
        return COMPILE_TIME_ONLY
    return None


COMPILE_TIME_ONLY = "compile-time only: no executable to run"


CROSS_DLLS = []


def use_cross_libs(args):
    """Cross-build the third-party C libraries the stdlib tests link (stb_*, FreeType, lz4...)
    with the MinGW toolchain, and point `jaic` at them (JAIC_CROSS_LIBS). They are static
    archives, so the executables need no DLL beside them, except SDL2, which comes as the
    official release's import library and DLL (copied beside the executables, see `build`)."""
    platform = target_platform(args)
    subprocess.run([sys.executable, str(ROOT / "tools/build_native_libs.py"), "--platform", platform], check=True)
    import build_native_libs

    os.environ["JAIC_CROSS_LIBS"] = str(build_native_libs.output_dir(platform))
    # SDL2 is linked through its import library, so its DLL goes beside the executables.
    CROSS_DLLS.extend(build_native_libs.output_dir(platform).glob("SDL2.dll"))


def build(args):
    out = pathlib.Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    if args.stdlib and not args.host:
        use_cross_libs(args)
    expected, failures, skipped = {}, [], []
    negative_checks = 0
    if args.stdlib:
        catalogue = stdlib_runtime.validated_entries()
        target = [] if args.host else ["-os", "windows", "-cpu", args.cpu]
        for case in catalogue:
            if case.get("mode") != "check-fail" and "error_contains" not in case:
                continue
            flags = ["-check"] if case.get("mode") == "check-fail" else ["-interp", "-no_workspace_output"]
            code, output = stdlib_runtime.run_command([str(pathlib.Path(args.jaic).resolve()), str(case["path"]), *target, *flags],
                                                      case["path"].parent, 300, dict(os.environ))
            valid = stdlib_runtime.contract_result(case, code, output)
            if case.get("positive_baseline"):
                baseline = next(c for c in catalogue if c["id"] == "contracts/" + case["positive_baseline"])
                bcode, bout = stdlib_runtime.run_command([str(pathlib.Path(args.jaic).resolve()), str(baseline["path"]), *target, "-check"],
                                                         baseline["path"].parent, 300, dict(os.environ))
                valid = valid and bcode == 0
                if bcode != 0:
                    output += "\npositive baseline failed:\n" + bout
            if valid:
                negative_checks += 1
            else:
                failures.append(case["id"] + ": expected precise rejection did not match:\n" + output)
        print(f"{negative_checks} expected negative target checks; no native execution claim")
    all_cases = list(cases(args.stdlib, target_platform(args)))
    # Builds are independent; run several at once (each is mostly single-threaded).
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count() or 2) as pool:
        errors = list(pool.map(lambda case: build_one(args, case[1], out / case[0]), all_cases))
    for (case_id, source, exit_code, stdout, required), error in zip(all_cases, errors):
        if error == COMPILE_TIME_ONLY:
            skipped.append(f"{case_id}: {error}")
            continue
        if error:
            (failures if required else skipped).append(f"{case_id}: {error[-3000:]}")
            continue
        # Programs run from their source directory (relative to the checkout), as the sweep does.
        cwd = source.parent.relative_to(ROOT).as_posix()
        status = {"error": exit_code} if isinstance(exit_code, str) else {"exit_code": exit_code}
        expected[case_id] = {**status, "stdout": stdout, "cwd": cwd}
    work = out / "c-structs-src"
    shutil.copytree(ROOT / "tests/native/c-structs-by-value", work, dirs_exist_ok=True)
    error = c_structs_library(args, work)
    if error:
        failures.append(f"c-structs library: {error}")
    else:
        if (work / "libstructs.dll").exists():
            shutil.copy(work / "libstructs.dll", out / "libstructs.dll")
        for name, stdout in C_STRUCTS.items():
            case_id = f"c-structs-{name}"
            error = build_one(args, work / f"{name}.jai", out / case_id)
            if error:
                failures.append(f"{case_id}: {error}")
                continue
            expected[case_id] = {"exit_code": 0, "stdout": stdout}
    for dll in CROSS_DLLS:
        shutil.copy2(dll, out / dll.name)
    (out / "expected.json").write_text(json.dumps(expected, indent=2))
    print(f"built {len(expected)} programs into {out}; {len(skipped)} compile-time only or optional ones do not build")
    for line in skipped:
        print(f"not built: {line}")
    for failure in failures:
        print(f"BUILD FAILED {failure}")
    return 1 if failures and not args.keep_going else 0


def run(args):
    directory = pathlib.Path(args.dir).resolve()
    expected = json.loads((directory / "expected.json").read_text())
    failures = []
    for case_id, want in sorted(expected.items()):
        exe = directory / f"{case_id}.exe"
        cwd = ROOT / want.get("cwd", ".")
        try:
            result = subprocess.run(
                [str(exe)], cwd=cwd if cwd.is_dir() else directory, capture_output=True, timeout=60
            )
        except subprocess.TimeoutExpired:
            failures.append(f"{case_id}: timed out")
            continue
        stdout = result.stdout.decode("utf-8", "replace").replace("\r\n", "\n")
        stderr = result.stderr.decode("utf-8", "replace")
        stdout_ok = want["stdout"] is None or stdout == want["stdout"]
        if "error" in want:
            # A runtime error: any failing status, with the check's message on stderr.
            status_ok = result.returncode != 0 and want["error"] in stderr
            wanted = f"a runtime error, {want['error']!r}"
        else:
            status_ok = result.returncode == want["exit_code"]
            wanted = want["exit_code"]
        if not status_ok or not stdout_ok:
            failures.append(
                f"{case_id}: exit {result.returncode} (want {wanted}), "
                f"stdout {stdout!r} (want {want['stdout']!r}), "
                f"stderr {stderr[-2000:]!r}"
            )
    print(f"{len(expected) - len(failures)} of {len(expected)} programs match")
    for failure in failures:
        print(f"FAIL {failure}")
    return 1 if failures else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)
    b = sub.add_parser("build")
    b.add_argument("--jaic", required=True)
    b.add_argument("--out", required=True)
    b.add_argument("--keep-going", action="store_true", help="exit 0 even if some builds fail")
    b.add_argument("--host", action="store_true", help="build for the host (on Windows) instead of -os windows")
    b.add_argument(
        "--cpu",
        choices=["x64", "arm64"],
        default="x64",
        help="target CPU when cross-building (arm64 needs llvm-mingw on PATH)",
    )
    b.add_argument("--stdlib", action="store_true", help="also the stdlib runtime tests")
    r = sub.add_parser("run")
    r.add_argument("--dir", required=True)
    args = parser.parse_args()
    return build(args) if args.command == "build" else run(args)


if __name__ == "__main__":
    sys.exit(main())
