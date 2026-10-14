#!/usr/bin/env python3
"""Compare licensed public C synthesis bases to the direct mathematical matrix.

This validates the permitted C algorithm adaptation, not a Jai compatibility API.
The C executable is built here from source; no supplied reference binary is run.
"""
import math
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
output = ROOT / "target/portable-tests/mpeg-matrix-check"
output.parent.mkdir(parents=True, exist_ok=True)
subprocess.run(["clang", "-std=c11", "-Wall", "-Wextra", "-Werror",
                str(ROOT / "tests/portable-stdlib/native/mpeg-matrix-check.c"),
                "-lm", "-o", str(output)], check=True)
raw = subprocess.check_output([str(output)], timeout=30)
assert len(raw) == 32 * 64 * 4
values = struct.unpack("=" + "f" * (32 * 64), raw)
worst = max(abs(values[k * 64 + i] - math.cos((16 + i) * (2 * k + 1) * math.pi / 64))
            for k in range(32) for i in range(64))
assert worst < 0.00001, worst
print(f"MPEG synthesis basis comparison passed: max absolute difference={worst:.9g}")
