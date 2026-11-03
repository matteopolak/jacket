#!/usr/bin/env python3
"""Completion latency while typing: a three-letter prefix of an in-scope identifier, one `didChange` per
character with a `completion` request right behind each, the way an editor does it.

    PYTHONPATH=tools python3 tools/bench/typing_latency.py --server target/release/jailsp FILE.jai... [--gap-ms 100,150]
        [--rounds 6] [--pull] [--max-p50-ms 50] [--max-ms 100]

With `--max-p50-ms` or `--max-ms` it exits 1 when a run's completion median or worst case is over the bound.

Prints, per file and gap, the completion latency (p50/p90/max, ms), the answer sizes, and the time from the
last keystroke of a round to its diagnostics. Run it with `JAIC_STDLIB` pointing at the stdlib.
"""
from __future__ import annotations

import argparse
import statistics
import sys
import time
from pathlib import Path

from bench.lsp_client import Client, capabilities, file_uri
from bench.lsp_positions import insert_edit, pick_positions

SETTLE = 1.5


def change(client: Client, uri: str, version: int, edits: list[dict]) -> None:
    client.notify('textDocument/didChange', {'textDocument': {'uri': uri, 'version': version}, 'contentChanges': edits})


def run(server: str, path: Path, gap: float, rounds: int, pull: bool) -> dict:
    text = path.read_text()
    picks = pick_positions(text)
    if not picks:
        raise SystemExit(f'{path}: no procedure body to type in')
    word = picks['hover']['name'][:3]
    line, indent = picks['insert_line'], picks['indent']
    workspace = path.resolve().parent
    uri = file_uri(path)
    client = Client([server], cwd=workspace)
    try:
        number, _ = client.send_request('initialize', {
            'processId': None, 'clientInfo': {'name': 'typing-latency'}, 'rootUri': file_uri(workspace),
            'workspaceFolders': [{'uri': file_uri(workspace), 'name': workspace.name}],
            'capabilities': capabilities(pull)})
        client.wait_response(number, 60)
        client.notify('initialized')
        opened = client.notify('textDocument/didOpen', {'textDocument': {
            'uri': uri, 'languageId': 'jai', 'version': 1, 'text': text}})
        if pull:
            client.request('textDocument/diagnostic', {'textDocument': {'uri': uri}}, 120)
        else:
            client.wait_notification('textDocument/publishDiagnostics', uri, opened, 120)
        time.sleep(SETTLE * 2)
        version = 1
        latencies: list[float] = []
        sizes: list[int] = []
        publish_after: list[float] = []
        for _ in range(rounds):
            version += 1
            change(client, uri, version, [insert_edit(line, 0, indent + '\n')])
            time.sleep(SETTLE)
            seen = len(client.notifications)
            pending = []
            last = 0.0
            start = time.perf_counter()
            for i, ch in enumerate(word):
                version += 1
                target = start + i * gap
                time.sleep(max(0, target - time.perf_counter()))
                change(client, uri, version, [insert_edit(line, len(indent) + i, ch)])
                pending.append(client.send_request('textDocument/completion', {
                    'textDocument': {'uri': uri}, 'position': {'line': line, 'character': len(indent) + i + 1},
                    'context': {'triggerKind': 1}}))
                last = time.perf_counter()
            for number, sent in pending:
                stamp, message = client.wait_response(number, 60)
                latencies.append((stamp - sent) * 1000)
                result = message.get('result') or []
                sizes.append(len(result.get('items', []) if isinstance(result, dict) else result))
            if not pull:
                try:
                    publish_after.append(client.wait_notification(
                        'textDocument/publishDiagnostics', uri, last, 30, seen) * 1000)
                except Exception:
                    pass
            time.sleep(SETTLE)
            version += 1
            change(client, uri, version, [{'range': {'start': {'line': line, 'character': 0},
                                                      'end': {'line': line + 1, 'character': 0}}, 'text': ''}])
            time.sleep(SETTLE)
        return {'word': word, 'lat': latencies, 'sizes': sizes, 'publish': publish_after}
    finally:
        client.finish(True)


def pct(values: list[float], p: float) -> float:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(len(ordered) * p))]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--server', required=True)
    parser.add_argument('--gap-ms', default='100,150')
    parser.add_argument('--rounds', type=int, default=6)
    parser.add_argument('--pull', action='store_true')
    parser.add_argument('--max-p50-ms', type=float, default=None)
    parser.add_argument('--max-ms', type=float, default=None)
    parser.add_argument('files', nargs='+', type=Path)
    args = parser.parse_args()
    failed = False
    for path in args.files:
        for gap in (float(g) / 1000 for g in args.gap_ms.split(',')):
            r = run(args.server, path, gap, args.rounds, args.pull)
            lat = r['lat']
            by_key = [[v for i, v in enumerate(lat) if i % len(r['word']) == k] for k in range(len(r['word']))]
            print(f'{path.name:16} gap {gap * 1000:3.0f} ms  "{r["word"]}"  n={len(lat)}  completion ms: '
                  f'p50 {statistics.median(lat):6.1f}  p90 {pct(lat, .9):6.1f}  max {max(lat):6.1f}  '
                  f'(by key p50: {" / ".join(f"{statistics.median(v):.0f}" for v in by_key)})  items~{statistics.median(r["sizes"]):.0f}  '
                  f'diagnostics after last key ms: {", ".join(f"{v:.0f}" for v in r["publish"])}', flush=True)
            if (args.max_p50_ms is not None and statistics.median(lat) > args.max_p50_ms) or (
                    args.max_ms is not None and max(lat) > args.max_ms):
                print(f'  over the bound: p50 <= {args.max_p50_ms}, max <= {args.max_ms}', flush=True)
                failed = True
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main())
