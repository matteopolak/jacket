#!/usr/bin/env python3
"""Pinned SDL3 native build; also called by build_native_libs.py. Never consults another checkout."""
from __future__ import annotations
import argparse
import hashlib
import json
import ntpath
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
WINDOWS_ARTIFACTS={'SDL3.lib':('lib/SDL3-static.lib','static'),
                   'SDL3-import.lib':('lib/SDL3.lib','import'),
                   'SDL3.dll':('bin/SDL3.dll','shared')}

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
        options += ['-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded','-DSDL_LIBC=ON']
    return options

def select_windows_toolchain(instances, generators, plat, env):
    """Match real, component-filtered vswhere instances to this CMake's generators.

    No release-year mapping: new VS versions work when CMake advertises their
    matching generator. An explicit environment selection must also be installed.
    """
    cpu={'windows-x64':'x64','windows-arm64':'ARM64'}[plat]
    requested=env.get('CMAKE_GENERATOR')
    location=env.get('CMAKE_GENERATOR_INSTANCE') or env.get('VSINSTALLDIR')
    if env.get('CMAKE_GENERATOR_PLATFORM',cpu)!=cpu:
        raise RuntimeError(f'CMAKE_GENERATOR_PLATFORM conflicts with {plat}; expected {cpu}')
    toolset=f'host={cpu}'
    if env.get('CMAKE_GENERATOR_TOOLSET',toolset)!=toolset:
        raise RuntimeError(f'CMAKE_GENERATOR_TOOLSET conflicts with native MSVC selection; expected {toolset}')
    available={int(m[1]):g for g in generators if (m:=re.fullmatch(r'Visual Studio (\d+) \d{4}',g))}
    compatible=[]
    for instance in instances:
        if not instance.get('isComplete') or not instance.get('isLaunchable'):continue
        version=tuple(int(part) for part in instance['installationVersion'].split('.'))
        generator=available.get(version[0])
        path=instance['installationPath']
        if not generator or (requested and generator!=requested):continue
        if location and ntpath.normcase(ntpath.normpath(location))!=ntpath.normcase(ntpath.normpath(path)):continue
        compatible.append((version,path,generator))
    if not compatible:
        raise RuntimeError(f'No compatible installed Visual Studio C++ toolchain for {plat}. '
                           f'CMake generators: {generators}; component-filtered VS instances: {instances}. '
                           'Install native MSVC target tools and a Windows SDK; upgrade CMake if it '
                           'does not support the installed VS. Remove conflicting CMAKE_GENERATOR* '
                           'or VSINSTALLDIR overrides. Ninja/MinGW are not used by this MSVC builder.')
    version,path,generator=max(compatible)
    return ['-G',generator,'-A',cpu,'-T',toolset,f'-DCMAKE_GENERATOR_INSTANCE={path}']

def windows_toolchain(plat, env=None):
    env=dict(os.environ if env is None else env)
    # Microsoft's installer location is stable; PATH supports custom installations.
    candidates=[Path(env[key])/'Microsoft Visual Studio/Installer/vswhere.exe'
                for key in ('ProgramFiles(x86)','ProgramFiles') if env.get(key)]
    vswhere=next((str(p) for p in candidates if p.is_file()),None) or shutil.which('vswhere.exe')
    if not vswhere:raise RuntimeError('vswhere.exe not found: install Visual Studio C++ Build Tools and Windows SDK')
    component='ARM64' if plat=='windows-arm64' else 'x86.x64'
    result=subprocess.run([vswhere,'-products','*','-requires','Microsoft.Component.MSBuild',
                           f'Microsoft.VisualStudio.Component.VC.Tools.{component}',
                           '-format','json','-utf8'],check=True,capture_output=True,encoding='utf-8-sig')
    instances=json.loads(result.stdout)
    result=subprocess.run(['cmake','-E','capabilities'],check=True,capture_output=True,encoding='utf-8')
    generators=[g['name'] for g in json.loads(result.stdout)['generators']]
    return select_windows_toolchain(instances,generators,plat,env)

def prepare_windows_build_tree(builddir, args):
    """CMake forbids changing generator/instance/platform/toolset in an existing tree."""
    cache=builddir/'CMakeCache.txt'
    if not cache.exists():return
    values=dict(re.findall(r'^(CMAKE_GENERATOR(?:_INSTANCE|_PLATFORM|_TOOLSET)?):[^=]*=(.*)$',
                           cache.read_text(),re.MULTILINE))
    expected={'CMAKE_GENERATOR':args[args.index('-G')+1],
              'CMAKE_GENERATOR_PLATFORM':args[args.index('-A')+1],
              'CMAKE_GENERATOR_TOOLSET':args[args.index('-T')+1],
              'CMAKE_GENERATOR_INSTANCE':next(a.split('=',1)[1] for a in args if a.startswith('-DCMAKE_GENERATOR_INSTANCE='))}
    if any(values.get(key)!=value for key,value in expected.items()):
        print(f'SDL3: clearing incompatible generated CMake tree {builddir}')
        shutil.rmtree(builddir)

def package_windows_artifacts(stage, out):
    # CMake's static target and DLL linker target both end in .lib; never conflate them.
    if (stage/'lib/SDL3-static.lib').read_bytes()==(stage/'lib/SDL3.lib').read_bytes():
        raise RuntimeError('SDL3 static archive and DLL import library are identical')
    for name,(source,role) in WINDOWS_ARTIFACTS.items():
        shutil.copy2(stage/source,out/name)
    return list(WINDOWS_ARTIFACTS)

def installed_cmake_dir(stage, manifest):
    """Use CMake's actual install result, not a Unix directory convention."""
    configs=[Path(line) for line in manifest.read_text(encoding='utf-8').splitlines()
             if Path(line).name=='SDL3Config.cmake']
    if len(configs)!=1:raise RuntimeError(f'expected exactly one installed SDL3Config.cmake in {manifest}: {configs}')
    config=configs[0].resolve()
    if not config.is_relative_to(stage.resolve()):raise RuntimeError(f'installed SDL3 config outside package prefix: {config}')
    directory=config.parent
    required=('SDL3Config.cmake','SDL3ConfigVersion.cmake','SDL3headersTargets.cmake',
              'SDL3staticTargets.cmake','SDL3staticTargets-release.cmake',
              'SDL3sharedTargets.cmake','SDL3sharedTargets-release.cmake')
    for name in required:
        if not (directory/name).is_file():raise RuntimeError(f'missing installed CMake export: {directory/name}')
    return directory

def package_installed_sdk(stage, out, manifest):
    """Keep the official prefix-relative exports, headers and artifacts together."""
    directory=installed_cmake_dir(stage,manifest)
    relative=directory.relative_to(stage.resolve())
    sdk=out/'sdl3/sdk'
    if sdk.resolve().is_relative_to(stage.resolve()):raise ValueError('SDL3 SDK output must be outside its install staging tree')
    if sdk.exists():shutil.rmtree(sdk)
    # Materialize Unix alias symlinks as identical files so zip extraction also
    # preserves every imported SONAME/install-name path without symlink support.
    shutil.copytree(stage,sdk,symlinks=False)
    print(f'SDL3 installed CMake package: {relative.as_posix()} -> {sdk/relative}',flush=True)
    return {'prefix':'sdl3/sdk','cmake_dir':relative.as_posix(),
            'files':{p.relative_to(sdk).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()
                     for p in sorted(sdk.rglob('*')) if p.is_file()}}

def check_relocated_consumers(out, info, plat, cmake_args, jobs):
    work=ROOT/'artifacts/sdl3/consumers'/plat
    if work.exists():shutil.rmtree(work)
    work.mkdir(parents=True)
    relocated=work/'relocated SDK'
    shutil.copytree(out/info['prefix'],relocated)
    builddir=work/'build'
    subprocess.run(['cmake','-S',str(ROOT/'tools/native/sdl3/consumer'),'-B',str(builddir),
                    f'-DJAI_SDL3_PREFIX={relocated}',f'-DSDL3_DIR={relocated/info["cmake_dir"]}',
                    f'-DJAI_SDL3_PLATFORM={plat}','-DCMAKE_BUILD_TYPE=Release',*cmake_args],check=True)
    subprocess.run(['cmake','--build',str(builddir),'--config','Release','--parallel',str(jobs)],check=True)
    if plat==host():
        subprocess.run(['ctest','--test-dir',str(builddir),'-C','Release','--output-on-failure'],check=True)
    return {'compiled':['c-static','c-shared','cpp-static','cpp-shared'],'native_executed':plat==host()}

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
    if plat.startswith(('linux','windows')) and plat!=host(): raise ValueError('Linux/Windows SDL3 release builds require a native target runner (no host-architecture fallback)')
    cache=(cache or ROOT/'artifacts/sdl3').resolve()
    src,source=checked_archive(cache,offline)
    builddir=ROOT/'artifacts/sdl3/build'/plat
    stage=ROOT/'artifacts/sdl3/install'/plat
    options=configure_options(plat)
    consumer_args=[option for option in options if option.startswith(('-DCMAKE_OSX_','-DCMAKE_MSVC_RUNTIME_LIBRARY='))]
    if plat.startswith('windows'):
        toolchain=windows_toolchain(plat)
        print('SDL3 Windows toolchain: '+json.dumps(toolchain),flush=True)
        prepare_windows_build_tree(builddir,toolchain)
        options=toolchain+options
        consumer_args=toolchain+consumer_args
    if stage.exists(): shutil.rmtree(stage)
    subprocess.run(['cmake','-S',str(ROOT/'tools/native/sdl3'),'-B',str(builddir),
                    f'-DSDL_SOURCE={src}',f'-DSDL_JAI_PLATFORM={plat}',f'-DCMAKE_INSTALL_PREFIX={stage}',
                    *options],check=True)
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
        names=package_windows_artifacts(stage,out)
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
    # Match the exact LF/UTF-8 checked-in generated notice asset on Windows too.
    (lic/'THIRD-PARTY-NOTICES.txt').write_bytes(native_notices(src).encode('utf-8'))
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
    (metadata/'static-link-libraries.txt').write_text(dependencies)
    # Export files must remain at their installed prefix-relative depth with all
    # referenced libraries and headers; copying only cmake/ breaks relocation.
    sdk=package_installed_sdk(stage,out,builddir/'install_manifest.txt')
    old_metadata=metadata/'cmake'
    if old_metadata.exists():shutil.rmtree(old_metadata)
    for old_pc in metadata.glob('*.pc'):old_pc.unlink()
    consumers=check_relocated_consumers(out,sdk,plat,consumer_args,jobs)
    record={'platform':plat,'version':'3.4.18','source':source,'cmake_options':options,
            'sdk':sdk,'relocated_consumers':consumers,
            'native_smoke_executed': plat==host(),
            'artifacts':{n:hashlib.sha256((out/n).read_bytes()).hexdigest() for n in sorted(names)}}
    if plat.startswith('windows'):
        record['artifact_roles']={name:role for name,(source,role) in WINDOWS_ARTIFACTS.items()}
    (metadata/'build.json').write_text(json.dumps(record,indent=2)+'\n')
    print(f'SDL3 {plat}: '+', '.join(sorted(names)))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--platform',choices=PLATFORMS,default=host())
    p.add_argument('--out',type=Path)
    p.add_argument('--cache',type=Path)
    p.add_argument('--offline',action='store_true')
    p.add_argument('--jobs',type=int,default=2)
    p.add_argument('--show-toolchain',action='store_true',help='print discovered Windows CMake arguments without downloading/building')
    a=p.parse_args()
    if a.show_toolchain:
        if not a.platform.startswith('windows'):p.error('--show-toolchain requires a Windows platform')
        print(json.dumps(windows_toolchain(a.platform),indent=2))
    else:
        build(a.out or ROOT/'artifacts/native-libs'/a.platform,a.platform,a.cache,a.offline,a.jobs)
