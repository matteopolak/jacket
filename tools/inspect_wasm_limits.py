"""Read defined table/memory limits from at most three admitted wasm files.

Metadata only: this does not execute, rewrite or fully validate a module. Imported
table/memory types and other section payloads are outside this narrow report.
"""

import argparse
import hashlib
import json
from pathlib import Path

MAX_BYTES = 2 * 1024 * 1024


class Reader:
    def __init__(self, data, start=0, end=None):
        self.data = data
        self.pos = start
        self.end = len(data) if end is None else end

    def byte(self):
        if self.pos >= self.end:
            raise ValueError(f"truncated metadata at offset {self.pos}")
        value = self.data[self.pos]
        self.pos += 1
        return value

    def uint(self, bits):
        result = 0
        for shift in range(0, bits, 7):
            byte = self.byte()
            result |= (byte & 127) << shift
            if result >= 1 << bits:
                raise ValueError(f"integer exceeds u{bits} at offset {self.pos - 1}")
            if not byte & 128:
                return result
        raise ValueError(f"unterminated u{bits} at offset {self.pos - 1}")


def inspect(data):
    if len(data) > MAX_BYTES:
        raise ValueError("module exceeds the admitted 2 MiB capture limit")
    if data[:8] != b"\x00asm\x01\x00\x00\x00":
        raise ValueError("not a wasm version-one binary")
    reader = Reader(data, 8)
    records = []
    while reader.pos < reader.end:
        section = reader.byte()
        size = reader.uint(32)
        end = reader.pos + size
        if end > reader.end:
            raise ValueError(f"section {section} exceeds the module at {reader.pos}")
        if section in (4, 5):
            payload = Reader(data, reader.pos, end)
            count = payload.uint(32)
            for index in range(count):
                record = {"kind": "table" if section == 4 else "memory", "index": index}
                if section == 4:
                    element = payload.byte()
                    if element not in (0x70, 0x6F):
                        raise ValueError(
                            "this report supports plain funcref/externref tables only"
                        )
                    record["element"] = hex(element)
                record["flags_offset"] = payload.pos
                flags = payload.uint(32)
                # Memory shared bit is known; shared table/newer proposals need
                # their own decoder rather than guessing the following layout.
                if flags & ~(7 if section == 5 else 5):
                    raise ValueError(
                        f"unsupported limits flags {flags} at {record['flags_offset']}"
                    )
                record.update(flags=flags, index_bits=64 if flags & 4 else 32)
                record["minimum"] = payload.uint(record["index_bits"])
                record["maximum"] = (
                    payload.uint(record["index_bits"]) if flags & 1 else None
                )
                records.append(record)
            if payload.pos != end:
                raise ValueError(
                    f"unconsumed limits section bytes at offset {payload.pos}"
                )
        reader.pos = end
    return {
        "sha256": hashlib.sha256(data).hexdigest(),
        "bytes": len(data),
        "defined_limits": records,
        "scope": "defined table/memory metadata only; imports and code not validated",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("files", type=Path, nargs="+")
    args = parser.parse_args()
    if len(args.files) > 3:
        parser.error("at most three explicitly admitted files may be inspected")
    results = []
    failed = False
    for path in args.files:
        try:
            if path.stat().st_size > MAX_BYTES:
                raise ValueError("module exceeds the admitted 2 MiB capture limit")
            result = inspect(path.read_bytes())
        except (OSError, ValueError) as error:
            result = {"error": str(error)}
            failed = True
        results.append({"path": str(path), **result})
    print(json.dumps(results, indent=2))
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
