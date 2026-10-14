#!/usr/bin/env python3
"""Run desktop checks and real host native tests with an explicit own compiler.

Results distinguish unsupported platforms, compiler failures, declaration
checks, public-surface checks, and native executions. A declaration import that
passes while a required wrapper is missing does not count as a complete module.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

from config import TARGETS
from generate import MODULES, condition_for
from spec import parse_spec, Declaration, Spec
from symbols import check_symbols
import re
from conditions import evaluate

ROOT = Path(__file__).resolve().parents[2]
TARGET_CONFIGS = [(os_name, cpu) for os_name in ('linux', 'macos', 'windows') for cpu in ('x64', 'arm64')]


def enabled(condition, os_name, cpu, parameters=None):
    if not condition:
        return True
    values = {'OS': os_name.upper(), 'CPU': cpu.upper(), 'ENABLED': False,
              'ENABLE_ALL_EXTENSIONS': False, 'DUMP_GL_ERRORS': False, 'INCLUDE_DEBUG_BINDINGS': False,
              'INCLUDE_MEDIA_BINDINGS': False, 'INCLUDE_CAMERA_BINDINGS': False,
              'DEFINE_ANDROID_MAIN_AND_CALL_THIS': False, 'ENABLE_STATISTICS': True,
              'RPMALLOC_FIRST_CLASS_HEAPS': True, 'USE_RAW':False,
              'USE_GLUE_NATIVE_PORT':True, 'NINTENDO_SUPPORT':False}
    values.update(parameters or {})
    return evaluate(condition,values)


def public_source(module, spec, os_name, cpu, parameters=None, arguments=''):
    lines = ['M :: #import "'+module+'"'+arguments+';']
    names = sorted(set(d.name for d in spec.declarations if d.name != 'operator =='))
    for i, name in enumerate(names):
        records = [d for d in spec.declarations if d.name == name]
        if any(enabled(condition_for(module,d,spec),os_name,cpu,parameters) for d in records):
            variable = any(re.match(r'(?:using )?\w+\s*:(?!:)',d.code) for d in records)
            expression = 'type_of(M.'+name+')' if variable else 'M.'+name
            lines.append('visible_'+str(i)+' :: '+expression+';')
    coverage = json.loads((ROOT/'docs/native/bindings-coverage.json').read_text())['modules'][module]
    members = sorted(set((d['name'],m['signature'].split('::',1)[0].strip(),m['condition'])
                         for d in coverage['declarations'] for m in d.get('members',[])))
    for i, (owner, member, condition) in enumerate(members):
        if enabled(condition,os_name,cpu,parameters):
            lines.append('visible_member_'+str(i)+' :: M.'+owner+'.'+member+';')
    nominal = set()
    for d in spec.declarations:
        if enabled(condition_for(module,d,spec),os_name,cpu,parameters) and re.search(r'::\s*(?:struct|union|enum(?:_flags)?)\b(?!\s*\()',d.code):
            nominal.add(d.name)
    for name in sorted(nominal):
        lines.append('#assert type_info(M.'+name+').name == "'+name+'";')
    for d in spec.declarations:
        alias = re.fullmatch(r'(?:using )?\w+\s*::\s*(\w+)',d.code)
        if alias and alias[1] in nominal and enabled(condition_for(module,d,spec),os_name,cpu,parameters):
            lines.append('#assert M.'+d.name+' == M.'+alias[1]+';')
            lines.append('#assert type_info(M.'+d.name+').name == "'+alias[1]+'";')
    lines.append('main :: () {}')
    return '\n'.join(lines)


def stage(destination, foundation=None):
    # This tree contains only other workers' NEW independent sources and our
    # owned bindings. Never copy a bootstrap definition into our source tree.
    foundation = foundation or ROOT.parent / 'stdlib' / 'stdlib'
    if not (foundation / 'prelude/module.jai').is_file():
        raise SystemExit('The independent foundation bootstrap is not ready')
    shutil.copytree(foundation, destination, dirs_exist_ok=True)
    for worker, modules, internals in [
        ('foundations',['Basic','String','Hash','File_Utilities'],['basic','basic-interface','string','hash','files','native-path']),
        ('window-render',['Window_Creation','Window_Type','Input','Keymap','Gamepad','Simp','sRGB','Clipboard'],['window','rendering','input','gamepad','clipboard']),
        ('numerics',['Math','Sloppy_Math','Float16','Color_Interpolation','GetRect','GetRect_LeftHanded'],['numerics']),
        ('compiler',['Compiler','Jai_Lexer','Code_Visit','Program_Print'],['compiler-api','jai-lexer','code-visit','program-print'])
    ]:
        source = ROOT.parent/worker/'stdlib'
        for name in modules:
            if (source/name/'module.jai').is_file():
                shutil.copytree(source/name,destination/name,dirs_exist_ok=True)
        for name in internals:
            if (source/'internal'/name).is_dir():
                shutil.copytree(source/'internal'/name,destination/'internal'/name,dirs_exist_ok=True)
    for module in MODULES:
        shutil.copytree(ROOT/'stdlib'/module, destination/module, dirs_exist_ok=True)
    shutil.copytree(ROOT/'stdlib/internal/bindings', destination/'internal/bindings', dirs_exist_ok=True)


def invoke(command, env, timeout=120):
    try:
        p = subprocess.run([str(x) for x in command], env=env, capture_output=True, text=True, timeout=timeout)
        return {'exit': p.returncode, 'stdout': p.stdout, 'stderr': p.stderr}
    except subprocess.TimeoutExpired as e:
        return {'exit': None, 'stdout': str(e.stdout or ''), 'stderr': 'Timeout after ' + str(timeout) + ' seconds'}


def select_stdlib(work, foundation=None, production=None):
    if production is not None:
        if foundation is not None:
            raise ValueError('production --stdlib cannot be combined with --foundation overlays')
        root = production.resolve(strict=True)
        if not (root / 'prelude/module.jai').is_file():
            raise ValueError('production library requires its canonical prelude/module.jai')
        return root
    destination = work / 'stdlib'
    stage(destination, foundation)
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--module', action='append', choices=MODULES)
    parser.add_argument('--runtime', action='store_true')
    parser.add_argument('--native-libs', type=Path, default=ROOT/'.work/native', help='Actual source-built native libraries used by runtime checks')
    parser.add_argument('--privacy-only', action='store_true', help='Run the full target privacy matrix without repeating declaration/runtime suites')
    parser.add_argument('--report', type=Path, default=ROOT/'.work/verification.json')
    parser.add_argument('--foundation', type=Path, help='Explicit previously staged NEW independent foundation sources')
    parser.add_argument('--stdlib', type=Path, help='Verify this production library directly, without any source overlays')
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    if not compiler.is_relative_to(ROOT/'target'):
        parser.error('compiler must be source-built under this checkout target directory')
    native_libraries = args.native_libs.resolve()
    work = args.report.resolve().parent/'runs'/(args.report.stem+'-'+str(os.getpid()))
    work.mkdir(parents=True, exist_ok=True)
    stdlib = select_stdlib(work, args.foundation, args.stdlib)
    temporary = work / 'tmp'
    temporary.mkdir()
    env = dict(os.environ, JAIC_STDLIB=str(stdlib), JAIC_NATIVE_LIBS=str(native_libraries), TMPDIR=str(temporary))
    report = {'compiler': str(compiler), 'compiler_sha256': hashlib.sha256(compiler.read_bytes()).hexdigest(), 'staging_root': str(work),
              'stdlib_root': str(stdlib), 'production': args.stdlib is not None, 'native_libraries': str(native_libraries),
              'targets': [], 'native': [], 'privacy': []}
    report['staged_sources'] = {str(p.relative_to(stdlib)):hashlib.sha256(p.read_bytes()).hexdigest()
                                for p in sorted(stdlib.rglob('*.jai')) if 'Extensions' not in p.parts}
    report['specs'] = {}
    report['symbols'] = []
    report['test_sources'] = {}
    report['native_artifacts'] = {p.name:hashlib.sha256(p.read_bytes()).hexdigest()
                                  for p in sorted(native_libraries.glob('lib*')) if p.is_file()} if args.runtime else {}
    report['privacy_only'] = args.privacy_only
    for module, slug in ([] if args.privacy_only else MODULES.items()):
        if args.module and module not in args.module:
            continue
        spec = parse_spec((ROOT/'docs/specs/stdlib/modules'/f'{slug}.md').read_text())
        report['specs'][module] = hashlib.sha256((ROOT/'docs/specs/stdlib/modules'/f'{slug}.md').read_bytes()).hexdigest()
        if args.runtime and module in ('lz4','meshoptimizer','Curl','freetype','stb_image','stb_image_resize','stb_image_write','stb_vorbis','MojoShader','ImGui','SDL'):
            coverage = json.loads((ROOT/'docs/native/bindings-coverage.json').read_text())['modules'][module]
            members = [Declaration(m['signature'],m['condition']) for d in coverage['declarations'] for m in d.get('members',[])]
            symbols = Spec(declarations=spec.declarations+members)
            report['symbols'].extend(check_symbols(module, symbols, enabled, native_libraries))
        for os_name, cpu in TARGET_CONFIGS:
            file = work/'module-check.jai'
            file.write_text('#import "' + module + '"; main :: () {}\n')
            command = [compiler, file, '-check', '-no_dce', '-os', os_name, '-cpu', cpu]
            result = invoke(command, env)
            result.update(module=module, os=os_name, cpu=cpu, check='declarations')
            result['status'] = 'passed' if result['exit'] == 0 else 'unsupported' if 'requires its documented native platform' in result['stderr'] else 'failed'
            report['targets'].append(result)
            if result['status'] == 'passed':
                # Qualified aliases force lookup of required importer-visible
                # names. They do not promote helpers or make missing procedures
                # disappear behind DCE.
                file.write_text(public_source(module,spec,os_name,cpu))
                public_result = invoke(command, env)
                public_result.update(module=module, os=os_name, cpu=cpu, check='public_names',
                                     status='passed' if public_result['exit'] == 0 else 'failed')
                report['targets'].append(public_result)
            print(module, os_name+'-'+cpu, result['status'])
    for module, filename, configs in [
        ('Objective_C', 'appkit.jai', [('macos','x64'),('macos','arm64')]),
        ('GL', 'gl-output-buffers.jai', TARGET_CONFIGS),
        ('GL', 'glx-context.jai', [('linux','x64'),('linux','arm64')]),
        ('GL', 'wgl-context.jai', [('windows','x64'),('windows','arm64')]),
        ('GL', 'gl-debug-output.jai', TARGET_CONFIGS),
    ]:
        if args.privacy_only:
            continue
        if args.module and module not in args.module:
            continue
        source = ROOT/'tests/native-bindings'/filename
        report['test_sources'][str(source.relative_to(ROOT))] = hashlib.sha256(source.read_bytes()).hexdigest()
        for os_name, cpu in configs:
            result = invoke([compiler,source,'-check','-no_dce','-os',os_name,'-cpu',cpu],env)
            result.update(module=module, os=os_name, cpu=cpu, check='source_regression',
                          source=str(source.relative_to(ROOT)), status='passed' if result['exit']==0 else 'failed')
            report['targets'].append(result)
    for module, private in [('lz4', 'liblz4'), ('xxHash', 'binding_XXH32'), ('stb_image', 'FILE'),
                            ('Curl', 'Native'), ('Curl', 'Varargs'), ('dxc_compiler', 'IMalloc'),
                            ('MojoShader', 'MOJOSHADER_uniformType'), ('Objective_C', 'binding_objc_class'),
                            ('Objective_C', 'NSWindow'), ('Objective_C/AppKit', 'NSView'),
                            ('Objective_C/AppKit', 'NSScreen'), ('Objective_C/AppKit', 'send_result'),
                            ('SDL', 'NativeSDL')]:
        if args.module and module.split('/')[0] not in args.module and module not in args.module:
            continue
        for os_name, cpu in TARGET_CONFIGS:
            result = {'module':module, 'member':private, 'os':os_name, 'cpu':cpu}
            target = TARGETS.get(module.split('/')[0], '')
            if target and not enabled(target,os_name,cpu):
                result.update(status='unsupported', reason='Reviewed module target condition: '+target)
                report['privacy'].append(result)
                continue
            file = work/('privacy-'+module.replace('/','_')+'-'+private+'-'+os_name+'-'+cpu+'.jai')
            file.write_text('M :: #import "'+module+'"; leaked :: M.'+private+'; main :: () {}\n')
            result.update(invoke([compiler,file,'-check','-os',os_name,'-cpu',cpu], env))
            result['status'] = 'passed' if result['exit'] != 0 and ('has no exported member `'+private+'`') in result['stderr'] else 'failed'
            if result['status'] == 'failed':
                result['reason'] = 'Required privacy diagnostic was not reached; failure is not evidence that privacy passed'
            report['privacy'].append(result)
    if args.runtime and not args.privacy_only:
        runtime_tests = ['lz4', 'xxhash', 'images', 'meshoptimizer', 'freetype', 'posix', 'posix-abi', 'curl', 'ios', 'objective-c', 'appkit', 'mojoshader', 'macos', 'metal', 'gl', 'nsgl', 'gl-context', 'reflection-procedure-flags', 'graphics-math', 'sdl', 'sdl-wm', 'sdl-layout']
        if sys.platform == 'win32':
            runtime_tests.append('wgl-context')
        for name in runtime_tests:
            relevant = {'xxhash': ['xxHash'], 'images': ['stb_image', 'stb_image_write', 'stb_image_resize', 'stb_vorbis'], 'posix': ['POSIX'], 'curl':['Curl'], 'ios':['iOS'], 'objective-c':['Objective_C'], 'mojoshader':['MojoShader'], 'macos':['macOS'], 'metal':['Metal'], 'gl':['GL'], 'graphics-math':['d3d12'], 'sdl':['SDL'], 'sdl-layout':['SDL']}.get(name, [name])
            if name == 'appkit':
                relevant = ['Objective_C']
            if name == 'sdl-wm':
                relevant = ['SDL']
            if name == 'posix-abi':
                relevant = ['POSIX']
            if name == 'wgl-context':
                relevant = ['GL']
            if name in ('gl-context','reflection-procedure-flags','nsgl'):
                relevant = ['GL']
            if args.module and not any(m in args.module for m in relevant):
                continue
            source = ROOT/'tests/native-bindings'/f'{name}.jai'
            if name == 'sdl-wm':
                source = ROOT/'stdlib/SDL/tests/sdl-wm.jai'
            if not source.is_file():
                report['native'].append({'name': name, 'status': 'not_implemented'})
                continue
            exe = work/(name+'-test')
            report['test_sources'][str(source.relative_to(ROOT))] = hashlib.sha256(source.read_bytes()).hexdigest()
            result = invoke([compiler, source, '-exe', exe], env)
            scope = 'private_arithmetic' if name=='graphics-math' else 'compiler_semantics' if name=='reflection-procedure-flags' else 'library_runtime'
            result.update(name=name, phase='build',scope=scope,status='passed' if result['exit']==0 else 'failed')
            report['native'].append(result)
            if result['exit'] == 0:
                result = invoke([exe], env)
                result.update(name=name, phase='run', status='passed' if result['exit']==0 else 'failed')
                report['native'].append(result)
        if not args.module or 'ImGui' in args.module:
            pin = json.loads((ROOT/'tools/native_bindings/dependencies.json').read_text())['imgui']
            exe = work/'imgui-ffi-test'
            source = ROOT/'tests/native-bindings/imgui_ffi.cpp'
            report['test_sources'][str(source.relative_to(ROOT))] = hashlib.sha256(source.read_bytes()).hexdigest()
            command = ['c++','-std=c++17','-Wall','-Wextra','-Werror','-isystem',ROOT/'.work/vendor'/pin['archive']['root'],
                       ROOT/'tests/native-bindings/imgui_ffi.cpp',native_libraries/'libimgui.a',
                       native_libraries/'libbinding_ffi.a','-lffi','-o',exe]
            result = invoke(command,env)
            result.update(name='imgui-ffi',language='C++',phase='build',header_sha256=pin['files']['imgui.h'],
                          status='passed' if result['exit']==0 else 'failed')
            report['native'].append(result)
            if result['exit'] == 0:
                result = invoke([exe],env)
                result.update(name='imgui-ffi',language='C++',phase='run',status='passed' if result['exit']==0 else 'failed')
                report['native'].append(result)
    args.report.write_text(json.dumps(report, indent=2)+'\n')
    failures = [x for category in ['targets','privacy','native','symbols'] for x in report[category] if x['status'] in ['failed','not_implemented','unavailable']]
    print(str(len(failures))+' failed or unimplemented checks; report: '+str(args.report))
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
