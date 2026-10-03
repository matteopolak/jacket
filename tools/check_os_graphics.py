#!/usr/bin/env python3
"""Check genuine pinned OS and graphics roots without native builds or effects."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import tempfile

from check_corpus import ROOT, digest, inventory
from check_standard_libraries import LOCATION, bootstrap_search_sources, controlled_environment, profile_environment

ROOTS = {
    'reference:modules/File/module.jai': 'module',
    'reference:modules/Process/module.jai': 'module',
    'reference:modules/Thread/module.jai': 'module',
    'reference:modules/File_Async/module.jai': 'module',
    'focus-editor/focus:modules/File_Async/module.jai': 'module',
    'ostef/Vk-Engine:Build.jai': 'metaprogram',
    'roeyb1/sgpu:examples/build.jai': 'metaprogram',
    'roeyb1/sgpu:examples/01_memory.jai': 'program',
}


def validate_compiler(compiler: Path, expected: str) -> Path:
    compiler = compiler.resolve()
    allowed = [ROOT/'target', ROOT/'artifacts/integration-checkpoints']
    if compiler.name != 'jai-rs' or not any(compiler.is_relative_to(path.resolve()) for path in allowed):
        raise ValueError('compiler must be our built CLI or an owned integration checkpoint')
    if not compiler.is_file() or digest(compiler) != expected:
        raise ValueError('compiler SHA-256 differs from the selected frozen checkpoint')
    return compiler


def check(compiler: Path, expected: str, timeout: float) -> dict:
    compiler = validate_compiler(compiler, expected)
    sources = inventory(ROOT)
    for source in sources:
        if source.sha256 != source.expected_sha256:
            raise ValueError(f'pinned source missing or changed: {source.id}')
    known = {source.id: source for source in sources}
    locations = {str(source.path.resolve()): source for source in sources}
    results = []
    with tempfile.TemporaryDirectory(prefix='jai-os-graphics-source-') as directory:
        for source_id, kind in ROOTS.items():
            source = known[source_id]
            environment = profile_environment(source, 'preload')
            row = {'id': source_id, 'source': str(source.path), 'sha256': source.sha256,
                   'revision': source.revision, 'kind': kind,
                   'bootstrap_search': bootstrap_search_sources(environment), 'stages': {}}
            for stage in ['parse', 'check']:
                action = 'check-library' if stage == 'check' and kind == 'module' else stage
                command = [str(compiler), action, str(source.path)]
                proof = {'command': command, 'environment': environment, 'status': 'failed',
                         'exit_code': None, 'diagnostic': '', 'reason': ''}
                try:
                    result = subprocess.run(command, cwd=directory, env=controlled_environment(environment),
                                            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=timeout)
                    diagnostic = next((line for line in result.stderr.decode(errors='replace').splitlines() if line.strip()), '')[:4096]
                    proof.update(exit_code=result.returncode, status='passed' if result.returncode == 0 else 'failed', diagnostic=diagnostic)
                    location = LOCATION.match(diagnostic)
                    if location:
                        path, line, column, message = location.groups()
                        failure_source = locations.get(str(Path(path).resolve()))
                        proof['failure_location'] = {'path': path, 'line': int(line), 'column': int(column),
                            'message': message, 'source_id': failure_source.id if failure_source else None,
                            'sha256': failure_source.sha256 if failure_source else None,
                            'source_hash_verified': bool(failure_source and digest(failure_source.path) == failure_source.expected_sha256)}
                except subprocess.TimeoutExpired:
                    proof['reason'] = 'compiler timeout'
                except OSError as error:
                    proof.update(status='blocked', reason=str(error))
                row['stages'][stage] = proof
                print(source_id, stage, proof['status'], proof['diagnostic'] or proof['reason'], flush=True)
                if proof['status'] != 'passed':
                    break
            results.append(row)
    validate_compiler(compiler, expected)
    return {'format': 1, 'compiler': {'binary': str(compiler), 'binary_sha256': expected,
            'evidence_kind': 'integrated-cli-frozen-checkpoint'},
            'input_manifests': {path: digest(ROOT/path) for path in ['corpus/reference-inputs.json', 'corpus/upstreams.json']},
            'selected': len(results), 'results': results, 'project_build_successes': 0,
            'limitations': ['Actual pinned module/application roots: parse and NoEffects check-library/check only.',
                'Real reference Preload enabled; Runtime_Support disabled.',
                'No native bytes loaded; no upstream scripts, builds, links or executions.',
                'All pinned source input hashes verified. Current tree source fingerprints are not attributed to this frozen binary.']}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--binary-sha256', required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--timeout', type=float, default=25)
    args = parser.parse_args()
    report = args.report.resolve()
    if args.timeout <= 0 or not report.is_relative_to((ROOT/'artifacts').resolve()):
        parser.error('positive timeout and an owned artifacts report path are required')
    if report == args.compiler.resolve():
        parser.error('report cannot replace the compiler')
    try:
        payload = check(args.compiler, args.binary_sha256, args.timeout)
    except (ValueError, KeyError) as error:
        parser.error(str(error))
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(payload, indent=2)+'\n')
    return int(any(proof['status'] != 'passed' for row in payload['results'] for proof in row['stages'].values()))


if __name__ == '__main__':
    raise SystemExit(main())
