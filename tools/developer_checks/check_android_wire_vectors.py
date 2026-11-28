#!/usr/bin/env python3
"""Write the Android wire vectors from Python's own integer, hashlib and zlib code, then run the tests that read them.

The vectors are `portable-asn1/vectors.bin`, `portable-sha/vectors.bin` and `portable-zip-wire/crc-vectors.bin` under
the output directory; `stdlib/Android/tests/driven/{asn1,digests,zip-wire}/vectors.jai` read them. The Jai side
implements ASN.1, SHA and CRC itself, so the expected values must not come from it.
"""
import hashlib
import random
import struct
import zlib

from developer_checks.dev_common import ROOT, cargo_binary, labeled_runner, output_dir

run = labeled_runner("Android wire vectors")

TESTS = ROOT / "stdlib/Android/tests/driven"
MAX_ARCS = 12  # ASN1_Object_Identifier.MAX_SUBIDENTIFIERS


def block(data):
    return struct.pack("<I", len(data)) + data


def base128(value):
    """The DER base-128 encoding of one arc."""
    out = [value & 127]
    value >>= 7
    while value:
        out.append((value & 127) | 128)
        value >>= 7
    return bytes(reversed(out))


def integer_content(value, width):
    """`width` big-endian content octets of a nonnegative INTEGER, with leading zero octets as needed."""
    return value.to_bytes(width, "big")


def asn1_vectors():
    rng = random.Random(1)
    integers = []
    for width in range(1, 9):
        top = 1 << (8 * width - 1)  # the first content octet must have its top bit clear
        for value in {0, 1, 127, 128, top - 1, top // 2, rng.randrange(top), rng.randrange(top)}:
            if value < top:
                integers.append((integer_content(value, width), value))

    identifiers = []
    arcs = [
        [1, 2, 840, 113549, 1, 1, 11],
        [2, 5, 4, 3],
        [0, 39],
        [1, 39, 0],
        [2, 0],
        [2, 39, 3],
        [2, 47, 1],
        [2, 999, 3],
        [2, (1 << 64) - 1 - 80 + 1, 5],
        [1, 3, 6, 1, 4, 1, 311, 21, 20],
        [1, 2] + [rng.randrange(1 << 63) for _ in range(MAX_ARCS - 2)],
    ]
    for parts in arcs:
        encoded = base128(40 * parts[0] + parts[1]) + b"".join(base128(arc) for arc in parts[2:])
        text = "".join(f"{arc}." for arc in parts).encode()
        identifiers.append((encoded, parts, text))

    out = struct.pack("<I", len(integers))
    for content, value in integers:
        out += block(content) + struct.pack("<Q", value)
    out += struct.pack("<I", len(identifiers))
    for encoded, parts, text in identifiers:
        out += block(encoded) + struct.pack("<I", len(parts))
        out += b"".join(struct.pack("<Q", arc) for arc in parts)
        out += block(text)
    return out


def sha_input(length, mode):
    if mode == 1:
        return b"a" * length
    return bytes((index * 73 + length * 19) & 255 for index in range(length))


def sha_vectors():
    lengths = [0, 1, 2, 3, 55, 56, 57, 63, 64, 65, 111, 112, 119, 120, 127, 128, 129, 255, 256, 1000, 4095, 4096, 65537, 200000]
    cases = [(length, mode) for length in lengths for mode in (0, 1)]
    out = struct.pack("<II", 0x53484132, len(cases))
    for length, mode in cases:
        data = sha_input(length, mode)
        out += struct.pack("<II", length, mode) + hashlib.sha1(data).digest() + hashlib.sha256(data).digest()
    return out


def crc_vectors():
    rng = random.Random(3)
    cases = [bytes([value]) for value in range(256)]
    cases += [b"", b"123456789", b"The quick brown fox jumps over the lazy dog"]
    cases += [rng.randbytes(length) for length in (2, 3, 7, 8, 9, 63, 64, 65, 1000, 65536, 100001)]
    out = struct.pack("<I", len(cases))
    for data in cases:
        out += block(data) + struct.pack("<I", zlib.crc32(data) & 0xffffffff)
    return out


def main():
    jaic = cargo_binary("jaic")
    vectors = [
        ("asn1", "portable-asn1/vectors.bin", asn1_vectors()),
        ("digests", "portable-sha/vectors.bin", sha_vectors()),
        ("zip-wire", "portable-zip-wire/crc-vectors.bin", crc_vectors()),
    ]
    for folder, relative, data in vectors:
        path = output_dir(relative)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        source = TESTS / folder / "vectors.jai"
        run([jaic, source, "-interp"], "ok\n")
        executable = output_dir(f"android-wire-{folder}")
        run([jaic, source, "-exe", executable])
        run([executable], "ok\n")
    print("ASN.1 integer and object identifier, SHA-1/SHA-256 and CRC-32 vectors matched Python on the interpreter and native")


if __name__ == "__main__":
    main()
