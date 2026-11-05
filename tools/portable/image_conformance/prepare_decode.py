"""Prepare every BMP Suite and pinned PNM case with independent pixel oracles.

python3 prepare_decode.py CORPUS
"""

from pathlib import Path
from html.parser import HTMLParser
from PIL import Image
import json
import shutil
import struct
import re
import collections
import sys

root = Path(sys.argv[1]).resolve()
inputs = root / "decode-inputs"
oracle = root / "decode-oracle"
inputs.mkdir(exist_ok=True)
oracle.mkdir(exist_ok=True)
cases = []


class Rows(HTMLParser):
    def __init__(self):
        super().__init__()
        self.rows = []
        self.cells = None
        self.at = None

    def handle_starttag(self, tag, attrs):
        if tag == "tr":
            self.cells = []
        if tag == "td" and self.cells is not None:
            self.cells.append({"text": "", "images": []})
            self.at = self.cells[-1]
        if tag == "img" and self.at is not None:
            self.at["images"].append(dict(attrs)["src"])

    def handle_endtag(self, tag):
        if tag == "td":
            self.at = None
        if tag == "tr" and self.cells is not None:
            self.rows.append(self.cells)
            self.cells = None
            self.at = None

    def handle_data(self, text):
        if self.at is not None:
            self.at["text"] += text


bmp = root / "bmpsuite" / "bmpsuite-2.8"
parser = Rows()
parser.feed((bmp / "html/bmpsuite.html").read_text())
refs = {
    row[0]["text"].strip(): row[2]["images"] for row in parser.rows if len(row) >= 3
}
counts = collections.Counter()
for path in sorted(bmp.glob("*/*.bmp")):
    rel = str(path.relative_to(bmp))
    group = rel.split("/")[0]
    if group not in ("g", "q", "b", "x"):
        continue
    name = "bmp-" + rel.replace("/", "-")
    shutil.copyfile(path, inputs / name)
    required = 1 if group == "g" else -1
    width = height = 0
    expected = None
    if group == "g":
        data = path.read_bytes()
        header = struct.unpack_from("<I", data, 14)[0]
        (width, height) = struct.unpack_from("<HH" if header == 12 else "<ii", data, 18)
        height = abs(height)
        possible = []
        for image in refs[rel]:
            reference = (bmp / "html" / image).resolve()
            with Image.open(reference) as im:
                if im.size == (width, height):
                    possible.append(
                        (str(reference.relative_to(bmp)), im.convert("RGBA").tobytes())
                    )
        assert possible, (rel, width, height, refs[rel])
        expected = possible[0][1]
        (oracle / (name + ".rgba")).write_bytes(expected)
    cases.append(
        {
            "name": name,
            "source": "BMP Suite2.8:" + rel,
            "required": required,
            "width": width,
            "height": height,
            "delta": 0,
            "group": "bmp-" + group,
        }
    )
    counts["bmp-" + group] += 1
pnm = root / "codec-corpus/pnm-conformance"
for path in sorted(pnm.rglob("*")):
    if path.suffix.lower() not in (".pnm", ".pbm", ".pgm", ".ppm", ".pam"):
        continue
    rel = str(path.relative_to(pnm))
    data = path.read_bytes()
    name = "pnm-" + rel.replace("/", "-")
    shutil.copyfile(path, inputs / name)
    unsupported = data[:2] in (b"P1", b"P4", b"P7")
    required = 0 if rel.startswith("invalid/") or unsupported else 1
    width = height = 0
    delta = 0
    if required == 1:
        tokens = re.findall(b"#[^\\r\\n]*|[^\\s#]+", data)
        tokens = [t for t in tokens if not t.startswith(b"#")]
        maxval = int(tokens[3])
        delta = 1 if maxval != 255 else 0
        with Image.open(path) as im:
            (width, height) = im.size
            if im.mode.startswith("I"):
                expected = bytes(
                    (
                        v
                        for sample in im.getdata()
                        for v in (round(sample * 255 / maxval),) * 3 + (255,)
                    )
                )
            else:
                expected = im.convert("RGBA").tobytes()
        (oracle / (name + ".rgba")).write_bytes(expected)
    cases.append(
        {
            "name": name,
            "source": "codec-corpus:" + rel,
            "required": required,
            "width": width,
            "height": height,
            "delta": delta,
            "group": "pnm-"
            + (
                "unsupported"
                if unsupported
                else "invalid"
                if rel.startswith("invalid/")
                else "valid"
            ),
        }
    )
    counts[cases[-1]["group"]] += 1
(root / "decode-cases.json").write_text(json.dumps(cases, indent=2) + "\n")
(root / "decode_cases.jai").write_text(
    "File_Case :: struct { name: string; required: s32; width: s32; height: s32; delta: s32; }\nCASES :: File_Case.[\n"
    + "".join(
        (
            "    .{"
            + json.dumps(c["name"])
            + f", {c['required']}, {c['width']}, {c['height']}, {c['delta']}"
            + "},\n"
            for c in cases
        )
    )
    + "];\n"
)
print("Full decoder corpora", len(cases), dict(counts))
print(
    "PNM only P2/P3/P5/P6 are declared; all16 PBM/PAM cases explicitly tested for rejection. BMP g mandatory pixels; all q/b/x executed for safe handling per original author suite rules."
)
