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
import tempfile
import time
import urllib.request
from sdl_sdk_headers import prepare_runtime_headers
from config import RPMALLOC_SDK_BUILD_OPTIONS
from prepare_rpmalloc import prepare as prepare_rpmalloc
from rpmalloc_profiles import PROFILES
from build_rpmalloc import build as build_rpmalloc

ROOT = Path(__file__).resolve().parents[2]

INPUTS = {
    'lz4': {'lz4'}, 'xxhash070': {'xxhash'}, 'meshoptimizer': {'meshoptimizer'},
    'binding_ffi': set(), 'graphics_macro_probe': set(),
    'sdl_abi_probe': {'sdl2-oracle'}, 'imgui': {'imgui'},
    'sdl_runtime_abi_probe': {'sdl2-runtime-oracle'},
    'stb_image': {'stb'}, 'stb_image_resize': {'stb'},
    'stb_image_write': {'stb'}, 'stb_vorbis': {'stb'}, 'mojoshader': {'mojoshader'},
    'binding_rpmalloc_sdk': {'rpmalloc'}, 'rpmalloc_sdk_probe': {'rpmalloc'},
    'binding_rpmalloc_sdk_nostats': {'rpmalloc'},
    'binding_callback_context': set(),
    'binding_rpmalloc_callbacks': {'rpmalloc'},
    'binding_rpmalloc_callbacks_nostats': {'rpmalloc'},
    'rpmalloc_callback_probe': {'rpmalloc'}, 'rpmalloc_storage_probe': {'rpmalloc'},
    'fsevents_context_probe': set(), 'objc_runtime_probe': set(),
    'posix_abi_probe': set(), 'imgui_abi_probe': {'imgui'},
}

LIBRARY_DEPENDENCIES = {
    'binding_rpmalloc_callbacks': {'binding_rpmalloc_sdk'},
    'binding_rpmalloc_callbacks_nostats': {'binding_rpmalloc_sdk_nostats'},
    'rpmalloc_callback_probe': {'binding_rpmalloc_sdk'},
}
for profile in PROFILES:
    INPUTS[profile['sdk']] = {'rpmalloc'}
    INPUTS[profile['callbacks']] = {'rpmalloc'}
    LIBRARY_DEPENDENCIES[profile['callbacks']] = {profile['sdk']}


def library_closure(selected):
    selected = set(selected)
    pending = list(selected)
    while pending:
        for dependency in LIBRARY_DEPENDENCIES.get(pending.pop(), ()):
            if dependency not in selected:
                selected.add(dependency)
                pending.append(dependency)
    return selected


def run(command):
    subprocess.run([str(x) for x in command], check=True)


def download(url, attempts=4):
    """Fetch a pinned archive; hosted runners drop connections now and then, so retry with backoff."""
    for attempt in range(1, attempts + 1):
        try:
            return urllib.request.urlopen(url, timeout=60).read()
        except OSError:
            if attempt == attempts:
                raise
            time.sleep(5 * attempt)


def prepare_source(name, source, vendor):
    """Validate a pinned release before using its files; otherwise use pinned raw inputs."""
    archive_root = None
    if 'archive' in source:
        pin = source['archive']
        archive = vendor / (name + '.tar.gz')
        if not archive.exists():
            archive.parent.mkdir(parents=True, exist_ok=True)
            archive.write_bytes(download(pin['url']))
        if hashlib.sha256(archive.read_bytes()).hexdigest() != pin['sha256']:
            raise RuntimeError('Archive hash mismatch: ' + name)
        archive_root = vendor / pin['root']
        # Recreate the complete tree from the verified archive. Checking only
        # listed primary headers leaves cached transitive headers/source mutable.
        with tempfile.TemporaryDirectory(prefix=name+'-extract-',dir=vendor) as fresh:
            with tarfile.open(archive) as stream:
                stream.extractall(fresh, filter='data')
            extracted = Path(fresh)/pin['root']
            if not extracted.is_dir() or extracted.is_symlink():
                raise RuntimeError('Archive source root is missing or invalid: '+name)
            if archive_root.is_symlink():
                raise RuntimeError('Archive source root cannot be a symlink: '+name)
            if archive_root.exists():
                shutil.rmtree(archive_root)
            extracted.rename(archive_root)
    for filename, expected in source['files'].items():
        path = vendor / name / filename
        if not path.exists():
            if archive_root is not None:
                data = (archive_root / filename).read_bytes()
            else:
                url = 'https://raw.githubusercontent.com/' + source['repository'] + '/' + source['revision'] + '/' + filename
                data = urllib.request.urlopen(url, timeout=60).read()
            if hashlib.sha256(data).hexdigest() != expected:
                raise RuntimeError('Native source hash mismatch: ' + name + '/' + filename)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise RuntimeError('Native source hash mismatch: ' + str(path))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / '.work/native')
    parser.add_argument('--library', action='append', choices=sorted(INPUTS),
                        help='Build only selected real native libraries and their pinned inputs')
    args = parser.parse_args()
    vendor = ROOT / '.work/vendor'
    manifest = json.loads((Path(__file__).with_name('dependencies.json')).read_text())
    selected = library_closure(args.library or INPUTS)
    # Preserve full-manifest validation by default. A scoped build records only
    # its actual dependency closure; it is not evidence for the other SDK pins.
    dependencies = set().union(*(INPUTS[name] for name in selected)) if args.library else set(manifest)
    for name in sorted(dependencies):
        prepare_source(name, manifest[name], vendor)
    args.output.mkdir(parents=True, exist_ok=True)
    rpmalloc_source = (prepare_rpmalloc(vendor/'rpmalloc', args.output/'rpmalloc-source')
                       if 'rpmalloc' in dependencies else None)
    if rpmalloc_source is not None:
        build_rpmalloc(args.output, selected=selected, source=rpmalloc_source)
    if platform.system() == 'Darwin':
        apple_object = args.output/'apple_abi_probe.o'
        run(['cc','-Wall','-Wextra','-Werror','-c',ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/apple_abi_probe.m',
             '-o',apple_object])
        run(['ar','rcs',args.output/'libapple_abi_probe.a',apple_object])
        run(['cc','-dynamiclib',ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/apple_abi_probe.m',
             '-framework','Foundation','-framework','Metal',
             '-o',args.output/'libapple_abi_probe.dylib'])
    runtime_sdl_headers = (prepare_runtime_headers(
        vendor/manifest['sdl2-runtime-oracle']['archive']['root'],
        args.output/'sdl2-runtime-headers')
        if 'sdl_runtime_abi_probe' in selected else [])
    objects = args.output / 'objects'
    objects.mkdir(exist_ok=True)
    units = {
        'lz4': [vendor/'lz4/lib/lz4.c', vendor/'lz4/lib/lz4hc.c'],
        'xxhash070': [vendor/'xxhash/xxhash.c'],
        'meshoptimizer': sorted((vendor/'meshoptimizer/src').glob('*.cpp')),
        'binding_ffi': [ROOT/'stdlib/internal/bindings/varargs/bridge.c'],
        'binding_callback_context': [ROOT/'stdlib/internal/bindings/callback-context/bridge.c'],
        'graphics_macro_probe': [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/generated/d3d12_macro_probe.c'],
        'sdl_abi_probe': [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/sdl_abi_probe.c'],
        'sdl_runtime_abi_probe': [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/sdl_runtime_abi_probe.c'],
        'posix_abi_probe': [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/posix_abi_probe.c'],
        'rpmalloc_sdk_probe': [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/generated/rpmalloc_sdk_probe.c'],
        'rpmalloc_callback_probe': [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_callback_probe.c',
                                   ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_sdk_config_probe.c'],
        'rpmalloc_storage_probe': [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/generated/rpmalloc_storage_probe.c'],
        'imgui_abi_probe': [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/imgui_abi_probe.cpp'],
        'imgui': [vendor/manifest['imgui']['archive']['root']/name for name in
                  ('imgui.cpp','imgui_draw.cpp','imgui_tables.cpp','imgui_widgets.cpp','imgui_demo.cpp')],
    }
    if platform.system() == 'Darwin':
        units['fsevents_context_probe'] = [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/fsevents_context_probe.c']
        units['objc_runtime_probe'] = [ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/objc_runtime_probe.m']
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
                includes = [flag for root in runtime_sdl_headers for flag in ('-isystem', root)]
            if name == 'imgui_abi_probe':
                includes = ['-isystem',vendor/manifest['imgui']['archive']['root']]
            if name.startswith('binding_rpmalloc_') or name in ('rpmalloc_sdk_probe', 'rpmalloc_callback_probe', 'rpmalloc_storage_probe'):
                includes = ['-isystem', rpmalloc_source]
                includes += ['-D'+option+'='+str(value) for option,value in RPMALLOC_SDK_BUILD_OPTIONS.items()]
            run(['c++' if path.suffix == '.cpp' else 'cc', '-O2', '-fPIC', '-I', vendor/'stb', *includes, '-c', path, '-o', obj])
            compiled.append(obj)
        run(['ar', 'rcs', args.output / ('lib' + name + '.a'), *compiled])
        suffix = '.dylib' if platform.system() == 'Darwin' else '.so'
        extra = ['-lffi'] if name == 'binding_ffi' else []
        if name == 'fsevents_context_probe':
            extra = ['-framework', 'CoreServices']
        if name == 'objc_runtime_probe':
            extra = ['-framework', 'Foundation']
        if name == 'rpmalloc_callback_probe':
            # Shared adapters must use the same SDK instance as allocations.
            # Statically embedding a second allocator would split ownership.
            # The SDK sits next to the probe: the interpreter dlopens the probe on its own, so the
            # loader needs the probe's directory in its search path to find the SDK.
            extra = ['-L', args.output, '-lbinding_rpmalloc_sdk'+('_nostats' if name.endswith('_nostats') else ''),
                     '-Wl,-rpath,@loader_path' if suffix == '.dylib' else '-Wl,-rpath,$ORIGIN']
        run(['c++' if name in ('meshoptimizer','imgui') else 'cc', '-dynamiclib' if suffix == '.dylib' else '-shared', *compiled,
             '-lm', *extra, '-o', args.output / ('lib' + name + suffix)])
        print('Built ' + name)
    if 'mojoshader' not in selected:
        return
    mojo = vendor/manifest['mojoshader']['archive']['root']
    # MojoShader's CMake asks the version control tool for the changeset, and the pinned archive is
    # unpacked inside this repository, so it would report this checkout's history (or "???" for a
    # shallow clone). Name the pinned revision instead.
    changeset = 'git-' + manifest['mojoshader']['revision'][:8]
    identity = f'-DMOJOSHADER_NO_VERSION_INCLUDE -DMOJOSHADER_CHANGESET=\\"{changeset}\\"'
    for shared in (False, True):
        build = args.output/('mojoshader-shared' if shared else 'mojoshader-static')
        run(['cmake','-S',mojo,'-B',build,'-DCMAKE_BUILD_TYPE=Release',
             '-DCMAKE_POLICY_VERSION_MINIMUM=3.5','-DCMAKE_C_FLAGS='+identity,'-DCOMPILER_SUPPORT=ON','-DEFFECT_SUPPORT=ON',
             '-DBUILD_SHARED_LIBS='+('ON' if shared else 'OFF')])
        run(['cmake','--build',build,'--target','mojoshader','--parallel','4'])
        suffix = ('.dylib' if platform.system()=='Darwin' else '.so') if shared else '.a'
        shutil.copy2(build/('libmojoshader'+suffix),args.output/('libmojoshader'+suffix))


if __name__ == '__main__':
    main()
