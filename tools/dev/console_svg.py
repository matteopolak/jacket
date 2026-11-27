#!/usr/bin/env python3
"""Render terminal output, colours included, as SVG windows for the changelog.

Each image in SHOTS is either a capture a guide already keeps (`<guide>.console.json`,
what `jacket doc` recorded with colour on) or a command run here on a small source file,
with FORCE_COLOR=1 and the Unicode diagnostic layout. The output is real; only the window around it is drawn.

    CARGO_TARGET_DIR=... JAIC_STDLIB=$PWD/stdlib cargo build -p jaic-cli -p jailint
    PYTHONPATH=tools python3 tools/dev/console_svg.py [NAME...]

writes one SVG per shot into docs/images/console/ (all shots by default, or the ones named).
The changelog shows them two to a row, in SHOTS order; both images of a row are as wide as
the row's longest line, so they scale alike side by side. To add one, add a Shot below
and rerun.
"""
import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from html import escape
from pathlib import Path

from toolkit.repo import ROOT, cargo_binary

OUT = ROOT / "docs" / "images" / "console"

FONT = "ui-monospace, 'SF Mono', SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace"
SANS = "-apple-system, 'Segoe UI', Helvetica, Arial, sans-serif"
SIZE = 13  # font size in px
CHAR = SIZE * 0.6  # advance of one monospace column
LINE = 16  # line height in px: about 1.2em, so box-drawing verticals join up
PAD = 20  # padding inside the window
BAR = 36  # title bar height
MARGIN = 18  # room around the window for its shadow

FG = "#d5dae3"
PALETTE = {  # SGR 30-37, then 90-97
    30: "#4b5263", 31: "#ff6b6b", 32: "#6bd68f", 33: "#f2c76e",
    34: "#6cb2ff", 35: "#d291f0", 36: "#5fd4dc", 37: "#d5dae3",
    90: "#6b7385", 91: "#ff8787", 92: "#8ee6a8", 93: "#f7d58f",
    94: "#93c7ff", 95: "#e0aef5", 96: "#86e3e9", 97: "#f4f6fa",
}
DIM = "#7d8596"
PROMPT = "#8a7bff"


@dataclass
class Shot:
    """One image: its file name, window title, and where its output comes from."""

    name: str
    title: str
    # A guide capture: (guide stem, block index, command index within the block).
    capture: tuple[str, int, int] | None = None
    # Or a command run in a folder holding `files`, shown as `$ command`.
    command: str | None = None
    files: dict[str, str] = field(default_factory=dict)


SIZES = """\
#import "Basic";

main :: () {
    total: s64 = 70000;
    small: s16 = total;
    print("%\\n", small);
}
"""

FREED = """\
#import "Basic";

main :: () {
    buffer := alloc(64);
    free(buffer);
    memset(buffer, 0, 64);
}
"""

SHOTS = [
    Shot("test-pass", "jacket test", capture=("testing", 0, 0)),
    Shot("test-fail", "jacket test: a failed expectation", capture=("testing", 2, 0)),
    Shot("test-property", "jacket test: a shrunk counterexample", capture=("testing", 6, 0)),
    Shot("test-golden", "jacket test: golden output", capture=("testing", 9, 0)),
    Shot("bench-baseline", "jacket bench against a baseline", capture=("benchmarking", 4, 0)),
    Shot("run-board", "jacket run on a board under QEMU", capture=("other-targets", 4, 0)),
    Shot("jaic-error", "jaic -check", command="jaic -check sizes.jai", files={"sizes.jai": SIZES}),
    Shot("jailint-error", "jailint", command="jailint freed.jai", files={"freed.jai": FREED}),
]

SGR = re.compile(r"\x1b\[([0-9;]*)m")


def spans(line: str) -> list[tuple[str, dict]]:
    """Split one line of output into (text, style) runs, following its SGR escapes."""
    style: dict = {}
    out = []
    pos = 0
    for match in SGR.finditer(line):
        if match.start() > pos:
            out.append((line[pos:match.start()], dict(style)))
        codes = [int(c) for c in match.group(1).split(";") if c] or [0]
        for code in codes:
            if code == 0:
                style = {}
            elif code == 1:
                style["bold"] = True
            elif code == 2:
                style["dim"] = True
            elif code == 22:
                style.pop("bold", None)
                style.pop("dim", None)
            elif code == 39:
                style.pop("fg", None)
            elif code in PALETTE:
                style["fg"] = PALETTE[code]
        pos = match.end()
    if pos < len(line):
        out.append((line[pos:], dict(style)))
    return out


def visible(line: str) -> str:
    """The text of `line` without its escapes."""
    return SGR.sub("", line)


def text_of(text: str) -> str:
    """`text` escaped for SVG, with non-breaking spaces: some renderers collapse runs of spaces even under `white-space: pre`."""
    return escape(text, quote=False).replace(" ", "\u00a0")


def tspan(text: str, style: dict) -> str:
    colour = style.get("fg") or (DIM if style.get("dim") else FG)
    weight = ' font-weight="700"' if style.get("bold") else ""
    return f'<tspan fill="{colour}"{weight}>{text_of(text)}</tspan>'


def render(title: str, command: str, output: str, columns: int) -> str:
    """The SVG of a window titled `title` showing `$ command` and then `output`."""
    lines = output.rstrip("\n").split("\n")
    body_w = columns * CHAR
    width = round(body_w + 2 * PAD)
    height = BAR + PAD + LINE * (len(lines) + 1) + PAD - (LINE - SIZE)
    full_w, full_h = width + 2 * MARGIN, height + 2 * MARGIN
    x0, y0 = MARGIN, MARGIN
    text_x = x0 + PAD
    rows = [
        f'<text x="{text_x}" y="{y0 + BAR + PAD + SIZE}">'
        f'<tspan fill="{PROMPT}" font-weight="700">$</tspan>'
        f'<tspan fill="{PALETTE[97]}">{text_of(" " + command)}</tspan></text>'
    ]
    for i, line in enumerate(lines, 1):
        y = y0 + BAR + PAD + SIZE + i * LINE
        runs = "".join(tspan(t, s) for t, s in spans(line))
        rows.append(f'<text x="{text_x}" y="{y}">{runs}</text>')
    dots = "".join(
        f'<circle cx="{x0 + 20 + 18 * i}" cy="{y0 + BAR / 2}" r="5.5" fill="{c}"/>'
        for i, c in enumerate(["#ff5f57", "#febc2e", "#28c840"])
    )
    return f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {full_w} {full_h}" width="{full_w}" height="{full_h}" role="img" aria-label="{escape(command)}">
<defs>
<linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1a1f2b"/><stop offset="1" stop-color="#12151d"/></linearGradient>
<filter id="shadow" x="-10%" y="-10%" width="120%" height="130%"><feDropShadow dx="0" dy="6" stdDeviation="7" flood-color="#000" flood-opacity="0.35"/></filter>
</defs>
<rect x="{x0}" y="{y0}" width="{width}" height="{height}" rx="12" fill="url(#bg)" stroke="#2c3343" filter="url(#shadow)"/>
<path d="M{x0} {y0 + BAR}H{x0 + width}" stroke="#262c3a"/>
{dots}
<text x="{x0 + width / 2}" y="{y0 + BAR / 2 + 4.5}" text-anchor="middle" font-family="{SANS}" font-size="12.5" fill="#8b93a5">{escape(title, quote=False)}</text>
<g font-family="{FONT}" font-size="{SIZE}" xml:space="preserve" style="white-space:pre">
{chr(10).join(rows)}
</g>
</svg>
"""


def captured(guide: str, block: int, index: int) -> tuple[str, str]:
    """The command and coloured output a guide's capture recorded."""
    data = json.loads((ROOT / "docs" / "guides" / f"{guide}.console.json").read_text(encoding="utf-8"))
    entry = data["blocks"][block][index]
    return entry["command"], entry["output"]


def run(shot: Shot) -> tuple[str, str]:
    """Run `shot.command` in a folder holding its files, with colour and Unicode diagnostics on."""
    tool, *args = shot.command.split()
    exe = cargo_binary(tool)
    if not exe.is_file():
        sys.exit(f"console_svg: {exe} is missing; build it with `cargo build -p {tool}`")
    env = {**os.environ, "FORCE_COLOR": "1", "JAIC_DIAGNOSTICS": "unicode"}
    if "JAIC_STDLIB" not in env:
        sys.exit("console_svg: set JAIC_STDLIB to the stdlib folder, e.g. JAIC_STDLIB=$PWD/stdlib")
    with tempfile.TemporaryDirectory() as folder:
        for name, text in shot.files.items():
            (Path(folder) / name).write_text(text, encoding="utf-8")
        done = subprocess.run([exe, *args], cwd=folder, env=env, capture_output=True, text=True)
    return shot.command, done.stdout + done.stderr


def output_of(shot: Shot) -> tuple[str, str]:
    return captured(*shot.capture) if shot.capture else run(shot)


def main(names: list[str]) -> int:
    unknown = set(names) - {s.name for s in SHOTS}
    if unknown:
        sys.exit(f"console_svg: no shot named {', '.join(sorted(unknown))}")
    outputs = {s.name: output_of(s) for s in SHOTS}
    widest = {
        name: max(len(visible(line)) for line in [f"$ {command}", *text.split("\n")])
        for name, (command, text) in outputs.items()
    }
    # Shots pair up into the changelog's two-column rows; a row shares one width.
    columns = {}
    for i in range(0, len(SHOTS), 2):
        row = SHOTS[i : i + 2]
        width = max(widest[s.name] for s in row) + 1
        columns.update({s.name: width for s in row})
    OUT.mkdir(parents=True, exist_ok=True)
    for shot in SHOTS:
        if names and shot.name not in names:
            continue
        command, text = outputs[shot.name]
        (OUT / f"{shot.name}.svg").write_text(render(shot.title, command, text, columns[shot.name]), encoding="utf-8")
        print(f"wrote docs/images/console/{shot.name}.svg")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
