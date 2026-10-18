#!/usr/bin/env python3
"""Write the CPUID words target/portable-machine/cpuid.bin that Machine_X64/tests/basics/main.jai
compares with the Jai reading: the output of the independent C program in
stdlib/Machine_X64/tests/native/cpuid-features.c, run on this machine. Only x86-64 hosts have
CPUID to read, so other hosts write nothing (the test is x64-only too)."""
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "stdlib/Machine_X64/tests/native/cpuid-features.c"


def main():
    if platform.machine().lower() not in ("x86_64", "amd64") or sys.platform == "win32":
        return 0
    output = ROOT / "target/portable-machine/cpuid.bin"
    output.parent.mkdir(parents=True, exist_ok=True)
    compiler = os.environ.get("CC", "cc")
    with tempfile.TemporaryDirectory() as scratch:
        executable = Path(scratch) / "cpuid-features"
        subprocess.run([compiler, "-std=c11", "-Wall", "-Wextra", "-Werror", str(SOURCE), "-o", str(executable)], check=True)
        words = subprocess.run([str(executable)], check=True, capture_output=True).stdout
    if len(words) != 48:
        raise RuntimeError(f"the CPUID oracle wrote {len(words)} bytes, expected 48")
    output.write_bytes(words)
    return 0


if __name__ == "__main__":
    sys.exit(main())
