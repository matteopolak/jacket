#!/usr/bin/env python3
"""Check Windows Socket bodies, with explicit native execution on Windows only."""
import argparse
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", action="store_true")
    parser.add_argument("--jaic", type=Path, default=ROOT / "target/debug" / ("jaic.exe" if os.name == "nt" else "jaic"))
    parser.add_argument("--stdlib", type=Path, default=ROOT / "stdlib", help="combined production library; no overlays")
    args = parser.parse_args()
    if args.native and os.name != "nt":
        parser.error("native Winsock verification requires an actual Windows host")
    library = args.stdlib.resolve(strict=True)
    compiler = args.jaic.resolve(strict=True)
    source = ROOT / "tests/portable-stdlib/windows-socket/main.jai"
    env = dict(os.environ, JAIC_STDLIB=str(library))

    def checked(command, expected=None):
        result = subprocess.run([str(p) for p in command], cwd=ROOT, env=env,
                                text=True, capture_output=True, timeout=90)
        if result.returncode or result.stderr or (expected is not None and result.stdout != expected):
            raise RuntimeError(f"{command!r}\n{result.stdout}{result.stderr}")

    for cpu in ["x64", "arm64"]:
        checked([compiler, "-check", "-os", "windows", "-cpu", cpu, source])
        print(f"Windows/{cpu} Winsock wrappers/loopback/ABI/error bodies check", flush=True)
    if not args.native:
        print("Actual Winsock execution remains unverified; no native pass claimed")
        return
    checked([compiler, "-interp", source], "ok\n")
    for optimization in ["-O0", "-O2"]:
        output = ROOT / f"target/windows-socket{optimization}.exe"
        checked([compiler, source, optimization, "-exe", output])
        checked([output], "ok\n")
    print("Actual Winsock interpreter/native O0/O2 passed")


if __name__ == "__main__":
    main()
