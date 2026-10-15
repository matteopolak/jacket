#!/usr/bin/env python3
"""Verify Windows channel bodies; actual runtime checks require --native on Windows."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", action="store_true")
    args = parser.parse_args()
    if args.native and os.name != "nt":
        parser.error("Windows channel runtime verification requires an actual Windows host")
    library = ROOT / "target/portable-integration-stdlib"
    for relative in ["Shared_Memory_Channel", "internal/ipc"]:
        shutil.copytree(ROOT / "stdlib" / relative, library / relative, dirs_exist_ok=True)
    compiler = ROOT / ("target/compiler/debug/jaic.exe" if os.name == "nt" else "target/compiler/debug/jaic")
    env = dict(os.environ, JAIC_STDLIB=str(library))
    source = ROOT / "tests/portable-stdlib/windows-channel/main.jai"

    def checked(command, expected=None):
        result = subprocess.run([str(p) for p in command], cwd=ROOT, env=env,
                                text=True, capture_output=True, timeout=90)
        if result.returncode or result.stderr or (expected is not None and result.stdout != expected):
            raise RuntimeError(f"{command!r}\n{result.stdout}{result.stderr}")

    for cpu in ["x64", "arm64"]:
        checked([compiler, "-check", "-os", "windows", "-cpu", cpu, source])
        for role in ["reader", "writer"]:
            checked([compiler, "-check", "-os", "windows", "-cpu", cpu,
                     ROOT / f"tests/portable-stdlib/ipc/{role}.jai", "-add", 'CHANNEL_NAME :: "portable-source-check";'])
        print(f"Windows/{cpu} channel lifecycle/packet/IPC bodies check", flush=True)
    if not args.native:
        print("Windows mapping execution remains unverified; no native pass claimed")
        return
    checked([compiler, "-interp", source], "ok\n")
    for optimization in ["-O0", "-O2"]:
        output = ROOT / f"target/windows-channel{optimization}.exe"
        checked([compiler, source, optimization, "-exe", output])
        checked([output], "ok\n")
    subprocess.run([sys.executable, "-B", str(ROOT / "tools/check_portable_channels.py"),
                    "--jaic", str(compiler), "--stdlib", str(library), "--native"],
                   cwd=ROOT, env=env, check=True)
    print("Actual Windows interpreter/native O0/O2 and all four interprocess pairings passed")


if __name__ == "__main__":
    main()
