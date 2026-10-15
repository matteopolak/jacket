#!/usr/bin/env python3
"""Compile all real allocator domains from prepared pinned SDK/owned adapters."""
import argparse
import json
from pathlib import Path
import platform
import subprocess

from prepare_rpmalloc import ROOT, prepare
from rpmalloc_profiles import PROFILES, c_definitions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT/'.work/native')
    parser.add_argument('--arch', action='append', choices=('x86_64', 'arm64'), default=[])
    args = parser.parse_args()
    if args.arch and platform.system() != 'Darwin':
        parser.error('--arch is supported by the Mac universal build only')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    source = prepare(ROOT/'.work/vendor/rpmalloc', output/'rpmalloc-source')
    api = json.loads((ROOT/'stdlib/internal/bindings/rpmalloc-sdk/provenance.json').read_text())
    checks = []
    for profile in PROFILES:
        for kind, folder in [('sdk', 'rpmalloc-heaps'), ('callbacks', 'rpmalloc-callbacks')]:
            archives = []
            for arch in args.arch or [None]:
                name = profile[kind]+('-'+arch if arch else '')
                obj = output/(name+'.o')
                archive = output/(name+'.a')
                commands = [['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-O2', '-fPIC',
                             *(['-arch', arch] if arch else []), '-isystem', source,
                             *c_definitions(profile, api['functions']), '-c',
                             ROOT/'stdlib/internal/bindings'/folder/'bridge.c', '-o', obj],
                            ['ar', 'rcs', archive, obj]]
                for command in commands:
                    command = [str(x) for x in command]
                    p = subprocess.run(command, capture_output=True, text=True)
                    checks.append(dict(command=command, exit=p.returncode, stdout=p.stdout, stderr=p.stderr))
                    (output/'rpmalloc-build.json').write_text(json.dumps(checks, indent=2)+'\n')
                    if p.returncode:
                        raise SystemExit(p.stderr)
                archives.append(archive)
            target = output/('lib'+profile[kind]+'.a')
            if len(archives) == 1:
                target.write_bytes(archives[0].read_bytes())
            else:
                subprocess.run(['lipo', '-create', *map(str, archives), '-output', str(target)], check=True)
            print('Built '+str(target), flush=True)


if __name__ == '__main__':
    main()
