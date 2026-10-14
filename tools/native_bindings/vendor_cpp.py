#!/usr/bin/env python3
"""Translate the NVTT member API expressly delegated to pinned vendor headers.

Clang supplies C++ mangling; this generator never invents linker spellings.
Existing reviewed Jai overloads, names and defaults take precedence.
"""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT/'stdlib/internal/bindings/vendor/nvtt-members.json'


def jai_type(native):
    native = re.sub(r'\b(?:const|volatile|nvtt::)\b', '', native).strip()
    if native.endswith(('*', '&')):
        return '*' + jai_type(native[:-1])
    return {'int':'s32', 'unsigned int':'u32', 'unsigned char':'u8',
            'float':'float', 'char':'u8', 'bool':'bool', 'void':'void'}.get(native, native)


def default_value(parameter, header):
    if 'init' not in parameter:
        return ''
    begin, end = parameter['range']['begin'], parameter['range']['end']
    text = header[begin['offset']:end['offset']+end['tokLen']]
    value = text.split('=', 1)[1].strip()
    if parameter['type']['qualType'].endswith(('*', '&')) and value == '0':
        value = 'null'
    elif parameter['type']['qualType'] == 'bool' and value in ('0', '1'):
        value = 'true' if value == '1' else 'false'
    else:
        value = re.sub(r'(?<=\d)[fF]$', '', value)
        value = re.sub(r'\.f$', '.0', value)
    return ' = ' + value


def main():
    pin = json.loads((ROOT/'tools/native_bindings/dependencies.json').read_text())['nvtt']
    path = ROOT/'.work/vendor/nvtt/src/nvtt/nvtt.h'
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != pin['files']['src/nvtt/nvtt.h']:
        raise SystemExit('NVTT header hash mismatch')
    header = data.decode()
    result = {'header_sha256': digest, 'revision': pin['revision'], 'targets': {}}
    targets = {'WINDOWS':'x86_64-pc-windows-msvc', 'LINUX':'x86_64-unknown-linux-gnu',
               'MACOS':'x86_64-apple-macos11'}
    for target, triple in targets.items():
        process = subprocess.run(['clang++','-x','c++','-std=c++11','-target',triple,
                                  '-Wno-pragma-once-outside-header','-Xclang','-ast-dump=json',
                                  '-fsyntax-only',str(path)], check=True, capture_output=True, text=True)
        ast = json.loads(process.stdout)
        namespace = next(n for n in ast['inner'] if n.get('name') == 'nvtt')
        classes = {}
        for node in namespace['inner']:
            owner = node.get('name')
            if owner not in ('Compressor','Surface','CubeSurface') or not node.get('completeDefinition'):
                continue
            members = []
            for method in node['inner']:
                if method['kind'] != 'CXXMethodDecl' or method.get('isImplicit'):
                    continue
                if method['name'] == 'operator=':
                    continue  # This public free overload is already reviewed.
                parameters = [p for p in method.get('inner',[]) if p['kind'] == 'ParmVarDecl']
                params = ['this: *'+owner]
                params += [p['name']+': '+jai_type(p['type']['qualType'])+default_value(p,header) for p in parameters]
                returns = jai_type(method['type']['qualType'].split('(',1)[0].strip())
                symbol = method['mangledName']
                if target == 'MACOS':
                    symbol = symbol.removeprefix('_')  # Mach-O external prefix is not a dlsym name.
                directive = ' #cpp_return_type_is_non_pod' if returns in ('Surface','CubeSurface') else ''
                code = method['name']+' :: ('+', '.join(params)+') -> '+returns+' #cpp_method'+directive+' #foreign libnvtt "'+symbol+'"'
                members.append(code)
            classes[owner] = members
        result['targets'][target] = classes
    OUTPUT.parent.mkdir(parents=True,exist_ok=True)
    OUTPUT.write_text(json.dumps(result,indent=2)+'\n')
    print('Generated native NVTT methods:', {t:sum(map(len,v.values())) for t,v in result['targets'].items()})


if __name__ == '__main__':
    main()
