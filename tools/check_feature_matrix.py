#!/usr/bin/env python3
"""Assert self-written feature fixtures through distinct compiler/native stages."""
from __future__ import annotations

import argparse
from dataclasses import asdict
import json
import os
import stat
from pathlib import Path
import subprocess
import tempfile

from check_corpus import (ROOT, Evidence, Source, Stage, Status, compiler_fingerprint,
                          digest, evaluate, inside, totals, validate_cases)

FIXTURES = ROOT/'tests/corpus'


def load_cases(root: Path = FIXTURES) -> tuple[dict, list[tuple[Source, dict]]]:
    manifest = json.loads((root/'manifest.json').read_text())
    validate_cases(manifest)
    cases = []
    listed = set()
    for case in manifest['cases']:
        if case.get('suite') not in ('core', 'expanded', 'negative'):
            raise ValueError('invalid feature suite')
        path = inside(root, case['source'])
        if path.suffix != '.jai' or not path.is_file():
            raise ValueError('fixture must be a local Jai source')
        listed.add(path)
        expected = case.get('sha256')
        if not isinstance(expected, str) or len(expected) != 64 or digest(path) != expected:
            raise ValueError(f"fixture missing or fingerprint changed: {case['id']}")
        for support in case.get('support_sources', []):
            supporting = inside(root, support['source'])
            listed.add(supporting)
            if supporting.suffix != '.jai' or not supporting.is_file() or digest(supporting) != support['sha256']:
                raise ValueError('support fixture missing or fingerprint changed')
        if case['kind'] == 'program':
            runtime = case.get('runtime', {})
            if not case.get('host_build_reviewed') or set(runtime) != {'exit_code', 'stdout', 'stderr'}:
                raise ValueError('program requires reviewed host build and exact runtime assertions')
            if type(runtime['exit_code']) is not int or not 0 <= runtime['exit_code'] <= 255:
                raise ValueError('runtime exit code must be 0..255')
            if not all(isinstance(runtime[key], str) for key in ('stdout', 'stderr')):
                raise ValueError('runtime output expectations must be strings')
        cases.append((Source(case['id'], path, expected, expected, 'self-written-fixtures', None), case))
    actual = {path.resolve() for path in root.rglob('*.jai')}
    if actual != listed:
        raise ValueError('unlisted fixture source; fingerprint every dependency')
    return manifest, cases


def runtime_artifact_hash(output: Path) -> str:
    metadata = output.lstat()
    if not stat.S_ISREG(metadata.st_mode) or not metadata.st_mode & 0o111 or not os.access(output, os.X_OK):
        raise ValueError('runtime artifact must be a regular nonsymlink executable file')
    return digest(output)


def run_native(output: Path, expected: dict, timeout: float, *, expected_sha256: str | None) -> Evidence:
    command = [str(output)]
    try:
        before = runtime_artifact_hash(output)
    except (OSError, ValueError) as error:
        return Evidence(Status.FAILED, str(error), command)
    if not expected_sha256 or before != expected_sha256:
        return Evidence(Status.FAILED, 'runtime artifact differs from successful BUILD fingerprint', command,
                        output_sha256=before)
    try:
        run = subprocess.run(command, capture_output=True, timeout=timeout, cwd=output.parent)
    except subprocess.TimeoutExpired:
        return Evidence(Status.FAILED, 'generated fixture runtime timeout', command, output_sha256=before)
    except OSError as error:
        return Evidence(Status.BLOCKED, str(error), command, output_sha256=before)
    diagnostic = run.stderr.decode('utf-8', errors='replace')[:8192]
    try:
        after = runtime_artifact_hash(output)
    except (OSError, ValueError) as error:
        return Evidence(Status.FAILED, f'generated executable unavailable after runtime: {error}',
                        command, run.returncode, diagnostic, before)
    if after != before:
        return Evidence(Status.FAILED, 'generated executable changed during runtime', command,
                        run.returncode, diagnostic, after)
    # Byte assertions retain embedded NULs and do not discard invalid UTF-8.
    matched = (run.returncode == expected['exit_code'] and
               run.stdout == expected['stdout'].encode() and
               run.stderr == expected['stderr'].encode())
    return Evidence(Status.PASSED if matched else Status.FAILED,
                    '' if matched else 'generated fixture behavior differs from exact assertions',
                    command, run.returncode, diagnostic, before)


def check(source: Source, case: dict, compiler: Path, through: Stage, timeout: float, output: Path):
    result = evaluate(source, case, compiler, through, timeout, output)
    if through == Stage.RUN and case['kind'] == 'program' and result.stages['build'].status == Status.PASSED:
        result.stages['run'] = run_native(output, case['runtime'], timeout,
                                          expected_sha256=result.stages['build'].output_sha256)
    elif case['kind'] == 'negative':
        result.stages['run'] = Evidence(Status.NOT_RUN, 'negative fixture has no runtime')
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, default=ROOT/'target/debug/jai-rs')
    parser.add_argument('--suite', choices=['all', 'core', 'expanded', 'negative'], default='all')
    parser.add_argument('--select', action='append', default=[])
    parser.add_argument('--through', type=Stage, choices=list(Stage), default=Stage.CHECK)
    parser.add_argument('--timeout', type=float, default=10)
    parser.add_argument('--report', type=Path, default=ROOT/'artifacts/feature-matrix.json')
    args = parser.parse_args()
    compiler = args.compiler.resolve()
    if not compiler.is_relative_to((ROOT/'target').resolve()) or compiler.name != 'jai-rs' or not compiler.is_file():
        parser.error('compiler must be repository-built target/.../jai-rs')
    if args.timeout <= 0: parser.error('timeout must be positive')
    report = args.report.resolve()
    if any(report.is_relative_to(path.resolve()) for path in (ROOT/'reference', ROOT/'corpus/upstream', FIXTURES)):
        parser.error('report must be outside source and fixture corpus')
    try:
        manifest, cases = load_cases()
    except ValueError as error:
        parser.error(str(error))
    known = {source.id for source, _ in cases}
    if set(args.select) - known: parser.error('unknown fixture selection')
    selected = [(source, case) for source, case in cases
                if (source.id in args.select if args.select else args.suite == 'all' or case['suite'] == args.suite)]
    results = []
    with tempfile.TemporaryDirectory(prefix='jai-features-') as directory:
        for index, (source, case) in enumerate(selected):
            results.append(check(source, case, compiler, args.through, args.timeout, Path(directory)/f'{index}.output'))
    payload = {'format': 1, 'compiler': compiler_fingerprint(ROOT, compiler), 'through': args.through.value,
               'manifest_sha256': digest(FIXTURES/'manifest.json'), 'provenance': manifest['provenance'],
               'selected': len(selected), 'totals': totals(results), 'results': [asdict(result) for result in results],
               'project_build_successes': 0,
               'limitations': ['Small self-written feature contracts do not establish full corpus or standard-library compilation.',
                               'Native execution is bounded host execution, not a VM isolation guarantee.',
                               'Expanded fixture failures are missing acceptance contracts, never successful unsupported-feature rejections.']}
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(payload, indent=2)+'\n')
    print(json.dumps({'selected': len(selected), 'totals': payload['totals'], 'report': str(report)}))
    failed = any(evidence.status in (Status.FAILED, Status.UNEXPECTED_ACCEPTANCE, Status.BLOCKED)
                 for result in results for stage, evidence in result.stages.items()
                 if stage != 'run' or args.through == Stage.RUN)
    # A negative fixture cannot pass when its designated rejection stage was not attempted.
    untested_negative = any(case['kind'] == 'negative' and
                            not any(result.stages[stage].status == Status.EXPECTED_REJECTION for stage in case['negative'])
                            for (_, case), result in zip(selected, results))
    return int(failed or untested_negative)

if __name__ == '__main__':
    raise SystemExit(main())
