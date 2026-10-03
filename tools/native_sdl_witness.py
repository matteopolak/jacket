#!/usr/bin/env python3
"""Build an exact reviewed SDL2 source revision and execute a bounded CPU witness."""
from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
import hashlib
import json
from pathlib import Path
import platform
import shlex
import subprocess
import tarfile

import native_dependencies as native

REPOSITORY = 'libsdl-org/SDL'
REVISION = 'f461d91cd265d7b9a44b4d472b1df0c0ad2855a0'  # official release-2.30.2
SOURCE_MANIFEST_SHA256 = 'c0dd50c945986d2f9b85a00cc6ca3f320728c9b7d043661bc58b263005c6b572'
SOURCE_ARCHIVE_SHA256 = '95850b99c5b3c5b3407f5ff1fc0b0a8e811ad4c7f05281b275324f54eb982cdc'
DEFAULT_SOURCE = native.ROOT / 'artifacts/native-dependencies/source/sdl2-2.30.2'
DISABLED = ('AUDIO', 'VIDEO', 'RENDER', 'JOYSTICK', 'HAPTIC', 'POWER', 'LOADSO',
            'CPUINFO', 'FILESYSTEM', 'DLOPEN', 'SENSOR', 'HIDAPI')
FLAGS = ('-DCMAKE_POLICY_VERSION_MINIMUM=3.5', '-DCMAKE_BUILD_TYPE=Release',
         '-DCMAKE_C_FLAGS=-MD', '-DSDL_SHARED=OFF', '-DSDL_STATIC=ON',
         '-DSDL_TESTS=OFF', '-DSDL_TEST_LIBRARY=OFF', '-DCMAKE_DISABLE_FIND_PACKAGE_PkgConfig=TRUE',
         '-DCMAKE_DISABLE_FIND_PACKAGE_Git=TRUE',
         '-DSDL_OPENGL=OFF', '-DSDL_OPENGLES=OFF', '-DSDL_VULKAN=OFF',
         '-DSDL_ASSEMBLY=OFF', *(f'-DSDL_{name}=OFF' for name in DISABLED))
WITNESS = r'''#include <SDL.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    SDL_version version;
    SDL_GetVersion(&version);
    if (version.major != 2 || version.minor != 30 || version.patch != 2) return 10;
    if (sizeof(SDL_Rect) != 16 || _Alignof(SDL_Rect) != 4) return 11;
    if (offsetof(SDL_Rect, x) != 0 || offsetof(SDL_Rect, y) != 4 ||
        offsetof(SDL_Rect, w) != 8 || offsetof(SDL_Rect, h) != 12) return 12;
    if (sizeof(SDL_UserEvent) != 32 || _Alignof(SDL_UserEvent) != 8 ||
        offsetof(SDL_UserEvent, code) != 12 || offsetof(SDL_UserEvent, data1) != 16 ||
        offsetof(SDL_UserEvent, data2) != 24) return 13;
    if (sizeof(SDL_Event) != 56 || _Alignof(SDL_Event) != 8) return 14;
    SDL_Rect a = {0, 0, 10, 8}, b = {4, 3, 8, 9}, out;
    if (!SDL_IntersectRect(&a, &b, &out) || out.x != 4 || out.y != 3 || out.w != 6 || out.h != 5) return 15;
    SDL_UnionRect(&a, &b, &out);
    if (out.x != 0 || out.y != 0 || out.w != 12 || out.h != 12) return 16;
    SDL_ClearError();
    if (SDL_IntersectRect(NULL, &b, &out) != SDL_FALSE || !strstr(SDL_GetError(), "A")) return 17;
    SDL_ClearError();
    if (SDL_Init(SDL_INIT_EVENTS) != 0) return 18;
    Uint32 type = SDL_RegisterEvents(1);
    if (type == (Uint32)-1) return 19;
    int value = 42;
    SDL_Event event = {0}, received = {0};
    event.type = type;
    event.user.code = 73;
    event.user.data1 = &value;
    if (SDL_PushEvent(&event) != 1 || SDL_PollEvent(&received) != 1) return 20;
    if (received.type != type || received.user.code != 73 || received.user.data1 != &value) return 21;
    if (SDL_PollEvent(&received) != 0) return 22;
    SDL_Quit();
    puts("SDL_version 2 30 2");
    puts("SDL_Rect 16 4 0 4 8 12");
    puts("SDL_UserEvent 32 8 12 16 24");
    puts("SDL_Event 56 8");
    puts("cpu_geometry_error_event_queue 1");
    return 0;
}
'''
EXPECTED = ('SDL_version 2 30 2\nSDL_Rect 16 4 0 4 8 12\n'
            'SDL_UserEvent 32 8 12 16 24\nSDL_Event 56 8\ncpu_geometry_error_event_queue 1\n')


@dataclass(frozen=True)
class SdlWitness:
    repository: str
    revision: str
    source_manifest: native.Fingerprint
    target: str
    tools: tuple[native.Fingerprint, ...]
    source_inputs: tuple[native.Fingerprint, ...]
    transitive_inputs: tuple[native.Fingerprint, ...]
    build_flags: tuple[str, ...]
    cmake_source_sha256: str
    artifact: native.Fingerprint
    witness_source_sha256: str
    executable: native.Fingerprint
    observed: str
    runtime_exit_code: int
    full_project_acceptance: bool = False
    link_authority: bool = False


def prepare_source(archive: Path, output: Path, root: Path = native.ROOT) -> None:
    archive, output = native.outside_inputs(archive, root), native.outside_inputs(output, root)
    if native.sha256(archive) != SOURCE_ARCHIVE_SHA256:
        raise ValueError('SDL source archive differs from reviewed official source pin')
    if output.exists() or not output.is_relative_to((root / 'artifacts/native-dependencies').resolve()):
        raise ValueError('SDL source extraction requires a fresh managed directory')
    rows = []
    with tarfile.open(archive) as source:
        for member in source:
            if not member.isfile():
                continue  # No links, devices or directories are extracted from an archive.
            relative = Path(*Path(member.name).parts[1:])
            if relative.is_absolute() or '..' in relative.parts:
                raise ValueError('unsafe SDL source archive path')
            if (relative.name not in ('CMakeLists.txt', 'COPYING.txt', 'README.txt', 'LICENSE.txt')
                    and relative.suffix not in ('.c', '.h', '.m', '.cmake', '.in')):
                continue
            content = source.extractfile(member).read()
            if b'\0' in content:
                raise ValueError('selected SDL source contains binary bytes')
            destination = output / 'tree' / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(content)
            rows.append({'path': relative.as_posix(), 'sha256': hashlib.sha256(content).hexdigest()})
    manifest = {'repository': REPOSITORY, 'revision': REVISION,
                'archive_sha256': SOURCE_ARCHIVE_SHA256, 'files': sorted(rows, key=lambda row: row['path'])}
    path = output / 'source-manifest.json'
    path.write_text(json.dumps(manifest, indent=2) + '\n')
    source_inputs(output, root)


def source_inputs(source: Path, root: Path = native.ROOT) -> tuple[native.Fingerprint, ...]:
    source = native.outside_inputs(source, root)
    manifest = source / 'source-manifest.json'
    if native.sha256(manifest) != SOURCE_MANIFEST_SHA256:
        raise ValueError('SDL source manifest differs from reviewed source pin')
    data = json.loads(manifest.read_text())
    if data['repository'] != REPOSITORY or data['revision'] != REVISION:
        raise ValueError('SDL source provenance mismatch')
    found = []
    base = source / 'tree'
    listed = set()
    for item in data['files']:
        relative = Path(item['path'])
        if relative.is_absolute() or '..' in relative.parts or relative in listed:
            raise ValueError('unsafe or repeated SDL source path')
        listed.add(relative)
        path = base / relative
        if path.is_symlink() or not path.resolve().is_relative_to(base.resolve()):
            raise ValueError('SDL source input escapes the reviewed tree')
        entry = native.fingerprint(path, root)
        if entry.sha256 != item['sha256']:
            raise ValueError(f'SDL source fingerprint changed: {relative}')
        found.append(entry)
    if {path.relative_to(base) for path in base.rglob('*') if path.is_file()} != listed:
        raise ValueError('unlisted SDL source input')
    return tuple(found)


def check_observed(stdout: str, stderr: str, status: int) -> str:
    if status != 0 or stderr or stdout != EXPECTED:
        raise ValueError(f'SDL CPU witness failed: exit={status}, stdout={stdout!r}, stderr={stderr!r}')
    return stdout


def build(source: Path, output: Path, root: Path = native.ROOT) -> SdlWitness:
    if platform.system() != 'Darwin' or platform.machine() not in ('arm64', 'aarch64'):
        raise ValueError('reviewed SDL witness currently supports macOS arm64 only')
    entries = source_inputs(source, root)
    output = native.outside_inputs(output, root)
    if output.exists() or not output.is_relative_to((root / 'artifacts/native-dependencies').resolve()):
        raise ValueError('SDL witness requires a fresh managed output directory')
    tools = tuple(native.installed_tool(Path(name), root) for name in
                  ('/opt/homebrew/bin/cmake', '/usr/bin/clang', '/usr/bin/clang++', '/usr/bin/make', '/usr/bin/ar'))
    cmake, compiler, cxx, make, archiver = tools
    output.mkdir(parents=True)
    overlay = output / 'source-overlay'
    overlay.mkdir()
    base = source.resolve() / 'tree'
    modified = (base / 'CMakeLists.txt').read_text()
    (overlay / 'CMakeLists.txt').write_text(modified)
    for path in base.iterdir():
        if path.name != 'CMakeLists.txt':
            (overlay / path.name).symlink_to(path, target_is_directory=path.is_dir())
    binary = output / 'build'
    commands = [[cmake.path, '-S', str(overlay), '-B', str(binary), '-G', 'Unix Makefiles',
                 f'-DCMAKE_C_COMPILER={compiler.path}', f'-DCMAKE_CXX_COMPILER={cxx.path}',
                 f'-DCMAKE_MAKE_PROGRAM={make.path}', f'-DCMAKE_AR={archiver.path}',
                 '-DCMAKE_OSX_ARCHITECTURES=arm64', *FLAGS],
                [cmake.path, '--build', str(binary), '--target', 'SDL2-static', '--parallel', '1']]
    with (output / 'build.log').open('w') as log:
        for command in commands:
            log.write(shlex.join(command) + '\n'); log.flush()
            result = subprocess.run(command, env=native.clean_environment(), stdout=log,
                                    stderr=subprocess.STDOUT, cwd=output, timeout=600)
            if result.returncode:
                raise ValueError(f'SDL source build failed; inspect {output / "build.log"}')
    # All native bytes used below were emitted by this invocation from the verified
    # official source tree. No saved archive or installed SDL library is selected.
    archive = binary / 'libSDL2.a'
    oracle = output / 'cpu-witness.c'
    oracle.write_text(WITNESS)
    executable, dependencies = output / 'cpu-witness', output / 'cpu-witness.d'
    command = [compiler.path, '-std=c11', '-O2', '-I', str(binary / 'include/SDL2'), '-I', str(binary / 'include-config-release/SDL2'),
               '-MD', '-MF', str(dependencies), str(oracle), str(archive), '-liconv',
               '-framework', 'CoreVideo', '-framework', 'Cocoa', '-framework', 'Carbon', '-o', str(executable)]
    with (output / 'witness-build.log').open('w') as log:
        log.write(shlex.join(command) + '\n'); log.flush()
        subprocess.run(command, check=True, env=native.clean_environment(), stdout=log, stderr=subprocess.STDOUT, timeout=120)
    observed = subprocess.run([str(executable)], capture_output=True, text=True,
                              env=native.clean_environment(), timeout=10)
    observation = check_observed(observed.stdout, observed.stderr, observed.returncode)
    inputs = set()
    for dep in [dependencies, *binary.rglob('*.o.d')]:
        for path in native.dependency_paths(dep):
            # Make depfiles can use relative paths with the build directory as cwd.
            inputs.add((path if path.is_absolute() else binary / path).resolve())
    transitive = tuple(native.fingerprint(path, root) for path in sorted(inputs) if path != oracle)
    proof = SdlWitness(REPOSITORY, REVISION, native.fingerprint(source / 'source-manifest.json', root),
                       'arm64-apple-darwin', tools, entries, transitive, FLAGS,
                       hashlib.sha256(modified.encode()).hexdigest(), native.fingerprint(archive, root),
                       hashlib.sha256(WITNESS.encode()).hexdigest(), native.fingerprint(executable, root),
                       observation, observed.returncode)
    # Recheck the full official source set after the build/witness before publishing evidence.
    if source_inputs(source, root) != entries:
        raise ValueError('SDL source changed while building the witness')
    (output / 'receipt.json').write_text(json.dumps(asdict(proof), indent=2) + '\n')
    return proof


def verify_receipt(path: Path, root: Path = native.ROOT) -> dict:
    """Verify saved build evidence without rerunning or authorizing a native link."""
    path = native.outside_inputs(path, root)
    data = json.loads(path.read_text())
    if (data.get('repository') != REPOSITORY or data.get('revision') != REVISION
            or data.get('target') != 'arm64-apple-darwin' or data.get('build_flags') != list(FLAGS)
            or data.get('witness_source_sha256') != hashlib.sha256(WITNESS.encode()).hexdigest()
            or data.get('link_authority') is not False or data.get('full_project_acceptance') is not False):
        raise ValueError('SDL receipt configuration differs from reviewed recipe')
    check_observed(data['observed'], '', data['runtime_exit_code'])
    entries = [data['source_manifest'], data['artifact'], data['executable'],
               *data['tools'], *data['source_inputs'], *data['transitive_inputs']]
    for entry in entries:
        if (not isinstance(entry, dict) or set(entry) != {'path', 'sha256'}
                or not isinstance(entry['path'], str) or not Path(entry['path']).is_absolute()):
            raise ValueError('invalid SDL receipt file fingerprint')
        actual = native.fingerprint(Path(entry['path']), root)
        if actual.sha256 != entry['sha256']:
            raise ValueError(f'SDL receipt fingerprint changed: {actual.path}')
    managed = (root / 'artifacts/native-dependencies').resolve()
    if not all(Path(data[key]['path']).resolve().is_relative_to(managed) for key in ('artifact', 'executable')):
        raise ValueError('SDL receipt artifacts outside managed outputs')
    source = Path(data['source_manifest']['path']).parent
    expected = source_inputs(source, root)
    if json.loads(json.dumps([asdict(item) for item in expected])) != data['source_inputs']:
        raise ValueError('SDL receipt source set differs from reviewed manifest')
    if data['cmake_source_sha256'] != native.sha256(source / 'tree/CMakeLists.txt'):
        raise ValueError('SDL build definition differs from reviewed source')
    return data


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify-receipt', type=Path)
    parser.add_argument('--prepare-source-archive', type=Path)
    parser.add_argument('--source', type=Path, default=DEFAULT_SOURCE)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if args.verify_receipt:
        data = verify_receipt(args.verify_receipt)
        print(json.dumps({'artifact': data['artifact'], 'observed': data['observed'], 'link_authority': False}))
        return
    if args.output is None:
        parser.error('--output is required for source preparation or a fresh build')
    if args.prepare_source_archive:
        prepare_source(args.prepare_source_archive, args.output)
        print(args.output)
        return
    proof = build(args.source, args.output)
    print(json.dumps({'artifact': asdict(proof.artifact), 'observed': proof.observed,
                      'link_authority': proof.link_authority, 'full_project_acceptance': False}))


if __name__ == '__main__':
    main()
