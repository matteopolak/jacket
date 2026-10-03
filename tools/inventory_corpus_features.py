#!/usr/bin/env python3
"""Read pinned source only and count feature spellings; never execute corpus code."""
from __future__ import annotations

import argparse
from bisect import bisect_right
from collections import Counter
import hashlib
import json
from pathlib import Path
import re

from check_corpus import ROOT, digest, inventory

# This is intentionally a lexical inventory, not an AST or a support claim.
FEATURES = {
    'imports': r'#import\b', 'loads': r'#load\b',
    'module-parameters': r'#module_parameters\b',
    'conditional-compilation': r'#if\b', 'compile-time-run': r'#run\b',
    'compile-time-assert': r'#assert\b', 'source-insertion': r'#insert\b',
    'polymorphism': r'\$[A-Za-z_]\w*', 'structs': r'\bstruct\b',
    'unions': r'\bunion\b', 'enums': r'\benum(?:_flags)?\b',
    'using': r'\busing\b', 'fixed-arrays': r'\[\s*\d+\s*\]',
    'slices': r'\[\s*\]', 'dynamic-arrays': r'\[\s*\.\.\s*\]',
    'pointer-types-or-addresses': r'\*\s*[A-Za-z_]\w*',
    'dereference': r'\.\*|<<', 'floating-types': r'\b(?:float(?:32|64)?|f32|f64)\b',
    'string-types': r'\bstring\b', 'context': r'\bcontext\b|#add_context\b',
    'procedure-expansion': r'#expand\b', 'foreign-abi': r'#(?:foreign|c_call)\b',
    'reflection': r'\b(?:type_of|type_info|size_of|align_of|offset_of)\b',
    'compiler-workspaces': r'\b(?:compiler_create_workspace|compiler_begin_intercept|compiler_get_message|Compiler_Message)\b',
    'defer': r'\bdefer\b', 'cases': r'\bcase\b',
    'targets': r'\b(?:OS|CPU|BUILD_OPTIONS|Target_OS|Target_CPU)\b',
}


def decode(data: bytes) -> str:
    if data.startswith((b'\xff\xfe', b'\xfe\xff')):
        return data.decode('utf-16')
    return data.decode('utf-8-sig', errors='replace')


def mask_noncode(source: str) -> str:
    """Preserve offsets/newlines while blanking quotes and nested comments.

    Jai here-strings have custom delimiters: those are not decoded here. Matches
    inside them remain possible and are explicitly listed as a report limitation.
    """
    out = list(source)
    at, depth, quote = 0, 0, None
    while at < len(source):
        if depth:
            width = 2 if source[at:at+2] in ('/*', '*/') else 1
            if source[at:at+2] == '/*': depth += 1
            elif source[at:at+2] == '*/': depth -= 1
        elif quote:
            width = 2 if source[at] == '\\' and at + 1 < len(source) else 1
            if source[at] == quote: quote = None
        elif source[at:at+2] == '/*':
            depth, width = 1, 2
        elif source[at:at+2] == '//':
            end = source.find('\n', at)
            width = (len(source) if end < 0 else end) - at
        elif source[at] in ('"', "'"):
            quote, width = source[at], 1
        else:
            at += 1
            continue
        for index in range(at, min(at + width, len(source))):
            if source[index] not in '\r\n': out[index] = ' '
        at += width
    return ''.join(out)


def scan(root: Path, sample_limit: int = 8) -> dict:
    sources = inventory(root)
    patterns = {key: re.compile(value) for key, value in FEATURES.items()}
    features = {key: {'occurrences': 0, 'files': 0, 'projects': {}, 'samples': []}
                for key in patterns}
    rows = []
    for source in sources:
        if source.sha256 != source.expected_sha256:
            raise ValueError(f'pinned source missing or changed: {source.id}')
        code = mask_noncode(decode(source.path.read_bytes()))
        newlines = [index for index, char in enumerate(code) if char == '\n']
        counts = {}
        for key, pattern in patterns.items():
            matches = list(pattern.finditer(code))
            if not matches: continue
            counts[key] = len(matches)
            feature = features[key]
            feature['occurrences'] += len(matches)
            feature['files'] += 1
            feature['projects'][source.project] = feature['projects'].get(source.project, 0) + 1
            for match in matches:
                if len(feature['samples']) >= sample_limit or any(sample['source'].startswith(source.project + ':') for sample in feature['samples']): break
                feature['samples'].append({'source': source.id, 'line': bisect_right(newlines, match.start()) + 1,
                                           'sha256': source.sha256, 'revision': source.revision})
        rows.append({'source': source.id, 'sha256': source.sha256, 'revision': source.revision, 'features': counts})
    return {'format': 1, 'method': 'static-lexical-patterns', 'inventory': len(rows),
            'projects': dict(Counter(source.project for source in sources)),
            'input_manifests': {name: digest(root/name) for name in ('corpus/reference-inputs.json', 'corpus/upstreams.json')},
            'scanner_sha256': digest(Path(__file__)),
            'source_inventory_sha256': hashlib.sha256(json.dumps([(s.id, s.sha256) for s in sources]).encode()).hexdigest(),
            'patterns': FEATURES, 'features': features, 'sources': rows,
            'limitations': ['Occurrences are spellings, not AST nodes or accepted language features.',
                            'Quoted strings and nested comments are masked; custom here-string bodies may produce false positives.',
                            'Legacy invalid UTF-8 is replaced for scanning only; this is not compiler source validation.',
                            'No source text is included. No compiler, reference executable, library or project script is run.',
                            'No parse, semantic, LLVM, native, SDK or full project acceptance is implied.']}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report', type=Path, default=ROOT/'artifacts/corpus-features.json')
    args = parser.parse_args()
    output = args.report.resolve()
    for forbidden in (ROOT/'reference', ROOT/'corpus/upstream'):
        if output.is_relative_to(forbidden.resolve()): parser.error('report must be outside source corpus')
    report = scan(ROOT)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'inventory': report['inventory'], 'projects': report['projects'],
                      'features': {key: value['files'] for key, value in report['features'].items()}, 'report': str(output)}))
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
