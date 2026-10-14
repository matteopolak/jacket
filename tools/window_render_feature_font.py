#!/usr/bin/env python3
"""Generate an original, deterministic TrueType/GSUB native test asset.

All outlines, advances, names and substitution choices are authored here. No font
or Jai implementation is copied. The binary format follows Microsoft's public
OpenType head/hhea/maxp/glyf/loca/cmap/name/post/OS2/GSUB documentation. FreeType,
not this generator, supplies production glyph metrics and rasterization in tests.
"""
import argparse
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "tests/window-render/assets/feature-font.ttf"


def unsigned16(*values):
    return struct.pack(">" + "H" * len(values), *values)


def signed16(*values):
    return struct.pack(">" + "h" * len(values), *values)


def unsigned32(*values):
    return struct.pack(">" + "I" * len(values), *values)


def padded(data):
    return data + bytes((-len(data)) % 4)


def checksum(data):
    data = padded(data)
    return sum(struct.unpack(">" + "I" * (len(data) // 4), data)) & 0xffffffff


def rectangle(width, height):
    if not width or not height:
        return signed16(0, 0, 0, 0, 0) + unsigned16(0)
    # Clockwise contour: four on-curve points, no hinting instructions.
    return (signed16(1, 0, 0, width, height) + unsigned16(3, 0)
            + bytes([1, 1, 1, 1]) + signed16(0, 0, width, 0)
            + signed16(0, height, 0, -height))


def character_map():
    mappings = [(32, 7), (49, 4), (50, 5), (63, 1), (65, 3), (97, 2), (0xfffd, 6), (0xffff, 0)]
    count = len(mappings)
    power = 1 << (count.bit_length() - 1)
    subtable = unsigned16(4, 16 + 8 * count, 0, count * 2,
                          power * 2, power.bit_length() - 1, count * 2 - power * 2)
    subtable += unsigned16(*(code for code, _ in mappings)) + unsigned16(0)
    subtable += unsigned16(*(code for code, _ in mappings))
    subtable += unsigned16(*((glyph - code) & 0xffff for code, glyph in mappings))
    subtable += unsigned16(*([0] * count))
    return unsigned16(0, 1, 3, 1) + unsigned32(12) + subtable


def font_names():
    names = [(1, "Cleanroom Feature Test"), (2, "Regular"),
             (4, "Cleanroom Feature Test Regular"), (5, "Version 1.0"),
             (6, "CleanroomFeatureTest-Regular")]
    records = bytearray()
    strings = bytearray()
    for identifier, name in names:
        text = name.encode("utf-16-be")
        records += unsigned16(3, 1, 0x409, identifier, len(text), len(strings))
        strings += text
    return unsigned16(0, len(names), 6 + 12 * len(names)) + records + strings


def substitutions(filtered):
    # smcp: extension -> single format2, with range coverage. A following
    # single format1 changes glyph3 to8, exercising ordered lookup application.
    ranges = unsigned16(2, 2, 2, 2, 0, 4, 5, 1)
    single = unsigned16(2, 12, 3, 3, 6, 6) + ranges
    extension = unsigned16(1, 1) + unsigned32(8) + single
    lookup0 = unsigned16(7, 2 if filtered else 0, 1, 8) + extension
    # lnum: alternate format1, selecting the first available alternate.
    alternate = unsigned16(1, 18, 2, 10, 14, 1, 5, 1, 6) + unsigned16(1, 2, 4, 5)
    lookup1 = unsigned16(3, 4 if filtered else 0, 1, 8) + alternate
    lookup2 = unsigned16(1, 8 if filtered else 0, 1, 8) + unsigned16(1, 6, 5, 1, 1, 3)
    lookups = (unsigned16(3, 8, 8 + len(lookup0), 8 + len(lookup0) + len(lookup1))
               + lookup0 + lookup1 + lookup2)
    # Feature records are sorted by tag, independently of lookup ordering.
    features = (unsigned16(2) + b"lnum" + unsigned16(14) + b"smcp" + unsigned16(20)
                + unsigned16(0, 1, 1) + unsigned16(0, 2, 0, 2))
    scripts = (unsigned16(1) + b"DFLT" + unsigned16(8)
               + unsigned16(4, 0, 0, 0xffff, 2, 0, 1))
    return unsigned32(0x10000) + unsigned16(10, 10 + len(scripts),
                                          10 + len(scripts) + len(features)) + scripts + features + lookups


def build_font(filtered=False):
    # Native glyph IDs: notdef, question, a, A, oldstyle1, lining1,
    # replacement, space, smallcap. Data is test artwork, not guessed metrics.
    designs = [(400, 700, 500), (300, 500, 400), (300, 400, 400),
               (450, 600, 550), (180, 400, 300), (280, 700, 450),
               (600, 600, 650), (0, 0, 300), (400, 550, 500)]
    glyphs = bytearray()
    offsets = []
    for width, height, _ in designs:
        offsets.append(len(glyphs) // 2)
        glyphs += rectangle(width, height)
    offsets.append(len(glyphs) // 2)
    head = (unsigned32(0x10000, 0x10000, 0, 0x5f0f3cf5) + unsigned16(3, 1000)
            + struct.pack(">qq", 0, 0) + signed16(0, 0, 600, 700)
            + unsigned16(0, 8) + signed16(2, 0, 0))
    hhea = (unsigned32(0x10000) + signed16(800, -200, 0) + unsigned16(650)
            + signed16(0, 50, 600, 1, 0, 0, 0, 0, 0, 0, 0) + unsigned16(len(designs)))
    os2 = (unsigned16(0) + signed16(sum(d[2] for d in designs) // len(designs))
           + unsigned16(400, 5, 0) + signed16(650, 600, 0, 75, 650, 600, 0, 350, 50, 250, 0)
           + bytes(10) + unsigned32(1, 0, 32, 0) + b"CRFT" + unsigned16(64, 32, 0xfffd)
           + signed16(800, -200, 0) + unsigned16(800, 200))
    assert len(head) == 54 and len(hhea) == 36 and len(os2) == 78
    tables = {
        b"head": head, b"hhea": hhea, b"OS/2": os2,
        b"maxp": unsigned32(0x10000) + unsigned16(len(designs), 4, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0),
        b"hmtx": b"".join(unsigned16(advance, 0) for _, _, advance in designs),
        b"glyf": bytes(glyphs), b"loca": unsigned16(*offsets),
        b"cmap": character_map(), b"name": font_names(),
        b"post": unsigned32(0x30000, 0) + signed16(-75, 50) + unsigned32(0, 0, 0, 0, 0),
        b"GSUB": substitutions(filtered),
    }
    if filtered:
        # Original classifications exercise IgnoreBaseGlyphs, IgnoreLigatures,
        # and IgnoreMarks, using the native GDEF class-definition format2.
        classes = unsigned16(2, 4, 2, 3, 3, 4, 4, 1, 5, 5, 2, 6, 6, 3)
        tables[b"GDEF"] = unsigned16(1, 0, 12, 0, 0, 0) + classes
    count = len(tables)
    power = 1 << (count.bit_length() - 1)
    directory = bytearray(unsigned32(0x10000) + unsigned16(count, power * 16,
                                                        power.bit_length() - 1, count * 16 - power * 16))
    contents = bytearray()
    head_offset = None
    for tag, data in sorted(tables.items()):
        offset = 12 + 16 * count + len(contents)
        if tag == b"head":
            head_offset = offset
        directory += tag + unsigned32(checksum(data), offset, len(data))
        contents += padded(data)
    result = directory + contents
    assert head_offset is not None
    struct.pack_into(">I", result, head_offset + 8, (0xb1b0afba - checksum(result)) & 0xffffffff)
    assert checksum(result) == 0xb1b0afba
    return bytes(result)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    for filtered in [False, True]:
        output = OUTPUT.with_name("filtered-feature-font.ttf") if filtered else OUTPUT
        data = build_font(filtered)
        if args.check:
            if output.read_bytes() != data:
                raise SystemExit(f"{output.name} differs from generator output")
            print(f"verified original {len(data)}-byte {output.name}")
        else:
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_bytes(data)
            print(f"generated original {len(data)}-byte {output.name}")


if __name__ == "__main__":
    main()
