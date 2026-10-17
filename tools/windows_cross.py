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
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
import stdlib_runtime  # noqa: E402

# Self-checking programs (each prints "ok") exercised on Windows besides the corpus.
WINDOWS_PROGRAMS = [
    "tests/compiler/native/windows/runtime.jai",
    "tests/compiler/native/windows/variadic.jai",
    "tests/compiler/regressions/struct-literal-overrides-default-string.jai",
    "tests/compiler/regressions/array-literal-view-lifetime.jai",
    "tests/compiler/regressions/c-variadic-foreign-calls.jai",
    "tests/compiler/regressions/member-align-lowers-alignment.jai",
    "tests/compiler/regressions/over-aligned-allocation.jai",
]

# These exact fixtures execute target File/Process operations during ordinary
# #run. The reviewed cross-execution contract does not remap arbitrary Win32
# calls to the build host. They remain required in native Windows build jobs.
WINDOWS_BUILD_HOST_CASES = frozenset(
    {
        "Basic/tests/corpus-api/file-compile-time-qualified",
        "Program_Print/tests/plugin-file-output",
        "compiler/rules-compile-time-execution",
    }
)


# Programs linked against fixture or developer-tool libraries that only the Windows host jobs build
# (tools/build_ci_native_bindings.py, tools/build_developer_*.py); the cross jobs build no such
# library, so these stay required in the native Windows jobs.
WINDOWS_HOST_LIBRARY_CASES = frozenset(
    {
        "SDL/tests/sdl-layout",
        "SDL/tests/sdl-rwops",
        "SDL/tests/sdl-wm",
        "native/sdl2-geometry-timer",
        "Debug/tests/ignore-ownership",
        "Bindings_Generator/tests/generator-macro-shared-state",
        "Bindings_Generator/tests/generator-run-macro-print",
        "Bindings_Generator/tests/generator-run-toplevel",
    }
)


def fixture_stale_contract_reason(case):
    # Exact legacy source oracle; a corrected fixture must become required.
    if (
        case["id"] == "compiler/lang-asm"
        and hashlib.sha256(case["path"].read_bytes()).hexdigest()
        == "e64cb05be3757f5e236b1536dc61a1cf537161d25c183abff42feb61df0d220b"
    ):
        return "pinned legacy fixture asserts CPUID EBX == 0/no features; reviewed architectural CPUID returns real leaf/subleaf data; correct fixture against native SDK, never fabricate zero"
    return None


def fixture_disabled_reason(case):
    return stdlib_runtime.disabled_reason(case)


def fixture_unavailable_reason(case, platform, build_host):
    # The catalogue knows `windows-arm64`; the `-mingw` suffix only names the cross toolchain.
    reason = stdlib_runtime.unavailable_reason(case, "native", "-".join(platform.split("-")[:2]))
    if reason:
        return reason
    if (
        case["id"] in WINDOWS_BUILD_HOST_CASES
        and platform.startswith("windows-")
        and not build_host.startswith("windows-")
    ):
        return "target Win32 File/Process calls during ordinary #run require a Windows build host; native Windows coverage remains required"
    if (
        case["id"] in WINDOWS_HOST_LIBRARY_CASES
        and platform.startswith("windows-")
        and not build_host.startswith("windows-")
    ):
        return "links native fixture libraries that only the Windows host jobs build; native Windows coverage remains required"
    if case["id"] == "compiler/lang-asm" and platform.split("-")[1] != "x64":
        return "fixture contains architectural x86 CPUID/PAUSE/fences; requires x64, not an ARM64 instruction-emulation claim"
    if case["id"] == "ImGui/tests/headless-frames":
        profile = json.loads((ROOT / "tools/native-libs.json").read_text())[
            "libraries"
        ]["imgui"]
        if platform in profile.get("except", []):
            return "pinned imgui native profile excludes this MinGW target (Windows bindings use MSVC C++ ABI); native Windows MSVC fixture remains required"
    return None


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
            yield (
                "stdlib-" + case["id"].replace("/", "-"),
                case["path"],
                case.get("exit_code", 0),
                case.get("stdout"),
                True,
            )


# tests/compiler/native/c-structs-by-value: C structs by value both ways across the C ABI, against C
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
        steps = [
            [
                f"{mingw_prefix(args)}-gcc",
                "-shared",
                "-O1",
                "-o",
                "libstructs.dll",
                "structs.c",
            ]
        ]
    for step in steps:
        result = subprocess.run(step, cwd=work, capture_output=True, text=True)
        if result.returncode != 0:
            return f"{' '.join(step)}: {result.stderr.strip()}"
    return None


def build_socket_sdk_probe(args, output):
    """Independent real SDK control, never a replacement for the TLS program."""
    compiler = "clang" if args.host else f"{mingw_prefix(args)}-gcc"
    command = [
        compiler,
        "-Wall",
        "-Wextra",
        "-Werror",
        "-std=c11",
        str(ROOT / "tests/compiler/native/windows/inet-pton-sdk.c"),
        "-lws2_32",
        "-o",
        str(output),
    ]
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=300)
    except (OSError, subprocess.TimeoutExpired) as error:
        return f"SDK probe compiler {compiler}: {error}"
    if result.returncode != 0:
        return f"SDK probe exit {result.returncode}: {result.stdout}{result.stderr}"
    if not output.is_file():
        return "SDK probe compiler returned success without producing the required executable"
    return None


def build_one(args, source, output):
    target = [] if args.host else ["-os", "windows", "-cpu", args.cpu]
    try:
        result = subprocess.run(
            [
                str(pathlib.Path(args.jaic).resolve()),
                str(source),
                *target,
                "-exe",
                str(output),
            ],
            cwd=source.parent,
            capture_output=True,
            text=True,
            timeout=300,
        )
        if result.returncode != 0 and (
            "no exported 'main'" in result.stderr
            or "has no `main` procedure" in result.stderr
        ):
            checked = subprocess.run(
                [
                    str(pathlib.Path(args.jaic).resolve()),
                    str(source),
                    *target,
                    "-check",
                ],
                cwd=source.parent,
                capture_output=True,
                text=True,
                timeout=300,
            )
            if checked.returncode == 0:
                return COMPILE_TIME_ONLY
            return f"source validation exit {checked.returncode}: {checked.stdout}{checked.stderr}"
    except subprocess.TimeoutExpired:
        return "build timed out"
    # A program without `main`, or whose metaprogram writes no output, did its work at compile time.
    if result.returncode != 0:
        return f"exit {result.returncode}: {result.stdout}{result.stderr}".strip()
    if not output.with_name(output.name + ".exe").exists():
        return COMPILE_TIME_ONLY
    return None


COMPILE_TIME_ONLY = "compile-time only: no executable to run"


CROSS_DLLS = []


def ensure_cross_host_bindings(host):
    """Prepare all accepted ABI8 host profiles before compile-time foreign calls.

    Host shared providers and target PE libraries have distinct search paths.
    An explicit caller build directory is required; no local temp fallback.
    """
    import build_ci_native_bindings

    _, profiles, _ = build_ci_native_bindings.selected_libraries()
    required = {"binding_callback_context", "rpmalloc_callback_probe"}
    required.update(
        profile[kind] for profile in profiles for kind in ("sdk", "callbacks")
    )
    suffix = ".dylib" if host.startswith("macos-") else ".so"
    directories = [
        pathlib.Path(path)
        for path in os.environ.get("JAIC_NATIVE_LIBS", "").split(os.pathsep)
        if path
    ]
    for directory in directories:
        if verified_cross_host_bindings(directory, profiles, required, suffix):
            os.environ["JAIC_NATIVE_LIBS"] = os.pathsep.join(
                [
                    str(directory),
                    *[str(path) for path in directories if path != directory],
                ]
            )
            return
    assigned = os.environ.get("CARGO_TARGET_DIR") or os.environ.get("RUNNER_TEMP")
    if not assigned:
        raise ValueError(
            "assign CARGO_TARGET_DIR or CI RUNNER_TEMP for real cross-host providers"
        )
    base = pathlib.Path(assigned).resolve()
    base.mkdir(parents=True, exist_ok=True)
    work = pathlib.Path(tempfile.mkdtemp(prefix="windows-cross-host-", dir=base))
    output = work / "native"
    subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools/build_ci_native_bindings.py"),
            "--output",
            str(output),
        ],
        check=True,
    )
    receipt = json.loads((output / "ci-provider-receipt.json").read_text())
    if receipt["native_abi"] != 8 or len(receipt["profiles"]) != 23:
        raise ValueError("cross-host provider build did not verify all23 ABI8 profiles")
    for name, digest in receipt["assets"].items():
        if (
            pathlib.Path(name).name != name
            or hashlib.sha256((output / name).read_bytes()).hexdigest() != digest
        ):
            raise ValueError(
                "cross-host provider artifact differs from its build receipt"
            )
    if not all((output / ("lib" + name + suffix)).is_file() for name in required):
        raise ValueError("cross-host callback provider closure is incomplete")
    os.environ["JAIC_NATIVE_LIBS"] = os.pathsep.join(
        [str(output), *map(str, directories)]
    )


def verified_cross_host_bindings(directory, profiles, required, suffix):
    """A complete filename set alone cannot qualify cached ABI7/stale providers."""
    record = directory / "ci-provider-receipt.json"
    if not record.is_file():
        return False
    receipt = json.loads(record.read_text())
    if receipt.get("native_abi") != 8:
        return False
    keys = [
        "statistics",
        "raw",
        "public_heaps",
        "global_cache",
        "thread_cache",
        "assertions",
        "validation",
        "adaptive",
    ]
    expected = {
        p["sdk"]: (8 << 16) | sum(int(bool(p[k])) << i for i, k in enumerate(keys))
        for p in profiles
    }
    rows = receipt.get("profiles", [])
    if (
        len(rows) != len(expected)
        or {p["profile"]: p["actual_flags"] for p in rows} != expected
    ):
        return False
    inputs = receipt.get("source_inputs", {})
    critical = {
        "tools/build_ci_native_bindings.py",
        "tools/native_bindings/build_dependencies.py",
        "tools/native_bindings/build_rpmalloc.py",
        "tools/native_bindings/prepare_rpmalloc.py",
        "tools/native_bindings/rpmalloc_profiles.py",
        "tools/native_bindings/config.py",
        "tools/native_bindings/dependencies.json",
        "stdlib/internal/bindings/rpmalloc-sdk/provenance.json",
        "stdlib/internal/bindings/rpmalloc-heaps/bridge.c",
        "stdlib/internal/bindings/rpmalloc-callbacks/bridge.c",
        "stdlib/internal/bindings/callback-context/bridge.c",
    }
    for suffix in ("*.c", "*.h", "*.inc", "*.patch"):
        critical.update(
            str(path.relative_to(ROOT))
            for path in (ROOT / "stdlib/internal/bindings/rpmalloc-heaps").glob(suffix)
        )
    if not critical <= inputs.keys():
        return False
    for name, digest in inputs.items():
        path = pathlib.Path(name)
        if (
            path.is_absolute()
            or ".." in path.parts
            or not (
                name == "tools/build_ci_native_bindings.py"
                or name.startswith(
                    (
                        "tools/native_bindings/",
                        "stdlib/Bindings_Generator/tests/driven/sdk/",
                        "stdlib/internal/bindings/",
                    )
                )
            )
        ):
            raise ValueError("invalid cross-host provider source receipt path")
        if (
            not (ROOT / path).is_file()
            or hashlib.sha256((ROOT / path).read_bytes()).hexdigest() != digest
        ):
            return False
    assets = receipt.get("assets", {})
    if any("lib" + name + suffix not in assets for name in required):
        return False
    for name, digest in assets.items():
        if pathlib.Path(name).name != name:
            raise ValueError("invalid cross-host provider artifact receipt path")
        if (
            not (directory / name).is_file()
            or hashlib.sha256((directory / name).read_bytes()).hexdigest() != digest
        ):
            return False
    return True


def use_cross_libs(args):
    """Cross-build the third-party C libraries the stdlib tests link (stb_*, FreeType, lz4...)
    with the MinGW toolchain, and point `jaic` at them (JAIC_CROSS_LIBS). They are static
    archives, so the executables need no DLL beside them, except SDL2, which comes as the
    official release's import library and DLL (copied beside the executables, see `build`)."""
    # Compile-time code executes in the host compiler process; its real native
    # providers must not be replaced with target PE archives/DLLs.
    stdlib_runtime.use_native_libs(stdlib_runtime.host_platform())
    ensure_cross_host_bindings(stdlib_runtime.host_platform())
    platform = target_platform(args)
    subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools/build_native_libs.py"),
            "--platform",
            platform,
        ],
        check=True,
    )
    import build_native_libs

    target_libraries = build_native_libs.output_dir(platform)
    subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools/build_windows_binding_providers.py"),
            "--platform",
            platform,
            "--output",
            str(target_libraries),
        ],
        check=True,
    )
    os.environ["JAIC_CROSS_LIBS"] = str(target_libraries)
    # SDL2 is linked through its import library, so its DLL goes beside the executables.
    CROSS_DLLS.extend(build_native_libs.output_dir(platform).glob("SDL2.dll"))


def build(args):
    out = pathlib.Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    if args.stdlib and not args.host:
        use_cross_libs(args)
    expected, failures, skipped = {}, [], []
    records = []
    platform = target_platform(args)
    build_host = stdlib_runtime.host_platform()
    catalogue = []
    negative_checks = 0
    if args.stdlib:
        catalogue = stdlib_runtime.validated_entries()
        target = [] if args.host else ["-os", "windows", "-cpu", args.cpu]
        for case in catalogue:
            if case.get("mode") != "check-fail" and "error_contains" not in case:
                continue
            reason = fixture_unavailable_reason(case, platform, build_host)
            if reason:
                records.append(
                    {
                        "id": case["id"],
                        "status": "platform_unavailable",
                        "passed": None,
                        "reason": reason,
                    }
                )
                continue
            flags = (
                ["-check"]
                if case.get("mode") == "check-fail"
                else ["-interp", "-no_workspace_output"]
            )
            code, stdout, stderr = stdlib_runtime.run_command(
                [
                    str(pathlib.Path(args.jaic).resolve()),
                    str(case["path"]),
                    *target,
                    *flags,
                ],
                case["path"].parent,
                300,
                dict(os.environ),
            )
            valid = stdlib_runtime.contract_result(case, code, stdout, stderr)
            output = stdout + stderr
            if case.get("positive_baseline"):
                baseline = next(
                    c
                    for c in catalogue
                    if c["id"] == case["positive_baseline_id"]
                )
                bcode, bout, berr = stdlib_runtime.run_command(
                    [
                        str(pathlib.Path(args.jaic).resolve()),
                        str(baseline["path"]),
                        *target,
                        "-check",
                    ],
                    baseline["path"].parent,
                    300,
                    dict(os.environ),
                )
                valid = valid and bcode == 0
                if bcode != 0:
                    status = "timeout" if bcode is None else f"exit {bcode}"
                    output += f"\npositive baseline failed ({status}):\n" + bout + berr
            if valid:
                negative_checks += 1
                records.append(
                    {
                        "id": case["id"],
                        "status": "expected_negative_check",
                        "passed": None,
                        "validated": True,
                    }
                )
            else:
                status = "timeout" if code is None else f"exit {code}"
                failures.append(
                    case["id"]
                    + f": expected precise rejection did not match ({status}):\n"
                    + output
                )
                records.append(
                    {
                        "id": case["id"],
                        "status": "failed",
                        "passed": None,
                        "reason": failures[-1],
                    }
                )
        print(
            f"{negative_checks} expected negative target checks; no native execution claim"
        )
    all_cases = list(cases(args.stdlib, target_platform(args)))
    metadata = {case["path"]: case for case in catalogue}
    supported = []
    for item in all_cases:
        case_id, source = item[:2]
        case = metadata.get(source, {"id": case_id, "family": "corpus"})
        reason = fixture_unavailable_reason(case, platform, build_host)
        if reason:
            records.append(
                {
                    "id": case_id,
                    "status": "platform_unavailable",
                    "passed": None,
                    "reason": reason,
                }
            )
        elif rejected := fixture_disabled_reason(case):
            records.append(
                {
                    "id": case_id,
                    "status": "disabled",
                    "passed": None,
                    "reason": rejected,
                }
            )
        elif invalid := fixture_stale_contract_reason(case):
            records.append(
                {
                    "id": case_id,
                    "status": "stale_fixture_contract",
                    "passed": None,
                    "reason": invalid,
                }
            )
        else:
            supported.append(item)
    all_cases = supported
    if any(
        metadata.get(item[1], {}).get("id") == "Extensions/Tls/tests/windows-target"
        for item in all_cases
    ):
        probe_id = "socket-inet-pton-sdk"
        error = build_socket_sdk_probe(args, out / (probe_id + ".exe"))
        if error:
            failures.append(f"{probe_id}: {error}")
            records.append(
                {"id": probe_id, "status": "failed", "passed": None, "reason": error}
            )
        else:
            expected[probe_id] = {"exit_code": 0, "stdout": ""}
            records.append({"id": probe_id, "status": "built", "passed": None})
    # Builds are independent; run several at once (each is mostly single-threaded).
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count() or 2) as pool:
        errors = list(
            pool.map(lambda case: build_one(args, case[1], out / case[0]), all_cases)
        )
    for (case_id, source, exit_code, stdout, required), error in zip(all_cases, errors):
        if error == COMPILE_TIME_ONLY:
            skipped.append(f"{case_id}: {error}")
            records.append(
                {
                    "id": case_id,
                    "status": "compile_time_only",
                    "passed": None,
                    "reason": error,
                }
            )
            continue
        if error:
            (failures if required else skipped).append(f"{case_id}: {error[-3000:]}")
            records.append(
                {"id": case_id, "status": "failed", "passed": None, "reason": error}
            )
            continue
        # Programs run from their source directory (relative to the checkout), as the sweep does.
        cwd = source.parent.relative_to(ROOT).as_posix()
        status = (
            {"error": exit_code}
            if isinstance(exit_code, str)
            else {"exit_code": exit_code}
        )
        expected[case_id] = {**status, "stdout": stdout, "cwd": cwd}
        if source in metadata:
            case = metadata[source]
            expected[case_id]["contract"] = {
                key: case[key]
                for key in ("stdout", "stderr", "error_contains", "exit_code")
                if key in case
            }
            expected[case_id]["fixture"] = {
                key: case[key] for key in ("id", "arguments", "input") if key in case
            }
        records.append({"id": case_id, "status": "built", "passed": None})
    work = out / "c-structs-src"
    shutil.copytree(ROOT / "tests/compiler/native/c-structs-by-value", work, dirs_exist_ok=True)
    error = c_structs_library(args, work)
    if error:
        failures.append(f"c-structs library: {error}")
        records.append(
            {
                "id": "c-structs-library",
                "status": "failed",
                "passed": None,
                "reason": error,
            }
        )
    else:
        if (work / "libstructs.dll").exists():
            shutil.copy(work / "libstructs.dll", out / "libstructs.dll")
        for name, stdout in C_STRUCTS.items():
            case_id = f"c-structs-{name}"
            error = build_one(args, work / f"{name}.jai", out / case_id)
            if error:
                failures.append(f"{case_id}: {error}")
                records.append(
                    {"id": case_id, "status": "failed", "passed": None, "reason": error}
                )
                continue
            expected[case_id] = {"exit_code": 0, "stdout": stdout}
            records.append({"id": case_id, "status": "built", "passed": None})
    for dll in CROSS_DLLS:
        shutil.copy2(dll, out / dll.name)
    (out / "expected.json").write_text(json.dumps(expected, indent=2))
    counts = {
        status: sum(r["status"] == status for r in records)
        for status in (
            "built",
            "failed",
            "platform_unavailable",
            "compile_time_only",
            "expected_negative_check",
            "stale_fixture_contract",
            "disabled",
        )
    }
    (out / "build-report.json").write_text(
        json.dumps(
            {
                "schema": 1,
                "target": platform,
                "build_host": build_host,
                "native_execution": False,
                "counts": counts,
                "results": records,
            },
            indent=2,
        )
        + "\n"
    )
    for record in records:
        if record["status"] in {
            "platform_unavailable",
            "stale_fixture_contract",
            "disabled",
        }:
            print(f"{record['status'].upper()} {record['id']}: {record['reason']}")
    print(
        "build accounting: "
        + ", ".join(f"{count} {status}" for status, count in counts.items())
    )
    print(
        f"built {len(expected)} programs into {out}; {len(skipped)} compile-time only or optional ones do not build"
    )
    for line in skipped:
        print(f"not built: {line}")
    for failure in failures:
        print(f"BUILD FAILED {failure}")
    return 1 if failures else 0


def run_environment():
    """The environment fixtures see: a Windows runner always has Arial, so fixtures that need a
    caller-owned font (Simp/tests/glyph-atlas) get it unless the caller named another."""
    env = dict(os.environ)
    if not env.get("JAIC_TEST_FONT_PATH"):
        windir = pathlib.Path(env.get("WINDIR") or env.get("SystemRoot") or r"C:\Windows")
        font = windir / "Fonts" / "arial.ttf"
        if font.is_file():
            env["JAIC_TEST_FONT_PATH"] = str(font)
    return env


def run(args):
    directory = pathlib.Path(args.dir).resolve()
    environment = run_environment()
    expected = json.loads((directory / "expected.json").read_text())
    failures = []
    unrunnable = []
    for case_id, want in sorted(expected.items()):
        exe = directory / f"{case_id}.exe"
        cwd = ROOT / want.get("cwd", ".")
        try:
            with tempfile.TemporaryDirectory(
                prefix="windows-fixture-", dir=directory
            ) as scratch:
                arguments = stdlib_runtime.program_arguments(
                    want.get("fixture", {}), scratch, environment
                )
                result = subprocess.run(
                    [str(exe), *arguments],
                    cwd=cwd if cwd.is_dir() else directory,
                    capture_output=True,
                    timeout=60,
                )
        except ValueError as error:
            failures.append(f"{case_id}: fixture input: {error}")
            continue
        except subprocess.TimeoutExpired:
            failures.append(f"{case_id}: timed out")
            continue
        stdout = result.stdout.decode("utf-8", "replace").replace("\r\n", "\n")
        stderr = result.stderr.decode("utf-8", "replace").replace("\r\n", "\n")
        stdout_ok = want["stdout"] is None or stdout == want["stdout"]
        contract_ok = "contract" not in want or stdlib_runtime.contract_result(
            want["contract"], result.returncode, stdout, stderr
        )
        if "error" in want:
            # A runtime error: any failing status, with the check's message on stderr.
            status_ok = result.returncode != 0 and want["error"] in stderr
            wanted = f"a runtime error, {want['error']!r}"
        else:
            status_ok = result.returncode == want["exit_code"]
            wanted = want["exit_code"]
        if (
            getattr(args, "without_opengl", False)
            and not status_ok
            and stderr.startswith("WGL creation failed")
        ):
            # This runner has only GDI's OpenGL 1.1 and no driver for the CPU (Mesa ships none
            # for arm64): the program cannot create its context here, whatever it does after.
            unrunnable.append(case_id)
            continue
        if not status_ok or not stdout_ok or not contract_ok:
            failures.append(
                f"{case_id}: exit {result.returncode} (want {wanted}), "
                f"stdout {stdout!r} (want {want['stdout']!r}), "
                f"stderr {stderr[-2000:]!r}"
            )
    print(
        f"{len(expected) - len(failures) - len(unrunnable)} of {len(expected) - len(unrunnable)} "
        "programs match"
    )
    for case_id in unrunnable:
        print(f"SKIPPED {case_id}: no OpenGL 3.3 driver on this runner")
    for failure in failures:
        print(f"FAIL {failure}")
    return 1 if failures else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)
    b = sub.add_parser("build")
    b.add_argument("--jaic", required=True)
    b.add_argument("--out", required=True)
    b.add_argument(
        "--keep-going",
        action="store_true",
        help="retained spelling: all cases are attempted; required failures remain fatal",
    )
    b.add_argument(
        "--host",
        action="store_true",
        help="build for the host (on Windows) instead of -os windows",
    )
    b.add_argument(
        "--cpu",
        choices=["x64", "arm64"],
        default="x64",
        help="target CPU when cross-building (arm64 needs llvm-mingw on PATH)",
    )
    b.add_argument(
        "--stdlib", action="store_true", help="also the stdlib runtime tests"
    )
    r = sub.add_parser("run")
    r.add_argument("--dir", required=True)
    r.add_argument(
        "--without-opengl",
        action="store_true",
        help="the runner has no OpenGL 3.3 driver: programs that cannot create a context are skipped",
    )
    args = parser.parse_args()
    if (
        args.command == "run" or args.host
    ) and not stdlib_runtime.host_platform().startswith("windows-"):
        parser.error(
            "native Windows execution and --host builds require an actual Windows host; use build without --host for cross compilation"
        )
    return build(args) if args.command == "build" else run(args)


if __name__ == "__main__":
    sys.exit(main())
