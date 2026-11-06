"""Generate complete QOI reference/opcode/negative cases using pinned official C code.

python3 prepare_qoi.py CORPUS
"""

from pathlib import Path
import ctypes as C
import subprocess
import struct
import json
import sys

root = Path(sys.argv[1]).resolve()
reference = root / "qoi-reference"
library = reference / "libqoi_reference.so"
subprocess.run(
    [
        "cc",
        "-O2",
        *(["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]),
        "-x",
        "c",
        "-DQOI_IMPLEMENTATION",
        str(reference / "qoi.h"),
        "-o",
        str(library),
    ],
    check=True,
)


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
inputs = root / "decode-inputs"
oracles = root / "decode-oracle"
basecases = json.loads((root / "decode-cases.json").read_text())
basecases = [c for c in basecases if not c["name"].startswith("qoi-")]
new = []


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


def add(
    name,
    data,
    required=1,
    group="qoi-reference",
    *,
    reference_accepts=None,
    source="official QOI reference ffb2d2cb74a1de60819b21b939f7209aa53e91c1",
):
    """Save one independently classified input; e.g. add("cut", data, required=0)."""
    name = "qoi-" + name + ".qoi"
    (inputs / name).write_bytes(data)
    result = decoded(data)
    if reference_accepts is None:
        reference_accepts = required == 1
    assert (result is not None) == reference_accepts, name
    if required == 1:
        assert result, name
        (desc, rgba) = result
        (oracles / (name + ".rgba")).write_bytes(rgba)
        width = desc.width
        height = desc.height
    else:
        width = height = 0
    new.append(
        {
            "name": name,
            "source": source,
            "reference_accepted": result is not None,
            "required": required,
            "width": width,
            "height": height,
            "delta": 0,
            "group": group,
        }
    )


for case in json.loads((root / "png-cases.json").read_text()):
    if not case["valid"]:
        continue
    rgba = (root / "oracle" / (case["name"] + ".rgba")).read_bytes()
    for channels in (3, 4):
        raw = (
            rgba
            if channels == 4
            else bytes(
                (v for p in range(len(rgba) // 4) for v in rgba[p * 4 : p * 4 + 3])
            )
        )
        buf = C.create_string_buffer(raw)
        for colorspace in (0, 1):
            desc = Desc(case["width"], case["height"], channels, colorspace)
            length = C.c_int()
            encoded = ref.qoi_encode(buf, C.byref(desc), C.byref(length))
            assert encoded
            try:
                data = C.string_at(encoded, length.value)
            finally:
                free(encoded)
            add(f"png-{case['name']}-c{channels}-s{colorspace}", data)
padding = b"\x00\x00\x00\x00\x00\x00\x00\x01"
streams = {
    "index": (
        128,
        b"".join((bytes((255, i * 43 & 63, 0, 0, 0)) for i in range(64)))
        + bytes(range(64)),
    ),
    "diff": (64, bytes(range(64, 128))),
    "luma": (
        64 * 256,
        bytes((v for a in range(128, 192) for b in range(256) for v in (a, b))),
    ),
    "run": (sum(range(1, 63)), bytes(range(192, 254))),
    "rgb": (
        256,
        bytes((v for i in range(256) for v in (254, i, 255 - i, i * 73 & 255))),
    ),
    "rgba": (
        256,
        bytes(
            (
                v
                for i in range(256)
                for v in (255, i * 3 & 255, i * 7 & 255, i * 11 & 255, i)
            )
        ),
    ),
}
for kind, (width, stream) in streams.items():
    for channels in (3, 4):
        for colorspace in (0, 1):
            add(
                f"op-{kind}-c{channels}-s{colorspace}",
                b"qoif"
                + struct.pack(">II", width, 1)
                + bytes((channels, colorspace))
                + stream
                + padding,
                group="qoi-opcodes",
            )
valid = (
    b"qoif" + struct.pack(">II", 1, 1) + bytes((4, 0, 255, 10, 20, 30, 40)) + padding
)
for length in range(22):
    add(f"negative-cut-{length}", valid[:length], 0, "qoi-negative")
for field, value in (
    (0, ord("x")),
    (12, 0),
    (12, 1),
    (12, 2),
    (12, 5),
    (12, 255),
    (13, 2),
    (13, 255),
):
    bad = bytearray(valid)
    bad[field] = value
    add(f"negative-header-{field}-{value}", bytes(bad), 0, "qoi-negative")
for offset in (4, 8):
    bad = bytearray(valid)
    bad[offset : offset + 4] = bytes(4)
    add(f"negative-zero-{offset}", bytes(bad), 0, "qoi-negative")
# The reference intentionally tolerates these malformed frames; the specification defines the file boundary.
normative = []
for offset in range(8):
    bad = bytearray(valid)
    bad[len(bad) - 8 + offset] ^= 0x40
    normative.append((f"normative-marker-{offset}", bytes(bad)))
for length in range(22, len(valid)):
    normative.append((f"normative-truncated-marker-{length}", valid[:length]))
normative.append(
    (
        "normative-run-overrun",
        b"qoif" + struct.pack(">II", 1, 1) + bytes((4, 0, 0xC1)) + padding,
    )
)
normative.append(("normative-unused-opcode", valid[:-8] + b"\0" + padding))
for name, data in normative:
    add(
        name,
        data,
        0,
        "qoi-normative-framing",
        reference_accepts=True,
        source="QOI specification 1.0 (2022-01-05), complete pixel count and eight-byte end marker",
    )
allcases = basecases + new
for c in allcases:
    c["codec"] = (
        "BMP"
        if c["name"].startswith("bmp-")
        else "PNM"
        if c["name"].startswith("pnm-")
        else "QOI"
    )
(root / "decode-cases.json").write_text(json.dumps(allcases, indent=2) + "\n")
(root / "decode_cases.jai").write_text(
    "Codec :: enum { BMP; PNM; QOI; }\nFile_Case :: struct { name: string; required: s32; width: s32; height: s32; delta: s32; codec: Codec; }\nCASES :: File_Case.[\n"
    + "".join(
        (
            "    .{"
            + json.dumps(c["name"])
            + f", {c['required']}, {c['width']}, {c['height']}, {c['delta']}, .{c['codec']}"
            + "},\n"
            for c in allcases
        )
    )
    + "];\n"
)
print("Prepared full QOI corpus", len(new), "cases", flush=True)
