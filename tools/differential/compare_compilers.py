#!/usr/bin/env python3
"""Compare two compiler binaries, A and B, on many small programs and report where they disagree.

Usage: tools/differential/compare_compilers.py --a BIN --b BIN [--mode check|interp|run|run-opt] [--mode-a M] [--mode-b M]
                                     [--args-a ARGS] [--args-b ARGS] [--sandbox] [--jobs N] [--timeout S]
                                     [--out DIR] [--reduce [N]] SET...

Sets: `grid` (or `grid:context,poly,using`), `rewrite:PATH[,PATH...]` (text edits of existing programs),
`gen:SEED:COUNT` (tools/differential/jaigen.py programs), or .jai files and folders of them.

A and B are opaque binaries (any two builds, or one binary in two modes: `--a X --mode-a interp --b X --mode-b run`).
Each program is compiled (and run) by both; the outcome of each side is its status, exit code, stdout and the first
error line without paths, line numbers or colour. Pairs are classified agree, both_reject, a_only, b_only,
output_differs or failure (timeout or crash), clustered by error text and by the generators' feature tags, and
written to OUT/report.md and OUT/results.json (default target/compare/). Exit 1 when any pair disagrees, 2 when a
generated program does not even parse. See docs/tools/testing.md.
"""
from __future__ import annotations

import argparse
import json
import random
import re
import shlex
import shutil
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path

from repo_output import output_dir
import compare_grid
import compare_rewrite
import jaigen
from bench.sandbox import Sandbox, environment
from bench.cmd_compile import install_reads
from compare_core import (CLASSES, CRASH, OK,
                          REJECTED, RUNTIME, TIMEOUT, Outcome, Program, Record, cluster_by_error, cluster_by_tag, cluster_rejections, first_error, has_diagnostic, looks_crashed,
                          reduce_lines, summarise)
from bench.dialects import Compiler, Mode, Request, detect_dialect
from bench.measure import run_measured
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
MODES = {'check': Mode.CHECK, 'interp': Mode.INTERP, 'run': Mode.BUILD, 'run-opt': Mode.OPTIMIZED}
MAX_STDOUT = 20000
DEFAULT_REWRITE_PATHS = ['tests/compiler/semantics', 'tests/compiler/language']
SYNTAX_ERROR = re.compile(r'^error:? (expected|unexpected|unbalanced|unterminated|unclosed)', re.IGNORECASE)


@dataclass
class Side:
    label: str
    compiler: Compiler
    mode: Mode
    extra: list[str] = field(default_factory=list)

    def describe(self) -> dict:
        return {**self.compiler.describe(), 'mode': self.mode.value, 'extra_args': self.extra}


@dataclass
class Settings:
    out: Path
    timeout: float = 60
    sandbox: bool = False
    sandbox_reads: list[Path] = field(default_factory=list)
    keep_work: bool = False


def make_side(label: str, binary: str, mode: str, extra: str) -> Side:
    path = Path(binary).expanduser().resolve()
    if not path.exists():
        sys.exit(f'compare: --{label.lower()} {binary}: no such file')
    return Side(label, Compiler(label, path, detect_dialect(path)), MODES[mode], shlex.split(extra))


def prepare(program: Program, workdir: Path) -> Path:
    """Write the program into `workdir` and link the entries of its `root` beside it."""
    shutil.rmtree(workdir, ignore_errors=True)
    workdir.mkdir(parents=True)
    entry = workdir / program.filename
    entry.write_text(program.source)
    if program.root:
        for item in program.root.iterdir():
            link = workdir / item.name
            if not link.exists():
                link.symlink_to(item.resolve())
    return entry


def run_command(command: list[str], cwd: Path, scratch: Path, reads: list[Path], writes: list[Path],
                settings: Settings):
    env = None
    if settings.sandbox:
        command = Sandbox(reads=reads, writes=writes).wrap(command)
        env = environment(scratch)
    return run_measured(command, cwd=cwd, env=env, timeout=settings.timeout)


def run_side(side: Side, entry: Path, scratch: Path, settings: Settings, reads: list[Path]) -> Outcome:
    """Compile (and for `run`, execute) the program at `entry` with one compiler."""
    scratch.mkdir(parents=True, exist_ok=True)
    workdir = entry.parent
    try:
        request = Request(entry, workdir, side.mode, (), scratch)
        invocation = side.compiler.dialect.invocation(side.compiler.path, request)
        reads = [*reads, *install_reads(side.compiler), workdir, scratch, *settings.sandbox_reads]
        writes = [workdir, scratch]
        build = run_command([*invocation.command, *side.extra], invocation.cwd, scratch, reads, writes, settings)
        return judge(side, build, invocation.executable, workdir, scratch, settings, reads, writes)
    except OSError as error:
        return Outcome(CRASH, error=f'cannot run: {error}')


def judge(side: Side, build, executable: Path | None, workdir: Path, scratch: Path, settings: Settings,
          reads: list[Path], writes: list[Path]) -> Outcome:
    error = first_error(build.stderr, build.stdout)
    if build.timed_out:
        return Outcome(TIMEOUT, error='timeout while compiling')
    if settings.sandbox and re.match(r'(sandbox-exec|bwrap):', build.stderr):
        sys.exit(f'compare: the sandbox cannot start: {build.stderr.strip().splitlines()[0]}')
    if build.returncode < 0 or looks_crashed(build.stderr):
        return Outcome(CRASH, build.returncode, build.stdout[:MAX_STDOUT], error or f'signal {-build.returncode}')
    if build.returncode != 0:
        if side.mode is Mode.INTERP and not has_diagnostic(build.stderr):
            return Outcome(RUNTIME, build.returncode, build.stdout[:MAX_STDOUT], error)
        return Outcome(REJECTED, build.returncode, '', error)
    if executable is None:
        return Outcome(OK, 0, build.stdout[:MAX_STDOUT])
    if not executable.exists():
        return Outcome(CRASH, 0, '', 'the build wrote no executable')
    ran = run_command([str(executable)], workdir, scratch, reads, writes, settings)
    if ran.timed_out:
        return Outcome(TIMEOUT, error='timeout while running')
    status = OK if ran.returncode == 0 else RUNTIME
    return Outcome(status, ran.returncode, ran.stdout[:MAX_STDOUT], first_error(ran.stderr) if status != OK else '')


def run_pair(program: Program, a: Side, b: Side, settings: Settings) -> Record:
    safe = re.sub(r'[^\w.-]+', '_', program.name)
    workdir = settings.out / 'work' / safe
    scratch = settings.out / 'scratch' / safe
    entry = prepare(program, workdir)
    reads = [program.root] if program.root else []
    try:
        outcome_a = run_side(a, entry, scratch / 'a', settings, reads)
        outcome_b = run_side(b, entry, scratch / 'b', settings, reads)
    finally:
        if not settings.keep_work:
            shutil.rmtree(workdir, ignore_errors=True)
            shutil.rmtree(scratch, ignore_errors=True)
    return Record(program, outcome_a, outcome_b)


def self_check(programs: list[Program], checker: Side, settings: Settings, jobs: int) -> list[tuple[str, str]]:
    """Generator bugs: generated programs the checker's front end cannot even parse (a syntax-class error).
    A type error is not one: the near misses are meant to produce those."""
    scratch_root = settings.out / 'selfcheck'

    def check(program: Program):
        safe = re.sub(r'[^\w.-]+', '_', program.name)
        entry = prepare(program, scratch_root / safe)
        outcome = run_side(checker, entry, scratch_root / f'{safe}-s', settings, [program.root] if program.root else [])
        shutil.rmtree(scratch_root / safe, ignore_errors=True)
        shutil.rmtree(scratch_root / f'{safe}-s', ignore_errors=True)
        return program.name, outcome

    with ThreadPoolExecutor(jobs) as pool:
        results = list(pool.map(check, programs))
    shutil.rmtree(scratch_root, ignore_errors=True)
    return [(name, o.error) for name, o in results if o.status == REJECTED and SYNTAX_ERROR.match(o.error)]


# ---- program sets ----

def find_sources(paths: list[Path]) -> list[Path]:
    found: list[Path] = []
    for path in paths:
        found += sorted(path.rglob('*.jai')) if path.is_dir() else [path]
    return found


def build_programs(specs: list[str], args: argparse.Namespace) -> list[Program]:
    programs: list[Program] = []
    for spec in specs:
        kind, _, rest = spec.partition(':')
        if spec == 'grid' or kind == 'grid':
            programs += compare_grid.grid([a for a in rest.split(',') if a] or None)
        elif kind == 'rewrite':
            paths = [ROOT / p for p in (rest.split(',') if rest else DEFAULT_REWRITE_PATHS)]
            names = args.rewrites.split(',') if args.rewrites else None
            rewritten = compare_rewrite.rewrite_files(find_sources(paths), names, args.rewrite_sites, ROOT)
            programs += rewritten
        elif kind == 'gen':
            seed, _, count = rest.partition(':')
            if not seed.isdigit() or not count.isdigit():
                sys.exit(f'compare: {spec!r}: expected gen:SEED:COUNT')
            for n in range(int(count)):
                programs.append(Program(f'gen_{int(seed) + n}', jaigen.generate(int(seed) + n, args.gen_size),
                                        ('axis:gen', f'seed:{int(seed) + n}')))
        else:
            path = Path(spec)
            if not path.exists():
                sys.exit(f'compare: {spec}: not a program set, file or folder')
            for source in find_sources([path]):
                label = re.sub(r'[^\w.-]+', '_', source.with_suffix('').as_posix().strip('/'))
                programs.append(Program(f'file_{label}', source.read_text(errors='replace'),
                                        ('axis:file', f'source:{source.name}'), source.parent.resolve(), source.name))
    seen: set[str] = set()
    unique = []
    for program in programs:
        if program.name not in seen:
            seen.add(program.name)
            unique.append(program)
    if args.only:
        unique = [p for p in unique if args.only in p.name or any(args.only in t for t in p.tags)]
    if args.limit and len(unique) > args.limit:
        keep = set(random.Random(args.seed).sample(range(len(unique)), args.limit))
        unique = [p for i, p in enumerate(unique) if i in keep]
    return unique


# ---- reduction ----

def reduce_record(record: Record, a: Side, b: Side, settings: Settings, budget: int) -> str | None:
    """Delete lines while the pair is still classified the same with the same characteristic message."""
    want = (record.verdict, record.key_text())
    program = record.program

    def keeps(lines: list[str]) -> bool:
        candidate = Program('reduce_candidate', '\n'.join(lines) + '\n', (), program.root, program.filename)
        again = run_pair(candidate, a, b, settings)
        return (again.verdict, again.key_text()) == want

    lines = program.source.rstrip('\n').split('\n')
    reduced = reduce_lines(lines, keeps, budget)
    return '\n'.join(reduced) + '\n' if len(reduced) < len(lines) else None


# ---- report ----

def cell(text: str, width: int = 90) -> str:
    text = text.replace('|', '\\|').replace('\n', ' ')
    return text if len(text) <= width else text[:width - 1] + '…'


def describe_outcome(o: Outcome) -> str:
    if o.status == OK:
        return f'ok, stdout {o.stdout.strip()[:60]!r}'
    if o.status == RUNTIME:
        return f'runtime failure (exit {o.exit_code}), stdout {o.stdout.strip()[:40]!r}'
    return f'{o.status}: {o.error}'


def write_report(records: list[Record], a: Side, b: Side, settings: Settings, problems: list[tuple[str, str]],
                 elapsed: float) -> dict:
    out = settings.out
    summary = summarise(records)
    errors = cluster_by_error(records)
    tags = cluster_by_tag(records)
    rejections = cluster_rejections(records)
    disagreements = [r for r in records if r.disagrees]
    (out / 'programs').mkdir(parents=True, exist_ok=True)
    for record in disagreements:
        safe = re.sub(r'[^\w.-]+', '_', record.program.name)
        (out / 'programs' / f'{safe}.jai').write_text(record.program.source)
        if record.reduced:
            (out / 'programs' / f'{safe}.reduced.jai').write_text(record.reduced)

    data = {
        'a': a.describe(), 'b': b.describe(), 'seconds': round(elapsed, 1), 'summary': summary,
        'generator_problems': [{'program': n, 'error': e} for n, e in problems],
        'error_clusters': [{'verdict': c.verdict, 'message': c.key, 'count': len(c.records),
                            'smallest': c.smallest.program.name, 'programs': [r.program.name for r in c.records]}
                           for c in errors],
        'rejection_clusters': [{'message': c.key, 'count': len(c.records), 'smallest': c.smallest.program.name,
                                'programs': [r.program.name for r in c.records]} for c in rejections],
        'tag_clusters': [{'tag': t, 'disagreements': len(found), 'programs': total} for t, found, total in tags],
        'records': [r.to_json() for r in records if r.disagrees or r.diagnostics_differ],
    }
    (out / 'results.json').write_text(json.dumps(data, indent=1) + '\n')

    md = ['# Compiler comparison', '',
          f'- A: `{a.compiler.path}` ({a.compiler.dialect.name} dialect, mode `{a.mode.value}`{extra_text(a)})',
          f'- B: `{b.compiler.path}` ({b.compiler.dialect.name} dialect, mode `{b.mode.value}`{extra_text(b)})',
          f'- {summary["total"]} programs in {elapsed:.0f} s', '',
          '| class | programs |', '| --- | ---: |']
    md += [f'| {name} | {summary[name]} |' for name in CLASSES]
    md += [f'| (both reject, different message) | {summary["diagnostics_differ"]} |', '']
    if problems:
        md += ['## Generator problems', '', 'Programs the checker could not parse; fix the template, not the compiler.', '']
        md += [f'- `{n}`: {e}' for n, e in problems[:30]] + ['']
    if not disagreements:
        md += ['No disagreements.', '']
    else:
        md += ['## Disagreements by error text', '',
               '| count | class | message | smallest example |', '| ---: | --- | --- | --- |']
        md += [f'| {len(c.records)} | {c.verdict} | {cell(c.key)} | `{c.smallest.program.name}` |' for c in errors]
        md += ['', '## Disagreements by feature tag', '',
               'Highest share first; a tag every program carries (`axis:`) only totals.', '',
               '| tag | disagree / programs | classes |', '| --- | ---: | --- |']
        for tag, found, total in tags[:60]:
            mix = ', '.join(f'{v} {k}' for k, v in count_by(found).items())
            md.append(f'| `{tag}` | {len(found)} / {total} | {mix} |')
        md += ['', '## Smallest example per cluster', '']
        for cluster in errors[:15]:
            record = cluster.smallest
            source = record.reduced or record.program.source
            md += [f'### {cluster.verdict}: {cell(cluster.key, 120)} ({len(cluster.records)})', '',
                   f'`{record.program.name}`' + (' (reduced)' if record.reduced else '')
                   + f' tags: {", ".join(record.program.tags)}', '',
                   f'- A: {describe_outcome(record.a)}', f'- B: {describe_outcome(record.b)}', '',
                   '```jai', source.rstrip('\n'), '```', '']
        md += ['## All disagreements', '', '| program | class | A | B |', '| --- | --- | --- | --- |']
        md += [f'| `{r.program.name}` | {r.verdict} | {cell(describe_outcome(r.a), 60)} | '
               f'{cell(describe_outcome(r.b), 60)} |' for r in disagreements[:400]]
        if len(disagreements) > 400:
            md.append(f'| … {len(disagreements) - 400} more in results.json | | | |')
        md.append('')
    if rejections:
        md += ['## Both reject, not a near miss', '',
               'Not disagreements. A generator template may be wrong, or both compilers share a gap.', '',
               '| count | message | smallest example |', '| ---: | --- | --- |']
        md += [f'| {len(c.records)} | {cell(c.key)} | `{c.smallest.program.name}` |' for c in rejections[:40]] + ['']
    (out / 'report.md').write_text('\n'.join(md))
    return data


def extra_text(side: Side) -> str:
    return f', extra `{" ".join(side.extra)}`' if side.extra else ''


def count_by(records: list[Record]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for record in records:
        counts[record.verdict] = counts.get(record.verdict, 0) + 1
    return counts


# ---- command line ----

def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__.split('\n')[0], formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('sets', nargs='+', metavar='SET', help='grid[:AXES], rewrite:PATHS, gen:SEED:COUNT, a .jai file or folder')
    p.add_argument('--a', required=True, metavar='BIN', help='compiler A')
    p.add_argument('--b', required=True, metavar='BIN', help='compiler B')
    p.add_argument('--mode', choices=MODES, default='check', help='check (default), interp, or run (native build, then execute)')
    p.add_argument('--mode-a', choices=MODES, help='override --mode for A')
    p.add_argument('--mode-b', choices=MODES, help='override --mode for B')
    p.add_argument('--args-a', default='', help='extra command-line arguments for A (one shell-quoted string)')
    p.add_argument('--args-b', default='', help='extra command-line arguments for B')
    p.add_argument('--jobs', type=int, default=3, help='programs compiled at once (default 3)')
    p.add_argument('--timeout', type=float, default=60, help='seconds per compile or run (default 60)')
    p.add_argument('--out', type=Path, default=output_dir('compare'), help='output folder (default target/compare)')
    p.add_argument('--sandbox', action='store_true', help='run every compile and program without network and with minimal file access')
    p.add_argument('--sandbox-read', action='append', type=Path, default=[], metavar='PATH', help='another path the sandbox may read')
    p.add_argument('--reduce', nargs='?', const=5, type=int, default=0, metavar='N',
                   help='shrink the smallest example of each of the first N error clusters (default 5 when given)')
    p.add_argument('--reduce-budget', type=int, default=150, help='compiles per reduction (default 150)')
    p.add_argument('--no-self-check', action='store_true', help='skip checking that generated programs parse')
    p.add_argument('--check-with', metavar='BIN', help='compiler for the generation-time parse check (default: B)')
    p.add_argument('--only', metavar='TEXT', help='keep programs whose name or a tag contains TEXT')
    p.add_argument('--limit', type=int, default=0, help='run a deterministic random sample of this many programs')
    p.add_argument('--seed', type=int, default=0, help='seed of the --limit sample')
    p.add_argument('--rewrites', help='comma-separated rewrite names (default all: ' + ', '.join(compare_rewrite.REWRITES) + ')')
    p.add_argument('--rewrite-sites', type=int, default=3, help='sites rewritten per file and rewrite (default 3)')
    p.add_argument('--gen-size', type=float, default=1.0, help='jaigen --size for gen: sets')
    p.add_argument('--keep-work', action='store_true', help='keep each program\'s working folder under OUT/work')
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    a = make_side('A', args.a, args.mode_a or args.mode, args.args_a)
    b = make_side('B', args.b, args.mode_b or args.mode, args.args_b)
    if args.sandbox and (why := Sandbox.available()):
        sys.exit(f'compare: cannot sandbox: {why}')
    settings = Settings(args.out.resolve(), args.timeout, args.sandbox, [p.resolve() for p in args.sandbox_read],
                        args.keep_work)
    shutil.rmtree(settings.out, ignore_errors=True)
    settings.out.mkdir(parents=True)
    programs = build_programs(args.sets, args)
    if not programs:
        sys.exit('compare: no programs')

    problems: list[tuple[str, str]] = []
    generated = [p for p in programs if p.root is None or 'axis:rewrite' in p.tags]
    if not args.no_self_check and generated:
        checker = b if not args.check_with else make_side('check', args.check_with, 'check', '')
        checker = Side(checker.label, checker.compiler, Mode.CHECK, [])
        problems = self_check(generated, checker, settings, args.jobs)
        if problems:
            data = write_report([], a, b, settings, problems, 0)
            for name, error in problems[:20]:
                print(f'generator problem: {name}: {error}', file=sys.stderr)
            print(f'compare: {len(problems)} generated programs do not parse; see {settings.out / "report.md"}', file=sys.stderr)
            return 2

    print(f'compare: {len(programs)} programs, A={a.compiler.path.name} ({a.mode.value}), '
          f'B={b.compiler.path.name} ({b.mode.value}), {args.jobs} jobs', file=sys.stderr)
    started = time.time()
    done = 0

    def work(program: Program) -> Record:
        nonlocal done
        record = run_pair(program, a, b, settings)
        done += 1
        if done % 50 == 0:
            print(f'  {done}/{len(programs)}', file=sys.stderr, flush=True)
        return record

    with ThreadPoolExecutor(args.jobs) as pool:
        records = list(pool.map(work, programs))

    if args.reduce:
        for cluster in cluster_by_error(records)[:args.reduce]:
            target = cluster.smallest
            target.reduced = reduce_record(target, a, b, settings, args.reduce_budget)
    data = write_report(records, a, b, settings, problems, time.time() - started)
    summary = data['summary']
    print(' '.join(f'{name}={summary[name]}' for name in CLASSES), file=sys.stderr)
    print(f'report: {settings.out / "report.md"}', file=sys.stderr)
    return 1 if any(r.disagrees for r in records) else 0


if __name__ == '__main__':
    sys.exit(main())
