#!/usr/bin/env python3
"""Build the portable stdlib's native fixtures and run its integration checks.

The module tests themselves (including every expected runtime fault and compile-time
rejection) are catalogued under stdlib/**/tests and run by tools/stdlib_runtime.py; this
driver builds the fixtures they read from target/portable-tests, then runs what a catalogued
case cannot: the generated-data cross-checks, parent-exit cancellation, the fatal-signal
crash handlers and the float power audit. `--check-target OS/CPU` type-checks the whole
catalogue for that target through `stdlib_runtime.py --modes check`.
"""
import argparse
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time
from portable_toolchain import default_compiler, checked_compiler

ROOT = Path(__file__).resolve().parents[1]


# Fatal-signal programs: the process must die by the signal after printing both texts.
CRASH_FAULTS = {
    "stdlib/Runtime_Support_Crash_Handler/tests/faults/basics.jai": ("crash signal 11", "native stack:"),
    "stdlib/Runtime_Support_Crash_Handler/tests/faults/memory-fault.jai": ("crash signal ", "native stack:"),
}


def run(command, env, expected=None, cwd=ROOT, expected_stderr="", expected_code=0):
    result = subprocess.run(command, cwd=cwd, env=env, text=True,
                            capture_output=True, timeout=45)
    matches = expected(result.stdout) if callable(expected) else expected is None or result.stdout == expected
    if result.returncode != expected_code or not matches:
        print("FAILED:", " ".join(map(str, command)), file=sys.stderr)
        print(result.stdout, end="", file=sys.stderr)
        print(result.stderr, end="", file=sys.stderr)
        raise RuntimeError(f"exit={result.returncode}, expected stdout={expected!r}")
    if result.stderr != expected_stderr:
        print(result.stderr, end="", file=sys.stderr)
        raise RuntimeError("unexpected compiler/runtime diagnostics")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, default=default_compiler())
    parser.add_argument("--stdlib", type=Path, help="Production tree; no source overlays")
    parser.add_argument("--posix-root", type=Path,
                        help="new independently implemented POSIX module directory")
    parser.add_argument("--native", action="store_true")
    parser.add_argument("--keep-going", action="store_true", help="Run independent cases after failures; overall exit still fails")
    parser.add_argument("--hash-root", type=Path,
                        help="new independently implemented Hash module directory for allocator integration")
    parser.add_argument("--curl-root", type=Path,
                        help="new Curl module directory; defaults beside the supplied POSIX module")
    parser.add_argument("--check-target", action="append", default=[],
                        help="OS/CPU pair, e.g. linux/x64; type-checks the catalogue for that target")
    args = parser.parse_args()
    compiler = checked_compiler(args.jaic, parser)
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_zip_fixtures.py")], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_archive_fixtures.py")], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_mpeg_matrix.py")], cwd=ROOT, check=True)
    if args.stdlib:
        root = args.stdlib.resolve(strict=True)
    else:
        if not args.posix_root:
            parser.error("supply --stdlib for production or --posix-root for explicit staging")
        root = ROOT / "target/portable-integration-stdlib"
        shutil.copytree(ROOT / "stdlib", root, dirs_exist_ok=True)
        shutil.copytree(args.posix_root, root / "POSIX", dirs_exist_ok=True)
        if args.hash_root:
            shutil.copytree(args.hash_root, root / "Hash", dirs_exist_ok=True)
            shutil.copytree(args.hash_root.parent / "internal/hash", root / "internal/hash", dirs_exist_ok=True)
        curl_root = args.curl_root or args.posix_root.parent / "Curl"
        if not curl_root.is_dir():
            parser.error("supply the new Curl dependency with --curl-root")
        shutil.copytree(curl_root, root / "Curl", dirs_exist_ok=True)
        shutil.copytree(curl_root.parent / "internal/bindings", root / "internal/bindings", dirs_exist_ok=True)
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_mpeg_fixtures.py")], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/build_portable_meow_fixtures.py")], cwd=ROOT, check=True)
    env = dict(os.environ, JAIC_STDLIB=str(root))
    output = ROOT / "target/portable-tests"
    output.mkdir(parents=True, exist_ok=True)
    if args.native:
        subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_runtime_configuration.py"),
                        "--jaic", str(compiler), "--stdlib", str(root)], cwd=ROOT, env=env, check=True)
        if sys.platform == "darwin":
            subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_watcher.py"),
                            "--jaic", str(compiler), "--stdlib", str(root)], cwd=ROOT, env=env, check=True)
    run([str(compiler), "-check", str(ROOT / "stdlib/prelude/tests/bootstrap.jai")], env)
    failures = []
    # This fixture emits 2300 numerical rows, not the ordinary "ok" protocol.
    # Its independent rational oracle remains mandatory, including interpreter execution.
    sweep = [sys.executable, str(ROOT / "tools/check_float_power_sweep.py"),
             "--jaic", str(compiler), "--stdlib", str(root)]
    sweep += ["--interp", "--wasm"] if args.native else ["--interp-only"]
    try:
        subprocess.run(sweep, cwd=ROOT, env=env, check=True)
    except (subprocess.CalledProcessError, OSError) as error:
        if not args.keep_going:
            raise
        failures.append("float-power-sweep exact-rational audit")
        print("FAILED specialized float-power audit:", error, flush=True)
    parent_source = ROOT / "stdlib/Process/tests/parent.jai"
    for native in ([False, True] if args.native else [False]):
        for keep in [False, True]:
            marker = parent_source.parent / ("process-retained-marker.tmp" if keep else "process-killed-marker.tmp")
            marker.unlink(missing_ok=True)
            extra = ["-add", "PROCESS_KEEP_ALIVE :: true;"] if keep else []
            try:
                if native:
                    executable = output / f"parent-{keep}"
                    run([str(compiler), str(parent_source), *extra, "-exe", str(executable)], env)
                    run([str(executable)], env, "ok\n", parent_source.parent)
                else:
                    run([str(compiler), "-interp", str(parent_source), *extra, "--"], env, "ok\n")
                deadline = time.monotonic() + (2 if keep else 0.5)
                while time.monotonic() < deadline:
                    if marker.exists():
                        break
                    time.sleep(0.01)
                if marker.exists() != keep:
                    raise RuntimeError(f"parent-exit cleanup mismatch: native={native}, keep={keep}")
            finally:
                marker.unlink(missing_ok=True)
    print("passed parent-exit cancellation and opt-out", flush=True)
    for relative, texts in CRASH_FAULTS.items():
        source = ROOT / relative
        commands = [[str(compiler), "-interp", str(source), "--"]]
        if args.native:
            executable = output / source.stem
            run([str(compiler), str(source), "-exe", str(executable)], env)
            commands.append([str(executable)])
        for command in commands:
            result = subprocess.run(command, cwd=ROOT, env=env, text=True,
                                    capture_output=True, timeout=45)
            if result.returncode == 0 or not all(text in result.stderr for text in texts):
                raise RuntimeError(f"expected checked runtime fault: {result.returncode}: {result.stdout}{result.stderr}")
            if result.returncode not in {-signal.SIGSEGV, -signal.SIGBUS}:
                raise RuntimeError("crash handler failed to preserve fatal signal termination")
        print("passed fatal-signal crash report", source.name, flush=True)
    if (output / "cpp-failure/old").read_text() != "preserved":
        raise RuntimeError("failed C compilation replaced the existing output")
    leftovers = list(output.rglob("*.build-*")) + list(output.rglob("*.universal-*"))
    if leftovers:
        raise RuntimeError(f"C helper left intermediate directories: {leftovers}")
    mail = [sys.executable, "-B", str(ROOT / "tools/check_portable_mail.py"),
            "--jaic", str(compiler), "--stdlib", str(root)]
    if args.native:
        mail.append("--native")
    try:
        subprocess.run(mail, cwd=ROOT, env=env, check=True)
    except subprocess.CalledProcessError:
        if not args.keep_going: raise
        failures.append("SMTP/MIME integration")
    channels = [sys.executable, "-B", str(ROOT / "tools/check_portable_channels.py"),
                "--jaic", str(compiler), "--stdlib", str(root)]
    if args.native:
        channels.append("--native")
    try:
        subprocess.run(channels, cwd=ROOT, env=env, check=True)
    except subprocess.CalledProcessError:
        if not args.keep_going: raise
        failures.append("Shared-memory interprocess integration")
    if args.native:
        subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_crash.py"),
                        "--jaic", str(compiler), "--stdlib", str(root)], cwd=ROOT, env=env, check=True)
    for target in args.check_target:
        before = len(failures)
        checked = subprocess.run([sys.executable, str(ROOT / "tools/stdlib_runtime.py"), "--jaic", str(compiler),
                                  "--modes", "check", "--platform", target.replace("/", "-")], cwd=ROOT, env=env)
        if checked.returncode:
            if not args.keep_going:
                raise RuntimeError(f"target source checks failed: {target}")
            failures.append(f"target {target}")
        print("checked the stdlib catalogue for" if len(failures) == before else "FAILED target source checks for", target, flush=True)
    if failures:
        raise RuntimeError("Failed checks: " + ", ".join(failures))


if __name__ == "__main__":
    main()
