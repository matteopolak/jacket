#!/usr/bin/env python3
"""Generate required public DWARF enum data only from the reviewed inventory."""
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    spec = (ROOT / "docs/specs/stdlib/modules/debug-info.md").read_text()
    output = ["// Public DWARF values from the reviewed spec; independently grouped by role.", ""]
    for name, storage in [("Dwarf_Base_Type", ""), ("Dwarf_Language", " u16"), ("Dwarf_Tag", "")]:
        section = spec.split(f"#### `{name}` (enum)", 1)[1].split("#### ", 1)[0]
        members = re.findall(r"^- `([A-Z_0-9]+)\s*::\s*(0x[0-9a-fA-F]+|[0-9]+)`", section, re.MULTILINE)
        if not members or len({name for name, _ in members}) != len(members):
            raise RuntimeError(f"Incomplete or duplicate reviewed {name} members")
        output.append(f"{name} :: enum{storage} {{")
        output.extend(f"    {member} :: {value};" for member, value in sorted(members))
        output.extend(["}", ""])
    destination = ROOT / "stdlib/Debug_Info/dwarf-values.jai"
    destination.write_text("\n".join(output))
    subprocess.run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", destination], check=True)


if __name__ == "__main__":
    main()
