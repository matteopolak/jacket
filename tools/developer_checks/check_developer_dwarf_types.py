#!/usr/bin/env python3
"""Original C DWARF type metadata drives public lookup; no Jai byte parser claim."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run(command):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, text=True,
                            capture_output=True, timeout=60)
    if result.returncode or result.stderr:
        raise RuntimeError(f"Original DWARF type fixture failed: {command}: {result.returncode}\n{result.stderr}")
    return result.stdout


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    generated = ["check_native_type_lookups :: () {"]
    evidence = {"scope": "original C x86/x64/arm64 O0/O2 DWARF4 metadata decoded by official SDK; host public model lookup, not a Jai byte parser or target execution", "fixtures": []}
    for architecture, target in [("x86", "i386-unknown-linux-gnu"), ("x64", "x86_64-unknown-linux-gnu"), ("arm64", "aarch64-unknown-linux-gnu")]:
        for optimization in [0, 2]:
            base = ROOT / f"target/developer-tools-dwarf-types-{architecture}-o{optimization}"
            run([prefix / "bin/clang", "-target", target, "-std=c11", "-gdwarf-4", "-g", f"-O{optimization}",
                 "-Wall", "-Wextra", "-Werror", "-c", ROOT / "stdlib/Debug_Info/tests/driven/dwarf-types-native.c", "-o", base.with_suffix(".o")])
            run([prefix / "bin/llvm-objcopy", "--dump-section", f".debug_info={base.with_suffix('.bin')}", base.with_suffix(".o")])
            text = run([prefix / "bin/llvm-dwarfdump", "--debug-info", base.with_suffix(".o")])
            records = []
            for match in re.finditer(r"^0x([0-9a-f]+):\s+DW_TAG_([a-z0-9_]+)\n(.*?)(?=^0x[0-9a-f]+:|\Z)", text, re.MULTILINE | re.DOTALL):
                offset, tag, attributes = match.groups()
                if not (tag.endswith("_type") or tag == "typedef"):
                    continue
                name = re.search(r'DW_AT_name\s+\("([^"\n]*)"\)', attributes)
                size = re.search(r"DW_AT_byte_size\s+\((0x[0-9a-f]+|[0-9]+)\)", attributes)
                encoding = re.search(r"DW_AT_encoding\s+\(DW_ATE_([a-z_]+)\)", attributes)
                records.append({"offset": int(offset, 16), "tag": tag.upper(), "name": name[1] if name else "",
                                "size": int(size[1], 0) if size else 0, "encoding": encoding[1].upper() if encoding else None})
            if len(records) < 5 or len({record["offset"] for record in records}) != len(records):
                raise RuntimeError("Original C fixture lacks independent unique DWARF type metadata")
            # Reverse table order to ensure the helper does not assume indexes or sorting.
            generated += ["    {", "        unit: D.Compilation_Unit;", "        defer array_free(unit.types);"]
            for record in reversed(records):
                encoding = "." + record["encoding"] if record["encoding"] else "cast(D.Dwarf_Base_Type) 0"
                generated.append(f"        array_add(*unit.types, D.Debug_Type.{{offset = {record['offset']}, tag = .{record['tag']}, name = {json.dumps(record['name'])}, size = {record['size']}, base_type = {encoding}}});")
            generated += [f"        for offset: 0..{max(record['offset'] for record in records) + 1} {{",
                          "            expected: *D.Debug_Type;", "            for *type: unit.types if type.offset == offset expected = type;",
                          "            actual := D.get_type(unit, cast(u32) offset);", "            assert(actual == expected);", "        }", "    }"]
            evidence["fixtures"].append({"architecture": architecture, "optimization": optimization, "types": records,
                                         "debug_info_sha256": hashlib.sha256(base.with_suffix('.bin').read_bytes()).hexdigest()})
    generated.append("}")
    destination = ROOT / "target/developer-tools-dwarf-type-oracle.jai"
    destination.write_text("\n".join(generated) + "\n")
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", destination])
    (ROOT / "target/developer-tools-dwarf-type-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
