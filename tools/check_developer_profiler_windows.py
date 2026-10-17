#!/usr/bin/env python3
"""Verify canonical Windows clock SDK ABI/import links; never claim target execution."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run(command):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, text=True, capture_output=True, timeout=60)
    if result.returncode or result.stderr:
        raise RuntimeError(f"Windows profiling SDK check failed: {result.returncode}\n{result.stderr}")
    return result.stdout


def main():
    sdk = Path(os.environ.get("DEVELOPER_TOOLS_MINGW_PREFIX", "/opt/homebrew/opt/mingw-w64/toolchain-x86_64"))
    llvm = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    fixture = ROOT / "tests/developer-tools/iprof-windows-clock.c"
    gcc = sdk / "bin/x86_64-w64-mingw32-gcc"
    evidence = {"scope": "original Windows x64 C O0/O2 SDK ABI/executable/import-link checks and arm64 object; no Windows execution or Jai native timing claim", "imports": []}
    expected = {"QueryPerformanceCounter", "QueryPerformanceFrequency", "Sleep"}
    for optimization in [0, 2]:
        executable = ROOT / f"target/developer-tools-iprof-windows-clock-o{optimization}.exe"
        run([gcc, "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", fixture, "-lkernel32", "-o", executable])
        metadata = run([llvm / "bin/llvm-readobj", "--coff-imports", executable])
        imports = set(re.findall(r"Symbol: (\S+)", metadata))
        if not expected <= imports:
            raise RuntimeError("Profiler clock executable lacks required public SDK imports")
        evidence["imports"].append({"optimization": optimization, "verified": sorted(expected & imports)})
    run([llvm / "bin/clang", "--target=aarch64-w64-windows-gnu", f"--sysroot={sdk / 'x86_64-w64-mingw32'}",
         "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-c", fixture,
         "-o", ROOT / "target/developer-tools-iprof-windows-clock-arm64.obj"])
    evidence["fixture_sha256"] = hashlib.sha256(fixture.read_bytes()).hexdigest()
    evidence["sdk_header_sha256"] = {name: hashlib.sha256((sdk / "x86_64-w64-mingw32/include" / name).read_bytes()).hexdigest()
                                      for name in ["windows.h", "profileapi.h", "winnt.h"]}
    (ROOT / "target/developer-tools-profiler-windows-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
    print("Windows profiler SDK ABI/import links passed; native timing execution pending")


if __name__ == "__main__":
    main()
