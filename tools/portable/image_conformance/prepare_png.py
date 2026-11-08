"""Prepare all original PngSuite cases and pinned independent 8-bit RGBA oracles."""

import hashlib
import json
from pathlib import Path
import shutil
import sys
from PIL import Image

root = Path(sys.argv[1]).resolve()
manifest = json.loads((root / "source-manifest.json").read_text())
cases = manifest["png_cases"]
inputs, oracle = root / "inputs", root / "oracle"
inputs.mkdir(exist_ok=True)
oracle.mkdir(exist_ok=True)
for case in cases:
    name = case["name"]
    source = root / "pngsuite" / (name if name == "PngSuite.png" else "png/" + name)
    data = source.read_bytes()
    shutil.copyfile(source, inputs / name)
    if not case["valid"]:
        continue
    depth, color = data[24:26]
    with Image.open(source) as image:
        assert image.size == (case["width"], case["height"]), name
        transparency = image.info.get("transparency")
        if color == 0 and depth == 16:
            # The public codec returns the high byte; transparency compares original samples.
            rgba = bytes(
                value
                for sample in image.getdata()
                for value in ((sample >> 8),) * 3
                + (0 if sample == transparency else 255,)
            )
        else:
            rgba = bytearray(image.convert("RGBA").tobytes())
            if color == 0 and depth < 8 and transparency is not None:
                # PNG tRNS compares before rescaling. Pillow 10.4 compares an unscaled key.
                key = transparency * 255 // ((1 << depth) - 1)
                for pixel in range(len(rgba) // 4):
                    rgba[pixel * 4 + 3] = 0 if rgba[pixel * 4] == key else 255
            rgba = bytes(rgba)
    digest = hashlib.sha256(rgba).hexdigest()
    assert digest == case["oracle_sha256"], (name, "independent oracle changed", digest)
    (oracle / (name + ".rgba")).write_bytes(rgba)
(root / "png-cases.json").write_text(json.dumps(cases, indent=2) + "\n")
(root / "cases.jai").write_text(
    "Case :: struct { name: string; valid: bool; width: s32; height: s32; }\nCASES :: Case.[\n"
    + "".join(
        "    .{"
        + json.dumps(c["name"])
        + ", "
        + str(c["valid"]).lower()
        + f", {c['width']}, {c['height']}"
        + "},\n"
        for c in cases
    )
    + "];\n"
)
print(f"Prepared complete PngSuite: {len(cases)} cases")
