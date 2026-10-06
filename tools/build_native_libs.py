#!/usr/bin/env python3
"""Build the third-party C libraries the stdlib binds (stb_image, ...) from pinned sources.

Output: artifacts/native-libs/<os>-<arch>/lib<name>.a and lib<name>.<dylib|so>, which `jaic`
searches for `#system_library` / `#library` names (see docs/tools/native-libs.md).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / 'tools' / 'native-libs.json'


def shared_root() -> Path:
    """The main checkout: git worktrees share its (ignored) artifacts/ instead of rebuilding."""
    common = subprocess.run(['git', 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                            cwd=ROOT, capture_output=True, text=True)
    if common.returncode == 0:
        return Path(common.stdout.strip()).parent
    return ROOT


def output_dir() -> Path:
    return shared_root() / 'artifacts' / 'native-libs' / host_dir()


def missing() -> list[str]:
    """Libraries in the manifest whose static archive has not been built for this host."""
    names = json.loads(MANIFEST.read_text())['libraries']
    return sorted(name for name in names if not (output_dir() / f'lib{name}.a').exists())


def host_dir() -> str:
    system = {'Darwin': 'macos', 'Linux': 'linux'}.get(platform.system())
    arch = {'arm64': 'arm64', 'aarch64': 'arm64', 'x86_64': 'x64', 'AMD64': 'x64'}.get(platform.machine())
    if not system or not arch:
        sys.exit(f'unsupported host {platform.system()} {platform.machine()}')
    return f'{system}-{arch}'


def fetch(source: dict, cache: Path) -> Path:
    """Download the pinned files of one source into the cache, verifying their hashes."""
    base = cache / source['revision']
    for rel, digest in source['files'].items():
        path = base / rel
        if path.exists() and hashlib.sha256(path.read_bytes()).hexdigest() == digest:
            continue
        url = f"https://raw.githubusercontent.com/{source['repository']}/{source['revision']}/{rel}"
        data = urllib.request.urlopen(url, timeout=60).read()
        actual = hashlib.sha256(data).hexdigest()
        if actual != digest:
            sys.exit(f'{url}: sha256 {actual}, expected {digest}')
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    return base


def build(name: str, lib: dict, src: Path, out: Path) -> None:
    shared_ext = 'dylib' if platform.system() == 'Darwin' else 'so'
    with tempfile.TemporaryDirectory() as tmp:
        unit = Path(tmp) / f'jai_{name}_unit.c'
        unit.write_text(lib['code'])
        obj = Path(tmp) / f'{name}.o'
        subprocess.run(['cc', '-O2', '-fPIC', '-w', '-I', str(src), '-c', str(unit), '-o', str(obj)], check=True)
        static = out / f'lib{name}.a'
        static.unlink(missing_ok=True)
        subprocess.run(['ar', 'rcs', str(static), str(obj)], check=True)
        shared = out / f'lib{name}.{shared_ext}'
        args = ['cc', '-shared', '-o', str(shared), str(obj), '-lm']
        if shared_ext == 'dylib':
            # Executables find it through their rpath, not the working directory.
            args.append(f'-Wl,-install_name,@rpath/lib{name}.dylib')
        subprocess.run(args, check=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('names', nargs='*', help='libraries to build (default: all)')
    args = parser.parse_args()
    manifest = json.loads(MANIFEST.read_text())
    names = args.names or sorted(manifest['libraries'])
    out = output_dir()
    out.mkdir(parents=True, exist_ok=True)
    cache = out.parent / 'sources'
    for name in names:
        lib = manifest['libraries'].get(name)
        if lib is None:
            sys.exit(f'unknown library {name!r}')
        src = fetch(manifest['sources'][lib['source']], cache / lib['source'])
        build(name, lib, src, out)
        print(f'built {name} -> {out}')


if __name__ == '__main__':
    main()
