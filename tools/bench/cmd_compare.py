"""`compare`: a fair head-to-head of two compiler binaries (baseline vs candidate) that take the same command line."""
from __future__ import annotations

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import dataclasses
import json
import os
import shlex
import shutil
import statistics
import sys
import tempfile
import zlib
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

import common
import pairing
import module_swap
import program_facts
import provenance
import quiet
from cmd_compile import JobRunner, first_difference, normalized
from dialects import Compiler, Mode, PlainDialect, Request, parse_compiler
from measure import Run, run_measured, warm_file_cache
from bench.sandbox import Sandbox
from results import ROOT
from stats import Stat
from workloads import ALL_BUILD_MODES, Category, Generated, Workload, all_workloads, single

NAMES = ('baseline', 'candidate')
DEFAULT_SIZES = '10000,100000,250000,500000'
FORMAT = 'compare-1'


def split_env(pairs: list[str]) -> dict[str, str]:
    """`KEY=VALUE` options as a dictionary."""
    found = {}
    for pair in pairs:
        key, separator, value = pair.partition('=')
        if not separator or not key:
            sys.exit(f'expected KEY=VALUE, got {pair!r}')
        found[key] = value
    return found


def default_out_dir() -> Path:
    return ROOT / 'playground' / 'bench' / datetime.now().strftime('%Y%m%d-%H%M%S')


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument('--baseline', required=True, metavar='PATH', help='the first compiler binary (A)')
    parser.add_argument('--candidate', required=True, metavar='PATH', help='the second compiler binary (B)')
    parser.add_argument('--baseline-env', action='append', default=[], metavar='KEY=VALUE',
                        help='an environment variable for the baseline only (for example JAIC_STDLIB)')
    parser.add_argument('--candidate-env', action='append', default=[], metavar='KEY=VALUE',
                        help='an environment variable for the candidate only')
    parser.add_argument('--sizes', default=DEFAULT_SIZES, help='generated-program line counts (default %(default)s)')
    parser.add_argument('--seed', type=int, default=1, help='generator seed, and base of the run-order seeds')
    parser.add_argument('--no-corpus', action='store_true', help='skip the real-project workloads')
    parser.add_argument('--modes', default='check,build,optimized', help='subset of check, build, optimized')
    parser.add_argument('--rounds', type=int, default=15, help='warm rounds, each timing both binaries once')
    parser.add_argument('--cold-rounds', type=int, default=5, help='cold rounds (0 to skip the cold phase)')
    parser.add_argument('--warmup', type=int, default=1, help='untimed warm runs per binary before the warm rounds')
    parser.add_argument('--order', choices=('shuffle', 'alternate'), default='shuffle',
                        help='shuffle: random order per round, reversed the next round; alternate: A B every round')
    parser.add_argument('--only', default='', help='comma-separated substrings; run only workload/mode keys matching one')
    parser.add_argument('--timeout', type=float, default=900, help='seconds before a single run is killed')
    parser.add_argument('--max-load', type=float, default=None,
                        help='highest 1-minute load average accepted (default: a quarter of the cores, at least 1)')
    parser.add_argument('--force', action='store_true',
                        help='start even though the machine is loaded or thermally limited (blocks stay flagged)')
    parser.add_argument('--no-purge', action='store_true', help='do not drop the OS file cache before cold runs')
    parser.add_argument('--sandbox-read', action='append', metavar='PATH', default=[],
                        help='another path the sandboxed compilers may read (repeatable)')
    parser.add_argument('--no-sandbox', action='store_true', help='run without the sandbox (see sandbox.py)')
    parser.add_argument('--program-runs', type=int, default=5,
                        help='counted runs of each built program that is safe to run (0 to skip counting)')
    parser.add_argument('--programs-only', action='store_true',
                        help='only measure the built programs (size, instructions, retired instructions); time nothing')
    parser.add_argument('--swap-modules', default='', metavar='NAMES',
                        help='comma-separated modules to take from --baseline-modules / --candidate-modules')
    parser.add_argument('--baseline-modules', type=Path, default=None, metavar='DIR',
                        help='build the baseline against the --swap-modules found in DIR (see module_swap.py); '
                             "its diagnostics are left out of the results")
    parser.add_argument('--candidate-modules', type=Path, default=None, metavar='DIR',
                        help='the same for the candidate')
    parser.add_argument('--list', action='store_true', help='print what would run and exit')
    parser.add_argument('--out-dir', type=Path, default=None,
                        help='results folder (default: playground/bench/<timestamp>/, gitignored)')


# ------------------------------------------------------------------------------------------- plan

@dataclass(frozen=True)
class Job:
    workload: Workload
    mode: Mode

    @property
    def key(self) -> str:
        return f'{self.workload.name}/{self.mode.value}'


def workloads_for(args: argparse.Namespace) -> list[Workload]:
    sizes = [int(size) for size in args.sizes.split(',') if size.strip()]
    found = [Workload(f'gen-{n // 1000}k', Category.LOTS_OF_CODE, Generated(n, args.seed, max(1, n // 5000)),
                      single('main.jai', *ALL_BUILD_MODES), compare_output=True) for n in sizes]
    found += [w for w in all_workloads() if w.category is Category.GENERATED_CODE]
    if not args.no_corpus:
        found += [w for w in all_workloads() if w.category is Category.REAL_PROJECT]
    return found


def plan_jobs(args: argparse.Namespace) -> list[Job]:
    modes = {Mode(name) for name in args.modes.split(',') if name}
    return [Job(w, mode) for w in workloads_for(args) for mode in w.modes
            if mode in modes and mode is not Mode.INTERP and common.selected(args.only, f'{w.name}/{mode.value}')]


def block_seed(base: int, key: str) -> int:
    """A seed for one block's run order that depends on the base seed and the block, nothing else."""
    return base * 1_000_003 + zlib.crc32(key.encode())


# -------------------------------------------------------------------------------- output checking

@dataclass
class Observed:
    """What one binary did with a job: the compile, and for a built program that is safe to run, the run."""
    compile_exit: int
    compile_stdout: list[str]
    program_exit: int | None = None
    program_stdout: list[str] | None = None
    error: str = ''
    # What the build produced (`program_facts`): sizes, instructions, and counters from running it.
    program: dict | None = None


def compare_outputs(a: Observed, b: Observed, compare_compile_output: bool, quote: bool = True) -> str:
    """'' when the two binaries built the job and agree, otherwise why the job is not timed. Without
    `quote` the reason says what differs but quotes no output."""
    def difference(x: list[str], y: list[str]) -> str:
        return first_difference(x, y) if quote else f'{len(x)} lines vs {len(y)} lines'

    for name, seen in zip(NAMES, (a, b)):
        if seen.error:
            return f'{name}: {seen.error}'
    if a.compile_exit != b.compile_exit:
        return f'compile exit codes differ ({a.compile_exit} vs {b.compile_exit})'
    if compare_compile_output and a.compile_stdout != b.compile_stdout:
        return f'compiler output differs: {difference(a.compile_stdout, b.compile_stdout)}'
    if a.program_exit != b.program_exit:
        return f'program exit codes differ ({a.program_exit} vs {b.program_exit})'
    if a.program_stdout != b.program_stdout:
        return f'program output differs: {difference(a.program_stdout or [], b.program_stdout or [])}'
    return ''


# ----------------------------------------------------------------------------------------- summary

def summarize_samples(samples: list[dict], seed: int) -> dict:
    """Per-binary statistics and the paired ratio from the samples of one block (rounds line up)."""
    walls = {n: [s['wall_s'] for s in sorted(samples, key=lambda s: s['round']) if s['compiler'] == n] for n in NAMES}
    summary: dict = {n: {'wall': Stat.of(walls[n]).to_json(),
                         'cpu': Stat.of([s['cpu_s'] for s in samples if s['compiler'] == n]).to_json(),
                         'rss_mib': Stat.of([s['rss_mib'] for s in samples if s['compiler'] == n]).to_json()}
                     for n in NAMES}
    summary['wall_ratio'] = pairing.ratio_summary(walls[NAMES[0]], walls[NAMES[1]], seed)
    for metric in ('cpu_s', 'rss_mib'):
        series = {n: [s[metric] for s in sorted(samples, key=lambda s: s['round']) if s['compiler'] == n] for n in NAMES}
        pairs = [c / b for b, c in zip(*series.values()) if b > 0]
        summary[metric.split('_')[0] + '_ratio'] = statistics.median(pairs) if pairs else None
    stages = {}
    names = {key for s in samples for key in s.get('stages', {})}
    for key in sorted(names):
        medians = {n: statistics.median(s['stages'].get(key, 0.0) for s in samples if s['compiler'] == n)
                   for n in NAMES}
        stages[key] = {**medians, 'ratio': medians[NAMES[1]] / medians[NAMES[0]] if medians[NAMES[0]] else None}
    if stages:
        summary['stages'] = stages
    return summary


def fmt_stat(stat: dict) -> str:
    return f"{stat['median']:.3f} (±{stat['mad']:.3f}, p95 {stat['p95']:.3f})"


def fmt_ratio(ratio: dict) -> str:
    if ratio['ratio'] is None:
        return '-'
    if ratio['low'] is None:
        return f"{ratio['ratio']:.3f}"
    return f"{ratio['ratio']:.3f} [{ratio['low']:.3f}, {ratio['high']:.3f}]"


def render(results: dict) -> str:
    """The Markdown summary of a results dictionary."""
    m = results['machine']
    out = [f"# Compiler comparison ({results['date'][:19]})", '',
           f"Machine: {m.get('cpu') or m.get('machine')}, {m.get('cpus')} cores, {m.get('memory_gib')} GiB, "
           f"{m.get('system')} {m.get('os_version') or m.get('release')}.", '']
    for name in NAMES:
        b = results['binaries'][name]
        out.append(f"- {name}: `{b['path']}`, version `{b.get('version', '')}`, sha256 `{b.get('sha256', '')[:16]}`, "
                   f"{b['dialect']} dialect")
    s = results['settings']
    out += ['', f"{s['rounds']} warm rounds and {s['cold_rounds']} cold rounds per block, order `{s['order']}`, seed "
            f"{s['seed']}. Seconds are medians over the rounds (±MAD, p95). The ratio is candidate / baseline: the "
            'median of the per-round ratios with a bootstrap 95% interval; below 1 the candidate is faster. '
            f"Cold: fresh output and cache folders, {results['purge']}.", '']
    done = [b for b in results['blocks'] if b['status'] == 'ok']
    flagged = [b for b in done if b['flags']]
    out += [f"Blocks timed: {len(done)}, flagged as noisy: {len(flagged)}, not timed: {len(results['blocks']) - len(done)}.", '']
    if results['start_warnings']:
        out += ['Start-up warnings:', *(f'- {w}' for w in results['start_warnings']), '']
    for phase in ('warm', 'cold'):
        rows = [b for b in done if b['phase'] == phase]
        if not rows:
            continue
        out += [f'## {phase.capitalize()} runs', '',
                '| workload | mode | baseline wall s | candidate wall s | wall ratio [95% CI] | verdict | cpu ratio | rss ratio | quiet |',
                '|---|---|---:|---:|---:|---|---:|---:|---|']
        for b in rows:
            r = b['summary']
            cpu, rss = (f'{r[k]:.3f}' if r.get(k) is not None else '-' for k in ('cpu_ratio', 'rss_ratio'))
            out.append(f"| {b['workload']} | {b['mode']} | {fmt_stat(r['baseline']['wall'])} | "
                       f"{fmt_stat(r['candidate']['wall'])} | {fmt_ratio(r['wall_ratio'])} | "
                       f"{r['wall_ratio']['verdict']} | {cpu} | {rss} | {'flagged' if b['flags'] else 'ok'} |")
        out.append('')
    staged = [b for b in done if b['phase'] == 'warm' and b['summary'].get('stages')]
    if staged:
        out += ['## Steps (warm, from `-timings`, both binaries)', '', '| workload | mode | step | baseline s | candidate s | ratio |',
                '|---|---|---|---:|---:|---:|']
        for b in staged:
            for step, v in b['summary']['stages'].items():
                ratio = f"{v['ratio']:.3f}" if v['ratio'] else '-'
                out.append(f"| {b['workload']} | {b['mode']} | {step} | {v['baseline']:.3f} | {v['candidate']:.3f} | {ratio} |")
        out.append('')
    out += render_programs(results.get('programs', []))
    if flagged:
        out += ['## Flagged blocks', '', *(f"- {b['workload']}/{b['mode']} ({b['phase']}): {'; '.join(b['flags'])}" for b in flagged), '']
    skipped = [b for b in results['blocks'] if b['status'] != 'ok']
    if skipped:
        out += ['## Not timed', '', *(f"- {b['workload']}/{b['mode']} ({b['phase']}): {b['status']}: {b['reason']}" for b in skipped), '']
    out += ['## Provenance', '', 'See `results.json` (inputs, hashes, machine, quiet-machine snapshots) and `raw.csv` '
            '(every timed run) in this folder.', '']
    return '\n'.join(out)


PROGRAM_COLUMNS = (('file_bytes', 'file bytes'), ('instructions', 'instructions in code'),
                   ('retired_instructions', 'retired instructions'), ('cycles', 'cycles'))


def render_programs(programs: list[dict]) -> list[str]:
    """The table of what each binary built: per measure, baseline, candidate and candidate / baseline."""
    if not programs:
        return []
    out = ['## Built programs', '',
           'What each binary produced for the same program. Instructions in code are counted by the '
           'disassembler over the whole executable (runtime and libraries included); retired instructions '
           'and cycles are hardware counters of running it, less what starting a process that does nothing costs '
           '(median of the counted runs; programs that are '
           'not safe to run have none). Below 1 the candidate produced less.', '',
           '| workload | mode | measure | baseline | candidate | ratio |', '|---|---|---|---:|---:|---:|']
    for p in programs:
        for key, label in PROGRAM_COLUMNS:
            a, b = p[NAMES[0]].get(key), p[NAMES[1]].get(key)
            if a is None or b is None:
                continue
            ratio = f'{b / a:.3f}' if a else '-'
            out.append(f"| {p['workload']} | {p['mode']} | {label} | {a:,} | {b:,} | {ratio} |")
    return [*out, '']


# ------------------------------------------------------------------------------------------ runner

def same_command_line(compilers: dict[str, Compiler]) -> dict[str, Compiler]:
    """Both binaries must be run the same way: when only one has jaic's own flags, both get the
    portable command line."""
    if len({c.dialect.name for c in compilers.values()}) == 1:
        return compilers
    return {name: dataclasses.replace(c, dialect=PlainDialect()) for name, c in compilers.items()}


class Comparison:
    def __init__(self, compilers: dict[str, Compiler], envs: dict[str, dict[str, str]], scratch: Path,
                 args: argparse.Namespace):
        self.compilers = compilers
        self.envs = envs
        self.scratch = scratch
        self.args = args
        self.runner = JobRunner(list(compilers.values()), scratch, args)
        self.max_load = args.max_load or quiet.default_max_load(os.cpu_count())
        self.purge_ok, self.purge_note = (False, 'skipped (--no-purge)') if args.no_purge else quiet.purge_caches()
        # Per-step seconds come from `-timings`, which only a jaic-dialect binary prints: both or neither.
        self.report_steps = all(c.dialect.name == 'jaic' for c in compilers.values())
        self.samples: list[dict] = []
        self.blocks: list[dict] = []
        self.programs: list[dict] = []
        self.launch_cost: dict[str, int] | None = None
        # Per binary, a folder of another tree's library modules it builds against (`module_swap`).
        self.module_dirs: dict[str, tuple[str, ...]] = {name: () for name in compilers}
        for name, source in zip(NAMES, (args.baseline_modules, args.candidate_modules)):
            if source:
                folder = scratch / 'modules' / name
                module_swap.module_folder(source.resolve(), folder, [n for n in args.swap_modules.split(',') if n])
                self.module_dirs[name] = (str(folder),)
        # A binary built against another tree's modules may report on that tree's code: quote none of it.
        self.quote_output = not any(self.module_dirs.values())

    # one process --------------------------------------------------------------------------------

    def environment(self, name: str, tag: str) -> dict[str, str]:
        state = self.scratch / 'state' / tag / name
        (state / 'cache').mkdir(parents=True, exist_ok=True)
        (state / 'tmp').mkdir(exist_ok=True)
        return {**(self.runner.env or os.environ), 'JAIC_CACHE_DIR': str(state / 'cache'),
                'TMPDIR': str(state / 'tmp'), **self.envs[name]}

    def invocation(self, job: Job, name: str, workdir: Path, tag: str):
        plan = job.workload.plans[job.mode]
        where = self.scratch / 'work' / tag / job.workload.name / job.mode.value / name
        where.mkdir(parents=True, exist_ok=True)
        imports = tuple(str((workdir / d).resolve()) for d in plan.import_dirs)
        request = Request(workdir / plan.entry, workdir, job.mode, plan.metaprogram_args, where,
                          plan.redirect_output, imports, self.module_dirs[name])
        compiler = self.compilers[name]
        # Executables are built through the phase-timing metaprogram, so both binaries link with the
        # same system linker and the timed work is the same; so is every build against swapped modules.
        through_phases = job.mode.writes_executable or any(self.module_dirs.values())
        if through_phases and not JobRunner.has_own_build(request):
            return compiler.dialect.phase_invocation(compiler.path, request)
        return compiler.dialect.invocation(compiler.path, request)

    def run_once(self, job: Job, name: str, workdir: Path, tag: str) -> tuple[Run, object]:
        invocation = self.invocation(job, name, workdir, tag)
        if invocation.executable:
            invocation.executable.unlink(missing_ok=True)
        return self.runner.measure(invocation.command, invocation.cwd, self.environment(name, tag)), invocation

    def observe(self, job: Job, name: str, workdir: Path) -> Observed:
        run, invocation = self.run_once(job, name, workdir, 'check')
        if not run.ok:
            return Observed(run.returncode, [], error=run.failure() if self.quote_output else f'exit {run.returncode}')
        seen = Observed(run.returncode, normalized(run.stdout))
        executable = invocation.executable
        if job.mode.writes_executable and job.workload.compare_output:
            if not executable or not executable.exists():
                seen.error = 'the build wrote no executable'
                return seen
            program = self.runner.measure([str(executable)], invocation.cwd, self.environment(name, 'check'))
            if program.timed_out:
                seen.error = 'the program timed out'
            seen.program_exit, seen.program_stdout = program.returncode, normalized(program.stdout)
        if job.mode.writes_executable and executable and executable.exists():
            seen.program = program_facts.static_facts(executable)
            if job.workload.compare_output and not seen.error:
                seen.program.update(self.count(executable, invocation.cwd, self.environment(name, 'check')))
        return seen

    def count(self, executable: Path, cwd: Path | None, env: dict[str, str]) -> dict[str, int]:
        """Retired instructions and cycles of the built program, the median of `--program-runs` runs."""
        if self.launch_cost is None:
            # What starting any process through the same wrapper costs, so the counts are the program's own.
            self.launch_cost = self.counters([program_facts.NOTHING], cwd, env)
        return program_facts.subtract(self.counters([str(executable)], cwd, env), self.launch_cost)

    def counters(self, command: list[str], cwd: Path | None, env: dict[str, str]) -> dict[str, int]:
        if self.runner.sandbox:
            command = self.runner.sandbox.wrap(command)
        counted = program_facts.counter_command(command)
        if not counted or self.args.program_runs < 1:
            return {}
        command, pattern = counted
        runs = []
        for _ in range(self.args.program_runs):
            run = run_measured(command, cwd=cwd, env=env, timeout=self.args.timeout)
            if run.ok:
                runs.append(program_facts.parse_counters(run.stderr, pattern))
        return program_facts.median_counters(runs)

    # one block ----------------------------------------------------------------------------------

    def block(self, job: Job, phase: str, workdir: Path, rounds: int, source: dict) -> dict:
        names = list(self.compilers)
        key = f'{job.key}/{phase}'
        seed = block_seed(self.args.seed, key)
        orders = pairing.interleave(names, rounds, seed, shuffle=self.args.order == 'shuffle')
        before = quiet.snapshot()
        if phase == 'warm':
            warm_file_cache(workdir)
            for _ in range(self.args.warmup):
                for name in names:
                    self.run_once(job, name, workdir, 'warm')
        samples: list[dict] = []
        failure = ''
        for number, order in enumerate(orders):
            for position, name in enumerate(order):
                tag = f'cold-{number}' if phase == 'cold' else 'warm'
                if phase == 'cold' and self.purge_ok:
                    quiet.purge_caches()
                run, _ = self.run_once(job, name, workdir, tag)
                sample = {'workload': job.workload.name, 'mode': job.mode.value, 'phase': phase, 'round': number,
                          'position': position, 'compiler': name, 'wall_s': round(run.wall, 6),
                          'cpu_s': round(run.cpu, 6), 'rss_mib': round(run.rss_mib, 2), 'exit': run.returncode,
                          'stages': stage_seconds(self.compilers[name], run) if self.report_steps else {}}
                samples.append(sample)
                if not run.ok:
                    failure = f'{name}: {run.failure() if self.quote_output else f"exit {run.returncode}"}'
                if phase == 'cold':
                    shutil.rmtree(self.scratch / 'work' / tag, ignore_errors=True)
                    shutil.rmtree(self.scratch / 'state' / tag, ignore_errors=True)
        after = quiet.snapshot()
        self.samples += samples
        record = {'workload': job.workload.name, 'mode': job.mode.value, 'phase': phase, 'rounds': rounds,
                  'orders': orders, 'seed': seed, 'before': before, 'after': after,
                  'flags': quiet.block_flags(before, after, self.max_load), 'source': source}
        if failure:
            return {**record, 'status': 'failed', 'reason': failure}
        return {**record, 'status': 'ok', 'reason': '', 'summary': summarize_samples(samples, seed)}

    def job(self, job: Job) -> None:
        workload = job.workload
        missing = workload.source.available()
        if missing:
            self.blocks.append(skipped_block(job, 'both', 'skipped', missing))
            return
        workdir = workload.source.materialize(self.scratch)
        entry = next(iter(workload.plans.values())).entry
        source = {'category': workload.category.value, 'tree_sha256': provenance.tree_sha256(workdir),
                  'entry': entry, 'plan': {'import_dirs': list(workload.plans[job.mode].import_dirs)}}
        if isinstance(workload.source, Generated):
            g = workload.source
            source['generator'] = {'script': 'tools/benchgen.py', 'lines': g.lines, 'seed': g.seed, 'files': g.files}
        warm_file_cache(*(c.path for c in self.compilers.values()))
        names = list(self.compilers)
        seen = {n: self.observe(job, n, workdir) for n in names}
        reason = compare_outputs(seen[names[0]], seen[names[1]], workload.compare_output, self.quote_output)
        if reason:
            self.blocks.append(skipped_block(job, 'both', 'skipped', reason, source))
            return
        if all(seen[n].program for n in names):
            self.programs.append({'workload': workload.name, 'mode': job.mode.value,
                                  **{n: seen[n].program for n in names}})
            self.save()
        if self.args.programs_only:
            return
        phases = [('warm', self.args.rounds)] + ([('cold', self.args.cold_rounds)] if self.args.cold_rounds else [])
        for phase, rounds in phases:
            block = self.block(job, phase, workdir, rounds, source)
            self.blocks.append(block)
            self.save()
            common.print_progress(f'{job.key}/{phase}', block_progress(block))

    def save(self) -> None:
        self.on_save(self.blocks, self.samples, self.programs)

    on_save = staticmethod(lambda blocks, samples, programs: None)


def stage_seconds(compiler: Compiler, run: Run) -> dict[str, float]:
    if not run.ok:
        return {}
    return {bucket.value: round(seconds, 6) for bucket, seconds in compiler.dialect.detail(run).items()}


def skipped_block(job: Job, phase: str, status: str, reason: str, source: dict | None = None) -> dict:
    return {'workload': job.workload.name, 'mode': job.mode.value, 'phase': phase, 'status': status,
            'reason': reason, 'flags': [], 'source': source or {}}


def block_progress(block: dict) -> str:
    if block['status'] != 'ok':
        return f"{block['status']}: {block['reason']}"
    ratio = block['summary']['wall_ratio']
    return f"ratio {fmt_ratio(ratio)} {ratio['verdict']}" + (' [flagged]' if block['flags'] else '')


# ----------------------------------------------------------------------------------------- command

def run(args: argparse.Namespace) -> int:
    if args.rounds < 1 or args.cold_rounds < 0 or args.program_runs < 0:
        sys.exit('--rounds must be at least 1, and --cold-rounds and --program-runs at least 0')
    if bool(args.swap_modules) != bool(args.baseline_modules or args.candidate_modules):
        sys.exit('--swap-modules goes with --baseline-modules or --candidate-modules, and they with it')
    jobs = plan_jobs(args)
    if args.list:
        for job in jobs:
            print(f'{job.key:40} {job.workload.category.value}')
        return 0
    envs = {'baseline': split_env(args.baseline_env), 'candidate': split_env(args.candidate_env)}
    compilers = {}
    for name, path in zip(NAMES, (args.baseline, args.candidate)):
        compiler = parse_compiler(f'{name}={path}')
        if not compiler.path.exists():
            sys.exit(f'{compiler.path} ({name}) not found')
        compilers[name] = compiler
    compilers = same_command_line(compilers)
    if not args.no_sandbox:
        why = Sandbox.available()
        if why:
            sys.exit(f'cannot sandbox the compilers: {why}; install it or pass --no-sandbox')
    args.sandbox_read = [*args.sandbox_read, *(p for e in envs.values() for k, p in e.items()
                                               if k in ('JAIC_STDLIB', 'JAIC_NATIVE_LIBS')),
                         *(str(p.resolve()) for p in (args.baseline_modules, args.candidate_modules) if p)]

    start = quiet.snapshot()
    max_load = args.max_load or quiet.default_max_load(os.cpu_count())
    errors, warnings = quiet.start_problems(start, max_load)
    for line in errors:
        print(f'machine not quiet: {line}', file=sys.stderr)
    for line in warnings:
        print(f'warning: {line}', file=sys.stderr)
    if errors and not args.force:
        sys.exit('refusing to time on a busy machine (--force to run anyway; affected blocks are flagged)')

    out_dir = args.out_dir or default_out_dir()
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / 'command.txt').write_text(shlex.join([sys.executable, *sys.argv]) + '\n')
    binaries = {name: {'path': str(c.path), 'dialect': c.dialect.name, 'env': envs[name],
                       **provenance.binary_facts(c.path, {**os.environ, **envs[name]})}
                for name, c in compilers.items()}
    scratch = Path(tempfile.mkdtemp(prefix='bench-compare-'))
    try:
        comparison = Comparison(compilers, envs, scratch, args)
        results = {'format': FORMAT, 'date': datetime.now().astimezone().isoformat(timespec='seconds'),
                   'machine': provenance.machine_specs(), 'binaries': binaries,
                   'settings': {'rounds': args.rounds, 'cold_rounds': args.cold_rounds, 'warmup': args.warmup,
                                'order': args.order, 'seed': args.seed, 'modes': args.modes, 'sizes': args.sizes,
                                'corpus': not args.no_corpus, 'only': args.only, 'sandbox': not args.no_sandbox,
                                'max_load': max_load, 'forced': args.force, 'program_runs': args.program_runs,
                                'programs_only': args.programs_only,
                                'modules': {name: str(p) for name, p in zip(NAMES, (args.baseline_modules,
                                                                                    args.candidate_modules)) if p},
                                'swap_modules': args.swap_modules},
                   'purge': 'OS file cache purged before each cold run' if comparison.purge_ok
                            else f'file cache not purged: {comparison.purge_note}',
                   'start': start, 'start_warnings': [*errors, *warnings], 'blocks': []}

        def save(blocks, samples, programs):
            results['blocks'] = blocks
            results['programs'] = programs
            (out_dir / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
            provenance.write_raw_csv(out_dir / 'raw.csv', samples)

        comparison.on_save = save
        for job in jobs:
            comparison.job(job)
        save(comparison.blocks, comparison.samples, comparison.programs)
        results['inputs'] = {f"{b['workload']}": b['source'] for b in comparison.blocks if b.get('source')}
        (out_dir / 'inputs.json').write_text(json.dumps(results['inputs'], indent=2) + '\n')
        save(comparison.blocks, comparison.samples, comparison.programs)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    markdown = render(results)
    (out_dir / 'summary.md').write_text(markdown)
    print(markdown)
    print(f'wrote results to {out_dir}', file=sys.stderr)
    return 1 if any(b['status'] == 'failed' for b in comparison.blocks) else 0

