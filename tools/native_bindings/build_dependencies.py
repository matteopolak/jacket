#!/usr/bin/env python3
"""Fetch hash-pinned official native C sources and build real host libraries.

All downloads and outputs stay inside this clean-room workspace. No Git access,
credentials, reference binaries, or reference Jai sources are involved.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import shutil
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[2]

INPUTS = {
    'lz4': {'lz4'}, 'xxhash070': {'xxhash'}, 'meshoptimizer': {'meshoptimizer'},
    'binding_ffi': set(), 'graphics_macro_probe': set(),
    'sdl_abi_probe': {'sdl2-oracle'}, 'imgui': {'imgui'},
    'sdl_runtime_abi_probe': {'sdl2-runtime-oracle'},
    'stb_image': {'stb'}, 'stb_image_resize': {'stb'},
    'stb_image_write': {'stb'}, 'stb_vorbis': {'stb'}, 'mojoshader': {'mojoshader'},
}


def run(command):
    subprocess.run([str(x) for x in command], check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / '.work/native')
    parser.add_argument('--library', action='append', choices=sorted(INPUTS),
                        help='Build only selected real native libraries and their pinned inputs')
    args = parser.parse_args()
    vendor = ROOT / '.work/vendor'
    manifest = json.loads((Path(__file__).with_name('dependencies.json')).read_text())
    selected = set(args.library or INPUTS)
    # Preserve full-manifest validation by default. A scoped build records only
    # its actual dependency closure; it is not evidence for the other SDK pins.
    dependencies = set().union(*(INPUTS[name] for name in selected)) if args.library else set(manifest)
    for name in sorted(dependencies):
        source = manifest[name]
        for filename, expected in source['files'].items():
            path = vendor / name / filename
            if not path.exists():
                url = 'https://raw.githubusercontent.com/' + source['repository'] + '/' + source['revision'] + '/' + filename
                data = urllib.request.urlopen(url, timeout=60).read()
                if hashlib.sha256(data).hexdigest() != expected:
                    raise SystemExit('Hash mismatch: ' + url)
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
            if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
                raise SystemExit('Hash mismatch: ' + str(path))
        if 'archive' in source:
            pin = source['archive']
            archive = vendor/(name+'.tar.gz')
            if not archive.exists():
                archive.write_bytes(urllib.request.urlopen(pin['url'], timeout=60).read())
            if hashlib.sha256(archive.read_bytes()).hexdigest() != pin['sha256']:
                raise SystemExit('Archive hash mismatch: '+name)
            if not (vendor/pin['root']).is_dir():
                with tarfile.open(archive) as stream:
                    stream.extractall(vendor, filter='data')
    args.output.mkdir(parents=True, exist_ok=True)
    if platform.system() == 'Darwin':
        apple_object = args.output/'apple_abi_probe.o'
        run(['cc','-Wall','-Wextra','-Werror','-c',ROOT/'tests/native-bindings/apple_abi_probe.m',
             '-o',apple_object])
        run(['ar','rcs',args.output/'libapple_abi_probe.a',apple_object])
        run(['cc','-dynamiclib',ROOT/'tests/native-bindings/apple_abi_probe.m',
             '-framework','Foundation','-framework','Metal',
             '-o',args.output/'libapple_abi_probe.dylib'])
    objects = args.output / 'objects'
    objects.mkdir(exist_ok=True)
    units = {
        'lz4': [vendor/'lz4/lib/lz4.c', vendor/'lz4/lib/lz4hc.c'],
        'xxhash070': [vendor/'xxhash/xxhash.c'],
        'meshoptimizer': sorted((vendor/'meshoptimizer/src').glob('*.cpp')),
        'binding_ffi': [ROOT/'stdlib/internal/bindings/varargs/bridge.c'],
        'graphics_macro_probe': [ROOT/'tests/native-bindings/generated/d3d12_macro_probe.c'],
        'sdl_abi_probe': [ROOT/'tests/native-bindings/sdl_abi_probe.c'],
        'sdl_runtime_abi_probe': [ROOT/'tests/native-bindings/sdl_runtime_abi_probe.c'],
        'posix_abi_probe': [ROOT/'tests/native-bindings/posix_abi_probe.c'],
        'imgui': [vendor/manifest['imgui']['archive']['root']/name for name in
                  ('imgui.cpp','imgui_draw.cpp','imgui_tables.cpp','imgui_widgets.cpp','imgui_demo.cpp')],
    }
    macros = {'stb_image': 'STB_IMAGE_IMPLEMENTATION',
              'stb_image_resize': 'STB_IMAGE_RESIZE_IMPLEMENTATION',
              'stb_image_write': 'STB_IMAGE_WRITE_IMPLEMENTATION'}
    for name, macro in macros.items():
        header = ('deprecated/' if name == 'stb_image_resize' else '') + name + '.h'
        path = objects / (name + '.c')
        path.write_text('#define ' + macro + '\n#include "' + header + '"\n')
        units[name] = [path]
    units['stb_vorbis'] = [vendor/'stb/stb_vorbis.c']
    for name, sources in units.items():
        if name not in selected:
            continue
        compiled = []
        for i, path in enumerate(sources):
            obj = objects / (name + str(i) + '.o')
            includes = ['-isystem',vendor/manifest['sdl2-oracle']['archive']['root']/'include'] if name=='sdl_abi_probe' else []
            if name == 'sdl_runtime_abi_probe':
                includes = ['-isystem',vendor/manifest['sdl2-runtime-oracle']['archive']['root']/'include/SDL2']
            run(['c++' if path.suffix == '.cpp' else 'cc', '-O2', '-fPIC', '-I', vendor/'stb', *includes, '-c', path, '-o', obj])
            compiled.append(obj)
        run(['ar', 'rcs', args.output / ('lib' + name + '.a'), *compiled])
        suffix = '.dylib' if platform.system() == 'Darwin' else '.so'
        extra = ['-lffi'] if name == 'binding_ffi' else []
        run(['c++' if name in ('meshoptimizer','imgui') else 'cc', '-dynamiclib' if suffix == '.dylib' else '-shared', *compiled,
             '-lm', *extra, '-o', args.output / ('lib' + name + suffix)])
        print('Built ' + name)
    if 'mojoshader' not in selected:
        return
    mojo = vendor/manifest['mojoshader']['archive']['root']
    for shared in (False, True):
        build = args.output/('mojoshader-shared' if shared else 'mojoshader-static')
        run(['cmake','-S',mojo,'-B',build,'-DCMAKE_BUILD_TYPE=Release',
             '-DCMAKE_POLICY_VERSION_MINIMUM=3.5','-DCOMPILER_SUPPORT=ON','-DEFFECT_SUPPORT=ON',
             '-DBUILD_SHARED_LIBS='+('ON' if shared else 'OFF')])
        run(['cmake','--build',build,'--target','mojoshader','--parallel','4'])
        suffix = ('.dylib' if platform.system()=='Darwin' else '.so') if shared else '.a'
        shutil.copy2(build/('libmojoshader'+suffix),args.output/('libmojoshader'+suffix))


if __name__ == '__main__':
    main()
