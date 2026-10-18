"""What every subcommand shares: command-line options, output files, the regression check, tables."""
from __future__ import annotations

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import os
import sys
from dataclasses import dataclass
from pathlib import Path

from compare import Limits, find_regressions
from repo_output import output_dir
from results import Report, Status, load_report
from stats import Stat


@dataclass(frozen=True)
class Defaults:
    """Per-subcommand defaults for the shared options."""
    repeat: int = 5
    warmup: int = 1
    threshold: float = 0.10
    min_seconds: float = 0.10
    timeout: float = 900


def add_common_arguments(parser: argparse.ArgumentParser, tool: str, defaults: Defaults) -> None:
    parser.add_argument('--repeat', type=int, default=defaults.repeat, help='timed runs per result (the median is reported)')
    parser.add_argument('--warmup', type=int, default=defaults.warmup,
                        help='untimed runs first, to fill the OS file cache (0 to skip)')
    parser.add_argument('--only', default='', help='comma-separated substrings; run only results whose key matches one')
    parser.add_argument('--timeout', type=float, default=defaults.timeout, help='seconds before a single run is killed')
    parser.add_argument('--list', action='store_true', help='print what would run and exit')
    default_json = output_dir('bench', f'{tool}.json')
    parser.add_argument('--out', type=Path, default=default_json, help='JSON results (default: %(default)s)')
    parser.add_argument('--markdown', type=Path, default=None,
                        help='Markdown summary (default: next to --out, with .md)')
    parser.add_argument('--compare', type=Path, help='JSON from an earlier --out; exit 1 on regressions')
    parser.add_argument('--threshold', type=float, default=defaults.threshold,
                        help='relative growth that counts as a regression (default: %(default)s)')
    parser.add_argument('--min-seconds', type=float, default=defaults.min_seconds,
                        help='ignore smaller absolute time growth (default: %(default)s)')
    parser.add_argument('--min-mib', type=float, default=16, help='ignore smaller absolute RSS growth')


def default_binary(name: str) -> Path:
    """`<target>/release/<name>`, where target is $CARGO_TARGET_DIR or this checkout's target/."""
    return Path(os.environ.get('CARGO_TARGET_DIR') or output_dir()) / 'release' / name


def selected(only: str, key: str) -> bool:
    terms = [term.strip() for term in only.split(',') if term.strip()]
    return not terms or any(term in key for term in terms)


def finish(report: Report, markdown: str, args: argparse.Namespace) -> int:
    """Write the JSON and Markdown, print the summary, check regressions; the exit status."""
    markdown_path = args.markdown or args.out.with_suffix('.md')
    report.write(args.out, markdown_path, markdown)
    print(markdown)
    print(f'wrote {args.out} and {markdown_path}', file=sys.stderr)
    status = 0
    if report.failures():
        status = 1
    if args.compare:
        limits = Limits(args.threshold, args.min_seconds, args.min_mib)
        regressions = find_regressions(args.baseline, report, limits)
        for regression in regressions:
            print(f'REGRESSION {regression}')
        if regressions:
            status = 1
    return status


def print_progress(key: str, text: str) -> None:
    print(f'{key:44} {text}', file=sys.stderr, flush=True)


# ------------------------------------------------------------------------------------------ tables

def table(headers: list[str], rows: list[list[str]], left_columns: int = 1) -> str:
    """A Markdown table; the first `left_columns` columns are left-aligned, the rest right-aligned."""
    align = ['---'] * left_columns + ['---:'] * (len(headers) - left_columns)
    lines = ['| ' + ' | '.join(headers) + ' |', '|' + '|'.join(align) + '|']
    lines += ['| ' + ' | '.join(row) + ' |' for row in rows]
    return '\n'.join(lines)


def spread(stat: Stat | None, scale: float = 1.0, digits: int = 2) -> str:
    """`median (min-max)` of a stat, or `-`."""
    if stat is None:
        return '-'
    return f'{stat.median * scale:.{digits}f} ({stat.min * scale:.{digits}f}-{stat.max * scale:.{digits}f})'


def median_text(stat: Stat | None, scale: float = 1.0, digits: int = 2) -> str:
    return '-' if stat is None else f'{stat.median * scale:.{digits}f}'


def machine_line(report: Report) -> str:
    m = report.machine
    return (f"Machine: {m.get('cpu') or m.get('machine')}, {m.get('cpus')} cores, {m.get('memory_gib')} GiB, "
            f"{m.get('system')} {m.get('release')}.")


def skipped_section(report: Report) -> str:
    lines = [f'- `{key}`: {r.status.value}: {r.reason}' for key, r in report.results.items()
             if r.status is not Status.OK]
    return '\n'.join(['', 'Not timed:', '', *lines, '']) if lines else ''


def check_arguments(args: argparse.Namespace) -> None:
    """Stop before any timing on options that cannot work, including a --compare file in another format."""
    if args.repeat < 1:
        sys.exit('--repeat must be at least 1')
    if args.compare:
        try:
            args.baseline = load_report(args.compare)
        except (OSError, ValueError) as error:
            sys.exit(f'--compare: {error}')
