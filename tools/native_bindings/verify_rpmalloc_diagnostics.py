#!/usr/bin/env python3
"""Verify required rpmalloc diagnostic options using immutable current inputs."""
import argparse
import hashlib
import importlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from shared_snapshot import capture, unchanged

CONFIGURATION_FAILURES = (
    ('ENABLE_GLOBAL_CACHE=true, ENABLE_THREAD_CACHE=false', 'ENABLE_GLOBAL_CACHE requires ENABLE_THREAD_CACHE'),
    ('ENABLE_GLOBAL_CACHE=false, ENABLE_UNLIMITED_CACHE=true', 'ENABLE_UNLIMITED_CACHE requires ENABLE_GLOBAL_CACHE'),
    ('DISABLE_UNMAP=true', 'DISABLE_UNMAP requires global and unlimited caches'),
    ('ENABLE_ADAPTIVE_THREAD_CACHE=true, ENABLE_THREAD_CACHE=false, ENABLE_GLOBAL_CACHE=false',
     'ENABLE_ADAPTIVE_THREAD_CACHE requires ENABLE_THREAD_CACHE'),
    ('HEAP_ARRAY_SIZE=0', 'HEAP_ARRAY_SIZE must be positive'),
    ('DEFAULT_SPAN_MAP_COUNT=0', 'DEFAULT_SPAN_MAP_COUNT must be positive'),
    ('GLOBAL_CACHE_MULTIPLIER=0', 'GLOBAL_CACHE_MULTIPLIER must be positive'),
)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'jailint', 'formatter', 'vendor', 'output_root'):
        parser.add_argument('--'+name.replace('_', '-'), type=Path, required=True)
    parser.add_argument('--native-compiler', type=Path, required=True,
                        help='Compiler matching Jai sanitizer runtime (normally its LLVM clang)')
    args = parser.parse_args()
    tools = {name: getattr(args, name).resolve(strict=True) for name in ('compiler', 'jailint', 'formatter')}
    native_compiler = args.native_compiler.resolve(strict=True)
    tools['native_compiler'] = native_compiler
    output = args.output_root.resolve()
    snapshot, identity = capture(output, (
        'tools/native_bindings', 'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-diagnostic-options.jai',
        'docs/specs/stdlib/modules/rpmalloc.md', 'jaifmt.toml', 'jailint.toml'))
    work = Path(tempfile.mkdtemp(prefix='rpmalloc-diagnostics-', dir=output))
    pin = json.loads((snapshot/'tools/native_bindings/dependencies.json').read_text())['rpmalloc']
    vendor = work/'vendor'
    for name, expected in pin['files'].items():
        source = args.vendor/name
        target = vendor/name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        if digest(target) != expected:
            raise ValueError('Pinned SDK input mismatch: '+name)
    # Builders loaded from the immutable capture resolve their ROOT to it.
    sys.path.insert(0, str(snapshot/'tools/native_bindings'))
    prepare = importlib.import_module('prepare_rpmalloc').prepare
    build = importlib.import_module('build_rpmalloc').build
    profiles = importlib.import_module('rpmalloc_profiles').PROFILES
    prepared = prepare(vendor, work/'prepared')
    native_inputs = {str(p.relative_to(work)): digest(p) for p in prepared.iterdir() if p.is_file()}
    for p in prepared.iterdir():
        if p.is_file():
            p.chmod(0o444)
    selected_profiles = [p for p in profiles if not p['raw'] and p['statistics'] and not p['adaptive'] and
                         p['global_cache'] and p['thread_cache']]
    selected = {p[kind] for p in selected_profiles for kind in ('sdk', 'callbacks')}
    report = dict(snapshot=str(snapshot), work=str(work), input_identity=identity,
                  tools={str(p): digest(p) for p in tools.values()}, checks=[],
                  native_inputs=native_inputs, selected_native_domains=selected_profiles,
                  scope='Required assertion/validation options; no optional profile/performance matrix')
    env = dict(os.environ, JAIC_STDLIB=str(snapshot/'stdlib'), TMPDIR=str(work),
               PYTHONDONTWRITEBYTECODE='1')
    fixture = snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-diagnostic-options.jai'

    def invoke(label, command, **details):
        result = subprocess.run([str(x) for x in command], cwd=snapshot, env=env,
                                capture_output=True, text=True, timeout=180)
        row = dict(label=label, command=[str(x) for x in command], exit=result.returncode,
                   stdout=result.stdout, stderr=result.stderr,
                   status='passed' if result.returncode == 0 else 'failed', **details)
        report['checks'].append(row)
        return row['status'] == 'passed'

    def dependencies(folder, sanitize=()):
        build(folder, selected=selected, source=prepared, sanitize=sanitize, cc=native_compiler)
        command = [native_compiler, '-std=c11', '-Wall', '-Wextra', '-Werror', '-dynamiclib']
        if sanitize:
            command += ['-fsanitize='+','.join(sanitize)]
        command += [snapshot/'stdlib/internal/bindings/callback-context/bridge.c',
                    '-o', folder/'libbinding_callback_context.dylib']
        if not invoke('native context dependency', command, sanitize=list(sanitize)):
            raise RuntimeError('Native context dependency failed')
        env.update(JAIC_NATIVE_LIBS=str(folder), LIBRARY_PATH=str(folder))

    dependencies(work/'native')
    invoke('public interpreter behavior', [tools['compiler'], fixture, '-interp'])
    for mode in ('O0', 'O2'):
        exe = work/('public-'+mode)
        if invoke('public native build', [tools['compiler'], fixture, '-'+mode, '-exe', exe], mode=mode):
            invoke('public native behavior', [exe], mode=mode)
    dependencies(work/'native-sanitized', ('address', 'undefined'))
    exe = work/'public-sanitized'
    if invoke('instrumented native build', [tools['compiler'], fixture, '-O0', '-sanitize',
                                          'address,undefined', '-exe', exe]):
        invoke('instrumented public and SDK behavior', [exe])
    for os_name in ('macos', 'linux', 'windows'):
        for cpu in ('x64', 'arm64'):
            invoke('required desktop source', [tools['compiler'], fixture, '-check', '-no_dce',
                                              '-os', os_name, '-cpu', cpu], target=os_name+'-'+cpu)
    for index, (parameters, diagnostic) in enumerate(CONFIGURATION_FAILURES):
        negative = work/('invalid-configuration-'+str(index)+'.jai')
        negative.write_text('M :: #import "rpmalloc"('+parameters+'); main :: () {}\n')
        negative.chmod(0o444)
        invoke('reviewed invalid configuration', [tools['compiler'], negative, '-check', '-no_dce'],
               expected_diagnostic=diagnostic)
        row = report['checks'][-1]
        row['status'] = 'passed' if row['exit'] != 0 and diagnostic in row['stderr'] else 'failed'
    invoke('strict complete-context lint', [tools['jailint'], '-D', 'warnings', '--config',
                                           snapshot/'jailint.toml', '--context', fixture,
                                           snapshot/'stdlib/rpmalloc',
                                           snapshot/'stdlib/internal/bindings/rpmalloc-checked', fixture])
    invoke('production format check', [tools['formatter'], '--check', snapshot/'stdlib/rpmalloc',
                                      snapshot/'stdlib/internal/bindings/rpmalloc-checked', fixture])
    native_unchanged = all(digest(work/name) == value for name, value in native_inputs.items())
    report['checks'].append(dict(label='immutable source and SDK inputs',
                                 status='passed' if unchanged(snapshot, identity) and native_unchanged else 'failed'))
    report['native_artifacts'] = {str(p.relative_to(work)): digest(p)
                                  for folder in ('native', 'native-sanitized')
                                  for p in (work/folder).iterdir() if p.is_file()}
    (work/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
    failures = sum(row['status'] != 'passed' for row in report['checks'])
    print(str(failures)+' failures; '+str(work/'verification.json'))
    return 1 if failures else 0


if __name__ == '__main__':
    raise SystemExit(main())
