#!/usr/bin/env python3
"""Execute source-built macOS POSIX tests against each CPU's public C SDK ABI.

The x64 executable may use the host's OS translation support. Neither an SDK
compiler mock nor a changed stdlib/compiler source is used to obtain a pass.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import sys

from verify import ROOT, invoke, stage


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != 'Darwin':
        raise SystemExit('This SDK/CPU execution driver requires a real macOS host')
    compiler = args.compiler.resolve(strict=True)
    work = ROOT/'.work/runs'/(args.report.stem+'-'+str(os.getpid()))
    work.mkdir(parents=True, exist_ok=True)
    stdlib = work/'stdlib'
    stage(stdlib)
    env = dict(os.environ, JAIC_STDLIB=str(stdlib), TMPDIR=str(ROOT/'.work/tmp'))
    report = {'scope':'Worker native macOS CPU ABI/runtime tests; not coordinator integration',
              'compiler':str(compiler), 'compiler_sha256':digest(compiler),
              'host_cpu':platform.machine(), 'staging_root':str(work),
              'staging_adjustments':[], 'checks':[], 'test_sources':{},
              'staged_sources':{str(p.relative_to(stdlib)):digest(p)
                                for p in sorted(stdlib.rglob('*.jai')) if 'Extensions' not in p.parts}}
    sdk = invoke(['xcrun','--show-sdk-path'], env)
    if sdk['exit'] != 0:
        raise SystemExit(sdk['stderr'])
    sdk_path = Path(sdk['stdout'].strip())
    report['sdk_headers'] = {name:digest(sdk_path/'usr/include'/name) for name in
                              ('time.h','sys/time.h','sys/uio.h','sys/ipc.h','poll.h','signal.h')}
    report['sdk_version'] = invoke(['xcrun','--show-sdk-version'], env)
    report['sdk_build'] = invoke(['xcrun','--show-sdk-build-version'], env)
    report['os_version'] = invoke(['sw_vers'], env)
    probe = ROOT/'stdlib/Bindings_Generator/tests/driven/sdk/posix_abi_probe.c'
    report['test_sources'][str(probe.relative_to(ROOT))] = digest(probe)
    for cpu, arch, triple in [('x64','x86_64','x86_64-apple-darwin'),
                               ('arm64','arm64','aarch64-apple-darwin')]:
        native = work/cpu
        native.mkdir()
        obj = native/'posix_abi_probe.o'
        archive = native/'libposix_abi_probe.a'
        target_env = dict(env, JAIC_NATIVE_LIBS=str(native))
        commands = [('C-oracle-build',['cc','-arch',arch,'-Wall','-Wextra','-Werror','-c',probe,'-o',obj]),
                    ('C-oracle-archive',['ar','rcs',archive,obj])]
        for name, command in commands:
            result = invoke(command,target_env)
            result.update(cpu=cpu,name=name,status='passed' if result['exit']==0 else 'failed')
            report['checks'].append(result)
            if result['exit'] != 0:
                break
        if result['exit'] != 0:
            continue
        report.setdefault('native_oracles',{})[cpu] = {'path':str(archive),'sha256':digest(archive)}
        for name in ('posix-abi','posix','posix-stat'):
            source = ROOT/'stdlib/Bindings_Generator/tests/driven/sdk'/(name+'.jai')
            report['test_sources'][str(source.relative_to(ROOT))] = digest(source)
            exe = native/name
            result = invoke([compiler,source,'-target',triple,'-exe',exe], target_env)
            result.update(cpu=cpu,name=name,phase='build',status='passed' if result['exit']==0 else 'failed')
            report['checks'].append(result)
            if result['exit'] != 0:
                continue
            result = invoke([exe],target_env)
            result.update(cpu=cpu,name=name,phase='run',executable_sha256=digest(exe),
                          status='passed' if result['exit']==0 else 'failed')
            report['checks'].append(result)
    args.report.write_text(json.dumps(report,indent=2)+'\n')
    failures = sum(r['status'] != 'passed' for r in report['checks'])
    print(str(failures)+' failed native CPU checks; report: '+str(args.report))
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
