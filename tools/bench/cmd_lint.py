"""`lint`: jailint on the whole repository, one large file, `--fix` on a copy, and `-j` scaling."""
from __future__ import annotations

import argparse
import re
import shutil
import sys
import tempfile
from pathlib import Path

import toolpath  # noqa: F401  (puts tools/ on sys.path for benchgen, jaistats, repo_output)
import benchgen
import common
from common import Defaults
from results import ROOT, Report, checkout_info
from scenarios import Scenario, render_report, run_scenarios, wanted

DEFAULTS = Defaults(repeat=3, warmup=1, threshold=0.10, min_seconds=0.20, timeout=1800)
JOB_COUNTS = (1, 2, 4)
LARGE_FILE_LINES = 25_000
FINDING = re.compile(r'^(warning|error)\[', re.MULTILINE)
LINT_CASES = ROOT / 'tests' / 'lint'


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument('--jailint', type=Path, default=common.default_binary('jailint'), help='the linter (default: %(default)s)')
    common.add_common_arguments(parser, 'lint', DEFAULTS)


def densest_case() -> Path:
    """The `bad.jai` of tests/lint whose expected output reports the most findings: one file that
    triggers many rules' worth of diagnostics."""
    def findings(case: Path) -> int:
        return len(FINDING.findall((case.parent / 'bad.expected').read_text()))
    return max(sorted(LINT_CASES.glob('*/bad.jai')), key=findings)


def scenarios(jailint: Path, scratch: Path) -> list[Scenario]:
    """Everything runs from the scratch directory: the repository's jailint.toml excludes tests/lint."""
    exe = str(jailint)
    large = scratch / 'large'
    large.mkdir()
    (large / 'main.jai').write_text(benchgen.generate(LARGE_FILE_LINES, 1, 1)['main.jai'])
    cases = scratch / 'cases'
    shutil.copytree(LINT_CASES, cases)
    dense = cases / densest_case().relative_to(LINT_CASES)
    fix_copy = scratch / 'fix' / 'bad.jai'
    fix_copy.parent.mkdir()

    def fresh_copy() -> None:
        shutil.copyfile(dense, fix_copy)

    programs = len(list(cases.glob('*/*.jai')))
    found = [Scenario('repo', [exe], cwd=ROOT, ok_codes=(0, 1), heavy=True),
             Scenario('large-file', [exe, str(large / 'main.jai')], cwd=scratch, ok_codes=(0, 1),
                      inputs=(large,), info={'lines': LARGE_FILE_LINES}),
             Scenario('dense-file', [exe, str(dense)], cwd=scratch, ok_codes=(0, 1),
                      info={'file': dense.parent.name}),
             Scenario('fix-copy', [exe, '--fix', str(fix_copy)], cwd=scratch, ok_codes=(0, 1),
                      prepare=fresh_copy, info={'file': dense.parent.name})]
    found += [Scenario(f'many-rules-j{jobs}', [exe, '-j', str(jobs), str(cases)], cwd=scratch, ok_codes=(0, 1),
                       inputs=(cases,), info={'jobs': jobs, 'programs': programs})
              for jobs in JOB_COUNTS]
    return found


def run(args: argparse.Namespace) -> int:
    common.require_repeat(args)
    jailint = args.jailint.resolve()
    with tempfile.TemporaryDirectory(prefix='bench-lint-') as scratch:
        todo = [s for s in scenarios(jailint, Path(scratch)) if wanted(s, args.only)]
        if args.list:
            for scenario in todo:
                print(scenario.key)
            return 0
        if not jailint.exists():
            sys.exit(f'{jailint} not found; build it with cargo build --release -p jailint or pass --jailint')
        report = Report('lint', {'repeat': args.repeat, 'warmup': args.warmup},
                        {'jailint': {'path': str(jailint), **checkout_info()}})
        report.results = run_scenarios(todo, args)
    subject = [f"- `jailint`: `{jailint}`, `{report.subjects['jailint']['commit'][:12]}`"]
    return common.finish(report, render_report('Lint benchmark', report, subject), args)
