#!/usr/bin/env python3
r"""Edit-to-response benchmark for every registered LSP request.

Local gate (shared-runner timing is deliberately not a CI check):
 PYTHONPATH=tools JAIC_STDLIB=$PWD/stdlib python3 tools/bench/request_latency.py \
   --server /absolute/path/to/jailsp --formatter /absolute/path/to/jaifmt --rounds 7 --output /tmp/requests.json --max-p50-ms 25 --max-ms 120 \
   examples/basics/structs.jai examples/kvdb/main.jai examples/editor/editor.jai

By default each sample inserts a trailing comment, immediately sends one request,
and removes the comment. Formatting and rename results are not applied. Lifecycle requests are timed once.
Hierarchy/resolve/delta requests use real items/result IDs from preparatory requests;
missing hierarchy items are reported as skipped rather than timed with fake data.
JSON includes load averages, response sizes, skips, protocol errors, and server peak
RSS and CPU time. --include-payloads also saves selected positions and timed request
parameters and results for correctness inspection. --methods selects a comma-separated
subset for profiling. --scenario warm-unchanged skips edits and requires one
--binding declaration snippet per input file; hover, reference and linked-editing
ranges must select that identifier, and hover contents must be nonempty. --declaration can select each rename target explicitly.
--scenario edit-recheck inserts the same comment, requests
full diagnostics, then measures the selected answer and total edit-to-answer time.
Example: --scenario warm-unchanged --binding
'knight: Knight' examples/basics/structs.jai. Run at low load for comparisons.
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

# Independent protocol fixtures paired with the server dispatch case they exercise.
# Keep this order stable: the historical workload measures methods in this order.
SCENARIOS = [
    ('Hover', 'textDocument/hover'),
    ('Completion', 'textDocument/completion'),
    ('Definition', 'textDocument/definition'),
    ('TypeDefinition', 'textDocument/typeDefinition'),
    ('Declaration', 'textDocument/declaration'),
    ('PrepareTypeHierarchy', 'textDocument/prepareTypeHierarchy'),
    ('LinkedEditingRange', 'textDocument/linkedEditingRange'),
    ('Implementation', 'textDocument/implementation'),
    ('PrepareRename', 'textDocument/prepareRename'),
    ('Rename', 'textDocument/rename'),
    ('DocumentHighlight', 'textDocument/documentHighlight'),
    ('SignatureHelp', 'textDocument/signatureHelp'),
    ('PrepareCallHierarchy', 'textDocument/prepareCallHierarchy'),
    ('Expansion', 'jai/expansion'),
    ('DocumentSymbol', 'textDocument/documentSymbol'),
    ('Formatting', 'textDocument/formatting'),
    ('DocumentLink', 'textDocument/documentLink'),
    ('FoldingRange', 'textDocument/foldingRange'),
    ('CodeLens', 'textDocument/codeLens'),
    ('DocumentDiagnostic', 'textDocument/diagnostic'),
    ('SemanticTokens', 'textDocument/semanticTokens/full'),
    ('RangeFormatting', 'textDocument/rangeFormatting'),
    ('References', 'textDocument/references'),
    ('InlayHint', 'textDocument/inlayHint'),
    ('CodeAction', 'textDocument/codeAction'),
    ('SelectionRange', 'textDocument/selectionRange'),
    ('SemanticTokensRange', 'textDocument/semanticTokens/range'),
    ('SemanticTokensDelta', 'textDocument/semanticTokens/full/delta'),
    ('WorkspaceSymbol', 'workspace/symbol'),
    ('WorkspaceDiagnostic', 'workspace/diagnostic'),
    ('CompletionResolve', 'completionItem/resolve'),
    ('Supertypes', 'typeHierarchy/supertypes'),
    ('Subtypes', 'typeHierarchy/subtypes'),
    ('IncomingCalls', 'callHierarchy/incomingCalls'),
    ('OutgoingCalls', 'callHierarchy/outgoingCalls'),
    ('ExecuteCommand', 'workspace/executeCommand'),
    ('Source', 'jai/source'),
    ('Initialize', 'initialize'),
    ('Shutdown', 'shutdown'),
]
METHODS = [method for _, method in SCENARIOS]
DOCUMENT = 'documentSymbol formatting documentLink foldingRange codeLens diagnostic semanticTokens/full'.split()


def registered(source: str | None = None) -> set[str]:
    """Dispatch cases, independent of whether their wire names are literals or constants."""
    if source is None:
        source = (ROOT / 'crates/jai-language-server/src/protocol/methods.rs').read_text()
    table = source.split('impl Request {', 1)[1].split('const METHODS:', 1)[1].split('];', 1)[0]
    return set(re.findall(r'\bSelf::(\w+)', table))


def wait_for_load(bound: float | None) -> None:
    deadline = time.monotonic() + 180
    while bound is not None and os.getloadavg()[0] >= bound:
        if time.monotonic() >= deadline:
            raise RuntimeError(f'load stayed above {bound} for three minutes')
        time.sleep(1)


def run(server: str, path: Path, rounds: int, selected: set[str], settle: float, profile: str | None = None, max_load: float | None = None, formatter: str | None = None, include_payloads: bool = False, scenario: str = 'comment-edit', binding: str | None = None, declaration: str | None = None) -> dict:
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
    binding_name = None
    if scenario in ('warm-unchanged', 'edit-recheck'):
        if not binding or not (snippet := re.search(r'(?<![A-Za-z0-9_])' + re.escape(binding), text)):
            raise ValueError(f'{path}: binding snippet not found: {binding!r}')
        offset = snippet.start()
        match = re.match(r'[A-Za-z_]\w*', binding)
        if not match:
            raise ValueError('binding snippet must begin with an identifier')
        binding_name = match.group()
        prefix = text[:offset]
        pos = {'line': prefix.count('\n'), 'character': utf16_col(prefix.rsplit('\n', 1)[-1], len(prefix.rsplit('\n', 1)[-1]))}
    decl = position(picks['declaration'])
    declaration_name = picks['declaration']['name']
    if declaration:
        if not (match := re.match(r'[A-Za-z_]\w*', declaration)) or not (found := re.search(r'(?<![A-Za-z0-9_])' + re.escape(declaration), text)):
            raise ValueError(f'{path}: declaration snippet not found: {declaration!r}')
        declaration_name = match.group()
        prefix = text[:found.start()]
        decl = {'line': prefix.count('\n'), 'character': utf16_col(prefix.rsplit('\n', 1)[-1], len(prefix.rsplit('\n', 1)[-1]))}
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
        initialize_ms = elapsed
        client.notify('initialized')
        client.notify('textDocument/didOpen', {'textDocument': {**td, 'languageId': 'jai', 'version': 1, 'text': text}})
        first_diagnostic_ms, _ = request('textDocument/diagnostic', {'textDocument': td})
        time.sleep(settle)
        params = {m: dict(base) for m in METHODS}
        for method in DOCUMENT:
            params['textDocument/' + method] = {'textDocument': td}
        params['textDocument/completion']['position'] = {**pos, 'character': pos['character'] + min(3, len(picks['hover']['name']))}
        params['textDocument/rename']['newName'] = declaration_name + '_bench'
        for method in ('rename', 'prepareRename'):
            params['textDocument/' + method]['position'] = decl
        params['textDocument/references'] = {**base, 'position': pos if binding_name else decl, 'context': {'includeDeclaration': True}}
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
            values, sizes, loads, payloads, total_values = [], [], [], [], []
            for sample in range(rounds):
                wait_for_load(max_load)
                loads.append(os.getloadavg()[0])
                total_start = time.perf_counter()
                if scenario in ('comment-edit', 'edit-recheck'):
                    version += 1
                    suffix = f'\n// latency {method} {sample}\n'
                    client.notify('textDocument/didChange', {'textDocument': {**td, 'version': version},
                        'contentChanges': [{'range': {'start': end, 'end': end}, 'text': suffix}]})
                p = params[method]
                if method == 'textDocument/semanticTokens/full/delta':
                    p = {'textDocument': td, 'previousResultId': token_id}
                if scenario == 'edit-recheck':
                    _, checked = request('textDocument/diagnostic', {'textDocument': td})
                    if not isinstance(checked, dict) or checked.get('kind') != 'full':
                        raise ValueError(f'{path}: expected complete diagnostic response')
                elapsed, result = request(method, p)
                if scenario == 'edit-recheck':
                    total_values.append((time.perf_counter() - total_start) * 1000)
                values.append(elapsed)
                sizes.append(len(json.dumps(result)))
                if include_payloads:
                    payload = {'document_version': version, 'params': p, 'result': result}
                    if scenario == 'edit-recheck':
                        payload['recheck_diagnostic'] = checked
                    payloads.append(payload)
                if method == 'textDocument/semanticTokens/full/delta':
                    token_id = result['resultId']
                if binding_name and method in ('textDocument/hover', 'textDocument/references', 'textDocument/linkedEditingRange'):
                    if method.endswith('/hover'):
                        contents = (result or {}).get('contents', {})
                        value = contents if isinstance(contents, str) else contents.get('value', '')
                        if not value.strip() or not (result or {}).get('range'):
                            raise ValueError(f'{path}: hover returned no useful binding description/range')
                        locations = [result['range']]
                    else:
                        locations = result if method.endswith('/references') else (result or {}).get('ranges', [])
                    if not locations:
                        raise ValueError(f'{path}: {method} returned no binding ranges')
                    for location in locations:
                        if 'uri' in location and location['uri'] != uri:
                            raise ValueError(f'{path}: binding range in another file')
                        span = location.get('range', location)
                        start, stop = span['start'], span['end']
                        if start['line'] != stop['line']:
                            raise ValueError(f'{path}: multiline binding range')
                        line = text.splitlines()[start['line']].encode('utf-16-le')
                        actual = line[start['character'] * 2:stop['character'] * 2].decode('utf-16-le')
                        if actual != binding_name:
                            raise ValueError(f'{path}: expected {binding_name!r}, got {actual!r}')
                if scenario in ('comment-edit', 'edit-recheck'):
                    version += 1
                    client.notify('textDocument/didChange', {'textDocument': {**td, 'version': version},
                        'contentChanges': [{'range': {'start': end, 'end': {'line': end['line'] + 2, 'character': 0}}, 'text': ''}]})
                time.sleep(settle)
            rows[method] = {'ms': values, 'bytes': sizes, 'loads': loads}
            if scenario == 'edit-recheck':
                rows[method]['edit_to_answer_ms'] = total_values
            if include_payloads:
                rows[method]['payloads'] = payloads
            print(f'{path.parent.name}/{path.name} {method}: {statistics.median(values):.2f}/{max(values):.2f} ms', flush=True)
        formatter_errors = [m['params']['message'] for _, m in client.notifications
                            if m.get('method') == 'window/logMessage'
                            and m.get('params', {}).get('message', '').startswith('jaifmt:')]
        if formatter_errors:
            raise RuntimeError(f'formatter did not succeed: {formatter_errors}')
        elapsed, _ = request('shutdown', {})
        rows['shutdown'] = {'ms': [elapsed]}
        client.notify('exit')
        report = {'file': str(path), 'load': os.getloadavg(),
                  'startup': {'initialize_ms': initialize_ms, 'first_diagnostic_ms': first_diagnostic_ms},
                  'requests': {m: r for m, r in rows.items() if m in selected}}
        if declaration:
            report['declaration'] = {'name': declaration_name, 'position': decl}
        if binding_name:
            report['binding'] = {'name': binding_name, 'position': pos, 'version': 1}
        if include_payloads:
            report['positions'] = picks
    finally:
        peak_rss = client.finish(False)
    report['process'] = {'peak_rss_mib': peak_rss, 'cpu_user_s': client.rusage.ru_utime,
                         'cpu_system_s': client.rusage.ru_stime} if client.rusage else None
    return report


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
    parser.add_argument('--include-payloads', action='store_true', help='save selected positions and timed request parameters and results')
    parser.add_argument('--scenario', choices=['comment-edit', 'warm-unchanged', 'edit-recheck'], default='comment-edit')
    parser.add_argument('--binding', action='append', help='binding declaration snippet, once per input file for warm-unchanged')
    parser.add_argument('--declaration', action='append', help='rename declaration snippet, once per input file')
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
    if args.scenario in ('warm-unchanged', 'edit-recheck') and (not args.binding or len(args.binding) != len(args.files)):
        parser.error('this scenario requires one --binding snippet per file')
    if args.declaration and len(args.declaration) != len(args.files):
        parser.error('provide one --declaration per file')
    if args.profile and len(args.files) != 1:
        parser.error('--profile requires one file')
    missing = registered() - {variant for variant, _ in SCENARIOS}
    if missing:
        parser.error(f'add benchmark cases for registered dispatch variants: {sorted(missing)}')
    selected = set(args.methods.split(','))
    if selected - set(METHODS) or args.rounds < 1:
        parser.error('unknown methods or nonpositive rounds')
    report = {'load_start': os.getloadavg(), 'rounds': args.rounds, 'settle_s': args.settle,
              'server': str(Path(args.server).resolve()), 'formatter': args.formatter, 'methods': sorted(selected),
              'scenario': args.scenario,
              'max_load': args.max_load, 'results': []}
    limits = json.loads(args.limits.read_text()) if args.limits else {}
    failed = False
    try:
        for index, path in enumerate(args.files):
            binding = args.binding[index] if args.binding else None
            declaration = args.declaration[index] if args.declaration else None
            row = run(str(Path(args.server).resolve()), path, args.rounds, selected, args.settle, args.profile, args.max_load, args.formatter, args.include_payloads, args.scenario, binding, declaration)
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
