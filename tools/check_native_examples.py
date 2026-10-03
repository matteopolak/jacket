#!/usr/bin/env python3
"""Check explicitly reviewed original programs using actual Preload and our output."""
from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
import json
from pathlib import Path
import re
import tempfile

from check_corpus import (ROOT, Evidence, Source, Stage, Status, compiler_fingerprint,
                          digest, evaluate, execute, inventory, totals)
from check_feature_matrix import run_native
from inventory_corpus_features import decode, mask_noncode

@dataclass(frozen=True)
class RuntimeExpectation:
    exit_code: int
    stdout: str
    stderr: str

@dataclass(frozen=True)
class NativeCase:
    source: Source
    review: str
    runtime: RuntimeExpectation
    preload: Source


def reviewed_cases(manifest_path: Path) -> tuple[dict, list[NativeCase]]:
    manifest = json.loads(manifest_path.read_text())
    if manifest.get('format') != 1:
        raise ValueError('unsupported reviewed-native manifest')
    sources = {source.id: source for source in inventory(ROOT)}
    preload = sources['reference:modules/module.jai']
    if preload.sha256 != preload.expected_sha256 or preload.sha256 != manifest.get('preload_sha256'):
        raise ValueError('actual reference Preload fingerprint differs from review')
    cases = []
    seen = set()
    for record in manifest['cases']:
        identity = record['id']
        if identity in seen or identity not in sources:
            raise ValueError('duplicate or unpinned native source')
        seen.add(identity)
        source = sources[identity]
        if source.sha256 != source.expected_sha256 or source.sha256 != record.get('sha256'):
            raise ValueError('native source fingerprint differs from review')
        if record.get('host_build_reviewed') is not True or not isinstance(record.get('review'), str) or not record['review'].strip():
            raise ValueError('original native example requires explicit source review')
        code = mask_noncode(decode(source.path.read_bytes()))
        if '#' in code or re.search(r'\bCompiler\b', code):
            raise ValueError('reviewed standalone native examples cannot have directives or Compiler API references')
        runtime = record['runtime']
        if set(runtime) != {'exit_code', 'stdout', 'stderr'} or type(runtime['exit_code']) is not int or not all(isinstance(runtime[k], str) for k in ('stdout', 'stderr')):
            raise ValueError('native example needs exact byte-output and exit assertions')
        cases.append(NativeCase(source, record['review'], RuntimeExpectation(**runtime), preload))
    if not cases:
        raise ValueError('native manifest has no reviewed cases')
    return manifest, cases


def audit_calls(ir: str) -> list[str]:
    definitions = set(re.findall(r'^define\b[^\n@]*@([A-Za-z_.$][A-Za-z0-9_.$]*)\(', ir, re.MULTILINE))
    if 'main' not in definitions:
        raise ValueError('generated LLVM lacks a native main definition')
    calls = []
    for line in ir.splitlines():
        if not re.search(r'\b(?:call|invoke)\b', line):
            continue
        target = re.search(r'@([A-Za-z_.$][A-Za-z0-9_.$]*)\(', line)
        if target is None or (target[1] not in definitions and not target[1].startswith('llvm.')):
            raise ValueError('reviewed native IR contains an indirect or external call')
        calls.append(target[1])
    return calls


def check_case(case: NativeCase, compiler: Path, timeout: float, output: Path):
    config = {'kind': 'program', 'target': 'host', 'dependencies': ['actual unchanged reference Preload'], 'host_build_reviewed': True}
    result = evaluate(case.source, config, compiler, Stage.CODEGEN, timeout, output, bootstrap='search')
    audit = {'review': case.review, 'calls': [], 'status': 'not-run'}
    if result.stages['codegen'].status != Status.PASSED:
        return result, audit
    try:
        audit['calls'] = audit_calls(output.read_text())
        audit['status'] = 'passed'
    except (OSError, ValueError) as error:
        audit['status'] = 'blocked'
        result.stages['build'] = Evidence(Status.BLOCKED, str(error))
        return result, audit
    if digest(case.source.path) != case.source.sha256 or digest(case.preload.path) != case.preload.sha256:
        result.stages['build'] = Evidence(Status.BLOCKED, 'reviewed source or actual Preload changed before build')
        return result, audit
    result.stages['build'] = execute(compiler, case.source, Stage.BUILD, timeout, output,
                                     bootstrap='search', artifact_root=output.parent)
    if result.stages['build'].status == Status.PASSED:
        if digest(case.source.path) != case.source.sha256 or digest(case.preload.path) != case.preload.sha256:
            result.stages['run'] = Evidence(Status.BLOCKED, 'reviewed source or actual Preload changed before runtime')
            return result, audit
        result.stages['run'] = run_native(output, asdict(case.runtime), timeout,
                                           expected_sha256=result.stages['build'].output_sha256)
    return result, audit


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, default=ROOT/'corpus/native-safe-examples.json')
    parser.add_argument('--report', type=Path, default=ROOT/'artifacts/native-corpus-examples.json')
    parser.add_argument('--timeout', type=float, default=10)
    args = parser.parse_args()
    compiler = args.compiler.resolve()
    if not compiler.is_relative_to((ROOT/'target').resolve()) or compiler.name != 'jai-rs' or not compiler.is_file():
        parser.error('compiler must be repository-built target/.../jai-rs')
    output = args.report.resolve()
    if any(output.is_relative_to(p.resolve()) for p in (ROOT/'reference', ROOT/'corpus', ROOT/'vendor')):
        parser.error('report must be outside immutable source/manifest corpus')
    if args.timeout <= 0:
        parser.error('timeout must be positive')
    try:
        manifest, cases = reviewed_cases(args.manifest)
    except (OSError, ValueError, KeyError) as error:
        parser.error(str(error))
    fingerprint = compiler_fingerprint(ROOT, compiler)
    results, audits = [], {}
    with tempfile.TemporaryDirectory(prefix='jai-native-corpus-') as directory:
        for index, case in enumerate(cases):
            result, audit = check_case(case, compiler, args.timeout, Path(directory)/f'{index}.output')
            results.append(result)
            audits[case.source.id] = audit
    if digest(compiler) != fingerprint['binary_sha256']:
        parser.error('compiler changed during reviewed native run')
    payload = {'format': 1, 'compiler': fingerprint, 'manifest_sha256': digest(args.manifest),
               'preload_sha256': manifest['preload_sha256'], 'selected': len(results), 'bootstrap': 'search',
               'totals': totals(results), 'results': [asdict(result) for result in results], 'ir_audits': audits,
               'project_build_successes': 0,
               'limitations': ['Only explicitly reviewed immutable standalone programs are attempted.',
                               'Actual original Preload is selected; Runtime_Support is disabled.',
                               'Only our fresh emitted IR and freshly linked trusted-system-clang output execute.',
                               'Bounded host execution is not VM isolation or full project/platform acceptance.',
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(payload, indent=2)+'\n')
    print(json.dumps({'selected': len(results), 'totals': payload['totals'], 'report': str(output)}))
    return int(any(result.stages['run'].status != Status.PASSED for result in results))

if __name__ == '__main__':
    raise SystemExit(main())
