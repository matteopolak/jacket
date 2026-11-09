#!/usr/bin/env python3
"""Build docs/images/banner.svg: the jacket logo beside a hand-drawn "Jacket".

The wordmark is Lilita One (SIL Open Font License 1.1) converted to outlines
and run through rough.js, so it matches the logo's double-stroke style and
renders in <img> tags without any font. The logo is embedded unchanged. The
wordmark is ink on light backgrounds and off-white on dark ones.

Rerun (needs Node, npm and network access; nothing shipped needs them):

    python3 -m venv /tmp/banner-venv && /tmp/banner-venv/bin/pip install fonttools
    curl -fsSLo /tmp/LilitaOne-Regular.ttf \
      https://raw.githubusercontent.com/google/fonts/main/ofl/lilitaone/LilitaOne-Regular.ttf
    PYTHONPATH=tools /tmp/banner-venv/bin/python tools/dev/readme_banner.py /tmp/LilitaOne-Regular.ttf

Tweak WORD, KERN, ROUGH or the layout constants below and rerun. The output is
deterministic for a given seed.
"""
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont
from toolkit.repo import ROOT

LOGO = ROOT / "docs/images/logo.svg"
OUT = ROOT / "docs/images/banner.svg"

WORD = "Jacket"
KERN = {"J": 10, "a": 6, "c": 10, "k": 12, "e": 14}  # extra advance (font units) after a glyph, e.g. {"J": -20}
FONT_SIZE = 150  # px; sets the wordmark height
WIDTH, HEIGHT = 614, 190
LOGO_SIZE = 150  # logo is drawn square at this size
GAP = 26  # between logo and wordmark
ROUGH = {"roughness": 0.75, "bowing": 1.2, "strokeWidth": 2.4, "seed": 7, "maxRandomnessOffset": 1.6}
INK = "oklch(31.04% 0.0088 116.10)"  # the logo's outline colour
INK_DARK = "oklch(94% 0.01 85)"


def glyph_paths(font_file):
    font = TTFont(font_file)
    glyphs, cmap = font.getGlyphSet(), font.getBestCmap()
    scale = FONT_SIZE / font["head"].unitsPerEm
    x, items = 0.0, []
    for ch in WORD:
        name = cmap[ord(ch)]
        items.append((name, x))
        x += glyphs[name].width + KERN.get(ch, 0)
    # Ink bounds of the whole word, to centre it.
    bounds = [BoundsPen(glyphs) for _ in items]
    for (name, ox), pen in zip(items, bounds):
        glyphs[name].draw(pen)
    left = min(b.bounds[0] + ox for (_, ox), b in zip(items, bounds))
    right = max(b.bounds[2] + ox for (_, ox), b in zip(items, bounds))
    top = max(b.bounds[3] for b in bounds)
    bottom = min(b.bounds[1] for b in bounds)
    text_w = (right - left) * scale
    x0 = LOGO_SIZE + GAP
    assert x0 + text_w < WIDTH, "wordmark does not fit; raise WIDTH"
    y_mid = HEIGHT / 2
    out = []
    for name, ox in items:
        pen = SVGPathPen(glyphs, ntos=lambda v: f"{v:.2f}")
        t = (scale, 0, 0, -scale, x0 + (ox - left) * scale, y_mid + (top + bottom) / 2 * scale)
        glyphs[name].draw(TransformPen(pen, t))
        out.append(pen.getCommands())
    return out


def rough(paths):
    with tempfile.TemporaryDirectory() as tmp:
        subprocess.run(["npm", "install", "--silent", "--no-audit", "--no-fund", "roughjs"], cwd=tmp, check=True)
        script = Path(__file__).with_name("readme_banner_rough.ts")
        shutil_copy = Path(tmp) / script.name
        shutil_copy.write_text(script.read_text())
        res = subprocess.run(
            ["node", str(shutil_copy)],
            cwd=tmp, check=True, capture_output=True, text=True,
            input=json.dumps({"paths": paths, "options": ROUGH}),
        )
    return json.loads(res.stdout)


def short(d):
    return re.sub(r"-?\d+\.\d+", lambda m: f"{float(m.group()):.1f}".rstrip("0").rstrip("."), d)


def main():
    drawn = rough(glyph_paths(sys.argv[1]))
    logo = LOGO.read_text()
    vb = re.search(r'viewBox="([^"]+)"', logo).group(1)
    inner = re.search(r"</desc>\s*(.*)</svg>", logo, re.S).group(1).strip()
    fills = "".join(f'<path d="{short(d)}" fill-rule="evenodd"/>' for g in drawn for d in g["fill"])
    strokes = "".join(f'<path d="{short(d)}"/>' for g in drawn for d in g["stroke"])
    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-labelledby="title"><title id="title">Jacket</title>
<style>.fill{{fill:{INK}}}.edge{{stroke:{INK}}}@media (prefers-color-scheme:dark){{.fill{{fill:{INK_DARK}}}.edge{{stroke:{INK_DARK}}}}}</style>
<svg x="0" y="{(HEIGHT - LOGO_SIZE) / 2}" width="{LOGO_SIZE}" height="{LOGO_SIZE}" viewBox="{vb}">
{inner}
</svg>
<g class="fill" stroke="none">{fills}</g>
<g class="edge" fill="none" stroke-width="{ROUGH['strokeWidth']}" stroke-linecap="round" stroke-linejoin="round">{strokes}</g>
</svg>
"""
    OUT.write_text(svg)
    print(f"{OUT} {len(svg)} bytes")


main()
