#!/usr/bin/env python3
"""Render source-free stage tables from measured immutable-compiler reports."""
from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import asdict, dataclass
import hashlib
import json
from pathlib import Path
import re

from check_corpus import ROOT, Stage, Status

SHA256 = re.compile(r'^[0-9a-f]{64}$')
PROFILES = {'off', 'search', 'diagnostic', 'preload', 'runtime-library', 'unspecified'}
KINDS = {'program', 'module', 'support-file', 'metaprogram', 'negative'}
MAX_REPORT_BYTES = 64 * 1024 * 1024


def fingerprint(path: Path) -> str:
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def require_hash(value, field: str) -> str:
    if not isinstance(value, str) or not SHA256.fullmatch(value):
        raise ValueError(f'{field} requires a SHA-256 fingerprint')
    return value


def label(value, field: str) -> str:
    if not isinstance(value, str) or not value or len(value) > 4096 or any(ord(c) < 32 for c in value):
        raise ValueError(f'{field} requires a bounded single-line label')
    return value


@dataclass(frozen=True)
class Compiler:
    binary: str
    binary_sha256: str
    evidence_kind: str
    observed_inputs_sha256: str | None


def frozen_compiler(record: dict, root: Path) -> Compiler:
    expected = require_hash(record.get('binary_sha256'), 'compiler.binary_sha256')
    binary = Path(label(record.get('binary'), 'compiler.binary')).resolve()
    roots = [(root / 'target' / name).resolve()
             for name in ('corpus-snapshots', 'standard-library-snapshots')]
    if not any(binary == directory / expected / 'jai-rs' for directory in roots):
        raise ValueError('compiler must be a content-addressed frozen target snapshot matching its hash')
    if not binary.is_file() or fingerprint(binary) != expected:
        raise ValueError('frozen compiler is missing or its bytes differ from the report')
    if record.get('binary_unchanged') is False or record.get('binary_sha256_after', expected) != expected:
        raise ValueError('report records a compiler that changed during measurement')
    kind = record.get('evidence_kind', 'unspecified')
    if kind not in {'integrated-cli', 'isolated-frontend-adapter', 'unspecified'}:
        raise ValueError('unknown compiler evidence kind')
    observed = record.get('inputs_sha256')
    if observed is not None:
        require_hash(observed, 'compiler.inputs_sha256')
    return Compiler(str(binary), expected, kind, observed)


def stage_evidence(value: dict, stage: Stage) -> dict:
    if not isinstance(value, dict):
        raise ValueError(f'{stage.value} evidence must be an object')
    try:
        status = Status(value.get('status', Status.NOT_RUN.value))
    except ValueError as error:
        raise ValueError(f'unknown {stage.value} outcome') from error
    exit_code = value.get('exit_code')
    if exit_code is not None and type(exit_code) is not int:
        raise ValueError('exit_code must be an integer or null')
    output = value.get('output_sha256')
    if output is not None:
        require_hash(output, 'stage.output_sha256')
    if status == Status.PASSED:
        if exit_code is None or exit_code < 0 or (stage != Stage.RUN and exit_code != 0):
            raise ValueError('successful compiler stage requires exit code zero; runtime success requires a normal exit')
        if stage in (Stage.CODEGEN, Stage.BUILD, Stage.RUN) and output is None:
            raise ValueError('successful output/runtime stage requires its artifact hash')
    if status == Status.NOT_RUN and (exit_code is not None or output is not None):
        raise ValueError('unattempted stage cannot carry execution or artifact evidence')
    if stage == Stage.RUN and status in {Status.EXPECTED_REJECTION, Status.UNEXPECTED_ACCEPTANCE}:
        raise ValueError('runtime behavior cannot establish a compiler rejection contract')
    if status == Status.EXPECTED_REJECTION and exit_code != 1:
        raise ValueError('expected rejection requires ordinary compiler exit code one')
    if status == Status.UNEXPECTED_ACCEPTANCE and exit_code != 0:
        raise ValueError('unexpected acceptance requires compiler exit code zero')
    return {'status': status.value, 'exit_code': exit_code, 'output_sha256': output}


def row_summary(row: dict, report: dict, compiler: Compiler) -> dict:
    if not isinstance(row, dict):
        raise ValueError('acceptance result must be an object')
    identifier = label(row.get('id'), 'result.id')
    kind = row.get('kind', 'support-file')
    if kind not in KINDS:
        raise ValueError('unknown acceptance case kind')
    profile = row.get('profile', report.get('bootstrap', 'unspecified'))
    if profile not in PROFILES:
        raise ValueError('unknown source checking profile')
    raw_stages = row.get('stages')
    if not isinstance(raw_stages, dict) or set(raw_stages) - {stage.value for stage in Stage}:
        raise ValueError('result has invalid stage evidence')
    stages = {stage.value: stage_evidence(raw_stages.get(stage.value, {}), stage) for stage in Stage}
    source_hash = row.get('sha256') or None
    if source_hash is not None:
        require_hash(source_hash, 'result.sha256')
    if any(e['status'] in {'passed', 'expected-rejection', 'unexpected-acceptance'}
           for e in stages.values()) and source_hash is None:
        raise ValueError('measured source outcome requires its input hash')
    if compiler.evidence_kind == 'isolated-frontend-adapter' and any(
            stages[stage.value]['status'] in {'passed', 'expected-rejection', 'unexpected-acceptance'}
            for stage in (Stage.CHECK, Stage.CODEGEN, Stage.BUILD, Stage.RUN)):
        raise ValueError('frontend adapter cannot establish later compiler/runtime stages')
    if stages['run']['status'] == 'passed':
        if stages['build']['status'] != 'passed' or stages['run']['output_sha256'] != stages['build']['output_sha256']:
            raise ValueError('runtime success must use the exact successful build artifact')
    matched = [stage for stage, evidence in stages.items() if evidence['status'] == 'expected-rejection']
    unreviewed = row.get('role') == 'expected-negative-fixture-unreviewed'
    intent = ('upstream-unreviewed' if unreviewed else 'declared-negative' if kind == 'negative' or row.get('role') == 'expected-negative'
              else 'matched-stage-contract' if matched else 'not-declared')
    if unreviewed and matched:
        raise ValueError('unreviewed negative fixture cannot establish an expected rejection')
    return {'id': identifier, 'profile': profile, 'kind': kind, 'sha256': source_hash,
            'negative_intent': intent, 'matched_rejection_stages': matched, 'stages': stages}


def summarize(path: Path, root: Path = ROOT) -> dict:
    with path.open('rb') as stream:
        recorded = stream.read(MAX_REPORT_BYTES + 1)
    if len(recorded) > MAX_REPORT_BYTES:
        raise ValueError('acceptance report exceeds 64 MiB')
    recorded_hash = hashlib.sha256(recorded).hexdigest()
    report = json.loads(recorded)
    if not isinstance(report, dict) or report.get('format') != 1 or not isinstance(report.get('compiler'), dict) or not isinstance(report.get('results'), list):
        raise ValueError('expected measured format-one acceptance report')
    compiler = frozen_compiler(report['compiler'], root)
    rows = [row_summary(row, report, compiler) for row in report['results']]
    identities = [(row['profile'], row['id']) for row in rows]
    if len(identities) != len(set(identities)):
        raise ValueError('duplicate case identity within a checking profile')
    source_hashes = {}
    for row in rows:
        if row['id'] in source_hashes and source_hashes[row['id']] != row['sha256']:
            raise ValueError('checking profiles must measure the same source bytes for each case')
        source_hashes[row['id']] = row['sha256']
    cohorts = []
    for profile in sorted({row['profile'] for row in rows}):
        selected = [row for row in rows if row['profile'] == profile]
        cohorts.append({'profile': profile, 'cases': len(selected),
                        'attempted': sum(any(e['status'] != 'not-run' for e in row['stages'].values()) for row in selected),
                        'totals': {stage.value: dict(Counter(row['stages'][stage.value]['status'] for row in selected)) for stage in Stage}})
    if fingerprint(Path(compiler.binary)) != compiler.binary_sha256:
        raise ValueError('frozen compiler changed while rendering')
    if fingerprint(path) != recorded_hash:
        raise ValueError('acceptance report changed while rendering')
    return {'report': label(path.name, 'report filename'), 'report_sha256': recorded_hash, 'compiler': asdict(compiler),
            'cohorts': cohorts, 'results': rows}


def cell(value: str) -> str:
    return value.replace('\\', '\\\\').replace('|', '\\|').replace('`', '\\`').replace('<', '&lt;').replace('>', '&gt;')


def markdown(summary: dict) -> str:
    lines = ['# Corpus stage evidence', '',
             'Each row reports an observed stage outcome. Missing stages remain unattempted; expected rejection is separate from successful acceptance.', '',
             '| Report | Compiler SHA-256 | Evidence kind | Observed source-input SHA-256 |',
             '| --- | --- | --- | --- |']
    for report in summary['reports']:
        compiler = report['compiler']
        lines.append(f"| {cell(report['report'])} | {compiler['binary_sha256']} | {compiler['evidence_kind']} | {compiler['observed_inputs_sha256'] or 'not recorded'} |")
    lines.extend(['', '| Report / profile | Stage | Passed | Expected rejection | Failed | Unexpected acceptance | Blocked | Not run |',
                  '| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |'])
    for report in summary['reports']:
        for cohort in report['cohorts']:
            name = cell(report['report'] + ' / ' + cohort['profile'])
            for stage in Stage:
                counts = cohort['totals'][stage.value]
                values = [counts.get(status, 0) for status in ('passed', 'expected-rejection', 'failed', 'unexpected-acceptance', 'blocked', 'not-run')]
                lines.append(f"| {name} | {stage.value} | " + ' | '.join(map(str, values)) + ' |')
    lines.extend(['', '| Report / profile | Intended negative fixture | Intent | Matched rejection stages |',
                  '| --- | --- | --- | --- |'])
    negatives = 0
    for report in summary['reports']:
        for row in report['results']:
            if row['negative_intent'] == 'not-declared':
                continue
            negatives += 1
            lines.append(f"| {cell(report['report'] + ' / ' + row['profile'])} | {cell(row['id'])} | {row['negative_intent']} | {', '.join(row['matched_rejection_stages']) or 'none'} |")
    if not negatives:
        lines.append('| — | — | no declared or matched negative fixtures | — |')
    lines.extend(['', 'Binary hashes are checked against existing content-addressed frozen compiler files. Current source-input fingerprints are independent observations and do not establish build provenance.',
                  'This renderer does not run a compiler, generated executable, native tool, or project script. Counts do not establish full project acceptance. Diagnostics, source/IR output, commands and environment values are omitted.', ''])
    return '\n'.join(lines)


def destination(path: Path, reports: list[Path]) -> Path:
    resolved = path.resolve()
    if resolved.suffix not in {'.md', '.json'} or resolved in {report.resolve() for report in reports}:
        raise ValueError('summary output must be a distinct Markdown or JSON file')
    if any(resolved.is_relative_to((ROOT / name).resolve()) for name in ('reference', 'vendor', 'corpus/upstream', '.git')):
        raise ValueError('summary output cannot replace protected input data')
    return resolved


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reports', nargs='+', type=Path)
    parser.add_argument('--json', type=Path)
    parser.add_argument('--markdown', type=Path)
    args = parser.parse_args()
    if not args.json and not args.markdown:
        parser.error('choose --json and/or --markdown output')
    try:
        summaries = [summarize(path) for path in args.reports]
        payload = {'format': 1, 'method': 'source-free-measured-stage-summary', 'reports': summaries}
        outputs = [(destination(path, args.reports), value) for path, value in (
            (args.json, json.dumps(payload, indent=2) + '\n'), (args.markdown, markdown(payload))) if path]
        if len({path for path, _ in outputs}) != len(outputs):
            raise ValueError('JSON and Markdown outputs must be distinct')
        for path, value in outputs:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(value)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.error(str(error))
    print(json.dumps({'reports': len(summaries), 'compiler_hashes': sorted({s['compiler']['binary_sha256'] for s in summaries})}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
