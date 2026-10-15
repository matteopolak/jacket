#!/usr/bin/env python3
"""Emit only the refreshed reviewed CodeView type aggregate contract."""
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
TAGS = {
    "Builtin": "BUILTIN", "Struct": "STRUCTURE", "Enum": "ENUM", "Array": "ARRAY",
    "Pointer": "POINTER", "Procedure": "PROCEDURE", "Method_Function": "MFUNCTION",
    "Overloaded_Method": "METHOD", "Method_List": "METHODLIST", "Method": "ONEMETHOD",
    "List": "FIELDLIST", "Arguments": "ARGLIST", "String_Id": "STRING_ID",
    "Udt_Source_Line": "UDT_SRC_LINE", "Function_Id": "FUNC_ID", "Alias": "ALIAS",
    "Typeserver2": "TYPESERVER2", "Modifier": "MODIFIER", "Member": "MEMBER",
    "Enumerate": "ENUMERATE", "Nested_Type": "NESTTYPE",
}


def main():
    spec = (ROOT / "docs/specs/stdlib/modules/debug-info.md").read_text()
    output = ["// Canonical reviewed CodeView type records, including common using base storage."]
    for suffix, tag in TAGS.items():
        name = "Codeview_" + suffix
        section = spec.split(f"#### `{name}` (struct)", 1)[1].split("#### ", 1)[0]
        fields = re.findall(r"^- `([^`]+)`", section, re.M)
        output += [f"{name} :: struct {{", f"    using base: Codeview_Type = .{{kind = .{tag}}};"]
        for field in fields:
            if field.startswith("base:"):
                continue
            output.append("    " + field + ("" if field.endswith("}") else ";"))
        output += ["}", ""]
    for name, kind, width in [("Struct_Properties", "enum_flags", "u16"),
                              ("Function_Attributes", "enum_flags", "u8"),
                              ("Codeview_Access", "enum", "u16"),
                              ("Codeview_Builtin_Type", "enum", ""),
                              ("Codeview_Pointer_Type", "enum", "u8")]:
        section = spec.split(f"#### `{name}` (", 1)[1].split("#### ", 1)[0]
        fields = re.findall(r"^- `([^`]+)`", section, re.M)
        output += [f"{name} :: {kind} {width} {{"]
        output += ["    " + field + ";" for field in fields]
        output += ["}", ""]
    section = spec.split("#### `Guid` (struct)", 1)[1].split("#### ", 1)[0]
    output += ["Guid :: struct {"]
    output += ["    " + field + ";" for field in re.findall(r"^- `([^`]+)`", section, re.M)]
    output += ["}", ""]
    (ROOT / "stdlib/Debug_Info/codeview-type-model.jai").write_text("\n".join(output))
    output = ["// Reviewed public PDB aggregates; contextual map storage is independently designed."]
    for name in ["Pdb", "Pdb_Header", "Tpi_Stream_Header"]:
        section = spec.split(f"#### `{name}` (struct)", 1)[1].split("#### ", 1)[0]
        output += [f"{name} :: struct {{"]
        output += ["    " + field + ";" for field in re.findall(r"^- `([^`]+)`", section, re.M)]
        output += ["}", ""]
    for name in ["Pdb_Version", "Tpi_Stream_Version"]:
        section = spec.split(f"#### `{name}` (enum)", 1)[1].split("#### ", 1)[0]
        output += [f"{name} :: enum u32 {{"]
        output += ["    " + field + ";" for field in re.findall(r"^- `([^`]+)`", section, re.M)]
        output += ["}", ""]
    output += ["#scope_file", "", "GUID :: struct { Data1: u32; Data2: u16; Data3: u16; Data4: [8] u8; }",
               "String_Offset :: #type,distinct s32;", "Name_Index :: #type,distinct u32;",
               "Name_Table :: struct { buffer: [] u8; string_to_name_map: Pdb_Map(String_Offset, Name_Index); }",
               "Pdb_Map :: struct(K: Type, V: Type) { keys: [..] K; values: [..] V; present: Int_Set; deleted: Int_Set; }", ""]
    (ROOT / "stdlib/Debug_Info/pdb-model.jai").write_text("\n".join(output))
    subprocess.run([str(ROOT / "target/jaifmt"), "--config", str(ROOT / "jaifmt.toml"),
                    str(ROOT / "stdlib/Debug_Info/codeview-type-model.jai"),
                    str(ROOT / "stdlib/Debug_Info/pdb-model.jai")], cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
