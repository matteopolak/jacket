#!/usr/bin/env python3
"""Retain both-cache-off public behavior, concurrency and canonical ABI controls."""
import argparse
import hashlib
import importlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from shared_snapshot import capture, unchanged


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'native_compiler', 'jailint', 'formatter', 'vendor', 'output_root'):
        parser.add_argument('--'+name.replace('_', '-'), type=Path, required=True)
    parser.add_argument('--adaptive', action='store_true', help='Verify required adaptive statistics-off native storage')
    args = parser.parse_args()
    snapshot, identity = capture(args.output_root, ('tools/native_bindings',
        'tests/native-bindings/generated', 'tests/native-bindings/rpmalloc-no-thread-cache.jai',
        'tests/native-bindings/rpmalloc-cache-exercise.jai', 'tests/native-bindings/rpmalloc_callback_probe.c',
        'tests/native-bindings/rpmalloc-adaptive-cache.jai', 'tests/native-bindings/rpmalloc_adaptive_probe.c',
        'docs/specs/stdlib/modules/rpmalloc.md', 'jailint.toml', 'jaifmt.toml'))
    work = Path(tempfile.mkdtemp(prefix='no-thread-', dir=args.output_root))
    sys.path.insert(0, str(snapshot/'tools/native_bindings'))
    prepare = importlib.import_module('prepare_rpmalloc').prepare
    build = importlib.import_module('build_rpmalloc').build
    profiles = importlib.import_module('rpmalloc_profiles').PROFILES
    prepared = prepare(args.vendor, work/'prepared')
    selected_profiles = [p for p in profiles if not p['global_cache'] and not p['thread_cache'] and not p['adaptive'] and
        p['assertions'] and p['validation'] and (not p['raw'] or not p['public_heaps'])]
    if args.adaptive:
        selected_profiles = [p for p in profiles if p['global_cache'] and p['thread_cache'] and
            p['assertions'] and p['validation'] and
            ((not p['statistics'] and p['adaptive'] and (not p['raw'] or not p['public_heaps'])) or
             (p['statistics'] and not p['adaptive'] and not p['raw']))]
    selected = {p[kind] for p in selected_profiles for kind in ('sdk', 'callbacks')}
    report = dict(snapshot=str(snapshot), input_identity=identity, checks=[],
                  native_profiles=selected_profiles, scope='Required cache fixture; Mac ARM64 native only',
                  tools={str(p.resolve()): hashlib.sha256(p.resolve().read_bytes()).hexdigest() for p in
                         (args.compiler, args.native_compiler, args.jailint, args.formatter)})
    env = dict(os.environ, JAIC_STDLIB=str(snapshot/'stdlib'), TMPDIR=str(work), PYTHONDONTWRITEBYTECODE='1')
    fixture = snapshot/('tests/native-bindings/rpmalloc-adaptive-cache.jai' if args.adaptive else
                        'tests/native-bindings/rpmalloc-no-thread-cache.jai')

    def invoke(label, command):
        result = subprocess.run([str(x) for x in command], cwd=snapshot, env=env,
                                capture_output=True, text=True, timeout=180)
        report['checks'].append(dict(label=label, command=[str(x) for x in command], exit=result.returncode,
            stdout=result.stdout, stderr=result.stderr, status='passed' if result.returncode == 0 else 'failed'))
        (work/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(label, result.returncode, result.stderr[-1200:], flush=True)
        return result.returncode == 0

    def dependencies(folder, sanitize=()):
        build(folder, selected=selected, source=prepared, sanitize=sanitize, cc=args.native_compiler)
        env.update(JAIC_NATIVE_LIBS=str(folder), LIBRARY_PATH=str(folder))
        for source, name in ((snapshot/'stdlib/internal/bindings/callback-context/bridge.c', 'binding_callback_context'),
                             (snapshot/'tests/native-bindings/rpmalloc_callback_probe.c', 'rpmalloc_callback_probe')):
            invoke('real native dependency '+name, [args.native_compiler, '-dynamiclib', '-pthread',
                '-Wall', '-Wextra', '-Werror', *(['-fsanitize='+','.join(sanitize)] if sanitize else []),
                source, '-o', folder/('lib'+name+'.dylib')])
        if args.adaptive:
            functions = json.loads((snapshot/'stdlib/internal/bindings/rpmalloc-sdk/provenance.json').read_text())['functions']
            profile = next(p for p in selected_profiles if p['adaptive'] and not p['raw'])
            flags = ['-D'+key+'='+str(value) for key, value in profile['options'].items()]
            flags += ['-D'+f['name']+'=binding_adaptive_probe_'+f['name'] for f in functions]
            if sanitize:
                flags += ['-fsanitize='+','.join(sanitize)]
            obj = folder/'adaptive-probe.o'
            invoke('independent adaptive SDK types', [args.native_compiler, '-std=c11', '-O2', '-fPIC',
                '-Wall', '-Wextra', '-Werror', '-isystem', prepared, *flags, '-c',
                snapshot/'tests/native-bindings/rpmalloc_adaptive_probe.c', '-o', obj])
            invoke('native adaptive SDK introspection archive', ['ar', 'rcs', folder/'librpmalloc_adaptive_probe.a', obj])
            invoke('interpreter adaptive SDK introspection library', [args.native_compiler, '-dynamiclib',
                *(['-fsanitize='+','.join(sanitize)] if sanitize else []), obj, '-o', folder/'librpmalloc_adaptive_probe.dylib'])

    dependencies(work/'native')
    invoke('public interpreter', [args.compiler, fixture, '-interp'])
    for mode in ('O0', 'O2'):
        exe = work/('public-'+mode)
        if invoke('public native build '+mode, [args.compiler, fixture, '-'+mode, '-exe', exe]):
            invoke('public native behavior and ABI '+mode, [exe])
    dependencies(work/'native-sanitized', ('address', 'undefined'))
    exe = work/'public-sanitized'
    if invoke('instrumented public build', [args.compiler, fixture, '-O0', '-sanitize', 'address,undefined', '-exe', exe]):
        invoke('instrumented public SDK/concurrency/ABI', [exe])
    for os_name in ('macos', 'linux', 'windows'):
        for cpu in ('x64', 'arm64'):
            invoke('required source '+os_name+'-'+cpu,
                [args.compiler, fixture, '-check', '-no_dce', '-os', os_name, '-cpu', cpu])
    invoke('strict complete-context lint', [args.jailint, '-D', 'warnings', '--config', snapshot/'jailint.toml',
        '--context', fixture, snapshot/'stdlib/rpmalloc', fixture])
    invoke('production formatting', [args.formatter, '--check', snapshot/'stdlib/rpmalloc', fixture])
    report['checks'].append(dict(label='complete input immutability', status='passed' if unchanged(snapshot, identity) else 'failed'))
    report['native_artifacts'] = {str(p.relative_to(work)): hashlib.sha256(p.read_bytes()).hexdigest()
        for folder in ('native', 'native-sanitized') for p in (work/folder).iterdir() if p.is_file()}
    report['native_sdk_provenance'] = json.loads((prepared/'provenance.json').read_text())
    (work/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
    failures = sum(row['status'] != 'passed' for row in report['checks'])
    print(f'{failures} failures; {work / "verification.json"}')
    return 1 if failures else 0


if __name__ == '__main__':
    raise SystemExit(main())
