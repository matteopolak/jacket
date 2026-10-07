#!/usr/bin/env python3
"""Download the pinned wgpu-native release for this host into artifacts/native-libs/<os>-<arch>/.

    python3 tools/fetch_wgpu_native.py [--force]

tools/webgpu.json pins the release tag and the sha256 of each platform's zip. The zip's
`lib/libwgpu_native.{a,dylib,so}` (Windows: `wgpu_native.dll` and its import library) land next to
the libraries tools/build_native_libs.py builds, where `jaic` finds them for `#library
"libwgpu_native"` (stdlib/WebGPU). The release also carries the webgpu.yml it was built from;
it must match the revision tools/webgpu_gen.py generated the bindings from.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import platform
import sys
import urllib.request
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from build_native_libs import output_dir  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / 'tools' / 'webgpu.json'


def host_key() -> str:
    system = {'Darwin': 'macos', 'Linux': 'linux', 'Windows': 'windows'}.get(platform.system())
    arch = {'arm64': 'arm64', 'aarch64': 'arm64', 'x86_64': 'x64', 'AMD64': 'x64'}.get(platform.machine())
    if not system or not arch:
        sys.exit(f'unsupported host {platform.system()} {platform.machine()}')
    return f'{system}-{arch}'


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--force', action='store_true', help='download even when the library is present')
    args = ap.parse_args()
    pins = json.loads(MANIFEST.read_text())
    native, headers = pins['wgpu_native'], pins['webgpu_headers']
    asset = native['assets'].get(host_key())
    if not asset:
        sys.exit(f'no wgpu-native asset pinned for {host_key()}')
    out = output_dir()
    stamp = out / 'wgpu_native.version'
    if not args.force and stamp.exists() and stamp.read_text().strip() == native['tag']:
        print(f'wgpu-native {native["tag"]} already in {out}')
        return 0
    url = f'https://github.com/{native["repository"]}/releases/download/{native["tag"]}/{asset["name"]}'
    print(f'downloading {url}')
    data = urllib.request.urlopen(url, timeout=300).read()
    digest = hashlib.sha256(data).hexdigest()
    if digest != asset['sha256']:
        sys.exit(f'{asset["name"]}: sha256 {digest}, expected {asset["sha256"]}')
    with zipfile.ZipFile(io.BytesIO(data)) as zf:
        spec = zf.read('wgpu-native-meta/webgpu.yml')
        if hashlib.sha256(spec).hexdigest() != headers['files']['webgpu.yml']:
            sys.exit('the release was built from another webgpu.yml than tools/webgpu.json pins; '
                     'update the pin and rerun tools/webgpu_gen.py')
        out.mkdir(parents=True, exist_ok=True)
        wanted = ('libwgpu_native.a', 'libwgpu_native.dylib', 'libwgpu_native.so',
                  'wgpu_native.dll', 'wgpu_native.dll.lib', 'wgpu_native.lib')
        for name in zf.namelist():
            base = name.rsplit('/', 1)[-1]
            if name.startswith('lib/') and base in wanted:
                (out / base).write_bytes(zf.read(name))
                print(f'  {out / base}')
    stamp.write_text(native['tag'] + '\n')
    return 0


if __name__ == '__main__':
    sys.exit(main())
