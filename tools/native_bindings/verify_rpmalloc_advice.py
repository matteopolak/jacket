#!/usr/bin/env python3
"""Observe real retained-span advice on an immutable Mac SDK input capture."""
import argparse
import hashlib
import importlib
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import tempfile

from shared_snapshot import capture, unchanged


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'native_compiler', 'jailint', 'formatter', 'vendor', 'output_root'):
        parser.add_argument('--'+name.replace('_', '-'), type=Path, required=True)
    args = parser.parse_args()
    snapshot, identity = capture(args.output_root, ('tools/native_bindings',
        'tests/native-bindings/rpmalloc-retained-advice.jai',
        'tests/native-bindings/rpmalloc_advice_probe.c', 'docs/specs/stdlib/modules/rpmalloc.md',
        'jaifmt.toml', 'jailint.toml'))
    work = Path(tempfile.mkdtemp(prefix='advice-', dir=args.output_root))
    sys.path.insert(0, str(snapshot/'tools/native_bindings'))
    prepare = importlib.import_module('prepare_rpmalloc').prepare
    build = importlib.import_module('build_rpmalloc').build
    profiles = importlib.import_module('rpmalloc_profiles').PROFILES
    profile = next(p for p in profiles if not p['raw'] and p['statistics'] and not p['adaptive'] and
                   not p['global_cache'] and p['thread_cache'] and p['assertions'] and p['validation'])
    prepared_source = prepare(args.vendor, work/'prepared')
    build(work/'native', selected={profile['sdk'], profile['callbacks']}, source=prepared_source, cc=args.native_compiler)
    env = dict(os.environ, JAIC_STDLIB=str(snapshot/'stdlib'), JAIC_NATIVE_LIBS=str(work/'native'),
               LIBRARY_PATH=str(work/'native'), TMPDIR=str(work), PYTHONDONTWRITEBYTECODE='1')
    checks = []

    def invoke(label, command):
        result = subprocess.run([str(x) for x in command], cwd=snapshot, env=env,
                                capture_output=True, text=True, timeout=120,
                                preexec_fn=lambda: resource.setrlimit(resource.RLIMIT_CORE, (0, 0)))
        checks.append(dict(label=label, command=[str(x) for x in command], exit=result.returncode,
                           stdout=result.stdout, stderr=result.stderr))
        (work/'commands.json').write_text(json.dumps(checks, indent=2)+'\n')
        return result.returncode == 0

    for helper_source, name in ((snapshot/'stdlib/internal/bindings/callback-context/bridge.c', 'binding_callback_context'),
                         (snapshot/'tests/native-bindings/rpmalloc_advice_probe.c', 'rpmalloc_advice_probe')):
        invoke('independent native helper '+name, [args.native_compiler, '-dynamiclib', '-Wall', '-Wextra', '-Werror',
                                                  helper_source, '-o', work/('native/lib'+name+'.dylib')])
    for mode in ('O0', 'O2'):
        exe = work/('advice-'+mode)
        if invoke('native build '+mode, [args.compiler, snapshot/'tests/native-bindings/rpmalloc-retained-advice.jai',
                                        '-'+mode, '-exe', exe]):
            invoke('actual kernel advice '+mode, [exe])
    fixture = snapshot/'tests/native-bindings/rpmalloc-retained-advice.jai'
    for os_name in ('macos', 'linux', 'windows'):
        for cpu in ('x64', 'arm64'):
            invoke('public advice source '+os_name+'-'+cpu,
                   [args.compiler, fixture, '-check', '-no_dce', '-os', os_name, '-cpu', cpu])
    invoke('strict context lint', [args.jailint, '-D', 'warnings', '--config', snapshot/'jailint.toml',
                                  '--context', fixture, snapshot/'stdlib/rpmalloc', fixture])
    invoke('production formatter', [args.formatter, '--check', snapshot/'stdlib/rpmalloc', fixture])
    report = dict(snapshot=str(snapshot), input_identity=identity, checks=checks,
                  immutable=unchanged(snapshot, identity), scope='Native Mac ARM64 syscall behavior only')
    report['tools'] = {str(path.resolve()): hashlib.sha256(path.resolve().read_bytes()).hexdigest()
                       for path in (args.compiler, args.native_compiler, args.jailint, args.formatter)}
    report['native_sdk_provenance'] = json.loads((prepared_source/'provenance.json').read_text())
    report['native_artifacts'] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                                 for p in (work/'native').iterdir() if p.is_file()}
    (work/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
    for row in checks:
        print(row['label'], row['exit'], row['stderr'][-1200:])
    print(work/'verification.json')
    return 1 if not report['immutable'] or any(row['exit'] for row in checks) else 0


if __name__ == '__main__':
    raise SystemExit(main())
