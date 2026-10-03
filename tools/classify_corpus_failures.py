#!/usr/bin/env python3
"""Group measured parser failures with pinned locations and local syntax hints."""
from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path
import re

from check_corpus import ROOT, digest
from inventory_corpus_features import FEATURES, decode, mask_noncode

LOCATION = re.compile(r'^(.+):(\d+):(\d+): error: (.+)$')
PATTERNS = {name: re.compile(pattern) for name, pattern in FEATURES.items()}


def classify(report_path: Path, stage: str = 'parse') -> dict:
    report = json.loads(report_path.read_text())
    groups = {}
    stale = []
    source_rows = {row['source']: row for row in report['results']}
    for row in report['results']:
        evidence = row['stages'][stage]
        if evidence['status'] != 'failed':
            continue
        root_path = Path(row['source'])
        if not root_path.is_file() or digest(root_path) != row['sha256']:
            stale.append(row['id'])
            continue
        first = evidence['diagnostic'].splitlines()
        location = LOCATION.match(first[0]) if first else None
        if not location:
            message = first[0] if first else evidence['reason']
            key, line, column, hints = message, None, None, []
        else:
            path, line_text, column_text, message = location.groups()
            line, column = int(line_text), int(column_text)
            located_row = source_rows.get(path)
            source = Path(path)
            if located_row is None or not source.is_file() or digest(source) != located_row['sha256']:
                stale.append(row['id'])
                continue
            code = mask_noncode(decode(source.read_bytes()))
            lines = code.splitlines()
            context = lines[line-1] if line <= len(lines) else ''
            hints = [name for name, pattern in PATTERNS.items() if pattern.search(context)]
            # Diagnostic text stays exact; lexical hints are explicitly noncausal.
            key = message
        project = row['id'].split(':', 1)[0]
        group = groups.setdefault(key, {'diagnostic': key, 'count': 0, 'projects': Counter(), 'syntax_hints': Counter(), 'locations': []})
        group['count'] += 1
        group['projects'][project] += 1
        group['syntax_hints'].update(hints)
        group['locations'].append({'source': located_row['id'] if location else row['id'],
                                   'root_source': row['id'], 'line': line, 'column': column,
                                   'sha256': located_row['sha256'] if location else row['sha256'], 'syntax_hints': hints,
                                   'diagnostic_source': located_row['id'] if location else row['id']})
    ordered = sorted(groups.values(), key=lambda g: (-g['count'], g['diagnostic']))
    for group in ordered:
        group['projects'] = dict(group['projects'])
        group['syntax_hints'] = dict(group['syntax_hints'])
        group['locations'].sort(key=lambda p: (p['source'].startswith('reference:'), p['source']))
        dependencies = {}
        for location in group['locations']:
            identity = (location['source'], location['line'], location['column'])
            dependency = dependencies.setdefault(identity, {
                'source': location['source'], 'line': location['line'], 'column': location['column'],
                'sha256': location['sha256'], 'root_count': 0, 'sample_roots': []})
            dependency['root_count'] += 1
            if len(dependency['sample_roots']) < 8:
                dependency['sample_roots'].append(location['root_source'])
        group['dependency_locations'] = sorted(dependencies.values(), key=lambda d: (-d['root_count'], d['source']))
    return {'format': 1, 'acceptance_report': str(report_path), 'acceptance_report_sha256': digest(report_path),
            'compiler': report['compiler'], 'inventory': report['inventory'], 'stage': stage, 'stage_totals': report['totals'][stage], 'parse_totals': report['totals']['parse'],
            'stale_source_locations': stale, 'groups': ordered,
            'limitations': ['Counts group first measured stage diagnostics only; later errors are hidden.',
                            'Line-level lexical hints are observations, not proven causes or feature support.',
                            'Locations are sorted modern upstream first; no source excerpts are retained.',
                            'No checking, LLVM, native build, SDK or runtime acceptance is implied.']}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    parser.add_argument('--stage', choices=['lex', 'parse', 'check'], default='parse')
    parser.add_argument('--output', type=Path, default=ROOT/'artifacts/corpus-parse-gaps.json')
    args = parser.parse_args()
    output = args.output.resolve()
    if any(output.is_relative_to(p.resolve()) for p in (ROOT/'reference', ROOT/'corpus/upstream')):
        parser.error('output must be outside read-only source corpus')
    result = classify(args.report.resolve(), args.stage)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'stage': result['stage'], 'stage_totals': result['stage_totals'], 'groups': len(result['groups']),
                      'stale_source_locations': len(result['stale_source_locations']), 'output': str(output)}))
    return int(bool(result['stale_source_locations']))

if __name__ == '__main__':
    raise SystemExit(main())
