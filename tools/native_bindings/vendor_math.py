#!/usr/bin/env python3
"""Select complete public SDK enum/macro definitions for a native math oracle."""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args = parser.parse_args()
    pin = json.loads((ROOT/'tools/native_bindings/dependencies.json').read_text())['windows-sdk']
    name = 'generation/WinSDK/RecompiledIdlHeaders/um/d3d12.h'
    header = ROOT/'.work/vendor/windows-sdk'/name
    data = header.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != pin['files'][name]:
        raise SystemExit('D3D12 SDK header hash mismatch')
    text = data.decode().replace('\r\n','\n')
    enums = ('D3D12_FILTER','D3D12_FILTER_REDUCTION_TYPE','D3D12_FILTER_TYPE','D3D12_SHADER_COMPONENT_MAPPING')
    definitions = []
    for enum in enums:
        definitions.append(re.search(r'typedef\s+enum '+enum+r'\s*\{[^{}]*\}\s*'+enum+r'\s*;',text)[0])
    for match in re.finditer(r'^#define(?:[^\n]*\\\n)*[^\n]*',text,re.M):
        definition = match[0]
        if re.search(r'D3D12_(?:SHADER_COMPONENT_MAPPING_|FILTER_REDUCTION_TYPE_|FILTER_TYPE_MASK|MIN_FILTER_SHIFT|MAG_FILTER_SHIFT|MIP_FILTER_SHIFT|ANISOTROPIC_FILTERING_BIT|ENCODE_|DECODE_|DEFAULT_SHADER_4_COMPONENT_MAPPING)',definition):
            definitions.append(definition)
    preamble = '// Selected complete public arithmetic definitions; no SDK functions are replaced.\n'
    preamble += '// Copyright Microsoft Corporation. Public SDK header SHA-256: '+digest+'\n'
    code = preamble+'#include <stdint.h>\n#include <stdlib.h>\n\n'+'\n\n'.join(definitions)+'\n\n'
    code += '''int32_t binding_components(int32_t a, int32_t b, int32_t c, int32_t d) {
    return D3D12_ENCODE_SHADER_4_COMPONENT_MAPPING(a,b,c,d);
}
int32_t binding_component(int32_t index, int32_t mapping) {
    return D3D12_DECODE_SHADER_4_COMPONENT_MAPPING(index,mapping);
}
int32_t binding_filter(int32_t a, int32_t b, int32_t c, int32_t d) {
    return D3D12_ENCODE_BASIC_FILTER(a,b,c,d);
}
int32_t binding_anisotropic(int32_t reduction) {
    return D3D12_ENCODE_ANISOTROPIC_FILTER(reduction);
}
int32_t binding_filter_part(int32_t filter, int32_t part) {
    switch (part) {
        case 0: return D3D12_DECODE_MIN_FILTER(filter);
        case 1: return D3D12_DECODE_MAG_FILTER(filter);
        case 2: return D3D12_DECODE_MIP_FILTER(filter);
        case 3: return D3D12_DECODE_FILTER_REDUCTION(filter);
        default: abort();
    }
}
int32_t binding_is_comparison(int32_t filter) { return D3D12_DECODE_IS_COMPARISON_FILTER(filter); }
int32_t binding_is_anisotropic(int32_t filter) { return D3D12_DECODE_IS_ANISOTROPIC_FILTER(filter); }
'''
    code = '\n'.join(line.rstrip() for line in code.splitlines())+'\n'
    path = ROOT/'tests/native-bindings/generated/d3d12_macro_probe.c'
    if args.check:
        if path.read_text() != code:
            raise SystemExit('Stale SDK arithmetic probe')
    else:
        path.parent.mkdir(parents=True,exist_ok=True)
        path.write_text(code)


if __name__ == '__main__':
    main()
