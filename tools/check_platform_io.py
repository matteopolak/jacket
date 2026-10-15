#!/usr/bin/env python3
"""Original platform I/O fixtures; use an explicit source-built compiler.

Run against an integrated stdlib using --stdlib. On Windows --native exercises
actual OS behavior. Cross source checks never count as Windows execution.
"""
import argparse
import hashlib
import json
import os
import platform
import re
import shutil
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def fault_failures(result, enabled, symbols=()):
    """Check real child results; symbols add application-frame quality coverage."""
    failed = []
    if (result.returncode & 0xffffffff) != 0xc0000005:
        failed.append("exception_status")
    if result.stdout:
        failed.append("unexpected_stdout")
    if not enabled:
        if result.stderr:
            failed.append("disabled_handler_stderr")
        return failed
    if b"Windows exception 0x00000000c0000005" not in result.stderr:
        failed.append("exception_header")
    if b"native stack:" not in result.stderr:
        failed.append("stack_header")
    frames = [line for line in result.stderr.splitlines() if line.startswith(b"  0x")]
    if len(frames) < 2:
        failed.append(f"frame_count={len(frames)}")
    for text in [b"unwind failed", b"invalid native stack", b"no progress", b"stack unavailable", b"stack truncated"]:
        if text in result.stderr:
            failed.append(text.decode())
    for symbol in symbols:
        if not any(len(line.split()) >= 2 and line.split()[1] == symbol.encode() for line in frames):
            failed.append("missing_symbol=" + symbol)
    return failed


def nullguard_failures(result):
    """O0 Jai near-null safety is separate from hardware-exception handling."""
    failed = []
    if result.returncode != 1:
        failed.append("nullguard_status")
    if result.stdout:
        failed.append("unexpected_stdout")
    expected = (rb"(?:.*[/])?windows-nullguard\.jai:[0-9]+: error: null pointer dereference: "
                rb"write at address 0x1, just past null \(a member of a null struct pointer\?\)\n")
    if re.fullmatch(expected, result.stderr.replace(b"\\", b"/")) is None:
        failed.append("nullguard_diagnostic")
    return failed


def save_result(directory, result, failed=()):
    """Only original own-fixture output; no environment, auth, or vendor logs."""
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "stdout.bin").write_bytes(result.stdout)
    (directory / "stderr.bin").write_bytes(result.stderr)
    (directory / "result.json").write_text(json.dumps({
        "command": list(map(str, result.args)), "returncode": result.returncode,
        "status_u32": result.returncode & 0xffffffff, "failed": list(failed),
        "stdout_repr": repr(result.stdout), "stderr_repr": repr(result.stderr),
    }, indent=2) + "\n")


def run(command, env, expected=None, expected_stderr=b""):
    result = subprocess.run([str(arg) for arg in command], cwd=ROOT, env=env,
                            capture_output=True, timeout=60)
    if result.returncode or result.stderr.replace(b"\\", b"/") != expected_stderr or (expected is not None and result.stdout != expected.encode()):
        key = hashlib.sha256(repr(list(map(str, command))).encode()).hexdigest()[:16]
        save_result(ROOT / "target/platform-io-tests/failures" / key, result,
                    [f"expected_stdout={expected!r}", f"expected_stderr={expected_stderr!r}"])
        raise RuntimeError(f"{' '.join(map(str, command))}\nexit={result.returncode}\n"
                           f"stdout={result.stdout!r}\nstderr={result.stderr!r}\n"
                           f"expected_stdout={expected!r}; expected_stderr={expected_stderr!r}; own_result={key}")


def expect_diagnostic(command, env, message):
    result = subprocess.run([str(arg) for arg in command], cwd=ROOT, env=env,
                            capture_output=True, timeout=60)
    if result.returncode == 0 or message.encode() not in result.stdout + result.stderr:
        raise RuntimeError(f"missing required diagnostic {message!r}: {result}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, required=True)
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--native", action="store_true")
    parser.add_argument("--cross-check", action="store_true")
    parser.add_argument("--wasm", action="store_true", help="execute Wasm interpreter and real WASI O0/O2")
    parser.add_argument("--case", action="append", choices=sorted(p.stem for p in (ROOT / "tests/platform-io").glob("*-contract.jai")),
                        help="select contract cases; omitted means all cases, argument bridge always runs")
    parser.add_argument("--windows-build", choices=["x64", "arm64"],
                        help="cross-build Windows fixtures; requires the selected native toolchain")
    parser.add_argument("--windows-process", action="store_true",
                        help="check and run the public Windows Process ownership/capture suite")
    parser.add_argument("--windows-crash", action="store_true",
                        help="source-build Windows exception diagnostics and the enabled-startup fault fixture")
    args = parser.parse_args()
    failures = []

    def attempt(command, env, expected=None, expected_stderr=b""):
        try:
            run(command, env, expected, expected_stderr)
            return True
        except (RuntimeError, OSError, subprocess.SubprocessError) as error:
            failures.append(str(error))
            print("FAILED", error, file=sys.stderr, flush=True)
            return False

    def diagnostic(command, env, message):
        try:
            expect_diagnostic(command, env, message)
            return True
        except (RuntimeError, OSError, subprocess.SubprocessError) as error:
            failures.append(str(error))
            print("FAILED", error, file=sys.stderr, flush=True)
            return False

    def report(before, *label):
        print("passed" if len(failures) == before else "FAILED", *label, flush=True)
    compiler = args.jaic.resolve(strict=True)
    if not compiler.is_relative_to(ROOT / "target"):
        parser.error("use the compiler source-built under this checkout target directory")
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve(strict=True)))
    conflict = ROOT / "tests/platform-io/negative/windows-runtime-configuration-conflict.jai"
    for cpu in ["x64", "arm64"]:
        rejected = subprocess.run([str(compiler), str(conflict), "-os", "windows", "-cpu", cpu, "-check"],
                                  cwd=ROOT, env=env, capture_output=True, timeout=60)
        if rejected.returncode != 1 or b"conflicts with the configured runtime instance" not in rejected.stderr or rejected.stdout:
            raise RuntimeError(f"expected original runtime configuration conflict: {rejected}")
    output = ROOT / "target/platform-io-tests"
    output.mkdir(parents=True, exist_ok=True)
    # Portable control-flow unit test only; SDK/native fixtures below prove OS behavior.
    if args.windows_crash:
        iterator_exe = output / ("stack-iteration" + (".exe" if os.name == "nt" else ""))
        if attempt(["clang", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                    ROOT / "tests/platform-io/stack-iteration.c", "-o", iterator_exe], env):
            attempt([iterator_exe], env, "")
    cpu = args.windows_build or ("arm64" if platform.machine().lower() in {"arm64", "aarch64"} and os.name == "nt" else "x64")
    fault_sdk_dll = output / "platform_io_fault_sdk.dll"
    if args.windows_crash:
        # Test-only hardware fault source, compiled independently of Jai guards.
        fault_sdk_library = output / ("platform_io_fault_sdk.lib" if os.name == "nt" else "libplatform_io_fault_sdk.dll.a")
        fault_sdk_dll.unlink(missing_ok=True)
        fault_sdk_library.unlink(missing_ok=True)
        if os.name == "nt":
            fault_compiler = "clang"
            triple = "aarch64" if cpu == "arm64" else "x86_64"
            fault_target = [f"--target={triple}-pc-windows-msvc"]
            fault_linker = [f"-Wl,/IMPLIB:{fault_sdk_library}"]
        else:
            triple = "aarch64" if cpu == "arm64" else "x86_64"
            fault_compiler = shutil.which(triple + "-w64-mingw32-clang") or triple + "-w64-mingw32-gcc"
            fault_target = []
            fault_linker = [f"-Wl,--out-implib,{fault_sdk_library}"]
        attempt([fault_compiler, *fault_target, "-std=c11", "-shared", "-O2", "-g", "-Wall", "-Wextra", "-Werror",
                 ROOT / "tests/platform-io/windows-fault-sdk.c", "-o", fault_sdk_dll, *fault_linker], env)
    if os.name == "nt" or args.windows_build or (args.windows_crash and cpu == "x64"):
        subprocess.run([sys.executable, str(ROOT / "tools/build_platform_io_runtime.py"),
                        "--stdlib", str(args.stdlib.resolve()), "--cpu", cpu], cwd=ROOT, check=True)
        shutil.copy2(args.stdlib.resolve() / "internal/runtime/crash-native/platform_io_windows_crash.dll",
                     output / "platform_io_windows_crash.dll")
    if cpu == "x64" and (os.name == "nt" or args.windows_build or args.windows_crash):
        # This SDK oracle supplies real host-image probes for the test only.
        # It neither replaces nor links into the production crash backend.
        sdk_dll = output / "platform_io_section_sdk.dll"
        sdk_library = output / ("platform_io_section_sdk.lib" if os.name == "nt" else "libplatform_io_section_sdk.dll.a")
        sdk_dll.unlink(missing_ok=True)
        sdk_library.unlink(missing_ok=True)
        if os.name == "nt":
            c_compiler = "clang"
            c_target = ["--target=x86_64-pc-windows-msvc"]
            c_linker = [f"-Wl,/IMPLIB:{sdk_library}"]
        else:
            c_compiler = shutil.which("x86_64-w64-mingw32-clang") or "x86_64-w64-mingw32-gcc"
            c_target = []
            c_linker = [f"-Wl,--out-implib,{sdk_library}"]
        attempt([c_compiler, *c_target, "-std=c11", "-shared", "-O2", "-Wall", "-Wextra", "-Werror",
                 ROOT / "tests/platform-io/windows-section-sdk.c", "-o", sdk_dll, *c_linker, "-ldbghelp"], env)
    helper = output / "process-helper.exe"
    if os.name == "nt":
        helper.unlink(missing_ok=True)
        attempt([compiler, ROOT / "tests/platform-io/windows-process-child.jai", "-O2", "-exe", helper], env)
    fixtures = sorted((ROOT / "tests/platform-io").glob("*-contract.jai"))
    if args.case:
        fixtures = [source for source in fixtures if source.stem in args.case]
    for source in fixtures:
        before = len(failures)
        if args.cross_check:
            for system, target_cpu in [("windows", "x64"), ("windows", "arm64"),
                                ("macos", "arm64"), ("linux", "x64")]:
                attempt([compiler, source, "-check", "-no_dce", "-os", system, "-cpu", target_cpu], env)
        fixture_args = [helper] if os.name == "nt" and source.stem == "process-contract" else []
        if not fixture_args:
            # Exercise ordinary CLI startup as well as the explicit argv form.
            # Always supplying -- hid the Windows zero-vector SDK hook failure.
            attempt([compiler, source, "-interp"], env, "ok\n")
        attempt([compiler, source, "-interp", "--", *fixture_args], env, "ok\n")
        if args.native:
            for optimization in ["-O0", "-O2"]:
                executable = output / (source.stem + optimization + (".exe" if os.name == "nt" else ""))
                if attempt([compiler, source, optimization, "-exe", executable], env):
                    attempt([executable, *fixture_args], env, "ok\n")
        if args.wasm:
            attempt([compiler, source, "-check", "-no_dce", "-os", "wasm"], env)
            attempt([compiler, source, "-interp", "-os", "wasm"], env, "ok\n")
            for optimization in ["-O0", "-O2"]:
                executable = output / (source.stem + optimization + ".wasm")
                if attempt([compiler, source, "-os", "wasm", optimization, "-exe", executable], env):
                    attempt(["node", "--no-warnings", ROOT / "tools/wasi_run.mjs", executable], env, "ok\n")
        report(before, source.relative_to(ROOT))
    source = ROOT / "tests/platform-io/windows-thread-noassert.jai"
    before = len(failures)
    if args.cross_check or os.name == "nt" or args.windows_build:
        for target_cpu in ["x64", "arm64"]:
            attempt([compiler, source, "-os", "windows", "-cpu", target_cpu, "-check", "-no_dce"], env)
    if os.name == "nt":
        attempt([compiler, source, "-interp"], env, "ok\n")
        attempt([compiler, source, "-interp", "--"], env, "ok\n")
        if args.native:
            for optimization in ["-O0", "-O2"]:
                executable = output / (source.stem + optimization + ".exe")
                if attempt([compiler, source, optimization, "-exe", executable], env):
                    attempt([executable], env, "ok\n")
    elif args.windows_build:
        for optimization in ["-O0", "-O2"]:
            attempt([compiler, source, "-os", "windows", "-cpu", args.windows_build,
                     optimization, "-exe", output / (source.stem + optimization + ".exe")], env)
    report(before, "Windows Thread lifecycle, caller assertions disabled", source.relative_to(ROOT))
    source = ROOT / "tests/platform-io/arguments.jai"
    arguments = ["雪😀", "", 'quote" and slash\\']
    before = len(failures)
    attempt([compiler, source, "-interp", "--", *arguments], env, "ok\n")
    if args.native:
        for optimization in ["-O0", "-O2"]:
            executable = output / ("arguments" + optimization + (".exe" if os.name == "nt" else ""))
            if attempt([compiler, source, optimization, "-exe", executable], env):
                attempt([executable, *arguments], env, "ok\n")
    report(before, source.relative_to(ROOT))
    if args.windows_build:
        for source in [*fixtures, ROOT / "tests/platform-io/arguments.jai"]:
            before = len(failures)
            for optimization in ["-O0", "-O2"]:
                executable = output / (source.stem + "-windows-" + args.windows_build + optimization + ".exe")
                attempt([compiler, source, "-os", "windows", "-cpu", args.windows_build,
                     optimization, "-exe", executable], env)
            report(before, "cross-build", source.relative_to(ROOT), args.windows_build)
    if args.windows_process:
        before = len(failures)
        source = ROOT / "tests/platform-io/windows-process.jai"
        helper = ROOT / "tests/platform-io/windows-process-child.jai"
        for target_cpu in ["x64", "arm64"]:
            attempt([compiler, source, "-os", "windows", "-cpu", target_cpu, "-check", "-no_dce"], env)
            attempt([compiler, helper, "-os", "windows", "-cpu", target_cpu, "-check", "-no_dce"], env)
        if os.name == "nt" or args.windows_build:
            target = [] if os.name == "nt" else ["-os", "windows", "-cpu", args.windows_build]
            for optimization in ["-O0", "-O2"]:
                executable = output / ("windows-process" + optimization + ".exe")
                child = output / ("windows-process-child" + optimization + ".exe")
                child_built = attempt([compiler, helper, *target, optimization, "-exe", child], env)
                parent_built = attempt([compiler, source, *target, optimization, "-exe", executable], env)
                if os.name == "nt" and child_built and parent_built:
                    attempt([executable, child], env, "ok\n")
                    for mode in ["parent-kill", "parent-keep"]:
                        marker = output / (mode + optimization + ".marker")
                        marker.unlink(missing_ok=True)
                        try:
                            if not attempt([executable, child, mode, marker], env, ""):
                                continue
                            deadline = time.monotonic() + 2
                            while time.monotonic() < deadline and not marker.exists():
                                time.sleep(0.02)
                            if marker.exists() != (mode == "parent-keep"):
                                failures.append(f"parent exit lifetime mismatch: {mode}")
                        finally:
                            marker.unlink(missing_ok=True)
        report(before, "public Windows Process", "native execution" if os.name == "nt" else "source/cross-build only")
    native_sections = ROOT / "tests/platform-io/windows-crash-native-sections.jai"
    if args.cross_check:
        attempt([compiler, native_sections, "-os", "windows", "-cpu", "x64", "-check", "-no_dce"], env)
    if cpu == "x64" and ((os.name == "nt" and (args.native or args.windows_crash)) or args.windows_build == "x64"):
        before = len(failures)
        target = [] if os.name == "nt" else ["-os", "windows", "-cpu", "x64"]
        for optimization in ["-O0", "-O2"]:
            executable = output / ("windows-crash-native-sections" + optimization + ".exe")
            if attempt([compiler, native_sections, *target, optimization, "-exe", executable], env) and os.name == "nt":
                attempt([executable], env, "ok\n")
        report(before, "native compiled-symbol PE sections", "execution" if os.name == "nt" else "cross-build only")
    if args.windows_crash:
        if os.name == "nt":
            # Native SDK lifecycle assertions execute only on an actual ARM64 OS.
            library = args.stdlib.resolve() / "internal/runtime/crash-native/platform_io_windows_crash.lib"
            for optimization in ["-O0", "-O2"]:
                executable = output / (cpu + "-crash-lifecycle" + optimization + ".exe")
                triple = "aarch64" if cpu == "arm64" else "x86_64"
                c_source = ROOT / f"tests/platform-io/windows-{cpu}-crash-lifecycle.c"
                before = len(failures)
                if attempt(["clang", f"--target={triple}-pc-windows-msvc", "-std=c11",
                            "-Wall", "-Wextra", "-Werror", optimization,
                            c_source, library, "-ldbghelp", "-o", executable], env):
                    attempt([executable], env, "")
                report(before, "native SDK crash lifecycle", cpu, optimization)
                if cpu == "x64":
                    sdk_exe = output / ("x64-stackwalk-sdk" + optimization + ".exe")
                    if attempt(["clang", "--target=x86_64-pc-windows-msvc", "-std=c11", "-Wall", "-Wextra",
                                "-Werror", optimization, ROOT / "tests/platform-io/windows-x64-stackwalk-sdk.c",
                                "-ldbghelp", "-o", sdk_exe], env):
                        try:
                            result = subprocess.run([str(sdk_exe)], cwd=ROOT, env=env, capture_output=True, timeout=60)
                        except (OSError, subprocess.SubprocessError) as error:
                            failures.append(f"native SDK walk execution failed: {error}")
                            print("FAILED", failures[-1], file=sys.stderr, flush=True)
                            continue
                        save_result(output / ("sdk-walk" + optimization), result)
                        print("native SDK walk", optimization, repr(result.stdout), repr(result.stderr), flush=True)
                        if result.returncode or result.stderr:
                            failures.append(f"native SDK walk failed: {result}")
        bootstrap = ROOT / "stdlib/Runtime_Support/tests/windows-crash-bootstrap.jai"
        for target_cpu in ["x64", "arm64"]:
            attempt([compiler, bootstrap, "-os", "windows", "-cpu", target_cpu, "-check", "-no_dce"], env)
        diagnostic([compiler, ROOT / "tests/platform-io/pending/windows-arm64-crash-handler.jai",
                           "-os", "windows", "-cpu", "arm64", "-check", "-no_dce"], env,
                          "the reviewed Windows crash handler supports x64 only")
        for optimized in [False, True]:
            for policy in ["off", "on", "default"]:
                children = ["startup-arguments-contract", "windows-fault", "windows-symbol-fault"]
                if not optimized:
                    children.append("windows-nullguard")
                for child_name in children:
                    enabled = policy != "off"
                    builder = ROOT / "tests/platform-io" / (
                        "windows-default-startup-build.jai" if policy == "default" else "windows-startup-build.jai")
                    label = f"{cpu}-O{2 if optimized else 0}-{policy}-{child_name}"
                    crash_output = output / "windows-crash" / label
                    crash_output.mkdir(parents=True, exist_ok=True)
                    definitions = (f'OPTIMIZED :: {str(optimized).lower()}; BACKTRACE_ENABLED :: {str(enabled).lower()}; '
                                   f'OUTPUT_PATH :: "{crash_output.as_posix()}"; '
                                   f'CHILD_SOURCE :: "{(ROOT / "tests/platform-io" / (child_name + ".jai")).as_posix()}";')
                    command = [compiler, builder, "-os", "windows", "-cpu", cpu, "-add", definitions,
                               "-exe", crash_output / "builder.exe"]
                    if args.cross_check:
                        other_cpu = "arm64" if cpu == "x64" else "x64"
                        workspace = "default Windows startup" if policy == "default" else "Windows entry contract"
                        # The CLI deliberately warns that a checked metaprogram's
                        # child executable is unwritten. Require exactly that
                        # diagnostic; every other warning/error still fails.
                        expected_warning = (
                            f"warning: `jaic -check` does not write {crash_output.relative_to(ROOT).as_posix()}/startup-crash "
                            f"(workspace `{workspace}` asks for an executable)\n"
                            f"help: `jaic {builder.as_posix()}` (or `jaic {builder.as_posix()} -interp`) writes it\n").encode()
                        attempt([compiler, builder, "-os", "windows", "-cpu", other_cpu,
                                 "-check", "-no_dce", "-add", definitions], env, "", expected_warning)
                    if not attempt(command, env):
                        continue
                    if enabled:
                        shutil.copy2(args.stdlib.resolve() / "internal/runtime/crash-native/platform_io_windows_crash.dll",
                                     crash_output / "platform_io_windows_crash.dll")
                    if child_name in {"windows-fault", "windows-symbol-fault"}:
                        shutil.copy2(fault_sdk_dll, crash_output / fault_sdk_dll.name)
                    if os.name == "nt":
                        if child_name == "startup-arguments-contract":
                            before = len(failures)
                            attempt([crash_output / "startup-crash.exe"], env, "ok\n")
                            report(before, "native Windows startup option", label)
                            continue
                        try:
                            result = subprocess.run([str(crash_output / "startup-crash.exe")], cwd=ROOT, env=env,
                                                    capture_output=True, timeout=60)
                        except (OSError, subprocess.SubprocessError) as error:
                            failures.append(f"Windows fault execution failed ({label}): {error}")
                            print("FAILED", failures[-1], file=sys.stderr, flush=True)
                            continue
                        symbols = ("fault_leaf", "fault_middle") if child_name == "windows-symbol-fault" else ()
                        failed = (nullguard_failures(result) if child_name == "windows-nullguard"
                                  else fault_failures(result, enabled, symbols))
                        save_result(crash_output / "actual-result", result, failed)
                        if failed:
                            failures.append(f"Windows exception diagnostic mismatch ({label}): {failed}; {result}")
                            print("FAILED native Windows exception option", label, file=sys.stderr, flush=True)
                        else:
                            print("passed native Windows exception option", label, flush=True)
                    else:
                        print("cross-built Windows exception option; native execution outstanding", label, flush=True)
    if failures:
        raise RuntimeError(f"{len(failures)} platform I/O checks failed; all independent checks above were retained\n" + "\n".join(failures))


if __name__ == "__main__":
    main()
