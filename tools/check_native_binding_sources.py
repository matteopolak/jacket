#!/usr/bin/env python3
"""Check authored graphics binding syntax using an explicit frozen own CLI.

No reference implementation, native dependency or executable artifact from the
supplied distribution is read or run. Procedure witnesses contain only already
authored stdlib source, making earlier declaration-parser gaps distinguishable
from helper-body syntax. This is never a native behavior or ABI acceptance test.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

from rewrite_native_api_contracts import balanced_end, normalize_contract, tokenize


FAMILIES = ('GL', 'Metal', 'Vulkan', 'Curl', 'ImGui', 'd3d11', 'd3d12',
            'dxgi', 'd3d_compiler', 'dxc_compiler')


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def procedures(source: str) -> list[dict]:
    tokens = tokenize(source)
    lines = source.splitlines(keepends=True)
    result = []
    index = 1
    while index < len(tokens) - 1:
        if tokens[index].value != '::':
            index += 1
            continue
        begin = index + 1
        if tokens[begin].value == 'inline':
            begin += 1
        if tokens[begin].value != '(':
            index += 1
            continue
        body = balanced_end(tokens, begin, '(', ')')
        while body < len(tokens) and tokens[body].value not in {'{', ';'}:
            if tokens[body].value == '(':
                body = balanced_end(tokens, body, '(', ')')
            elif tokens[body].value == '[':
                body = balanced_end(tokens, body, '[', ']')
            else:
                body += 1
        if body == len(tokens) or tokens[body].value != '{':
            index += 1
            continue
        end = balanced_end(tokens, body, '{', '}')
        first, last = tokens[index - 1].line, tokens[end - 1].line
        result.append({'name': tokens[index - 1].value, 'line': first,
                       'signature': ' '.join(t.value for t in tokens[begin:body]),
                       'source': ''.join(lines[first - 1:last]),
                       'empty_body': end == body + 2})
        index = end
    return result


def parse(parser: Path, path: Path) -> dict:
    completed = subprocess.run([str(parser.resolve()), 'parse', str(path)],
        capture_output=True, text=True, timeout=30, check=False)
    return {'syntax_pass': completed.returncode == 0,
            'syntax_output': (completed.stdout + completed.stderr).strip(),
            'syntax_only': True, 'native_execution_performed': False}


def main() -> None:
    arguments = argparse.ArgumentParser(description=__doc__)
    arguments.add_argument('--parser', required=True, type=Path)
    arguments.add_argument('--output', type=Path, default=Path('stdlib/.coverage/graphics-source-parse.json'))
    arguments.add_argument('--witness-dir', type=Path,
        default=Path('artifacts/stdlib-rewrite/native-body-syntax'))
    options = arguments.parse_args()
    parser_before = digest(options.parser)
    files, witnesses = [], []
    options.witness_dir.mkdir(parents=True, exist_ok=True)
    for family in FAMILIES:
        bodies = []
        paths = sorted((Path('stdlib') / family).glob('*.jai'))
        for path in paths:
            before = digest(path)
            source = path.read_text()
            _, inventory = normalize_contract(source)
            own_procedures = procedures(source)
            adapters = []
            for match in re.finditer(r'^([ \t]*)(\w+) :: (.*?) #foreign Native_Adapters(?: .*?)? ;$', source, re.M):
                adapters.append({'name': match[2], 'signature': match[3],
                    'line': source[:match.start()].count('\n') + 1,
                    'status': 'unimplemented-adapter-contract'})
            row = {'path': str(path), 'sha256_before': before, **parse(options.parser, path),
                   'foreign_declarations': inventory['native_declarations'],
                   'unimplemented_adapter_declarations': len(adapters),
                   'unimplemented_adapters': adapters,
                   'authored_procedure_bodies': len(own_procedures),
                   'empty_bodies': [p['name'] for p in own_procedures if p['empty_body']],
                   'library_locators': inventory['library_locators']}
            row['sha256_after'] = digest(path)
            row['source_stable_during_check'] = before == row['sha256_after']
            files.append(row)
            for procedure in own_procedures:
                bodies.append('// Authored source: ' + str(path) + ':' + str(procedure['line']) + '\n' + procedure['source'] + '\n')
        if bodies:
            witness = options.witness_dir / (family.lower().replace('_', '-') + '.jai')
            witness.write_text('// Source-only helper syntax witness. No imports, native calls or linking are executed.\n' + '\n'.join(bodies))
            witnesses.append({'path': str(witness), 'family': family,
                'sha256': digest(witness), 'procedure_body_count': len(bodies),
                **parse(options.parser, witness)})
    parser_after = digest(options.parser)
    report = {'format': 2, 'check': 'authored-source-syntax-only',
        'parser': str(options.parser), 'parser_sha256_before': parser_before,
        'parser_sha256_after': parser_after, 'parser_stable': parser_before == parser_after,
        'parser_build_inputs_verified': False,
        'semantic_resolution_verified': False, 'native_abi_verified': False,
        'native_behavior_verified': False, 'files': files, 'procedure_witnesses': witnesses,
        'totals': {'files': len(files), 'source_syntax_pass': sum(f['syntax_pass'] for f in files),
            'authored_procedure_bodies': sum(f['authored_procedure_bodies'] for f in files),
            'foreign_declarations': sum(f['foreign_declarations'] for f in files),
            'unimplemented_adapter_declarations': sum(f['unimplemented_adapter_declarations'] for f in files),
            'empty_body_occurrences': sum(len(f['empty_bodies']) for f in files),
            'procedure_witnesses': len(witnesses), 'procedure_witness_syntax_pass': sum(w['syntax_pass'] for w in witnesses)}}
    options.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report['totals']))


if __name__ == '__main__':
    main()
