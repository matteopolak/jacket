#!/usr/bin/env python3
"""Compile all real allocator domains from prepared pinned SDK/owned adapters."""
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess

from prepare_rpmalloc import ROOT, prepare
from rpmalloc_profiles import PROFILES, c_definitions


def build(output, architectures=(), selected=None, source=None, sanitize=(), cc=None):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    source = source or prepare(ROOT/'.work/vendor/rpmalloc', output/'rpmalloc-source')
    api = json.loads((ROOT/'stdlib/internal/bindings/rpmalloc-sdk/provenance.json').read_text())
    checks = []
    cc = cc or os.environ.get('CC', 'cc')
    system = platform.system()
    if system not in ('Darwin', 'Linux'):
        raise ValueError('Native allocator artifact construction currently requires macOS or Linux; Windows SDK packaging remains unsupported')

    def checked(command):
        command = [str(x) for x in command]
        process = subprocess.run(command, capture_output=True, text=True)
        checks.append(dict(command=command, exit=process.returncode,
                           stdout=process.stdout, stderr=process.stderr))
        (output/'rpmalloc-build.json').write_text(json.dumps(checks, indent=2)+'\n')
        if process.returncode:
            raise SystemExit(process.stdout + process.stderr)

    sanitizer_flags = ['-fsanitize='+','.join(sanitize)] if sanitize else []
    for profile in PROFILES:
        for kind, folder in [('sdk', 'rpmalloc-heaps'), ('callbacks', 'rpmalloc-callbacks')]:
            if selected is not None and profile[kind] not in selected:
                continue
            archives = []
            shared_libraries = []
            for arch in architectures or [None]:
                name = profile[kind]+('-'+arch if arch else '')
                obj = output/(name+'.o')
                archive = output/(name+'.a')
                commands = [[cc, '-std=c11', '-Wall', '-Wextra', '-Werror', '-O2', '-fPIC',
                             *(['-arch', arch] if arch else []), '-isystem', source,
                             *c_definitions(profile, api['functions']), *sanitizer_flags, '-c',
                             ROOT/'stdlib/internal/bindings'/folder/'bridge.c', '-o', obj],
                            ['ar', 'rcs', archive, obj]]
                for command in commands:
                    checked(command)
                archives.append(archive)
                suffix = '.dylib' if system == 'Darwin' else '.so'
                shared = output/(name+suffix)
                filename = 'lib'+profile[kind]+suffix
                dependencies = []
                if kind == 'callbacks':
                    dependencies = ['-L', output, '-l'+profile['sdk']]
                if system == 'Darwin':
                    flags = ['-dynamiclib', '-Wl,-install_name,@rpath/'+filename,
                             '-Wl,-rpath,@loader_path']
                else:
                    flags = ['-shared', '-Wl,-soname,'+filename, '-Wl,-rpath,$ORIGIN',
                             '-pthread', '-lm']
                checked([cc, *(['-arch', arch] if arch else []), *flags, *sanitizer_flags,
                         obj, *dependencies, '-o', shared])
                shared_libraries.append(shared)
            target = output/('lib'+profile[kind]+'.a')
            if len(archives) == 1:
                target.write_bytes(archives[0].read_bytes())
            else:
                checked(['lipo', '-create', *archives, '-output', target])
            print('Built '+str(target), flush=True)
            target = output/('lib'+profile[kind]+suffix)
            if len(shared_libraries) == 1:
                target.write_bytes(shared_libraries[0].read_bytes())
            else:
                checked(['lipo', '-create', *shared_libraries, '-output', target])
            print('Built '+str(target), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT/'.work/native')
    parser.add_argument('--arch', action='append', choices=('x86_64', 'arm64'), default=[])
    parser.add_argument('--sanitize', action='append', choices=('address', 'undefined'), default=[])
    parser.add_argument('--cc', type=Path, help='Native compiler; use the Jai LLVM toolchain for sanitizer builds')
    args = parser.parse_args()
    if args.arch and platform.system() != 'Darwin':
        parser.error('--arch is supported by the Mac universal build only')
    build(args.output, args.arch, sanitize=args.sanitize, cc=args.cc)


if __name__ == '__main__':
    main()
