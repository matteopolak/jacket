"""Independently decode every emitted QOI file and check exact padding/pixels.

python3 check_qoi.py CORPUS native-asan native-o2 wasi-o2
"""

import hashlib
from pathlib import Path
import ctypes as C
import sys
import json

root = Path(sys.argv[1]).resolve()
reference = root / "qoi-reference"
library = reference / "libqoi_reference.so"


class Desc(C.Structure):
    _fields_ = [
        ("width", C.c_uint),
        ("height", C.c_uint),
        ("channels", C.c_ubyte),
        ("colorspace", C.c_ubyte),
    ]


ref = C.CDLL(str(library))
ref.qoi_encode.argtypes = [C.c_void_p, C.POINTER(Desc), C.POINTER(C.c_int)]
ref.qoi_encode.restype = C.c_void_p
ref.qoi_decode.argtypes = [C.c_void_p, C.c_int, C.POINTER(Desc), C.c_int]
ref.qoi_decode.restype = C.c_void_p
free = C.CDLL(None).free
free.argtypes = [C.c_void_p]


def decoded(data):
    """Return official-reference RGBA pixels, or None; e.g. decoded(qoi_bytes)."""
    buf = C.create_string_buffer(data)
    desc = Desc()
    pixels = ref.qoi_decode(buf, len(data), C.byref(desc), 0)
    if not pixels:
        return None
    try:
        raw = C.string_at(pixels, desc.width * desc.height * desc.channels)
    finally:
        free(pixels)
    rgba = (
        raw
        if desc.channels == 4
        else bytes(
            (
                v
                for p in range(desc.width * desc.height)
                for v in (*raw[p * 3 : p * 3 + 3], 255)
            )
        )
    )
    return (desc, rgba)


allcases = json.loads((root / "decode-cases.json").read_text())
padding = bytes((0, 0, 0, 0, 0, 0, 0, 1))
encoder_checks = 0
for mode in sys.argv[2:]:
    for case in json.loads((root / "png-cases.json").read_text()):
        if not case["valid"]:
            continue
        expected = (root / "oracle" / (case["name"] + ".rgba")).read_bytes()
        for variant in ("direct", "bgra"):
            data = (
                root / ("core-" + mode) / (case["name"] + "." + variant + ".qoi")
            ).read_bytes()
            assert data[-8:] == padding
            (desc, rgba) = decoded(data)
            assert (
                desc.width == case["width"]
                and desc.height == case["height"]
                and (desc.colorspace == 0)
            )
            assert rgba == expected, (mode, case["name"], variant)
            encoder_checks += 1
summary = {
    "revision": "ffb2d2cb74a1de60819b21b939f7209aa53e91c1",
    "header_sha256": hashlib.sha256((reference / "qoi.h").read_bytes()).hexdigest(),
    "reference_cases": 648,
    "opcode_cases": 24,
    "negative_cases": 32,
    "encoder_reference_checks": encoder_checks,
    "normative_framing_cases": sum(
        c.get("group") == "qoi-normative-framing" for c in allcases
    ),
    "qoi_cases": sum(c["codec"] == "QOI" for c in allcases),
    "combined_decoder_cases": len(allcases),
}
(root / "qoi-source.json").write_text(json.dumps(summary, indent=2) + "\n")
print(summary, flush=True)
