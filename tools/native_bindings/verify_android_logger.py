#!/usr/bin/env python3
"""Check the logger against the actual shared Runtime, without source staging.

Host output controls are independent of Android SDK/device execution. Required
Android target failures remain failures. Native runs require the integrator's
serialized native slot; omit --native for source/interpreter/lint checks only.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
EXPECTED_OUT = '4:Jai:payload\nsink-complete\nfresh-out\n6:tag:second\nsink-complete\n'
EXPECTED_ERR = 'fresh-error\n'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_identity():
    paths = [p for p in (ROOT / 'stdlib').rglob('*.jai') if 'Extensions' not in p.parts]
    paths += [ROOT / 'tests/native-bindings/android-logger-output.jai',
              ROOT / 'tests/native-bindings/android-helpers.jai', Path(__file__).resolve()]
    return {str(p.relative_to(ROOT)): digest(p) for p in sorted(paths)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--jailint', type=Path, required=True)
    parser.add_argument('--formatter', type=Path, required=True)
    parser.add_argument('--output-root', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--native', action='store_true')
    args = parser.parse_args()
    output = args.output_root.resolve()
    report_path = args.report.resolve()
    for destination in (output, report_path):
        if destination == ROOT or ROOT in destination.parents:
            parser.error('outputs and reports must be worker-private and outside shared source')
    output.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='android-logger-', dir=output))
    compiler = args.compiler.resolve(strict=True)
    lint = args.jailint.resolve(strict=True)
    formatter = args.formatter.resolve(strict=True)
    source = ROOT / 'stdlib/Android/tests/logger-output.jai'
    before = source_identity()
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / 'stdlib'), TMPDIR=str(work))
    report = dict(scope='Actual shared Runtime/logger host controls; not Android SDK/device execution',
                  work=str(work), native_requested=args.native, checks=[],
                  tools={str(p): digest(p) for p in (compiler, lint, formatter)},
                  sources_before=before)

    def invoke(phase, command, **details):
        command = [str(x) for x in command]
        try:
            result = subprocess.run(command, cwd=ROOT, env=env, text=True,
                                    capture_output=True, timeout=180)
            row = dict(command=command, exit=result.returncode,
                       stdout=result.stdout, stderr=result.stderr)
        except subprocess.TimeoutExpired as error:
            row = dict(command=command, exit=None, stdout=str(error.stdout or ''),
                       stderr=str(error.stderr or ''), timeout=True)
        row.update(phase=phase, status='passed' if row['exit'] == 0 else 'failed', **details)
        report['checks'].append(row)
        return row

    for target_os in ('macos', 'linux'):
        for cpu in ('x64', 'arm64'):
            invoke('host-core-source', [compiler, source, '-check', '-no_dce',
                                       '-os', target_os, '-cpu', cpu], target=target_os + '-' + cpu)

    fixtures = [('logger-helpers', ROOT / 'tests/native-bindings/android-helpers.jai')]
    for module in ('Android/AAudio', 'Android/EGL', 'POSIX'):
        fixture = work / (module.replace('/', '-') + '.jai')
        fixture.write_text('M :: #import "' + module + '"; main :: () {}\n')
        fixtures.append((module, fixture))
    for label, fixture in fixtures:
        for cpu in ('x64', 'arm64'):
            row = invoke('required-Android-source', [compiler, fixture, '-check', '-no_dce',
                                                    '-os', 'android', '-cpu', cpu],
                         module=label, target='android-' + cpu)
            if row['exit'] != 0:
                row['status'] = 'required_target_unverified'

    modes = ['interpreter']
    if args.native:
        modes += ['native-O0', 'native-O2', 'native-sanitized']
    for mode in modes:
        if mode == 'interpreter':
            command = [compiler, source, '-interp']
        else:
            exe = work / mode
            options = ['-O2'] if mode == 'native-O2' else ['-O0']
            if mode == 'native-sanitized':
                options += ['-sanitize', 'address,undefined']
            build = invoke('host-build', [compiler, source, *options, '-exe', exe], mode=mode)
            if build['exit'] != 0:
                continue
            command = [exe]
        row = invoke('host-observable-output', command, mode=mode,
                     expected_stdout=EXPECTED_OUT, expected_stderr=EXPECTED_ERR)
        if row['stdout'] != EXPECTED_OUT or row['stderr'] != EXPECTED_ERR:
            row['status'] = 'failed'

    invoke('strict-context-lint', [lint, '-D', 'warnings', '--config', ROOT / 'jailint.toml',
                                  '--context', ROOT / 'tests/native-bindings/android-logger-output.jai',
                                  ROOT / 'stdlib/internal/bindings/android-log',
                                  source, ROOT / 'tests/native-bindings/android-logger-output.jai'])
    invoke('format-check', [formatter, '--check', source,
                           ROOT / 'stdlib/Android/wrappers.jai',
                           ROOT / 'stdlib/internal/bindings/android-log/module.jai',
                           ROOT / 'tests/native-bindings/android-logger-output.jai'])
    after = source_identity()
    changed = sorted(p for p in before.keys() | after.keys() if before.get(p) != after.get(p))
    report['checks'].append(dict(phase='shared-source-immutability', changed=changed,
                                 status='failed' if changed else 'passed'))
    report['sources_after'] = after
    report['native_remaining'] = [] if args.native else ['O0', 'O2', 'address/undefined sanitizers']
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + '\n')
    failures = sum(row['status'] != 'passed' for row in report['checks'])
    print(str(failures) + ' failed/unverified controls; ' + str(args.report))
    return 1 if failures else 0


if __name__ == '__main__':
    raise SystemExit(main())
