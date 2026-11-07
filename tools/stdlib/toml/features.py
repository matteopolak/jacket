"""Observe precision, profile and limit boundaries without conflating differences with failures.

    python3 features.py --own /path/decoder --rust /path/toml-oracle --community /path/community --scratch /tmp/feature-run --report /tmp/features.json

All libraries receive the same independently authored bytes. Community execution is sandboxed;
its public read/write output is retained verbatim, rather than converted through a precision-
losing datetime adapter. Diagnostics from that opaque implementation are never displayed.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

from bench.sandbox import Sandbox, environment

HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--own', type=Path, required=True)
    parser.add_argument('--rust', type=Path, required=True)
    parser.add_argument('--community', type=Path, required=True)
    parser.add_argument('--scratch', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if sys.platform not in ('darwin', 'linux'):
        parser.error('community subprocess comparison requires the macOS/Linux bench Sandbox launcher')
    scratch = args.scratch.resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    cases = {
        'fraction-offset-12': 'value=1979-05-27T07:32:00.123456789012+01:30\n',
        'fraction-local-12': 'value=07:32:00.123456789012\n',
        'negative-floating-zero': 'value=-0.0\n',
        'negative-zero-offset': 'value=1979-05-27T07:32:00-00:00\n',
        'historical-leap-spelling': 'value=2016-12-31T23:59:60Z\n',
        'structural-leap-spelling': 'value=2024-02-29T12:00:60Z\n',
        'integer-above-s64': 'value=9223372036854775808\n',
        'nested-arrays-129': 'value=' + '[' * 129 + '0' + ']' * 129 + '\n',
    }
    sandbox = Sandbox(reads=[args.community.resolve().parent], writes=[scratch])
    jobs = {
        'own': ([str(args.own.resolve())], None),
        'rust': ([str(args.rust.resolve())], None),
        'python': ([sys.executable, str(HERE / 'python_decoder.py')], None),
        'community-scientific': (sandbox.wrap([str(args.community.resolve()), 'scientific']), environment(scratch)),
    }
    report = dict(python=sys.version, binaries={name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in [('own', args.own), ('rust', args.rust), ('community', args.community)]}, cases=[])
    for name, source in cases.items():
        row = dict(name=name, input=source, outcomes={})
        for library, (command, env) in jobs.items():
            result = subprocess.run(command, input=source.encode(), capture_output=True, env=env, timeout=10)
            outcome = dict(exit=result.returncode, output=result.stdout.decode(errors='backslashreplace'))
            if result.returncode not in (0, 1):
                outcome['crash'] = True
            row['outcomes'][library] = outcome
        report['cases'].append(row)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(cases=len(cases), outcomes=len(cases) * len(jobs), crashes=sum(bool(outcome.get('crash')) for row in report['cases'] for outcome in row['outcomes'].values()))))


if __name__ == '__main__':
    main()
