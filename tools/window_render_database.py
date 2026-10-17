#!/usr/bin/env python3
"""Normalize the public SDL 2.0.9 native C controller database into Jai data.

Input is the official SDL C SDK source header, never a Jai implementation.
The selected artifact digest is verified before generation. No network access occurs.
"""
import argparse
import ast
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
URL = "https://raw.githubusercontent.com/libsdl-org/SDL/release-2.0.9/src/joystick/SDL_gamecontrollerdb.h"
PLATFORMS = {
    "#if SDL_JOYSTICK_XINPUT": "Windows",
    "#if SDL_JOYSTICK_DINPUT": "Windows",
    "#if defined(__MACOSX__)": "Mac OS X",
    "#if defined(__LINUX__)": "Linux",
    "#if defined(__ANDROID__)": "Android",
    "#if defined(SDL_JOYSTICK_MFI)": "iOS",
    "#if defined(SDL_JOYSTICK_EMSCRIPTEN)": "Emscripten",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--sha256", required=True)
    args = parser.parse_args()
    source = args.source.read_bytes()
    digest = hashlib.sha256(source).hexdigest()
    if digest != args.sha256:
        parser.error(f"native SDL source digest mismatch: {digest}")
    text = source.decode("utf-8")
    license_text = text[:text.index("*/") + 2]
    records = []
    platform = ""
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("#if"):
            if line not in PLATFORMS:
                parser.error(f"unreviewed platform conditional: {line}")
            platform = PLATFORMS[line]
        elif line == "#endif":
            platform = ""
        elif line.startswith('"'):
            literal = re.match(r'"(?:[^"\\]|\\.)*"', line)
            if literal is None:
                parser.error("malformed native mapping string")
            mapping = ast.literal_eval(literal.group())
            if platform:
                mapping += f"platform:{platform},"
            records.append((platform, mapping.split(",", 1)[0], mapping))
    # Independent organization by platform and device identity, stable for duplicates.
    records.sort(key=lambda item: item[:2])
    payload = "# Derived from the official SDL 2.0.9 native controller database.\n"
    payload += "# Normalized and platform-qualified; see tools/window-render-sdl-database.json.\n"
    payload += "".join(record + "\n" for _, _, record in records)
    out = ROOT / "stdlib/Gamepad/SDL_Input_Map/database.jai"
    out.write_text(license_text + "\n// Altered data extraction: platform-qualified and sorted by device identity.\n"
                   + "SDL_BUILTIN_GAME_CONTROLLER_DB :: #string SDL_DATABASE\n"
                   + payload + "SDL_DATABASE\n", encoding="utf-8")
    metadata = {"source_url": URL, "source_sha256": digest, "record_count": len(records),
                "normalized_payload_sha256": hashlib.sha256(payload.encode()).hexdigest(),
                "normalization": "LF, explicit platform metadata, stable platform/GUID sorting",
                "license": "SDL zlib notice retained in database.jai"}
    (ROOT / "tools/window-render-sdl-database.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(f"generated {len(records)} native SDK mapping records")


if __name__ == "__main__":
    main()
