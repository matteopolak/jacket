"""Small text-level edits of existing programs that should keep them valid.

Each rewrite finds sites line by line with a regular expression and yields one edited program per site
(at most `max_sites` per file and rewrite), tagged `rewrite:NAME` and `source:PATH`. There is no parser: a
rewrite can be wrong where the text is not what the pattern assumes (a pattern inside a block comment or
a multi-line string, a declaration spanning lines, a macro-like `#insert`). Lines that are comments or
contain a string literal are skipped by the rewrites that would mangle one, and the rewrites whose result is
often invalid are tagged `noisy:...`, so a report can be filtered on them. A rewrite that makes a program
invalid for both compilers is still an agreement, so noise costs time, not correctness.

To add a rewrite: write a function `(lines) -> Iterator[(line_index, new_lines, tag)]` and list it in
`REWRITES`.
"""
from __future__ import annotations

import re
from pathlib import Path
from typing import Callable, Iterable, Iterator

from compare_core import Program

Edit = tuple[int, list[str], str]    # (index of the edited line, the whole new text, extra tag or '')
SIMPLE_TYPE = r'(?:\*\s*)?[A-Za-z_]\w*(?:\.\w+)*'


def code_lines(lines: list[str]) -> Iterator[tuple[int, str]]:
    """The lines that are neither blank nor comments."""
    for index, line in enumerate(lines):
        if line.strip() and not line.lstrip().startswith('//'):
            yield index, line


def replace_line(lines: list[str], index: int, new: str) -> list[str]:
    return [*lines[:index], new, *lines[index + 1:]]


def erase_literal_type(lines: list[str]) -> Iterator[Edit]:
    """`x: T = T.{...}` -> `x: T = .{...}`, `return T.{...}` in a procedure returning T -> `return .{...}`.
    Any other `T.{` is erased too, tagged noisy: the expected type there may not be known."""
    returns = ''
    for index, line in code_lines(lines):
        header = re.search(rf'->\s*({SIMPLE_TYPE})\s*(?:#\w+\s*)*\{{', line)
        if header:
            returns = header[1]
        declared = re.search(rf'\b\w+\s*:\s*({SIMPLE_TYPE})\s*=\s*\1\.\{{', line)
        if declared:
            yield index, replace_line(lines, index, line.replace(f'= {declared[1]}.{{', '= .{', 1)), ''
            continue
        returned = re.search(rf'\breturn\s+({SIMPLE_TYPE})\.\{{', line)
        if returned and returned[1] == returns:
            yield index, replace_line(lines, index, line.replace(f'return {returns}.{{', 'return .{', 1)), ''
            continue
        any_literal = re.search(r'(?<![\w.])([A-Z]\w*)\.\{', line)
        if any_literal:
            yield (index, replace_line(lines, index, line.replace(f'{any_literal[1]}.{{', '.{', 1)),
                   'noisy:unknown_context')


def swap_commutative(lines: list[str]) -> Iterator[Edit]:
    """`a + b`, `a * b`, `a == b` and `a != b` on two plain operands (names, members, numbers) swap sides."""
    operand = r'[A-Za-z_][\w.]*|\d[\w.]*'
    pattern = re.compile(rf'(?<![\w.)\]])({operand}) (\+|\*|==|!=) ({operand})(?![\w.(\[])')
    for index, line in code_lines(lines):
        if '"' in line or "'" in line:
            continue
        match = pattern.search(line)
        if match:
            new = line[:match.start()] + f'{match[3]} {match[2]} {match[1]}' + line[match.end():]
            yield index, replace_line(lines, index, new), ''


def inline_single_use_local(lines: list[str]) -> Iterator[Edit]:
    """`x := e;` followed by exactly one other mention of `x` in the file: put `(e)` at the mention.
    Moves the evaluation of `e` later, so a side-effecting `e` can change the program (tagged noisy)."""
    text = '\n'.join(lines)
    for index, line in code_lines(lines):
        declared = re.match(r'^(\s*)(\w+) := (.+);\s*$', line)
        if not declared or '"' in line:
            continue
        name, value = declared[2], declared[3]
        mentions = [i for i, other in enumerate(lines) if i != index and re.search(rf'(?<![\w.]){name}\b', other)]
        if len(re.findall(rf'(?<![\w.]){name}\b', text)) != 2 or len(mentions) != 1 or mentions[0] < index:
            continue
        target = lines[mentions[0]]
        if re.search(rf'{name}\s*(?:=|\+=|-=|\*=)(?!=)', target) or f'*{name}' in target:
            continue
        edited = re.sub(rf'(?<![\w.]){name}\b', lambda _: f'({value})', target, count=1)
        new = [line for i, line in enumerate(lines) if i != index]
        new[mentions[0] - 1] = edited
        yield index, new, 'noisy:evaluation_order'


def typed_to_cast(lines: list[str]) -> Iterator[Edit]:
    """`x: T = e;` -> `x := cast(T) e;`. Not for `---`, a struct or array literal, or an `e` that already casts."""
    for index, line in code_lines(lines):
        match = re.match(rf'^(\s*)(\w+): ({SIMPLE_TYPE}) = (?!---|\.\{{|\.\[|cast)(.+);\s*$', line)
        if match:
            new = f'{match[1]}{match[2]} := cast({match[3]}) {match[4]};'
            yield index, replace_line(lines, index, new), ''


def infer_to_type_of(lines: list[str]) -> Iterator[Edit]:
    """`x := e;` -> `x: type_of(e) = e;`."""
    for index, line in code_lines(lines):
        match = re.match(r'^(\s*)(\w+) := (?!---)([^;{]+);\s*$', line)
        if match and '"' not in line:
            yield index, replace_line(lines, index, f'{match[1]}{match[2]}: type_of({match[3]}) = {match[3]};'), ''


def parenthesise_initialiser(lines: list[str]) -> Iterator[Edit]:
    """`x := e;` -> `x := (e);`"""
    for index, line in code_lines(lines):
        match = re.match(r'^(\s*)(\w+) := (?!---)([^;{]+);\s*$', line)
        if match:
            yield index, replace_line(lines, index, f'{match[1]}{match[2]} := ({match[3]});'), ''


def expand_compound_assignment(lines: list[str]) -> Iterator[Edit]:
    """`x += e;` -> `x = x + e;` for a plain name or member `x`."""
    for index, line in code_lines(lines):
        match = re.match(r'^(\s*)([A-Za-z_][\w.]*) (\+|-|\*)= ([^;]+);\s*$', line)
        if match:
            yield index, replace_line(lines, index, f'{match[1]}{match[2]} = {match[2]} {match[3]} ({match[4]});'), ''


REWRITES: dict[str, Callable[[list[str]], Iterator[Edit]]] = {
    'erase_literal_type': erase_literal_type,
    'swap_commutative': swap_commutative,
    'inline_single_use_local': inline_single_use_local,
    'typed_to_cast': typed_to_cast,
    'infer_to_type_of': infer_to_type_of,
    'parenthesise_initialiser': parenthesise_initialiser,
    'expand_compound_assignment': expand_compound_assignment,
}


def rewrite_text(text: str, source: str, root: Path | None = None, names: Iterable[str] | None = None,
                 max_sites: int = 3, stem: str = 'prog', filename: str = 'prog.jai') -> list[Program]:
    """The rewrites of one file's text. `source` labels the origin in tags and names."""
    lines = text.split('\n')
    if lines and lines[-1] == '':
        lines.pop()
    programs = []
    for name in names or REWRITES:
        for site, (index, new_lines, noise) in enumerate(REWRITES[name](lines)):
            if site >= max_sites:
                break
            tags = ('axis:rewrite', f'rewrite:{name}', f'source:{source}', *([noise] if noise else []))
            programs.append(Program(f'rewrite_{stem}__{name}__{index + 1}', '\n'.join(new_lines) + '\n', tags,
                                    root, filename))
    return programs


def rewrite_files(paths: Iterable[Path], names: Iterable[str] | None = None, max_sites: int = 3,
                  base: Path | None = None) -> list[Program]:
    """Rewrites of the programs in `paths` that define `main` (a file without it cannot be run)."""
    programs = []
    for path in paths:
        text = path.read_text(errors='replace')
        if not re.search(r'^main\s*::', text, re.MULTILINE):
            continue
        label = path.relative_to(base).as_posix() if base and path.is_relative_to(base) else path.name
        stem = re.sub(r'[^\w.-]+', '_', label.removesuffix('.jai'))
        programs += rewrite_text(text, label, path.parent, names, max_sites, stem, path.name)
    return programs
