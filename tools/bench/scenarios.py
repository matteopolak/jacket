"""Command-line scenarios: a command, run repeatedly and measured, for the lint and fmt subcommands."""
from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

from bench import common
from bench.measure import Run, run_measured, warm_file_cache
from bench.results import Report, Result
from bench.stats import Stat


@dataclass(frozen=True)
class Scenario:
    key: str
    command: list[str]
    cwd: Path | None = None
    stdin_bytes: bytes | None = None
    # Exit codes that count as a successful run (a linter or `--check` exits 1 when it finds something).
    ok_codes: tuple[int, ...] = (0,)
    # Runs before every execution, outside the timed window (a fresh copy for a tool that edits in place).
    prepare: Callable[[], None] | None = None
    # Files the command reads, read once before timing to fill the OS cache.
    inputs: tuple[Path, ...] = ()
    info: dict = field(default_factory=dict)
    # Takes minutes: runs only when `--only` names it.
    heavy: bool = False


def wanted(scenario: Scenario, only: str) -> bool:
    return common.selected(only, scenario.key) and (not scenario.heavy or bool(only))


def execute(scenario: Scenario, timeout: float) -> Run:
    if scenario.prepare:
        scenario.prepare()
    return run_measured(scenario.command, cwd=scenario.cwd, timeout=timeout, stdin_bytes=scenario.stdin_bytes)


def measure(scenario: Scenario, args: argparse.Namespace) -> tuple[Result, Run | None]:
    """The scenario's result and its last run (for output checks). A run with an unexpected exit status fails it."""
    warm_file_cache(Path(scenario.command[0]), *scenario.inputs)
    runs: list[Run] = []
    for index in range(args.warmup + args.repeat):
        run = execute(scenario, args.timeout)
        if run.timed_out or run.returncode not in scenario.ok_codes:
            return Result.failed(run.failure(), **scenario.info), run
        if index >= args.warmup:
            runs.append(run)
    result = Result(wall=Stat.of([r.wall for r in runs]), cpu=Stat.of([r.cpu for r in runs]),
                    rss_mib=Stat.of([r.rss_mib for r in runs]), info=dict(scenario.info))
    return result, runs[-1]


def run_scenarios(scenarios: list[Scenario], args: argparse.Namespace) -> dict[str, Result]:
    results = {}
    for scenario in scenarios:
        if not wanted(scenario, args.only):
            continue
        result, _ = measure(scenario, args)
        results[scenario.key] = result
        text = f'{result.wall.median:.3f} s' if result.wall else f'{result.status.value}: {result.reason}'
        common.print_progress(scenario.key, text)
    return results


def table_rows(results: dict[str, Result]) -> list[list[str]]:
    rows = []
    for key, result in results.items():
        if result.wall:
            rows.append([key, common.spread(result.wall, digits=3), common.median_text(result.cpu, digits=3),
                         common.median_text(result.rss_mib, digits=0), str(result.wall.n)])
    return rows


def render_table(results: dict[str, Result]) -> str:
    return common.table(['scenario', 'wall s (min-max)', 'CPU s', 'RSS MiB', 'runs'], table_rows(results))


def render_report(title: str, report: Report, subject_lines: list[str]) -> str:
    out = [f'# {title} ({report.date[:10]})', '', common.machine_line(report), '',
           f"{report.settings['repeat']} timed runs per scenario after {report.settings['warmup']} untimed "
           '(wall time is the median, the range is min-max; CPU is user + system).', '', *subject_lines, '',
           render_table(report.results), common.skipped_section(report)]
    return '\n'.join(out).rstrip() + '\n'
