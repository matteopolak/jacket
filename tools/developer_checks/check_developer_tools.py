#!/usr/bin/env python3
"""Developer tooling checks against this checkout's source-built tools, as unittest.

    python3 tools/developer_checks/check_developer_tools.py                        # everything this host can run
    python3 tools/developer_checks/check_developer_tools.py ManifestChecks.test_check_developer_vtables
    python3 tools/developer_checks/check_developer_tools.py -k dwarf -v

Three test classes: `ManifestChecks` runs the `main()` of each `check_developer_*` / `build_developer_*`
module in CHECKS (those with hosts are skipped elsewhere, never passed), `DrivenCases` runs each driven
case interpreted and native at O0 and O2, and `Inline` holds the rest (format and lint, Windows target
checks, rejections). A test that reads what another leaves under target/ names it in NEEDS and runs it first,
once per process, so any single test or any subset works on its own.

`JACKET_TOOLING_PART=K/N` keeps every N-th test starting at the K-th (CI splits the suite into legs this way
through check_stdlib_suites.py --tooling-part).
"""
import csv
import importlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import unittest
from unittest import mock

from developer_checks import build_developer_bindings
from developer_checks.dev_common import ENV, JAIC, JAIFMT, ROOT, run_checked, sha256_file, cargo_binary, output_dir
from developer_checks.driven_sources import driven, driven_matching

PART_ENV = "JACKET_TOOLING_PART"

# (module, hosts): the checks in the order they used to run; a module's main() is the check. `hosts` are
# platform.system() values; elsewhere the test is skipped.
CHECKS = [
    ("check_developer_bindings", ()),
    ("check_generator_platform", ("Darwin",)),
    ("check_developer_cpp_custom", ()),
    ("check_developer_vtables", ()),
    ("check_developer_cpp_wrappers", ("Darwin", "Linux")),
    ("check_developer_type_emission", ()),
    ("check_developer_value_emission", ()),
    ("check_developer_check", ()),
    ("check_developer_check_plugin", ()),
    ("check_developer_refined_emission", ()),
    ("check_developer_multi_input", ()),
    ("check_developer_pointer_defaults", ()),
    ("check_developer_procedure_results", ()),
    ("check_developer_string_macros", ()),
    ("check_developer_declaration_visitors", ()),
    ("check_developer_integer_range", ()),
    ("check_developer_cpp_scopes", ()),
    ("check_developer_anonymous_types", ()),
    ("check_developer_bitfield_layout", ()),
    ("check_developer_output_format", ("Darwin", "Linux")),
    ("check_developer_variadic", ()),
    ("check_developer_sdk_configuration", ()),
    ("check_developer_native_package", ("Darwin",)),
    ("build_developer_debug", ()),
    ("check_developer_debug_lifecycle", ()),
    ("check_developer_debug_windows", ("Darwin",)),
    ("check_developer_msf", ()),
    ("check_developer_native_abi", ()),
    ("check_developer_dwarf_lines", ()),
    ("check_developer_dwarf_line_wire", ()),
    ("check_developer_dwarf_frames", ()),
    ("check_developer_eh_frames", ()),
    ("check_developer_dwarf_types", ()),
    ("check_developer_codeview_strings", ()),
    ("check_developer_profiler_windows", ("Darwin",)),
]

_done = set()


def once(name, body):
    """Run `body` the first time `name` is asked for in this process."""
    if name not in _done:
        _done.add(name)
        body()


def run_check(module):
    """The check module's main(), once. A check that parses its own command line sees only its name."""
    with mock.patch.object(sys, "argv", [module]):
        once(module, importlib.import_module(f"developer_checks.{module}").main)


def run(command, *, output=None, error=""):
    return run_checked(command, label="Developer tools check", expected=output, stderr=error, timeout=60).stdout


def check_framework_consumer(prefix, emitter):
    if platform.system() == "Darwin":
        # macOS creates AppleDouble metadata files on external filesystems. The
        # framework is an owned generated fixture; remove metadata sidecars so
        # Clang's umbrella scan sees its actual header set. Keep -Werror intact.
        run(["/usr/sbin/dot_clean", "-m", output_dir("developer-tools-frameworks")])
    for optimization in [0, 2]:
        tag = f"{emitter}-c{optimization}"
        exe = output_dir(f"developer-tools-framework-{tag}")
        run([prefix / "bin/clang", "-x", "objective-c", "-std=c11", "-fmodules", f"-O{optimization}",
             f"-fmodules-cache-path={output_dir('developer-tools-module-cache-' + tag)}",
             f"-F{output_dir("developer-tools-frameworks")}", "-Wall", "-Wextra", "-Werror",
             ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-module-consumer.m", "-o", exe])
        run([exe])


def prepare_clang_time_inputs():
    # Some external filesystems clamp pre-epoch times or change distant dates.
    # These are generated test inputs, not build outputs/caches. Keep them in
    # the owned worktree and verify storage before testing libclang's time_t.
    directory = ROOT / ".work/developer-tools-time"
    if not directory.resolve().is_relative_to(ROOT.resolve()):
        raise RuntimeError("Timestamp fixture directory must remain inside this worktree")
    directory.mkdir(parents=True, exist_ok=True)
    for suffix, timestamp in [("future", 0x100000123), ("past", -12345)]:
        timed_file = directory / f"developer-tools-time-{suffix}.h"
        if timed_file.is_symlink():
            raise RuntimeError("Timestamp fixture must be an owned file")
        timed_file.write_text("int original_time_value;\n")
        nanoseconds = timestamp * 1_000_000_000
        os.utime(timed_file, ns=(nanoseconds, nanoseconds))
        if timed_file.stat().st_mtime_ns != nanoseconds:
            raise RuntimeError("Owned timestamp fixture filesystem cannot preserve the requested time_t values")


def prepare_cpp_visitors(prefix):
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I", prefix / "include",
         ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-visitors.c", "-L", prefix / "lib", "-lclang",
         f"-Wl,-rpath,{prefix / 'lib'}", "-o", output_dir("developer-tools-cpp-visitors")])
    visitor_results = run([output_dir("developer-tools-cpp-visitors"), ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-clang-cpp.h"]).split()
    if len(visitor_results) != 8 or any(not value.isdecimal() for value in visitor_results):
        raise RuntimeError("Independent C++ native visitor oracle is incomplete")
    visitor_oracle = output_dir("developer-tools-cpp-visitor-oracle.jai")
    visitor_oracle.write_text("SDK_Visitor_Results :: u32.[" + ", ".join(visitor_results) + "];\n")
    run([JAIFMT, "--config", ROOT / "jacket.toml", visitor_oracle])
    visitor_names = ["fields_return", "fields_count", "bases_return", "bases_count", "methods_return", "methods_count", "stopped_return", "stopped_count"]
    visitor_evidence = {"scope": "host official SDK C++17 AST queries and independent C visitor return/count oracle; C++ method bodies not executed",
                        "native_c_results": dict(zip(visitor_names, map(int, visitor_results))),
                        "return_comment_discrepancy": "Installed public Index.h describes nonzero only for early termination; this SDK returns1 for complete and stopped visits. Direct Jai bindings preserve raw native returns.",
                        "sdk_evidence": "developer-tools-clang-evidence.json",
                        "original_fixture_sha256": {name: sha256_file(driven(name))
                                                   for name in ["bindings-clang-cpp.h", "bindings-cpp-visitors.c", "bindings-clang-cpp.jai"]}}
    (output_dir("developer-tools-cpp-evidence.json")).write_text(json.dumps(visitor_evidence, indent=2) + "\n")
    for triple in ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"]:
        run([prefix / "bin/clang++", "-target", triple, "-x", "c++", "-std=c++17", "-Wall", "-Wextra", "-Werror",
             "-fsyntax-only", ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-clang-cpp.h"])
    return visitor_oracle


def check_bridge_consumers(prefix, emitter, workflow=False):
    stem = "cpp-workflow" if workflow else "bridge"
    library = "cpp_workflow" if workflow else "bridge"
    if workflow:
        run([prefix / "bin/clang++", "-x", "c++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-fsyntax-only",
             ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-cpp-nonpod.h"])
        for suffix in ["generated", "linux", "windows"]:
            generated = output_dir(f"developer-tools-cpp-workflow-{suffix}.jai")
            run([JAIFMT, "--config", ROOT / "jacket.toml", generated])
            run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", generated])
            run([cargo_binary("jailint"), "-D", "warnings", generated])
        for os_name in ["linux", "windows"]:
            run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-cpp-workflow-target.jai", "-check", "-os", os_name, "-cpu", "x64", "-I", output_dir()])
    extension = "dylib" if platform.system() == "Darwin" else "so"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    for cpp_optimization in [0, 2]:
        run([prefix / "bin/clang++", "-std=c++17", f"-O{cpp_optimization}", "-Wall", "-Wextra", "-Werror",
             *flags, output_dir(f"developer-tools-{stem}.cpp"), driven(f"bindings-{stem}-native.cpp"),
             "-o", output_dir(f"libdeveloper_tools_{library}.{extension}")])
        source = driven(f"bindings-{stem}-roundtrip.jai")
        run([JAIC, source, "-interp", "-I", output_dir()], output="ok\n")
        for optimization in [0, 2]:
            exe = output_dir(f"developer-tools-{stem}-{emitter}-cpp{cpp_optimization}-jai{optimization}")
            run([JAIC, source, f"-O{optimization}", "-exe", exe, "-I", output_dir()])
            run([exe], output="ok\n")


def check_measured_profiler_csv():
    lines = (output_dir("developer-tools-iprof-measured.csv")).read_text().splitlines()
    rows = list(csv.DictReader(lines[1:]))
    if len(rows) != 2 or [row["name"] for row in rows] != ["original outer", "original inner"] or [float(row["calls"]) for row in rows] != [3, 1]:
        raise RuntimeError("Measured profiler CSV lacks actual runtime zones/call counts")
    if float(rows[0]["self (ms)"]) < 10 or float(rows[0]["hier (ms)"]) < 25 or float(rows[1]["hier (ms)"]) < 5:
        raise RuntimeError("Measured profiler CSV does not preserve actual elapsed milliseconds")


def check_profiler_client_children():
    for configuration in range(8):
        for optimization in [0, 2]:
            run([output_dir(f"developer-tools-iprof-client-c{configuration}-o{optimization}")], output="ok\n")
        source = output_dir(f"developer-tools-iprof-client-input-c{configuration}.jai")
        run([JAIFMT, "--config", ROOT / "jacket.toml", source])
        run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", source])
        run([cargo_binary("jailint"), "-D", "warnings", source])



VISITOR_ORACLE = output_dir("developer-tools-cpp-visitor-oracle.jai")


def inline_setup():
    """The fixtures the cases and the format/lint step share: libraries, generated bindings and oracles under target/."""
    prefix = build_developer_bindings.llvm_prefix()
    prepare_cpp_visitors(prefix)
    framework = output_dir("developer-tools-frameworks", "OriginalNative.framework")
    (framework / "Modules").mkdir(parents=True, exist_ok=True)
    (framework / "Headers").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-module-header.h", framework / "Headers/OriginalNative.h")
    prepare_clang_time_inputs()
    run([prefix / "bin/clang", "-target", "x86_64-unknown-linux-gnu", "-g", "-fno-omit-frame-pointer",
         "-c", "stdlib/Debug_Info/tests/driven/dwarf-native.c", "-o", output_dir("developer-tools-dwarf.o")])
    run([prefix / "bin/llvm-objcopy", "--dump-section", f".eh_frame={output_dir('developer-tools-eh-frame.bin')}",
         output_dir("developer-tools-dwarf.o")])
    for architecture, triple in [("x64", "x86_64-pc-windows-msvc"), ("arm64", "aarch64-pc-windows-msvc")]:
        object_path = output_dir(f"developer-tools-codeview-{architecture}.obj")
        data_path = output_dir(f"developer-tools-codeview-{architecture}.bin")
        run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-target", triple,
             "-c", "stdlib/Debug_Info/tests/driven/codeview-lines.c", "-o", object_path])
        run([prefix / "bin/llvm-objcopy", "--dump-section", f".data={data_path}", object_path])
        register_object = output_dir(f"developer-tools-codeview-register-{architecture}.obj")
        register_data = output_dir(f"developer-tools-codeview-register-{architecture}.bin")
        run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-target", triple,
             "-c", "stdlib/Debug_Info/tests/driven/codeview-register.c", "-o", register_object])
        run([prefix / "bin/llvm-objcopy", "--dump-section", f".data={register_data}", register_object])
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I", prefix / "include",
         "stdlib/Bindings_Generator/tests/driven/bindings-sdk-layout.c", "-o", output_dir("developer-tools-sdk-layout")])
    run([output_dir("developer-tools-sdk-layout")], output="CXString=16 CXCursor=32 CXType=24 CXUnsavedFile=24\nCXIndexOptions=24 PreambleStoragePath=8 InvocationEmissionPath=16\nCXToken=24 ptr_data=16 CXTokenKind=4\n")
    run([JAIC, "stdlib/Generate_C_Header/tests/driven/c-header.jai", "-interp"], output="ok\n")
    run(["clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(), "-I", "target",
         "stdlib/Bindings_Generator/tests/driven/point.c", "-o", build_developer_bindings.shared_library("developer_tools_point")])
    run([JAIC, "stdlib/Generate_C_Header/tests/driven/c-header-modes.jai", "-interp"], output="ok\n")
    for standard, source, exe in [("c11", "header-enum-normal.c", "enum-normal"),
                                   ("c23", "header-enum-fixed.c", "enum-fixed")]:
        run([prefix / "bin/clang", f"-std={standard}", "-Wall", "-Wextra", "-Werror", "-I", "target",
             driven(f"{source}"), "-o", output_dir(f"developer-tools-{exe}")])
        run([output_dir(f"developer-tools-{exe}")])
    run([prefix / "bin/clang++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-I", "target",
         "stdlib/Bindings_Generator/tests/driven/header-enum-fixed.cpp", "-o", output_dir("developer-tools-enum-cpp")])
    run([output_dir("developer-tools-enum-cpp")])
    run([prefix / "bin/clang", "-std=c23", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(), "-I", "target",
         "stdlib/Bindings_Generator/tests/driven/header-support.c", "-o", build_developer_bindings.shared_library("developer_tools_support")])
    run([prefix / "bin/clang++", "-x", "c++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-I", "target",
         "-fsyntax-only", "stdlib/Bindings_Generator/tests/driven/header-support.c"])
    # The generator loads every configured library, so each must exist before its
    # first generation. The C++ bridge libraries are rebuilt with generated bridges later.
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         "stdlib/Bindings_Generator/tests/driven/bindings-sdk.c", "-o", build_developer_bindings.shared_library("developer_tools_sdk")])
    for stem, library in [("bridge", "bridge"), ("cpp-workflow", "cpp_workflow")]:
        run([prefix / "bin/clang++", "-x", "c++", "-std=c++17", *build_developer_bindings.shared_flags(),
             f"stdlib/Bindings_Generator/tests/driven/bindings-{stem}-native.cpp", "-o", build_developer_bindings.shared_library(f"developer_tools_{library}")])
    run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-public.jai", "-interp"], output="ok\n")
    generated_public = output_dir("developer-tools-public-generated.jai")
    run([JAIFMT, "--config", ROOT / "jacket.toml", generated_public])
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", generated_public])
    run([cargo_binary("jailint"), "-D", "warnings", generated_public])
    run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-sdk-public.jai", "-interp"], output="ok\n")
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         "stdlib/Bindings_Generator/tests/driven/bindings-sdk.c", "-o", build_developer_bindings.shared_library("developer_tools_sdk")])
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(), "-I", prefix / "include",
         "stdlib/Bindings_Generator/tests/driven/bindings-options.c", "-o", build_developer_bindings.shared_library("developer_tools_options")])
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         "stdlib/Bindings_Generator/tests/driven/bindings-macro-values.c", "-o", build_developer_bindings.shared_library("developer_tools_macros")])
    run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-macros.jai", "-interp"], output="ok\n")
    run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-macro-custom.jai", "-interp"], output="ok\n")
    run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-macro-enums.jai", "-interp"], output="ok\n")
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         "stdlib/Bindings_Generator/tests/driven/bindings-macro-merge.c", "-o", build_developer_bindings.shared_library("developer_tools_merge")])
    run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-macro-merge.jai", "-interp"], output="ok\n")
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         "stdlib/Bindings_Generator/tests/driven/bindings-model-properties.c", "-o", build_developer_bindings.shared_library("developer_tools_properties")])
    run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-model-properties.jai", "-interp"], output="ok\n")
    run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-custom.jai", "-interp"], output="ok\n")
    for name in ["custom", "model"]:
        generated = output_dir(f"developer-tools-{name}-generated.jai")
        run([JAIFMT, "--config", ROOT / "jacket.toml", generated])
        run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", generated])
        run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), generated])
    for os_name in ["linux", "windows"]:
        run([JAIC, output_dir("developer-tools-model-generated.jai"), "-check", "-os", os_name, "-cpu", "x64"])


def inline_case(case):
    """One driven case: interpreted, then native at O0 and O2."""
    prefix = build_developer_bindings.llvm_prefix()
    source = output_dir("developer-tools-native-abi.jai") if case == "bindings-native-abi" else driven(f"{case}.jai")
    error = "C header: C cannot project multiple return values\n" if case == "c-header-reject" else ""
    if case == "debug-native":
        error = "native debug output\n"
    expected = "x = 7 (s32)\nok\n" if case == "print-vars" else "ok\n"
    run([JAIC, source, "-interp", "-I", output_dir()], output=expected, error=error)
    if case == "bindings-clang-build-system":
        check_framework_consumer(prefix, "interp")
    if case == "bindings-bridge":
        check_bridge_consumers(prefix, "interp")
    if case == "bindings-cpp-workflow":
        check_bridge_consumers(prefix, "interp", workflow=True)
    if case == "iprof-manual-clock":
        check_measured_profiler_csv()
    if case == "iprof-client-setup":
        check_profiler_client_children()
    if case == "bindings-cpp-fragments":
        for cpp_optimization in [0, 2]:
            cpp_exe = output_dir(f"developer-tools-cpp-fragments-interp-cxx{cpp_optimization}")
            run([prefix / "bin/clang++", "-std=c++17", f"-O{cpp_optimization}", "-Wall", "-Wextra", "-Werror",
                 output_dir("developer-tools-cpp-fragments.cpp"), "-o", cpp_exe])
            run([cpp_exe])
    for optimization in [0, 2]:
        exe = output_dir(f"developer-tools-{case}-o{optimization}")
        run([JAIC, source, f"-O{optimization}", "-exe", exe, "-I", output_dir()])
        run([exe], output=expected, error=error)
        if case == "bindings-clang-build-system":
            check_framework_consumer(prefix, f"o{optimization}")
        if case == "bindings-bridge":
            check_bridge_consumers(prefix, f"o{optimization}")
        if case == "bindings-cpp-workflow":
            check_bridge_consumers(prefix, f"o{optimization}", workflow=True)
        if case == "iprof-manual-clock":
            check_measured_profiler_csv()
        if case == "iprof-client-setup":
            check_profiler_client_children()
        if case == "bindings-cpp-fragments":
            for cpp_optimization in [0, 2]:
                cpp_exe = output_dir(f"developer-tools-cpp-fragments-o{optimization}-cxx{cpp_optimization}")
                run([prefix / "bin/clang++", "-std=c++17", f"-O{cpp_optimization}", "-Wall", "-Wextra", "-Werror",
                     output_dir("developer-tools-cpp-fragments.cpp"), "-o", cpp_exe])
                run([cpp_exe])


def inline_performance():
    for optimization in [0, 2]:
        exe = output_dir(f"developer-tools-performance-o{optimization}")
        report = run([JAIC, "stdlib/Performance_Report/tests/driven/performance-target.jai", f"-O{optimization}",
                      "-exe", exe, "+Performance_Report"])
        for required in ["Compiler performance report", "#run time:", "Bytecode instructions:",
                         "Calls:", "Polymorph solves:", "work:"]:
            if required not in report:
                raise RuntimeError(f"Missing measured report section {required}")
        rows = [float(x) for x in re.findall(r"^  .*: ([0-9.e+-]+) seconds,", report, re.MULTILINE)]
        if not rows or rows != sorted(rows, reverse=True) or not any(x > 0 for x in rows):
            raise RuntimeError("Run records must contain actual positive timings in descending order")
        run([exe], output="ok\n")


def inline_debug():
    run(["clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "stdlib/Debug/tests/driven/debug-protection.c",
         build_developer_bindings.shared_library("developer_tools_debug", ROOT / "stdlib/internal/developer-tools/debug-native"),
         "-o", output_dir("developer-tools-debug-protection")])
    run([output_dir("developer-tools-debug-protection")])
    abort_source = ROOT / "stdlib/Debug/tests/driven/debug-abort.jai"
    abort_commands = [[JAIC, abort_source, "-interp"]]
    for optimization in [0, 2]:
        exe = output_dir(f"developer-tools-debug-abort-o{optimization}")
        run([JAIC, abort_source, f"-O{optimization}", "-exe", exe])
        abort_commands.append([exe])
    for command in abort_commands:
        aborted = subprocess.run([str(x) for x in command], cwd=ROOT, env=ENV,
                                 text=True, capture_output=True, timeout=60)
        if aborted.returncode != 2 or aborted.stdout or aborted.stderr:
            raise RuntimeError("Debug.abort did not terminate with Exit_Code.Abort")


def inline_missing_report():
    missing = subprocess.run([str(JAIC), "stdlib/Performance_Report/tests/driven/performance-missing.jai", "-check"],
                             cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
    if missing.returncode or "warning: Performance_Report: compiler did not provide a performance report" not in missing.stderr:
        raise RuntimeError("Missing performance message did not produce its diagnostic")


def inline_rewrite():
    rewritten = output_dir("developer-tools-rewrite-program.jai")
    shutil.copyfile(ROOT / "stdlib/Rewrite/tests/driven/rewrite-input.jai", rewritten)
    rewrite_report = run([JAIC, "stdlib/Rewrite/tests/driven/rewrite-driver.jai", "-check"])
    if " -> (pointer).*" not in rewrite_report or "<<pointer" in rewritten.read_text():
        raise RuntimeError("Actual compiler-message rewrite did not replace the deprecated expression")
    run([JAIC, rewritten, "-interp"], output="ok\n")
    for optimization in [0, 2]:
        exe = output_dir(f"developer-tools-rewritten-o{optimization}")
        run([JAIC, rewritten, f"-O{optimization}", "-exe", exe])
        run([exe], output="ok\n")
    shutil.copyfile(ROOT / "stdlib/Rewrite/tests/driven/rewrite-input.jai", rewritten)


def inline_lint():
    paths = [ROOT / "stdlib/Generate_C_Header/module.jai",
             ROOT / "stdlib/Performance_Report/module.jai",
             ROOT / "stdlib/Print_Vars/module.jai",
             ROOT / "stdlib/Rewrite/module.jai",
             *sorted((ROOT / "stdlib/Debug_Info").glob("*.jai")),
             ROOT / "stdlib/Debug/module.jai",
             ROOT / "stdlib/Iprof/module.jai",
             *sorted((ROOT / "stdlib/Codex").glob("*.jai")),
             ROOT / "stdlib/Check/module.jai",
             *sorted((ROOT / "stdlib/Bindings_Generator").glob("*.jai")),
             *sorted((ROOT / "stdlib/internal/developer-tools").rglob("*.jai")),
             *driven_matching("*.jai"),
             output_dir("developer-tools-native-abi.jai"), VISITOR_ORACLE,
             output_dir("developer-tools-dwarf-line-oracle.jai"),
             output_dir("developer-tools-dwarf-frame-oracle.jai"),
             output_dir("developer-tools-eh-frame-oracle.jai"),
             output_dir("developer-tools-dwarf-type-oracle.jai"),
             output_dir("developer-tools-codeview-string-oracle.jai"),
             output_dir("developer-tools-dwarf-line-wire.jai"),
             output_dir("developer-tools-dwarf-line-wire-oracle.jai")]
    format_paths = [*paths, *sorted((ROOT / "tests/compiler/rejections").glob("*.jai")),
                    *sorted((ROOT / "tests/compiler/metaprograms").glob("*.jai")),
                    ROOT / "stdlib/Iprof/tests/driven/iprof-client-input.jai.inc"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *format_paths])
    # This private source requires the Windows facade's DEBUG_BINDINGS constant;
    # lint it below with the actual public platform active, not in a host main module.
    checker_contextual = {ROOT / "stdlib/internal/developer-tools/check-bindings.jai",
                          ROOT / "stdlib/internal/developer-tools/porting-check.jai"}
    # Round-trip consumers import a module that their own check generates into a private
    # scratch directory; that check lints them. Only programs whose inputs exist on this host belong here.
    def not_lintable_on_host(path):
        if path.suffix != ".jai" or not path.is_relative_to(ROOT / "tests"):
            return False
        if "#assert OS == .WINDOWS" in path.read_text() and platform.system() != "Windows":
            return True  # Windows-only programs are linted on a Windows host.
        names = re.findall(r'#import "(developer-tools-[^"]+)"', path.read_text())
        loads = re.findall(r'#load "([^"]+)"', path.read_text())
        return (any(not (output_dir(f"{name}.jai")).exists() for name in names)
                or any(not (path.parent / name).resolve().exists() for name in loads))

    # Generated oracle fragments are #loaded into the driven programs, which are linted with them;
    # on their own they lack the imports of the program that loads them.
    consumers = "".join(path.read_text() for path in driven_matching("*.jai"))

    def loaded_fragment(path):
        return path.parent == output_dir() and f'target/{path.name}"' in consumers

    host_lint_paths = [path for path in paths if path not in checker_contextual
                       and not not_lintable_on_host(path) and not loaded_fragment(path)]
    run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), *host_lint_paths])
    print(f"Linted {len(host_lint_paths)} of {len(paths)} programs, format-checked {len(format_paths)} files", flush=True)


def inline_windows_checks():
    check_paths = [ROOT / "stdlib/Check/module.jai", ROOT / "stdlib/internal/developer-tools/check-engine/module.jai",
                   ROOT / "stdlib/Check/tests/driven/check-porting.jai"]
    run([cargo_binary("jailint"), "-D", "warnings", "-os", "windows", "-cpu", "x64", *check_paths])
    run([JAIC, "stdlib/Check/tests/driven/check-porting.jai", "-check", "-os", "windows", "-cpu", "x64"])
    debug_windows = [ROOT / "stdlib/Debug/module.jai", ROOT / "stdlib/internal/developer-tools/debug-native.jai",
                     ROOT / "stdlib/Debug/tests/driven/debug-windows-contract.jai"]
    run([cargo_binary("jailint"), "-D", "warnings", "-os", "windows", "-cpu", "x64", *debug_windows])
    for cpu in ["x64", "arm64"]:
        run([JAIC, "stdlib/Debug/tests/driven/debug-windows-contract.jai", "-check", "-os", "windows", "-cpu", cpu])
        for os_name in ["linux", "windows"]:
            for case in ["msf-wire", "msf-public", "debug-int-set", "dwarf-cfa", "dwarf-line-query", "dwarf-type-lookup", "dwarf-fde", "codeview-strings", "bindings-equivalence", "iprof-history-values", "iprof-stack-depth", "iprof-manual-clock", "bindings-clang-time", "bindings-clang-tokens", "bindings-clang-build-system", "bindings-clang-source", "bindings-clang-cpp"]:
                run([JAIC, driven(f"{case}.jai"), "-check", "-os", os_name, "-cpu", cpu])


def inline_check_rejections():
    for case, reason in [("check-array", "fixed array used as a pointer in assignment"),
                         ("check-small-driver", "small integer addition requires an explicit intermediate integer cast")]:
        rejected = subprocess.run([str(JAIC), f"tests/compiler/rejections/{case}.jai", "-check", "-os", "windows", "-cpu", "x64"],
                                  cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
        if rejected.returncode != 1 or reason not in rejected.stderr:
            raise RuntimeError(f"Expected actual Check rejection {case}: {rejected.returncode}, {rejected.stderr}")
    private_time = subprocess.run([str(JAIC), "tests/compiler/rejections/bindings-private-time.jai", "-check"],
                                  cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
    if private_time.returncode != 1 or "no exported member `time_t`" not in private_time.stderr:
        raise RuntimeError("Bindings_Generator must not export its private native time_t alias")
    private_page = subprocess.run([str(JAIC), "tests/compiler/rejections/msf-private-page.jai", "-check"],
                                  cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
    if private_page.returncode != 1 or "no exported member `UPN`" not in private_page.stderr:
        raise RuntimeError("Debug_Info must not add an export for the contextual page-number alias")
    private_profile = subprocess.run([str(JAIC), "tests/compiler/rejections/iprof-private-runtime.jai", "-check"],
                                     cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
    if private_profile.returncode != 1 or "unknown identifier `profile_owner`" not in private_profile.stderr:
        raise RuntimeError("Iprof must not export its private runtime allocator state")


def inline_iprof_rejections():
    for case, diagnostic in [("iprof-unbalanced", "Iprof: zone_end without an active zone"),
                              ("iprof-depth", "Iprof: maximum profiling depth exceeded"),
                              ("msf-page-bounds", "Debug_Info: page outside initialized free-page map")]:
        fixture = ROOT / f"tests/compiler/rejections/{case}.jai"
        for mode in ["interp", "0", "2"]:
            executable = output_dir(f"developer-tools-{case}-o{mode}")
            if mode != "interp":
                run([JAIC, fixture, f"-O{mode}", "-exe", executable])
            command = [str(JAIC), str(fixture), "-interp"] if mode == "interp" else [str(executable)]
            def disable_core_dump():
                import resource
                resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
            rejected = subprocess.run(command, cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60,
                                      preexec_fn=disable_core_dump if os.name == "posix" else None)
            if rejected.returncode == 0 or diagnostic not in rejected.stdout + rejected.stderr:
                raise RuntimeError(f"Developer-tool rejection {case}/{mode} did not fail with the required diagnostic")


def inline_target_checks():
    for os_name, cpu in [("linux", "x64"), ("windows", "x64"), ("wasm", "x64")]:
        run([JAIC, "stdlib/Generate_C_Header/tests/driven/c-header.jai", "-check", "-os", os_name, "-cpu", cpu])
        run([JAIC, "stdlib/Debug/tests/driven/debug-target.jai", "-check", "-os", os_name, "-cpu", cpu])
        if os_name != "wasm":
            run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-literals.jai", "-check", "-os", os_name, "-cpu", cpu])
            run([JAIC, "stdlib/Bindings_Generator/tests/driven/bindings-clang-options.jai", "-check", "-os", os_name, "-cpu", cpu])


CASES = [
    "c-header",
    "c-header-roundtrip",
    "c-header-reject",
    "c-header-modes",
    "c-header-lifetime",
    "c-header-support-roundtrip",
    "bindings-public",
    "bindings-public-roundtrip",
    "bindings-literals",
    "bindings-tree",
    "bindings-sdk-public",
    "bindings-sdk-roundtrip",
    "bindings-clang-api",
    "print-vars",
    "rewrite",
    "dwarf-cfa",
    "debug-native",
    "iprof-report",
    "codex-inspection",
    "bindings-clang-diagnostics",
    "bindings-callback-context",
    "bindings-custom",
    "bindings-custom-roundtrip",
    "bindings-clang-options",
    "bindings-macros",
    "bindings-macros-roundtrip",
    "codeview-lines",
    "codeview-register",
    "bindings-macro-custom",
    "bindings-macro-custom-roundtrip",
    "bindings-macro-enums",
    "bindings-macro-enums-roundtrip",
    "bindings-macro-merge",
    "bindings-macro-merge-roundtrip",
    "bindings-cpp-fragments",
    "bindings-model-properties",
    "bindings-model-properties-roundtrip",
    "msf-wire",
    "msf-public",
    "debug-int-set",
    "bindings-clang-tokens",
    "print-vars-expressions",
    "bindings-clang-build-system",
    "bindings-native-abi",
    "bindings-clang-source",
    "bindings-clang-cpp",
    "dwarf-line-query",
    "dwarf-line-wire",
    "dwarf-fde",
    "bindings-clang-time",
    "dwarf-eh-frame",
    "bindings-bridge",
    "bindings-bridge-roundtrip",
    "bindings-equivalence",
    "iprof-history-values",
    "iprof-stack-depth",
    "dwarf-type-lookup",
    "bindings-cpp-workflow",
    "bindings-cpp-workflow-roundtrip",
    "codeview-strings",
    "codeview-sentinels",
    "iprof-manual-clock",
    "iprof-client-setup",
    "codex-metrics",
    "msf-page-bits",
]


def case_groups():
    """CASES with each "-roundtrip" case kept right behind the case whose generated module it consumes."""
    groups = []
    for case in CASES:
        if groups and case == groups[-1][0] + "-roundtrip":
            groups[-1].append(case)
        else:
            groups.append([case])
    return groups


# What a test reads that a manifest check leaves under target/ (fixture binaries, oracle fragments the check
# generates and a case #loads, evidence JSON). Everything else comes from `inline_setup`.
NEEDS = {
    "check_developer_dwarf_line_wire": ["check_developer_dwarf_lines"],
    "bindings-native-abi": ["check_developer_native_abi"],
    "dwarf-line-query": ["check_developer_dwarf_lines"],
    "dwarf-line-wire": ["check_developer_dwarf_lines", "check_developer_dwarf_line_wire"],
    "dwarf-fde": ["check_developer_dwarf_frames"],
    "dwarf-eh-frame": ["check_developer_eh_frames"],
    "dwarf-type-lookup": ["check_developer_dwarf_types"],
    "codeview-strings": ["check_developer_codeview_strings"],
    "msf-wire": ["check_developer_msf"],
    "msf-public": ["check_developer_msf"],
    "msf-page-bits": ["check_developer_msf"],
    "windows_checks": ["check_developer_dwarf_lines", "check_developer_dwarf_frames", "check_developer_dwarf_types",
                       "check_developer_codeview_strings", "check_developer_msf"],
    "lint": ["check_developer_native_abi", "check_developer_dwarf_lines", "check_developer_dwarf_line_wire",
             "check_developer_dwarf_frames", "check_developer_eh_frames", "check_developer_dwarf_types",
             "check_developer_codeview_strings", "check_developer_msf"],
}


def setup():
    """The fixtures the cases and the format/lint step share: libraries, generated bindings and oracles under target/."""
    run_check("check_developer_bindings")
    once("setup", inline_setup)


def need(name):
    for module in NEEDS.get(name, ()):
        run_check(module)


class ManifestChecks(unittest.TestCase):
    pass


def add_manifest_check(module, hosts):
    def test(self):
        run_check(module)

    test.__name__ = "test_" + module
    if hosts:
        test = unittest.skipUnless(platform.system() in hosts, f"needs a {'/'.join(hosts)} host")(test)
    setattr(ManifestChecks, test.__name__, test)


for _module, _hosts in CHECKS:
    add_manifest_check(_module, _hosts)


class DrivenCases(unittest.TestCase):
    pass


def add_case(group):
    def test(self):
        setup()
        need(group[0])
        for case in group:
            inline_case(case)

    test.__name__ = "test_" + group[0].replace("-", "_")
    setattr(DrivenCases, test.__name__, test)


for _group in case_groups():
    add_case(_group)


class Inline(unittest.TestCase):
    def test_performance(self):
        inline_performance()

    def test_debug(self):
        inline_debug()

    def test_missing_report(self):
        inline_missing_report()

    def test_rewrite(self):
        inline_rewrite()

    def test_lint(self):
        setup()
        need("lint")
        inline_lint()

    def test_windows_checks(self):
        setup()
        need("windows_checks")
        inline_windows_checks()

    def test_check_rejections(self):
        inline_check_rejections()

    def test_iprof_rejections(self):
        inline_iprof_rejections()

    def test_target_checks(self):
        inline_target_checks()


def load_tests(loader, tests, pattern):
    """Apply JACKET_TOOLING_PART=K/N: every N-th test, from the K-th."""
    part = os.environ.get(PART_ENV)
    if not part:
        return tests
    index, _, count = part.partition("/")
    if not (index.isdecimal() and count.isdecimal() and 1 <= int(index) <= int(count)):
        raise SystemExit(f"{PART_ENV} must be K/N with 1 <= K <= N, such as 2/3")
    every = [test for suite in tests for test in suite]
    kept = unittest.TestSuite(every[int(index) - 1 :: int(count)])
    return kept


if __name__ == "__main__":
    unittest.main()
