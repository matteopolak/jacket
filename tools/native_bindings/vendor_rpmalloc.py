#!/usr/bin/env python3
"""Generate a private, complete rpmalloc C SDK ABI from the pinned vendor header.

This backend is distinct from both reviewed public Jai implementations. Parse
heap-disabled/enabled SDK configurations separately and reject unknown C forms.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
from config import RPMALLOC_SDK_BUILD_OPTIONS

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'stdlib/internal/bindings/rpmalloc-sdk'
SCALARS = {'void': 'void', 'int': 's32', 'unsigned int': 'u32',
           'size_t': 'u64', 'char': 'u8'}


def run(command):
    return subprocess.run([str(x) for x in command], check=True, capture_output=True, text=True)


def native_type(text):
    text = re.sub(r'\bconst\s+', '', text).strip()
    function = re.fullmatch(r'(.*?)\(\*\)\((.*?)\)', text)
    if function:
        arguments = [] if function[2] in ('', 'void') else function[2].split(',')
        params = ', '.join('arg'+str(i)+': '+native_type(t) for i, t in enumerate(arguments))
        result = native_type(function[1])
        return '#type ('+params+')'+(' -> '+result if result != 'void' else '')+' #c_call'
    if text.endswith('*'):
        return '*'+native_type(text[:-1])
    if text in SCALARS:
        return SCALARS[text]
    if text in ('rpmalloc_config_t', 'rpmalloc_thread_statistics_t',
                'rpmalloc_global_statistics_t', 'rpmalloc_heap_t'):
        return text
    raise ValueError('Unreviewed C SDK type: '+text)


def record(node, owner, path='', jai_path=''):
    if node.get('tagUsed') != 'struct' or not node.get('completeDefinition'):
        raise ValueError('Expected a complete C struct')
    lines, checks, nested = [], [], None
    for child in node.get('inner', []):
        kind = child['kind']
        if kind == 'FullComment':
            continue
        if kind == 'RecordDecl':
            if nested:
                raise ValueError('Unassociated nested record')
            nested = child
            continue
        if kind != 'FieldDecl' or not child.get('name') or child.get('isBitfield'):
            raise ValueError('Unsupported record member: '+kind)
        if any(x['kind'] != 'FullComment' for x in child.get('inner', [])):
            raise ValueError('Attributed/implicit member requires explicit ABI support')
        name, ctype = child['name'], child['type']['qualType']
        cpath, jpath = path+name, jai_path+name
        checks.append((owner, cpath, jpath))
        if nested:
            # A named array field of an anonymous record remains nested. Match
            # its source association instead of flattening or dropping fields.
            if child['range']['begin']['offset'] != nested['range']['begin']['offset']:
                raise ValueError('Nested record/field association is ambiguous')
            count = re.search(r'\[(\d+)\]$', ctype)
            if 'unnamed' not in ctype or not count:
                raise ValueError('Unsupported nested record declarator: '+ctype)
            body, rows = record(nested, owner, cpath+'[0].', jpath+'[0].')
            lines.append(name+': ['+count[1]+'] struct {\n'+body+'\n};')
            checks.extend(rows)
            nested = None
        else:
            lines.append(name+': '+native_type(ctype)+';')
    if nested:
        raise ValueError('Unassociated terminal nested record')
    return '\n'.join('    '+line.replace('\n', '\n    ') for line in lines), checks


def function(node):
    args = [x for x in node.get('inner', []) if x['kind'] == 'ParmVarDecl']
    if any(not x.get('name') for x in args):
        raise ValueError('SDK parameter name missing')
    result = native_type(node['type']['qualType'].split('(', 1)[0])
    params = ', '.join(x['name']+': '+native_type(x['type']['qualType']) for x in args)
    return node['name']+' :: ('+params+')'+(' -> '+result if result != 'void' else '')+' #foreign sdk;'


def responsibility(node):
    name = node['name']
    if name in ('rpmalloc_initialize', 'rpmalloc_initialize_config', 'rpmalloc_config', 'rpmalloc_finalize'):
        return 0, name
    if 'statistics' in name:
        return 2, name
    if name.startswith('rpmalloc_thread_') or name == 'rpmalloc_is_thread_initialized':
        return 1, name
    if name.startswith(('rpmalloc_heap_', 'rpmalloc_get_heap_')):
        return 4, name
    return (5 if name == 'rpmalloc_linker_reference' else 3), name


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--formatter', type=Path, required=True)
    args = parser.parse_args()
    pin = json.loads((ROOT/'tools/native_bindings/dependencies.json').read_text())['rpmalloc']
    vendor = ROOT/'.work/vendor/rpmalloc'
    for name, expected in pin['files'].items():
        if hashlib.sha256((vendor/name).read_bytes()).hexdigest() != expected:
            raise SystemExit('Pinned official rpmalloc input mismatch: '+name)
    header = vendor/'rpmalloc/rpmalloc.h'
    work = ROOT/'.work/rpmalloc-sdk-generation'
    work.mkdir(exist_ok=True)
    resource = run(['clang', '-print-resource-dir']).stdout.strip()
    profiles, record_profiles = {}, {}
    for heaps in (0, 1):
        ast = json.loads(run(['clang', '-DRPMALLOC_FIRST_CLASS_HEAPS='+str(heaps),
                             '-Xclang', '-ast-dump=json', '-fsyntax-only', header]).stdout)
        profiles[heaps] = [n for n in ast['inner'] if n['kind'] == 'FunctionDecl']
        record_profiles[heaps] = {n['name']:record(n,n['name']) for n in ast['inner']
                                 if n['kind']=='RecordDecl' and n.get('completeDefinition')
                                 and n.get('name','').startswith('rpmalloc_')}
    if record_profiles[0] != record_profiles[1]:
        raise ValueError('Native heap configurations have different records; preserve separate layouts')
    nodes = ast['inner']
    records = sorted([n for n in nodes if n['kind'] == 'RecordDecl' and
                      n.get('name', '').startswith('rpmalloc_') and n.get('completeDefinition')],
                     key=lambda n:n['name'])
    if len(records) != 3:
        raise ValueError('Expected all three complete native SDK records')
    blocks, checks = [], []
    for node in records:
        body, rows = record(node, node['name'])
        blocks.append(node['name']+' :: struct {\n'+body+'\n}')
        checks.extend(rows)
    functions = sorted(profiles[1], key=responsibility)
    code = '// Generated from complete pinned official rpmalloc C SDK; not a public Jai facade.\n'
    code += '// Opaque native heaps never alias the reviewed public Jai heap records.\n'
    code += '#assert size_of(*void) == 8, "The private rpmalloc SDK backend requires the verified 64-bit C ABI";\n\n'
    code += '\n\n'.join(blocks)+'\n\nrpmalloc_heap_t :: struct {}\n\n'
    code += '\n'.join(function(n) for n in functions)
    code += '\n\n#scope_file\nsdk :: #library,system "binding_rpmalloc_sdk";\n'
    temporary = work/'module.jai'
    temporary.write_text(code)
    run([args.formatter.resolve(), '--config', ROOT/'jaifmt.toml', temporary])
    code = temporary.read_text()
    # Runtime layout probes compare actual C SDK sizes/offsets with compiler
    # layout; no guessed offsets or host sizeof values are baked into Jai.
    probe = '#include "rpmalloc.h"\n#include <stddef.h>\n'
    probe += '_Static_assert(sizeof(void*) == 8 && sizeof(size_t) == 8 && sizeof(int) == 4, "64-bit C ABI");\n'
    for name, expressions in (
        ('size', ['sizeof('+n['name']+')' for n in records]),
        ('offset', ['offsetof('+owner+', '+path+')' for owner,path,_ in checks]),
        ('field_size', ['sizeof((('+owner+'*)0)->'+path+')' for owner,path,_ in checks])):
        probe += 'unsigned long long binding_rpmalloc_'+name+'(unsigned index) {\n    switch(index) {\n'
        probe += ''.join('        case '+str(i)+': return '+expr+';\n' for i,expr in enumerate(expressions))
        probe += '        default: __builtin_trap();\n    }\n}\n'
    source = work/'abi-probe.c'
    source.write_text(probe)
    targets = ['x86_64-linux-gnu', 'aarch64-linux-gnu', 'x86_64-apple-darwin',
               'aarch64-apple-darwin', 'x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc']
    for target in targets:
        run(['clang', '-target', target, '-nostdinc', '-isystem', Path(resource)/'include',
             '-I', header.parent, '-std=c11', '-Wall', '-Wextra', '-Werror', '-fsyntax-only', source])
    jai = 'sdk_layout_checks :: () {\n'
    for i,node in enumerate(records):
        jai += '    value'+str(i)+': '+node['name']+';\n'
        jai += '    check(size_of('+node['name']+') == sdk_size('+str(i)+'));\n'
    for i,(owner,path,jpath) in enumerate(checks):
        variable = 'value'+str(next(i for i,n in enumerate(records) if n['name']==owner))
        jai += '    check(cast(u64) *'+variable+'.'+jpath+' - cast(u64) *'+variable+' == sdk_offset('+str(i)+'));\n'
        jai += '    check(size_of(type_of('+variable+'.'+jpath+')) == sdk_field_size('+str(i)+'));\n'
    jai += '}\n'
    temporary.write_text(jai)
    run([args.formatter.resolve(), '--config', ROOT/'jaifmt.toml', temporary])
    provenance = dict(repository=pin['repository'], revision=pin['revision'], files=pin['files'],
                      scope='Private native SDK only; reviewed public Jai callbacks/heaps require separate adapters',
                      native_build_options=RPMALLOC_SDK_BUILD_OPTIONS,
                      functions=[dict(name=n['name'],signature=function(n)) for n in functions],
                      configurations={str(k):[n['name'] for n in v] for k,v in profiles.items()},
                      C_header_targets=targets, fields=[dict(record=o,path=p) for o,p,_ in checks])
    outputs = {OUT/'module.jai':code,
               ROOT/'tests/native-bindings/generated/rpmalloc_sdk_probe.c':probe,
               ROOT/'tests/native-bindings/generated/rpmalloc-sdk-layout.jai':temporary.read_text(),
               OUT/'provenance.json':json.dumps(provenance,indent=2,sort_keys=True)+'\n'}
    for path, data in outputs.items():
        if args.check:
            if not path.exists() or path.read_text() != data:
                raise SystemExit('Stale native rpmalloc generation: '+str(path))
        else:
            path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text(data)
    print(str(len(functions))+' native SDK functions, '+str(len(checks))+' field probes; six C target checks passed')


if __name__ == '__main__':
    main()
