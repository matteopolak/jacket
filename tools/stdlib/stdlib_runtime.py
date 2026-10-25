#!/usr/bin/env python3
"""Run every stdlib module test and compiler regression through one catalogue.

No inherited compatibility skip/coverage baseline is consulted. Native-only SDK
and graphics cases are reported unavailable in WASM modes, not counted passed.
Tool-driven integration suites (native SDK probes, generator workflows) are separate gates.

`--modes check --platform windows-arm64` type-checks every case that applies to that target
(`-check -no_dce -os windows -cpu arm64`; `--platform wasm-wasm64` for WASI) without running
anything: the cross-target source check. The other modes run on this host only.
"""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import json
import threading
import os
import shutil
import subprocess
import sys
import tempfile
import time
from stdlib.stdlib_test_catalog import (
    validated_entries,
    unavailable_reason,
    disabled_reason,
    case_for_mode,
)
from concurrent.futures import ThreadPoolExecutor, as_completed
from repo_output import host_platform
from pathlib import Path
from toolkit.cli import add_jaic_argument
from toolkit.hashing import sha256_file
from toolkit.repo import ROOT

MODES = ["interp", "native", "wasm-interp", "wasm-native", "check"]
COVERAGE_LOCK = threading.Lock()
CLIPBOARD_LOCK = threading.Lock()
# Only one application at a time is active on a Mac, and a program's windows are key (and its
# focus events arrive) only while it is: GUI programs would otherwise take focus from each other.
WINDOW_LEASE = threading.Lock()


def shard(text):
    """`--shard K/N`: the K-th (from 1) of N parts; argparse type, shared with jaic_sweep.py."""
    try:
        index, count = (int(part) for part in text.split("/"))
    except ValueError:
        raise argparse.ArgumentTypeError(f"expected K/N, got {text!r}")
    if not 1 <= index <= count:
        raise argparse.ArgumentTypeError(f"shard {text} is out of range")
    return index, count


def take_shard(items, part):
    """Every N-th item from the K-th: neighbouring cases (often a module's slow tests) spread
    across the parts, and the split is the same on every run."""
    index, count = part
    return items[index - 1 :: count]


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
    # An interpreted run prints the compiler's warnings on the program's stderr; a compiled
    # program's own stderr stays empty (its build output is checked where it is built).
    if "warning_contains" in case and not native:
        error_ok = case["warning_contains"] in error
    else:
        error_ok = error == case.get("stderr", "")
    if "stdout" in case:
        return code == case.get("exit_code", 0) and output == case["stdout"] and error_ok
    if "stdout_one_of" in case:
        return code == case.get("exit_code", 0) and output in case["stdout_one_of"] and error_ok
    return code == 0


def use_native_libs(plat):
    """Point jaic at the third-party C libraries some tests link (stb_vorbis, rpmalloc, FreeType
    on Windows...), building any that are missing for `plat`, as tools/differential/jaic_sweep.py does."""
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
    from native import build_native_libs

    if build_native_libs.missing(plat):
        extra = ["--platform", plat] if plat else []
        subprocess.run(
            [sys.executable, str(ROOT / "tools/native/build_native_libs.py"), *extra],
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
            note = describe_stuck_process(proc.pid)
            proc.kill()
            proc.wait()
            code = None
            error.seek(0, 2)
            error.write(note.encode())
        out.seek(0)
        error.seek(0)
        return (
            code,
            out.read().decode("utf-8", "replace").replace("\r\n", "\n"),
            error.read().decode("utf-8", "replace").replace("\r\n", "\n"),
        )


def describe_stuck_process(pid):
    """What a timed-out program was doing when it was killed (Linux): whether it burned CPU or
    waited, and where, so a hang and a slow run can be told apart from the log."""
    lines = []
    try:
        for name in ("status", "wchan", "stat"):
            text = Path(f"/proc/{pid}/{name}").read_text(errors="replace")
            if name == "status":
                text = "".join(
                    line
                    for line in text.splitlines(True)
                    if line.split(":")[0] in ("State", "Threads", "VmRSS")
                )
            lines.append(f"{name}: {text.strip()[:400]}")
        for task in sorted(Path(f"/proc/{pid}/task").iterdir())[:8]:
            lines.append(
                f"task {task.name}: wchan {(task / 'wchan').read_text().strip()} stat {(task / 'stat').read_text().split(')')[-1].split()[0]}"
            )
    except OSError:
        return ""
    return "\n[timed out] " + "\n[timed out] ".join(lines) + "\n"


def write_ico_fixtures(directory, env):
    """The bitmaps the Ico_File filename test reads, and the 16x16 gradient resized by the real
    stb_image_resize library (the reference its encoder output is compared against)."""
    import ctypes
    import struct

    # Public BMP file format: BITMAPFILEHEADER + BITMAPINFOHEADER, 16x16 BGR.
    pixels = bytes((30, 20, 10)) * (16 * 16)
    header = struct.pack("<2sIHHI", b"BM", 54 + len(pixels), 0, 0, 54)
    dib = struct.pack("<IiiHHIIiiII", 40, 16, 16, 1, 24, 0, len(pixels), 0, 0, 0, 0)
    (directory / "bitmap-\u00fc.bmp").write_bytes(header + dib + pixels)
    gradient = bytes(
        value
        for row in range(16)
        for column in range(16)
        for value in (row * 10, column * 10, 60 + row)
    )
    (directory / "gradient.bmp").write_bytes(header + dib + gradient)
    suffix = (
        ".dll" if os.name == "nt" else (".dylib" if sys.platform == "darwin" else ".so")
    )
    library = next(
        (
            Path(folder) / ("libstb_image_resize" + suffix)
            for folder in env.get("JAIC_NATIVE_LIBS", "").split(os.pathsep)
            if folder and (Path(folder) / ("libstb_image_resize" + suffix)).is_file()
        ),
        None,
    )
    if library is None:
        raise ValueError(
            "the stb_image_resize library is missing from JAIC_NATIVE_LIBS"
        )
    resize = ctypes.CDLL(str(library)).stbir_resize_uint8_srgb
    resize.argtypes = [
        ctypes.c_void_p,
        ctypes.c_int,
        ctypes.c_int,
        ctypes.c_int,
    ] * 2 + [ctypes.c_int] * 3
    resize.restype = ctypes.c_int
    bgra = bytes(
        value
        for row in range(16)
        for column in range(16)
        for value in (row * 10, column * 10, 60 + row, 255)
    )
    source = ctypes.create_string_buffer(bgra)
    destination = ctypes.create_string_buffer(16 * 16 * 4)
    if resize(source, 16, 16, 64, destination, 16, 16, 64, 4, 3, 0) != 1:
        raise ValueError("stb_image_resize failed to build the gradient fixture")
    (directory / "gradient-resized.bgra").write_bytes(destination.raw)


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
    if kind not in {"scratch-file", "scratch-directory", "ico-fixtures"}:
        raise ValueError("unknown fixture input: " + kind)
    directory = Path(scratch) / "inputs" / case["id"].replace("/", "-")
    directory.mkdir(parents=True, exist_ok=True)
    if kind == "ico-fixtures":
        write_ico_fixtures(directory, env)
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


def uses_clipboard(case):
    """Clipboard tests share the one system clipboard (on Linux, the Xvfb display's selection),
    so two running at once, such as a test's interp and native runs, overwrite each other."""
    return bool(case.get("clipboard_guard")) or case.get("id", "").startswith(
        "Clipboard/"
    )


def run_unleased(command, cwd, timeout, env, case):
    prefix = runtime_prefix(case, env)
    if not prefix:
        if uses_clipboard(case):
            with CLIPBOARD_LOCK:
                return run_command(command, cwd, timeout, env)
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
            digest = sha256_file(source)
            if key in planned and planned[key][1] != digest:
                raise ValueError("conflicting Windows DLL providers: " + source.name)
            planned.setdefault(key, (source, digest))
    for source, digest in planned.values():
        target = Path(destination) / source.name
        if target.exists() and sha256_file(target) != digest:
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
        env["JAIC_NATIVE_LIBS"] = os.pathsep.join(
            map(
                str,
                windows_library_dirs(
                    filter(None, env["JAIC_NATIVE_LIBS"].split(os.pathsep))
                ),
            )
        )
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
        if (
            metadata
            and "error_contains" in metadata
            and metadata.get("mode") != "fault"
        ):
            return (
                None,
                "expects an error while interpreting; nothing to check statically",
            )
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
        coverage_flags = ["-coverage_file", str(record)] if args.coverage else []
        code, stdout, stderr = run_program(
            [
                args.jaic,
                str(path),
                "-interp",
                *coverage_flags,
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
        warning = (metadata or {}).get("warning_contains")
        if warning and warning not in out:
            return False, f"build did not warn `{warning}`:\n{out}"
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
            metadata or {}, code, stdout, stderr, native=True
        ), out if code is not None else f"timed out after {args.timeout:.0f}s\n{out}"
    finally:
        remove_build(exe.parent)


def remove_build(folder):
    """Delete a build folder, waiting out Windows, which keeps an exe locked for a moment
    after it exits. One still locked after five seconds stays in the temp folder."""
    for _ in range(20):
        try:
            shutil.rmtree(folder)
            return
        except PermissionError:
            if sys.platform != "win32":
                raise
            time.sleep(0.25)


def modes_given_cross(args):
    """Whether this run executes programs, which only the host can do."""
    return any(m.strip() not in ("check", "") for m in args.modes.split(","))


DATA_FIXTURE_BUILDERS = (
    "zip",
    "archive",
    "mpeg",
    "meow",
    "machine",
)  # tools/portable/build_portable.py subcommands


def build_data_fixtures():
    """Write the generated inputs (ZIP, archive, MPEG, Meow and CPUID data) the tests read from
    target/portable-tests; each builder is deterministic and takes no compiler."""
    for builder in DATA_FIXTURE_BUILDERS:
        subprocess.run(
            [
                sys.executable,
                "-B",
                str(ROOT / "tools/portable/build_portable.py"),
                builder,
            ],
            cwd=ROOT,
            check=True,
        )


def main():
    # Test output may hold any Unicode text; the Windows console codec cannot print it.
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    add_jaic_argument(ap, type=str)
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
        "--shard",
        type=shard,
        default=(1, 1),
        metavar="K/N",
        help="run only the K-th of N equal parts of the selection (CI splits the sweep this way)",
    )
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
        help="print the output of every failure in full, and every unavailable or disabled test",
    )
    args = ap.parse_args()
    if (
        modes_given_cross(args)
        and args.platform.removesuffix("-mingw") != host_platform()
    ):
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
    if modes_given_cross(args):
        build_data_fixtures()
    catalogue = validated_entries()
    unknown = set(args.case) - {case["id"] for case in catalogue}
    if unknown:
        ap.error("unknown catalogue IDs: " + ", ".join(sorted(unknown)))
    todo = [
        case
        for case in catalogue
        if args.filter in case["id"] and (not args.case or case["id"] in args.case)
    ]
    todo = take_shard(todo, args.shard)
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
            rejected = disabled_reason(case)
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
                        c for c in catalogue if c["id"] == case["positive_baseline_id"]
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

    failures, unavailable, disabled, counts = (
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
                                        "disabled",
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
                        disabled.append(f"{test} [{mode}]: {rejected}")
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
                            "disabled": rejected,
                            "source_check_only": source_only,
                            "elapsed_seconds": elapsed,
                            "timed_out": detail.startswith("timed out after "),
                            "failure_detail": detail if passed is False else None,
                        }
                    )
                write_report("running")
    finally:
        remove_build(scratch)
    write_report("finished")
    for test, mode, detail in failures:
        lines = detail.strip().splitlines()
        shown = lines if args.verbose else lines[-25:]
        print(f"FAIL {test} [{mode}]")
        for line in shown:
            print(f"    {line}")
    if args.verbose:
        for line in unavailable:
            print("UNAVAILABLE " + line)
        for line in disabled:
            print("DISABLED " + line)
    print(f"\nstdlib runtime tests on {args.platform} ({len(todo)} tests):")
    for mode in modes:
        p, f, u, b, n, r, s = counts[mode]
        print(
            f"  {mode:<12} {p} passed, {f} failed, {u} platform unavailable, {b} compile-time only, {n} expected negative checks, {r} disabled, {s} source-check only"
        )
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
