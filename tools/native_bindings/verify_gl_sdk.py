#!/usr/bin/env python3
"""Real macOS GL SDK checks with an explicit own source-built compiler.

Uses an explicitly selected production library without source staging.
An optional proposal path labels a separately built trial compiler honestly.
Unavailable debug-driver support remains a failing verification requirement.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import sys

from verify import ROOT, invoke


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--stdlib', type=Path, required=True)
    parser.add_argument('--compiler-proposal', type=Path)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != 'Darwin':
        raise SystemExit('The CGL runtime test requires the public macOS SDK and runtime')
    compiler = args.compiler.resolve(strict=True)
    if not compiler.is_relative_to(ROOT/'target'):
        parser.error('use a compiler source-built under this checkout target directory')
    stdlib = args.stdlib.resolve(strict=True)
    args.report = args.report.resolve()
    args.report.parent.mkdir(parents=True, exist_ok=True)
    work = args.report.parent/(args.report.stem+'-'+str(os.getpid()))
    work.mkdir(parents=True, exist_ok=True)
    native = work/'native'
    native.mkdir()
    temporary = work/'tmp'
    temporary.mkdir()
    env = dict(os.environ, JAIC_STDLIB=str(stdlib),
               JAIC_NATIVE_LIBS=str(native), TMPDIR=str(temporary))
    report = {'scope':'Production SDK verification against explicit integrated sources',
              'compiler':str(compiler), 'compiler_sha256':digest(compiler),
              'compiler_proposal':None, 'stdlib':str(stdlib), 'work':str(work), 'staging_adjustments':[],
              'checks':[], 'staged_sources':{}, 'test_sources':{}}
    if args.compiler_proposal:
        proposal = args.compiler_proposal.resolve(strict=True)
        report['compiler_proposal'] = {'path':str(proposal), 'sha256':digest(proposal)}
        report['scope'] += '; explicitly labeled trial compiler proposal remains unintegrated'
    report['staged_sources'] = {str(p.relative_to(stdlib)):digest(p)
                                for p in sorted(stdlib.rglob('*.jai')) if 'Extensions' not in p.parts}
    sdk_result = invoke(['xcrun','--show-sdk-path'],env)
    if sdk_result['exit'] != 0:
        raise SystemExit(sdk_result['stderr'])
    sdk = Path(sdk_result['stdout'].strip())
    pins = json.loads((ROOT/'docs/native/apple-sdk-inputs.json').read_text())['files']
    report['sdk_headers'] = {}
    for name in ('OpenGL.h','CGLTypes.h','CGLCurrent.h'):
        relative = 'System/Library/Frameworks/OpenGL.framework/Headers/'+name
        actual = digest(sdk/relative)
        if actual != pins[relative]:
            raise SystemExit('Native SDK header differs from reviewed pin: '+relative)
        report['sdk_headers'][relative] = actual
    probe_source = ROOT/'tests/native-bindings/apple_abi_probe.m'
    probe = native/'libapple_abi_probe.a'
    obj = native/'apple_abi_probe.o'
    report['test_sources'][str(probe_source.relative_to(ROOT))] = digest(probe_source)
    result = invoke(['cc','-c','-Wall','-Wextra','-Werror',probe_source,'-o',obj],env)
    result.update(name='native-C-oracle',phase='build',status='passed' if result['exit']==0 else 'failed')
    report['checks'].append(result)
    if result['exit'] == 0:
        result = invoke(['ar','rcs',probe,obj],env)
        result.update(name='native-C-oracle',phase='archive',status='passed' if result['exit']==0 else 'failed')
        report['checks'].append(result)
    if result['exit'] == 0:
        report['native_oracle_sha256'] = digest(probe)
        for name in ('reflection-procedure-flags','gl-context'):
            source = ROOT/'tests/native-bindings'/(name+'.jai')
            report['test_sources'][str(source.relative_to(ROOT))] = digest(source)
            exe = work/name
            build = invoke([compiler,source,'-exe',exe],env)
            build.update(name=name,phase='build',status='passed' if build['exit']==0 else 'failed')
            report['checks'].append(build)
            if build['exit'] != 0:
                continue
            run = invoke([exe],env)
            run.update(name=name,phase='run',status='passed' if run['exit']==0 else 'failed')
            report['checks'].append(run)
            if name == 'gl-context':
                callback = run['exit'] == 0 and 'OpenGL debug [' in run['stdout'] and 'binding_debug_marker' in run['stdout']
                unavailable = run['exit'] == 0 and 'SDK_DEBUG_OUTPUT_UNAVAILABLE' in run['stdout']
                report['checks'].append({'name':'GL-debug-callback','phase':'run',
                                         'status':'passed' if callback else 'unavailable' if unavailable else 'failed',
                                         'evidence':run['stdout']})
            if name == 'reflection-procedure-flags':
                interpreted = invoke([compiler,source,'-interp'],env)
                interpreted.update(name=name,phase='interpreter',status='passed' if interpreted['exit']==0 else 'failed')
                report['checks'].append(interpreted)
    args.report.write_text(json.dumps(report,indent=2)+'\n')
    failed = [r for r in report['checks'] if r['status'] != 'passed']
    print(str(len(failed))+' failed/unavailable SDK checks; report: '+str(args.report))
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main())
