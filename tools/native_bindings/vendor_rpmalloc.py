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
from rpmalloc_heap_facade import public_c_header, layout_probes
from rpmalloc_profiles import PROFILES, parameters, select_blocks, heap_module, callbacks_module

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'stdlib/internal/bindings/rpmalloc-sdk'
SCALARS = {'void': 'void', 'int': 's32', 'unsigned int': 'u32',
           'size_t': 'u64', 'char': 'u8'}
STORAGE_RECORDS = {
    'heap_size_class_t':'Size_Class_Storage',
    'span_cache_t':'Span_Cache_Storage',
    'span_large_cache_t':'Large_Cache_Storage',
    'span_use_t':'Span_Counters_Storage',
    'size_class_use_t':'Size_Counters_Storage',
    'span_t':'Span_Storage', 'heap_t':'Heap_Storage',
}


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


def storage_type(text):
    text = re.sub(r'\b(?:const|volatile|struct)\s+', '', text).strip()
    array = re.fullmatch(r'(.*?)\s*\[(\d+)\]', text)
    if array:
        return '['+array[2]+'] '+storage_type(array[1])
    if text.endswith('*'):
        return '*'+storage_type(text[:-1])
    scalars = dict(SCALARS, uintptr_t='u64', uint32_t='u32', int32_t='s32',
                   uint64_t='u64', int64_t='s64', atomic32_t='s32',
                   atomic64_t='s64', atomicptr_t='*void')
    if text in scalars:
        return scalars[text]
    if text in STORAGE_RECORDS:
        return STORAGE_RECORDS[text]
    raise ValueError('Unknown native storage type: '+text)


def record(node, owner, path='', jai_path='', type_mapper=native_type):
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
            body, rows = record(nested, owner, cpath+'[0].', jpath+'[0].', type_mapper)
            lines.append(name+': ['+count[1]+'] struct {\n'+body+'\n};')
            checks.extend(rows)
            nested = None
        else:
            lines.append(name+': '+type_mapper(ctype)+';')
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
    if name.startswith('rpmalloc_thread_') or name in ('rpmalloc_is_thread_initialized','rpmalloc_set_main_thread'):
        return 1, name
    if name.startswith(('rpmalloc_heap_', 'rpmalloc_get_heap_')):
        return 4, name
    return (5 if name == 'rpmalloc_linker_reference' else 3), name


def storage_outputs(vendor, functions, work, formatter):
    profiles = {}
    source_exports = {}
    source = vendor/'rpmalloc/rpmalloc.c'
    for heaps in (0, 1):
        options = dict(RPMALLOC_SDK_BUILD_OPTIONS, RPMALLOC_FIRST_CLASS_HEAPS=heaps)
        ast = json.loads(run(['clang','-std=c11',
                             *['-D'+key+'='+str(value) for key,value in options.items()],
                             '-Xclang','-ast-dump=json','-fsyntax-only',source]).stdout)
        nodes = {n['name']:n for n in ast['inner'] if n['kind']=='RecordDecl'
                 and n.get('completeDefinition') and n.get('name') in STORAGE_RECORDS}
        if set(nodes) != set(STORAGE_RECORDS):
            raise ValueError('Incomplete official native heap/storage record family')
        profiles[heaps] = {name:record(nodes[name],name,type_mapper=storage_type)
                           for name in STORAGE_RECORDS}
        source_exports[heaps] = {n['name']:n for n in ast['inner'] if n['kind']=='FunctionDecl'
                                and n.get('storageClass') != 'static'
                                and not n.get('loc',{}).get('includedFrom')
                                and any(child['kind']=='CompoundStmt' for child in n.get('inner',[]))}
        if not {n['name'] for n in functions if n['name'] in source_exports[heaps]}:
            raise ValueError('No native source exports identified')
    chosen = profiles[1]
    code = '// Generated from complete pinned official rpmalloc C source, SDK build profile.\n'
    code += '// Storage layout only: these scalar views do not implement atomic operations.\n'
    code += '// Never alias reviewed public Jai heaps/spans to these native records.\n'
    code += '#assert size_of(*void) == 8, "Verified native storage requires 64-bit pointers";\n\n'
    code += '\n\n'.join(STORAGE_RECORDS[name]+' :: struct {\n'+chosen[name][0]+'\n}' for name in STORAGE_RECORDS)+'\n'
    temporary = work/'storage.jai'
    temporary.write_text(code)
    run([formatter.resolve(),'--config',ROOT/'jaifmt.toml',temporary])
    outputs = {ROOT/'stdlib/internal/bindings/rpmalloc-storage/module.jai':temporary.read_text()}
    # Include unmodified pinned C source so probes measure its actual private
    # declarations. Prefix all SDK exports; this probe never initializes or
    # allocates from its separately compiled SDK instance.
    probe = '// Generated native ABI probe from complete pinned official C source.\n'
    probe += ''.join('#define '+name+' binding_storage_probe_'+name+'\n' for name in sorted(source_exports[1]))
    probe += '#include "rpmalloc.c"\n'
    probe += 'long long binding_rpmalloc_native_heap_counter(void* pointer,unsigned family,unsigned index,unsigned field) { heap_t* heap=pointer;\n'
    for family,(name,member,limit) in enumerate((('span_use_t','span_use','LARGE_CLASS_COUNT'),('size_class_use_t','size_class_use','SIZE_CLASS_COUNT+1'))):
        probe += 'if(family=='+str(family)+' && index<('+limit+')) { switch(field) {\n'
        for i,(_,field,_) in enumerate(chosen[name][1]):
            expression = 'heap->'+member+'[index].'+field
            if field not in ('high','alloc_peak','spans_peak','unused'):
                expression = 'atomic_load32(&'+expression+')'
            probe += 'case '+str(i)+':return '+expression+';\n'
        probe += '} }\n'
    probe += 'abort(); }\n'
    probe += ''.join('#undef '+name+'\n' for name in sorted(source_exports[1]))
    probe += '#include <stddef.h>\n'
    checks = [row for name in STORAGE_RECORDS for row in chosen[name][1]]
    for suffix, expressions in (
        ('size',['sizeof('+name+')' for name in STORAGE_RECORDS]),
        ('offset',['offsetof('+owner+', '+path+')' for owner,path,_ in checks]),
        ('field_size',['sizeof((('+owner+'*)0)->'+path+')' for owner,path,_ in checks]),
    ):
        probe += 'unsigned long long binding_rpmalloc_storage_'+suffix+'(unsigned index) {\n    switch(index) {\n'
        probe += ''.join('        case '+str(i)+': return '+expression+';\n' for i,expression in enumerate(expressions))
        probe += '        default: __builtin_trap();\n    }\n}\n'
    outputs[ROOT/'tests/native-bindings/generated/rpmalloc_storage_probe.c'] = probe
    outputs[ROOT/'tests/native-bindings/generated/rpmalloc_storage_probe.c'] += (
        'long long binding_rpmalloc_native_heap_id(const void *heap) { return ((const heap_t*)heap)->id; }\n'
        'unsigned long long binding_rpmalloc_native_heap_owner(const void *heap) { return ((const heap_t*)heap)->owner_thread; }\n')
    outputs[ROOT/'tests/native-bindings/generated/rpmalloc_storage_probe.c'] += (
        'long long binding_rpmalloc_public_atomic32(void *value) { return atomic_load32((atomic32_t*)value); }\n')
    jai = 'storage_layout_checks :: () {\n'
    for i,name in enumerate(STORAGE_RECORDS):
        jai += '    value'+str(i)+': Storage.'+STORAGE_RECORDS[name]+';\n'
        jai += '    assert(size_of(Storage.'+STORAGE_RECORDS[name]+') == storage_size('+str(i)+'));\n'
    for i,(owner,path,jpath) in enumerate(checks):
        variable = 'value'+str(list(STORAGE_RECORDS).index(owner))
        jai += '    assert(cast(u64) *'+variable+'.'+jpath+' - cast(u64) *'+variable+' == storage_offset('+str(i)+'));\n'
        jai += '    assert(size_of(type_of('+variable+'.'+jpath+')) == storage_field_size('+str(i)+'));\n'
    jai += '}\n'
    # Differences are derived by name, never guessed field indexes. The public
    # record oracle is the reviewed spec; the native oracle is pinned C source.
    jai += 'public_storage_separation_checks :: () {\n    heap: Heap;\n    span: Span;\n'
    for public, native, variable, member in [('Heap','heap_t','heap','span_cache'),
                                           ('Heap','heap_t','heap','span_free_deferred'),
                                           ('Span','span_t','span','free_list_deferred')]:
        index = next(i for i,(owner,path,_) in enumerate(checks) if owner==native and path==member)
        comparison = '!=' if public == 'Span' else '=='
        jai += '    assert(size_of('+public+') '+comparison+' storage_size('+str(list(STORAGE_RECORDS).index(native))+'));\n'
        jai += '    assert(cast(u64) *'+variable+'.'+member+' - cast(u64) *'+variable+' != storage_offset('+str(index)+'));\n'
    jai += '    assert(type_info(Heap).name == "Heap");\n    assert(type_info(Span).name == "Span");\n}\n'
    temporary.write_text(jai)
    run([formatter.resolve(),'--config',ROOT/'jaifmt.toml',temporary])
    outputs[ROOT/'tests/native-bindings/generated/rpmalloc-storage-layout.jai'] = temporary.read_text()
    metadata = {'scope':'Native SDK storage ABI only; not public heap implementation or atomic operations',
                'source':'rpmalloc/rpmalloc.c',
                'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
                'native_build_options':RPMALLOC_SDK_BUILD_OPTIONS,
                'profiles':{str(k):{name:[dict(owner=o,path=p) for o,p,_ in value[1]]
                                    for name,value in v.items()} for k,v in profiles.items()},
                'native_field_count':len(checks), 'records':STORAGE_RECORDS,
                'source_export_profiles':{str(k):sorted(v) for k,v in source_exports.items()},
                'probe_sdk_exports':'All pinned SDK exports prefixed; probe instance never executed',
                'verification':'Host C source parsing here; native/source evidence indexed separately'}
    outputs[ROOT/'stdlib/internal/bindings/rpmalloc-storage/provenance.json'] = json.dumps(metadata,indent=2,sort_keys=True)+'\n'
    return outputs, metadata, source_exports[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--formatter', type=Path, required=True)
    parser.add_argument('--vendor', type=Path, default=ROOT/'.work/vendor/rpmalloc')
    parser.add_argument('--work', type=Path, default=ROOT/'.work/rpmalloc-sdk-generation')
    args = parser.parse_args()
    pin = json.loads((ROOT/'tools/native_bindings/dependencies.json').read_text())['rpmalloc']
    vendor = args.vendor.resolve()
    for name, expected in pin['files'].items():
        if hashlib.sha256((vendor/name).read_bytes()).hexdigest() != expected:
            raise SystemExit('Pinned official rpmalloc input mismatch: '+name)
    header = vendor/'rpmalloc/rpmalloc.h'
    work = args.work.resolve()
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
                      build_fixes=[dict(path='stdlib/internal/bindings/rpmalloc-heaps/sdk-page-granularity.patch',
                                       sha256=hashlib.sha256((ROOT/'stdlib/internal/bindings/rpmalloc-heaps/sdk-page-granularity.patch').read_bytes()).hexdigest(),
                                       purpose='Round native heap-pool mappings to whole configured pages')],
                      scope='Private native SDK only; reviewed public Jai callbacks/heaps require separate adapters',
                      native_build_options=RPMALLOC_SDK_BUILD_OPTIONS,
                      functions=[dict(name=n['name'],signature=function(n)) for n in functions],
                      configurations={str(k):[n['name'] for n in v] for k,v in profiles.items()},
                      C_header_targets=targets, fields=[dict(record=o,path=p) for o,p,_ in checks])
    outputs = {OUT/'module.jai':code,
               ROOT/'tests/native-bindings/generated/rpmalloc_sdk_probe.c':probe,
               ROOT/'tests/native-bindings/generated/rpmalloc-sdk-layout.jai':temporary.read_text(),
               OUT/'provenance.json':json.dumps(provenance,indent=2,sort_keys=True)+'\n'}
    storage, storage_metadata, source_exports = storage_outputs(vendor,functions,work,args.formatter)
    outputs.update(storage)
    for heaps, nodes in profiles.items():
        missing = {n['name'] for n in nodes} - set(storage_metadata['source_export_profiles'][str(heaps)])
        if missing:
            raise ValueError('Header/source export mismatch for heap profile '+str(heaps)+': '+repr(missing))
    header_names = {n['name'] for n in functions}
    if header_names - source_exports.keys():
        raise ValueError('Header API has no source definition: '+repr(header_names-source_exports.keys()))
    extra_exports = sorted((node for name,node in source_exports.items() if name not in header_names),key=responsibility)
    if extra_exports:
        prefix = code[:code.index(functions[0]['name']+' :: (')]
        functions = sorted(functions+extra_exports,key=responsibility)
        code = prefix+'\n'.join(function(node) for node in functions)+'\n\n#scope_file\nsdk :: #library,system "binding_rpmalloc_sdk";\n'
        temporary.write_text(code)
        run([args.formatter.resolve(),'--config',ROOT/'jaifmt.toml',temporary])
        outputs[OUT/'module.jai'] = temporary.read_text()
        provenance['functions'] = [dict(name=n['name'],signature=function(n),
                                       origin='public_header' if n['name'] in header_names else 'official_c_source_definition') for n in functions]
    provenance['native_storage'] = storage_metadata
    # Distinct domains preserve callback, ownership and public record layouts.
    prefix = code[:code.index(functions[0]['name']+' :: (')]
    code = parameters()+prefix+select_blocks(lambda p: '\n'.join(
        function(node).replace('#foreign sdk;', '#foreign sdk "'+p['symbol_prefix']+node['name']+'";')
        if p['symbol_prefix'] else function(node) for node in functions))
    code += '#scope_file\n'+select_blocks(lambda p: 'sdk :: #library,system "'+p['sdk']+'";')
    temporary.write_text(code)
    run([args.formatter.resolve(),'--config',ROOT/'jaifmt.toml',temporary])
    outputs[OUT/'module.jai'] = temporary.read_text()
    provenance['native_profiles'] = {p['sdk']: p for p in PROFILES}
    for folder, body in [('rpmalloc-heaps', heap_module()), ('rpmalloc-callbacks', callbacks_module())]:
        temporary.write_text(body)
        run([args.formatter.resolve(), '--config', ROOT/'jaifmt.toml', temporary])
        outputs[ROOT/'stdlib/internal/bindings'/folder/'module.jai'] = temporary.read_text()
    outputs[OUT/'provenance.json'] = json.dumps(provenance,indent=2,sort_keys=True)+'\n'
    outputs[ROOT/'stdlib/internal/bindings/rpmalloc-heaps/public-layout.h'] = public_c_header(
        (ROOT/'docs/specs/stdlib/modules/rpmalloc.md').read_text())
    reviewed = (ROOT/'docs/specs/stdlib/modules/rpmalloc.md').read_text()
    probe, jai_checks = layout_probes(reviewed)
    raw_probe, raw_checks = layout_probes(reviewed, raw=True)
    _, raw_base_checks = layout_probes(reviewed, raw=True, public_heaps=False)
    outputs[ROOT/'stdlib/internal/bindings/rpmalloc-heaps/layout-probes.inc'] = '#if BINDING_RPMALLOC_RAW\n'+raw_probe+'#else\n'+probe+'#endif\n'
    temporary.write_text(jai_checks)
    run([args.formatter.resolve(),'--config',ROOT/'jaifmt.toml',temporary])
    outputs[ROOT/'tests/native-bindings/generated/rpmalloc-heap-facade-layout.jai'] = temporary.read_text()
    temporary.write_text(raw_checks)
    run([args.formatter.resolve(), '--config', ROOT/'jaifmt.toml', temporary])
    outputs[ROOT/'tests/native-bindings/generated/rpmalloc-raw-heap-facade-layout.jai'] = temporary.read_text()
    temporary.write_text(raw_base_checks)
    run([args.formatter.resolve(), '--config', ROOT/'jaifmt.toml', temporary])
    outputs[ROOT/'tests/native-bindings/generated/rpmalloc-raw-base-heap-facade-layout.jai'] = temporary.read_text()
    for raw, public_heaps, label in ((False, True, "rewrite"), (True, False, "raw-base"), (True, True, "raw")):
        _, assertions = layout_probes(reviewed, raw=raw, public_heaps=public_heaps, thread_cache=False)
        temporary.write_text(assertions)
        run([args.formatter.resolve(), "--config", ROOT/"jaifmt.toml", temporary])
        outputs[ROOT/"tests/native-bindings/generated"/("rpmalloc-no-thread-cache-"+label+"-layout.jai")] = temporary.read_text()
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
