#!/usr/bin/env python3
"""Run tests/webgpu/headless.jai natively: interpreted (`jaic run`, which loads the shared
wgpu-native) and built (`jaic build`, which links the static one), on this host's GPU or a
software adapter (Mesa's lavapipe on Linux, WARP on Windows).

    python3 tools/test_webgpu_native.py --jaic target/debug/jaic [--require-adapter] [--platform windows-arm64]

Fetches the pinned wgpu-native first (tools/fetch_wgpu_native.py). The test exits 77 when the
host has no adapter; each mode then tries again asking for the fallback (software) adapter.
Without --require-adapter, no adapter at all is a skip (macOS runners are VMs that may have no
Metal device); with it, a failure. See docs/stdlib/webgpu.md#tests.
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from build_native_libs import host_dir, output_dir  # noqa: E402
from fetch_wgpu_native import fetch  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
TEST = ROOT / 'tests' / 'webgpu' / 'headless.jai'
NO_ADAPTER = 77


def attempt(command: list[str], label: str) -> int:
    """Run the test, then once more on the fallback adapter if there was none."""
    for extra in ([], ['--', 'fallback'] if command[1:2] == ['run'] else ['fallback']):
        print(f'--- {label}{" (fallback adapter)" if extra else ""}', flush=True)
        status = subprocess.run(command + extra, cwd=ROOT).returncode
        if status != NO_ADAPTER:
            return status
    return NO_ADAPTER


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--jaic', required=True, type=Path)
    ap.add_argument('--platform', help='<os>-<arch> of the wgpu-native to fetch (default: this host)')
    ap.add_argument('--require-adapter', action='store_true', help='fail when no adapter exists')
    ap.add_argument('--out', type=Path, default=ROOT / 'target' / 'webgpu-headless', help='where the built test goes')
    args = ap.parse_args()
    plat = args.platform or host_dir()
    libs = output_dir(plat)
    fetch(plat, libs)
    os.environ['JAIC_NATIVE_LIBS'] = str(libs)
    jaic = str(args.jaic.resolve())
    exe = args.out.with_suffix('.exe') if plat.startswith('windows') else args.out
    args.out.parent.mkdir(parents=True, exist_ok=True)

    results = {'jaic run': attempt([jaic, 'run', str(TEST)], 'jaic run')}
    built = subprocess.run([jaic, 'build', str(TEST), '-o', str(args.out)], cwd=ROOT).returncode
    results['jaic build'] = attempt([str(exe)], 'built executable') if built == 0 else built

    failed = False
    for mode, status in results.items():
        if status == 0:
            print(f'{mode}: passed')
        elif status == NO_ADAPTER and not args.require_adapter:
            print(f'{mode}: skipped, no WebGPU adapter on this host')
        else:
            failed = True
            print(f'{mode}: FAILED (exit {status})')
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main())
