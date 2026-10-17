#!/usr/bin/env python3
"""Required numeric policies, transactional storage, real SDK and public checks."""
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


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'native_compiler', 'jailint', 'formatter', 'vendor', 'output_root'):
        parser.add_argument('--'+name.replace('_', '-'), type=Path, required=True)
    parser.add_argument('--retention', action='store_true', help='Also verify required retention/page policies and retained callback controls')
    args = parser.parse_args()
    snapshot, identity = capture(args.output_root, ('tools/native_bindings',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-runtime-dimensions.jai',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_runtime_dimensions_probe.c',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-retained-advice.jai',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_advice_probe.c',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-retention-options.jai',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-effective-config.jai',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_retention_probe.c',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-public-configurable.jai',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-public-callbacks.jai',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_callback_probe.c',
        'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_sdk_config_probe.c',
        'docs/specs/stdlib/modules/rpmalloc.md', 'jaifmt.toml', 'jailint.toml'))
    work = Path(tempfile.mkdtemp(prefix='dimensions-', dir=args.output_root))
    sys.path.insert(0, str(snapshot/'tools/native_bindings'))
    prepare = importlib.import_module('prepare_rpmalloc').prepare
    builder = importlib.import_module('build_rpmalloc')
    profiles = importlib.import_module('rpmalloc_profiles')
    prepared = prepare(args.vendor, work/'prepared')
    reproduced = prepare(args.vendor, work/'reproduced')
    prepared_hashes = {p.name: digest(p) for p in prepared.iterdir() if p.is_file()}
    report = dict(snapshot=str(snapshot), input_identity=identity, checks=[],
                  tools={str(p.resolve()): digest(p.resolve()) for p in
                         (args.compiler, args.native_compiler, args.jailint, args.formatter)},
                  native_sdk_provenance=json.loads((prepared/'provenance.json').read_text()),
                  scope='Mac ARM64 native; six desktop source targets, no other native claim')
    report['checks'].append(dict(label='independently reproduced SDK preparation',
        status='passed' if prepared_hashes == {p.name: digest(p) for p in reproduced.iterdir() if p.is_file()} else 'failed'))
    env = dict(os.environ, JAIC_STDLIB=str(snapshot/'stdlib'), TMPDIR=str(work),
               PYTHONDONTWRITEBYTECODE='1', ASAN_OPTIONS='allocator_may_return_null=1')
    fixture = snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-runtime-dimensions.jai'
    api = json.loads((snapshot/'stdlib/internal/bindings/rpmalloc-sdk/provenance.json').read_text())
    selected_profiles = [p for p in profiles.PROFILES if not p['raw'] and p['statistics'] and not p['adaptive'] and
                         p['thread_cache'] and p['assertions'] and p['validation']]
    selected = {p[kind] for p in selected_profiles for kind in ('sdk', 'callbacks')}

    def invoke(label, command):
        result = subprocess.run([str(x) for x in command], cwd=snapshot, env=env,
                                capture_output=True, text=True, timeout=180)
        row = dict(label=label, command=[str(x) for x in command], exit=result.returncode,
                   stdout=result.stdout, stderr=result.stderr,
                   status='passed' if result.returncode == 0 else 'failed')
        report['checks'].append(row)
        (work/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(label, result.returncode, result.stderr[-900:], flush=True)
        return result.returncode == 0

    def dependencies(folder, sanitize=()):
        builder.build(folder, selected=selected, source=prepared, sanitize=sanitize, cc=args.native_compiler)
        flags = ['-fsanitize='+','.join(sanitize)] if sanitize else []
        env.update(JAIC_NATIVE_LIBS=str(folder), LIBRARY_PATH=str(folder))
        for source, name in ((snapshot/'stdlib/internal/bindings/callback-context/bridge.c', 'binding_callback_context'),
                             (snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_advice_probe.c', 'rpmalloc_advice_probe')):
            invoke('native dependency '+name, [args.native_compiler, '-dynamiclib', '-Wall', '-Wextra', '-Werror',
                *flags, source, '-o', folder/('lib'+name+'.dylib')])
        if args.retention:
            objects = []
            for name in ('rpmalloc_callback_probe', 'rpmalloc_sdk_config_probe'):
                obj = folder/(name+'.o')
                objects.append(obj)
                invoke('native SDK introspection object '+name, [args.native_compiler, '-std=c11', '-fPIC', '-pthread',
                    '-Wall', '-Wextra', '-Werror', '-isystem', prepared, *flags,
                    '-c', snapshot/('stdlib/Bindings_Generator/tests/driven/sdk/'+name+'.c'), '-o', obj])
            # Native programs link one SDK and its introspection object into
            # the same image; a helper dylib must not introduce a second SDK.
            invoke('native SDK introspection archive', ['ar', 'rcs', folder/'librpmalloc_callback_probe.a', *objects])
            invoke('real paired callback SDK probe', [args.native_compiler, '-dynamiclib', '-pthread',
                '-Wall', '-Wextra', '-Werror', '-isystem', prepared, *flags,
                snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_callback_probe.c',
                snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_sdk_config_probe.c', '-L', folder,
                '-lbinding_rpmalloc_sdk', '-Wl,-rpath,@loader_path', '-o', folder/'librpmalloc_callback_probe.dylib'])
        for profile in selected_profiles:
            probe = folder/('sdk-storage-'+profile['sdk'])
            # Preserve the deliberately failing real allocator call even if
            # LLVM infers its unused result. calloc-assert-control independently
            # reproduces the optimization without any SDK/adapter source.
            command = [args.native_compiler, '-std=c11', '-O2', '-fno-builtin-calloc', '-Wall', '-Wextra', '-Werror',
                '-isystem', prepared, '-I', snapshot/'stdlib/internal/bindings/rpmalloc-heaps',
                *profiles.c_definitions(profile, api['functions']), *flags,
                snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_runtime_dimensions_probe.c', '-o', probe]
            if invoke('independent SDK storage build '+profile['sdk'], command):
                invoke('independent SDK storage and failure rollback '+profile['sdk'], [probe])
            if args.retention and profile['global_cache']:
                command[-3] = snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc_retention_probe.c'
                command[-1] = folder/'sdk-retention'
                if invoke('SDK actual OS release probe build', command):
                    invoke('SDK actual OS release and retention', [command[-1]])

    dependencies(work/'native')
    invoke('public interpreter', [args.compiler, fixture, '-interp'])
    names = ['rpmalloc-runtime-dimensions', 'rpmalloc-retained-advice']
    if args.retention:
        names += ['rpmalloc-retention-options', 'rpmalloc-effective-config',
                  'rpmalloc-public-configurable', 'rpmalloc-public-callbacks']
    for name in names:
        for mode in ('O0', 'O2'):
            exe = work/(name+'-'+mode)
            if invoke('public native build '+name+' '+mode,
                [args.compiler, snapshot/('stdlib/Bindings_Generator/tests/driven/sdk/'+name+'.jai'), '-'+mode, '-exe', exe]):
                invoke('public native behavior '+name+' '+mode, [exe])
    dependencies(work/'native-sanitized', ('address', 'undefined'))
    exe = work/'public-sanitized'
    if invoke('instrumented public build', [args.compiler, fixture, '-O0', '-sanitize', 'address,undefined', '-exe', exe]):
        invoke('instrumented public and SDK behavior', [exe])
    if args.retention:
        exe = work/'retention-sanitized'
        if invoke('instrumented retention build', [args.compiler,
            snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-retention-options.jai', '-O0', '-sanitize', 'address,undefined', '-exe', exe]):
            invoke('instrumented retention behavior', [exe])
    for os_name in ('macos', 'linux', 'windows'):
        for cpu in ('x64', 'arm64'):
            invoke('required numeric source '+os_name+'-'+cpu,
                [args.compiler, fixture, '-check', '-no_dce', '-os', os_name, '-cpu', cpu])
            if args.retention:
                invoke('required retention source '+os_name+'-'+cpu,
                    [args.compiler, snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-retention-options.jai',
                     '-check', '-no_dce', '-os', os_name, '-cpu', cpu])
    invoke('strict complete context lint', [args.jailint, '-D', 'warnings', '--config', snapshot/'jailint.toml',
        '--context', fixture, snapshot/'stdlib/rpmalloc', fixture])
    invoke('production formatting', [args.formatter, '--check', snapshot/'stdlib/rpmalloc', fixture])
    if args.retention:
        retention = snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-retention-options.jai'
        invoke('strict retention context lint', [args.jailint, '-D', 'warnings', '--config', snapshot/'jailint.toml',
            '--context', retention, snapshot/'stdlib/rpmalloc', retention])
        invoke('production retention formatting', [args.formatter, '--check', retention,
            snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/rpmalloc-effective-config.jai'])
    report['checks'].append(dict(label='immutable source and SDK inputs', status='passed' if
        unchanged(snapshot, identity) and prepared_hashes == {p.name: digest(p) for p in prepared.iterdir() if p.is_file()}
        else 'failed'))
    report['native_artifacts'] = {str(p.relative_to(work)): digest(p)
        for folder in ('native', 'native-sanitized') for p in (work/folder).iterdir() if p.is_file()}
    (work/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
    failures = sum(row['status'] != 'passed' for row in report['checks'])
    print(f'{failures} failures; {work / "verification.json"}')
    return 1 if failures else 0


if __name__ == '__main__':
    raise SystemExit(main())
