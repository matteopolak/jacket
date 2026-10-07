#!/usr/bin/env python3
"""Download the pinned wgpu-native release into artifacts/native-libs/<os>-<arch>/.

    python3 tools/fetch_wgpu_native.py [--platform windows-x64] [--out DIR] [--force]

tools/webgpu.json pins the release tag and the sha256 of each platform's zip. The zip's
static library (`libwgpu_native.a`, Windows `wgpu_native.lib`), which `jaic build` links, and
its shared library (`libwgpu_native.dylib`/`.so`, Windows `wgpu_native.dll`), which `jaic run`
loads, land next to the libraries tools/build_native_libs.py builds, where `jaic` finds them
for `#library "libwgpu_native"` (stdlib/WebGPU). The release also carries the webgpu.yml it was
built from; it must match the revision tools/webgpu_gen.py generated the bindings from.

`--platform` and `--out` fetch another platform's libraries into any directory (the release
workflow puts them in each archive's artifacts/native-libs/<platform>/).
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import sys
import time
import urllib.request
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from build_native_libs import host_dir, output_dir  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / 'tools' / 'webgpu.json'

# What each platform needs from the zip's lib/ folder: the static library first.
LIBRARIES = {
    'macos': ('libwgpu_native.a', 'libwgpu_native.dylib'),
    'linux': ('libwgpu_native.a', 'libwgpu_native.so'),
    'windows': ('wgpu_native.lib', 'wgpu_native.dll'),
}


def download(url: str) -> bytes:
    for attempt in range(3):
        try:
            return urllib.request.urlopen(url, timeout=300).read()
        except OSError as error:
            if attempt == 2:
                raise
            print(f'  {error}; retrying')
            time.sleep(5 * (attempt + 1))
    raise AssertionError


def fetch(plat: str, out: Path, force: bool = False) -> list[Path]:
    """Put `plat`'s wgpu-native libraries in `out`; returns their paths."""
    pins = json.loads(MANIFEST.read_text())
    native, headers = pins['wgpu_native'], pins['webgpu_headers']
    asset = native['assets'].get(plat)
    if not asset:
        sys.exit(f'no wgpu-native asset pinned for {plat}')
    names = LIBRARIES[plat.split('-')[0]]
    stamp = out / 'wgpu_native.version'
    paths = [out / name for name in names]
    if (not force and stamp.exists() and stamp.read_text().strip() == native['tag']
            and all(p.exists() for p in paths)):
        print(f'wgpu-native {native["tag"]} already in {out}')
        return paths
    url = f'https://github.com/{native["repository"]}/releases/download/{native["tag"]}/{asset["name"]}'
    print(f'downloading {url}')
    data = download(url)
    digest = hashlib.sha256(data).hexdigest()
    if digest != asset['sha256']:
        sys.exit(f'{asset["name"]}: sha256 {digest}, expected {asset["sha256"]}')
    with zipfile.ZipFile(io.BytesIO(data)) as zf:
        # The Windows zips have it with CRLF line endings.
        spec = zf.read('wgpu-native-meta/webgpu.yml').replace(b'\r\n', b'\n')
        if hashlib.sha256(spec).hexdigest() != headers['files']['webgpu.yml']:
            sys.exit('the release was built from another webgpu.yml than tools/webgpu.json pins; '
                     'update the pin and rerun tools/webgpu_gen.py')
        out.mkdir(parents=True, exist_ok=True)
        members = {name.rsplit('/', 1)[-1]: name for name in zf.namelist() if name.startswith('lib/')}
        for name in names:
            if name not in members:
                sys.exit(f'{asset["name"]} has no lib/{name}')
            (out / name).write_bytes(zf.read(members[name]))
            print(f'  {out / name}')
    stamp.write_text(native['tag'] + '\n')
    return paths


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--platform', help='<os>-<arch> (default: this host)')
    ap.add_argument('--out', type=Path, help='output directory (default: artifacts/native-libs/<platform>)')
    ap.add_argument('--force', action='store_true', help='download even when the libraries are present')
    args = ap.parse_args()
    plat = args.platform or host_dir()
    fetch(plat, args.out or output_dir(plat), args.force)
    return 0


if __name__ == '__main__':
    sys.exit(main())
