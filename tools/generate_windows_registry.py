#!/usr/bin/env python3
"""Generate reviewed registry bindings; unresolved pseudo-handle widths omitted."""
from pathlib import Path
import re
import subprocess
from portable_api_spec import parse_spec

ROOT = Path(__file__).resolve().parents[1]


def main():
    spec = parse_spec((ROOT/'docs/specs/stdlib/modules/windows-registry.md').read_text())
    groups = {0: [], 1: [], 2: []}
    exports = []
    for declaration in spec.declarations:
        name = declaration.name
        if name.startswith('HKEY_'):
            continue  # Reviewed zero-extension versus native SDK sign-extension question.
        code = declaration.code.rstrip(';')
        group = 0 if name in ['REGSAM', 'LSTATUS'] else 1 if '#foreign' in code else 2
        groups[group].append((name, code+';'))
    lines = ['#assert OS == .WINDOWS;']
    for group in groups.values():
        for name, code in sorted(group):
            lines.append(code)
            exports.append(name)
    lines += ['#scope_file', 'W :: #import "Windows";', 'HKEY :: W.HKEY;',
              'DWORD :: W.DWORD;', 'FILETIME :: W.FILETIME;',
              'advapi :: #system_library "advapi32";', '']
    private = ROOT/'stdlib/internal/windows-registry/abi.jai'
    private.parent.mkdir(parents=True, exist_ok=True)
    private.write_text('\n'.join(lines))
    facade = ROOT/'stdlib/Windows_Registry/module.jai'
    facade.parent.mkdir(parents=True, exist_ok=True)
    facade.write_text('#if OS == .WINDOWS {\n' + ''.join(f'{name} :: ABI.{name};\n' for name in exports)
                      + '}\n#scope_file\n#if OS == .WINDOWS ABI :: #import "internal/windows-registry/abi";\n')
    for flags in [[], ['--check']]:
        subprocess.run([str(ROOT/'target/jaifmt'), '--config', str(ROOT/'jaifmt.toml'),
                        *flags, str(private), str(facade)], check=True)
    print(f'Generated {len(exports)} real registry declarations;10 pseudo-handles pending ABI review')


if __name__ == '__main__':
    main()
