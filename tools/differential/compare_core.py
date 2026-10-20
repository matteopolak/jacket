"""The parts of `compare_compilers.py` that need no compiler: what a run records, how two runs are classified,
how diagnostics are normalised, how disagreements are clustered, and a line-based reducer."""
from __future__ import annotations

import re
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

# Outcome statuses.
OK = 'ok'                # compiled and (if it was run) exited 0
RUNTIME = 'runtime'      # compiled; the program exited non-zero
REJECTED = 'rejected'    # the compiler reported an error
TIMEOUT = 'timeout'
CRASH = 'crash'          # the compiler died on a signal or panicked

# Classes of a pair of outcomes.
AGREE = 'agree'
BOTH_REJECT = 'both_reject'
A_ONLY = 'a_only'
B_ONLY = 'b_only'
OUTPUT_DIFFERS = 'output_differs'
FAILURE = 'failure'      # either side timed out or crashed
CLASSES = (A_ONLY, B_ONLY, OUTPUT_DIFFERS, FAILURE, BOTH_REJECT, AGREE)
DISAGREEMENTS = (A_ONLY, B_ONLY, OUTPUT_DIFFERS, FAILURE)


@dataclass(frozen=True)
class Program:
    """One generated or rewritten program. `tags` are `key:value` features the generator attached;
    `root` is a folder whose entries are linked next to the program (a rewrite of a file that loads siblings)."""
    name: str
    source: str
    tags: tuple[str, ...] = ()
    root: Path | None = None
    filename: str = 'prog.jai'


@dataclass
class Outcome:
    status: str
    exit_code: int | None = None
    stdout: str = ''
    error: str = ''   # first error line, normalised
    raw_error: str = ''

    @property
    def compiled(self) -> bool:
        return self.status in (OK, RUNTIME)

    def to_json(self) -> dict:
        return {'status': self.status, 'compiled': self.compiled, 'exit': self.exit_code,
                'stdout': self.stdout, 'error': self.error}


ANSI = re.compile(r'\x1b\[[0-9;]*[A-Za-z]')
LOCATION_PREFIX = re.compile(r'^\s*(?:[^\s:]|:(?!\d))*?:\d+(?::\d+)?:\s*')    # `path:12:5: `
PATHS = re.compile(r'(?:[A-Za-z]:)?[/\\]?(?:[\w.@+-]+[/\\])+[\w.@+-]+|\b[\w.-]+\.jai\b')
LINE_COL = re.compile(r'\b\d+:\d+\b|\(\d+,\d+\)')
DIAGNOSTIC = re.compile(r'\berror\b|\bpanicked at\b|\bInternal compiler error\b', re.IGNORECASE)
CRASH_TEXT = re.compile(r'panicked at|internal compiler error|stack overflow|SIGSEGV|SIGABRT', re.IGNORECASE)
TOOL_NOISE = ('jaic-timing:', 'jaic-alloc:', 'jaic-profile')


def strip_colour(text: str) -> str:
    return ANSI.sub('', text)


def normalise_error(line: str) -> str:
    """One diagnostic line without colour, the file, the line and column, and any path, so the same
    complaint worded the same compares equal wherever the program lives."""
    line = strip_colour(line).strip()
    line = LOCATION_PREFIX.sub('', line, count=1)
    line = PATHS.sub('FILE', line)
    line = LINE_COL.sub('L:C', line)
    return re.sub(r'\s+', ' ', line).strip()


def lines_of(text: str) -> list[str]:
    return [strip_colour(line) for line in text.splitlines()
            if line.strip() and not line.startswith(TOOL_NOISE)]


def first_error(stderr: str, stdout: str = '') -> str:
    """The first line that reports an error, normalised; else the first line of output; else ''."""
    for text in (stderr, stdout):
        lines = lines_of(text)
        for line in lines:
            if DIAGNOSTIC.search(line):
                return normalise_error(line)
    for text in (stderr, stdout):
        lines = lines_of(text)
        if lines:
            return normalise_error(lines[0])
    return ''


def has_diagnostic(stderr: str, stdout: str = '') -> bool:
    return any(DIAGNOSTIC.search(line) for text in (stderr, stdout) for line in lines_of(text))


def looks_crashed(stderr: str) -> bool:
    return bool(CRASH_TEXT.search(strip_colour(stderr)))


def same_result(a: Outcome, b: Outcome) -> bool:
    """Both compiled: the output must match; a non-zero exit is compared only as a fact, because backends
    word failures (and number their exit codes) differently."""
    if a.status != b.status or a.stdout != b.stdout:
        return False
    return a.status != OK or a.exit_code == b.exit_code


def classify(a: Outcome, b: Outcome) -> str:
    if TIMEOUT in (a.status, b.status) or CRASH in (a.status, b.status):
        return FAILURE
    if a.compiled and b.compiled:
        return AGREE if same_result(a, b) else OUTPUT_DIFFERS
    if a.compiled:
        return A_ONLY
    if b.compiled:
        return B_ONLY
    return BOTH_REJECT


@dataclass
class Record:
    program: Program
    a: Outcome
    b: Outcome
    verdict: str = ''
    reduced: str | None = None

    def __post_init__(self) -> None:
        self.verdict = classify(self.a, self.b)

    @property
    def disagrees(self) -> bool:
        return self.verdict in DISAGREEMENTS

    @property
    def diagnostics_differ(self) -> bool:
        return self.verdict == BOTH_REJECT and self.a.error != self.b.error

    def key_text(self) -> str:
        """What characterises the disagreement for clustering: the rejecting side's message, or the
        failing side's status."""
        if self.verdict == A_ONLY:
            return self.b.error
        if self.verdict == B_ONLY:
            return self.a.error
        if self.verdict == FAILURE:
            bad = [f'{side} {o.status}' for side, o in (('A', self.a), ('B', self.b)) if o.status in (TIMEOUT, CRASH)]
            return ', '.join(bad)
        if self.verdict == OUTPUT_DIFFERS:
            return 'output differs' if self.a.status == self.b.status else f'{self.a.status} vs {self.b.status}'
        return ''

    def to_json(self) -> dict:
        return {'name': self.program.name, 'tags': list(self.program.tags), 'verdict': self.verdict,
                'a': self.a.to_json(), 'b': self.b.to_json(), 'lines': self.program.source.count('\n')}


@dataclass
class Cluster:
    verdict: str
    key: str
    records: list[Record] = field(default_factory=list)

    @property
    def smallest(self) -> Record:
        return min(self.records, key=lambda r: (len(r.reduced or r.program.source), r.program.name))


def cluster_by_error(records: list[Record]) -> list[Cluster]:
    """Disagreements grouped by (verdict, characteristic message), largest first."""
    groups: dict[tuple[str, str], Cluster] = {}
    for record in records:
        if record.disagrees:
            key = (record.verdict, record.key_text())
            groups.setdefault(key, Cluster(*key)).records.append(record)
    return sorted(groups.values(), key=lambda c: (-len(c.records), c.verdict, c.key))


def cluster_rejections(records: list[Record]) -> list[Cluster]:
    """Programs both sides reject, grouped by A's message, leaving out the near misses a generator marks
    `variant:near_miss`: a rejected program that should have been valid is a generator bug or a gap both share."""
    groups: dict[str, Cluster] = {}
    for record in records:
        if record.verdict == BOTH_REJECT and 'variant:near_miss' not in record.program.tags:
            groups.setdefault(record.a.error, Cluster(BOTH_REJECT, record.a.error)).records.append(record)
    return sorted(groups.values(), key=lambda c: (-len(c.records), c.key))


def tag_totals(records: list[Record]) -> dict[str, int]:
    totals: dict[str, int] = defaultdict(int)
    for record in records:
        for tag in record.program.tags:
            totals[tag] += 1
    return dict(totals)


def cluster_by_tag(records: list[Record]) -> list[tuple[str, list[Record], int]]:
    """For every tag with at least one disagreement: (tag, its disagreeing records, how many programs carry it),
    the highest share of disagreements first. A tag every program carries (`axis:...`) sorts by its count."""
    totals = tag_totals(records)
    groups: dict[str, list[Record]] = defaultdict(list)
    for record in records:
        if record.disagrees:
            for tag in record.program.tags:
                groups[tag].append(record)
    return sorted(((tag, found, totals[tag]) for tag, found in groups.items()),
                  key=lambda g: (-len(g[1]) / g[2], -len(g[1]), g[0]))


def summarise(records: list[Record]) -> dict[str, int]:
    counts = {name: 0 for name in CLASSES}
    for record in records:
        counts[record.verdict] += 1
    counts['total'] = len(records)
    counts['diagnostics_differ'] = sum(1 for r in records if r.diagnostics_differ)
    return counts


def reduce_lines(lines: list[str], keeps: Callable[[list[str]], bool], budget: int = 200) -> list[str]:
    """Delete chunks of lines, halving the chunk size down to one line, while `keeps(candidate)` still holds;
    stops after `budget` calls. `keeps(lines)` must hold on entry."""
    tries = 0
    size = max(len(lines) // 2, 1)
    while True:
        index = 0
        progressed = False
        while index < len(lines):
            if tries >= budget:
                return lines
            tries += 1
            candidate = lines[:index] + lines[index + size:]
            if candidate and keeps(candidate):
                lines = candidate
                progressed = True
            else:
                index += size
        if size > 1:
            size //= 2
        elif not progressed:
            return lines
