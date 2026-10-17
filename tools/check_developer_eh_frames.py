#!/usr/bin/env python3
"""Link original C objects with official LLD to obtain relocated eh_frame data."""
import hashlib
import json
import os
import build_developer_bindings
from pathlib import Path
import re
import shutil
from check_developer_dwarf_lines import run

ROOT = Path(__file__).resolve().parents[1]


def main():
    prefix = build_developer_bindings.llvm_prefix()
    linker = os.environ.get("DEVELOPER_TOOLS_LD_LLD") or shutil.which("ld.lld")
    if not linker:
        raise RuntimeError("Relocated eh_frame fixtures require official ld.lld (DEVELOPER_TOOLS_LD_LLD)")
    declaration = "Native_Eh_Frame :: struct { filename: string; section_address: u64; fde_offset: s64; fde_end: s64; fde_length: u64; cie_offset: u64; pc_begin: u64; pc_range: u64; instruction_types: string; augmentation: string; encoding: u8; }"
    source = [declaration, "native_eh_frames :: Native_Eh_Frame.["]
    spec = (ROOT / "docs/specs/stdlib/modules/debug-info.md").read_text().split("#### `Call_Frame_Address_Instruction_Type`", 1)[1].split("#### ", 1)[0]
    opcodes = {name: int(value, 0) for name, value in re.findall(r"^- `([A-Z_0-9]+)\s*::\s*(0x[0-9a-fA-F]+|[0-9]+)`", spec, re.MULTILINE)}
    evidence = {"scope": "original C Linux ELF objects linked by official LLD; relocated eh_frame decoded on host; no ELF program executed", "fixtures": []}
    for arch, triple in [("x86", "i386-unknown-linux-gnu"), ("x64", "x86_64-unknown-linux-gnu"), ("arm64", "aarch64-unknown-linux-gnu")]:
        for optimization in [0, 2]:
            obj = ROOT / f"target/developer-tools-eh-{arch}-o{optimization}.o"
            run([prefix / "bin/clang", "-target", triple, "-gdwarf-4", "-g", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
                 "-c", ROOT / "stdlib/Debug_Info/tests/driven/dwarf-line-native.c", "-o", obj])
            for placement in ["default", "negative"]:
                base = ROOT / f"target/developer-tools-eh-{arch}-o{optimization}-{placement}"
                args = [linker, "--entry=original_line_query", "-o", base.with_suffix('.elf'), obj]
                if placement == "negative":
                    args += ["--image-base=0x10000", "--section-start=.text=0x20000", "--section-start=.eh_frame=0x40000"]
                run(args)
                sections = json.loads(run([prefix / "bin/llvm-readobj", "--sections", "--elf-output-style=JSON", base.with_suffix('.elf')]))[0]["Sections"]
                selected = [entry["Section"] for entry in sections if entry["Section"]["Name"]["Name"] == ".eh_frame"]
                if len(selected) != 1:
                    raise RuntimeError("Original linked object has no unique eh_frame section")
                section_address = selected[0]["Address"]
                run([prefix / "bin/llvm-objcopy", "--dump-section", f".eh_frame={base.with_suffix('.bin')}", base.with_suffix('.elf')])
                decoded = run([prefix / "bin/llvm-dwarfdump", "--eh-frame", base.with_suffix('.elf')])
                matches = re.findall(r"^([0-9a-f]+) ([0-9a-f]+) [0-9a-f]+ FDE cie=([0-9a-f]+) pc=([0-9a-f]+)\.\.\.([0-9a-f]+)$", decoded, re.MULTILINE)
                if len(matches) != 1 or 'Augmentation:          "zR"' not in decoded or "Version:               1" not in decoded:
                    raise RuntimeError("Original native fixture requires a single version1 zR FDE")
                offset, length, cie, begin, end = (int(value, 16) for value in matches[0])
                names = re.findall(r"^\s+DW_CFA_([A-Za-z_0-9]+)", decoded[decoded.index("FDE cie="):], re.MULTILINE)
                if any(name.upper() not in opcodes for name in names):
                    raise RuntimeError(f"Unreviewed native CFA opcode: {names}")
                kinds = "".join(f"\\x{opcodes[name.upper()]:02x}" for name in names)
                source.append(f'    .{{filename = "{base.with_suffix(".bin").name}", section_address = {section_address}, fde_offset = {offset}, fde_end = {offset + 4 + length}, fde_length = {length}, cie_offset = {cie}, pc_begin = {begin}, pc_range = {end - begin}, instruction_types = "{kinds}", augmentation = "zR", encoding = 27}},')
                evidence["fixtures"].append({"target": triple, "optimization": optimization, "placement": placement,
                                             "section_address": section_address, "fde_offset": offset, "length": length, "cie_offset": cie,
                                             "pc_begin": begin, "pc_range": end - begin, "instruction_types": names,
                                             "bytes_sha256": hashlib.sha256(base.with_suffix('.bin').read_bytes()).hexdigest()})
    for optimization in [0, 2]:
        base = ROOT / f"target/developer-tools-eh-absolute-o{optimization}"
        run([prefix / "bin/clang", "-target", "x86_64-unknown-linux-gnu", "-std=c11", f"-O{optimization}", "-fno-asynchronous-unwind-tables",
             "-fno-unwind-tables", "-Wall", "-Wextra", "-Werror", "-c", ROOT / "stdlib/Debug_Info/tests/driven/eh-absolute-native.c", "-o", base.with_suffix('.o')])
        run([linker, "--entry=original_absolute_frame", "-o", base.with_suffix('.elf'), base.with_suffix('.o')])
        sections = json.loads(run([prefix / "bin/llvm-readobj", "--sections", "--elf-output-style=JSON", base.with_suffix('.elf')]))[0]["Sections"]
        section_address = next(entry["Section"]["Address"] for entry in sections if entry["Section"]["Name"]["Name"] == ".eh_frame")
        run([prefix / "bin/llvm-objcopy", "--dump-section", f".eh_frame={base.with_suffix('.bin')}", base.with_suffix('.elf')])
        decoded = run([prefix / "bin/llvm-dwarfdump", "--eh-frame", base.with_suffix('.elf')])
        common = {}
        for match in re.finditer(r"^([0-9a-f]+) [0-9a-f]+ 00000000 CIE\n(.*?)(?=^[0-9a-f]+ |\Z)", decoded, re.MULTILINE | re.DOTALL):
            augmentation = re.search(r'Augmentation:\s+"([^"]*)"', match[2])
            if not augmentation or augmentation[1] not in ["", "zR"]:
                raise RuntimeError("Unexpected native absolute CIE augmentation")
            common[int(match[1], 16)] = augmentation[1]
        matches = list(re.finditer(r"^([0-9a-f]+) ([0-9a-f]+) [0-9a-f]+ FDE cie=([0-9a-f]+) pc=([0-9a-f]+)\.\.\.([0-9a-f]+)\n(.*?)(?=^[0-9a-f]+ |\Z)", decoded, re.MULTILINE | re.DOTALL))
        if len(matches) != 2 or len(common) != 2:
            raise RuntimeError("Original absolute/default fixture lost its two native frame forms")
        for match in matches:
            offset, length, cie, begin, end = (int(value, 16) for value in match.groups()[:5])
            names = re.findall(r"^\s+DW_CFA_([A-Za-z_0-9]+)", match[6], re.MULTILINE)
            kinds = "".join(f"\\x{opcodes[name.upper()]:02x}" for name in names)
            augmentation = common[cie]
            source.append(f'    .{{filename = "{base.with_suffix(".bin").name}", section_address = {section_address}, fde_offset = {offset}, fde_end = {offset + 4 + length}, fde_length = {length}, cie_offset = {cie}, pc_begin = {begin}, pc_range = {end - begin}, instruction_types = "{kinds}", augmentation = "{augmentation}", encoding = 0}},')
            evidence["fixtures"].append({"target": "x86_64-unknown-linux-gnu", "optimization": optimization, "placement": "absolute/default original C inline-assembly standard records",
                                         "section_address": section_address, "fde_offset": offset, "length": length, "cie_offset": cie, "pc_begin": begin,
                                         "pc_range": end - begin, "instruction_types": names, "augmentation": augmentation, "encoding": 0,
                                         "bytes_sha256": hashlib.sha256(base.with_suffix('.bin').read_bytes()).hexdigest()})
    source.append("];")
    path = ROOT / "target/developer-tools-eh-frame-oracle.jai"
    path.write_text("\n".join(source) + "\n")
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", path])
    (ROOT / "target/developer-tools-eh-frames-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
