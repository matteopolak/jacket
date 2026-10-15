#!/usr/bin/env python3
"""Cross-build owned Windows Debug code; never claim Windows execution on macOS."""
import hashlib
import json
import os
import build_developer_bindings
from pathlib import Path
import platform
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = {"developer_debug_" + name for name in ["abort", "backtrace", "breakpoint", "debugger_present",
                                                 "free_symbols", "output", "readable", "symbols", "break_once",
                                                 "initialize", "enable_handlers", "disable_handlers",
                                                 "attach", "dialog", "dump", "set_dump_state", "latest_dump"]}


def run(command):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, text=True, capture_output=True, timeout=60,
                            env=dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib")))
    if result.returncode or result.stderr:
        raise RuntimeError(f"Windows Debug build/check failed: {result.returncode}\n{result.stderr}")
    return result.stdout


def main():
    if platform.system() != "Darwin":
        raise RuntimeError("This cross-build recipe currently validates macOS-hosted MinGW-w64 x64 only")
    sdk = Path(os.environ.get("DEVELOPER_TOOLS_MINGW_PREFIX", "/opt/homebrew/opt/mingw-w64/toolchain-x86_64"))
    gcc = sdk / "bin/x86_64-w64-mingw32-gcc"
    llvm = build_developer_bindings.llvm_prefix()
    source = ROOT / "stdlib/internal/developer-tools/debug-native/runtime-windows.c"
    dll = ROOT / "target/developer_tools_debug.dll"
    import_library = ROOT / "target/libdeveloper_tools_debug.dll.a"
    run([gcc, "-std=c11", "-O2", "-fno-omit-frame-pointer", "-Wall", "-Wextra", "-Werror", "-shared",
         source, "-ldbghelp", "-lkernel32", "-luser32", "-ladvapi32", f"-Wl,--out-implib,{import_library}", "-o", dll])
    exports = run([llvm / "bin/llvm-readobj", "--coff-exports", dll])
    names = set(re.findall(r"^\s*Name: (\S+)", exports, re.MULTILINE))
    if names != EXPECTED:
        raise RuntimeError("Private Windows Debug export set does not match the Jai native signatures")
    fixture = ROOT / "tests/developer-tools/debug-windows-native.c"
    for optimization in [0, 2]:
        run([gcc, "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", fixture,
             import_library, "-o", ROOT / f"target/developer-tools-debug-windows-o{optimization}.exe"])
        run([gcc, "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
             ROOT / "tests/developer-tools/debug-windows-final.c", import_library,
             "-o", ROOT / f"target/developer-tools-debug-windows-final-o{optimization}.exe"])
        run([gcc, "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
             ROOT / "tests/developer-tools/debug-windows-dialog.c", import_library, "-luser32",
             "-o", ROOT / f"target/developer-tools-debug-windows-dialog-o{optimization}.exe"])
    run([llvm / "bin/clang", "--target=aarch64-w64-windows-gnu",
         f"--sysroot={sdk / 'x86_64-w64-mingw32'}", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
         "-c", source, "-o", ROOT / "target/developer-tools-debug-windows-arm64.obj"])
    build = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    # Check both public signatures and a future native behavior fixture. These
    # are deliberately not counted as Windows runtime evidence.
    for cpu in ["x64", "arm64"]:
        for name in ["debug-windows-contract", "debug-windows-lifecycle", "debug-windows-dumps"]:
            run([build / "debug/jaic", ROOT / f"tests/developer-tools/{name}.jai",
                 "-check", "-os", "windows", "-cpu", cpu])
        run([build / "debug/jailint", "--os", "windows", "--cpu", cpu, "-D", "warnings",
             ROOT / "stdlib/Debug/module.jai", ROOT / "tests/developer-tools/debug-windows-contract.jai",
             ROOT / "tests/developer-tools/debug-windows-lifecycle.jai", ROOT / "tests/developer-tools/debug-windows-dumps.jai"])
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check",
         ROOT / "stdlib/Debug/module.jai", ROOT / "stdlib/internal/developer-tools/debug-lifecycle.jai",
         ROOT / "tests/developer-tools/debug-windows-contract.jai",
         ROOT / "tests/developer-tools/debug-windows-lifecycle.jai", ROOT / "tests/developer-tools/debug-windows-dumps.jai"])
    headers = [sdk / "x86_64-w64-mingw32/include" / name for name in ["windows.h", "dbghelp.h"]]
    metadata = {"coverage": "Windows x64 SDK DLL/executable cross-build/link/export inspection and arm64 object compilation; no Windows execution",
                "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                "once_header_sha256": hashlib.sha256(source.with_name("once.h").read_bytes()).hexdigest(),
                "crash_state_header_sha256": hashlib.sha256(source.with_name("crash-state.h").read_bytes()).hexdigest(),
                "final_header_sha256": {name: hashlib.sha256(source.with_name(name).read_bytes()).hexdigest()
                                        for name in ["windows-command.h", "windows-launch.h", "windows-dumps.h"]},
                "fixture_sha256": hashlib.sha256(fixture.read_bytes()).hexdigest(),
                "sdk_header_sha256": {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in headers},
                "gcc_version": run([gcc, "--version"]).splitlines()[0],
                "exports": sorted(names), "runtime_status": "pending coordinator Windows native CI"}
    (ROOT / "docs/native/developer-tools-windows-debug-evidence.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print("Windows Debug x64 SDK DLL/import/export checks and arm64 compilation passed; native runtime evidence pending")


if __name__ == "__main__":
    main()
