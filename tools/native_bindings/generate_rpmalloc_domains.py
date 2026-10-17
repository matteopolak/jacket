#!/usr/bin/env python3
"""Derive complete private domain globals from the pinned official C SDK."""
import argparse
import difflib
import json
from pathlib import Path
import re
import subprocess

from prepare_rpmalloc import ROOT, prepare


def generate(vendor, work):
    prepared = prepare(vendor, work/'base', stop_before='sdk-domains.patch')
    source = (prepared/'rpmalloc.c').read_text()
    start = source.index('//! Initialized flag\n')
    end = source.index('////////////\n///\n/// Thread local heap and ID', start)
    region = source[start:end]
    names = sorted(set(re.findall(r'^static\s+[^;]*?\b(_\w+)\s*(?:\[[^;]*?\])?;', region, re.M)))
    if len(names) != 35:
        raise ValueError('Pinned SDK global family changed: '+repr(names))
    fields = re.sub(r'^static ', '', region, flags=re.M)
    for name in sorted(names, key=len, reverse=True):
        fields = re.sub(r'\b'+name+r'\b', 'sdk'+name, fields)
    fields += '#if defined(_WIN32) && (!defined(BUILD_DYNAMIC_LINK) || !BUILD_DYNAMIC_LINK)\nDWORD sdk_fls_key;\n#endif\n'
    aliases = ''.join('#define '+name+' (binding_domain_current()->sdk'+name+')\n' for name in names)
    aliases += '#if defined(_WIN32) && (!defined(BUILD_DYNAMIC_LINK) || !BUILD_DYNAMIC_LINK)\n#define fls_key (binding_domain_current()->sdk_fls_key)\n#endif\n'
    body = source[:start]+'#if !BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n'+region+'#else\n#include "runtime-domain.inc"\n#endif\n\n'+source[end:]
    body = body.replace('static DWORD fls_key;', '#if !BINDING_RPMALLOC_RUNTIME_CONFIGURATION\nstatic DWORD fls_key;\n#endif')
    body = body.replace('static _Thread_local heap_t* _memory_thread_heap TLS_MODEL;',
        '#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n#define _memory_thread_heap (*binding_domain_heap_slot())\n#else\nstatic _Thread_local heap_t* _memory_thread_heap TLS_MODEL;\n#endif')
    body = body.replace('fls_key = FlsAlloc(&_rpmalloc_thread_destructor);',
        '#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n\tfls_key = binding_domain_fls_allocate();\n\tif (fls_key == FLS_OUT_OF_INDEXES)\n\t\treturn ENOMEM;\n#else\n\tfls_key = FlsAlloc(&_rpmalloc_thread_destructor);\n#endif')
    body=body.replace('\tFlsFree(fls_key);', '#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n\tbinding_domain_fls_free();\n#else\n\tFlsFree(fls_key);\n#endif')
    body=body.replace('\trpmalloc_thread_finalize(1);\n\t//rpmalloc_dump_statistics(stdout);',
        '#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n\tbinding_domain_current()->owned_finalizing = 1;\n#endif\n\trpmalloc_thread_finalize(1);\n\t//rpmalloc_dump_statistics(stdout);')
    body=body.replace('\t_rpmalloc_initialized = 0;', '\t_rpmalloc_initialized = 0;\n#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n\tbinding_domain_current()->owned_finalizing = 0;\n#endif')
    for value in ('heap', '0'):
        old='FlsSetValue(fls_key, '+value+');'
        body=body.replace(old, '#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n\tbinding_domain_fls_heap('+value+');\n#else\n\t'+old+'\n#endif')
    # Audit every native host conditional separately; no implicit missing field
    # is filled with guessed storage. Windows FLS is separately source-guarded.
    original=vendor/'rpmalloc/rpmalloc.c'
    for statistics, global_cache, adaptive in ((1,1,0),(0,1,0),(0,1,1),(1,0,0),(0,0,0)):
        command=['clang','-std=c11','-DRPMALLOC_FIRST_CLASS_HEAPS=1','-DRPMALLOC_CONFIGURABLE=1',
                 '-DENABLE_STATISTICS='+str(statistics),'-DENABLE_GLOBAL_CACHE='+str(global_cache),
                 '-DENABLE_ADAPTIVE_THREAD_CACHE='+str(adaptive),'-Xclang','-ast-dump=json','-fsyntax-only',str(original)]
        ast=json.loads(subprocess.run(command,check=True,capture_output=True,text=True).stdout)
        actual={n['name'] for n in ast['inner'] if n['kind']=='VarDecl' and not n.get('loc',{}).get('includedFrom')}
        missing=actual-set(names)-{'_memory_thread_heap'}
        if missing:
            raise ValueError('Unowned SDK global state: '+repr(missing))
    folder=ROOT/'stdlib/internal/bindings/rpmalloc-heaps'
    outputs={folder/'sdk-domains.patch':''.join(difflib.unified_diff(source.splitlines(True),body.splitlines(True),
        fromfile='a/rpmalloc.c',tofile='b/rpmalloc.c')),
        folder/'runtime-domain-native-fields.inc':'/* Generated complete pinned SDK globals; preserve conditional fields. */\n'+fields,
        folder/'runtime-domain-native-aliases.inc':'/* Generated SDK state indirection; never a public ABI record. */\n'+aliases}
    return outputs


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--vendor',type=Path,required=True)
    parser.add_argument('--work',type=Path,required=True)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    stale=[]
    for path,text in generate(args.vendor,args.work).items():
        if args.check:
            if not path.exists() or path.read_text()!=text:
                stale.append(str(path.relative_to(ROOT)))
        else:
            path.write_text(text)
    print('Stale: '+repr(stale) if stale else 'Complete conditional SDK domain state verified')
    return int(bool(stale))


if __name__=='__main__':
    raise SystemExit(main())
