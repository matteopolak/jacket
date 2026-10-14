#!/usr/bin/env python3
"""Pinned SDL3 native build; also called by build_native_libs.py. Never consults another checkout."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import tarfile
import urllib.request
from pathlib import Path

ROOT=Path(__file__).resolve().parent.parent
PLATFORMS=('macos-x64','macos-arm64','linux-x64','linux-arm64','windows-x64','windows-arm64')

def checked_archive(cache, offline=False):
    source=json.loads((ROOT/'tools/native-libs.json').read_text())['sources']['sdl3']
    spec=source['archive']; cache.mkdir(parents=True,exist_ok=True)
    packed=cache/'SDL3-3.4.18.tar.gz'
    if not packed.exists():
        if offline: raise RuntimeError(f'offline: missing {packed}')
        data=urllib.request.urlopen(spec['url'],timeout=120).read()
        if hashlib.sha256(data).hexdigest()!=spec['sha256']: raise RuntimeError('download hash mismatch')
        packed.write_bytes(data)
    if hashlib.sha256(packed.read_bytes()).hexdigest()!=spec['sha256']: raise RuntimeError(f'cache hash mismatch: {packed}')
    # Validate every extracted file against the verified archive; preserve mtimes when exact.
    src=cache/spec['root']
    if src.is_symlink(): raise RuntimeError(f'source cache is a symlink: {src}')
    with tarfile.open(packed) as tar:
        members=tar.getmembers()
        for m in members:
            path=Path(m.name)
            if path.is_absolute() or '..' in path.parts or not (m.isdir() or m.isfile()):
                raise RuntimeError(f'unsafe source archive member: {m.name}')
        expected_files={Path(m.name) for m in members if m.isfile()}
        actual_files={p.relative_to(cache) for p in src.rglob('*') if p.is_file()} if src.exists() else set()
        valid=not any(p.is_symlink() for p in src.rglob('*')) and expected_files==actual_files and all(
            not (cache/m.name).is_symlink() and (cache/m.name).read_bytes()==tar.extractfile(m).read()
            for m in members if m.isfile())
        if not valid:
            if src.exists(): shutil.rmtree(src)
            tar.extractall(cache)
    version=(src/'include/SDL3/SDL_version.h').read_text()
    for key,n in [('MAJOR',3),('MINOR',4),('MICRO',18)]:
        if not re.search(rf'#define SDL_{key}_VERSION\s+{n}\b', version): raise RuntimeError('source version mismatch')
    return src,source

def host():
    system={'Darwin':'macos','Linux':'linux','Windows':'windows'}[platform.system()]
    cpu={'arm64':'arm64','aarch64':'arm64','ARM64':'arm64','x86_64':'x64','AMD64':'x64'}[os.environ.get('PROCESSOR_ARCHITEW6432',platform.machine())]
    return f'{system}-{cpu}'

def configure_options(plat):
    if plat not in PLATFORMS: raise ValueError(f'unsupported SDL3 platform: {plat}')
    system,cpu=plat.split('-')
    options=['-DSDL_STATIC=ON','-DSDL_SHARED=ON','-DSDL_TESTS=OFF','-DSDL_TEST_LIBRARY=OFF',
             '-DSDL_EXAMPLES=OFF','-DSDL_INSTALL_DOCS=OFF','-DSDL_INSTALL=ON',
             '-DSDL_AUDIO=ON','-DSDL_VIDEO=ON','-DSDL_EVENTS=ON','-DSDL_JOYSTICK=ON',
             '-DSDL_GAMEPAD=ON','-DSDL_RENDER=ON','-DSDL_GPU=ON','-DSDL_HAPTIC=ON',
             '-DSDL_HIDAPI=ON','-DCMAKE_BUILD_TYPE=Release','-DCMAKE_POSITION_INDEPENDENT_CODE=ON']
    if system=='macos':
        options += [f'-DCMAKE_OSX_ARCHITECTURES={"x86_64" if cpu=="x64" else "arm64"}',
                    f'-DCMAKE_OSX_DEPLOYMENT_TARGET={"10.12" if cpu=="x64" else "11.0"}',
                    '-DSDL_FRAMEWORK=OFF','-DCMAKE_INSTALL_NAME_DIR=@rpath',
                    '-DCMAKE_SHARED_LINKER_FLAGS=-Wl,-headerpad_max_install_names',
                    # Avoid dependence on Homebrew libiconv; SDL contains its own conversion fallback.
                    '-DSDL_LIBICONV=OFF']
    if system=='linux':
        options += ['-DSDL_UNIX_CONSOLE_BUILD=OFF','-DSDL_X11=ON','-DSDL_WAYLAND=ON',
                    '-DSDL_ALSA=ON','-DSDL_PULSEAUDIO=ON','-DSDL_PIPEWIRE=ON',
                    '-DSDL_X11_SHARED=ON','-DSDL_WAYLAND_SHARED=ON','-DSDL_ALSA_SHARED=ON',
                    '-DSDL_PULSEAUDIO_SHARED=ON','-DSDL_PIPEWIRE_SHARED=ON',
                    '-DSDL_LIBUSB_SHARED=ON','-DSDL_LIBUDEV_SHARED=ON','-DSDL_KMSDRM_SHARED=ON',
                    '-DSDL_LIBICONV=OFF']
    if system=='windows':
        options += ['-G','Visual Studio 17 2022','-A','x64' if cpu=='x64' else 'ARM64',
                    '-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded','-DSDL_LIBC=ON']
    return options

def check_dependencies(text,plat):
    """Reject optional packages that became hard static dependencies unexpectedly.
    The module's system dependencies must cover SDL's actual configured link interface.
    """
    module=(ROOT/f'stdlib/Extensions/SDL3/link-{plat.split("-")[0]}.jai').read_text()
    allowed=set(re.findall(r'#library,system,link_always "([^"]+)"',module))
    if plat.startswith('macos'):allowed |= {'CoreHaptics','UniformTypeIdentifiers','m','pthread'}
    for item in text.strip().replace('\n',';').split(';'):
        if not item or item=='$<TARGET_NAME:SDL3::Headers>':continue
        if item.startswith('$<LINK_ONLY:'):item=item[len('$<LINK_ONLY:'):-1]
        framework=re.fullmatch(r'\$<LINK_LIBRARY:(?:WEAK_)?FRAMEWORK,([^>]+)>',item)
        if framework:item=framework[1]
        if item=='-pthread':item='pthread'
        if item.startswith('-l'):item=item[2:]
        if item not in allowed:raise RuntimeError(f'{plat}: unbound transitive static link dependency: {item}')

def build(out,plat,cache=None,offline=False,jobs=2):
    if plat not in PLATFORMS: raise ValueError(plat)
    if plat.split('-')[0]!=host().split('-')[0]: raise ValueError('build SDL3 on the target OS with its native SDK')
    if plat.startswith('linux') and plat!=host(): raise ValueError('Linux SDL3 release builds require a native target runner (no host-architecture fallback)')
    cache=(cache or ROOT/'artifacts/sdl3').resolve()
    src,source=checked_archive(cache,offline)
    builddir=ROOT/'artifacts/sdl3/build'/plat
    stage=ROOT/'artifacts/sdl3/install'/plat
    if stage.exists(): shutil.rmtree(stage)
    subprocess.run(['cmake','-S',str(ROOT/'tools/native/sdl3'),'-B',str(builddir),
                    f'-DSDL_SOURCE={src}',f'-DSDL_JAI_PLATFORM={plat}',f'-DCMAKE_INSTALL_PREFIX={stage}',
                    *configure_options(plat)],check=True)
    subprocess.run(['cmake','--build',str(builddir),'--config','Release','--parallel',str(jobs)],check=True)
    if plat == host():
        for mode in ('static','shared'):
            program=builddir/('Release' if plat.startswith('windows') else '')/('jai-sdl3-smoke-'+mode+('.exe' if plat.startswith('windows') else ''))
            subprocess.run([str(program)],check=True)
    subprocess.run(['cmake','--install',str(builddir),'--config','Release'],check=True)
    ledger=json.loads((ROOT/'stdlib/Extensions/SDL3/coverage.json').read_text())
    installed={p.name:p for p in (stage/'include/SDL3').glob('*.h')}
    classified=ledger['headers']
    expected_headers={name for name,entry in classified.items() if entry['category']!='auxiliary_test_library'}
    if set(installed)!=expected_headers:raise RuntimeError('SDL installed header set differs from the comprehensive core coverage ledger')
    header_evidence={}
    for name,path in sorted(installed.items()):
        installed_hash=hashlib.sha256(path.read_bytes()).hexdigest()
        source_hash=classified[name]['sha256']
        if installed_hash!=source_hash and name!='SDL_revision.h':raise RuntimeError(f'installed SDL header differs from verified source: {name}')
        header_evidence[name]={'source_sha256':source_hash,'installed_sha256':installed_hash,
                               'status':'source_identical' if source_hash==installed_hash else 'generated_revision_metadata'}
    dependencies=(builddir/'static-link-libraries-Release.txt').read_text()
    check_dependencies(dependencies,plat)
    out=out.resolve();out.mkdir(parents=True,exist_ok=True)
    # jaic searches SDL3.lib/libSDL3.a first: these are the actual static archives.
    # Keep the distinct official Windows DLL import library as SDL3-import.lib.
    if plat.startswith('windows'):
        shutil.copy2(stage/'lib/SDL3-static.lib',out/'SDL3.lib')
        shutil.copy2(stage/'lib/SDL3.lib',out/'SDL3-import.lib')
        shutil.copy2(stage/'bin/SDL3.dll',out/'SDL3.dll')
        names=['SDL3.lib','SDL3-import.lib','SDL3.dll']
    else:
        shutil.copy2(stage/'lib/libSDL3.a',out/'libSDL3.a')
        ext='dylib' if plat.startswith('macos') else 'so'
        # Preserve SONAME/install-name aliases. The unversioned name is used by jaic -interp.
        for f in (stage/'lib').glob(f'libSDL3*.{ext}*'):
            shutil.copy2(f,out/f.name,follow_symlinks=True)
        names=['libSDL3.a']+[f.name for f in out.glob(f'libSDL3*.{ext}*')]
    lic=out/'licenses/sdl3';lic.mkdir(parents=True,exist_ok=True)
    from build_native_libs import copy_licenses
    from sdl3_gen import native_notices
    copy_licenses('sdl3',source,src,out)
    (lic/'THIRD-PARTY-NOTICES.txt').write_text(native_notices(src))
    metadata=out/'sdl3';metadata.mkdir(exist_ok=True)
    (metadata/'installed-headers.json').write_text(json.dumps(header_evidence,indent=2,sort_keys=True)+'\n')
    configs=list(builddir.glob('sdl/include-config-release/build_config/SDL_build_config.h'))
    if not configs: raise RuntimeError('missing generated SDL build configuration')
    config=configs[0].read_text()
    required={'macos':['SDL_VIDEO_DRIVER_COCOA','SDL_AUDIO_DRIVER_COREAUDIO','SDL_GPU_METAL','SDL_JOYSTICK_HIDAPI'],
              'linux':['SDL_VIDEO_DRIVER_X11','SDL_VIDEO_DRIVER_WAYLAND','SDL_AUDIO_DRIVER_ALSA','SDL_AUDIO_DRIVER_PULSEAUDIO','SDL_AUDIO_DRIVER_PIPEWIRE','SDL_GPU_VULKAN','SDL_JOYSTICK_HIDAPI'],
              'windows':['SDL_VIDEO_DRIVER_WINDOWS','SDL_AUDIO_DRIVER_WASAPI','SDL_GPU_D3D12','SDL_JOYSTICK_HIDAPI']}[plat.split('-')[0]]
    for flag in required:
        if f'#define {flag} 1' not in config: raise RuntimeError(f'required SDL backend disabled: {flag}')
    shutil.copy2(configs[0],metadata/'SDL_build_config.h')
    for f in (stage/'lib/pkgconfig').glob('*.pc'):
        text=f.read_text().replace(str(stage),'${pcfiledir}/../../..')
        (metadata/f.name).write_text(text)
    (metadata/'static-link-libraries.txt').write_text(dependencies)
    # Package CMake targets for native consumers and dependency auditing.
    shutil.copytree(stage/'lib/cmake/SDL3',metadata/'cmake',dirs_exist_ok=True)
    record={'platform':plat,'version':'3.4.18','source':source,'cmake_options':configure_options(plat),
            'native_smoke_executed': plat==host(),
            'artifacts':{n:hashlib.sha256((out/n).read_bytes()).hexdigest() for n in sorted(names)}}
    (metadata/'build.json').write_text(json.dumps(record,indent=2)+'\n')
    print(f'SDL3 {plat}: '+', '.join(sorted(names)))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--platform',choices=PLATFORMS,default=host())
    p.add_argument('--out',type=Path)
    p.add_argument('--cache',type=Path)
    p.add_argument('--offline',action='store_true')
    p.add_argument('--jobs',type=int,default=2)
    a=p.parse_args();build(a.out or ROOT/'artifacts/native-libs'/a.platform,a.platform,a.cache,a.offline,a.jobs)
