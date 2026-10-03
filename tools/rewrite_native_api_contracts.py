#!/usr/bin/env python3
"""Normalize read-only API contracts without retaining procedure implementations.

This utility is a source authoring aid, not a compiler or native build tool.  Its
output deliberately distinguishes existing foreign declarations from missing
independently authored adapters.  It never executes input or opens a library.
"""
from __future__ import annotations

from dataclasses import dataclass
import argparse
import hashlib
import json
from pathlib import Path
import re


@dataclass(frozen=True)
class Token:
    value: str
    line: int


LEXEME = re.compile(
    r'#[A-Za-z_][A-Za-z_0-9]*|[A-Za-z_][A-Za-z_0-9]*|'
    r'0[xXhHbB][A-Za-z_0-9]+|[0-9][0-9_]*(?:\.[0-9_]+)?(?:[eE][+-]?[0-9_]+)?|'
    r'@[A-Za-z_][A-Za-z_0-9]*|\$\$|---|\.\{|\.\.\.|\.\.|::|:=|->|==|!=|<=|>=|<<|>>|&&|\|\||\+=|-=|\*=|/=|%=|&=|\|=|\^=|\S'
)


def tokenize(source: str) -> list[Token]:
    result: list[Token] = []
    cursor, line = 0, 1
    while cursor < len(source):
        if source[cursor].isspace():
            line += source[cursor] == '\n'
            cursor += 1
        elif source.startswith('//', cursor):
            stop = source.find('\n', cursor)
            cursor = len(source) if stop < 0 else stop
        elif source.startswith('/*', cursor):
            depth = 1
            cursor += 2
            while cursor < len(source) and depth:
                if source.startswith('/*', cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith('*/', cursor):
                    depth -= 1
                    cursor += 2
                else:
                    line += source[cursor] == '\n'
                    cursor += 1
        elif source[cursor] == '"':
            start, start_line = cursor, line
            cursor += 1
            while cursor < len(source):
                if source[cursor] == '\\':
                    cursor += 2
                elif source[cursor] == '"':
                    cursor += 1
                    break
                else:
                    line += source[cursor] == '\n'
                    cursor += 1
            result.append(Token(source[start:cursor], start_line))
        else:
            match = LEXEME.match(source, cursor)
            assert match is not None
            value = match.group()
            result.append(Token(value, line))
            cursor = match.end()
            if value == '#string':
                delimiter = re.match(r'[ \t]+([A-Za-z_][A-Za-z_0-9]*)', source[cursor:])
                if delimiter:
                    marker = delimiter.group(1)
                    closing = re.search(r'(?m)^[ \t]*' + re.escape(marker) + r'[ \t]*\r?$', source[cursor + delimiter.end():])
                    stop = len(source) if closing is None else cursor + delimiter.end() + closing.end()
                    line += source[cursor:stop].count('\n')
                    cursor = stop
                    result[-1] = Token('""', result[-1].line)
    return result


def balanced_end(tokens: list[Token], start: int, left: str, right: str) -> int:
    depth = 0
    for index in range(start, len(tokens)):
        value = tokens[index].value
        if value == left or (left == '{' and value == '.{'):
            depth += 1
        elif value == right:
            depth -= 1
            if depth == 0:
                return index + 1
    raise ValueError(f'unbalanced {left} at line {tokens[start].line}')


def normalize_contract(source: str, libraries: dict[str, str] | None = None) -> tuple[str, dict]:
    """Retain ABI data and native prototypes; never retain a function body."""
    libraries = libraries or {}
    tokens = tokenize(source)
    output: list[Token] = []
    missing, native, effects, library_rows = [], [], [], []
    index = 0
    while index < len(tokens):
        current = tokens[index]
        value = current.value
        if value == '::':
            begin = index + 1
            if begin < len(tokens) and tokens[begin].value == 'inline':
                begin += 1
            if begin < len(tokens) and tokens[begin].value == '(':
                at = balanced_end(tokens, begin, '(', ')')
                while at < len(tokens) and tokens[at].value not in {'{', ';'}:
                    if tokens[at].value == '(':
                        at = balanced_end(tokens, at, '(', ')')
                    elif tokens[at].value == '[':
                        at = balanced_end(tokens, at, '[', ']')
                    else:
                        at += 1
                if at < len(tokens) and tokens[at].value == '{':
                    name = tokens[index - 1].value if index else '<anonymous>'
                    signature = [t for t in tokens[begin:at] if t.value not in {'#expand', '#must', '#no_context', '#deprecated'}]
                    # Deprecation messages describe the former implementation;
                    # omit the associated literal if the attribute was removed.
                    if any(t.value == '#deprecated' for t in tokens[begin:at]):
                        signature = [t for t in signature if not t.value.startswith('"')]
                    missing.append({'name': name, 'line': current.line,
                                    'signature': ' '.join(t.value for t in tokens[begin:at]),
                                    'status': 'unimplemented-adapter-contract'})
                    output.append(current)
                    output.extend(signature)
                    output.extend(Token(v, current.line) for v in ('#foreign', 'Native_Adapters', ';'))
                    index = balanced_end(tokens, at, '{', '}')
                    if index < len(tokens) and tokens[index].value == ';':
                        index += 1
                    continue
        if value == '#run' and index + 1 < len(tokens) and tokens[index + 1].value == '{':
            stop = balanced_end(tokens, index + 1, '{', '}')
            effects.append({'kind': 'run-block', 'line': current.line, 'status': 'removed'})
            index = stop
            if index < len(tokens) and tokens[index].value == ';':
                index += 1
            continue
        if value in {'#library', '#system_library'}:
            at = index + 1
            while at + 1 < len(tokens) and tokens[at].value == ',':
                at += 2
            if at < len(tokens) and tokens[at].value.startswith('"'):
                original = json.loads(tokens[at].value)
                binding = tokens[index - 2].value if index >= 2 and tokens[index - 1].value == '::' else None
                target = libraries.get(binding or '', libraries.get(original))
                if target is None:
                    if value == '#system_library' or any(t.value == 'system' for t in tokens[index:at]):
                        target = original
                    else:
                        # An intentionally absent external implementation is
                        # preferable to searching distribution-local binaries.
                        leaf = Path(original).name.removeprefix('lib')
                        target = 'jai-independent-' + leaf
                library_rows.append({'binding': binding, 'original_locator': original,
                                     'replacement': target, 'local_binary_included': False})
                output.extend((Token('#system_library', current.line), Token(json.dumps(target), current.line)))
                index = at + 1
                continue
        if value == '#foreign':
            native.append({'line': current.line, 'library': tokens[index + 1].value if index + 1 < len(tokens) else None})
        output.append(current)
        index += 1
    rendered = render(output)
    if missing:
        rendered += '\n#scope_file\nNative_Adapters :: #system_library "jai-stdlib-native-adapters";\n'
    report = {'native_declarations': len(native), 'removed_procedure_bodies': len(missing),
              'unimplemented_adapters': missing, 'removed_effects': effects,
              'library_locators': library_rows,
              'function_bodies_retained': 0, 'native_execution_performed': False}
    return rendered, report


def render(tokens: list[Token]) -> str:
    lines: list[str] = []
    row: list[str] = []
    indent = 0
    def flush() -> None:
        if row:
            lines.append('    ' * indent + ' '.join(row))
            row.clear()
    for token in tokens:
        value = token.value
        if value == '}':
            flush()
            indent = max(0, indent - 1)
            row.append(value)
            flush()
        elif value == '{':
            row.append(value)
            flush()
            indent += 1
        elif value == ';':
            row.append(value)
            flush()
        elif value.startswith('#scope_'):
            flush()
            row.append(value)
            flush()
        else:
            row.append(value)
    flush()
    return '\n'.join(lines) + '\n'


def rewrite_file(source_path: Path, output_path: Path, libraries: dict[str, str] | None = None) -> dict:
    raw = source_path.read_bytes()
    source = raw.decode('utf-8-sig', errors='strict')
    rewritten, report = normalize_contract(source, libraries)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text('// Independently normalized ABI contract. No reference procedure bodies or binaries.\n' + rewritten)
    report.update(source=str(source_path), output=str(output_path),
                  source_sha256=hashlib.sha256(raw).hexdigest(),
                  output_sha256=hashlib.sha256(output_path.read_bytes()).hexdigest())
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--libraries', type=Path)
    arguments = parser.parse_args()
    libraries = json.loads(arguments.libraries.read_text()) if arguments.libraries else {}
    print(json.dumps(rewrite_file(arguments.source, arguments.output, libraries), indent=2))


if __name__ == '__main__':
    main()
