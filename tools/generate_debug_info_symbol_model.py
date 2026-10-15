#!/usr/bin/env python3
"""Generate the explicitly reviewed symbol base records and annotation model."""
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    spec = (ROOT / "docs/specs/stdlib/modules/debug-info.md").read_text()
    output = ["// Reviewed Jai aggregates; serialized payloads are decoded separately."]
    names = {"Codeview_Symbol_Compile": "COMPILE", "Codeview_Symbol_Compile3": "COMPILE3",
             "Codeview_Symbol_Procedure": "GPROC32", "Codeview_Symbol_Inline_Site": "INLINESITE"}
    for name, tag in names.items():
        section = spec.split(f"#### `{name}` (struct)", 1)[1].split("#### ", 1)[0]
        fields = re.findall(r"^- `([^`]+)`", section, re.M)
        if not any(field.startswith("base:") for field in fields):
            raise RuntimeError(f"Reviewed common base is absent: {name}")
        output += [f"{name} :: struct {{", f"    using base: Codeview_Symbol = .{{symbol_type = .{tag}}};"]
        output += ["    " + field + ("" if field.endswith("}") else ";") for field in fields if not field.startswith("base:")]
        output += ["}", ""]
    for name in ["Codeview_Binary_Annotation", "Codeview_Parser_State", "Codeview_Type_Parser_State", "Codeview_Symbol_Parser_State"]:
        section = spec.split(f"#### `{name}` (struct)", 1)[1].split("#### ", 1)[0]
        output += [f"{name} :: struct {{"]
        output += ["    " + field + ("" if field.endswith("}") else ";") for field in re.findall(r"^- `([^`]+)`", section, re.M)]
        output += ["}", ""]
    for name, width in [("Codeview_Binary_Annotation_Opcode", "u8"), ("Codeview_Symbol_Section", "u32"), ("Codeview_Inlinee_Source_Variant", "u32")]:
        section = spec.split(f"#### `{name}` (enum)", 1)[1].split("#### ", 1)[0]
        output += [f"{name} :: enum {width} {{"]
        output += ["    " + field + ";" for field in re.findall(r"^- `([^`]+)`", section, re.M)]
        output += ["}", ""]
    path = ROOT / "stdlib/Debug_Info/codeview-symbol-model.jai"
    path.write_text("\n".join(output))
    subprocess.run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", path], cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
