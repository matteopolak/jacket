"""Markdown for the `compile` subcommand."""
from __future__ import annotations

from collections import defaultdict

from bench.common import machine_line, median_text, skipped_section, spread, table
from bench.results import Report, Result, Status
from bench.stages import Bucket
from bench.stats import Stat, ratio

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
        sha = f", sha256 `{info['sha256'][:12]}`" if info.get('sha256') else ''
        out.append(f"- `{name}`: {info['dialect']} dialect, `{info['path']}`{sha}")
    out.append('')
    groups = grouped(report)
    out += timing_tables(groups, compilers)
    out += stage_table(groups, compilers)
    out += detail_table(groups)
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


STAGE_NOTE = ('Every compiler builds the program through the same metaprogram, which timestamps the compiler\'s '
              'phase messages (see docs/tools/benchmarks.md). The messages cannot tell parsing, type checking and '
              'compile-time execution apart, so all of that is `front end` and `compile-time execution` is `-`; '
              '`codegen` includes writing the objects, and `link` is the system linker the metaprogram runs itself, '
              'identical work for every compiler. `total` is the wall time of that run, `other` what no phase accounts for '
              '(start-up, the metaprogram itself, exit).')
DETAIL_NOTE = ('The compiler\'s own `-timings` report from the direct runs above. Only binaries with the extra reporting '
               'flags print one, so these numbers are not comparable with a plain binary\'s and the table has no ratios.')


def stage_table(groups: dict[tuple[str, str], dict[str, Result]], compilers: list[str]) -> list[str]:
    rows = []
    for (workload, mode), results in groups.items():
        if mode.startswith('run-'):
            continue
        timed = {name: r for name, r in results.items() if r.status is Status.OK and r.stages}
        for name, result in timed.items():
            rows.append([f'{workload}/{mode}', name] + [stage_cell(result.stages, b) for b in STAGE_ORDER])
        if len(timed) > 1:
            base = timed.get(compilers[0])
            for name, other in timed.items():
                if base and other is not base:
                    rows.append([f'{workload}/{mode}', f'{name} / {compilers[0]}']
                                + [stage_ratio(other.stages, base.stages, b) for b in STAGE_ORDER])
    if not rows:
        return []
    return ['## Stages (seconds)', '', STAGE_NOTE, '', table(['workload/mode', 'compiler'] + [b.value for b in STAGE_ORDER],
                                                           rows, left_columns=2), '']


def detail_table(groups: dict[tuple[str, str], dict[str, Result]]) -> list[str]:
    rows = []
    for (workload, mode), results in groups.items():
        for name, result in results.items():
            if result.status is Status.OK and result.detail:
                rows.append([f'{workload}/{mode}', name] + [stage_cell(result.detail, b) for b in STAGE_ORDER])
    if not rows:
        return []
    return ['## Compiler-reported stages: `-timings` (seconds)', '', DETAIL_NOTE, '',
            table(['workload/mode', 'compiler'] + [b.value for b in STAGE_ORDER], rows, left_columns=2), '']


def stage_cell(stages: dict[str, Stat], bucket: Bucket) -> str:
    stat = stages.get(bucket.value)
    return '-' if stat is None else f'{stat.median:.3f}'


def stage_ratio(stages: dict[str, Stat], base: dict[str, Stat], bucket: Bucket) -> str:
    a, b = stages.get(bucket.value), base.get(bucket.value)
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
