#!/usr/bin/env python3
"""Stage genuine Meow vectors produced by our licensed native C source build.

--verify-native rebuilds that independent comparison and requires byte-identical
results. No supplied reference compiler/binary is run. Other hosts can use the
committed original test data without an x64 execution dependency.
"""
import argparse
import hashlib
from pathlib import Path
import platform
import shutil
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--verify-native", action="store_true")
args = parser.parse_args()
header = ROOT / "third_party/meow_hash/meow_hash_x64_aesni.h"
assert hashlib.sha256(header.read_bytes()).hexdigest() == "2328557212adc48ee3e8e8c9d25820a43b2fc0c48cf4296ab9b0b1ba52d376e2"
fixture = ROOT / "tests/portable-stdlib/fixtures/meow-vectors.bin"
data = fixture.read_bytes()
assert hashlib.sha256(data).hexdigest() == "b4b8af0a8629e1cdc1a9853486158126e7e5756e72a062682e4eb354765ff639"
count, = struct.unpack_from("<I", data, 4)
assert data[:4] == b"MCV5" and count == 317 and len(data) == 136 + count * 304
output = ROOT / "target/portable-tests/meow-fixtures"
output.mkdir(parents=True, exist_ok=True)
if args.verify_native:
    command = ["clang++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-O2"]
    if platform.system() == "Darwin":
        command += ["-arch", "x86_64"]
    elif platform.machine().lower() not in {"x86_64", "amd64"}:
        parser.error("native AES-NI comparison needs x64 or macOS x64 execution support")
    executable = output / "native-c-vectors"
    command += ["-maes", "-msse4.1", "-mssse3", str(ROOT / "tests/portable-stdlib/native/meow-c-check.cpp"), "-o", str(executable)]
    subprocess.run(command, check=True)
    actual = subprocess.check_output([str(executable)], timeout=45)
    assert actual == data, "native C comparison differs from the reviewed fixture"
    print("Source-built native C reproduced 317 Meow vectors byte-identically")
shutil.copyfile(fixture, output / "vectors.bin")
print("Staged 317 original Meow vectors with pinned C/data digests")
