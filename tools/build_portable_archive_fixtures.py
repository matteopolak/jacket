#!/usr/bin/env python3
"""Build original ar wire fixtures and a genuine host-tool C object archive."""
from pathlib import Path
import shutil
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "target/portable-tests/archive-fixtures"


def member(name, body):
    name = name.encode()
    assert len(name) <= 16
    header = (name.ljust(16) + b"0".ljust(12) + b"0".ljust(6)
              + b"0".ljust(6) + b"100644".ljust(8)
              + str(len(body)).encode().ljust(10) + b"`\n")
    assert len(header) == 60
    return header + body + (b"\n" if len(body) & 1 else b"")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    magic = b"!<arch>\n"
    basic = member("short.o/", b"abc")
    (OUT / "classic.a").write_bytes(magic + basic)
    extended = b"long member name.o"
    bsd_member = member(f"#1/{len(extended)}", extended + b"abc")
    for big in [False, True]:
        endian = ">" if big else "<"
        symbol = b"answer\0"
        length = 4 + 8 + 4 + len(symbol)
        # Extended index names also exercise NUL padding in the name prefix.
        index_name = b"__.SYMDEF\0\0\0"
        target = 8 + len(member(f"#1/{len(index_name)}", index_name + b"\0" * length))
        index = struct.pack(endian + "IIII", 8, 0, target, len(symbol)) + symbol
        data = magic + member(f"#1/{len(index_name)}", index_name + index) + bsd_member
        (OUT / f"bsd-{'big' if big else 'little'}.a").write_bytes(data)
        if not big:
            corrupt = bytearray(data)
            start = 8 + 60 + len(index_name) + 4
            corrupt[start:start + 4] = struct.pack("<I", len(symbol) + 1)
            (OUT / "bad-bsd-string.a").write_bytes(corrupt)
    for coff in [False, True]:
        long_table = extended + (b"\0" if coff else b"/\n")
        names = b"answer\0alias\0"
        first_size = 4 + 8 + len(names)
        second_size = 4 + 4 + 4 + 4 + len(names)
        target = 8 + len(member("/", b"\0" * first_size)) + len(member("//", long_table))
        if coff:
            target += len(member("/", b"\0" * second_size))
        first = struct.pack(">III", 2, target, target) + names
        data = magic + member("/", first)
        if coff:
            second = struct.pack("<IIIHH", 1, target, 2, 1, 1) + names
            data += member("/", second)
        data += member("//", long_table) + member("/0", b"abc")
        label = "coff" if coff else "gnu"
        (OUT / f"{label}.a").write_bytes(data)
        corrupt = bytearray(data)
        corrupt[72:76] = b"\xff" * 4  # First symbol points outside the archive.
        (OUT / f"bad-{label}-offset.a").write_bytes(corrupt)
        corrupt = bytearray(data)
        if coff:
            second_start = 8 + len(member("/", first)) + 60
            corrupt[second_start + 12:second_start + 14] = b"\0\0"
            (OUT / "bad-coff-index.a").write_bytes(corrupt)
        else:
            corrupt[68:72] = b"\xff" * 4
            (OUT / "bad-gnu-count.a").write_bytes(corrupt)
            corrupt = bytearray(data)
            corrupt[68 + first_size - 1] = ord("X")
            (OUT / "bad-symbol-termination.a").write_bytes(corrupt)
    (OUT / "bad-size.a").write_bytes(magic + basic[:48] + b"9999999999" + basic[58:])
    (OUT / "bad-padding.a").write_bytes(magic + basic[:-1] + b"x")
    (OUT / "bad-trailer.a").write_bytes(magic + basic[:58] + b"xx" + basic[60:])
    (OUT / "bad-name.a").write_bytes(magic + member("/0", b"abc"))
    (OUT / "unsupported64.a").write_bytes(magic + member("/SYM64/", b""))
    source = OUT / "original.c"
    source.write_text("int portable_archive_answer(void) { return 42; }\n")
    compiler = shutil.which("clang") or shutil.which("cc")
    archive_tool = shutil.which("ar")
    if not compiler or not archive_tool:
        raise RuntimeError("source-built archive fixture requires a C compiler and ar")
    subprocess.run([compiler, "-c", str(source), "-o", str(OUT / "original.o")], check=True)
    native = OUT / "native.a"
    native.unlink(missing_ok=True)
    subprocess.run([archive_tool, "rcs", str(native), str(OUT / "original.o")], check=True)
    print("Built original archive wire fixtures and genuine host C object archive")


if __name__ == "__main__":
    main()
