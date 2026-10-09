#!/usr/bin/env python3
"""Build the third-party C libraries the stdlib binds (stb_image, ...) from pinned sources.

Output: artifacts/native-libs/<os>-<arch>/, which `jaic` searches for `#system_library` /
`#library` names (see docs/tools/native-libs.md): lib<name>.a and lib<name>.<dylib|so> on macOS
and Linux, <name>.lib and <name>.dll on Windows (built with Clang for the MSVC toolchain).
`--platform windows-<cpu>-mingw` cross-builds lib<name>.a for `jaic build -os windows` (MinGW-w64
or llvm-mingw on the build host; the directory goes in JAIC_CROSS_LIBS).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / 'tools' / 'native-libs.json'

# The Clang target of each Windows platform directory.
WINDOWS_TRIPLES = {'windows-x64': 'x86_64-pc-windows-msvc', 'windows-arm64': 'aarch64-pc-windows-msvc'}
# The tool prefix of each MinGW cross platform (`windows-<cpu>-mingw`, built on Linux or macOS).
MINGW_PREFIXES = {'windows-x64-mingw': 'x86_64-w64-mingw32', 'windows-arm64-mingw': 'aarch64-w64-mingw32'}


def shared_root() -> Path:
    """The main checkout: git worktrees share its (ignored) artifacts/ instead of rebuilding."""
    common = subprocess.run(['git', 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                            cwd=ROOT, capture_output=True, text=True)
    if common.returncode == 0:
        return Path(common.stdout.strip()).parent
    return ROOT


def output_dir(plat: str | None = None) -> Path:
    return shared_root() / 'artifacts' / 'native-libs' / (plat or host_dir())


def static_name(name: str, plat: str) -> str:
    return f'{name}.lib' if plat.startswith('windows') and plat not in MINGW_PREFIXES else f'lib{name}.a'


def wanted(lib: dict, plat: str) -> bool:
    """Whether `lib` is built for `plat` (a manifest entry may name the OSes it is for)."""
    return plat.split('-')[0] in lib.get('platforms', [plat.split('-')[0]])


def missing(plat: str | None = None) -> list[str]:
    """Libraries in the manifest whose static archive has not been built for `plat`."""
    plat = plat or host_dir()
    libraries = json.loads(MANIFEST.read_text())['libraries']
    out = output_dir(plat)
    return sorted(name for name, lib in libraries.items()
                  if wanted(lib, plat) and not (out / static_name(name, plat)).exists())


def host_dir() -> str:
    system = {'Darwin': 'macos', 'Linux': 'linux', 'Windows': 'windows'}.get(platform.system())
    machine = platform.machine()
    arch = {'arm64': 'arm64', 'aarch64': 'arm64', 'ARM64': 'arm64', 'x86_64': 'x64', 'AMD64': 'x64'}.get(machine)
    if not system or not arch:
        sys.exit(f'unsupported host {platform.system()} {machine}')
    return f'{system}-{arch}'


def download(url: str, digest: str) -> bytes:
    data = urllib.request.urlopen(url, timeout=120).read()
    actual = hashlib.sha256(data).hexdigest()
    if actual != digest:
        sys.exit(f'{url}: sha256 {actual}, expected {digest}')
    return data


def fetch(source: dict, cache: Path) -> Path:
    """Download one pinned source into the cache, verifying its hashes: single files from a
    GitHub revision (`files`), or a release archive (`archive`) unpacked whole."""
    base = cache / source['revision']
    archive = source.get('archive')
    if archive:
        done = base / '.sha256'
        if done.exists() and done.read_text() == archive['sha256']:
            return base
        data = download(archive['url'], archive['sha256'])
        shutil.rmtree(base, ignore_errors=True)
        base.mkdir(parents=True)
        with tempfile.TemporaryDirectory() as tmp:
            packed = Path(tmp) / 'source.tar.gz'
            packed.write_bytes(data)
            with tarfile.open(packed) as tar:
                # Python before 3.12 (macOS's own) has no extraction filters.
                tar.extractall(tmp, **({'filter': 'data'} if hasattr(tarfile, 'data_filter') else {}))
            top = Path(tmp) / archive['root']
            for entry in top.iterdir():
                shutil.move(str(entry), base / entry.name)
        done.write_text(archive['sha256'])
        return base
    for rel, digest in source['files'].items():
        path = base / rel
        if path.exists() and hashlib.sha256(path.read_bytes()).hexdigest() == digest:
            continue
        url = f"https://raw.githubusercontent.com/{source['repository']}/{source['revision']}/{rel}"
        data = download(url, digest)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    return base


def compile_units(name: str, lib: dict, src: Path, tmp: Path, plat: str) -> list[Path]:
    """Object files of one library: its one-line `code` unit, or the `units` of its source."""
    if 'code' in lib:
        unit = tmp / f'jai_{name}_unit.c'
        unit.write_text(lib['code'])
        units = [unit]
    else:
        units = [src / rel for rel in lib['units']]
    if 'support' in lib:
        # Glue the library's own sources lack (see tools/native-libs.json), as one more unit.
        support = tmp / f'jai_{name}_support.{lib.get("support_suffix", "cpp")}'
        support.write_text(lib['support'])
        units.append(support)
    flags = ['-O2', '-w', '-I', str(src)]
    cxx_flags = ['-std=c++11', '-fno-exceptions', '-fno-rtti', '-fno-threadsafe-statics']
    flags += [f'-I{src / d}' for d in lib.get('include', [])]
    flags += [f'-D{d}' for d in lib.get('defines', [])]
    if plat in MINGW_PREFIXES:
        # Static archives for a MinGW-w64 link (`-l<name>`), built by the cross toolchain; the
        # executable runs on Windows with no DLL beside it.
        compiler = [f'{MINGW_PREFIXES[plat]}-gcc', '-D_CRT_SECURE_NO_WARNINGS']
        cxx = [f'{MINGW_PREFIXES[plat]}-g++', '-D_CRT_SECURE_NO_WARNINGS']
        suffix = 'o'
    elif plat.startswith('windows'):
        # Code for the static C runtime (`/MT`: plain `fopen`, no `__imp_fopen`) that names no
        # runtime library (`/Zl`). The archives then link into executables with either runtime:
        # Clang's driver links `libcmt` when it only links, Visual Studio's link.exe gets
        # `/DEFAULTLIB:msvcrt` from jaic, and MSVC's link.exe cannot resolve the `__imp_`
        # references of `/MD` code against `libcmt`.
        compiler = ['clang', f'--target={WINDOWS_TRIPLES[plat]}', '-fms-runtime-lib=static',
                    '-fms-omit-default-lib', '-D_CRT_SECURE_NO_WARNINGS']
        cxx = compiler  # Clang's driver compiles `.cpp` as C++
        suffix = 'obj'
    else:
        compiler = ['cc', '-fPIC']
        cxx = ['c++', '-fPIC']
        suffix = 'o'
    objects = []
    for unit in units:
        obj = tmp / f'{unit.stem}.{suffix}'
        # C++ units (meshoptimizer) need no exceptions or RTTI, so no C++ runtime library links in.
        command = [*cxx, *cxx_flags] if unit.suffix == '.cpp' else compiler
        subprocess.run([*command, *flags, '-c', str(unit), '-o', str(obj)], check=True)
        objects.append(obj)
    return objects


def windows_exports(objects: list[Path], tmp: Path, name: str) -> Path:
    """A module-definition file exporting every external symbol the objects define, so the DLL
    serves `GetProcAddress` the way a shared library's symbol table does elsewhere."""
    listing = subprocess.run(['llvm-nm', '--extern-only', '--defined-only', *map(str, objects)],
                             check=True, capture_output=True, text=True).stdout
    exports = set()
    for line in listing.splitlines():
        parts = line.split()
        # Compiler-generated names (`__real@...` constants, C++ decorations) are not API.
        if len(parts) == 3 and parts[1] in 'TDBR' and not re.search(r'^__|[@?$.]', parts[2]):
            exports.add(parts[2] + ('' if parts[1] == 'T' else ' DATA'))
    definitions = tmp / f'{name}.def'
    definitions.write_text(f'LIBRARY {name}\nEXPORTS\n' + ''.join(f'  {e}\n' for e in sorted(exports)))
    return definitions


def build(name: str, lib: dict, src: Path, out: Path, plat: str) -> None:
    with tempfile.TemporaryDirectory() as tmp_name:
        tmp = Path(tmp_name)
        objects = compile_units(name, lib, src, tmp, plat)
        static = out / static_name(name, plat)
        static.unlink(missing_ok=True)
        if plat in MINGW_PREFIXES:
            subprocess.run([f'{MINGW_PREFIXES[plat]}-ar', 'rcs', str(static), *map(str, objects)], check=True)
            return
        if plat.startswith('windows'):
            subprocess.run(['llvm-lib', '/nologo', f'/out:{static}', *map(str, objects)], check=True)
            definitions = windows_exports(objects, tmp, name)
            # The DLL's import library would take the static library's name: it goes to tmp.
            # The DLL carries its own copy of the static runtime (what `jaic run` loads).
            subprocess.run(['clang', f'--target={WINDOWS_TRIPLES[plat]}', '-fms-runtime-lib=static', '-shared',
                            '-o', str(out / f'{name}.dll'), *map(str, objects), f'-Wl,/DEF:{definitions}',
                            '-Wl,/DEFAULTLIB:libcmt', f'-Wl,/IMPLIB:{tmp / "import.lib"}'], check=True)
            return
        subprocess.run(['ar', 'rcs', str(static), *map(str, objects)], check=True)
        shared_ext = 'dylib' if plat.startswith('macos') else 'so'
        shared = out / f'lib{name}.{shared_ext}'
        args = ['cc', '-shared', '-o', str(shared), *map(str, objects), '-lm']
        if shared_ext == 'dylib':
            # Executables find it through their rpath, not the working directory. The header
            # padding lets Homebrew rewrite the install name when it installs a release archive.
            args += [f'-Wl,-install_name,@rpath/lib{name}.dylib', '-Wl,-headerpad_max_install_names']
        subprocess.run(args, check=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('names', nargs='*', help='libraries to build (default: all for the platform)')
    parser.add_argument('--platform', default=None,
                        help='output directory name, e.g. windows-arm64 (default: the host\'s)')
    parser.add_argument('--out', type=Path, default=None,
                        help='directory to write the libraries to (default: the main checkout\'s '
                             'artifacts/native-libs/<platform>); release.yml builds into the package')
    args = parser.parse_args()
    plat = args.platform or host_dir()
    if plat not in MINGW_PREFIXES and plat.split('-')[0] != host_dir().split('-')[0]:
        sys.exit(f'{plat}: libraries are built on the platform they are for '
                 f'(or cross-built for MinGW: {", ".join(MINGW_PREFIXES)})')
    if plat.startswith('windows') and plat not in WINDOWS_TRIPLES and plat not in MINGW_PREFIXES:
        sys.exit(f'{plat}: not one of {", ".join([*WINDOWS_TRIPLES, *MINGW_PREFIXES])}')
    manifest = json.loads(MANIFEST.read_text())
    names = args.names or sorted(n for n, lib in manifest['libraries'].items() if wanted(lib, plat))
    out = (args.out or output_dir(plat)).resolve()
    out.mkdir(parents=True, exist_ok=True)
    # Downloaded sources stay in the shared cache, never in --out (which may be a package).
    cache = output_dir(plat).parent / 'sources'
    for name in names:
        lib = manifest['libraries'].get(name)
        if lib is None:
            sys.exit(f'unknown library {name!r}')
        src = fetch(manifest['sources'][lib['source']], cache / lib['source'])
        build(name, lib, src, out, plat)
        print(f'built {name} -> {out}')


if __name__ == '__main__':
    main()
