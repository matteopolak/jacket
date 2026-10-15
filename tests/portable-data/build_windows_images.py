"""Build the reviewed real stb SDK DLLs from a Windows developer command prompt.

This reads producer pins without editing shared manifests. No Jai source is
downloaded, and no non-Windows execution is represented as a native build.
"""
from pathlib import Path
import argparse
import json
import os
import subprocess
import sys

if os.name != "nt":
    raise SystemExit("Native Windows is required")
root = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser()
parser.add_argument("--cpu", choices=("x64", "arm64"), default="x64")
args = parser.parse_args()
sys.path.insert(0, str(root / "tools/native_bindings"))
from build_dependencies import prepare_source

pins = json.loads((root / "tools/native_bindings/dependencies.json").read_text())
sdk = root / ".work/vendor"
prepare_source("stb", pins["stb"], sdk)
output = root / "target/portable-data/native"
output.mkdir(parents=True, exist_ok=True)
triple = "x86_64-pc-windows-msvc" if args.cpu == "x64" else "aarch64-pc-windows-msvc"
for name, macro, definition, header in (
    ("stb_image", "STB_IMAGE_IMPLEMENTATION", "STBIDEF", "stb_image.h"),
    ("stb_image_resize", "STB_IMAGE_RESIZE_IMPLEMENTATION", "STBIRDEF",
     "deprecated/stb_image_resize.h"),
):
    unit = output / f"{name}-windows.c"
    unit.write_text(f"#define {macro}\n#define {definition} __declspec(dllexport)\n"
                    f'#include "{header}"\n')
    subprocess.run(["clang-cl", f"--target={triple}", "/nologo", "/O2", "/LD",
                    f"/I{sdk / 'stb'}", str(unit),
                    f"/Fo{output / (name + '.obj')}",
                    f"/Fe{output / ('lib' + name + '.dll')}", "/link",
                    f"/implib:{output / (name + '.lib')}"],
                   cwd=output, check=True)
print("Real Windows stb SDK DLLs and import libraries built")
