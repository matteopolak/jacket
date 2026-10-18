"""Choosing deterministic places in a Jai document to hover, complete, rename and edit."""
from __future__ import annotations

import re

KEYWORDS = frozenset('''if else then for while return break continue case switch struct enum union cast defer
using inline no_inline true false null new delete push_context remove xx it it_index context size_of type_of
type_info is_constant else if ifx operator interface
u8 u16 u32 u64 s8 s16 s32 s64 int float float32 float64 bool string void Type Any trunc no_check'''.split())


def mask_line(line: str, block_depth: int = 0) -> tuple[str, int]:
    """`line` with comments and string contents replaced by spaces (same length), and the block comment depth
    left open at its end. Good enough for choosing positions; it does not try to be a lexer."""
    out = []
    i = 0
    in_string = False
    while i < len(line):
        two = line[i:i + 2]
        if block_depth:
            if two == '/*':
                block_depth += 1
                out.append('  ')
                i += 2
            elif two == '*/':
                block_depth -= 1
                out.append('  ')
                i += 2
            else:
                out.append(' ')
                i += 1
        elif in_string:
            if line[i] == '\\':
                out.append('  ')
                i += 2
                continue
            if line[i] == '"':
                in_string = False
                out.append('"')
            else:
                out.append(' ')
            i += 1
        elif two == '//':
            out.append(' ' * (len(line) - i))
            break
        elif two == '/*':
            block_depth = 1
            out.append('  ')
            i += 2
        elif line[i] == '"':
            in_string = True
            out.append('"')
            i += 1
        else:
            out.append(line[i])
            i += 1
    return ''.join(out), block_depth


PROC_START = re.compile(r'^([A-Za-z_]\w*)\s*::\s*\(.*\{\s*$')
# A header whose `{` is on the next line, alone at column 0 (Allman style).
PROC_HEADER = re.compile(r'^([A-Za-z_]\w*)\s*::\s*\(.*\)[^{;]*$')
IDENT = re.compile(r'(?<![\w.#$@])[A-Za-z_]\w*')
MEMBER = re.compile(r'(?<![\w.$])([A-Za-z_]\w*)\.([A-Za-z_]\w*)')
# `cast,trunc(T)` is a cast with a modifier, not a call.
CALL = re.compile(r'(?<![\w.#$@])(?<!cast,)([A-Za-z_]\w*)\(')


def utf16_col(line: str, col: int) -> int:
    """LSP column (UTF-16 code units) of the character index `col` in `line`."""
    return len(line[:col].encode('utf-16-le')) // 2


def procedure_bodies(masked: list[str]) -> list[tuple[int, int, str, int]]:
    """(first line, closing line, name, name column) of each top-level procedure whose `{` ends its header
    line, or is alone at column 0 on the next line, and whose closing `}` is alone at column 0."""
    bodies = []
    start = None
    for number, line in enumerate(masked):
        if start is None:
            match = PROC_START.match(line)
            header = PROC_HEADER.match(line)
            if match:
                start = (number, match.group(1))
            elif header and number + 1 < len(masked) and masked[number + 1].rstrip() == '{':
                start = (number, header.group(1))
        elif number == start[0] + 1 and line.rstrip() == '{':
            continue
        elif line.rstrip() in ('}', '};'):
            bodies.append((start[0], number, start[1], 0))
            start = None
        elif PROC_START.match(line) is None and line[:1] not in (' ', '\t', '') and line.strip():
            # A new top-level declaration before any closer: the header was not a body.
            start = None
            match = PROC_START.match(line)
            if match:
                start = (number, match.group(1))
    return bodies


def pick_positions(text: str, fraction: float = 0.6) -> dict | None:
    """Deterministic positions inside procedure bodies, `fraction` of the way through the candidates:

    hover: an identifier that is not a keyword, a member or a directive (line, utf-16 column, name);
    insert_line/indent: where to add a line inside the same body (the line after the hover's);
    declaration: the name of the procedure that body belongs to (for references);
    member: a `base.field` use inside a body, column just after the dot (None when there is none);
    call: a `name(` call inside a body, column just after the parenthesis (None when there is none).
    """
    lines = text.split('\n')
    masked = []
    depth = 0
    for line in lines:
        m, depth = mask_line(line.rstrip('\r'), depth)
        masked.append(m)
    bodies = procedure_bodies(masked)
    identifiers = []
    members = []
    calls = []
    for first, last, name, _ in bodies:
        for number in range(first + 1, last):
            line = masked[number]
            for m in IDENT.finditer(line):
                word = m.group(0)
                if len(word) >= 3 and word not in KEYWORDS and not line[m.end():].lstrip().startswith('::'):
                    identifiers.append((number, m.start(), word, (first, last, name)))
            for m in CALL.finditer(line):
                if m.group(1) not in KEYWORDS:
                    calls.append((number, m.end(), m.group(1)))
            for m in MEMBER.finditer(line):
                if m.group(1) not in KEYWORDS:
                    members.append((number, m.start(2), m.group(1), (first, last, name)))
    if not identifiers:
        return None
    number, col, word, body = identifiers[min(len(identifiers) - 1, int(len(identifiers) * fraction))]
    alternatives = []
    for other in (0.35, 0.5, 0.7, 0.8, 0.9, 0.2, 0.1):
        n, c, w, _ = identifiers[min(len(identifiers) - 1, int(len(identifiers) * other))]
        candidate = {'line': n, 'character': utf16_col(lines[n].rstrip('\r'), c), 'name': w}
        if w != word and candidate not in alternatives:
            alternatives.append(candidate)
    raw = lines[number].rstrip('\r')
    indent = raw[:len(raw) - len(raw.lstrip(' \t'))]
    pick = {
        'hover': {'line': number, 'character': utf16_col(raw, col), 'name': word},
        'hover_alternatives': alternatives,
        'insert_line': number + 1,
        'indent': indent,
        'declaration': {'line': body[0], 'character': 0, 'name': body[2]},
        'member': None,
        'call': None,
    }
    if members:
        mnumber, mcol, base, _ = members[min(len(members) - 1, int(len(members) * (fraction - 0.1)))]
        pick['member'] = {'line': mnumber, 'character': utf16_col(lines[mnumber].rstrip('\r'), mcol), 'base': base}
    if calls:
        cnumber, ccol, callee = calls[min(len(calls) - 1, int(len(calls) * fraction))]
        pick['call'] = {'line': cnumber, 'character': utf16_col(lines[cnumber].rstrip('\r'), ccol), 'name': callee}
    return pick


def insert_edit(line: int, character: int, text: str) -> dict:
    """An incremental `contentChanges` entry that inserts `text` at a position."""
    return {'range': {'start': {'line': line, 'character': character},
                      'end': {'line': line, 'character': character}}, 'text': text}


def replace_edit(first_line: int, first_character: int, last_line: int, last_character: int, text: str) -> dict:
    """An incremental `contentChanges` entry that replaces a range with `text`."""
    return {'range': {'start': {'line': first_line, 'character': first_character},
                      'end': {'line': last_line, 'character': last_character}}, 'text': text}


def delete_lines_edit(first: int, count: int = 1) -> dict:
    return {'range': {'start': {'line': first, 'character': 0}, 'end': {'line': first + count, 'character': 0}},
            'text': ''}


def typing_script(indent: str, word: str) -> list[tuple[int, str]]:
    """What typing `word` one character at a time inserts: (column to insert at, character), after the
    indentation is already in place."""
    return [(len(indent) + i, ch) for i, ch in enumerate(word)]
