#!/usr/bin/env python3
r"""Edit-to-response benchmark for every registered LSP request.

Local gate (shared-runner timing is deliberately not a CI check):
 PYTHONPATH=tools JAIC_STDLIB=$PWD/stdlib python3 tools/bench/request_latency.py \
   --server /absolute/path/to/jailsp --formatter /absolute/path/to/jaifmt --rounds 7 --output /tmp/requests.json --max-p50-ms 25 --max-ms 120 \
   examples/basics/structs.jai examples/kvdb/main.jai examples/editor/editor.jai

Each sample inserts/removes a trailing comment, then immediately sends ONE request.
No changes/formatting/rename results are applied. Lifecycle requests are timed once.
Hierarchy/resolve/delta requests use real items/result IDs from preparatory requests;
missing hierarchy items are reported as skipped rather than timed with fake data.
JSON includes load averages, response sizes, skips and protocol errors. --methods
selects a comma-separated subset for profiling. Run at low load for comparisons.
"""
from __future__ import annotations
import argparse
import json
import os
import re
import statistics
import sys
import time
from pathlib import Path
from bench.lsp_client import Client, capabilities, file_uri
from bench.lsp_positions import pick_positions, utf16_col
from toolkit.repo import ROOT

POSITION = '''hover completion definition typeDefinition declaration prepareTypeHierarchy linkedEditingRange implementation prepareRename rename documentHighlight signatureHelp prepareCallHierarchy jai/expansion'''.split()
DOCUMENT = '''documentSymbol formatting documentLink foldingRange codeLens diagnostic semanticTokens/full'''.split()
METHODS = [f'textDocument/{m}' for m in POSITION if '/' not in m] + ['jai/expansion']
METHODS += [f'textDocument/{m}' for m in DOCUMENT]
METHODS += ['textDocument/' + m for m in ['rangeFormatting', 'references', 'inlayHint', 'codeAction', 'selectionRange', 'semanticTokens/range', 'semanticTokens/full/delta']]
METHODS += ['workspace/symbol', 'workspace/diagnostic', 'completionItem/resolve', 'typeHierarchy/supertypes', 'typeHierarchy/subtypes', 'callHierarchy/incomingCalls', 'callHierarchy/outgoingCalls', 'workspace/executeCommand', 'jai/source', 'initialize', 'shutdown']


def registered() -> set[str]:
    source = ROOT / 'crates/jai-language-server/src/protocol/methods.rs'
    return set(re.findall(r'\(\s*"([^"]+)",\s*Self::', source.read_text().split('/// The notifications')[0]))


def wait_for_load(bound: float | None) -> None:
    deadline = time.monotonic() + 180
    while bound is not None and os.getloadavg()[0] >= bound:
        if time.monotonic() >= deadline:
            raise RuntimeError(f'load stayed above {bound} for three minutes')
        time.sleep(1)


def run(server: str, path: Path, rounds: int, selected: set[str], settle: float, profile: str | None = None, max_load: float | None = None, formatter: str | None = None) -> dict:
    wait_for_load(max_load)
    text = path.read_text()
    picks = pick_positions(text)
    if not picks:
        raise ValueError(f'{path}: no request positions')
    uri = file_uri(path)
    td = {'uri': uri}
    def position(p):
        return {k: p[k] for k in ('line', 'character')}
    pos = position(picks['hover'])
    decl = position(picks['declaration'])
    visible = {'start': {'line': max(0, pos['line'] - 10), 'character': 0},
               'end': {'line': min(len(text.splitlines()) - 1, pos['line'] + 30), 'character': 0}}
    base = {'textDocument': td, 'position': pos}
    client = Client(['samply', 'record', '--save-only', '--unstable-presymbolicate', '-o', str(Path(profile).resolve()), '--', server] if profile else [server], cwd=path.resolve().parent)
    rows = {}
    def request(method, params):
        elapsed, response = client.request(method, params, 120)
        if 'error' in response:
            raise RuntimeError(f'{method}: {response["error"]}')
        return elapsed * 1000, response.get('result')
    try:
        initialization = {'processId': None, 'rootUri': file_uri(path.resolve().parent),
                          'capabilities': capabilities(True)}
        if formatter:
            initialization['initializationOptions'] = {'formatter': {'path': str(Path(formatter).resolve())}}
        elapsed, _ = request('initialize', initialization)
        rows['initialize'] = {'ms': [elapsed]}
        client.notify('initialized')
        client.notify('textDocument/didOpen', {'textDocument': {**td, 'languageId': 'jai', 'version': 1, 'text': text}})
        request('textDocument/diagnostic', {'textDocument': td})
        time.sleep(settle)
        params = {m: dict(base) for m in METHODS}
        for method in DOCUMENT:
            params['textDocument/' + method] = {'textDocument': td}
        params['textDocument/completion']['position'] = {**pos, 'character': pos['character'] + min(3, len(picks['hover']['name']))}
        params['textDocument/rename']['newName'] = picks['declaration']['name'] + '_bench'
        for method in ('rename', 'prepareRename'):
            params['textDocument/' + method]['position'] = decl
        params['textDocument/references'] = {**base, 'position': decl, 'context': {'includeDeclaration': True}}
        params['textDocument/signatureHelp']['position'] = position(picks['call'] or picks['hover'])
        for method in ('rangeFormatting', 'inlayHint', 'codeAction', 'semanticTokens/range'):
            params['textDocument/' + method] = {'textDocument': td, 'range': visible}
        for method in ('formatting', 'rangeFormatting'):
            params['textDocument/' + method]['options'] = {'tabSize': 4, 'insertSpaces': True}
        params['textDocument/codeAction']['context'] = {'diagnostics': []}
        params['textDocument/selectionRange'] = {'textDocument': td, 'positions': [pos]}
        params['workspace/symbol'] = {'query': picks['declaration']['name'][:3]}
        params['workspace/diagnostic'] = {'previousResultIds': []}
        params['workspace/executeCommand'] = {'command': 'jai.showCompileTimeCosts', 'arguments': [{'uri': uri, 'position': pos}]}
        params['jai/source'] = td
        _, completions = request('textDocument/completion', base)
        completions = (completions or {}).get('items', []) if isinstance(completions, dict) else completions or []
        if completions:
            params['completionItem/resolve'] = next((i for i in completions if i.get('data')), completions[0])
        else:
            rows['completionItem/resolve'] = {'skip': 'no completion item to resolve'}
        for family, prepare in [('typeHierarchy', 'prepareTypeHierarchy'), ('callHierarchy', 'prepareCallHierarchy')]:
            preparation = {**base, 'position': decl}
            _, items = request('textDocument/' + prepare, preparation)
            if not items and family == 'typeHierarchy':
                # Find a real type declaration rather than timing a fabricated hierarchy item.
                for line, source in enumerate(text.splitlines()):
                    match = re.match(r'^(\w+)\s*::\s*(?:struct|enum|union)\b', source)
                    if match:
                        preparation = {**base, 'position': {'line': line, 'character': 0}}
                        _, items = request('textDocument/' + prepare, preparation)
                        if items:
                            break
            params['textDocument/' + prepare] = preparation
            for method in METHODS:
                if method.startswith(family + '/'):
                    if items:
                        params[method] = {'item': items[0]}
                    else:
                        rows[method] = {'skip': 'no hierarchy item at selected declaration'}
        _, tokens = request('textDocument/semanticTokens/full', {'textDocument': td})
        token_id = tokens['resultId']
        end = {'line': text.count('\n'), 'character': utf16_col(text.split('\n')[-1], len(text.split('\n')[-1]))}
        version = 1
        for method in METHODS:
            if method not in selected or method in rows or method == 'shutdown':
                continue
            if method == 'textDocument/semanticTokens/full/delta':
                _, tokens = request('textDocument/semanticTokens/full', {'textDocument': td})
                token_id = tokens['resultId']
            values, sizes, loads = [], [], []
            for sample in range(rounds):
                wait_for_load(max_load)
                loads.append(os.getloadavg()[0])
                version += 1
                suffix = f'\n// latency {method} {sample}\n'
                client.notify('textDocument/didChange', {'textDocument': {**td, 'version': version},
                    'contentChanges': [{'range': {'start': end, 'end': end}, 'text': suffix}]})
                p = params[method]
                if method == 'textDocument/semanticTokens/full/delta':
                    p = {'textDocument': td, 'previousResultId': token_id}
                elapsed, result = request(method, p)
                values.append(elapsed)
                sizes.append(len(json.dumps(result)))
                if method == 'textDocument/semanticTokens/full/delta':
                    token_id = result['resultId']
                version += 1
                client.notify('textDocument/didChange', {'textDocument': {**td, 'version': version},
                    'contentChanges': [{'range': {'start': end, 'end': {'line': end['line'] + 2, 'character': 0}}, 'text': ''}]})
                time.sleep(settle)
            rows[method] = {'ms': values, 'bytes': sizes, 'loads': loads}
            print(f'{path.parent.name}/{path.name} {method}: {statistics.median(values):.2f}/{max(values):.2f} ms', flush=True)
        formatter_errors = [m['params']['message'] for _, m in client.notifications
                            if m.get('method') == 'window/logMessage'
                            and m.get('params', {}).get('message', '').startswith('jaifmt:')]
        if formatter_errors:
            raise RuntimeError(f'formatter did not succeed: {formatter_errors}')
        elapsed, _ = request('shutdown', {})
        rows['shutdown'] = {'ms': [elapsed]}
        client.notify('exit')
        return {'file': str(path), 'load': os.getloadavg(), 'requests': {m: r for m, r in rows.items() if m in selected}}
    finally:
        client.finish(False)


def make_limits(report: dict, headroom: float = 1.25) -> dict:
    return {row['file']: {method: {'max_p50_ms': max(25, result['p50'] * headroom),
                                 'max_ms': max(120, result['max'] * headroom)}
                         for method, result in row['requests'].items() if 'ms' in result}
            for row in report['results']}


def exceeds(result: dict, p50: float | None, maximum: float | None) -> bool:
    return ((p50 is not None and result['p50'] > p50)
            or (maximum is not None and result['max'] > maximum))


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--server', required=True)
    parser.add_argument('--formatter', help="this checkout's jaifmt; otherwise the server searches beside itself and PATH")
    parser.add_argument('--profile', help='record the server with samply (one file only)')
    parser.add_argument('--rounds', type=int, default=7)
    parser.add_argument('--settle', type=float, default=.3)
    parser.add_argument('--methods', default=','.join(METHODS))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--max-load', type=float, help='wait (at most three minutes) before each sample for load below this bound')
    parser.add_argument('--limits', type=Path, help='per-file/per-method limits from --write-limits')
    parser.add_argument('--write-limits', type=Path, help='write measured limits with 25 percent headroom (floors: 25/120 ms)')
    parser.add_argument('--max-p50-ms', type=float)
    parser.add_argument('--max-ms', type=float)
    parser.add_argument('files', nargs='+', type=Path)
    args = parser.parse_args()
    if args.profile and len(args.files) != 1:
        parser.error('--profile requires one file')
    missing = registered() - set(METHODS)
    if missing:
        parser.error(f'add benchmark cases for registered methods: {sorted(missing)}')
    selected = set(args.methods.split(','))
    if selected - set(METHODS) or args.rounds < 1:
        parser.error('unknown methods or nonpositive rounds')
    report = {'load_start': os.getloadavg(), 'rounds': args.rounds, 'settle_s': args.settle,
              'server': str(Path(args.server).resolve()), 'formatter': args.formatter, 'methods': sorted(selected),
              'scenario': 'insert a unique trailing comment, immediately request, restore, settle',
              'max_load': args.max_load, 'results': []}
    limits = json.loads(args.limits.read_text()) if args.limits else {}
    failed = False
    try:
        for path in args.files:
            row = run(str(Path(args.server).resolve()), path, args.rounds, selected, args.settle, args.profile, args.max_load, args.formatter)
            report['results'].append(row)
            for method, result in row['requests'].items():
                if 'ms' in result:
                    result['p50'] = statistics.median(result['ms'])
                    result['max'] = max(result['ms'])
                    bound = limits.get(str(path), {}).get(method, {})
                    if args.limits and not bound:
                        raise ValueError(f'no limits for {path}: {method}')
                    over = exceeds(result, args.max_p50_ms if args.max_p50_ms is not None else bound.get('max_p50_ms'),
                                   args.max_ms if args.max_ms is not None else bound.get('max_ms'))
                    if over:
                        print(f'OVER BOUND {path}: {method}', flush=True)
                    failed |= over
                else:
                    print(f'SKIPPED {path}: {method}: {result["skip"]}', flush=True)
                    failed = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        args.output.write_text(json.dumps(report, indent=2) + '\n')
    if args.write_limits:
        args.write_limits.write_text(json.dumps(make_limits(report), indent=2) + '\n')
    return int(failed)


if __name__ == '__main__':
    sys.exit(main())
