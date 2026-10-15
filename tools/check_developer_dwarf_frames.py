#!/usr/bin/env python3
"""Produce original DWARF4 debug_frame bytes and native SDK FDE range oracles."""
import hashlib
import json
import os
import build_developer_bindings
from pathlib import Path
import re
from check_developer_dwarf_lines import run

ROOT = Path(__file__).resolve().parents[1]


def main():
    prefix = build_developer_bindings.llvm_prefix()
    source = ["Native_Frame :: struct { filename: string; fde_offset: s64; fde_end: s64; fde_length: u64; pc_begin: u64; pc_range: u64; instruction_types: string; }", "native_frames :: Native_Frame.["]
    spec = (ROOT / "docs/specs/stdlib/modules/debug-info.md").read_text().split("#### `Call_Frame_Address_Instruction_Type`", 1)[1].split("#### ", 1)[0]
    opcodes = {name: int(value, 0) for name, value in re.findall(r"^- `([A-Z_0-9]+)\s*::\s*(0x[0-9a-f]+|[0-9]+)`", spec, re.MULTILINE)}
    evidence = {"scope": "original x86/x64/arm64 Linux-target DWARF32 version4 debug_frame bytes, host decoder; no target executable run", "fixtures": []}
    for arch, target in [("x64", "x86_64-unknown-linux-gnu"), ("arm64", "aarch64-unknown-linux-gnu"), ("x86", "i386-unknown-linux-gnu")]:
        for optimization in [0, 2]:
            base = ROOT / f"target/developer-tools-debug-frame-{arch}-o{optimization}"
            run([prefix / "bin/clang", "-target", target, "-gdwarf-4", "-g", "-fno-asynchronous-unwind-tables", "-fno-unwind-tables",
                 f"-O{optimization}", "-Wall", "-Wextra", "-Werror", "-c", ROOT / "tests/developer-tools/dwarf-line-native.c", "-o", base.with_suffix('.o')])
            run([prefix / "bin/llvm-objcopy", "--dump-section", f".debug_frame={base.with_suffix('.bin')}", base.with_suffix('.o')])
            decoded = run([prefix / "bin/llvm-dwarfdump", "--debug-frame", base.with_suffix('.o')])
            matches = re.findall(r"^([0-9a-f]+) ([0-9a-f]+) [0-9a-f]+ FDE cie=([0-9a-f]+) pc=([0-9a-f]+)\.\.\.([0-9a-f]+)$", decoded, re.MULTILINE)
            address_size = re.search(r"Address size:\s+([48])", decoded)
            if len(matches) != 1 or "Version:               4" not in decoded or not address_size:
                raise RuntimeError("Original fixture did not emit one explicit-width v4 debug_frame FDE")
            offset, length, cie, begin, end = (int(value, 16) for value in matches[0])
            if cie != 0 or end <= begin:
                raise RuntimeError("Original native frame fixture has unexpected CIE/range")
            names = re.findall(r"^\s+DW_CFA_([A-Za-z_0-9]+)", decoded[decoded.index("FDE cie="):], re.MULTILINE)
            if any(name.upper() not in opcodes for name in names):
                raise RuntimeError(f"Native FDE has unreviewed opcode names: {names}")
            kinds = "".join(f"\\x{opcodes[name.upper()]:02x}" for name in names)
            source.append(f'    .{{filename = "{base.with_suffix(".bin").name}", fde_offset = {offset}, fde_end = {offset + 4 + length}, fde_length = {length}, pc_begin = {begin}, pc_range = {end - begin}, instruction_types = "{kinds}"}},')
            evidence["fixtures"].append({"target": target, "optimization": optimization, "fde_offset": offset, "length": length,
                                         "address_size": int(address_size[1]),
                                         "cie_offset": cie, "pc_begin": begin, "pc_range": end - begin,
                                         "instruction_types": names,
                                         "bytes_sha256": hashlib.sha256(base.with_suffix('.bin').read_bytes()).hexdigest()})
    source.append("];")
    path = ROOT / "target/developer-tools-dwarf-frame-oracle.jai"
    path.write_text("\n".join(source) + "\n")
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", path])
    (ROOT / "docs/native/developer-tools-dwarf-frames-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
