#!/usr/bin/env python3
"""Writes png_cases.jai: small PNG files covering every colour type, bit depth, filter, interlace
and transparency form, each with the pixels a decoder must return. The files are built here from
the PNG specification (zlib for compression), independently of the Jai encoder.

    python3 stdlib/Extensions/Image/tests/fixtures/generate_png_cases.py
"""

import random
import struct
import zlib
from pathlib import Path

PASSES = [(0, 0, 8, 8), (4, 0, 8, 8), (0, 4, 4, 8), (2, 0, 4, 4), (0, 2, 2, 4), (1, 0, 2, 2), (0, 1, 1, 2)]
CHANNELS = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}


def chunk(kind, body):
    data = kind + body
    return struct.pack(">I", len(body)) + data + struct.pack(">I", zlib.crc32(data) & 0xFFFFFFFF)


def pack_row(samples, depth):
    if depth == 16:
        return b"".join(struct.pack(">H", s) for s in samples)
    if depth == 8:
        return bytes(samples)
    out = bytearray()
    per_byte = 8 // depth
    for i in range(0, len(samples), per_byte):
        group = samples[i : i + per_byte]
        value = 0
        for k in range(per_byte):
            value = (value << depth) | (group[k] if k < len(group) else 0)
        out.append(value)
    return bytes(out)


def paeth(a, b, c):
    p = a + b - c
    pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
    if pa <= pb and pa <= pc:
        return a
    return b if pb <= pc else c


def filter_row(kind, row, prev, bpp):
    out = bytearray()
    for i, x in enumerate(row):
        a = row[i - bpp] if i >= bpp else 0
        b = prev[i]
        c = prev[i - bpp] if i >= bpp else 0
        if kind == 0:
            out.append(x)
        elif kind == 1:
            out.append((x - a) & 255)
        elif kind == 2:
            out.append((x - b) & 255)
        elif kind == 3:
            out.append((x - (a + b) // 2) & 255)
        else:
            out.append((x - paeth(a, b, c)) & 255)
    return bytes(out)


def scanlines(rows, depth, ctype, filters):
    bpp = max(1, depth * CHANNELS[ctype] // 8)
    out = bytearray()
    prev = bytes(len(pack_row(rows[0], depth))) if rows else b""
    for y, samples in enumerate(rows):
        row = pack_row(samples, depth)
        kind = filters[y % len(filters)]
        out.append(kind)
        out += filter_row(kind, row, prev, bpp)
        prev = row
    return bytes(out)


def build(width, height, depth, ctype, pixels, palette=None, trns=None, interlace=False, filters=(0,), split=0, extra=False):
    """pixels[y][x] is a tuple of samples (one per channel, at `depth` bits)."""
    header = struct.pack(">IIBBBBB", width, height, depth, ctype, 0, 0, 1 if interlace else 0)
    parts = [b"\x89PNG\r\n\x1a\n", chunk(b"IHDR", header)]
    if extra:
        parts.append(chunk(b"gAMA", struct.pack(">I", 45455)))
        parts.append(chunk(b"tEXt", b"Comment\x00hi"))
    if palette:
        parts.append(chunk(b"PLTE", bytes(v for entry in palette for v in entry)))
    if trns is not None:
        parts.append(chunk(b"tRNS", trns))

    if interlace:
        raw = b""
        for x0, y0, dx, dy in PASSES:
            rows = [[s for x in range(x0, width, dx) for s in pixels[y][x]] for y in range(y0, height, dy)]
            if rows and rows[0]:
                raw += scanlines(rows, depth, ctype, filters)
    else:
        rows = [[s for p in row for s in p] for row in pixels]
        raw = scanlines(rows, depth, ctype, filters)

    packed = zlib.compress(raw, 9)
    if split:
        pieces = [packed[i : i + split] for i in range(0, len(packed), split)]
    else:
        pieces = [packed]
    for i, piece in enumerate(pieces):
        parts.append(chunk(b"IDAT", piece))
        if extra and i == 0:
            # An animated PNG: its default image is the IDAT data; later frames are ignored.
            parts.append(chunk(b"fcTL", bytes(26)))
            parts.append(chunk(b"fdAT", bytes(8)))
    parts.append(chunk(b"IEND", b""))
    return b"".join(parts)


def scale(v, depth):
    if depth == 16:
        return v >> 8
    return v * 255 // ((1 << depth) - 1)


def jai_string(data):
    return '"' + "".join("\\x%02x" % b for b in data) + '"'


def main():
    rng = random.Random(7)
    cases = []

    def add(name, width, height, fmt, png, expected):
        cases.append((name, width, height, fmt, png, bytes(expected)))

    def random_pixels(width, height, depth, channels):
        top = (1 << depth) - 1
        return [[tuple(rng.randint(0, top) for _ in range(channels)) for _ in range(width)] for _ in range(height)]

    sizes = [(1, 1), (5, 3), (9, 7)]

    # Gray, every depth; every filter; interlaced; with and without a colour key.
    for depth in (1, 2, 4, 8, 16):
        for width, height in sizes:
            pixels = random_pixels(width, height, depth, 1)
            expected = [scale(p[0], depth) for row in pixels for p in row]
            for label, interlace, filters in (("plain", False, (0, 1, 2, 3, 4)), ("adam7", True, (4, 3, 2, 1, 0))):
                name = "gray%d_%dx%d_%s" % (depth, width, height, label)
                add(name, width, height, "GRAY8", build(width, height, depth, 0, pixels, interlace=interlace, filters=filters), expected)

            key = pixels[0][0][0]
            expected_alpha = []
            for row in pixels:
                for p in row:
                    expected_alpha += [scale(p[0], depth), 0 if p[0] == key else 255]
            add("gray%d_%dx%d_key" % (depth, width, height), width, height, "GRAY_ALPHA8", build(width, height, depth, 0, pixels, trns=struct.pack(">H", key), filters=(1, 2)), expected_alpha)

    # RGB and RGBA, 8 and 16 bits.
    for depth in (8, 16):
        for width, height in sizes:
            pixels = random_pixels(width, height, depth, 3)
            expected = [scale(s, depth) for row in pixels for p in row for s in p]
            add("rgb%d_%dx%d" % (depth, width, height), width, height, "RGB8", build(width, height, depth, 2, pixels, filters=(0, 1, 2, 3, 4)), expected)
            add("rgb%d_%dx%d_adam7" % (depth, width, height), width, height, "RGB8", build(width, height, depth, 2, pixels, interlace=True, filters=(4,)), expected)

            key = pixels[0][0]
            expected_alpha = []
            for row in pixels:
                for p in row:
                    expected_alpha += [scale(s, depth) for s in p] + [0 if p == key else 255]
            add("rgb%d_%dx%d_key" % (depth, width, height), width, height, "RGBA8", build(width, height, depth, 2, pixels, trns=b"".join(struct.pack(">H", s) for s in key), filters=(3, 4)), expected_alpha)

            pixels = random_pixels(width, height, depth, 4)
            expected = [scale(s, depth) for row in pixels for p in row for s in p]
            add("rgba%d_%dx%d" % (depth, width, height), width, height, "RGBA8", build(width, height, depth, 6, pixels, filters=(4, 3, 2, 1, 0)), expected)
            add("rgba%d_%dx%d_adam7" % (depth, width, height), width, height, "RGBA8", build(width, height, depth, 6, pixels, interlace=True, filters=(1, 4)), expected)

            pixels = random_pixels(width, height, depth, 2)
            expected = [scale(s, depth) for row in pixels for p in row for s in p]
            add("graya%d_%dx%d" % (depth, width, height), width, height, "GRAY_ALPHA8", build(width, height, depth, 4, pixels, filters=(2, 4)), expected)
            add("graya%d_%dx%d_adam7" % (depth, width, height), width, height, "GRAY_ALPHA8", build(width, height, depth, 4, pixels, interlace=True, filters=(0, 3)), expected)

    # Palettes.
    for depth in (1, 2, 4, 8):
        entries = min(1 << depth, 7 if depth == 8 else 1 << depth)
        palette = [(rng.randint(0, 255), rng.randint(0, 255), rng.randint(0, 255)) for _ in range(entries)]
        for width, height in sizes:
            pixels = [[(rng.randrange(entries),) for _ in range(width)] for _ in range(height)]
            expected = [c for row in pixels for p in row for c in palette[p[0]]]
            add("palette%d_%dx%d" % (depth, width, height), width, height, "RGB8", build(width, height, depth, 3, pixels, palette=palette, filters=(0, 1, 2, 3, 4)), expected)
            add("palette%d_%dx%d_adam7" % (depth, width, height), width, height, "RGB8", build(width, height, depth, 3, pixels, palette=palette, interlace=True, filters=(4,)), expected)

            alphas = [rng.randint(0, 255) for _ in range(max(1, entries - 1))]
            expected = []
            for row in pixels:
                for p in row:
                    alpha = alphas[p[0]] if p[0] < len(alphas) else 255
                    expected += list(palette[p[0]]) + [alpha]
            add("palette%d_%dx%d_alpha" % (depth, width, height), width, height, "RGBA8", build(width, height, depth, 3, pixels, palette=palette, trns=bytes(alphas), filters=(1, 4)), expected)

    # Structure: IDAT split into one-byte pieces, ancillary and animation chunks around the data.
    pixels = random_pixels(9, 7, 8, 3)
    expected = [s for row in pixels for p in row for s in p]
    add("rgb8_split_idat", 9, 7, "RGB8", build(9, 7, 8, 2, pixels, filters=(4, 2), split=1), expected)
    add("rgb8_ancillary_and_apng", 9, 7, "RGB8", build(9, 7, 8, 2, pixels, filters=(1,), extra=True), expected)

    lines = [
        "// Generated by generate_png_cases.py: do not edit.",
        "",
        "Png_Case :: struct {",
        "    name: string;",
        "    width: s32;",
        "    height: s32;",
        "    format: Pixel_Format;",
        "    png: string;",
        "    expected: string;",
        "}",
        "",
        "PNG_CASES :: Png_Case.[",
    ]
    for name, width, height, fmt, png, expected in cases:
        lines.append('    .{"%s", %d, %d, .%s,' % (name, width, height, fmt))
        lines.append("        %s," % jai_string(png))
        lines.append("        %s}," % jai_string(expected))
    lines.append("];")
    Path(__file__).with_name("png_cases.jai").write_text("\n".join(lines) + "\n")
    print(len(cases), "cases")


main()
