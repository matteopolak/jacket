#!/usr/bin/env python3
"""Original native C13 strings/checksums with independent official SDK file-ID oracles."""
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run(command):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, text=True, capture_output=True, timeout=60)
    if result.returncode or result.stderr:
        raise RuntimeError(f"Original CodeView fixture failed: {command}: {result.returncode}\n{result.stderr}")
    return result.stdout


def main():
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    source = ROOT / "tests/developer-tools/codeview-strings-native.c"
    header = ROOT / "tests/developer-tools/codeview-strings-native.h"
    allowed_paths = {str(source), str(header)}
    generated = ["check_native_codeview_strings :: () {"]
    evidence = {"scope": "original C Windows x64/arm64 O0/O2 C13 sections and SDK filename/file-ID/checksum oracles; host queries, no target execution or Jai symbol-parser claim", "fixtures": []}
    for architecture, target in [("x64", "x86_64-pc-windows-msvc"), ("arm64", "aarch64-pc-windows-msvc")]:
        for optimization in [0, 2]:
            base = ROOT / f"target/developer-tools-codeview-strings-{architecture}-o{optimization}"
            run([prefix / "bin/clang", "-target", target, "-std=c11", "-g", "-gcodeview", f"-O{optimization}",
                 "-Wall", "-Wextra", "-Werror", "-c", source, "-o", base.with_suffix(".obj")])
            binary = base.with_suffix(".debug-s.bin")
            run([prefix / "bin/llvm-objcopy", "--dump-section", f".debug$S={binary}", base.with_suffix(".obj")])
            data = binary.read_bytes()
            if len(data) < 4 or struct.unpack_from("<I", data)[0] != 4:
                raise RuntimeError("Original section is not C13")
            subsections = {}
            at = 4
            while at < len(data):
                if at + 8 > len(data):
                    raise RuntimeError("Truncated original subsection header")
                kind, size = struct.unpack_from("<II", data, at)
                at += 8
                if at + size > len(data):
                    raise RuntimeError("Truncated original subsection payload")
                if kind in [0xf3, 0xf4]:
                    if kind in subsections:
                        raise RuntimeError("Extend fixture support for multiple native string/checksum tables")
                    subsections[kind] = data[at:at + size]
                at = (at + size + 3) & ~3
            if set(subsections) != {0xf3, 0xf4}:
                raise RuntimeError("Original fixture lacks string/checksum tables")
            strings, checksums = subsections[0xf3], subsections[0xf4]
            table_path = base.with_suffix(".strings.bin")
            checksum_path = base.with_suffix(".checksums.bin")
            table_path.write_bytes(strings)
            checksum_path.write_bytes(checksums)
            metadata = run([prefix / "bin/llvm-readobj", "--codeview", base.with_suffix(".obj")])
            sdk_checksums = {}
            for block in re.findall(r"FileChecksum\s*\{(.*?)\n\s*\}", metadata, re.DOTALL):
                filename = re.search(r"Filename: (.*?) \(0x([0-9a-fA-F]+)\)", block)
                digest = re.search(r"ChecksumBytes: \(([^)]*)\)", block)
                if not filename or not digest or filename[1] not in allowed_paths:
                    raise RuntimeError("Unexpected native checksum path/metadata")
                sdk_checksums[int(filename[2], 16)] = (filename[1], bytes.fromhex(digest[1]))
            sdk_file_ids = {int(file_id, 16): name for name, file_id in re.findall(r"FilenameSegment\s*\[\s*Filename: (.*?) \(0x([0-9a-fA-F]+)\)", metadata)}
            records = []
            at = 0
            while at < len(checksums):
                if at + 6 > len(checksums):
                    raise RuntimeError("Truncated original checksum header")
                name_offset, size, kind = struct.unpack_from("<IBB", checksums, at)
                if at + 6 + size > len(checksums) or name_offset not in sdk_checksums or kind != 1 or size != 16:
                    raise RuntimeError("Unexpected original checksum record")
                name, expected_digest = sdk_checksums[name_offset]
                end = strings.find(b"\0", name_offset)
                actual_digest = checksums[at + 6:at + 6 + size]
                if end < 0 or strings[name_offset:end].decode("utf-8") != name or actual_digest != expected_digest or hashlib.md5(Path(name).read_bytes()).digest() != actual_digest or sdk_file_ids.get(at) != name:
                    raise RuntimeError("Native bytes differ from independent SDK filename/file-ID/source checksum oracle")
                records.append({"id": at, "filename_id": name_offset, "name": name, "digest_offset": at + 6, "digest_size": size})
                at = (at + 6 + size + 3) & ~3
            if len(records) < 2:
                raise RuntimeError("Original fixture did not retain both C/header filenames")
            generated += ["    {", f"        table, table_ok := read_entire_file({json.dumps(str(table_path))});", "        assert(table_ok);", "        defer free(table);",
                          f"        checksums, checksums_ok := read_entire_file({json.dumps(str(checksum_path))});", "        assert(checksums_ok);", "        defer free(checksums);",
                          "        info: D.Codeview_Debug_Info;", "        info.string_table = cast([] u8) table;", "        defer array_free(info.file_checksums);"]
            for record in records:
                generated += [f"        array_add(*info.file_checksums, D.Codeview_File_Checksum.{{id = {record['id']}, filename_id = {record['filename_id']}, checksum_type = .MD5, checksum = .{{{record['digest_size']}, checksums.data + {record['digest_offset']}}}}});",
                              f"        name_{record['id']}, ok_{record['id']} := D.get_filename_for_file_id(info, {record['id']});",
                              f"        assert(ok_{record['id']} && name_{record['id']} == {json.dumps(record['name'])});",
                              f"        assert(name_{record['id']}.data == table.data + {record['filename_id']});"]
            generated += ["        empty, empty_ok := D.get_string_from_table(info, 0);", "        assert(empty_ok && empty.count == 0);", "    }"]
            evidence["fixtures"].append({"architecture": architecture, "optimization": optimization, "files": records,
                                         "debug_s_sha256": hashlib.sha256(data).hexdigest()})
    generated.append("}")
    destination = ROOT / "target/developer-tools-codeview-string-oracle.jai"
    destination.write_text("\n".join(generated) + "\n")
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", destination])
    (ROOT / "docs/native/developer-tools-codeview-string-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
