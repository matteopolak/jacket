#!/usr/bin/env python3
"""Construct independent classic ZIP fixtures using Python's archive writer."""
from pathlib import Path
import struct
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    directory = ROOT / "target/portable-tests/zip"
    directory.mkdir(parents=True, exist_ok=True)
    valid = directory / "valid.zip"
    with zipfile.ZipFile(valid, "w", compression=zipfile.ZIP_STORED) as archive:
        archive.writestr("first.txt", b"hello")
        archive.writestr("binary.bin", bytes([0, 128, 255]))
        archive.comment = b"comment with false PK\x05\x06 marker"
    with zipfile.ZipFile(directory / "empty.zip", "w"):
        pass
    original = valid.read_bytes()
    end = original.index(b"PK\x05\x06")
    central = struct.unpack_from("<I", original, end + 16)[0]
    variants = {}
    variants["truncated"] = bytearray(original[:15])
    for name in ["bounds", "zip64", "multidisk", "count", "late-name", "signature", "descriptor"]:
        variants[name] = bytearray(original)
    struct.pack_into("<I", variants["bounds"], end + 16, len(original))
    struct.pack_into("<I", variants["zip64"], central + 24, 0xffffffff)
    struct.pack_into("<H", variants["multidisk"], end + 4, 1)
    struct.pack_into("<HH", variants["count"], end + 8, 3, 3)
    second = original.index(b"PK\x01\x02", central + 4)
    struct.pack_into("<H", variants["late-name"], second + 28, 65535)
    struct.pack_into("<I", variants["signature"], central, 0x12345678)
    struct.pack_into("<H", variants["descriptor"], central + 8, 8)
    for name, data in variants.items():
        (directory / f"{name}.zip").write_bytes(data)
    print("Built independent ZIP fixtures")


if __name__ == "__main__":
    main()
