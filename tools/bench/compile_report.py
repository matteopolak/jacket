"""Markdown for the `compile` subcommand."""
from __future__ import annotations

from collections import defaultdict

from common import machine_line, median_text, skipped_section, spread, table
from results import Report, Result, Status
from stages import Bucket
from stats import ratio

STAGE_ORDER = [Bucket.FRONT_END, Bucket.COMPILE_TIME, Bucket.CODEGEN, Bucket.LINK, Bucket.OTHER, Bucket.TOTAL]


def split_key(key: str) -> tuple[str, str, str]:
    workload, mode, compiler = key.split('/')
    return workload, mode, compiler


def grouped(report: Report) -> dict[tuple[str, str], dict[str, Result]]:
    """Results by (workload, mode), then compiler, in the order they ran."""
    groups: dict[tuple[str, str], dict[str, Result]] = defaultdict(dict)
    for key, result in report.results.items():
        workload, mode, compiler = split_key(key)
        groups[(workload, mode)][compiler] = result
    return groups


def lines_of(results: dict[str, Result]) -> int:
    first = next(iter(results.values()))
    return first.info.get('source', {}).get('lines', 0)


def ratio_text(value: float | None) -> str:
    return '-' if value is None else f'{value:.2f}x'


def render(report: Report) -> str:
    compilers = list(report.subjects)
    out = [f"# Compile benchmark ({report.date[:10]})", '', machine_line(report), '',
           f"{report.settings['repeat']} timed runs per result after {report.settings['warmup']} untimed "
           '(wall time is the median, the range is min-max; CPU is user + system). Compilers run interleaved.', '']
    for name, info in report.subjects.items():
        commit = f", `{info['commit'][:12]}`{' (dirty)' if info.get('dirty') else ''}" if info.get('commit') else ''
        out.append(f"- `{name}`: {info['dialect']} dialect, `{info['path']}`{commit}")
    out.append('')
    groups = grouped(report)
    out += timing_tables(groups, compilers)
    out += stage_table(groups, compilers)
    out += info_table(groups, compilers, 'binary_bytes', 'Executable size (KiB)', lambda v: f'{v / 1024:.0f}')
    out += info_table(groups, compilers, 'interp_instructions', 'Interpreted instructions (millions)',
                      lambda v: f'{v / 1e6:.3f}')
    out.append(skipped_section(report))
    return '\n'.join(out).rstrip() + '\n'


def timing_tables(groups: dict[tuple[str, str], dict[str, Result]], compilers: list[str]) -> list[str]:
    by_category: dict[str, list[tuple[tuple[str, str], dict[str, Result]]]] = defaultdict(list)
    for key, results in groups.items():
        category = next(iter(results.values())).info.get('category', 'other')
        by_category[category].append((key, results))
    out = []
    for category, entries in by_category.items():
        headers = ['workload', 'mode', 'code lines']
        for name in compilers:
            headers += [f'{name} wall s', f'{name} cpu s', f'{name} RSS MiB']
        if len(compilers) > 1:
            headers += [f'{name} / {compilers[0]}' for name in compilers[1:]]
        else:
            headers.append('s per 1k lines')
        rows = []
        for (workload, mode), results in entries:
            row = [workload, mode, str(lines_of(results)) if lines_of(results) else '-']
            for name in compilers:
                r = results.get(name)
                timed = r if r and r.status is Status.OK else None
                row += [spread(timed.wall, digits=3) if timed else (r.status.value if r else '-'),
                        median_text(timed.cpu, digits=3) if timed else '-',
                        median_text(timed.rss_mib, digits=0) if timed else '-']
            first = results.get(compilers[0])
            if len(compilers) > 1:
                for name in compilers[1:]:
                    other = results.get(name)
                    both = first and other and first.wall and other.wall
                    row.append(ratio_text(ratio(other.wall.median, first.wall.median)) if both else '-')
            else:
                lines = lines_of(results)
                row.append(f'{first.wall.median / (lines / 1000):.3f}' if first and first.wall and lines else '-')
            rows.append(row)
        out += [f'## {category}', '', table(headers, rows, left_columns=2), '']
    return out


def stage_table(groups: dict[tuple[str, str], dict[str, Result]], compilers: list[str]) -> list[str]:
    rows = []
    for (workload, mode), results in groups.items():
        if mode.startswith('run-'):
            continue
        timed = {name: r for name, r in results.items() if r.status is Status.OK and r.stages}
        for name, result in timed.items():
            rows.append([f'{workload}/{mode}', name] + [stage_cell(result, b) for b in STAGE_ORDER])
        if len(timed) > 1:
            base = timed.get(compilers[0])
            for name, other in timed.items():
                if base and other is not base:
                    rows.append([f'{workload}/{mode}', f'{name} / {compilers[0]}']
                                + [stage_ratio(other, base, b) for b in STAGE_ORDER])
    if not rows:
        return []
    note = ('Stages come from each compiler\'s own timing output, mapped onto common buckets (see '
            'docs/tools/benchmarks.md). A bucket a compiler does not report is `-`. The total is the wall time of '
            'the process; other is what no reported bucket accounts for, start-up and exit included.')
    return ['## Stages (seconds)', '', note, '', table(['workload/mode', 'compiler'] + [b.value for b in STAGE_ORDER],
                                                     rows, left_columns=2), '']


def stage_cell(result: Result, bucket: Bucket) -> str:
    stat = result.stages.get(bucket.value)
    return '-' if stat is None else f'{stat.median:.3f}'


def stage_ratio(result: Result, base: Result, bucket: Bucket) -> str:
    a, b = result.stages.get(bucket.value), base.stages.get(bucket.value)
    return ratio_text(ratio(a.median, b.median)) if a and b else '-'


def info_table(groups: dict[tuple[str, str], dict[str, Result]], compilers: list[str], field: str, title: str,
               show) -> list[str]:
    """A table of one number from the results' `info`, a row per workload/mode that has it."""
    rows = []
    for (workload, mode), results in groups.items():
        values = [results[name].info.get(field) if name in results else None for name in compilers]
        if any(values):
            rows.append([f'{workload}/{mode}'] + ['-' if v is None else show(v) for v in values])
    return [f'## {title}', '', table(['workload/mode'] + compilers, rows), ''] if rows else []
