#!/usr/bin/env python3
"""Measure the shape of real Jai code: procedure and struct sizes, control-flow mix, polymorphism, ...

    python3 tools/jaistats.py [DIR ...] [--out tools/corpus-shape.json] [--max-file-lines 20000]

Scans every `.jai` file under the given directories (default `corpus/upstream`) with a lightweight
lexer (no parsing), and writes the distributions that `tools/jaibench.py` samples from. Comments and
string contents are blanked first, so keywords inside them do not count. The numbers are approximate on
purpose: the lexer is good enough to find `name :: (...) {` bodies and `struct`/`enum` declarations.
"""
from __future__ import annotations

import argparse
import json
import re
import statistics
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def blank(text: str) -> str:
    """Replace comments and string/char literal contents with spaces, keeping newlines and quotes."""
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if text.startswith('//', i):
            j = text.find('\n', i)
            j = n if j < 0 else j
            out.append(' ' * (j - i)); i = j
        elif text.startswith('/*', i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith('/*', j): depth += 1; j += 2
                elif text.startswith('*/', j): depth -= 1; j += 2
                else: j += 1
            out.append(re.sub(r'[^\n]', ' ', text[i:j])); i = j
        elif text.startswith('#string', i):
            m = re.match(r'#string[ \t]+(\w+)[^\n]*\n', text[i:])
            if m:
                end = text.find(m.group(1), i + m.end())
                end = n if end < 0 else end + len(m.group(1))
                body = text[i:end]
                out.append('#string' + re.sub(r'[^\n]', ' ', body[7:])); i = end
            else:
                out.append(c); i += 1
        elif c == '"':
            j = i + 1
            while j < n and text[j] != '"' and text[j] != '\n':
                j += 2 if text[j] == '\\' else 1
            out.append('"' + ' ' * max(0, j - i - 1) + '"'); i = j + 1
        else:
            out.append(c); i += 1
    return ''.join(out)


def match_brace(s: str, open_at: int) -> int:
    depth = 0
    for k in range(open_at, len(s)):
        ch = s[k]
        if ch == '{': depth += 1
        elif ch == '}':
            depth -= 1
            if depth == 0: return k
    return -1


PROC = re.compile(r'(?m)^[ \t]*(\w+)\s*::\s*(?:inline\s+|no_inline\s+)?(\([^{;]*?\))\s*(->[^{;]*?)?\s*((?:#\w+(?:\([^)]*\))?\s*)*)\{')
STRUCT = re.compile(r'(?m)^[ \t]*(\w+)\s*::\s*(struct|union|enum|enum_flags)\b([^{;]*)\{')
SHORT = [('if', r'\bif\b'), ('else', r'\belse\b'), ('for', r'\bfor\b'), ('while', r'\bwhile\b'),
         ('case', r'^\s*case\b'), ('switch', r'\bif\s+[^{;]*?==\s*\{'), ('return', r'\breturn\b'),
         ('defer', r'\bdefer\b'), ('break_continue', r'\b(?:break|continue)\b'),
         ('cast', r'\bcast\b|\bxx\b'), ('using', r'\busing\b'), ('print', r'\b(?:print|tprint|sprint|log)\s*\('),
         ('array_add', r'\barray_add\b'), ('context', r'\bcontext\.'), ('new_free', r'\b(?:New|alloc|free|NewArray)\b'),
         ('remove', r'\bremove\b'), ('struct_lit', r'\.\{'), ('array_lit', r'\.\['),
         ('ptr_deref', r'<<|\*\w'), ('member', r'\w\.\w')]
BINOP = re.compile(r'(?<![<>=!+\-*/%&|^:])(?:\+|-|\*|/|%|<<|>>|&&|\|\||==|!=|<=|>=|<|>|&|\||\^)(?![=:>])')


def quantiles(values: list[float]) -> dict:
    if not values:
        return {}
    v = sorted(values)
    q = lambda p: v[min(len(v) - 1, int(p * len(v)))]
    return {'n': len(v), 'mean': round(statistics.fmean(v), 2), 'p10': q(.1), 'p25': q(.25), 'p50': q(.5),
            'p75': q(.75), 'p90': q(.9), 'p99': q(.99), 'max': v[-1]}


def histogram(values: list[int], edges: list[int]) -> list[float]:
    """Share of values per bucket [edges[i], edges[i+1]) (the last bucket is open-ended)."""
    counts = [0] * len(edges)
    for x in values:
        k = max(i for i, e in enumerate(edges) if x >= e) if x >= edges[0] else 0
        counts[k] += 1
    total = sum(counts) or 1
    return [round(c / total, 4) for c in counts]


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('dirs', nargs='*', type=Path)
    ap.add_argument('--out', type=Path)
    ap.add_argument('--max-file-lines', type=int, default=20000, help='skip bigger files (generated bindings)')
    a = ap.parse_args()
    dirs = a.dirs or [ROOT / 'corpus' / 'upstream']
    files = sorted(p for d in dirs for p in d.rglob('*.jai'))
    code_lines = 0
    nfiles = 0
    proc_stmts, proc_lines, proc_params, proc_poly, proc_macro = [], [], [], 0, 0
    nprocs = 0
    struct_fields, enum_members, nstructs, nenums, poly_structs = [], [], 0, 0, 0
    flow = Counter(); body_lines_total = 0
    directives = Counter(); imports = Counter(); strings = 0; ops_per_stmt = []
    ret_kinds = Counter(); param_types = Counter(); field_types = Counter(); for_kinds = Counter()
    nested = []
    for path in files:
        try:
            raw = path.read_text(errors='replace')
        except OSError:
            continue
        if raw.count('\n') > a.max_file_lines:
            continue
        nfiles += 1
        text = blank(raw)
        lines = [l for l in text.splitlines() if l.strip()]
        code_lines += len(lines)
        strings += len(re.findall(r'"[^"\n]*"', text))
        for d in re.findall(r'#(\w+)', text):
            directives[d] += 1
        for m in re.finditer(r'#import\s+"([^"]+)"', raw):
            imports[m.group(1).split('/')[0]] += 1
        for m in PROC.finditer(text):
            params, ret = m.group(2), m.group(3) or ''
            end = match_brace(text, m.end() - 1)
            if end < 0: continue
            body = text[m.end():end]
            blines = [l for l in body.splitlines() if l.strip()]
            if not blines: continue
            nprocs += 1
            proc_lines.append(len(blines))
            stmts = body.count(';') + len(re.findall(r'\bif\b|\bfor\b|\bwhile\b', body))
            proc_stmts.append(stmts)
            plist = [p for p in re.split(r',(?![^()]*\))', params[1:-1]) if p.strip()]
            proc_params.append(len(plist))
            for p in plist:
                t = p.split(':', 1)[1].strip().split('=')[0].strip() if ':' in p else '?'
                param_types[re.sub(r'\$', '', t)[:24]] += 1
            if '$' in params or '#modify' in m.group(4): proc_poly += 1
            if '#expand' in m.group(4): proc_macro += 1
            ret_kinds[ret.replace('->', '').strip().split(',')[0][:20] or 'void'] += 1
            for name, pat in SHORT:
                flow[name] += len(re.findall(pat, body, re.M))
            for fk in re.findall(r'\bfor\s*(\*?)\s*(?:\w+\s*,\s*\w+\s+in\s+|\w+\s+in\s+)?([^{;]*?)\{', body):
                expr = fk[1]
                for_kinds['range' if '..' in expr else 'array'] += 1
            depth = mx = 0
            for ch in body:
                if ch == '{': depth += 1; mx = max(mx, depth)
                elif ch == '}': depth -= 1
            nested.append(mx)
            for stmt in body.split(';'):
                if stmt.strip():
                    ops_per_stmt.append(min(12, len(BINOP.findall(stmt))))
            body_lines_total += len(blines)
        for m in STRUCT.finditer(text):
            kind = m.group(2)
            end = match_brace(text, m.end() - 1)
            if end < 0: continue
            body = text[m.end():end]
            if kind.startswith('enum'):
                nenums += 1
                enum_members.append(len([x for x in re.split(r'[;\n,]', body) if x.strip()]))
            else:
                nstructs += 1
                if '(' in m.group(3): poly_structs += 1
                fields = re.findall(r'(?m)^\s*(\w+(?:\s*,\s*\w+)*)\s*:\s*([^=;\n]+?)\s*(?:=[^;\n]*)?;', body)
                n = sum(len(f[0].split(',')) for f in fields)
                struct_fields.append(n)
                for f in fields:
                    field_types[f[1].strip()[:20]] += 1
    kloc = code_lines / 1000
    per_stmt = max(1, sum(proc_stmts))
    shape = {
        'files': nfiles, 'code_lines': code_lines, 'procs': nprocs, 'structs': nstructs, 'enums': nenums,
        'procs_per_kloc': round(nprocs / kloc, 2), 'structs_per_kloc': round(nstructs / kloc, 2),
        'enums_per_kloc': round(nenums / kloc, 2),
        'proc_body_lines': quantiles(proc_lines),
        'proc_body_lines_hist': {'edges': [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144],
                                 'share': histogram(proc_lines, [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144])},
        'proc_statements': quantiles(proc_stmts),
        'proc_params': {'hist': histogram(proc_params, [0, 1, 2, 3, 4, 5, 6])},
        'proc_max_nesting': quantiles(nested),
        'proc_body_share_of_code': round(body_lines_total / max(1, code_lines), 3),
        'struct_fields': quantiles(struct_fields),
        'struct_fields_hist': {'edges': [0, 1, 2, 3, 4, 6, 9, 14, 24],
                               'share': histogram(struct_fields, [0, 1, 2, 3, 4, 6, 9, 14, 24])},
        'enum_members': quantiles(enum_members),
        'poly_proc_share': round(proc_poly / max(1, nprocs), 4),
        'macro_proc_share': round(proc_macro / max(1, nprocs), 4),
        'poly_struct_share': round(poly_structs / max(1, nstructs), 4),
        'flow_per_statement': {k: round(v / per_stmt, 4) for k, v in flow.items()},
        'for_kinds': {k: round(v / max(1, sum(for_kinds.values())), 3) for k, v in for_kinds.items()},
        'binops_per_statement': {'mean': round(statistics.fmean(ops_per_stmt), 3) if ops_per_stmt else 0,
                                 'hist': histogram(ops_per_stmt, list(range(0, 8)))},
        'string_literals_per_kloc': round(strings / kloc, 1),
        'directives_per_kloc': {k: round(v / kloc, 2) for k, v in directives.most_common(30)},
        'imports': dict(imports.most_common(40)),
        'return_types': dict(ret_kinds.most_common(12)),
        'param_types': dict(param_types.most_common(16)),
        'field_types': dict(field_types.most_common(16)),
    }
    text = json.dumps(shape, indent=2) + '\n'
    if a.out:
        a.out.write_text(text)
    print(text)


if __name__ == '__main__':
    main()
