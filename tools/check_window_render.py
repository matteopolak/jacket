#!/usr/bin/env python3
"""Focused verification for this worker's supported paths, using source-built tools.

This does not claim integrated renderer, physical-device or cross-host conformance.
The macOS clipboard case writes and empties the actual host clipboard.
"""
import argparse
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, default=ROOT / "target/window-compiler/debug/jaic")
    parser.add_argument("--stdlib", type=Path, default=ROOT / "target/window-integration-stdlib")
    parser.add_argument("--native-libs", type=Path, default=ROOT / "target/window-native")
    parser.add_argument("--font", type=Path, help="caller-provided scalable font asset; never copied into the repository")
    args = parser.parse_args()
    compiler = args.jaic.resolve(strict=True)
    if not compiler.is_relative_to(ROOT / "target"):
        parser.error("use a source-built compiler inside this worker's target directory")
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve()),
               JAIC_NATIVE_LIBS=str(args.native_libs.resolve()))
    output = ROOT / "target/window-test-binaries"
    output.mkdir(parents=True, exist_ok=True)
    count = 0

    def run(command, expected=None):
        nonlocal count
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                capture_output=True, text=True, timeout=60)
        if result.returncode or result.stderr or (expected is not None and result.stdout != expected):
            raise RuntimeError(f'{command}\nexit {result.returncode}\n{result.stdout}{result.stderr}')
        count += 1

    host = ["srgb", "mapping", "bitmap-native"]
    if platform.system() in {"Darwin", "Windows"}:
        host.append("clipboard-host")
    if args.font:
        host.append("font-native")
    for name in host:
        source = ROOT / f"tests/window-render/{name}.jai"
        parameters = [args.font.resolve()] if name == "font-native" else []
        program_args = ["--", *parameters] if parameters else []
        run([compiler, source, "-interp", "-I", ROOT / "stdlib", *program_args], "ok\n")
        for optimization in ["O0", "O2"]:
            binary = output / f"{name}-{optimization}"
            run([compiler, source, f"-{optimization}", "-exe", binary, "-I", ROOT / "stdlib"])
            run([binary, *parameters], "ok\n")
        print(f"passed host interpreter and native O0/O2: {name}", flush=True)

    linux = ["types", "portable", "callback-types", "keycodes", "window-invalid", "bitmap-codec", "gamepad-virtual", "gamepad-shared-sdk", "texture-packing", "simp-none"]
    for name in linux:
        source = ROOT / f"tests/window-render/{name}.jai"
        run([compiler, source, "-interp", "-os", "linux", "-I", ROOT / "stdlib"], "ok\n")
        print(f"passed Linux-target interpreter: {name}", flush=True)

    if args.font:
        run([compiler, ROOT / "tests/window-render/font-public.jai", "-interp", "-os", "linux",
             "-I", ROOT / "stdlib", "--", args.font.resolve()], "ok\n")
        print("passed public font facade with native FreeType (Linux-target interpreter)", flush=True)

    for os_name in ["linux", "windows"]:
        for cpu in ["x64", "arm64"]:
            cases = ["types", "callback-types", "keycodes", "window-types", "texture-signatures", "simp-none"]
            cases += ["evdev-signatures", "window-invalid"] if os_name == "linux" else ["clipboard-host", "windows-queries"]
            for name in cases:
                run([compiler, ROOT / f"tests/window-render/{name}.jai", "-check", "-os", os_name,
                     "-cpu", cpu, "-I", ROOT / "stdlib"])
    visibility = subprocess.run([str(compiler), str(ROOT / "tests/window-render/private-helper-negative.jai"),
                                 "-check", "-os", "linux", "-I", str(ROOT / "stdlib")],
                                cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
    if visibility.returncode != 1 or visibility.stdout or "has no exported member `pack_bitmap`" not in visibility.stderr:
        raise RuntimeError(f"private-helper visibility regression: {visibility.stdout}{visibility.stderr}")
    count += 1
    print(f"passed {count} focused compiler/program invocations", flush=True)


if __name__ == "__main__":
    main()
