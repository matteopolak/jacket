#!/usr/bin/env python3
"""Emit compiler ABI declarations solely from the reviewed public specification.

The two report types awaiting a reviewed run-record shape are deliberately omitted.
No source distribution or old library implementation is an input.
"""
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/specs/stdlib/modules/compiler.md"
OUT = ROOT / "stdlib/Compiler/contract-records.jai"
NESTED = {
    "Code_Node": "Location :: struct { enclosing_load: *Message_File; l0, c0, l1, c1: s32; }",
    "Code_Procedure_Call": "Flags :: enum_flags u32 { INLINE_YES :: 0x1; INLINE_NO :: 0x2; RETURNS_PROCEDURE_POINTER_ONLY :: 0x10; NO_DEBUG :: 0x100; IS_MODULE_PARAMETERS :: 0x200; } Context_Modification :: struct { modification_expressions: [] *Code_Node; }",
    "Code_Return": "Flags :: enum_flags u32 { IS_BACKTICKED :: 0x01; AUTO_INSERTED_FOR_QUICK_LAMBDA :: 0x02; }",
    "Code_Binary_Operator": "Flags :: enum_flags u16 { SHIFT_MARKED_AS_LOGICAL :: 0x01; SHIFT_MARKED_AS_SMALL :: 0x02; }",
    "Polymorph_Report": "Record :: struct { source: *Code_Procedure_Header; num_call_sites: s64; num_deduplications_by_constants: s64; num_deduplications_by_bytecode: s64; distinct_polymorphs: [] Distinct_Polymorph; } Distinct_Polymorph :: struct { polymorphed: *Code_Procedure_Header; call_site: *Code_Node; location: Source_Code_Location; }",
}


def unescape(text):
    return re.sub(r"\\\s*_", "_", text)


def declaration_order(item):
    """Independent topical ordering; public member/layout order stays untouched."""
    name, _ = item
    if name.startswith(("Data_", "Global_", "Runtime_", "Type_Info_")):
        group = 0  # Runtime storage and reflection.
    elif name.startswith(("Build_", "Llvm_", "X64_", "Optimization_", "Version_")):
        group = 1  # Target configuration.
    elif name.startswith(("Message", "Intercept_", "Workspace_", "Provided_", "Report")):
        group = 2  # Workspace lifecycle and events.
    elif name.startswith(("Code_", "Operator_", "Get_Root_")):
        group = 3  # Syntax/reflection records.
    else:
        group = 4  # Measurement and ancillary records.
    return group, name


def main():
    source = SPEC.read_text()
    inventory = source.split("## Public type members")[0]
    declarations = dict(re.findall(r"^- `([A-Za-z_][A-Za-z_0-9]*) :: ((?:struct|enum|enum_flags)[^`]*)`$", inventory, re.M))
    chunks = ["// Generated from docs/specs/stdlib/modules/compiler.md; edit the spec/generator.\n"]
    registered = []
    sections = re.findall(r"^#### `([^`]+)`[^\n]*\n(.*?)(?=^#### |^## |\Z)", source, re.M | re.S)
    for name, body in sorted(sections, key=declaration_order):
        if name in {"Run_Directive_Report", "Message_Performance_Report"}:
            continue
        if name not in declarations:
            raise ValueError(f"no inventory declaration for {name}")
        if declarations[name] == "struct" and (name.startswith(("Code_", "Message")) or name in {"Time_Report", "Polymorph_Report", "Bytecode_Report", "Global_Data_Info", "Global_Data_Segment_Info"}):
            registered.append("C." + name)
        blocks = re.findall(r"```jai\n(.*?)```", body, re.S)
        if blocks:
            chunks.append(unescape(blocks[0]).strip() + "\n")
            continue
        header = declarations[name]
        fields = re.findall(r"^- `(.*)`$", body, re.M)
        if not fields:
            raise ValueError(f"missing reviewed shape for {name}")
        lines = [f"{name} :: {header} {{"]
        if name in NESTED:
            lines.append("    " + NESTED[name])
        for field in fields:
            field = unescape(field).strip()
            lines.append("    " + field.rstrip(";") + ";")
        lines.append("}\n")
        chunks.append("\n".join(lines))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n\n".join(chunks))
    registry = ROOT / "stdlib/internal/compiler-api/registry.jai"
    registry.parent.mkdir(parents=True, exist_ok=True)
    registered += ["Type_Info", "Type_Info_Integer", "Type_Info_Float", "Type_Info_String", "Type_Info_Pointer", "Type_Info_Procedure", "Type_Info_Struct", "Type_Info_Struct_Member", "Type_Info_Array", "Type_Info_Enum", "Type_Info_Variant", "Source_Code_Location"]
    registry.write_text("// Generated from reviewed compiler/prelude record inventories.\n#scope_module\nregister_compiler_types :: () {\n" + "".join("    register_type(type_info(" + name + "));\n" for name in registered) + "}\n#scope_file\nC :: #import \"Compiler\";\n")
    print(OUT)


if __name__ == "__main__":
    main()
