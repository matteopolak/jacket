#!/usr/bin/env python3
"""Reviewed Objective_C contracts plus independent SDK ABI/failure controls."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import tempfile

from shared_snapshot import capture, unchanged

POSITIVE = ('objc-reviewed-encodings', 'objc-reviewed-registration', 'objc-runtime', 'objective-c')
NEGATIVE = {
    'objc-context-method': 'Objective-C method implementation requires C calling convention',
    'objc-malformed-isa': 'Objective-C isa member must have exact Class type',
    'objc-selector-type': 'Objective-C method requires exact Selector argument',
    'objc-receiver-type': 'Objective-C method receiver has no supported object representation',
    'objc-any-encoding': 'reflected type has no supported Objective-C encoding',
    'objc-duplicate-method': 'Objective-C runtime rejected method installation',
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'jailint', 'formatter', 'output_root'):
        parser.add_argument('--'+name.replace('_', '-'), type=Path, required=True)
    args = parser.parse_args()
    tools = {name: getattr(args, name).resolve(strict=True) for name in ('compiler', 'jailint', 'formatter')}
    extras = ['tools/native_bindings', 'docs/specs/stdlib/modules/objective-c.md',
              'stdlib/Bindings_Generator/tests/driven/sdk/objc_runtime_probe.m', 'jailint.toml', 'jaifmt.toml']
    extras += ['stdlib/Bindings_Generator/tests/driven/sdk/'+name+'.jai' for name in POSITIVE]
    extras += ['stdlib/Bindings_Generator/tests/driven/sdk/negative/'+name+'.jai' for name in NEGATIVE]
    snapshot, identity = capture(args.output_root, extras)
    work = Path(tempfile.mkdtemp(prefix='objc-contracts-', dir=args.output_root))
    env = dict(os.environ, JAIC_STDLIB=str(snapshot/'stdlib'), JAIC_NATIVE_LIBS=str(work),
               LIBRARY_PATH=str(work), TMPDIR=str(work), PYTHONDONTWRITEBYTECODE='1')
    report = dict(scope='Current reviewed Objective_C; Mac ARM64 native and Mac x64 source only',
                  snapshot=str(snapshot), work=str(work), input_identity=identity,
                  tools={str(path): digest(path) for path in tools.values()}, checks=[])

    def invoke(label, command, **details):
        result = subprocess.run([str(x) for x in command], cwd=snapshot, env=env,
                                text=True, capture_output=True, timeout=180,
                                preexec_fn=lambda: resource.setrlimit(resource.RLIMIT_CORE, (0, 0)))
        row = dict(label=label, command=[str(x) for x in command], exit=result.returncode,
                   stdout=result.stdout, stderr=result.stderr,
                   status='passed' if result.returncode == 0 else 'failed', **details)
        report['checks'].append(row)
        return row

    sdk = Path(subprocess.check_output(['xcrun', '--show-sdk-path'], text=True).strip())
    headers = ('usr/include/objc/objc.h', 'usr/include/objc/runtime.h',
               'System/Library/Frameworks/Foundation.framework/Headers/NSObjCRuntime.h',
               'System/Library/Frameworks/Foundation.framework/Headers/NSMethodSignature.h')
    sdk_hashes = {str(sdk/name): digest(sdk/name) for name in headers}
    report['official_sdk'] = dict(path=str(sdk.resolve()),
                                  version=subprocess.check_output(['xcrun', '--show-sdk-version'], text=True).strip(),
                                  headers=sdk_hashes)
    invoke('independent SDK probe build', ['cc', '-dynamiclib', '-Wall', '-Wextra', '-Werror',
                                         '-framework', 'Foundation',
                                         snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/objc_runtime_probe.m',
                                         '-o', work/'libobjc_runtime_probe.dylib'])
    for name in POSITIVE:
        fixture = snapshot/('stdlib/Bindings_Generator/tests/driven/sdk/'+name+'.jai')
        invoke('public interpreter', [tools['compiler'], fixture, '-interp'], fixture=name)
        for mode in ('O0', 'O2'):
            exe = work/(name+'-'+mode)
            row = invoke('public native build', [tools['compiler'], fixture, '-'+mode, '-exe', exe],
                         fixture=name, mode=mode)
            if row['status'] == 'passed':
                invoke('public native execution', [exe], fixture=name, mode=mode)
        for cpu in ('x64', 'arm64'):
            invoke('public source', [tools['compiler'], fixture, '-check', '-no_dce', '-os', 'macos', '-cpu', cpu],
                   fixture=name, target='macos-'+cpu)
    for name, expected in NEGATIVE.items():
        fixture = snapshot/('stdlib/Bindings_Generator/tests/driven/sdk/negative/'+name+'.jai')
        exe = work/(name+'-negative')
        row = invoke('negative native build', [tools['compiler'], fixture, '-O0', '-exe', exe], fixture=name)
        if row['status'] == 'passed':
            row = invoke('active native assertion', [exe], fixture=name, expected_diagnostic=expected)
            row['status'] = 'passed' if row['exit'] != 0 and expected in row['stderr'] else 'failed'
    for member in ('String_Builder', 'ObjCEncoding', 'ObjCRegistration', 'ObjCStub', 'NSWindow'):
        fixture = work/('private-'+member+'.jai')
        fixture.write_text('O :: #import "Objective_C"; leaked :: O.'+member+'; main :: () {}\n')
        row = invoke('public privacy', [tools['compiler'], fixture, '-check'], member=member)
        expected = 'has no exported member \u0060'+member+'\u0060'
        row['status'] = 'passed' if row['exit'] != 0 and expected in row['stderr'] else 'failed'
    roots = [snapshot/'stdlib/Objective_C/wrappers.jai', snapshot/'stdlib/internal/bindings/objc-encoding',
             snapshot/'stdlib/internal/bindings/objc-registration', snapshot/'stdlib/internal/bindings/objc-stub']
    for name in POSITIVE:
        fixture = snapshot/('stdlib/Bindings_Generator/tests/driven/sdk/'+name+'.jai')
        invoke('strict complete-context lint', [tools['jailint'], '-D', 'warnings', '--config', snapshot/'jailint.toml',
                                               '--context', fixture, *roots, fixture], fixture=name)
    invoke('production formatter', [tools['formatter'], '--check', *roots,
                                    *[snapshot/('stdlib/Bindings_Generator/tests/driven/sdk/'+name+'.jai') for name in POSITIVE],
                                    *[snapshot/('stdlib/Bindings_Generator/tests/driven/sdk/negative/'+name+'.jai') for name in NEGATIVE]])
    stable = unchanged(snapshot, identity) and all(digest(Path(name)) == value for name, value in sdk_hashes.items())
    report['checks'].append(dict(label='immutable inputs and official SDK', status='passed' if stable else 'failed'))
    (work/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
    failures = sum(row['status'] != 'passed' for row in report['checks'])
    print(str(failures)+' failures; '+str(work/'verification.json'))
    return 1 if failures else 0


if __name__ == '__main__':
    raise SystemExit(main())
