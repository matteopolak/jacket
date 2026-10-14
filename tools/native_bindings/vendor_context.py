#!/usr/bin/env python3
"""Private contextual ABI types from hash-pinned official native C headers.

No generated name here is a public Jai export. This fills native enum types
reachable in public records/signatures but absent from the public inventory.
"""
import hashlib
import json
from pathlib import Path
import re
import argparse
from generate import format_source

ROOT = Path(__file__).resolve().parents[2]


def c_enums(header):
    header = re.sub(r'/\*.*?\*/', '', header, flags=re.S)
    header = re.sub(r'//[^\n]*', '', header)
    blocks = []
    for match in re.finditer(r'typedef\s+enum(?:\s+\w+)?\s*\{([^{}]*)\}\s*(\w+)\s*;', header, re.S):
        body, name = match.groups()
        members = []
        for entry in body.split(','):
            entry = entry.strip()
            if not entry:
                continue
            entry = re.sub(r'(?<=\d)[uUlL]+\b', '', entry)
            entry = entry.replace('=', '::', 1)
            if not re.fullmatch(r'[\w\s:+~<>()|&*\-/]+', entry):
                raise ValueError('Unsupported C enum expression: ' + entry)
            members.append('    ' + entry + ';')
        code = name + ' :: enum ' + ('s32' if re.search(r'::\s*-', '\n'.join(members)) else 'u32') + ' {\n' + '\n'.join(members) + '\n}'
        if 'enum u32' in code:
            code = '#if OS == .WINDOWS {\n' + code.replace('enum u32', 'enum s32', 1) + '\n} else {\n' + code + '\n}'
        blocks.append(code)
    return '\n\n'.join(blocks) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--formatter', type=Path)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    manifest = json.loads((Path(__file__).with_name('dependencies.json')).read_text())
    header = ROOT/'.work/vendor/mojoshader/mojoshader.h'
    digest = hashlib.sha256(header.read_bytes()).hexdigest()
    if digest != manifest['mojoshader']['files']['mojoshader.h']:
        raise SystemExit('Pinned MojoShader header hash does not match')
    target = ROOT/'stdlib/internal/bindings/vendor/mojoshader-context.jai'
    target.parent.mkdir(parents=True, exist_ok=True)
    source = format_source('// Generated only from pinned official MojoShader C header.\n// SHA-256: ' + digest + '\n' + c_enums(header.read_text()), args.formatter)
    if args.check:
        if target.read_text() != source:
            raise SystemExit('Stale generated file: ' + str(target.relative_to(ROOT)))
    else:
        target.write_text(source)
    print(target.relative_to(ROOT))


if __name__ == '__main__':
    main()
