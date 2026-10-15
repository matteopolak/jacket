#!/usr/bin/env python3
"""Stage the owned DWARF line decoder and compare native bytes with existing SDK row oracles."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    evidence = json.loads((ROOT / "docs/native/developer-tools-dwarf-lines-evidence.json").read_text())
    output = ["check_native_line_wire :: () {"]
    flag_bits = {"is_stmt": 1, "basic_block": 2, "prologue_end": 4, "epilogue_begin": 8}
    for fixture in evidence["fixtures"]:
        architecture = "arm64" if fixture["target"].startswith("aarch64") else "x86" if fixture["target"].startswith("i386") else "x64"
        address_size = 4 if architecture == "x86" else 8
        data = ROOT / f"target/developer-tools-dwarf-line-{architecture}-o{fixture['optimization']}.bin"
        if hashlib.sha256(data.read_bytes()).hexdigest() != fixture["line_bytes_sha256"]:
            raise RuntimeError("Native line section changed after official SDK decoding")
        output += ["    {", f"        bytes, bytes_ok := read_entire_file({json.dumps(str(data))});", "        assert(bytes_ok);", "        defer free(bytes);",
                   f"        table, ok := line_wire_decode(cast([] u8) bytes, 0, {address_size});", "        assert(ok && table.next_offset == bytes.count);", "        defer line_wire_release(*table);",
                   f"        assert(table.rows.count == {len(fixture['rows'])});", "        assert(table.header.version == 4 && table.header.size + 4 == bytes.count);",
                   "        assert(table.header.maximum_operations_per_instruction == 1);", "        assert(table.files.count > 0);"]
        for index, row in enumerate(fixture["rows"]):
            flags = sum(flag_bits.get(flag, 0) for flag in row["flags"])
            ending = f"table.rows[{index}].is_end" if "end_sequence" in row["flags"] else f"!table.rows[{index}].is_end"
            output += [f"        assert(table.rows[{index}].address == {row['address']} && table.rows[{index}].line == {row['line']} && table.rows[{index}].column == {row['column']});",
                       f"        assert(table.rows[{index}].file == {row['file_id']} && table.rows[{index}].flags == {flags} && {ending});",
                       f"        assert(table.rows[{index}].discriminator == {row['discriminator']} && table.rows[{index}].op_index == 0);"]
        output += ["        for prefix: 0..bytes.count - 1 {", f"            partial, partial_ok := line_wire_decode(.{{prefix, bytes.data}}, 0, {address_size});",
                   "            assert(!partial_ok && partial.rows.count == 0 && partial.files.count == 0 && partial.dirs.count == 0);", "        }", "    }"]
    output.append("}")
    oracle = ROOT / "target/developer-tools-dwarf-line-wire-oracle.jai"
    oracle.write_text("\n".join(output) + "\n")
    shutil.copyfile(ROOT / "stdlib/internal/developer-tools/debug-formats/dwarf-line-wire.jai",
                    ROOT / "target/developer-tools-dwarf-line-wire.jai")
    subprocess.run([str(ROOT / "target/jaifmt"), "--config", str(ROOT / "jaifmt.toml"), str(oracle)], cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
