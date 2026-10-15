#!/usr/bin/env python3
"""Produce original MSF fixtures and independent official-tool stream oracles."""
import os
import build_developer_bindings
from pathlib import Path
import re
import shutil
import subprocess
import struct

ROOT = Path(__file__).resolve().parents[1]


def run(command):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, text=True,
                            capture_output=True, timeout=60)
    if result.returncode or result.stderr:
        raise RuntimeError(f"MSF fixture command failed: {command}: {result.returncode}\n{result.stderr}")
    return result.stdout


def main():
    llvm = build_developer_bindings.llvm_prefix() / "bin"
    linker = os.environ.get("DEVELOPER_TOOLS_LLD_LINK") or shutil.which("lld-link")
    if not linker:
        raise RuntimeError("MSF native fixtures require official lld-link (DEVELOPER_TOOLS_LLD_LINK)")
    for architecture, target in [("x64", "x86_64-pc-windows-msvc"), ("arm64", "aarch64-pc-windows-msvc")]:
        base = ROOT / f"target/developer-tools-msf-{architecture}"
        run([llvm / "clang", "-target", target, "-g", "-gcodeview", "-O0", "-std=c11",
             "-Wall", "-Wextra", "-Werror", "-c", "tests/developer-tools/msf-native.c",
             "-o", base.with_suffix(".obj")])
        run([linker, "/nodefaultlib", "/entry:original_debug_entry", "/subsystem:console",
             "/debug", "/Brepro", f"/pdb:{base.with_suffix('.pdb')}",
             f"/out:{base.with_suffix('.exe')}", base.with_suffix(".obj")])
        metadata = run([llvm / "llvm-pdbutil", "dump", "-streams", base.with_suffix(".pdb")])
        streams = re.findall(r"Stream\s+(\d+)\s+\(\s*(\d+) bytes\)", metadata)
        if len(streams) < 5 or [int(index) for index, _ in streams] != list(range(len(streams))):
            raise RuntimeError("Official PDB stream metadata is incomplete")
        for index, size in streams:
            destination = ROOT / f"target/developer-tools-msf-{architecture}-stream-{index}.bin"
            run([llvm / "llvm-pdbutil", "export", f"--stream={index}", f"--out={destination}",
                 base.with_suffix(".pdb")])
            if destination.stat().st_size != int(size):
                raise RuntimeError("Official stream export size differs from metadata")
    # The compiler correctly prevents applications from loading stdlib internals.
    # Stage our NEW source for isolated private-unit evidence; no facade exports.
    shutil.copyfile(ROOT / "stdlib/internal/developer-tools/msf-wire.jai",
                    ROOT / "target/developer-tools-msf-wire.jai")
    # Original synthetic wire fixture: directory crosses a block boundary,
    # stream zero occupies reversed nonadjacent blocks, other streams are empty.
    block = 4096
    data = bytearray(block * 12)
    magic = b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\x00\x00\x00"
    data[:32] = magic
    directory = struct.pack("<I", 1024) + struct.pack("<1024I", 5000, *([0] * 1023)) + struct.pack("<2I", 10, 8)
    struct.pack_into("<6I", data, 32, block, 1, 12, len(directory), 0, 3)
    struct.pack_into("<2I", data, block * 3, 4, 7)
    data[block * 4:block * 5] = directory[:block]
    data[block * 7:block * 7 + len(directory) - block] = directory[block:]
    data[block:block * 2] = bytes([255]) * block
    for page in [0, 1, 2, 3, 4, 7, 8, 10]:
        data[block + page // 8] &= ~(1 << (page % 8))
    payload = bytes((index * 13 + 7) % 256 for index in range(5000))
    data[block * 10:block * 11] = payload[:block]
    data[block * 8:block * 8 + len(payload) - block] = payload[block:]
    (ROOT / "target/developer-tools-msf-synthetic.bin").write_bytes(data)


if __name__ == "__main__":
    main()
