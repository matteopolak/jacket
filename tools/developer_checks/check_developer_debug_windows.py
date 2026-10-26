#!/usr/bin/env python3
"""Cross-build owned Windows Debug code; never claim Windows execution on macOS."""
import json
import build_developer_bindings
from build_developer_bindings import mingw_prefix
import platform
import re
from driven_sources import driven
from dev_common import JAIFMT, ROOT, labeled_runner, sha256_file, cargo_target_dir, output_dir

EXPECTED = {"developer_debug_" + name for name in ["abort", "backtrace", "breakpoint", "debugger_present",
                                                 "free_symbols", "output", "readable", "symbols", "break_once",
                                                 "initialize", "enable_handlers", "disable_handlers",
                                                 "attach", "dialog", "dump", "set_dump_state", "latest_dump"]}


run = labeled_runner('Windows Debug build/check', expected=None, timeout=60, stdout=True)


def main():
    if platform.system() != "Darwin":
        raise RuntimeError("This cross-build recipe currently validates macOS-hosted MinGW-w64 x64 only")
    sdk = mingw_prefix()
    gcc = sdk / "bin/x86_64-w64-mingw32-gcc"
    llvm = build_developer_bindings.llvm_prefix()
    source = ROOT / "stdlib/internal/developer-tools/debug-native/runtime-windows.c"
    dll = output_dir("developer_tools_debug.dll")
    import_library = output_dir("libdeveloper_tools_debug.dll.a")
    run([gcc, "-std=c11", "-O2", "-fno-omit-frame-pointer", "-Wall", "-Wextra", "-Werror", "-shared",
         source, "-ldbghelp", "-lkernel32", "-luser32", "-ladvapi32", f"-Wl,--out-implib,{import_library}", "-o", dll])
    exports = run([llvm / "bin/llvm-readobj", "--coff-exports", dll])
    names = set(re.findall(r"^\s*Name: (\S+)", exports, re.MULTILINE))
    if names != EXPECTED:
        raise RuntimeError("Private Windows Debug export set does not match the Jai native signatures")
    fixture = ROOT / "stdlib/Debug/tests/driven/debug-windows-native.c"
    for optimization in [0, 2]:
        run([gcc, "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", fixture,
             import_library, "-o", output_dir(f"developer-tools-debug-windows-o{optimization}.exe")])
        run([gcc, "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
             ROOT / "stdlib/Debug/tests/driven/debug-windows-final.c", import_library,
             "-o", output_dir(f"developer-tools-debug-windows-final-o{optimization}.exe")])
        run([gcc, "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
             ROOT / "stdlib/Debug/tests/driven/debug-windows-dialog.c", import_library, "-luser32",
             "-o", output_dir(f"developer-tools-debug-windows-dialog-o{optimization}.exe")])
    run([llvm / "bin/clang", "--target=aarch64-w64-windows-gnu",
         f"--sysroot={sdk / 'x86_64-w64-mingw32'}", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
         "-c", source, "-o", output_dir("developer-tools-debug-windows-arm64.obj")])
    build = cargo_target_dir()
    # Check both public signatures and a future native behavior fixture. These
    # are deliberately not counted as Windows runtime evidence.
    for cpu in ["x64", "arm64"]:
        for name in ["debug-windows-contract", "debug-windows-lifecycle", "debug-windows-dumps"]:
            run([build / "debug/jaic", driven(f"{name}.jai"),
                 "-check", "-os", "windows", "-cpu", cpu])
        run([build / "debug/jailint", "-os", "windows", "-cpu", cpu, "-D", "warnings",
             ROOT / "stdlib/Debug/module.jai", ROOT / "stdlib/Debug/tests/driven/debug-windows-contract.jai",
             ROOT / "stdlib/Debug/tests/driven/debug-windows-lifecycle.jai", ROOT / "stdlib/Debug/tests/driven/debug-windows-dumps.jai"])
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check",
         ROOT / "stdlib/Debug/module.jai", ROOT / "stdlib/internal/developer-tools/debug-lifecycle.jai",
         ROOT / "stdlib/Debug/tests/driven/debug-windows-contract.jai",
         ROOT / "stdlib/Debug/tests/driven/debug-windows-lifecycle.jai", ROOT / "stdlib/Debug/tests/driven/debug-windows-dumps.jai"])
    headers = [sdk / "x86_64-w64-mingw32/include" / name for name in ["windows.h", "dbghelp.h"]]
    metadata = {"coverage": "Windows x64 SDK DLL/executable cross-build/link/export inspection and arm64 object compilation; no Windows execution",
                "source_sha256": sha256_file(source),
                "once_header_sha256": sha256_file(source.with_name("once.h")),
                "crash_state_header_sha256": sha256_file(source.with_name("crash-state.h")),
                "final_header_sha256": {name: sha256_file(source.with_name(name))
                                        for name in ["windows-command.h", "windows-launch.h", "windows-dumps.h"]},
                "fixture_sha256": sha256_file(fixture),
                "sdk_header_sha256": {str(path): sha256_file(path) for path in headers},
                "gcc_version": run([gcc, "--version"]).splitlines()[0],
                "exports": sorted(names), "runtime_status": "pending coordinator Windows native CI"}
    (output_dir("developer-tools-windows-debug-evidence.json")).write_text(json.dumps(metadata, indent=2) + "\n")
    print("Windows Debug x64 SDK DLL/import/export checks and arm64 compilation passed; native runtime evidence pending")


if __name__ == "__main__":
    main()
