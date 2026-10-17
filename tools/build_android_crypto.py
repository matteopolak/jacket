#!/usr/bin/env python3
"""Build/package canonical crypto from a configured real OpenSSL3 SDK archive.

Configuration applies only at package construction. No runtime override is added.
Only public headers/license are read; upstream implementation bodies are not used
as clean-room reference material. The native linker consumes the SDK's archive.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tempfile

from openssl_sdk import OpenSSLSDK, SDKError, discover_sdk
from android_crypto_linux import build_linux

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'tools/android-crypto-native.json'


def run(command):
    return subprocess.run(list(map(str, command)), check=True, capture_output=True,
                          text=True, timeout=180).stdout


def dependencies(binary):
    return [line.strip().split(' (compatibility')[0]
            for line in run(['otool', '-L', binary]).splitlines()[1:]]


def build(sdk, stdlib):
    spec = json.loads(MANIFEST.read_text())
    if platform.system() == 'Linux':
        descriptor = sdk if isinstance(sdk, OpenSSLSDK) else discover_sdk(sdk)
        return build_linux(descriptor, stdlib, spec, ROOT, MANIFEST)
    sdk = (sdk.prefix if isinstance(sdk, OpenSSLSDK) else sdk).absolute()
    for name in spec['sdk_inputs']:
        if not (sdk / name).is_file():
            raise ValueError(f'OpenSSL SDK is missing {name}: {sdk}')
    header = (sdk / 'include/openssl/opensslv.h').read_text()
    match = re.search(r'^\s*#\s*define\s+OPENSSL_VERSION_MAJOR\s+(\d+)\s*$', header, re.M)
    if not match or match[1] != '3':
        raise ValueError('OpenSSL SDK must declare OPENSSL_VERSION_MAJOR 3')
    version = re.search(r'^\s*#\s*define\s+OPENSSL_VERSION_TEXT\s+"([^"]+)"', header, re.M)
    if not version:
        raise ValueError('OpenSSL SDK has no public version text')
    if platform.system() != 'Darwin':
        raise ValueError('native packaging is implemented for Darwin and Linux; Windows remains unverified')
    output = stdlib.absolute() / spec['provider_relative_directory']
    output.parent.mkdir(parents=True, exist_ok=True)
    # Validate everything in a sibling staging directory before replacing output.
    with tempfile.TemporaryDirectory(prefix='android-crypto-build-', dir=output.parent) as temporary:
        stage = Path(temporary)
        archive = stage / 'libcrypto.a'
        shared = stage / 'libcrypto.dylib'
        shutil.copyfile(sdk / 'lib/libcrypto.a', archive)
        run(['cc', '-dynamiclib', '-Xlinker', '-force_load', '-Xlinker', archive,
             '-Wl,-install_name,@rpath/libcrypto.dylib', '-o', shared])
        identity = run(['otool', '-D', shared]).splitlines()[1:]
        if identity != [spec['macos']['shared_identity']]:
            raise ValueError(f'wrong shared identity: {identity}')
        deps = dependencies(shared)
        if not deps or deps[0] != spec['macos']['shared_identity']:
            raise ValueError(f'wrong dependency identity: {deps}')
        foreign = [name for name in deps[1:] if not any(
            name.startswith(prefix) for prefix in spec['macos']['allowed_dependency_prefixes'])]
        if foreign:
            raise ValueError(f'undeclared third-party dependency closure: {foreign}')
        run(['codesign', '--force', '--sign', '-', shared])
        run(['codesign', '--verify', shared])
        abi = ROOT / 'tests/native/android-crypto/abi.c'
        runtime_versions = {}
        for artifact in [archive, shared]:
            executable = stage / ('abi-' + artifact.suffix[1:])
            run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-I', sdk / 'include',
                 abi, artifact, '-Wl,-rpath,' + str(stage), '-o', executable])
            run([executable])
            executable.unlink()
            run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-I', sdk / 'include',
                 ROOT / 'tests/native/android-crypto/package-version.c', artifact,
                 '-Wl,-rpath,' + str(stage), '-o', executable])
            actual_version = run([executable]).strip()
            if actual_version != version[1]:
                raise ValueError(f'SDK header/archive version mismatch: {version[1]} / {actual_version}')
            runtime_versions[artifact.name] = actual_version
            executable.unlink()
        licenses = stage / 'licenses/openssl'
        licenses.mkdir(parents=True)
        for name in ['LICENSE.txt', 'AUTHORS.md']:
            shutil.copyfile(sdk / name, licenses / name)
        # Record logical distribution files, excluding filesystem AppleDouble metadata.
        files = ['libcrypto.a', 'libcrypto.dylib', 'licenses/openssl/LICENSE.txt', 'licenses/openssl/AUTHORS.md']
        hashes = {name: hashlib.sha256((stage / name).read_bytes()).hexdigest() for name in files}
        provenance = {
            'format': 1, 'implementation': version[1], 'openssl_major': 3,
            'sdk_prefix': str(sdk), 'sdk_resolved_prefix': str(sdk.resolve()),
            'sdk_archive_sha256': hashlib.sha256((sdk / 'lib/libcrypto.a').read_bytes()).hexdigest(),
            'shared_identity': identity[0], 'runtime_dependencies': deps[1:],
            'third_party_runtime_dependencies': foreign,
            'architectures': run(['lipo', '-archs', shared]).strip(),
            'build': 'native linker force-loads actual SDK libcrypto.a; no compatibility shim',
            'license': 'Apache-2.0', 'sha256': hashes,
            'runtime_versions': runtime_versions,
            'manifest_sha256': hashlib.sha256(MANIFEST.read_bytes()).hexdigest(),
            'native_platforms_verified_by_build': ['macos-' + platform.machine()],
            'linux_windows_native_verified': False
        }
        (stage / 'provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
        if output.exists():
            shutil.rmtree(output)
        shutil.copytree(stage, output, ignore=shutil.ignore_patterns('._*'))
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    selection = parser.add_mutually_exclusive_group(required=True)
    selection.add_argument('--openssl-sdk-descriptor', type=Path,
                           help='JSON descriptor from canonical OpenSSL SDK discovery')
    selection.add_argument('--openssl-prefix', type=Path,
                        help='configured package-build SDK input, not a runtime override')
    parser.add_argument('--stdlib', required=True, type=Path,
                        help='distribution staging stdlib to receive the native package')
    args = parser.parse_args()
    try:
        sdk = args.openssl_prefix
        if args.openssl_sdk_descriptor:
            fields = json.loads(args.openssl_sdk_descriptor.read_text())
            for key in ('prefix', 'include_dir', 'library'):
                fields[key] = Path(fields[key])
            for key in ('cflags', 'ldflags'):
                fields[key] = tuple(fields[key])
            sdk = OpenSSLSDK(**fields)
        print(build(sdk, args.stdlib))
    except (ValueError, SDKError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'{error}\n' + (error.stderr if isinstance(error, subprocess.CalledProcessError) else ''))


if __name__ == '__main__':
    main()
