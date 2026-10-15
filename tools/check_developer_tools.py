#!/usr/bin/env python3
"""Focused developer tooling evidence from this checkout's source-built tools."""
import os
from pathlib import Path
import re
import subprocess
import shutil
import build_developer_bindings
import check_developer_bindings
import build_developer_debug

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
JAIC = ROOT / "target/debug/jaic"


def run(command, *, output=None, error=""):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, env=ENV,
                            text=True, capture_output=True, timeout=60)
    if result.returncode or (output is not None and result.stdout != output) or result.stderr != error:
        raise RuntimeError(f"{command}: exit {result.returncode}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def main():
    check_developer_bindings.main()
    build_developer_debug.main()
    prefix = build_developer_bindings.llvm_prefix()
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
    cases = ["c-header", "c-header-roundtrip", "c-header-reject", "c-header-modes", "c-header-lifetime", "c-header-support-roundtrip", "bindings-public", "bindings-public-roundtrip", "bindings-literals", "bindings-tree", "bindings-sdk-public", "bindings-sdk-roundtrip", "bindings-clang-api", "print-vars", "rewrite", "dwarf-cfa", "debug-native", "iprof-report", "codex-inspection", "check-porting"]
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-I", prefix / "include",
         "tests/developer-tools/bindings-sdk-layout.c", "-o", "target/developer-tools-sdk-layout"])
    run([ROOT / "target/developer-tools-sdk-layout"], output="CXString=16 CXCursor=32 CXType=24 CXUnsavedFile=24\nCXIndexOptions=24 PreambleStoragePath=8 InvocationEmissionPath=16\n")
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
    run([JAIC, "tests/developer-tools/bindings-custom.jai", "-interp"], output="ok\n")
    for name in ["custom", "model"]:
        generated = ROOT / f"target/developer-tools-{name}-generated.jai"
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", generated])
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", generated])
        run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", generated])
    for os_name in ["linux", "windows"]:
        run([JAIC, "target/developer-tools-model-generated.jai", "-check", "-os", os_name, "-cpu", "x64"])
    for case in cases:
        source = ROOT / f"tests/developer-tools/{case}.jai"
        error = "C header: C cannot project multiple return values" if case == "c-header-reject" else ""
        if case == "debug-native":
            error = "native debug output\n"
        expected = "x = 7 (s32)\nok\n" if case == "print-vars" else "ok\n"
        run([JAIC, source, "-interp", "-I", ROOT / "target"], output=expected, error=error)
        for optimization in [0, 2]:
            exe = ROOT / f"target/developer-tools-{case}-o{optimization}"
            run([JAIC, source, f"-O{optimization}", "-exe", exe, "-I", ROOT / "target"])
            run([exe], output=expected, error=error)
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
             ROOT / "stdlib/Debug_Info/module.jai",
             ROOT / "stdlib/Debug/module.jai",
             ROOT / "stdlib/Iprof/module.jai",
             ROOT / "stdlib/Codex/module.jai",
             ROOT / "stdlib/Check/module.jai",
             *sorted((ROOT / "stdlib/Bindings_Generator").glob("*.jai")),
             *sorted((ROOT / "stdlib/internal/developer-tools").glob("*.jai")),
             *sorted((ROOT / "tests/developer-tools").glob("*.jai"))]
    format_paths = [*paths, *sorted((ROOT / "tests/developer-tools/rejections").glob("*.jai")),
                    *sorted((ROOT / "tests/developer-tools/compiler-requests").glob("*.jai"))]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *format_paths])
    shutil.copyfile(ROOT / "tests/developer-tools/rewrite-input.jai", rewritten)
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", *paths])
    check_paths = [ROOT / "stdlib/Check/module.jai", ROOT / "stdlib/internal/developer-tools/porting-check.jai",
                   ROOT / "tests/developer-tools/check-porting.jai"]
    run([ROOT / "target/debug/jailint", "-D", "warnings", "--os", "windows", "--cpu", "x64", *check_paths])
    run([JAIC, "tests/developer-tools/check-porting.jai", "-check", "-os", "windows", "-cpu", "x64"])
    for case, reason in [("check-array", "fixed array used as a pointer in assignment"),
                         ("check-small-driver", "small integer addition requires an explicit intermediate integer cast")]:
        rejected = subprocess.run([str(JAIC), f"tests/developer-tools/rejections/{case}.jai", "-check", "-os", "windows", "-cpu", "x64"],
                                  cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
        if rejected.returncode != 1 or reason not in rejected.stderr:
            raise RuntimeError(f"Expected actual Check rejection {case}: {rejected.returncode}, {rejected.stderr}")
    for os_name, cpu in [("linux", "x64"), ("windows", "x64"), ("wasm", "x64")]:
        run([JAIC, "tests/developer-tools/c-header.jai", "-check", "-os", os_name, "-cpu", cpu])
        run([JAIC, "tests/developer-tools/debug-target.jai", "-check", "-os", os_name, "-cpu", cpu])
        if os_name != "wasm":
            run([JAIC, "tests/developer-tools/bindings-literals.jai", "-check", "-os", os_name, "-cpu", cpu])
            run([JAIC, "tests/developer-tools/bindings-clang-options.jai", "-check", "-os", os_name, "-cpu", cpu])
    print(f"Developer tooling supported suite: {len(cases)} interpreter/native O0/O2 fixtures, native C/C++ round trips, diagnostics, scoped format/lint and target ABI checks passed")


if __name__ == "__main__":
    main()
