#!/usr/bin/env python3
"""Build the real Rust interpreter as wasm and stage its browser runner."""
from pathlib import Path
import argparse
import os
import shutil
import subprocess


def main():
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--output", type=Path, default=root / "artifacts/scripting-runtime")
    args = parser.parse_args()
    if shutil.disk_usage(root).free < 2 * 1024**3:
        raise SystemExit("build requires at least 2 GiB of free disk space")
    command = ["cargo", "build", "--offline", "--locked", "-j", "1", "-p", "jai-wasm", "--target", "wasm32-unknown-unknown"]
    if args.release:
        command.append("--release")
    subprocess.run(command, cwd=root, env={**os.environ, "CARGO_INCREMENTAL": "0"}, check=True)
    target_root = Path(os.environ.get("CARGO_TARGET_DIR", root / "target"))
    if not target_root.is_absolute():
        target_root = root / target_root
    wasm = target_root / "wasm32-unknown-unknown" / ("release" if args.release else "debug") / "jai_wasm.wasm"
    if wasm.read_bytes()[:8] != b"\x00asm\x01\x00\x00\x00":
        raise SystemExit("compiler output is not a WebAssembly module")
    args.output.mkdir(parents=True, exist_ok=True)
    for source in (root / "web/scripting-runtime").iterdir():
        shutil.copy2(source, args.output / source.name)
    shutil.copy2(wasm, args.output / "jai_wasm.wasm")
    print(f"Browser runner: {args.output}")


if __name__ == "__main__":
    main()
