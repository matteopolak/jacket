"""`compile`: time one or more compilers on the same workloads, interleaved and gated on correctness."""
from __future__ import annotations

import argparse
import os
import shutil
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

import common
from common import Defaults
from compile_report import render
from dialects import Compiler, Invocation, Mode, Request, parse_compiler
from measure import Run, run_measured, warm_file_cache
from results import Report, Result, checkout_info, sh
from sizes import facts_of
from stages import Bucket, Stages, with_total
from stats import Stat
from workloads import Workload, all_workloads

DEFAULTS = Defaults(repeat=5, warmup=1, threshold=0.10, min_seconds=0.10, timeout=900)


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument('--compiler', action='append', metavar='NAME=PATH', default=[],
                        help='a compiler to time (repeatable; default jaic=target/release/jaic). A binary called '
                             'jaic* takes jaic options, any other the reference compiler\'s.')
    parser.add_argument('--modes', default=','.join(m.value for m in Mode),
                        help='comma-separated subset of ' + ', '.join(m.value for m in Mode))
    parser.add_argument('--phase-repeat', type=int, default=3, metavar='N',
                        help='runs per compiler of the phase-timing metaprogram that fills the stage table '
                             '(default 3, 0 for none)')
    parser.add_argument('--categories', default='', help='comma-separated substrings of category names to run')
    common.add_common_arguments(parser, 'compile', DEFAULTS)


# ------------------------------------------------------------------------------------------- jobs

@dataclass(frozen=True)
class Job:
    workload: Workload
    mode: Mode

    @property
    def key(self) -> str:
        return f'{self.workload.name}/{self.mode.value}'


def plan_jobs(args: argparse.Namespace) -> list[Job]:
    modes = {Mode(name) for name in args.modes.split(',') if name}
    categories = [c for c in args.categories.split(',') if c]
    jobs = []
    for workload in all_workloads():
        if categories and not any(c in workload.category.value for c in categories):
            continue
        jobs += [Job(workload, mode) for mode in workload.modes
                 if mode in modes and common.selected(args.only, f'{workload.name}/{mode.value}')]
    return jobs


def default_compilers() -> list[Compiler]:
    return [parse_compiler(f'jaic={common.default_binary("jaic")}')]


def normalized(text: str) -> list[str]:
    return [line.rstrip() for line in text.splitlines() if line.strip()]


# ----------------------------------------------------------------------------------- one workload

@dataclass
class Attempt:
    """One compiler's warmed-up first run of a job: whether it worked, and what the program printed."""
    run: Run
    program_output: list[str]
    error: str = ''


class JobRunner:
    def __init__(self, compilers: list[Compiler], scratch: Path, args: argparse.Namespace):
        self.compilers = compilers
        self.scratch = scratch
        self.args = args

    def request(self, job: Job, compiler: Compiler, workdir: Path) -> Request:
        plan = job.workload.plans[job.mode]
        where = self.scratch / 'work' / job.workload.name / job.mode.value / compiler.name
        where.mkdir(parents=True, exist_ok=True)
        return Request(workdir / plan.entry, workdir, job.mode, plan.metaprogram_args, where, plan.redirect_output)

    def driver(self, job: Job, compiler: Compiler, request: Request) -> Invocation:
        """How a compiler is run for the timed runs. Comparing compilers, an executable is built through the
        phase-timing metaprogram for all of them, so they all link with the same system linker (a reference
        compiler's bundled linker may not even run on a machine). A job with its own build metaprogram, or
        a single compiler, uses the compiler's native command line."""
        if len(self.compilers) > 1 and request.mode.writes_executable and not request.metaprogram_args:
            return compiler.dialect.phase_invocation(compiler.path, request)
        return compiler.dialect.invocation(compiler.path, request)

    def compile(self, compiler: Compiler, invocation: Invocation) -> Run:
        if invocation.executable:
            invocation.executable.unlink(missing_ok=True)
        return run_measured(invocation.command, cwd=invocation.cwd, timeout=self.args.timeout)

    def execute(self, invocation: Invocation) -> Run | None:
        """Run the executable a build wrote (None when there is none)."""
        if invocation.executable and invocation.executable.exists():
            return run_measured([str(invocation.executable)], cwd=invocation.cwd, timeout=self.args.timeout)
        return None

    def attempt(self, job: Job, compiler: Compiler, invocation: Invocation) -> Attempt:
        run = self.compile(compiler, invocation)
        if not run.ok:
            return Attempt(run, [], run.failure())
        program_output = normalized(run.stdout)
        if job.mode.writes_executable and job.workload.compare_output:
            program = self.execute(invocation)
            if program is None:
                return Attempt(run, [], 'the build wrote no executable')
            if not program.ok:
                return Attempt(run, [], f'the program failed: {program.failure()}')
            program_output = normalized(program.stdout)
        return Attempt(run, program_output)

    def gate(self, job: Job, attempts: dict[str, Attempt]) -> str:
        """'' when every compiler built the job and agrees on the output, otherwise why the job is not timed."""
        failed = {name: a.error for name, a in attempts.items() if a.error}
        if failed:
            return '; '.join(f'{name}: {error}' for name, error in failed.items())
        if len(attempts) > 1 and job.workload.compare_output:
            names = list(attempts)
            first = attempts[names[0]].program_output
            for name in names[1:]:
                other = attempts[name].program_output
                if other != first:
                    return f'output differs ({names[0]} vs {name}): {first_difference(first, other)}'
        return ''

    def run(self, job: Job) -> dict[str, Result]:
        """Results keyed `<workload>/<mode>/<compiler>` (and `<workload>/run-<mode>/<compiler>` for the
        generated program's own runtime)."""
        workload = job.workload
        missing = workload.source.available()
        if missing:
            return {f'{job.key}/{c.name}': Result.skipped(missing) for c in self.compilers}
        workdir = workload.source.materialize(self.scratch)
        entry = next(iter(workload.plans.values())).entry
        facts = facts_of(workload.source.measured(workdir, entry)).to_json()
        info = {'category': workload.category.value, 'source': facts}
        requests = {c.name: self.request(job, c, workdir) for c in self.compilers}
        invocations = {c.name: self.driver(job, c, requests[c.name]) for c in self.compilers}
        warm_file_cache(workdir)
        attempts = {c.name: self.attempt(job, c, invocations[c.name]) for c in self.compilers}
        reason = self.gate(job, attempts)
        if reason:
            make = Result.failed if len(self.compilers) == 1 else Result.skipped
            return {f'{job.key}/{c.name}': make(reason, **info) for c in self.compilers}
        for _ in range(max(0, self.args.warmup - 1)):
            for c in self.compilers:
                self.compile(c, invocations[c.name])
        samples = self.rounds(job, invocations)
        results = self.summarize(job, samples, invocations, info)
        self.add_stages(job, requests, results)
        if job.mode is Mode.INTERP:
            self.count_instructions(job, invocations, results)
        return results

    def add_stages(self, job: Job, requests: dict[str, Request], results: dict[str, Result]) -> None:
        """Stage seconds from `--phase-repeat` runs per compiler of the phase-timing metaprogram, the same
        for every compiler. A job with its own build metaprogram is not covered: the wrapper would build
        the entry with the default options instead."""
        if self.args.phase_repeat < 1 or job.workload.plans[job.mode].metaprogram_args:
            return
        invocations = {c.name: c.dialect.phase_invocation(c.path, requests[c.name]) for c in self.compilers}
        rounds: dict[str, list[Stages]] = {c.name: [] for c in self.compilers}
        for _ in range(self.args.phase_repeat):
            for c in self.compilers:
                run = self.compile(c, invocations[c.name])
                stages = c.dialect.phase_parser.parse(run.stdout) if run.ok else {}
                if stages:
                    rounds[c.name].append(with_total(stages, run.wall))
        for c in self.compilers:
            result = results.get(f'{job.key}/{c.name}')
            if result and rounds[c.name]:
                result.stages = stage_stats(rounds[c.name])

    def count_instructions(self, job: Job, invocations: dict[str, Invocation], results: dict[str, Result]) -> None:
        """One more run per compiler that can count interpreted instructions (not timed)."""
        for c in self.compilers:
            if not c.dialect.profiling_env:
                continue
            invocation = invocations[c.name]
            run = run_measured(invocation.command, cwd=invocation.cwd, timeout=self.args.timeout,
                               env={**os.environ, **c.dialect.profiling_env})
            count = c.dialect.instruction_count(run)
            result = results.get(f'{job.key}/{c.name}')
            if count and result:
                result.info['interp_instructions'] = count

    def rounds(self, job: Job, invocations: dict[str, Invocation]) -> dict[str, list[Sample]]:
        """Compile in rounds, every compiler once per round (A B A B), so drift hits all of them equally."""
        samples: dict[str, list[Sample]] = {c.name: [] for c in self.compilers}
        for _ in range(self.args.repeat):
            for c in self.compilers:
                invocation = invocations[c.name]
                run = self.compile(c, invocation)
                program = self.execute(invocation) if job.workload.runtime and run.ok else None
                samples[c.name].append(Sample(run, c.dialect.detail(run) if run.ok else {}, program,
                                              invocation.executable.stat().st_size
                                              if invocation.executable and invocation.executable.exists() else 0))
        return samples

    def summarize(self, job: Job, samples: dict[str, list[Sample]], invocations: dict[str, Invocation],
                  info: dict) -> dict[str, Result]:
        results = {}
        for c in self.compilers:
            mine = samples[c.name]
            bad = next((s.run for s in mine if not s.run.ok), None)
            if bad:
                results[f'{job.key}/{c.name}'] = Result.failed(bad.failure(), **info)
                continue
            binary = max((s.binary_bytes for s in mine), default=0)
            results[f'{job.key}/{c.name}'] = result_of([s.run for s in mine], {
                **info, **({'binary_bytes': binary} if binary else {})}, detail=stage_stats([s.detail for s in mine]))
            programs = [s.program for s in mine]
            if job.workload.runtime and all(p and p.ok for p in programs):
                results[f'{job.workload.name}/run-{job.mode.value}/{c.name}'] = result_of(
                    programs, {**info, 'binary_bytes': binary})
        return results


@dataclass
class Sample:
    run: Run
    detail: Stages
    program: Run | None
    binary_bytes: int


def result_of(runs: list[Run], info: dict, detail: dict[str, Stat] | None = None) -> Result:
    return Result(wall=Stat.of([r.wall for r in runs]), cpu=Stat.of([r.cpu for r in runs]),
                  rss_mib=Stat.of([r.rss_mib for r in runs]), detail=detail or {}, info=info)


def stage_stats(rounds: list[Stages]) -> dict[str, Stat]:
    """Per bucket, the spread of its seconds over the rounds."""
    return {bucket.value: Stat.of([stages.get(bucket, 0.0) for stages in rounds])
            for bucket in Bucket if any(bucket in stages for stages in rounds)}


def first_difference(a: list[str], b: list[str]) -> str:
    for index, (x, y) in enumerate(zip(a, b)):
        if x != y:
            return f'line {index + 1}: {x[:60]!r} vs {y[:60]!r}'
    return f'{len(a)} lines vs {len(b)} lines'


# ----------------------------------------------------------------------------------------- command

def compiler_info(compiler: Compiler) -> dict:
    info = compiler.describe()
    if compiler.dialect.name == 'jaic':
        info.update(checkout_info())
        info['rustc'] = sh('rustc', '--version')
    return info


def run(args: argparse.Namespace) -> int:
    common.check_arguments(args)
    try:
        compilers = [parse_compiler(spec) for spec in args.compiler] or default_compilers()
    except ValueError as error:
        sys.exit(str(error))
    names = [c.name for c in compilers]
    if len(set(names)) != len(names):
        sys.exit('--compiler names must be distinct')
    jobs = plan_jobs(args)
    if args.list:
        for job in jobs:
            print(f'{job.key:40} {job.workload.category.value}')
        return 0
    for compiler in compilers:
        if not compiler.path.exists():
            sys.exit(f'{compiler.path} (compiler {compiler.name}) not found; build jaic with '
                     'cargo build --release -p jaic-cli or pass --compiler NAME=PATH')
        warm_file_cache(compiler.path)
    report = Report('compile', {'repeat': args.repeat, 'warmup': args.warmup, 'modes': args.modes},
                    {c.name: compiler_info(c) for c in compilers})
    scratch = Path(tempfile.mkdtemp(prefix='bench-compile-'))
    try:
        runner = JobRunner(compilers, scratch, args)
        for job in jobs:
            results = runner.run(job)
            report.results.update(results)
            common.print_progress(job.key, progress_text(results))
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    return common.finish(report, render(report), args)


def progress_text(results: dict[str, Result]) -> str:
    parts = []
    for key, result in results.items():
        name = key.rsplit('/', 1)[1]
        parts.append(f'{name} {result.wall.median:.3f}s' if result.wall else f'{name} {result.status.value}')
    return '  '.join(parts)
