"""Transcribe explicit reviewed declarations, never implementation/reference source.

Records provided by the other private executable files are excluded here.
"""
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/specs/stdlib/modules/executable-formats.md").read_text()
inventory = dict(re.findall(r"^- `([A-Za-z_0-9]+)\s*::\s*([^`]+)`$", SPEC.split("## Public type members")[0], re.M))
blocks = re.findall(r"^#### `([^`]+)` \(([^)]+)\)\n(.*?)(?=^#### |^## |\Z)", SPEC, re.M | re.S)
withheld = {"Archive", "Archive_File", "Elf_Program_Header",
            "Coff_Raw_Codeview_Info", "Coff_Codeview_Info", "Coff_Optional_Header", "Coff"}
out = ["// Explicit public ABI transcribed from the reviewed executable-formats spec.\n"]
written = set()
for name, declaration in inventory.items():
    if declaration.startswith(("struct", "enum")) or "->" in declaration or declaration.startswith("("):
        continue
    out.append(f"{name} :: {declaration};\n")
    written.add(name)
for name, kind, body in blocks:
    if name in withheld:
        continue
    declaration = inventory.get(name, kind)
    members = re.findall(r"^- `([^`]+)`", body, re.M)
    if kind.startswith("enum"):
        declaration = re.sub(r"\s+s\d+$", lambda m: m[0], declaration)
    if name in {"Coff_Symbol", "Coff_Relocation"}:
        members = [m.replace("zeroes: u32; offset: u32;", "zeroes: u32 #align 1; offset: u32 #align 1;") for m in members]
        members = [m.rstrip("; ") + " #align 1" for m in members]
    out.append(f"\n{name} :: {declaration} {{\n")
    for member in members:
        member = member.strip().rstrip(";")
        if name == "Coff_Optional_Header_Pe32_Plus" and member.startswith("Magic ::"):
            pass
        if name == "Cpu_Type":
            member = member.replace("xx CPU_ARCH.", "cast(s32) CPU_ARCH.")
        if name == "Cpu_Subtype":
            member = member.replace("0xff000000", "-16777216").replace("0x80000000", "-2147483648")
        out.append(f"    {member};\n")
    out.append("}\n")
    written.add(name)
out.append('''
Elf_Program_Header :: struct(Size_Type: Type) {
    type: Elf_Segment_Type;
    #if Size_Type == u64 { flags: Elf_Segment_Flags; }
    offset: Size_Type;
    vaddr: Size_Type;
    paddr: Size_Type;
    filesz: Size_Type;
    memsz: Size_Type;
    #if Size_Type == u32 { flags: Elf_Segment_Flags; }
    align: Size_Type;
}
''')
written.add("Elf_Program_Header")
(ROOT / "stdlib/internal/executable/abi.jai").write_text("".join(out))
(ROOT / "target").mkdir(exist_ok=True)
(ROOT / "target/executable-abi-names.txt").write_text("\n".join(sorted(written)))

# Explicit reviewed storage-word views. Setters truncate to their field width.
views = {
    "dylib_reference": [("isym", 0, 24, False), ("flags", 24, 8, False)],
    "twolevel_hint": [("isub_image", 0, 8, False), ("itoc", 8, 24, False)],
    "relocation_info": [("r_symbolnum", 0, 24, False), ("r_pcrel", 24, 1, True),
                        ("r_length", 25, 2, False), ("r_extern", 27, 1, True), ("r_type", 28, 4, False)],
    "scattered_relocation_info": [("r_address", 0, 24, False), ("r_type", 24, 4, False),
                                  ("r_length", 28, 2, False), ("r_pcrel", 30, 1, True),
                                  ("r_scattered", 31, 1, True)],
}
out = ["// Storage masks are part of the reviewed public contract.\n"]
for record, fields in views.items():
    for name, shift, width, boolean in fields:
        mask = (1 << width) - 1
        storage_mask = mask << shift
        value_type = "bool" if boolean else "u32"
        out.append(f"get_{name} :: (fields: {record}) -> {value_type} {{\n")
        expression = f"((fields.__bitfield >> {shift}) & 0x{mask:x})"
        out.append(f"    return {expression}" + (" != 0" if boolean else "") + ";\n}\n\n")
        out.append(f"set_{name} :: (fields: *{record}, value: {value_type}) {{\n")
        out.append("    if !fields return;\n")
        out.append("    lane := " + ("cast(u32) " if boolean else "") + "value;\n")
        out.append(f"    fields.__bitfield = (fields.__bitfield & 0x{0xffffffff ^ storage_mask:x}) | ((lane & 0x{mask:x}) << {shift});\n}}\n\n")
(ROOT / "stdlib/internal/executable/bitfields.jai").write_text("".join(out))
formatter = ROOT / "target/jaifmt"
if formatter.is_file():
    subprocess.run([str(formatter), str(ROOT / "stdlib/internal/executable/abi.jai"),
                    str(ROOT / "stdlib/internal/executable/bitfields.jai")], check=True)
