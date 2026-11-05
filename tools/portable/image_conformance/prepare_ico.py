"""Enumerate every pinned Pillow ICO frame, default selection and explicit invalid indices."""

from pathlib import Path
from PIL import Image
import io
import json
import struct
import sys

root = Path(sys.argv[1]).resolve()
manifest = json.loads((root / "source-manifest.json").read_text())
inventory = {
    "files": [
        {"path": f["path"].removeprefix("pillow-ico/assets/")}
        for f in manifest["files"]
        if f["path"].startswith("pillow-ico/assets/") and f["path"].endswith(".ico")
    ]
}
inputs = root / "ico-inputs"
oracle = root / "ico-oracle"
inputs.mkdir(exist_ok=True)
oracle.mkdir(exist_ok=True)
cases = []
for item in inventory["files"]:
    path = root / "pillow-ico/assets" / item["path"]
    data = path.read_bytes()
    name = item["path"].replace("/", "-")
    (inputs / name).write_bytes(data)
    count = struct.unpack_from("<H", data, 4)[0]
    areas = []
    frames = []
    for index in range(count):
        entry = data[6 + 16 * index : 22 + 16 * index]
        (w, h) = (entry[0] or 256, entry[1] or 256)
        (length, offset) = struct.unpack_from("<II", entry, 8)
        payload = data[offset : offset + length]
        png = payload.startswith(b"\x89PNG\r\n\x1a\n")
        supported = png
        if not png and len(payload) >= 40:
            (dib, bw, bh, planes, bits, compression) = struct.unpack_from(
                "<IiiHHI", payload
            )
            supported = (
                dib == 40
                and planes == 1
                and (bits in (24, 32))
                and (compression == 0)
                and (1 <= bw <= 256)
                and (2 <= bh <= 512)
                and (bh % 2 == 0)
            )
        expected = None
        if supported:
            singleton = (
                struct.pack("<HHH", 0, 1, 1)
                + entry[:12]
                + struct.pack("<I", 22)
                + payload
            )
            with Image.open(io.BytesIO(singleton)) as im:
                expected = im.convert("RGBA").tobytes()
                (w, h) = im.size
        frames.append((supported, w, h, expected))
        areas.append((entry[0] or 256) * (entry[1] or 256))
        cases.append(
            {
                "name": f"{name}-entry{index}",
                "file": name,
                "entry": index,
                "required": int(supported),
                "width": w,
                "height": h,
            }
        )
        if expected is not None:
            (oracle / (cases[-1]["name"] + ".rgba")).write_bytes(expected)
    chosen = areas.index(max(areas))
    (supported, w, h, expected) = frames[chosen]
    cases.append(
        {
            "name": name + "-default",
            "file": name,
            "entry": -1,
            "required": int(supported),
            "width": w,
            "height": h,
        }
    )
    if expected is not None:
        (oracle / (cases[-1]["name"] + ".rgba")).write_bytes(expected)
    for index in (count, count + 17):
        cases.append(
            {
                "name": f"{name}-invalid{index}",
                "file": name,
                "entry": index,
                "required": 0,
                "width": 0,
                "height": 0,
            }
        )
    for end in range(6 + 16 * count):
        truncated = f"{name}-prefix{end}"
        (inputs / truncated).write_bytes(data[:end])
        cases.append(
            {
                "name": truncated,
                "file": truncated,
                "entry": -1,
                "required": 0,
                "width": 0,
                "height": 0,
            }
        )
(root / "ico-cases.json").write_text(json.dumps(cases, indent=2) + "\n")
(root / "ico_cases.jai").write_text(
    "File_Case :: struct { name: string; file: string; entry: int; required: s32; width: s32; height: s32; }\nCASES :: File_Case.[\n"
    + "".join(
        (
            "    .{"
            + json.dumps(c["name"])
            + ", "
            + json.dumps(c["file"])
            + f", {c['entry']}, {c['required']}, {c['width']}, {c['height']}"
            + "},\n"
            for c in cases
        )
    )
    + "];\n"
)
print(
    "Complete pinned Pillow ICO corpus",
    len(inventory["files"]),
    "files",
    sum((1 for c in cases if "-entry" in c["name"])),
    "explicit frames;",
    len(cases),
    "actual cases",
    sum((c["required"] for c in cases)),
    "supported positives",
)
