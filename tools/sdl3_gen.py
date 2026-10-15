#!/usr/bin/env python3
"""Generate SDL3's desktop public C API with Clang, from the verified pinned archive.
No third-party Python packages or native execution. See docs/extensions/sdl3.md.
"""
from __future__ import annotations
import argparse
import json
import re
import subprocess
import sdl3_coverage as coverage
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / 'stdlib/Extensions/SDL3'
TARGETS = {'macos-x64': 'x86_64-apple-macos10.12', 'macos-arm64': 'arm64-apple-macos11',
           'linux-x64': 'x86_64-unknown-linux-gnu', 'linux-arm64': 'aarch64-unknown-linux-gnu',
           'windows-x64': 'x86_64-pc-windows-msvc', 'windows-arm64': 'aarch64-pc-windows-msvc'}
HEADER = '''#define SDL_MAIN_HANDLED 1
#define SDL_DISABLE_OLD_NAMES 1
#define SDL_DISABLE_ALLOCA 1
#define SDL_DISABLE_ANALYZE_MACROS 1
#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>
#include <SDL3/SDL_vulkan.h>
'''
BUILTINS = {'void': 'void', '_Bool': 'bool', 'bool': 'bool', 'char': 'u8', 'signed char': 's8',
            'unsigned char': 'u8', 'short': 's16', 'unsigned short': 'u16', 'int': 's32',
            'unsigned int': 'u32', 'long long': 's64', 'unsigned long long': 'u64',
            'size_t': 'u64', 'intptr_t': 's64', 'uintptr_t': 'u64', 'float': 'float32', 'double': 'float64',
            'int8_t': 's8', 'uint8_t': 'u8', 'int16_t': 's16', 'uint16_t': 'u16',
            'int32_t': 's32', 'uint32_t': 'u32', 'int64_t': 's64', 'uint64_t': 'u64',
            'wchar_t': 's32', 'long': 's64', 'unsigned long': 'u64', 'tagMSG': 'tagMSG', '_XEvent': '_XEvent',
            'MSG': 'MSG', 'XEvent': 'XEvent'}


def callback_parameter_names(text, name):
    """Recover names erased by Clang's function-pointer type spelling.
    The parameter types still come entirely from the AST. C public callback prototypes
    use named scalar/pointer parameters, not nested callback declarators.
    """
    m = re.search(r'\*\s*'+re.escape(name)+r'\s*\)\s*\(([^()]*)\)',text,re.S)
    if not m:return None
    result=[]
    for param in split_args(m[1]):
        ident=re.search(r'([A-Za-z_]\w*)\s*(?:\[[^]]*\])?$',param)
        result.append(ident[1] if ident else None)
    return result


def named_callback(c, text, name):
    typ=jai_type(c)
    names=callback_parameter_names(text,name)
    if names:
        for i,n in enumerate(names):
            if n:typ=re.sub(r'\barg'+str(i)+r':',n+':',typ)
    return typ


def split_args(s):
    return [a.strip() for a in s.split(',')] if s and s != 'void' else []


def jai_type(c):
    c = re.sub(r'\s*__attribute__\(\((cdecl|stdcall)\)\)', '', c)
    c = re.sub(r'\b(const|volatile|restrict|__restrict|SDL_ANALYZER_NORETURN)\b', '', c)
    c = re.sub(r'\s+', ' ', c).strip()
    m = re.fullmatch(r'(.*?)\(\s*\*\s*\)\s*\((.*)\)', c)
    if m:
        args = ['args: ..Any' if a == '...' else f'arg{i}: {jai_type(a)}'
                for i, a in enumerate(split_args(m[2]))]
        return f'({", ".join(args)}) -> {jai_type(m[1].strip())} #c_call'
    m = re.fullmatch(r'(.*?)\[(\d+)\]', c)
    if m: return f'[{m[2]}] {jai_type(m[1])}'
    if c.endswith('*'): return '*' + jai_type(c[:-1].strip())
    c = re.sub(r'^(struct|union|enum) ', '', c)
    if c in BUILTINS: return BUILTINS[c]
    if re.fullmatch(r'(SDL_\w+|[SU]int(8|16|32|64)|Vk\w+)', c): return c
    raise ValueError(f'unsupported C type: {c}')


def value(n):
    if 'value' in n: return int(n['value'])
    for i in n.get('inner', []):
        v = value(i)
        if v is not None: return v
    return None


def enum_values(n):
    last = -1
    result = []
    for i in n.get('inner', []):
        if i['kind'] == 'EnumConstantDecl':
            v = value(i)
            last = last + 1 if v is None else v
            result.append((i['name'], last))
    return result


def run(cmd):
    p = subprocess.run(cmd, text=True, capture_output=True)
    if p.returncode: raise RuntimeError(p.stderr)
    return p.stdout


def system_headers(work):
    """Minimal ISO C declaration environment for AST-only cross-target extraction.
    Built-in Clang types define the target widths. No SDL declaration is replaced.
    No system SDK or platform library is needed to parse SDL's public headers.
    """
    d = work / 'iso'; d.mkdir(exist_ok=True)
    (d / 'stdint.h').write_text(''.join(f'typedef __{s}INT{bits}_TYPE__ {s.lower()}int{bits}_t;\n'
        for bits in (8,16,32,64) for s in ('','U')) +
        'typedef __INTPTR_TYPE__ intptr_t; typedef __UINTPTR_TYPE__ uintptr_t;\n' +
        '#define INT64_C(x) x##LL\n#define UINT64_C(x) x##ULL\n#define INT32_C(x) x\n#define UINT32_C(x) x##U\n')
    (d / 'stddef.h').write_text('typedef __SIZE_TYPE__ size_t; typedef __WCHAR_TYPE__ wchar_t;\n#define NULL ((void*)0)\n#define offsetof(t,m) __builtin_offsetof(t,m)\n')
    (d / 'string.h').write_text('#include <stddef.h>\n')
    (d / 'wchar.h').write_text('#include <stddef.h>\n')
    (d / 'stdarg.h').write_text('typedef __builtin_va_list va_list;\n')
    (d / 'stdbool.h').write_text('#define bool _Bool\n#define true 1\n#define false 0\n')
    (d / 'process.h').write_text('void _beginthreadex(void); void _endthreadex(void);\n')
    (d / 'intrin.h').write_text('unsigned char _BitScanReverse(unsigned long *, unsigned long); unsigned short _byteswap_ushort(unsigned short); unsigned long _byteswap_ulong(unsigned long); unsigned long long _byteswap_uint64(unsigned long long);\n')
    (d / 'endian.h').write_text('#define __LITTLE_ENDIAN 1234\n#define __BIG_ENDIAN 4321\n#define __BYTE_ORDER __LITTLE_ENDIAN\n')
    (d / 'inttypes.h').write_text('#include <stdint.h>\n')
    return d


def stable_c_type(ctype, src):
    """Canonical audit spelling, retaining anonymous record kind and source location.

    Clang versions differ in repeated record keywords in unnamed-type diagnostic names.
    This is metadata only; translation and layout probes use the actual AST/types.
    """
    if ctype is None:return None
    ctype=ctype.replace('\\','/')
    for prefix in ((src.resolve()/'include').as_posix()+'/',(src/'include').as_posix()+'/'):
        ctype=ctype.replace(prefix,'')
    return re.sub(r'\((unnamed|anonymous) (?:union|struct) at ',r'(\1 at ',ctype)

def header_profile_flags(plat):
    """Parse Windows for the native MSVC package, not Clang's extra GNU builtins.

    SDL_begin_code.h explicitly permits overriding SDL_HAS_BUILTIN. MSVC has
    neither __builtin_add_overflow nor __builtin_mul_overflow. SDL's own public
    portable inline helpers remain; no substitute helper implementation is added.
    """
    if plat not in TARGETS:raise ValueError(f'unsupported SDL3 platform: {plat}')
    return ['-U__clang__','-U__GNUC__','-DSDL_HAS_BUILTIN(x)=0'] if plat.startswith('windows') else []

def generate(src, plat, work, clang):
    def stable_type(ctype):return stable_c_type(ctype,src)
    BUILTINS.update({'wchar_t':'u16' if plat.startswith('windows') else 's32',
                     'long':'s32' if plat.startswith('windows') else 's64',
                     'unsigned long':'u32' if plat.startswith('windows') else 'u64'})
    inc = src / 'include'
    iso = system_headers(work)
    unit = work / 'api.c'; unit.write_text(HEADER)
    flags = [clang, '-target', TARGETS[plat], '-std=c11', '-nostdinc', '-I', str(iso), '-I', str(inc),
             '-Wno-pragma-pack', '-Wno-ignored-attributes',*header_profile_flags(plat)]
    # The MSVC ARM64 barrier branch includes arm64intr.h. Use the frontend's
    # actual public intrinsic header (enum/constants), never invented SDK values.
    resource=Path(run([clang,'-print-resource-dir']).strip())/'include'
    flags += ['-idirafter',str(resource)]
    # Apple's public platform detection needs TargetConditionals; desktop only.
    (iso / 'AvailabilityMacros.h').write_text('#define MAC_OS_X_VERSION_MIN_REQUIRED 101200\n')
    (iso / 'TargetConditionals.h').write_text('#define TARGET_OS_OSX 1\n#define TARGET_OS_MAC 1\n#define TARGET_OS_IPHONE 0\n#define TARGET_OS_SIMULATOR 0\n')
    def ast(text):
        unit.write_text(HEADER + text)
        dump_flags = ['-Xclang', '-ast-dump=json']
        if text.startswith('enum JAI_PROBE_'):
            dump_flags += ['-Xclang', '-ast-dump-filter=JAI_PROBE_']
            raw = run(flags + dump_flags + ['-fsyntax-only', str(unit)])
            decoder = json.JSONDecoder(); result = []
            while raw.strip():
                node, end = decoder.raw_decode(raw.lstrip()); result.append(node); raw = raw.lstrip()[end:]
            return result
        return json.loads(run(flags + dump_flags + ['-fsyntax-only', str(unit)]))['inner']
    nodes = ast('')
    current_file=''
    for node in nodes:
        loc=node.get('loc',{})
        current_file=loc.get('file',loc.get('expansionLoc',{}).get('file',current_file))
        node['_header']=Path(current_file).name
    unit.write_text(HEADER)
    preprocessed=run(flags + ['-E','-P',str(unit)])
    macro_text = run(flags + ['-dM', '-E', str(unit)])
    function_macro_names = set(re.findall(r'^#define (SDL_\w+)\(', macro_text, re.M))
    names = lambda n: n.get('name','').startswith(('SDL_', 'Uint', 'Sint', 'Vk')) or n.get('name') in {'MSG','tagMSG','XEvent','_XEvent'}
    nodes = [n for n in nodes if names(n)]
    records = {}
    enums = {}
    typedefs = {}
    functions = {}
    for n in nodes:
        dest = {'RecordDecl': records, 'EnumDecl': enums, 'TypedefDecl': typedefs, 'FunctionDecl': functions}.get(n['kind'])
        if dest is not None:
            if n['kind'] != 'RecordDecl' or n.get('completeDefinition') or n['name'] not in dest:
                dest[n['name']] = n
    lines = ['// Generated by tools/sdl3_gen.py from official SDL 3.4.18 public C headers.',
             '// Altered translation; original copyright/license retained in LICENSE.txt.',
             '// Do not edit. Regenerate all six targets together.\n']
    declarations=[]
    def emit(name,kind,text,header):
        declarations.append((coverage.ordering(name,kind,header),name,kind,header,text))
    checks = []
    def record(n, indent='', cname=None, path=''):
        out = []
        pending = None
        for f in n.get('inner', []):
            if f['kind'] == 'RecordDecl': pending = f
            if f['kind'] != 'FieldDecl': continue
            if f.get('isBitfield'): raise ValueError('bitfield ' + str(f))
            name = f['name']
            if 'unnamed ' in f['type']['qualType']:
                assert pending is not None
                typ = pending['tagUsed'] + ' {\n' + '\n'.join(record(pending, indent+'    ', cname, path+name+'.')) + '\n' + indent + '    }'
                pending = None
            else:
                typ = jai_type(f['type']['qualType'])
                if '(*' in f['type']['qualType']:
                    start=re.search(r'(?:struct|union)\s+'+re.escape(n.get('name',''))+r'\s*\{',preprocessed) if n.get('name') else None
                    source=preprocessed[start.start():] if start else ''
                    typ=named_callback(f['type']['qualType'],source,name)
            jai_name = '`'+name if name in {'remove', 'using', 'ifx'} else name
            out.append(indent + f'    {jai_name}: {typ};')
            if cname: checks.append((f'__builtin_offsetof({cname}, {path}{name})', f'offset_of({cname}, \"{path}{name}\")'))
        return out
    for name,n in records.items():
        emit(name,'record','\n'.join([f'{name} :: {n["tagUsed"]} {{', *record(n, cname=name if n.get('completeDefinition') and name in typedefs else None), '}']),n['_header'])
        if n.get('completeDefinition') and name in typedefs:
            checks += [(f'sizeof({name})', f'size_of({name})'), (f'_Alignof({name})', f'align_of({name})')]
    for name,n in enums.items():
        vs = enum_values(n)
        base = 's32' if any(v<0 for _,v in vs) else 'u32'
        emit(name,'enum','\n'.join([f'using {name} :: enum {base} {{', *[f'    {k} :: {v};' for k,v in vs], '}']),n['_header'])
        checks += [(f'sizeof({name})', f'size_of({name})'), (f'_Alignof({name})', f'align_of({name})')]
    excluded = {}
    for name,n in typedefs.items():
        if name in records or name in enums: continue
        t = n['type']['qualType']
        emit(name,'callback' if '(*' in t else 'alias',f'{name} :: {named_callback(t,preprocessed,name)};',n['_header'])
        tag=re.sub(r'^(struct|union) ', '', t)
        if tag not in records or records[tag].get('completeDefinition'):
            checks += [(f'sizeof({name})',f'size_of({name})'),(f'_Alignof({name})',f'align_of({name})')]
    glue = []
    for name,n in functions.items():
        if name == 'SDL_main':
            excluded[name] = 'application-supplied entry point, not an SDL library export'; continue
        params = [i for i in n.get('inner',[]) if i['kind']=='ParmVarDecl']
        if any('va_list' in p['type']['qualType'] for p in params):
            excluded[name] = 'C va_list is platform-specific; use the corresponding variadic API'; continue
        ret = n['type']['qualType'].split('(')[0].strip()
        args = [f'{p.get("name", "arg"+str(i))}: {jai_type(p["type"]["qualType"])}' for i,p in enumerate(params)]
        if n.get('variadic'): args += ['args: ..Any']
        symbol = name
        if n.get('inline') or name in function_macro_names and not n.get('variadic'):
            symbol = 'JAI_SDL3_' + name
            cargs = ', '.join(p['type']['qualType'] + ' ' + p.get('name', 'arg'+str(i)) for i,p in enumerate(params)) or 'void'
            callargs = ', '.join(p.get('name', 'arg'+str(i)) for i,p in enumerate(params))
            glue.append(f'SDL_DECLSPEC {ret} SDLCALL {symbol}({cargs}) {{ '+('' if ret=='void' else 'return ')+f'{name}({callargs}); }}')
        foreign = f' #foreign SDL3' + (f' "{symbol}"' if symbol != name else '')
        emit(name,'procedure',f'{name} :: ({", ".join(args)}) -> {jai_type(ret)}{foreign};',n['_header'])
    # Evaluate all public object-like value macros in the target C frontend. Ignore only
    # explicitly inventoried compiler annotations/configuration/old-name poison macros.
    unit.write_text(HEADER)
    macros = run(flags + ['-dM','-E',str(unit)])
    strings, integers, macro_excluded, functional, floats = {}, [], {}, {}, {}
    declared = set(records)|set(enums)|set(typedefs)|set(functions)
    for line in macros.splitlines():
        m = re.match(r'#define (SDL_\w+|SDLK_\w+)(\([^ ]*\))?\s*(.*)', line)
        if not m: continue
        name,args,expr = m.groups()
        if args: functional[name] = expr; continue
        if name in declared: continue
        if re.fullmatch(r'"(?:[^"\\]|\\.)*"',expr): strings[name] = expr; continue
        if name in {'SDL_FILE','SDL_ASSERT_FILE','SDL_FUNCTION','SDL_LINE'}:
            macro_excluded[name]='call-site dependent preprocessor macro'; continue
        if not expr or name.endswith('_h_') or any(x in name for x in ('DECLSPEC','DEPRECATED','INLINE','RESTRICT','ANALYZE','PRINTF','SCANF','NORETURN','SDLCALL','MAIN_HANDLED','DISABLE_','PLATFORM_')):
            macro_excluded[name] = 'preprocessor configuration, annotation or header guard'; continue
        if expr.startswith('SDL_') and ('_renamed_' in expr or '_removed' in expr):
            macro_excluded[name] = 'removed SDL2 name poison macro'; continue
        integers.append((name,expr))
    # Determine evaluability without guessing expression semantics. Failures are inventoried.
    def numeric(batch):
        try:
            ns=ast('\n'.join(f'enum JAI_PROBE_{i} {{ JAI_CONST_{i} = ({name}) }};' for i,(name,_) in enumerate(batch)))
            return {batch[int(n['name'][10:])][0]:v for e in ns if e['kind']=='EnumDecl'
                    for n in e.get('inner',[]) if n.get('name','').startswith('JAI_CONST_')
                    for v in [value(n)]}
        except RuntimeError:
            if len(batch)==1:
                name,expr=batch[0]
                unit.write_text(HEADER+'\nJAI_EXPR '+name+'\n')
                expanded=run(flags+['-E','-P',str(unit)]).rsplit('JAI_EXPR',1)[1].strip()
                if re.fullmatch(r'(?:"(?:[^"\\]|\\.)*"\s*)+',expanded):
                    # SDL strings use ordinary C escapes, which JSON also accepts here.
                    strings[name]=json.dumps(''.join(json.loads(q) for q in re.findall(r'"(?:[^"\\]|\\.)*"',expanded)))
                elif re.fullmatch(r'[+-]?[0-9]+(?:\.[0-9]*)?(?:[eE][+-]?[0-9]+)?[fF]?',expanded):
                    floats[name]=(expanded.rstrip('fF'),'float32' if expanded[-1:] in ('f','F') else 'float64')
                else: macro_excluded[name] = 'non-value preprocessor surface: '+expr
                return {}
            mid=len(batch)//2; return {**numeric(batch[:mid]),**numeric(batch[mid:])}
    consts=numeric(integers)
    macro_headers={}
    for header in sorted((src/'include/SDL3').glob('*.h')):
        for match in re.finditer(r'^\s*#\s*define\s+(\w+)',header.read_text(),re.M):macro_headers[match[1]]=header.name
    for k,v in sorted(consts.items()):emit(k,'constant',f'{k} :: {v};',macro_headers.get(k,''))
    for k,(v,t) in sorted(floats.items()):emit(k,'constant',f'{k} :: cast({t}) {v};',macro_headers.get(k,''))
    for k,v in sorted(strings.items()):emit(k,'constant',f'{k} :: {v};',macro_headers.get(k,''))
    active_group=None
    for _,name,kind,header,text in sorted(declarations):
        responsibility=coverage.group(header)
        if responsibility!=active_group:
            lines += ['', '// '+responsibility.capitalize()+' — values and types, then lifecycle operations.']
            active_group=responsibility
        lines += [text,'']
    # Cross-target ABI evidence generated by Clang, consumed by Jai on that target.
    ns=ast('\n'.join(f'enum JAI_PROBE_{i} {{ JAI_LAYOUT_{i} = {c} }};' for i,(c,_) in enumerate(checks)))
    layouts={int(n['name'][11:]):value(n) for e in ns if e['kind']=='EnumDecl' for n in e.get('inner',[]) if n.get('name','').startswith('JAI_LAYOUT_')}
    assertions=['// Generated C layout checks; include in the native ABI test.']
    assertions += [f'#assert(size_of(*{name}) == 8);' for name in typedefs if name not in records and name not in enums]
    assertions += [f'#assert(size_of(type_of({name})) == 8);' for name in functions if name not in excluded]
    assertions += [f'#assert({j} == {layouts[i]});' for i,(_,j) in enumerate(checks)]
    c_assertions=HEADER+'\n#include <stddef.h>\n'+'\n'.join(f'_Static_assert(({c.replace("__builtin_offsetof","offsetof")}) == {layouts[i]}, "SDL3 C layout {i}");' for i,(c,_) in enumerate(checks))+'\n'
    # Compile every emitted call in the same header feature profile used for its
    # AST. A wrapper may never silently introduce an implicit external reference.
    unit.write_text(HEADER+'\n'.join(glue)+'\n')
    run(flags+['-Werror=implicit-function-declaration','-Werror=incompatible-pointer-types','-fsyntax-only',str(unit)])
    return '\n'.join(lines).rstrip()+'\n', '\n'.join(assertions)+'\n', c_assertions, glue, {
        'header_profile':{'parser':'Clang C11','target':TARGETS[plat],
                          'feature_flags':header_profile_flags(plat)},
        'ast_declarations': {n['kind']+':'+n['name']:{'name':n['name'],'kind':n['kind'],'header':n['_header'],'c_type':stable_type(n.get('type',{}).get('qualType')), 'parameters':[{'name':f.get('name'),'c_type':stable_type(f.get('type',{}).get('qualType'))} for f in n.get('inner',[]) if f['kind']=='ParmVarDecl'], 'fields':[{'name':f.get('name'),'c_type':stable_type(f.get('type',{}).get('qualType'))} for f in n.get('inner',[]) if f['kind']=='FieldDecl']} for n in [*records.values(),*enums.values(),*typedefs.values(),*functions.values()]},
        'layout_checks':[{'c':c,'jai':j,'expected':layouts[i]} for i,(c,j) in enumerate(checks)],
        'enum_members':{k:name for name,n in enums.items() for k,_ in enum_values(n)},
        'functions': sorted(set(functions)-set(excluded)), 'excluded_functions': excluded,
        'records': sorted(records), 'enums': sorted(enums), 'typedefs': sorted(typedefs),
        'integer_macros': sorted(consts), 'float_macros': sorted(floats), 'string_macros': sorted(strings),
        'excluded_object_macros': macro_excluded, 'function_macros': functional,
        'wrapped_function_macros': sorted(set(functional)&(set(functions)|{'SDL_INIT_INTERFACE'}|set(json.loads((ROOT/'tools/native/sdl3/macros.json').read_text())))),
        'unsupported_function_macros': sorted(set(functional)-set(functions)-{'SDL_INIT_INTERFACE'}-set(json.loads((ROOT/'tools/native/sdl3/macros.json').read_text()))),
        'layout_assertions': len(checks)}


def native_notices(src):
    notices={}
    for path in sorted((src/'src').rglob('*')):
        if path.suffix not in {'.c','.h','.cpp','.m','.mm'}:continue
        for comment in re.findall(r'/\*.*?\*/',path.read_text(errors='replace'),re.S):
            if re.search(r'copyright|permission to|license|public domain',comment,re.I):
                notices.setdefault(comment,[]).append(str(path.relative_to(src)))
    return 'Original notices from pinned SDL3 native sources (including optional backends).\nHIDAPI is used under its BSD-style license.\n\n'+'\n\n'.join(', '.join(paths)+'\n'+comment for comment,paths in notices.items())+'\n'

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--source', type=Path, required=True, help='verified extracted SDL3-3.4.18 source')
    p.add_argument('--clang',default='clang')
    p.add_argument('--check',action='store_true')
    p.add_argument('--jaifmt',type=Path,help='source-built repository jaifmt; normalize every generated Jai output using jaifmt.toml')
    a=p.parse_args()
    # Accept only headers from the pinned archive, not an arbitrary system installation.
    import hashlib, tarfile
    archive=a.source.parent/'SDL3-3.4.18.tar.gz'
    expected=json.loads((ROOT/'tools/native-libs.json').read_text())['sources']['sdl3']['archive']['sha256']
    if hashlib.sha256(archive.read_bytes()).hexdigest()!=expected: raise ValueError('source archive hash mismatch')
    with tarfile.open(archive) as t:
        for member in t.getmembers():
            if '/include/SDL3/' in member.name and member.isfile():
                rel=Path(member.name).relative_to('SDL3-3.4.18')
                if (a.source/rel).read_bytes()!=t.extractfile(member).read(): raise ValueError(f'header differs from pin: {rel}')
    work=ROOT/'artifacts/sdl3/generator'; work.mkdir(parents=True,exist_ok=True)
    inventory={}
    equivalents={}
    files={}
    files[OUT/'THIRD-PARTY-NOTICES.txt']=native_notices(a.source)
    notices=[]
    for header in sorted((a.source/'include/SDL3').glob('*.h')):
        text=header.read_text()
        match=re.match(r'/\*.*?\*/',text,re.S)
        if match: notices.append(header.name+'\n'+match[0])
    files[OUT/'NOTICES.txt']='Original notices from pinned SDL3 public headers; translations are altered source.\n\n'+'\n\n'.join(notices)+'\n'
    specs=json.loads((ROOT/'tools/native/sdl3/macros.json').read_text())
    c=['// Altered translation of official SDL3 macros. Copyright/license: stdlib/Extensions/SDL3/LICENSE.txt', '#include <SDL3/SDL.h>']
    j=['// Typed wrappers of official public SDL3 macros. No replacement semantics.']
    macro_headers={}
    for header in sorted((a.source/'include/SDL3').glob('*.h')):
        for match in re.finditer(r'^[ \t]*#[ \t]*define[ \t]+(\w+)',header.read_text(),re.M):macro_headers[match[1]]=header.name
    macro_declarations=[]
    for n,(ret,ps) in specs.items():
        cparams=', '.join(t+' '+('signed_' if name=='signed' else name) for t,name in ps) or 'void'
        call=', '.join('signed_' if name=='signed' else name for t,name in ps)
        c.append(f'SDL_DECLSPEC {ret} SDLCALL JAI_SDL3_{n}({cparams}) {{ return {n}({call}); }}')
        text=n+' :: ('+', '.join(name+': '+jai_type(t) for t,name in ps)+') -> '+jai_type(ret)+' #foreign SDL3 "JAI_SDL3_'+n+'";'
        macro_declarations.append((coverage.ordering(n,'procedure',macro_headers[n]),text))
    for index,typ in enumerate(('SDL_IOStreamInterface','SDL_StorageInterface','SDL_VirtualJoystickDesc')):
        c.append(f'SDL_DECLSPEC void SDLCALL JAI_SDL3_SDL_INIT_INTERFACE_{typ}({typ} *iface) {{ SDL_INIT_INTERFACE(iface); }}')
        text=f'SDL_INIT_INTERFACE :: (iface: *{typ}) -> void #foreign SDL3 "JAI_SDL3_SDL_INIT_INTERFACE_{typ}";'
        macro_declarations.append(((*coverage.ordering('SDL_INIT_INTERFACE','procedure','SDL_stdinc.h'),index),text))
    j += [text for _,text in sorted(macro_declarations,key=lambda entry:entry[0])]
    files[ROOT/'tools/native/sdl3/macros.c']='\n'.join(c)+'\n'
    files[OUT/'macros.jai']='\n'.join(j)+'\n'
    for plat in TARGETS:
        data,checks,c_checks,glue,report=generate(a.source,plat,work,a.clang)
        files[OUT/f'{plat}.jai']=data
        files[ROOT/f'tests/sdl3/layout-{plat}.jai']=checks
        files[ROOT/f'tests/sdl3/c-layout-{plat}.h']=c_checks
        files[ROOT/f'tools/native/sdl3/inline-{plat}.c']='// Generated altered translation. SDL copyright/license: stdlib/Extensions/SDL3/LICENSE.txt\n'+HEADER+'\n'.join(glue)+'\n'
        contracts=[json.dumps({'format':1,'profile':plat,'source_version':'3.4.18'},sort_keys=True)]
        contracts += [json.dumps({'declaration':key,**entry},sort_keys=True) for key,entry in sorted(report.pop('ast_declarations').items())]
        contracts += [json.dumps({'layout':entry['c'],'jai':entry['jai'],'expected':entry['expected']},sort_keys=True) for entry in report.pop('layout_checks')]
        report['contracts']=f'contracts-{plat}.jsonl'
        files[OUT/report['contracts']]='\n'.join(contracts)+'\n'
        report['contracts_sha256']=hashlib.sha256(files[OUT/report['contracts']].encode()).hexdigest()
        inventory[plat]=report
        exports={m[1]:f'{plat}.jai::{m[1]}' for m in re.finditer(r'^(?:using )?(\w+) ::',data,re.M)}
        exports.update({k:f'{plat}.jai::{enum}.{k}' for k,enum in report['enum_members'].items()})
        exports.update({m[1]:f'macros.jai::{m[1]}' for m in re.finditer(r'^(\w+) ::',files[OUT/'macros.jai'],re.M)})
        equivalents[plat]=exports
        print(plat,len(report['functions']),'procedures,',report['layout_assertions'],'layout assertions')
    (work/'reports.json').write_text(json.dumps(inventory))
    (work/'equivalents.json').write_text(json.dumps(equivalents))
    ledger=coverage.reconcile(a.source,inventory,equivalents)
    ledger['source']=json.loads((ROOT/'tools/native-libs.json').read_text())['sources']['sdl3']
    files[OUT/'coverage.json']=json.dumps(ledger,indent=2,sort_keys=True)+'\n'
    files[OUT/'inventory.json']=json.dumps(inventory,indent=2,sort_keys=True)+'\n'
    for path,data in files.items():
        if a.jaifmt and path.suffix=='.jai':
            result=subprocess.run([str(a.jaifmt.resolve()),'--stdin','--config',str(ROOT/'jaifmt.toml')],input=data,text=True,capture_output=True,check=True)
            data=result.stdout
            again=subprocess.run([str(a.jaifmt.resolve()),'--stdin','--config',str(ROOT/'jaifmt.toml')],input=data,text=True,capture_output=True,check=True)
            if again.stdout!=data:raise ValueError(f'non-idempotent jaifmt: {path}')
        if a.check:
            if path.read_text()!=data: raise ValueError(f'stale generated file: {path}')
        elif not path.exists() or path.read_text()!=data: path.write_text(data)

if __name__=='__main__':main()
