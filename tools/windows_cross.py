#!/usr/bin/env python3
"""Cross-build Windows test programs with `jaic build -os windows`, and run them on Windows.

    windows_cross.py build --jaic path/to/jaic --out dir   # any host with MinGW-w64
    windows_cross.py build --host --jaic jaic.exe --out dir # on Windows, its own toolchain
    windows_cross.py run --dir dir                          # on Windows

`build` compiles every corpus case with a runtime expectation (tests/corpus/manifest.json)
plus the self-checking programs in WINDOWS_PROGRAMS to `dir/<id>.exe` and writes
`dir/expected.json`. `run` executes them and compares exit code and stdout. See
docs/native/windows.md.
"""
import argparse
import json
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Self-checking programs (each prints "ok") exercised on Windows besides the corpus.
WINDOWS_PROGRAMS = [
    "tests/native/windows/runtime.jai",
    "tests/stdlib/struct-literal-overrides-default-string.jai",
    "tests/stdlib/array-literal-view-lifetime.jai",
    "tests/stdlib/c-variadic-foreign-calls.jai",
]


def cases():
    manifest = json.loads((ROOT / "tests/corpus/manifest.json").read_text())
    for case in manifest["cases"]:
        runtime = case.get("runtime")
        if runtime is None:
            continue
        yield (
            case["id"],
            ROOT / "tests/corpus" / case["source"],
            runtime.get("exit_code", 0),
            runtime.get("stdout", ""),
        )
    for program in WINDOWS_PROGRAMS:
        path = ROOT / program
        yield path.stem, path, 0, "ok\n"


# tests/native/c-structs-by-value: C structs by value both ways across the C ABI, against C
# compiled by a Windows toolchain (same expectations as crates/jaic-cli/tests/native.rs).
C_STRUCTS = {
    "foreign_calls": "{11, 22} {2, 4, 6} {5, 6, 7, 8} 10 {-7, 9} {99, 2.5} {11, 22, 33}\n832\n",
    "callbacks": "{111, 47} {10, 20, 30, 40} {8, 4}\n832\n",
}


def c_structs_library(args, work):
    """Build the fixture's C library in `work`: a static `libstructs.lib` with Clang on a
    Windows host, a `libstructs.dll` with MinGW-w64 GCC when cross-building (linked directly;
    it must sit next to the executables at run time)."""
    if args.host:
        steps = [
            ["clang", "-c", "-O1", "structs.c", "-o", "structs.o"],
            ["llvm-ar", "rcs", "libstructs.lib", "structs.o"],
        ]
    else:
        steps = [["x86_64-w64-mingw32-gcc", "-shared", "-O1", "-o", "libstructs.dll", "structs.c"]]
    for step in steps:
        result = subprocess.run(step, cwd=work, capture_output=True, text=True)
        if result.returncode != 0:
            return f"{' '.join(step)}: {result.stderr.strip()}"
    return None


def build_one(args, source, output):
    target = [] if args.host else ["-os", "windows"]
    result = subprocess.run(
        [str(pathlib.Path(args.jaic).resolve()), "build", str(source), *target, "-o", str(output)],
        cwd=source.parent,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return f"exit {result.returncode}: {result.stderr.strip()}"
    if not output.with_name(output.name + ".exe").exists():
        return f"no executable written (stdout {result.stdout!r}, stderr {result.stderr!r})"
    return None


def build(args):
    out = pathlib.Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    expected, failures = {}, []
    for case_id, source, exit_code, stdout in cases():
        error = build_one(args, source, out / case_id)
        if error:
            failures.append(f"{case_id}: {error}")
            continue
        expected[case_id] = {"exit_code": exit_code, "stdout": stdout}
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
    (out / "expected.json").write_text(json.dumps(expected, indent=2))
    print(f"built {len(expected)} programs into {out}")
    for failure in failures:
        print(f"BUILD FAILED {failure}")
    return 1 if failures and not args.keep_going else 0


def run(args):
    directory = pathlib.Path(args.dir).resolve()
    expected = json.loads((directory / "expected.json").read_text())
    failures = []
    for case_id, want in sorted(expected.items()):
        exe = directory / f"{case_id}.exe"
        try:
            result = subprocess.run([str(exe)], cwd=directory, capture_output=True, timeout=60)
        except subprocess.TimeoutExpired:
            failures.append(f"{case_id}: timed out")
            continue
        stdout = result.stdout.decode("utf-8", "replace").replace("\r\n", "\n")
        if result.returncode != want["exit_code"] or stdout != want["stdout"]:
            failures.append(
                f"{case_id}: exit {result.returncode} (want {want['exit_code']}), "
                f"stdout {stdout!r} (want {want['stdout']!r}), "
                f"stderr {result.stderr.decode('utf-8', 'replace')[-2000:]!r}"
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
    r = sub.add_parser("run")
    r.add_argument("--dir", required=True)
    args = parser.parse_args()
    return build(args) if args.command == "build" else run(args)


if __name__ == "__main__":
    sys.exit(main())
