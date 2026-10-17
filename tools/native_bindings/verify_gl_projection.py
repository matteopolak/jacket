#!/usr/bin/env python3
"""Verify reviewed live GL using projection without executing graphics calls."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from shared_snapshot import capture, unchanged


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'jailint', 'formatter', 'output_root'):
        parser.add_argument('--'+name.replace('_', '-'), type=Path, required=True)
    args = parser.parse_args()
    snapshot, identity = capture(args.output_root, ('stdlib/Bindings_Generator/tests/driven/sdk/gl-live-projection.jai',
        'tools/native_bindings', 'docs/specs/stdlib/modules/gl.md',
        'jaifmt.toml', 'jailint.toml'))
    work = Path(tempfile.mkdtemp(prefix='gl-projection-', dir=args.output_root))
    report = dict(snapshot=str(snapshot), input_identity=identity, checks=[],
        tools={str(p.resolve()): hashlib.sha256(p.resolve().read_bytes()).hexdigest() for p in
               (args.compiler, args.jailint, args.formatter)},
        scope='Mac ARM64 procedure-table behavior; actual SDK address only, no graphics calls/context')
    env = dict(os.environ, JAIC_STDLIB=str(snapshot/'stdlib'), TMPDIR=str(work), PYTHONDONTWRITEBYTECODE='1')

    def invoke(label, command):
        result = subprocess.run([str(x) for x in command], cwd=snapshot, env=env,
                                capture_output=True, text=True, timeout=120)
        report['checks'].append(dict(label=label, command=[str(x) for x in command], exit=result.returncode,
            stdout=result.stdout, stderr=result.stderr, status='passed' if result.returncode == 0 else 'failed'))
        (work/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(label, result.returncode, result.stderr[-1000:], flush=True)
        return result.returncode == 0

    fixture = snapshot/'stdlib/Bindings_Generator/tests/driven/sdk/gl-live-projection.jai'
    expanded = work/'all-extensions.jai'
    expanded.write_text(fixture.read_text().replace('#import "GL";', '#import "GL"(ENABLE_ALL_EXTENSIONS = true);'))
    expanded.chmod(0o444)
    expanded_hash = hashlib.sha256(expanded.read_bytes()).hexdigest()
    report['generated_input'] = dict(path=str(expanded), sha256=expanded_hash,
        derivation='Same independent test body; both imports select reviewed ENABLE_ALL_EXTENSIONS=true')
    for name, source in (('default', fixture), ('all-extensions', expanded)):
        invoke('live projection interpreter '+name, [args.compiler, source, '-interp'])
        for mode in ('O0', 'O2'):
            exe = work/(name+'-'+mode)
            if invoke('native projection build '+name+' '+mode, [args.compiler, source, '-'+mode, '-exe', exe]):
                invoke('live native storage '+name+' '+mode, [exe])
        for os_name in ('macos', 'linux', 'windows'):
            for cpu in ('x64', 'arm64'):
                invoke('required projection source '+name+' '+os_name+'-'+cpu,
                    [args.compiler, source, '-check', '-no_dce', '-os', os_name, '-cpu', cpu])
        invoke('strict complete projection context '+name, [args.jailint, '-D', 'warnings', '--config', snapshot/'jailint.toml',
            '--context', source, snapshot/'stdlib/GL', source])
    invoke('production formatting', [args.formatter, '--check', snapshot/'stdlib/GL', fixture])
    report['privacy_inputs'] = []
    for module, member in (('GL', 'Native'), ('GL', 'gl_lib'), ('rpmalloc', 'HeapBridge'),
                           ('rpmalloc', 'Callbacks'), ('rpmalloc', 'configure_policy'), ('rpmalloc', 'configure_dimensions')):
        negative = work/('private-'+module+'-'+member+'.jai')
        negative.write_text('M :: #import "'+module+'"; main :: () { value := M.'+member+'; }\n')
        negative.chmod(0o444)
        report['privacy_inputs'].append(dict(path=str(negative), sha256=hashlib.sha256(negative.read_bytes()).hexdigest()))
        invoke('required public privacy '+module+'.'+member, [args.compiler, negative, '-check', '-no_dce'])
        row = report['checks'][-1]
        exact = 'module `'+module+'` has no exported member `'+member+'`'
        row['status'] = 'passed' if row['exit'] != 0 and exact in row['stderr'] else 'failed'
    report['checks'].append(dict(label='complete immutable inputs', status='passed' if unchanged(snapshot, identity)
        and hashlib.sha256(expanded.read_bytes()).hexdigest() == expanded_hash else 'failed'))
    (work/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
    failures = sum(row['status'] != 'passed' for row in report['checks'])
    print(f'{failures} failures; {work / "verification.json"}')
    return 1 if failures else 0


if __name__ == '__main__':
    raise SystemExit(main())
