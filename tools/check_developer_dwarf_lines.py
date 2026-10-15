#!/usr/bin/env python3
"""Original native DWARF4 line fixtures and independently decoded SDK row oracles."""
import hashlib
import json
import os
import build_developer_bindings
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run(command):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, text=True,
                            capture_output=True, timeout=60)
    if result.returncode or result.stderr:
        raise RuntimeError(f"DWARF fixture command failed: {command}: {result.returncode}\n{result.stderr}")
    return result.stdout


def main():
    prefix = build_developer_bindings.llvm_prefix()
    generated = ["check_native_line_queries :: () {"]
    evidence = {"scope": "original Linux-target DWARF4 objects decoded by host SDK; no target executable run", "fixtures": []}
    for architecture, target in [("x86", "i386-unknown-linux-gnu"), ("x64", "x86_64-unknown-linux-gnu"), ("arm64", "aarch64-unknown-linux-gnu")]:
        for optimization in [0, 2]:
            base = ROOT / f"target/developer-tools-dwarf-line-{architecture}-o{optimization}"
            run([prefix / "bin/clang", "-target", target, "-gdwarf-4", "-g", f"-O{optimization}",
                 "-Wall", "-Wextra", "-Werror", "-c", ROOT / "tests/developer-tools/dwarf-line-native.c",
                 "-o", base.with_suffix(".o")])
            run([prefix / "bin/llvm-objcopy", "--dump-section", f".debug_line={base.with_suffix('.bin')}", base.with_suffix(".o")])
            text = run([prefix / "bin/llvm-dwarfdump", "--debug-line", base.with_suffix(".o")])
            rows = []
            for match in re.finditer(r"^0x([0-9a-f]+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)([^\n]*)$", text, re.MULTILINE):
                address, line, column, file_id, isa, discriminator, op_index, flags = match.groups()
                if int(isa) != 0 or int(op_index) != 0:
                    raise RuntimeError("Original fixture unexpectedly requires unsupported ISA/op-index projection")
                rows.append({"address": int(address, 16), "line": int(line), "column": int(column),
                             "file_id": int(file_id), "flags": flags.split(), "discriminator": int(discriminator)})
            if len(rows) < 3 or "end_sequence" not in rows[-1]["flags"]:
                raise RuntimeError("Native fixture did not produce a complete line sequence")
            if any("end_sequence" in row["flags"] for row in rows[:-1]):
                raise RuntimeError("Original fixture now has multiple sequences; extend the independent oracle")
            generated += ["    {", "        info: D.Line_Info;", "        sequence: D.Line_Info_Sequence;"]
            for row in rows:
                flag_map = {"is_stmt": "STATEMENT", "basic_block": "BASIC_BLOCK", "prologue_end": "PROLOGUE_END", "epilogue_begin": "EPILOGUE_BEGIN"}
                flags = ["." + flag_map[flag] for flag in row["flags"] if flag in flag_map]
                unsupported = set(row["flags"]) - set(flag_map) - {"end_sequence"}
                if unsupported:
                    raise RuntimeError(f"Unexpected native row flags: {unsupported}")
                expression = " | ".join(flags) if flags else "0"
                is_end = "true" if "end_sequence" in row["flags"] else "false"
                generated += [f"        array_add(*sequence.addresses, {row['address']});",
                              f"        array_add(*sequence.mappings, D.Line_Mapping.{{file_id = {row['file_id']}, line = {row['line']}, column = {row['column']}, is_end = {is_end}, flags = {expression}}});"]
            generated += ["        defer array_free(sequence.addresses);", "        defer array_free(sequence.mappings);",
                          "        array_add(*info.sequences, sequence);", "        defer array_free(info.sequences);",
                          "        for i: 0..sequence.addresses.count - 2 {",
                          "            first := sequence.addresses[i]; last := sequence.addresses[i + 1];",
                          "            if first == last continue;",
                          "            for address: first..last - 1 {",
                          "                result, ok := D.get_line_mapping_for_address(info, address);",
                          "                expected := sequence.mappings[i];",
                          "                assert(ok && result.line == expected.line && result.column == expected.column && result.file_id == expected.file_id && result.flags == expected.flags && !result.is_end);",
                          "            }",
                          "            range, range_ok := D.get_line_mappings_for_address_range(info, first, last);",
                          "            defer array_free(range);",
                          "            assert(range_ok && range.count == 1 && range[0].line == sequence.mappings[i].line);",
                          "        }",
                          "        past, past_ok := D.get_line_mapping_for_address(info, sequence.addresses[sequence.addresses.count - 1]);",
                          "        assert(!past_ok && past.line == 0);", "    }"]
            evidence["fixtures"].append({"target": target, "optimization": optimization, "rows": rows,
                                         "line_bytes_sha256": hashlib.sha256(base.with_suffix('.bin').read_bytes()).hexdigest()})
    generated.append("}")
    path = ROOT / "target/developer-tools-dwarf-line-oracle.jai"
    path.write_text("\n".join(generated) + "\n")
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", path])
    evidence["clang_version"] = run([prefix / "bin/clang", "--version"])
    (ROOT / "docs/native/developer-tools-dwarf-lines-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
