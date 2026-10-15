#!/usr/bin/env python3
"""Focused developer tooling evidence from this checkout's source-built tools."""
import os
import hashlib
import json
from pathlib import Path
import re
import subprocess
import shutil
import build_developer_bindings
import platform
import csv
import check_developer_bindings
import check_developer_cpp_custom
import check_developer_type_emission
import check_developer_value_emission
import check_developer_check
import check_developer_check_plugin
import check_developer_cursor_source
import check_developer_multi_input
import check_developer_pointer_defaults
import check_developer_string_macros
import check_developer_declaration_visitors
import check_developer_integer_range
import build_developer_debug
import check_developer_debug_lifecycle
import check_developer_debug_windows
import check_developer_msf
import check_developer_native_abi
import check_developer_dwarf_lines
import check_developer_dwarf_frames
import check_developer_eh_frames
import check_developer_sdk_configuration
import check_developer_dwarf_types
import check_developer_codeview_strings
import check_developer_dwarf_line_wire
import check_developer_profiler_windows
import check_generator_platform

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
JAIC = ROOT / "target/debug/jaic"


def run(command, *, output=None, error=""):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, env=ENV,
                            text=True, capture_output=True, timeout=60)
    if result.returncode or (output is not None and result.stdout != output) or result.stderr != error:
        diagnostic = (result.stderr or result.stdout).splitlines()[:2]
        raise RuntimeError(f"{command}: exit {result.returncode}; diagnostic={diagnostic}")
    return result.stdout


def check_framework_consumer(prefix, emitter):
    if platform.system() == "Darwin":
        # macOS creates AppleDouble metadata files on external filesystems. The
        # framework is an owned generated fixture; remove metadata sidecars so
        # Clang's umbrella scan sees its actual header set. Keep -Werror intact.
        run(["/usr/sbin/dot_clean", "-m", ROOT / "target/developer-tools-frameworks"])
    for optimization in [0, 2]:
        tag = f"{emitter}-c{optimization}"
        exe = ROOT / f"target/developer-tools-framework-{tag}"
        run([prefix / "bin/clang", "-x", "objective-c", "-std=c11", "-fmodules", f"-O{optimization}",
             f"-fmodules-cache-path={ROOT / ('target/developer-tools-module-cache-' + tag)}",
             f"-F{ROOT / 'target/developer-tools-frameworks'}", "-Wall", "-Wextra", "-Werror",
             ROOT / "tests/developer-tools/bindings-module-consumer.m", "-o", exe])
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
         ROOT / "tests/developer-tools/bindings-cpp-visitors.c", "-L", prefix / "lib", "-lclang",
         f"-Wl,-rpath,{prefix / 'lib'}", "-o", ROOT / "target/developer-tools-cpp-visitors"])
    visitor_results = run([ROOT / "target/developer-tools-cpp-visitors", ROOT / "tests/developer-tools/bindings-clang-cpp.h"]).split()
    if len(visitor_results) != 8 or any(not value.isdecimal() for value in visitor_results):
        raise RuntimeError("Independent C++ native visitor oracle is incomplete")
    visitor_oracle = ROOT / "target/developer-tools-cpp-visitor-oracle.jai"
    visitor_oracle.write_text("SDK_Visitor_Results :: u32.[" + ", ".join(visitor_results) + "];\n")
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", visitor_oracle])
    visitor_names = ["fields_return", "fields_count", "bases_return", "bases_count", "methods_return", "methods_count", "stopped_return", "stopped_count"]
    visitor_evidence = {"scope": "host official SDK C++17 AST queries and independent C visitor return/count oracle; C++ method bodies not executed",
                        "native_c_results": dict(zip(visitor_names, map(int, visitor_results))),
                        "return_comment_discrepancy": "Installed public Index.h describes nonzero only for early termination; this SDK returns1 for complete and stopped visits. Direct Jai bindings preserve raw native returns.",
                        "sdk_evidence": "developer-tools-clang-evidence.json",
                        "original_fixture_sha256": {name: hashlib.sha256((ROOT / 'tests/developer-tools' / name).read_bytes()).hexdigest()
                                                   for name in ["bindings-clang-cpp.h", "bindings-cpp-visitors.c", "bindings-clang-cpp.jai"]}}
    (ROOT / "docs/native/developer-tools-cpp-evidence.json").write_text(json.dumps(visitor_evidence, indent=2) + "\n")
    for triple in ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"]:
        run([prefix / "bin/clang++", "-target", triple, "-x", "c++", "-std=c++17", "-Wall", "-Wextra", "-Werror",
             "-fsyntax-only", ROOT / "tests/developer-tools/bindings-clang-cpp.h"])
    return visitor_oracle


def check_bridge_consumers(prefix, emitter, workflow=False):
    stem = "cpp-workflow" if workflow else "bridge"
    library = "cpp_workflow" if workflow else "bridge"
    if workflow:
        run([prefix / "bin/clang++", "-x", "c++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-fsyntax-only",
             ROOT / "tests/developer-tools/bindings-cpp-nonpod.h"])
        for suffix in ["generated", "linux", "windows"]:
            generated = ROOT / f"target/developer-tools-cpp-workflow-{suffix}.jai"
            run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
            run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", generated])
            run([ROOT / "target/debug/jailint", "-D", "warnings", generated])
        for os_name in ["linux", "windows"]:
            run([JAIC, "tests/developer-tools/bindings-cpp-workflow-target.jai", "-check", "-os", os_name, "-cpu", "x64", "-I", ROOT / "target"])
    extension = "dylib" if platform.system() == "Darwin" else "so"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    for cpp_optimization in [0, 2]:
        run([prefix / "bin/clang++", "-std=c++17", f"-O{cpp_optimization}", "-Wall", "-Wextra", "-Werror",
             *flags, ROOT / f"target/developer-tools-{stem}.cpp", ROOT / f"tests/developer-tools/bindings-{stem}-native.cpp",
             "-o", ROOT / f"target/libdeveloper_tools_{library}.{extension}"])
        source = ROOT / f"tests/developer-tools/bindings-{stem}-roundtrip.jai"
        run([JAIC, source, "-interp", "-I", ROOT / "target"], output="ok\n")
        for optimization in [0, 2]:
            exe = ROOT / f"target/developer-tools-{stem}-{emitter}-cpp{cpp_optimization}-jai{optimization}"
            run([JAIC, source, f"-O{optimization}", "-exe", exe, "-I", ROOT / "target"])
            run([exe], output="ok\n")


def check_measured_profiler_csv():
    lines = (ROOT / "target/developer-tools-iprof-measured.csv").read_text().splitlines()
    rows = list(csv.DictReader(lines[1:]))
    if len(rows) != 2 or [row["name"] for row in rows] != ["original outer", "original inner"] or [float(row["calls"]) for row in rows] != [3, 1]:
        raise RuntimeError("Measured profiler CSV lacks actual runtime zones/call counts")
    if float(rows[0]["self (ms)"]) < 10 or float(rows[0]["hier (ms)"]) < 25 or float(rows[1]["hier (ms)"]) < 5:
        raise RuntimeError("Measured profiler CSV does not preserve actual elapsed milliseconds")


def check_profiler_client_children():
    for configuration in range(8):
        for optimization in [0, 2]:
            run([ROOT / f"target/developer-tools-iprof-client-c{configuration}-o{optimization}"], output="ok\n")
        source = ROOT / f"target/developer-tools-iprof-client-input-c{configuration}.jai"
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", source])
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", source])
        run([ROOT / "target/debug/jailint", "-D", "warnings", source])


def main():
    check_developer_bindings.main()
    check_generator_platform.main()
    check_developer_cpp_custom.main()
    check_developer_type_emission.main()
    check_developer_value_emission.main()
    check_developer_check.main()
    check_developer_check_plugin.main()
    check_developer_cursor_source.main()
    check_developer_multi_input.main()
    check_developer_pointer_defaults.main()
    check_developer_string_macros.main()
    check_developer_declaration_visitors.main()
    check_developer_integer_range.main()
    configured_cases = check_developer_sdk_configuration.main()
    build_developer_debug.main()
    check_developer_debug_lifecycle.main()
    check_developer_debug_windows.main()
    check_developer_msf.main()
    check_developer_native_abi.main()
    check_developer_dwarf_lines.main()
    check_developer_dwarf_line_wire.main()
    check_developer_dwarf_frames.main()
    check_developer_eh_frames.main()
    check_developer_dwarf_types.main()
    check_developer_codeview_strings.main()
    check_developer_profiler_windows.main()
    prefix = build_developer_bindings.llvm_prefix()
    framework = ROOT / "target/developer-tools-frameworks/OriginalNative.framework"
    (framework / "Modules").mkdir(parents=True, exist_ok=True)
    (framework / "Headers").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ROOT / "tests/developer-tools/bindings-module-header.h", framework / "Headers/OriginalNative.h")
    visitor_oracle = prepare_cpp_visitors(prefix)
    prepare_clang_time_inputs()
    run([prefix / "bin/clang", "-target", "x86_64-unknown-linux-gnu", "-g", "-fno-omit-frame-pointer",
         "-c", "tests/developer-tools/dwarf-native.c", "-o", "target/developer-tools-dwarf.o"])
    run([prefix / "bin/llvm-objcopy", "--dump-section", ".eh_frame=target/developer-tools-eh-frame.bin",
         "target/developer-tools-dwarf.o"])
    for architecture, triple in [("x64", "x86_64-pc-windows-msvc"), ("arm64", "aarch64-pc-windows-msvc")]:
        object_path = ROOT / f"target/developer-tools-codeview-{architecture}.obj"
        data_path = ROOT / f"target/developer-tools-codeview-{architecture}.bin"
        run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-target", triple,
             "-c", "tests/developer-tools/codeview-lines.c", "-o", object_path])
        run([prefix / "bin/llvm-objcopy", "--dump-section", f".data={data_path}", object_path])
        register_object = ROOT / f"target/developer-tools-codeview-register-{architecture}.obj"
        register_data = ROOT / f"target/developer-tools-codeview-register-{architecture}.bin"
        run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-target", triple,
             "-c", "tests/developer-tools/codeview-register.c", "-o", register_object])
        run([prefix / "bin/llvm-objcopy", "--dump-section", f".data={register_data}", register_object])
    cases = ["c-header", "c-header-roundtrip", "c-header-reject", "c-header-modes", "c-header-lifetime", "c-header-support-roundtrip", "bindings-public", "bindings-public-roundtrip", "bindings-literals", "bindings-tree", "bindings-sdk-public", "bindings-sdk-roundtrip", "bindings-clang-api", "print-vars", "rewrite", "dwarf-cfa", "debug-native", "iprof-report", "codex-inspection"]
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I", prefix / "include",
         "tests/developer-tools/bindings-sdk-layout.c", "-o", "target/developer-tools-sdk-layout"])
    run([ROOT / "target/developer-tools-sdk-layout"], output="CXString=16 CXCursor=32 CXType=24 CXUnsavedFile=24\nCXIndexOptions=24 PreambleStoragePath=8 InvocationEmissionPath=16\nCXToken=24 ptr_data=16 CXTokenKind=4\n")
    run([JAIC, "tests/developer-tools/c-header.jai", "-interp"], output="ok\n")
    run(["clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(), "-I", "target",
         "tests/developer-tools/point.c", "-o", build_developer_bindings.shared_library("developer_tools_point")])
    run([JAIC, "tests/developer-tools/c-header-modes.jai", "-interp"], output="ok\n")
    for standard, source, exe in [("c11", "header-enum-normal.c", "enum-normal"),
                                   ("c23", "header-enum-fixed.c", "enum-fixed")]:
        run([prefix / "bin/clang", f"-std={standard}", "-Wall", "-Wextra", "-Werror", "-I", "target",
             ROOT / f"tests/developer-tools/{source}", "-o", ROOT / f"target/developer-tools-{exe}"])
        run([ROOT / f"target/developer-tools-{exe}"])
    run([prefix / "bin/clang++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-I", "target",
         "tests/developer-tools/header-enum-fixed.cpp", "-o", "target/developer-tools-enum-cpp"])
    run([ROOT / "target/developer-tools-enum-cpp"])
    run([prefix / "bin/clang", "-std=c23", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(), "-I", "target",
         "tests/developer-tools/header-support.c", "-o", build_developer_bindings.shared_library("developer_tools_support")])
    run([prefix / "bin/clang++", "-x", "c++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-I", "target",
         "-fsyntax-only", "tests/developer-tools/header-support.c"])
    run([JAIC, "tests/developer-tools/bindings-public.jai", "-interp"], output="ok\n")
    generated_public = ROOT / "target/developer-tools-public-generated.jai"
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated_public])
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", generated_public])
    run([ROOT / "target/debug/jailint", "-D", "warnings", generated_public])
    run([JAIC, "tests/developer-tools/bindings-sdk-public.jai", "-interp"], output="ok\n")
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         "tests/developer-tools/bindings-sdk.c", "-o", build_developer_bindings.shared_library("developer_tools_sdk")])
    cases.extend(["bindings-clang-diagnostics", "bindings-callback-context", "bindings-custom", "bindings-custom-roundtrip", "bindings-clang-options", "bindings-macros", "bindings-macros-roundtrip", "codeview-lines", "codeview-register"])
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(), "-I", prefix / "include",
         "tests/developer-tools/bindings-options.c", "-o", build_developer_bindings.shared_library("developer_tools_options")])
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         "tests/developer-tools/bindings-macro-values.c", "-o", build_developer_bindings.shared_library("developer_tools_macros")])
    run([JAIC, "tests/developer-tools/bindings-macros.jai", "-interp"], output="ok\n")
    run([JAIC, "tests/developer-tools/bindings-macro-custom.jai", "-interp"], output="ok\n")
    cases.extend(["bindings-macro-custom", "bindings-macro-custom-roundtrip"])
    run([JAIC, "tests/developer-tools/bindings-macro-enums.jai", "-interp"], output="ok\n")
    cases.extend(["bindings-macro-enums", "bindings-macro-enums-roundtrip"])
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         "tests/developer-tools/bindings-macro-merge.c", "-o", build_developer_bindings.shared_library("developer_tools_merge")])
    run([JAIC, "tests/developer-tools/bindings-macro-merge.jai", "-interp"], output="ok\n")
    cases.extend(["bindings-macro-merge", "bindings-macro-merge-roundtrip"])
    cases.append("bindings-cpp-fragments")
    cases.append("bindings-model-properties")
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         "tests/developer-tools/bindings-model-properties.c", "-o", build_developer_bindings.shared_library("developer_tools_properties")])
    run([JAIC, "tests/developer-tools/bindings-model-properties.jai", "-interp"], output="ok\n")
    cases.append("bindings-model-properties-roundtrip")
    cases.append("msf-wire")
    cases.append("msf-public")
    cases.append("debug-int-set")
    cases.append("bindings-clang-tokens")
    cases.append("bindings-clang-build-system")
    cases.append("bindings-native-abi")
    cases.append("bindings-clang-source")
    cases.append("bindings-clang-cpp")
    cases.append("dwarf-line-query")
    cases.append("dwarf-line-wire")
    cases.append("dwarf-fde")
    cases.append("bindings-clang-time")
    cases.append("dwarf-eh-frame")
    cases.extend(["bindings-bridge", "bindings-bridge-roundtrip"])
    cases.append("bindings-equivalence")
    cases.extend(["iprof-history-values", "iprof-stack-depth"])
    cases.append("dwarf-type-lookup")
    cases.extend(["bindings-cpp-workflow", "bindings-cpp-workflow-roundtrip"])
    cases.append("codeview-strings")
    cases.append("codeview-sentinels")
    cases.append("iprof-manual-clock")
    cases.append("iprof-client-setup")
    cases.append("codex-metrics")
    cases.append("msf-page-bits")
    run([JAIC, "tests/developer-tools/bindings-custom.jai", "-interp"], output="ok\n")
    for name in ["custom", "model"]:
        generated = ROOT / f"target/developer-tools-{name}-generated.jai"
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", generated])
        run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", generated])
    for os_name in ["linux", "windows"]:
        run([JAIC, "target/developer-tools-model-generated.jai", "-check", "-os", os_name, "-cpu", "x64"])
    for case in cases:
        source = ROOT / "target/developer-tools-native-abi.jai" if case == "bindings-native-abi" else ROOT / f"tests/developer-tools/{case}.jai"
        error = "C header: C cannot project multiple return values" if case == "c-header-reject" else ""
        if case == "debug-native":
            error = "native debug output\n"
        expected = "x = 7 (s32)\nok\n" if case == "print-vars" else "ok\n"
        run([JAIC, source, "-interp", "-I", ROOT / "target"], output=expected, error=error)
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
                cpp_exe = ROOT / f"target/developer-tools-cpp-fragments-interp-cxx{cpp_optimization}"
                run([prefix / "bin/clang++", "-std=c++17", f"-O{cpp_optimization}", "-Wall", "-Wextra", "-Werror",
                     ROOT / "target/developer-tools-cpp-fragments.cpp", "-o", cpp_exe])
                run([cpp_exe])
        for optimization in [0, 2]:
            exe = ROOT / f"target/developer-tools-{case}-o{optimization}"
            run([JAIC, source, f"-O{optimization}", "-exe", exe, "-I", ROOT / "target"])
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
                    cpp_exe = ROOT / f"target/developer-tools-cpp-fragments-o{optimization}-cxx{cpp_optimization}"
                    run([prefix / "bin/clang++", "-std=c++17", f"-O{cpp_optimization}", "-Wall", "-Wextra", "-Werror",
                         ROOT / "target/developer-tools-cpp-fragments.cpp", "-o", cpp_exe])
                    run([cpp_exe])
    for optimization in [0, 2]:
        exe = ROOT / f"target/developer-tools-performance-o{optimization}"
        report = run([JAIC, "tests/developer-tools/performance-target.jai", f"-O{optimization}",
                      "-exe", exe, "+Performance_Report"])
        for required in ["Compiler performance report", "#run time:", "Bytecode instructions:",
                         "Calls:", "Polymorph solves:", "work:"]:
            if required not in report:
                raise RuntimeError(f"Missing measured report section {required}")
        rows = [float(x) for x in re.findall(r"^  .*: ([0-9.e+-]+) seconds,", report, re.MULTILINE)]
        if not rows or rows != sorted(rows, reverse=True) or not any(x > 0 for x in rows):
            raise RuntimeError("Run records must contain actual positive timings in descending order")
        run([exe], output="ok\n")
    run(["clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "tests/developer-tools/debug-protection.c",
         build_developer_bindings.shared_library("developer_tools_debug", ROOT / "stdlib/internal/developer-tools/debug-native"),
         "-o", "target/developer-tools-debug-protection"])
    run([ROOT / "target/developer-tools-debug-protection"])
    abort_source = ROOT / "tests/developer-tools/debug-abort.jai"
    abort_commands = [[JAIC, abort_source, "-interp"]]
    for optimization in [0, 2]:
        exe = ROOT / f"target/developer-tools-debug-abort-o{optimization}"
        run([JAIC, abort_source, f"-O{optimization}", "-exe", exe])
        abort_commands.append([exe])
    for command in abort_commands:
        aborted = subprocess.run([str(x) for x in command], cwd=ROOT, env=ENV,
                                 text=True, capture_output=True, timeout=60)
        if aborted.returncode != 2 or aborted.stdout or aborted.stderr:
            raise RuntimeError("Debug.abort did not terminate with Exit_Code.Abort")
    missing = subprocess.run([str(JAIC), "tests/developer-tools/performance-missing.jai", "-check"],
                             cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
    if missing.returncode or "warning: Performance_Report: compiler did not provide a performance report" not in missing.stderr:
        raise RuntimeError("Missing performance message did not produce its diagnostic")
    rewritten = ROOT / "target/developer-tools-rewrite-program.jai"
    shutil.copyfile(ROOT / "tests/developer-tools/rewrite-input.jai", rewritten)
    rewrite_report = run([JAIC, "tests/developer-tools/rewrite-driver.jai", "-check"])
    if " -> (pointer).*" not in rewrite_report or "<<pointer" in rewritten.read_text():
        raise RuntimeError("Actual compiler-message rewrite did not replace the deprecated expression")
    run([JAIC, rewritten, "-interp"], output="ok\n")
    for optimization in [0, 2]:
        exe = ROOT / f"target/developer-tools-rewritten-o{optimization}"
        run([JAIC, rewritten, f"-O{optimization}", "-exe", exe])
        run([exe], output="ok\n")
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
             *sorted((ROOT / "tests/developer-tools").glob("*.jai")),
             ROOT / "target/developer-tools-native-abi.jai", visitor_oracle,
             ROOT / "target/developer-tools-dwarf-line-oracle.jai",
             ROOT / "target/developer-tools-dwarf-frame-oracle.jai",
             ROOT / "target/developer-tools-eh-frame-oracle.jai",
             ROOT / "target/developer-tools-dwarf-type-oracle.jai",
             ROOT / "target/developer-tools-codeview-string-oracle.jai",
             ROOT / "target/developer-tools-dwarf-line-wire.jai",
             ROOT / "target/developer-tools-dwarf-line-wire-oracle.jai"]
    format_paths = [*paths, *sorted((ROOT / "tests/developer-tools/rejections").glob("*.jai")),
                    *sorted((ROOT / "tests/developer-tools/compiler-requests").glob("*.jai")),
                    ROOT / "tests/developer-tools/iprof-client-input.jai.inc"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *format_paths])
    shutil.copyfile(ROOT / "tests/developer-tools/rewrite-input.jai", rewritten)
    # This private source requires the Windows facade's DEBUG_BINDINGS constant;
    # lint it below with the actual public platform active, not in a host main module.
    checker_contextual = {ROOT / "stdlib/internal/developer-tools/check-bindings.jai",
                          ROOT / "stdlib/internal/developer-tools/porting-check.jai"}
    host_lint_paths = [path for path in paths if path not in checker_contextual]
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", *host_lint_paths])
    check_paths = [ROOT / "stdlib/Check/module.jai", ROOT / "stdlib/internal/developer-tools/check-engine/module.jai",
                   ROOT / "tests/developer-tools/check-porting.jai"]
    run([ROOT / "target/debug/jailint", "-D", "warnings", "--os", "windows", "--cpu", "x64", *check_paths])
    run([JAIC, "tests/developer-tools/check-porting.jai", "-check", "-os", "windows", "-cpu", "x64"])
    debug_windows = [ROOT / "stdlib/Debug/module.jai", ROOT / "stdlib/internal/developer-tools/debug-native.jai",
                     ROOT / "tests/developer-tools/debug-windows-contract.jai"]
    run([ROOT / "target/debug/jailint", "-D", "warnings", "--os", "windows", "--cpu", "x64", *debug_windows])
    for cpu in ["x64", "arm64"]:
        run([JAIC, "tests/developer-tools/debug-windows-contract.jai", "-check", "-os", "windows", "-cpu", cpu])
        for os_name in ["linux", "windows"]:
            for case in ["msf-wire", "msf-public", "debug-int-set", "dwarf-cfa", "dwarf-line-query", "dwarf-type-lookup", "dwarf-fde", "codeview-strings", "bindings-equivalence", "iprof-history-values", "iprof-stack-depth", "iprof-manual-clock", "bindings-clang-time", "bindings-clang-tokens", "bindings-clang-build-system", "bindings-clang-source", "bindings-clang-cpp"]:
                run([JAIC, f"tests/developer-tools/{case}.jai", "-check", "-os", os_name, "-cpu", cpu])
    for case, reason in [("check-array", "fixed array used as a pointer in assignment"),
                         ("check-small-driver", "small integer addition requires an explicit intermediate integer cast")]:
        rejected = subprocess.run([str(JAIC), f"tests/developer-tools/rejections/{case}.jai", "-check", "-os", "windows", "-cpu", "x64"],
                                  cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
        if rejected.returncode != 1 or reason not in rejected.stderr:
            raise RuntimeError(f"Expected actual Check rejection {case}: {rejected.returncode}, {rejected.stderr}")
    private_time = subprocess.run([str(JAIC), "tests/developer-tools/rejections/bindings-private-time.jai", "-check"],
                                  cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
    if private_time.returncode != 1 or "no exported member `time_t`" not in private_time.stderr:
        raise RuntimeError("Bindings_Generator must not export its private native time_t alias")
    private_page = subprocess.run([str(JAIC), "tests/developer-tools/rejections/msf-private-page.jai", "-check"],
                                  cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
    if private_page.returncode != 1 or "no exported member `UPN`" not in private_page.stderr:
        raise RuntimeError("Debug_Info must not add an export for the contextual page-number alias")
    private_profile = subprocess.run([str(JAIC), "tests/developer-tools/rejections/iprof-private-runtime.jai", "-check"],
                                     cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
    if private_profile.returncode != 1 or "no exported member `profile_owner`" not in private_profile.stderr:
        raise RuntimeError("Iprof must not export its private runtime allocator state")
    for case, diagnostic in [("iprof-unbalanced", "Iprof: zone_end without an active zone"),
                              ("iprof-depth", "Iprof: maximum profiling depth exceeded"),
                              ("codex-conditional-metrics", "Codex: conditional metric weights require reviewed refinement"),
                              ("codex-basic-metrics", "Codex: Basic allocation metric recognition requires reviewed name set"),
                              ("msf-page-bounds", "Debug_Info: page outside initialized free-page map")]:
        fixture = ROOT / f"tests/developer-tools/rejections/{case}.jai"
        for mode in ["interp", "0", "2"]:
            executable = ROOT / f"target/developer-tools-{case}-o{mode}"
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
    for os_name, cpu in [("linux", "x64"), ("windows", "x64"), ("wasm", "x64")]:
        run([JAIC, "tests/developer-tools/c-header.jai", "-check", "-os", os_name, "-cpu", cpu])
        run([JAIC, "tests/developer-tools/debug-target.jai", "-check", "-os", os_name, "-cpu", cpu])
        if os_name != "wasm":
            run([JAIC, "tests/developer-tools/bindings-literals.jai", "-check", "-os", os_name, "-cpu", cpu])
            run([JAIC, "tests/developer-tools/bindings-clang-options.jai", "-check", "-os", os_name, "-cpu", cpu])
    print(f"Developer tooling supported suite: {len(cases) + configured_cases} interpreter/native O0/O2 fixtures, native C/C++ round trips, diagnostics, scoped format/lint and target ABI checks passed")


if __name__ == "__main__":
    main()
