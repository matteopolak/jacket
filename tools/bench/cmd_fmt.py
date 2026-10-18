"""`fmt`: jaifmt on the repository, a very large file, stdin, formatted and unformatted input, native and wasm."""
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import toolpath  # noqa: F401  (puts tools/ on sys.path for benchgen, jaistats, repo_output)
import benchgen
import common
from common import Defaults
from results import ROOT, Report, Result, checkout_info, sh
from scenarios import Scenario, render_report, run_scenarios, wanted

DEFAULTS = Defaults(repeat=5, warmup=1, threshold=0.10, min_seconds=0.05, timeout=900)
LARGE_FILE_LINES = 100_000
# The paths CI formats (see docs/tools/jaifmt.md).
REPOSITORY_PATHS = ['stdlib', 'tests', 'benchmarks', 'tools', 'jaifmt', 'examples']
SAMPLE_LINES = 20_000
CONFIG = ROOT / 'jaifmt.toml'
LEADING_SPACE = re.compile(r'^[ \t]+', re.MULTILINE)
SPACE_RUNS = re.compile(r'(?<=\S)[ \t]{2,}')


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument('--jaifmt', type=Path, default=ROOT / 'target' / 'jaifmt', help='the native formatter (default: %(default)s)')
    parser.add_argument('--wasm', type=Path, default=ROOT / 'target' / 'jaifmt.wasm', help='the WebAssembly build (default: %(default)s)')
    parser.add_argument('--node', default='node', help='node 24 or newer, to run the WebAssembly build')
    common.add_common_arguments(parser, 'fmt', DEFAULTS)


def unformat(text: str) -> str:
    """The same tokens with their layout wrecked: no indentation and runs of spaces collapsed to one. The
    formatter has to rebuild all of it, where already formatted input leaves it nothing to change."""
    return SPACE_RUNS.sub(' ', LEADING_SPACE.sub('', text))


def supports(jaifmt: Path, word: str) -> bool:
    return word in sh(str(jaifmt), '--help')


class Inputs:
    """Files the scenarios format, written once to a scratch directory."""

    def __init__(self, scratch: Path, jaifmt: Path):
        generated = benchgen.generate(SAMPLE_LINES, 1, 1)['main.jai']
        formatted = subprocess.run([str(jaifmt), '--stdin', '--config', str(CONFIG)], input=generated,
                                   capture_output=True, text=True).stdout
        if not formatted:
            sys.exit(f'{jaifmt} could not format the generated sample')
        self.formatted = scratch / 'formatted.jai'
        self.unformatted = scratch / 'unformatted.jai'
        self.formatted.write_text(formatted)
        self.unformatted.write_text(unformat(formatted))
        self.large = scratch / 'large.jai'
        self.large.write_text(benchgen.generate(LARGE_FILE_LINES, 1, 1)['main.jai'])
        self.rewrite = scratch / 'rewrite' / 'input.jai'
        self.rewrite.parent.mkdir()

    def fresh_unformatted(self) -> None:
        shutil.copyfile(self.unformatted, self.rewrite)


def native_scenarios(jaifmt: Path, inputs: Inputs) -> list[Scenario]:
    exe = str(jaifmt)
    config = ['--config', str(CONFIG)]  # the scratch copies are formatted by the repository's own settings
    paths = [p for p in REPOSITORY_PATHS if (ROOT / p).exists()]
    formatted, unformatted = inputs.formatted.read_bytes(), inputs.unformatted.read_bytes()
    return [
        Scenario('repo-check', [exe, '--check', *paths], cwd=ROOT, ok_codes=(0, 1), info={'paths': paths}),
        Scenario('large-file-check', [exe, '--check', *config, str(inputs.large)], ok_codes=(0, 1),
                 inputs=(inputs.large,), info={'lines': LARGE_FILE_LINES}),
        # Exit status 0 means nothing to change, 1 that files would change: a formatted input that
        # reports 1 (or the reverse) means the scenario no longer measures what its name says.
        Scenario('check-formatted', [exe, '--check', *config, str(inputs.formatted)], inputs=(inputs.formatted,)),
        Scenario('check-unformatted', [exe, '--check', *config, str(inputs.unformatted)], ok_codes=(1,),
                 inputs=(inputs.unformatted,)),
        Scenario('rewrite-unformatted', [exe, *config, str(inputs.rewrite)], prepare=inputs.fresh_unformatted),
        Scenario('stdin-formatted', [exe, '--stdin', *config], stdin_bytes=formatted),
        Scenario('stdin-unformatted', [exe, '--stdin', *config], stdin_bytes=unformatted),
    ]


def wasm_scenarios(node: str, wasm: Path, inputs: Inputs) -> list[Scenario]:
    # The module takes the text of the configuration, not a path.
    command = [node, '--no-warnings', str(ROOT / 'tools' / 'wasi_run.mjs'), str(wasm), '--name', 'main.jai',
               '--config', CONFIG.read_text()]
    return [Scenario('wasm-stdin-formatted', command, stdin_bytes=inputs.formatted.read_bytes()),
            Scenario('wasm-stdin-unformatted', command, stdin_bytes=inputs.unformatted.read_bytes())]


def unavailable(reason: str, *keys: str) -> dict[str, Result]:
    return {key: Result.skipped(reason) for key in keys}


def run(args: argparse.Namespace) -> int:
    common.require_repeat(args)
    jaifmt, wasm = args.jaifmt.resolve(), args.wasm.resolve()
    with tempfile.TemporaryDirectory(prefix='bench-fmt-') as scratch:
        if not jaifmt.exists():
            sys.exit(f'{jaifmt} not found; build it with `jaic jaifmt/build.jai` or pass --jaifmt')
        inputs = Inputs(Path(scratch), jaifmt)
        todo = native_scenarios(jaifmt, inputs)
        skipped: dict[str, Result] = {}
        if supports(jaifmt, 'range'):
            todo.append(Scenario('range-stdin', [str(jaifmt), '--stdin', '--range', '1-400'],
                                 stdin_bytes=inputs.formatted.read_bytes()))
        else:
            skipped['range-stdin'] = Result.skipped('jaifmt has no range option')
        node = shutil.which(args.node)
        if wasm.exists() and node:
            todo += wasm_scenarios(node, wasm, inputs)
        else:
            reason = f'{wasm} is missing' if not wasm.exists() else f'{args.node} is not installed'
            skipped.update(unavailable(reason, 'wasm-stdin-formatted', 'wasm-stdin-unformatted'))
        todo = [s for s in todo if wanted(s, args.only)]
        if args.list:
            for key in [s.key for s in todo] + list(skipped):
                print(key)
            return 0
        report = Report('fmt', {'repeat': args.repeat, 'warmup': args.warmup},
                        {'jaifmt': {'path': str(jaifmt), **checkout_info()}, 'wasm': {'path': str(wasm)}})
        report.results = run_scenarios(todo, args)
        report.results.update({k: v for k, v in skipped.items() if common.selected(args.only, k)})
    subject = [f"- `jaifmt`: `{jaifmt}`; wasm build `{wasm}`"]
    return common.finish(report, render_report('Formatter benchmark', report, subject), args)
