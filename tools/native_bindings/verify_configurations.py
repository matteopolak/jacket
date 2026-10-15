#!/usr/bin/env python3
"""Check generated alternatives separately, including unavailable mobile targets."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys

from generate import MODULES
from spec import parse_spec
from verify import ROOT, TARGET_CONFIGS, select_stdlib, invoke, public_source

# (module, parameter values, import suffix, applicable OS values)
CONFIGURATIONS = [
    ('GL', {'ENABLE_ALL_EXTENSIONS':True}, '(ENABLE_ALL_EXTENSIONS=true)', ('linux','macos','windows')),
    ('GL', {'DUMP_GL_ERRORS':True}, '(DUMP_GL_ERRORS=true)', ('linux','macos','windows')),
    ('GL', {'ENABLE_ALL_EXTENSIONS':True,'DUMP_GL_ERRORS':True}, '(ENABLE_ALL_EXTENSIONS=true,DUMP_GL_ERRORS=true)', ('linux','macos','windows')),
    ('d3d11', {'INCLUDE_DEBUG_BINDINGS':True}, '(INCLUDE_DEBUG_BINDINGS=true)', ('windows',)),
    ('d3d12', {'INCLUDE_DEBUG_BINDINGS':True}, '(INCLUDE_DEBUG_BINDINGS=true)', ('windows',)),
    ('telemetry3', {'ENABLED':True}, '(ENABLED=true)', ('windows',)),
    ('rpmalloc', {'USE_RAW':True}, '(USE_RAW=true)', ('linux','macos','windows')),
    ('rpmalloc', {'USE_RAW':True,'ENABLE_STATISTICS':False}, '(USE_RAW=true,ENABLE_STATISTICS=false)', ('linux','macos','windows')),
    ('rpmalloc', {'ENABLE_STATISTICS':False}, '(ENABLE_STATISTICS=false)', ('linux','macos','windows')),
    ('Android', {}, '', ('android',)),
    ('Android', {'INCLUDE_MEDIA_BINDINGS':True,'INCLUDE_CAMERA_BINDINGS':True}, '()(INCLUDE_MEDIA_BINDINGS=true,INCLUDE_CAMERA_BINDINGS=true)', ('android',)),
    ('Android', {'USE_GLUE_NATIVE_PORT':False}, '()(USE_GLUE_NATIVE_PORT=false)', ('android',)),
    ('Android/AAudio', {}, '', ('android',)),
    ('Android/EGL', {}, '', ('android',)),
    ('Objective_C', {}, '', ('ios',)),
    ('Metal', {}, '', ('ios',)),
    ('iOS', {}, '', ('ios',)),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--foundation', type=Path)
    parser.add_argument('--stdlib', type=Path, help='Production library, without peer overlays')
    parser.add_argument('--module', action='append', choices=MODULES)
    parser.add_argument('--rpmalloc-first-class-heaps', action='store_true')
    parser.add_argument('--report', type=Path, default=ROOT/'.work/configuration-verification.json')
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    work = args.report.resolve().parent/'runs'/(args.report.stem+'-'+str(os.getpid()))
    work.mkdir(parents=True,exist_ok=True)
    stdlib = select_stdlib(work, args.foundation, args.stdlib)
    from configure_rpmalloc import select_for_verification
    select_for_verification(stdlib, args.rpmalloc_first_class_heaps, args.stdlib is not None)
    temporary = work / 'tmp'
    temporary.mkdir()
    env = dict(os.environ,JAIC_STDLIB=str(stdlib),JAIC_NATIVE_LIBS=str(ROOT/'.work/native'),TMPDIR=str(temporary))

    report = {'compiler':str(compiler),'compiler_sha256':hashlib.sha256(compiler.read_bytes()).hexdigest(),
              'stdlib_root':str(stdlib),'production':args.stdlib is not None,
              'staging_root':str(work),'staged_sources':{
                  str(p.relative_to(stdlib)):hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in sorted(stdlib.rglob('*.jai')) if 'Extensions' not in p.parts}, 'checks':[]}
    for module, parameters, suffix, systems in CONFIGURATIONS:
        if args.module and module not in args.module:
            continue
        page = ROOT/'docs/specs/stdlib/modules'/(MODULES[module]+'.md')
        spec = parse_spec(page.read_text())
        if module=='rpmalloc' and args.rpmalloc_first_class_heaps:
            parameters = dict(parameters,RPMALLOC_FIRST_CLASS_HEAPS=True)
        for os_name in systems:
            for cpu in ('x64','arm64'):
                file = work/'configuration-check.jai'
                command = [compiler,file,'-check','-no_dce','-os',os_name,'-cpu',cpu]
                for check in ('declarations','public_names'):
                    source = public_source(module,spec,os_name,cpu,parameters,suffix) if check=='public_names' else 'M :: #import "'+module+'"'+suffix+'; main :: () {}\n'
                    file.write_text(source)
                    result = invoke(command,env)
                    result.update(module=module,parameters=parameters,os=os_name,cpu=cpu,check=check,
                                  spec_sha256=hashlib.sha256(page.read_bytes()).hexdigest(),
                                  status='passed' if result['exit']==0 else 'failed')
                    report['checks'].append(result)
                    # A failed declaration check already accounts for this
                    # configuration; qualification cannot compile it either.
                    if result['exit'] != 0:
                        break
                print(module,parameters,os_name,cpu,report['checks'][-1]['status'],flush=True)
                args.report.write_text(json.dumps(report,indent=2)+'\n')
    failures = sum(c['status']=='failed' for c in report['checks'])
    print(failures,'failed configuration checks; report:',args.report)
    return bool(failures)


if __name__ == '__main__':
    sys.exit(main())
