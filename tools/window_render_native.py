#!/usr/bin/env python3
"""Build only our image codec dependencies from pinned public native C sources.

Uses repository dependency metadata; output stays inside this worker's target directory.
Does not inspect worktree metadata, other checkouts, supplied binaries or credentials.
"""
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def main():
    if platform.system() not in {"Darwin", "Linux"}:
        raise RuntimeError("this scoped native test builder supports macOS/Linux; Windows needs the coordinator's source-built native SDK libraries")
    metadata = json.loads((ROOT / "tools/native-libs.json").read_text())
    spec = metadata["sources"]["stb"]
    source = ROOT / "target/window-native-source"
    output = ROOT / "target/window-native"
    source.mkdir(parents=True, exist_ok=True)
    output.mkdir(parents=True, exist_ok=True)
    for name in ["stb_image.h", "stb_image_write.h", "LICENSE"]:
        path = source / name
        if not path.exists():
            url = f'https://raw.githubusercontent.com/{spec["repository"]}/{spec["revision"]}/{name}'
            path.write_bytes(urllib.request.urlopen(url, timeout=30).read())
        if hashlib.sha256(path.read_bytes()).hexdigest() != spec["files"][name]:
            raise RuntimeError(f"native source digest mismatch: {name}")
    extension = "dylib" if platform.system() == "Darwin" else "so"
    shared_flag = "-dynamiclib" if platform.system() == "Darwin" else "-shared"
    for name in ["stb_image", "stb_image_write"]:
        unit = source / f"{name}.c"
        unit.write_text(metadata["libraries"][name]["code"])
        obj = output / f"{name}.o"
        subprocess.run(["cc", "-O2", "-fPIC", "-I", str(source), "-c", str(unit), "-o", str(obj)], check=True)
        subprocess.run(["ar", "rcs", str(output / f"lib{name}.a"), str(obj)], check=True)
        subprocess.run(["cc", shared_flag, str(obj), "-o", str(output / f"lib{name}.{extension}"), "-lm"], check=True)
    if platform.system() == "Darwin":
        probe = ROOT / "tests/window-render/clipboard-inspect.m"
        obj = output / "window_clipboard_probe.o"
        subprocess.run(["cc", "-O2", "-fobjc-arc", "-I", str(source), "-c", str(probe), "-o", str(obj)], check=True)
        decoder = output / "stb_image.o"
        subprocess.run(["ar", "rcs", str(output / "libwindow_clipboard_probe.a"), str(obj), str(decoder)], check=True)
        subprocess.run(["cc", "-dynamiclib", str(obj), str(decoder), "-framework", "AppKit", "-o",
                        str(output / "libwindow_clipboard_probe.dylib")], check=True)
    print(output)


if __name__ == "__main__":
    main()
