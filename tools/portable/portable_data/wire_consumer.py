"""Independent public-wire/SDK fixture checks; no Jai distribution input."""

import csv
import io
import shutil
from pathlib import Path
import struct
import sys
import xml.etree.ElementTree as ET
import zlib

from toolkit.repo import ROOT

root = Path(sys.argv[1])
root.mkdir(parents=True, exist_ok=True)
if len(sys.argv) > 2 and sys.argv[2] == "fixtures":
    # Public BMP file format: BITMAPFILEHEADER + BITMAPINFOHEADER, 16x16 BGR.
    pixels = bytes((30, 20, 10)) * (16 * 16)
    header = struct.pack("<2sIHHI", b"BM", 54 + len(pixels), 0, 0, 54)
    dib = struct.pack("<IiiHHIIiiII", 40, 16, 16, 1, 24, 0, len(pixels), 0, 0, 0, 0)
    (root / "bitmap-ü.bmp").write_bytes(header + dib + pixels)
    gradient = bytes(
        value
        for row in range(16)
        for column in range(16)
        for value in (row * 10, column * 10, 60 + row)
    )
    (root / "gradient.bmp").write_bytes(header + dib + gradient)
    # Scaling a picture to its own size keeps every pixel, so the expected result is the source as BGRA.
    bgra = bytes(
        value
        for row in range(16)
        for column in range(16)
        for value in (row * 10, column * 10, 60 + row, 255)
    )
    (root / "gradient-resized.bgra").write_bytes(bgra)

    def png_chunk(kind, data):
        return (
            struct.pack(">I", len(data))
            + kind
            + data
            + struct.pack(">I", zlib.crc32(kind + data))
        )

    def png(channels, color_type):
        rows = b"".join(
            b"\0" + bytes(value for x in range(16) for value in channels(x, 15 - y))
            for y in range(16)
        )
        return (
            b"\x89PNG\r\n\x1a\n"
            + png_chunk(
                b"IHDR", struct.pack(">IIBBBBB", 16, 16, 8, color_type, 0, 0, 0)
            )
            + png_chunk(b"IDAT", zlib.compress(rows))
            + png_chunk(b"IEND", b"")
        )

    def rgba(x, y):
        return (60 + y, x * 10, y * 10, 0 if x == 0 else 128 if x == 1 else 255)

    (root / "rgba.png").write_bytes(png(rgba, 6))
    palette = bytes(v for y in range(16) for x in range(16) for v in rgba(x, y)[:3])
    transparency = bytes(rgba(x, y)[3] for y in range(16) for x in range(16))
    indices = b"".join(
        b"\0" + bytes((15 - y) * 16 + x for x in range(16)) for y in range(16)
    )
    (root / "palette.png").write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", struct.pack(">IIBBBBB", 16, 16, 8, 3, 0, 0, 0))
        + png_chunk(b"PLTE", palette)
        + png_chunk(b"tRNS", transparency)
        + png_chunk(b"IDAT", zlib.compress(indices))
        + png_chunk(b"IEND", b"")
    )
    (root / "rgb.ppm").write_bytes(
        b"P6\n16 16\n255\n"
        + bytes(v for y in range(15, -1, -1) for x in range(16) for v in rgba(x, y)[:3])
    )
    (root / "rgba.qoi").write_bytes(
        b"qoif"
        + struct.pack(">II", 16, 16)
        + bytes((4, 0))
        + bytes(
            v for y in range(15, -1, -1) for x in range(16) for v in (255, *rgba(x, y))
        )
        + bytes((0, 0, 0, 0, 0, 0, 0, 1))
    )
    shutil.copyfile(
        ROOT / "stdlib/Extensions/Image/tests/fixtures/jpeg/block8.jpg",
        root / "rgb.jpg",
    )
    oversized = (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", struct.pack(">IIBBBBB", 8193, 8193, 8, 6, 0, 0, 0))
        + png_chunk(b"IDAT", zlib.compress(b""))
        + png_chunk(b"IEND", b"")
    )
    (root / "oversized.png").write_bytes(oversized)
    (root / "gray-alpha.png").write_bytes(png(lambda x, y: (x * 10, 128), 4))
    (root / "gray.pgm").write_bytes(b"P5\n16 16\n255\n" + bytes(range(256)))
    (root / "truncated.png").write_bytes((root / "rgba.png").read_bytes()[:24])
    # Uncompressed 32-bit TGA, bottom-up BGRA; preserves native-only acceptance.
    tga = struct.pack("<BBBHHBHHHHBB", 0, 0, 2, 0, 0, 0, 0, 0, 16, 16, 32, 8)
    tga += bytes(
        v
        for y in range(16)
        for x in range(16)
        for v in (y * 10, x * 10, 60 + y, rgba(x, y)[3])
    )
    (root / "rgba.tga").write_bytes(tga)
    sys.exit(0)

data = (root / "package.simp").read_bytes()
magic, version, flags, reserved, toc_offset = struct.unpack_from("<4sIIIq", data)
assert (magic, version, flags, reserved, toc_offset) == (b"simp", 1, 0, 0, 67)
toc_magic, reserved, count = struct.unpack_from("<4sIq", data, toc_offset)
assert (toc_magic, reserved, count) == (b"toc!", 0, 2)
cursor = toc_offset + 64
entries = []
for _ in range(count):
    (size,) = struct.unpack_from("<I", data, cursor)
    cursor += 4
    name = data[cursor : cursor + size].decode("utf-8")
    cursor += size
    assert data[cursor] == 0
    cursor += 1
    length, offset = struct.unpack_from("<qq", data, cursor)
    cursor += 16
    entries.append((name, data[offset : offset + length]))
assert cursor == len(data) and entries == [("zeta", bytes((1, 99, 3))), ("alpha", b"")]

csv_text = (root / "subtitles.csv").read_text()
rows = list(csv.reader(io.StringIO(csv_text)))
assert len(rows) == 3 and all(len(row) == 7 for row in rows)
assert rows[1] == [
    "262",
    "scene",
    'Alice "A"',
    'Hello <b>raw</b>\nSecond "line"',
    "gently",
    "scene_01.wav",
    "scene.wav",
]
assert rows[2] == ["263", "outro", "BOB", "Bye", "", "outro_01.wav", "outro.wav"]
print("independent wire/CSV consumer ok")


# Parse actual generated XML with the host XML consumer, rather than matching text.
xml_text = (root / "manifests-O2.xml").read_text()
documents = [
    part
    for part in xml_text.split(
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    )
    if part.strip()
]
assert len(documents) == 8
for bits, document in enumerate(documents):
    tree = ET.fromstring(document)
    assert tree.tag == "{urn:schemas-microsoft-com:asm.v1}assembly"
    assert tree.find("{urn:schemas-microsoft-com:asm.v1}assemblyIdentity") is None
    requested = tree.find(
        ".//{urn:schemas-microsoft-com:asm.v3}requestedExecutionLevel"
    )
    assert requested.attrib["level"] == (
        "requireAdministrator" if bits & 2 else "asInvoker"
    )
    legacy = tree.find(
        ".//{http://schemas.microsoft.com/SMI/2005/WindowsSettings}dpiAware"
    )
    modern = tree.find(
        ".//{http://schemas.microsoft.com/SMI/2016/WindowsSettings}dpiAwareness"
    )
    assert legacy.text == ("true/pm" if bits & 1 else "unaware")
    assert (modern is not None) == bool(bits & 1)
    heap = tree.find(
        ".//{http://schemas.microsoft.com/SMI/2020/WindowsSettings}heapType"
    )
    assert (heap is not None) == bool(bits & 4)
    code_page = tree.find(
        ".//{http://schemas.microsoft.com/SMI/2019/WindowsSettings}activeCodePage"
    )
    assert code_page.text == "UTF-8"
print("independent XML consumer ok")
