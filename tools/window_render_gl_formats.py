#!/usr/bin/env python3
"""Emit private GL texel descriptions from our reviewed-format/native-SDK mapping ledger."""
import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--formatter", type=Path, default=ROOT / "target/window-jaifmt")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    metadata = json.loads((ROOT / "tools/window-render-gl-formats.json").read_text())
    rows = metadata["scalar_formats"] + metadata["block_formats"]
    names = [row["format"] for row in rows]
    if len(set(names)) != len(names) or not set(metadata["srgb"]).issubset(names):
        parser.error("duplicate format or unknown sRGB mapping")
    lines = ["// Regenerate with tools/window_render_gl_formats.py.", "#scope_module",
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
    staging = ROOT / "target/window-generated/gl-formats.jai"
    staging.parent.mkdir(parents=True, exist_ok=True)
    staging.write_text("\n".join(lines) + "\n")
    subprocess.run([str(args.formatter.resolve(strict=True)), "--config", str(ROOT / "jaifmt.toml"), str(staging)], check=True)
    destination = ROOT / "stdlib/internal/rendering/gl-formats.jai"
    result = staging.read_bytes()
    if args.check:
        if destination.read_bytes() != result:
            raise SystemExit("GL format regeneration differs")
    else:
        destination.write_bytes(result)
    print(f"verified {len(rows)} native texel/block mappings")


if __name__ == "__main__":
    main()
