#!/usr/bin/env python3
"""Generate tiny JPEG color-hint regressions from constant DC-only blocks.

    python3 stdlib/Extensions/Image/tests/fixtures/jpeg/generate_headers.py

Each component has one block, quantizers are one, and AC is zero. A constant
sample v therefore has DC 8*(v-128). This fixes the expected pixels without
using our decoder or requiring an external JPEG encoder.
"""

from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parent
JFIF = b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0"
ADOBE = b"Adobe\0\x64\0\0\0\0"


def segment(code, payload):
    return bytes((255, code)) + struct.pack(">H", len(payload) + 2) + payload


def constant_jpeg(samples, identifiers, jfif=None, adobe=None):
    dc_table = bytes((0, 0, 0, 0, 12)) + bytes(12) + bytes(range(12))
    ac_table = bytes((16, 1)) + bytes(15) + b"\0"
    tables = segment(196, dc_table + ac_table)
    frame = bytes((8, 0, 1, 0, 1, len(samples)))
    frame += bytes(value for identifier in identifiers for value in (identifier, 17, 0))
    scan = bytes((len(samples),))
    scan += bytes(value for identifier in identifiers for value in (identifier, 0))
    scan += bytes((0, 63, 0))
    bits = ""
    for sample in samples:
        dc = 8 * (sample - 128)
        category = abs(dc).bit_length()
        bits += f"{category:04b}"
        if category:
            amplitude = dc if dc >= 0 else dc + (1 << category) - 1
            bits += f"{amplitude:0{category}b}"
        bits += "0"  # AC end-of-block.
    bits += "1" * (-len(bits) % 8)
    entropy = bytes(int(bits[at:at + 8], 2) for at in range(0, len(bits), 8))
    entropy = entropy.replace(b"\xff", b"\xff\0")
    out = b"\xff\xd8"
    if jfif is not None:
        out += segment(224, jfif)
    if adobe is not None:
        out += segment(238, ADOBE + bytes((adobe,)))
    return out + segment(219, b"\0" + bytes((1,)) * 64) + segment(192, frame) + tables + segment(218, scan) + entropy + b"\xff\xd9"


def main():
    ycc = (64, 128, 128)
    cases = (
        ("header_ycc", ycc, (1, 2, 3), None, None, (64, 64, 64)),
        ("header_jfif_adobe", ycc, (1, 2, 3), JFIF, 0, (64, 64, 64)),
        ("header_jfif_rgbids", ycc, (82, 71, 66), JFIF, None, (64, 64, 64)),
        ("header_adobe_ycc_rgbids", ycc, (82, 71, 66), None, 1, (64, 64, 64)),
        ("header_short_jfif_adobe", ycc, (1, 2, 3), b"JFIF\0", 0, ycc),
        ("header_rgb", ycc, (1, 2, 3), None, 0, ycc),
        ("header_rgb_ids", ycc, (82, 71, 66), None, None, ycc),
        ("header_cmyk_plain", (0, 0, 0, 0), (1, 2, 3, 4), None, None, (255, 255, 255)),
        ("header_cmyk_adobe", (0, 0, 0, 0), (1, 2, 3, 4), None, 0, (0, 0, 0)),
    )
    for name, samples, identifiers, jfif, adobe, expected in cases:
        (ROOT / (name + ".jpg")).write_bytes(constant_jpeg(samples, identifiers, jfif, adobe))
        (ROOT / (name + ".ppm")).write_bytes(b"P6\n1 1\n255\n" + bytes(expected))


if __name__ == "__main__":
    main()
