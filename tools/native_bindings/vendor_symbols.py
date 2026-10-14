#!/usr/bin/env python3
"""Ask Clang for the symbol omitted by the nested ImGui TextRange inventory.

The symbol-only declaration is a public native method signature, with opaque
scope/template declarations. It defines no SDK function and is never linked or
executed. A full-header host AST validates its mangling against the real header.
"""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def ast(path, target=None):
    command = ['clang++','-x','c++','-std=c++17','-Wno-pragma-once-outside-header',
               '-Xclang','-ast-dump=json','-fsyntax-only',str(path)]
    if target:
        command += ['-target',target]
    return json.loads(subprocess.run(command,check=True,capture_output=True,text=True).stdout)


def symbol(tree):
    owner = next(n for n in tree['inner'] if n.get('name')=='ImGuiTextFilter' and n.get('completeDefinition'))
    nested = next(n for n in owner['inner'] if n.get('name')=='ImGuiTextRange' and n.get('completeDefinition'))
    return next(n['mangledName'] for n in nested['inner'] if n.get('name')=='split')


def main():
    pin = json.loads((ROOT/'tools/native_bindings/dependencies.json').read_text())['imgui']
    header = ROOT/'.work/vendor'/pin['archive']['root']/'imgui.h'
    digest = hashlib.sha256(header.read_bytes()).hexdigest()
    if digest != pin['files']['imgui.h']:
        raise SystemExit('ImGui header hash mismatch')
    native = ast(header)
    # These native names and the exact signature come from the pinned public
    # header. Forward declarations supply only C++ name scopes to the mangler.
    declaration = '''template<typename T> struct ImVector;
struct ImGuiTextFilter {
    struct ImGuiTextRange {
        void split(char separator, ImVector<ImGuiTextRange>* out) const;
    };
};
'''
    path = ROOT/'.work/vendor/imgui/text-range-symbol.cpp'
    path.write_text(declaration)
    if symbol(ast(path)) != symbol(native):
        raise SystemExit('Symbol-only declaration differs from the full public header')
    targets = {'windows-x64':'x86_64-pc-windows-msvc',
               'windows-arm64':'aarch64-pc-windows-msvc',
               'linux-x64':'x86_64-unknown-linux-gnu',
               'linux-arm64':'aarch64-unknown-linux-gnu',
               'macos-x64':'x86_64-apple-macos11',
               'macos-arm64':'arm64-apple-macos11'}
    symbols = {}
    for target, triple in targets.items():
        value = symbol(ast(path,triple))
        symbols[target] = value.removeprefix('_') if target.startswith('macos') else value
    if symbols['windows-x64'] != symbols['windows-arm64'] or len({v for k,v in symbols.items() if not k.startswith('windows')}) != 1:
        raise SystemExit('This symbol requires separate target branches')
    output = ROOT/'stdlib/internal/bindings/vendor/imgui-symbols.json'
    output.write_text(json.dumps({'header_sha256':digest,'signature_source':declaration,'symbols':symbols},indent=2)+'\n')
    print(symbols)


if __name__ == '__main__':
    main()
