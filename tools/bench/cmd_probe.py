"""`probe`: run Jai files under jaic and under a reference compiler, side by side, and diff what they do."""
from __future__ import annotations

import argparse
import difflib
import json
import os
import re
import shutil
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

from bench import common
from bench.cmd_compile import install_reads
from bench.dialects import Compiler, PlainDialect
from bench.measure import run_measured
from bench.results import ROOT
from bench.sandbox import Sandbox, environment

# The reference cannot link on every machine, so a program runs at compile time: the wrapper loads the
# file, turns output off and (in the run phase) runs `main`. Both sides get the same wrappers and the same
# command line. The check phase leaves `main` alone, so "compiled" is the compiler's own verdict on the
# program, apart from what `main` does.
WRAPPER = '''#import "Compiler";
#run set_build_options_dc(.{{do_output=false}});
#load "{entry}";
{run}'''
RUN = '#run main();\n'
REFERENCE_ENV = 'JAI_REFERENCE'
ANSI = re.compile(r'\x1b\[[0-9;]*m')
SIDES = ('jaic', 'reference')
FIELDS = ('compiled', 'exit', 'stdout', 'stderr', 'timed_out')


@dataclass
class Side:
    name: str
    compiled: bool
    exit: int
    stdout: str
    stderr: str
    timed_out: bool = False

    def to_json(self) -> dict:
        return {'compiled': self.compiled, 'exit': self.exit, 'stdout': self.stdout, 'stderr': self.stderr,
                'timed_out': self.timed_out}


@dataclass
class Probe:
    file: str
    sides: dict[str, Side]
    differences: list[str] = field(default_factory=list)

    @property
    def agree(self) -> bool:
        return not self.differences

    def to_json(self) -> dict:
        return {'file': self.file, 'agree': self.agree, 'differences': self.differences,
                'sides': {name: side.to_json() for name, side in self.sides.items()}}


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument('files', nargs='+', type=Path, metavar='FILE.jai')
    parser.add_argument('--jaic', type=Path, default=None, help='the jaic binary (default: the release build)')
    parser.add_argument('--reference', type=Path, default=None, metavar='PATH',
                        help=f'the reference compiler binary (default: ${REFERENCE_ENV})')
    parser.add_argument('--x64', action='store_true', help='run both compilers under Rosetta (`arch -x86_64`)')
    parser.add_argument('--json', action='store_true', help='print machine-readable results instead of a report')
    parser.add_argument('--timeout', type=float, default=120, help='seconds before a compile-and-run is killed')
    parser.add_argument('--import-dir', action='append', default=[], metavar='DIR',
                        help='a module folder both compilers search (repeatable)')
    parser.add_argument('--sandbox-read', action='append', default=[], metavar='PATH',
                        help='another path the compilers may read (repeatable); each file\'s folder is always readable')
    parser.add_argument('--show', action='store_true', help='print each side\'s output even when they agree')


# ----------------------------------------------------------------------------------------- running

def normalize(text: str, names: dict[Path, str]) -> str:
    """Output without colour codes, trailing blanks or the paths in `names` (replaced by their labels, so
    a scratch folder or install location never decides whether two outputs agree)."""
    text = ANSI.sub('', text)
    forms = {form: label for path, label in names.items() for form in {str(path), str(path.resolve())}}
    for form in sorted(forms, key=len, reverse=True):
        text = text.replace(form, forms[form])
    return '\n'.join(line.rstrip() for line in text.replace('\r\n', '\n').rstrip('\n').split('\n')) if text else ''


def command_for(compiler: Compiler, wrapper: str, import_dirs: tuple[str, ...], x64: bool) -> list[str]:
    command = [*compiler.dialect.source_command(compiler.path, wrapper), *compiler.dialect.import_flags(import_dirs)]
    return ['arch', '-x86_64', *command] if x64 else command


def probe_side(compiler: Compiler, source: Path, scratch: Path, args: argparse.Namespace, extra_reads: list[Path]) -> Side:
    """Check the program, then (if it compiled) run it. A program that does not compile reports the check's
    output and exit code; otherwise the run's."""
    where = scratch / compiler.name
    where.mkdir(parents=True, exist_ok=True)
    install = compiler.path.parent.parent if compiler.path.parent.name == 'bin' else compiler.path.parent
    names = {where: '<scratch>', source.resolve().parent: '<source>', install: '<install>', ROOT: '<repo>'}
    sandbox = Sandbox(reads=[*install_reads(compiler), source.resolve().parent, *extra_reads], writes=[where])
    env = environment(where)
    try:
        for phase, run_text in (('check', ''), ('run', RUN)):
            (where / 'probe.jai').write_text(WRAPPER.format(entry=source.resolve().as_posix(), run=run_text))
            command = sandbox.wrap(command_for(compiler, 'probe.jai', tuple(args.import_dir), args.x64))
            run = run_measured(command, cwd=where, env=env, timeout=args.timeout)
            if run.returncode != 0 or run.timed_out:
                break
    finally:
        sandbox.close()
    compiled = phase == 'run' or (run.returncode == 0 and not run.timed_out)
    return Side(compiler.name, compiled, run.returncode, normalize(run.stdout, names), normalize(run.stderr, names),
                run.timed_out)


def compare_sides(a: Side, b: Side) -> list[str]:
    """The fields in which the two sides differ."""
    return [name for name in FIELDS if getattr(a, name) != getattr(b, name)]


def probe_file(source: Path, compilers: dict[str, Compiler], scratch: Path, args: argparse.Namespace) -> Probe:
    extra = [Path(p).resolve() for p in (*args.sandbox_read, *args.import_dir)]
    sides = {name: probe_side(compiler, source, scratch / source.stem, args, extra)
             for name, compiler in compilers.items()}
    return Probe(str(source), sides, compare_sides(sides['jaic'], sides['reference']))


# --------------------------------------------------------------------------------------- reporting

def diff_text(a: str, b: str, limit: int = 40) -> str:
    lines = list(difflib.unified_diff(a.split('\n'), b.split('\n'), 'jaic', 'reference', lineterm='', n=1))
    more = len(lines) - limit
    return '\n'.join(lines[:limit] + ([f'... {more} more lines'] if more > 0 else []))


def render(probes: list[Probe], show: bool) -> str:
    out = []
    for probe in probes:
        jaic, reference = probe.sides['jaic'], probe.sides['reference']
        out.append(f'{"AGREE" if probe.agree else "DIFFER"}  {probe.file}')
        if probe.agree and not show:
            continue
        if not probe.agree:
            out.append(f'  differs in: {", ".join(probe.differences)}')
        for side in (jaic, reference):
            out.append(f'  {side.name}: compiled={side.compiled} exit={side.exit}'
                       + (' (timed out)' if side.timed_out else ''))
        for name in ('stdout', 'stderr'):
            a, b = getattr(jaic, name), getattr(reference, name)
            if a != b:
                out.append(f'  {name}:\n' + '\n'.join('    ' + line for line in diff_text(a, b).split('\n')))
            elif show and a:
                out.append(f'  {name} (both):\n' + '\n'.join('    ' + line for line in a.split('\n')))
    differing = sum(not p.agree for p in probes)
    out.append(f'{len(probes)} probed, {len(probes) - differing} agree, {differing} differ')
    return '\n'.join(out)


# ------------------------------------------------------------------------------------------ command

def reference_path(args: argparse.Namespace) -> Path:
    given = args.reference or (Path(os.environ[REFERENCE_ENV]) if os.environ.get(REFERENCE_ENV) else None)
    if not given:
        sys.exit(f'no reference compiler: pass --reference PATH or set {REFERENCE_ENV}')
    return Path(given).expanduser().resolve()


def run(args: argparse.Namespace) -> int:
    why = Sandbox.available()
    if why:
        sys.exit(f'cannot sandbox the compilers: {why}')
    # The dialects are fixed, not probed: probing runs `-help` on the binary outside the sandbox.
    jaic = Path(args.jaic) if args.jaic else common.default_binary('jaic')
    compilers = {'jaic': Compiler('jaic', jaic.resolve(), PlainDialect()),
                 'reference': Compiler('reference', reference_path(args), PlainDialect())}
    for compiler in compilers.values():
        if not compiler.path.exists():
            sys.exit(f'{compiler.path} ({compiler.name}) not found')
    missing = [f for f in args.files if not f.is_file()]
    if missing:
        sys.exit(f'not a file: {missing[0]}')
    scratch = Path(tempfile.mkdtemp(prefix='bench-probe-'))
    try:
        probes = [probe_file(f, compilers, scratch, args) for f in args.files]
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    if args.json:
        print(json.dumps({'probes': [p.to_json() for p in probes]}, indent=2))
    else:
        print(render(probes, args.show))
    return 0 if all(p.agree for p in probes) else 1
