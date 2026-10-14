#!/usr/bin/env python3
"""Emit the reviewed Gamepad native declaration inventory in independent ABI groups.

Only reviewed public declarations are inputs. Contextual native C types are maintained
separately from public SDK headers in internal/gamepad/linux-abi.jai.
"""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def responsibility(name):
    if name.startswith("libevdev_uinput"):
        return "output"
    if name in {"libevdev_new", "libevdev_new_from_fd", "libevdev_free"}:
        return "allocation"
    if any(word in name for word in ["from_name", "get_name", "_is_type", "_is_code", "from_code_name", "get_max"]):
        return "text"
    if any(word in name for word in ["event", "slot", "abs_"]):
        return "events"
    if any(word in name for word in ["fd", "grab", "id_", "clock", "phys", "uniq", "log"]):
        return "devices"
    return "capabilities"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--formatter", type=Path, default=ROOT / "target/window-jaifmt")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    source = (ROOT / "docs/specs/stdlib/modules/gamepad.md").read_text()
    declarations = re.findall(r"^- (?:\[[^]]+\] )?`(libevdev[^`]+#foreign evdev_library)`", source, re.MULTILINE)
    if not declarations:
        parser.error("reviewed native declaration inventory is empty")
    groups = {name: [] for name in ["allocation", "devices", "capabilities", "events", "text", "output"]}
    for declaration in declarations:
        name = declaration.split(" ::", 1)[0]
        groups[responsibility(name)].append(declaration + ";")
    text = "// Regenerate with tools/window_render_evdev.py and the source-built repository formatter.\n"
    text += "libevdev :: struct {}\nlibevdev_uinput :: struct {}\n\n"
    text += "\n\n".join("// " + name + "\n" + "\n".join(sorted(group)) for name, group in groups.items())
    text += '\n\n#load "../internal/gamepad/linux-abi.jai";\n'
    staging = ROOT / "target/window-generated/evdev.jai"
    staging.parent.mkdir(parents=True, exist_ok=True)
    staging.write_text(text)
    subprocess.run([str(args.formatter.resolve(strict=True)), "--config", str(ROOT / "jaifmt.toml"), str(staging)], check=True)
    destination = ROOT / "stdlib/Gamepad/evdev.jai"
    result = staging.read_bytes()
    if args.check:
        if destination.read_bytes() != result:
            raise SystemExit("native declaration regeneration differs")
    else:
        destination.write_bytes(result)
    print(f"verified {len(declarations)} reviewed native declarations")


if __name__ == "__main__":
    main()
