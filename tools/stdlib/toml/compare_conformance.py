#!/usr/bin/env python3
"""Run the same pinned cases against independent Rust/Python and a permitted Jai black box.

The community adapter is deliberately labelled a read/write interoperability pipeline. Its
public API lacks documented tagged-value access; no implementation source is inspected.
"""
from __future__ import annotations
import argparse
from collections import Counter
import json
from pathlib import Path
import platform
import shlex
import shutil
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from run import PIN, prepare, sha256

HERE = Path(__file__).resolve().parent


def wrapper(path: Path, arguments: list[str]) -> Path:
    if sys.platform == 'win32':
        raise RuntimeError('Python/community comparator wrappers currently require a POSIX host')
    path.write_text('#!/bin/sh\nexec ' + shlex.join(arguments) + '\n')
    path.chmod(0o755)
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--rust', type=Path)
    parser.add_argument('--community', type=Path)
    parser.add_argument('--own-adapter', type=Path)
    parser.add_argument('--python', action='store_true')
    parser.add_argument('--community-float-mode', choices=('default', 'shortest', 'scientific', 'scientific-compact'), default='default')
    args = parser.parse_args()
    cache = args.cache.resolve()
    runner, expected = prepare(cache)
    jobs = []
    if args.python:
        if sys.version_info < (3, 15):
            raise RuntimeError('Python comparator requires explicit TOML 1.1 support (3.15+)')
        decoder = wrapper(cache / 'python-decoder', [sys.executable, str(HERE / 'python_decoder.py')])
        jobs.append(('python', decoder, None, {'version': platform.python_version(), 'mode': 'tomllib decoder'}))
    if args.rust:
        encoder = cache / ('rust-encoder.exe' if sys.platform == 'win32' else 'rust-encoder')
        shutil.copyfile(args.rust, encoder)
        encoder.chmod(0o755)
        jobs.append(('rust', args.rust.resolve(), encoder, {'version': 'toml 1.1.2', 'mode': 'independent native decoder and semantic encoder'}))
    if args.community:
        if not args.own_adapter:
            parser.error('--community requires --own-adapter for tagged construction')
        common = [sys.executable, str(HERE / 'community_pipeline.py'), '--community', str(args.community.resolve()), '--own-adapter', str(args.own_adapter.resolve()), '--scratch', str(cache / 'community-scratch')]
        common += ['--float-mode', args.community_float_mode]
        decoder = wrapper(cache / 'community-decoder', common)
        encoder = wrapper(cache / 'community-encoder', [*common, '--encode'])
        jobs.append(('community-' + args.community_float_mode, decoder, encoder, {'pin': json.loads((HERE / 'community-pin.json').read_text()), 'float_mode': args.community_float_mode, 'community_binary_sha256': sha256(args.community.read_bytes()), 'adapter_source_sha256': sha256((HERE / 'community.jai').read_bytes()), 'mode': 'read/write interoperability pipeline; decoder includes Python tagged conversion; encoder includes own semantic construction before community parse/write'}))
    if not jobs:
        parser.error('select at least one comparator')
    summaries = {}
    for name, decoder, encoder, metadata in jobs:
        command = [str(runner), 'test', '-toml=1.1', f'-decoder={decoder}', '-json', '-v', '-parallel=2']
        if encoder:
            command.append(f'-encoder={encoder}')
        result = subprocess.run(command, capture_output=True, text=True, timeout=1200)
        report = json.loads(result.stdout)
        report['comparison'] = metadata
        report['pin'] = PIN
        report['exit_code'] = result.returncode
        report['stderr'] = result.stderr
        report['decoder_sha256'] = sha256(decoder.read_bytes())
        report_path = cache / f'{name}.json'
        if report_path.is_file():
            prior = report_path.read_bytes()
            (cache / f'{name}.previous-{sha256(prior)[:12]}.json').write_bytes(prior)
        report_path.write_text(json.dumps(report, indent=2) + '\n')
        wanted = {case for case in expected if encoder or not case.startswith('encoder/')}
        if Counter(row['path'] for row in report['tests']) != Counter({case: 1 for case in wanted}):
            raise RuntimeError(f'{name}: incomplete result inventory')
        if report['skipped'] or any(row['skipped'] for row in report['tests']):
            raise RuntimeError(f'{name}: comparison exclusions are forbidden')
        summaries[name] = {key: report[key] for key in report if key.startswith(('passed_', 'failed_'))}
    (cache / 'comparison-summary.json').write_text(json.dumps(summaries, indent=2) + '\n')
    print(json.dumps(summaries, indent=2))
    # Comparator failures are evidence, not a reason to weaken or hide the comparison.
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
