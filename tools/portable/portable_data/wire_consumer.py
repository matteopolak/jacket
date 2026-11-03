"""Independent public-wire/SDK fixture checks; no Jai distribution input."""

import csv
import io
from pathlib import Path
import struct
import sys
import xml.etree.ElementTree as ET

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
