"""Markdown for the `lsp` subcommand."""
from __future__ import annotations

from collections import defaultdict

from bench.common import machine_line, median_text, skipped_section, table
from bench.results import Report, Result, Status


def render(report: Report) -> str:
    out = [f"# Language server benchmark ({report.date[:10]})", '', machine_line(report), '',
           f"{report.settings['repeat']} sessions per workload after {report.settings['warmup']} untimed, each a fresh "
           f"server; diagnostics by {report.settings['diagnostics']}. Latencies are over all samples of all sessions: "
           'p50 is the median, p95 the 95th percentile. CPU is the server\'s user + system time per sample; RSS is its '
           'resident set after the step (peak for `session`).', '']
    for name, info in report.subjects.items():
        commit = f", `{info['commit'][:12]}`{' (dirty)' if info.get('dirty') else ''}" if info.get('commit') else ''
        out.append(f"- `{name}`: {info.get('version') or 'version unknown'}, `{' '.join(info['command'])}`{commit}")
    out.append('')
    groups: dict[str, dict[str, Result]] = defaultdict(dict)
    for key, result in report.results.items():
        server, workload, probe = key.split('/')
        groups[f'{server}/{workload}'][probe] = result
    for group, results in groups.items():
        session = results.get('session')
        document = f", document {session.info['document_lines']} lines" if session and session.info else ''
        out += [f'## {group}{document}', '', table(
            ['probe', 'p50 ms', 'p95 ms', 'min-max ms', 'samples', 'CPU ms', 'RSS MiB'],
            [row(name, result) for name, result in results.items() if result.status is Status.OK]), '']
    out.append(skipped_section(report))
    return '\n'.join(out).rstrip() + '\n'


def row(name: str, result: Result) -> list[str]:
    wall = result.wall
    scale = 1000
    if name == 'session':  # seconds of the whole session and its CPU
        return [name, median_text(wall) + ' s', '-', f'{wall.min:.2f}-{wall.max:.2f} s', str(wall.n),
                median_text(result.cpu) + ' s', median_text(result.rss_mib, digits=0)]
    empty = result.info.get('empty_answers')
    samples = f'{wall.n} ({empty} empty)' if empty else str(wall.n)
    return [name, median_text(wall, scale, 1), f'{wall.p95 * scale:.1f}', f'{wall.min * scale:.1f}-{wall.max * scale:.1f}',
            samples, median_text(result.cpu, scale, 1), median_text(result.rss_mib, digits=0)]
