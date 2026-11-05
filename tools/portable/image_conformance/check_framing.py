"""Check every emitted PNG/BMP/PNM frame independently of the Jai decoders.

python3 check_framing.py CORPUS native-asan native-o2 wasi-o2
"""

from pathlib import Path
import json
import sys
import struct
import zlib
from collections import Counter

root = Path(sys.argv[1]).resolve()
counts = Counter()
for mode in sys.argv[2:]:
    for path in (root / ("core-" + mode)).iterdir():
        data = path.read_bytes()
        if path.suffix == ".png":
            assert data[:8] == b"\x89PNG\r\n\x1a\n"
            at = 8
            kinds = []
            compressed = bytearray()
            width = height = channels = 0
            while at < len(data):
                length, kind = struct.unpack_from(">I4s", data, at)
                body = data[at + 8 : at + 8 + length]
                crc = struct.unpack_from(">I", data, at + 8 + length)[0]
                assert crc == zlib.crc32(kind + body) & 0xFFFFFFFF, path
                kinds.append(kind)
                if kind == b"IHDR":
                    width, height, depth, color, compression, filtering, interlace = (
                        struct.unpack(">IIBBBBB", body)
                    )
                    assert (
                        depth == 8
                        and color in (0, 2, 4, 6)
                        and (compression, filtering, interlace) == (0, 0, 0)
                    ), path
                    channels = {0: 1, 2: 3, 4: 2, 6: 4}[color]
                elif kind == b"IDAT":
                    compressed.extend(body)
                elif kind == b"IEND":
                    assert length == 0
                at += 12 + length
            assert at == len(data) and kinds[0] == b"IHDR" and kinds[-1] == b"IEND"
            decoder = zlib.decompressobj()
            raw = decoder.decompress(bytes(compressed)) + decoder.flush()
            assert (
                decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail
            ), path
            assert len(raw) == height * (1 + width * channels), path
            assert all(raw[y * (1 + width * channels)] < 5 for y in range(height)), path
            counts["PNG valid chunks/CRCs/zlib/scanline framing"] += 1
        elif path.suffix == ".bmp":
            magic, size, reserved1, reserved2, offset = struct.unpack_from(
                "<2sIHHI", data
            )
            dib = struct.unpack_from("<I", data, 14)[0]
            width, height, planes, bits, compression, image_bytes = struct.unpack_from(
                "<iiHHII", data, 18
            )
            assert (
                magic == b"BM" and size == len(data) and reserved1 == reserved2 == 0
            ), path
            assert dib in (40, 108) and offset == 14 + dib, path
            assert (
                width > 0
                and height > 0
                and planes == 1
                and (bits, compression) in ((24, 0), (32, 3))
            ), path
            assert (
                image_bytes
                == height * ((width * bits + 31) // 32 * 4)
                == len(data) - offset
            ), path
            counts["BMP valid file/DIB framing"] += 1
        elif path.suffix == ".pnm":
            header, pixels = data.split(b"\n255\n", 1)
            magic, dimensions = header.split(b"\n")
            width, height = map(int, dimensions.split())
            assert magic in (b"P5", b"P6"), path
            assert len(pixels) == width * height * (1 if magic == b"P5" else 3), path
            counts["PNM valid P5/P6 framing"] += 1
print(dict(counts))
(root / "encoder-framing-result.json").write_text(
    json.dumps(dict(counts), indent=2) + "\n"
)
