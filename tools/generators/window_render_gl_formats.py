#!/usr/bin/env python3
"""Emit private GL texel descriptions from our reviewed-format/native-SDK mapping ledger."""

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import json
from pathlib import Path

from generators.jai_codegen import emit_generated, generator_parser

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = generator_parser(__doc__)
    args = parser.parse_args()
    metadata = json.loads((ROOT / "tools/generators/window-render-gl-formats.json").read_text())
    rows = metadata["scalar_formats"] + metadata["block_formats"]
    names = [row["format"] for row in rows]
    if len(set(names)) != len(names) or not set(metadata["srgb"]).issubset(names):
        parser.error("duplicate format or unknown sRGB mapping")
    lines = ["// Regenerate with tools/generators/window_render_gl_formats.py.", "#scope_module",
             "describe_gl_format :: (tf: Texture_Format, srgb: bool) -> GL_Image_Format, bool {",
             "    result: GL_Image_Format;", "    known := false;"]
    for row in metadata["scalar_formats"]:
        lines += [f'    if tf == .{row["format"]} {{',
                  f'        result = .{{GL.GL_{row["internal"]}, GL.GL_{row["external"]}, GL.GL_{row["scalar"]}}};',
                  "        known = true;"]
        if row["integer"]:
            lines.append("        result.integer = true;")
        lines.append("    }")
    for row in metadata["block_formats"]:
        lines += [f'    if tf == .{row["format"]} {{',
                  f'        result.internal = GL.GL_{row["internal"]};',
                  "        result.compressed = true;", "        known = true;", "    }"]
    lines += ["    if !known return .{}, false;", "    if srgb {"]
    first = True
    for name, internal in metadata["srgb"].items():
        branch = "if" if first else "else if"
        lines.append(f"        {branch} tf == .{name} result.internal = GL.GL_{internal};")
        first = False
    lines += ["        else return .{}, false;", "    }", "    return result, true;", "}"]
    emit_generated(ROOT / "stdlib/internal/rendering/gl-formats.jai", "\n".join(lines) + "\n", args.formatter, args.check)
    print(f"{'Verified' if args.check else 'Generated'} {len(rows)} native texel/block mappings")


if __name__ == "__main__":
    main()
