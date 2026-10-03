#!/usr/bin/env python3
"""Inventory pinned library sources and check genuine modules with our CLI only."""
from __future__ import annotations

import argparse
from collections import Counter
import json
import os
import platform
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

from check_corpus import ROOT, Source, compiler_fingerprint, digest, inventory, module_search_paths
from library_source_inventory import classify_source, scan as scan_libraries, scan_source

PROFILES = ('diagnostic', 'preload', 'runtime-library')
LOCATION = re.compile(r'^(.+):(\d+):(\d+): error: (.+)$')
# These particular pinned sources implement compatibility answers, not OS effects.
COMPATIBILITY_STUBS = {
    'withlang-dev/open-jai:modules/File/module.jai',
    'withlang-dev/open-jai:modules/Process/module.jai',
    'withlang-dev/open-jai:modules/Thread/module.jai',
}


def role(source: Source, negatives: dict[str, dict]) -> tuple[str, str, bool]:
    """Path conventions select entrypoints; they never establish acceptance."""
    kind, basis, candidate, _, _ = classify_source(source, negatives)
    return kind, basis, candidate


def observations(source: Source) -> dict:
    return scan_source(source.path)


def inventory_row(source: Source, observation: dict) -> dict:
    row = observation.copy()
    if row.get('upstream_expected_failure_evidence') and row['role'] != 'expected-negative':
        row.update(role='expected-negative-fixture-unreviewed',
                   role_basis='explicit upstream expect_compile_failure call; no reviewed diagnostic contract')
    row.update(selected_library_root=row['module_candidate'] and not row['role'].startswith('expected-negative'),
               provenance=('upstream-compatibility-stub' if source.id in COMPATIBILITY_STUBS else
                           'supplied-standard-library' if source.project == 'reference' and
                           source.id.startswith('reference:modules/') else 'pinned-upstream-or-example'))
    return row


def source_inventory(root: Path) -> tuple[list[Source], list[dict], dict, dict]:
    sources = inventory(root)
    metadata = scan_libraries(root)
    scanned = {row['id']: row for row in metadata['sources']}
    rows, cases = [], []
    for source in sources:
        if source.sha256 != source.expected_sha256:
            raise ValueError(f'pinned source missing or changed: {source.id}')
        row = inventory_row(source, scanned[source.id])
        rows.append(row)
        if row['selected_library_root']:
            dependencies = [f"{d['kind']}:{d['literal_operand']}" for d in row['directives']
                            if d['kind'] in {'import', 'load', 'library'} and d['literal_operand'] is not None]
            cases.append({'id': source.id, 'kind': 'support-file' if row['role'] == 'test-fixture' else 'module',
                          'target': 'selected-host-source-branches',
                          'dependencies': list(dict.fromkeys(dependencies))})
    return sources, rows, {'format': 1, 'cases': cases}, metadata


def ordered_module_paths(source: Source) -> list[Path]:
    paths = module_search_paths(source)
    if source.project != 'reference':
        base = ROOT/'corpus/upstream'/source.project.replace('/', '--')
        for parent in source.path.resolve().parents:
            if not parent.is_relative_to(base.resolve()):
                break
            if parent.name == 'my_modules':
                paths.insert(0, parent)
    return list(dict.fromkeys(paths))


def profile_environment(source: Source, profile: str) -> dict[str, str]:
    if profile not in PROFILES:
        raise ValueError(f'unknown checking profile: {profile}')
    configured = {'JAI_RS_MODULE_PATH': os.pathsep.join(map(str, ordered_module_paths(source))),
                  'JAI_RS_PRELOAD': 'off', 'JAI_RS_RUNTIME_SUPPORT': 'off'}
    if profile != 'diagnostic':
    if profile == 'runtime-library':
        configured.update(JAI_RS_RUNTIME_SUPPORT='search', JAI_RS_RUNTIME_ENTRY='0',
                          JAI_RS_RUNTIME_INITIALIZATION='1', JAI_RS_RUNTIME_BACKTRACE='0')
    return configured


def controlled_environment(configured: dict[str, str]) -> dict[str, str]:
    environment = os.environ.copy()
    # Clear every compiler override, including target/backend and workspace policy.
    for key in list(environment):
        if key.startswith('JAI_RS_'):
            del environment[key]
    environment.update(configured)
    return environment


def bootstrap_search_sources(configured: dict[str, str]) -> dict:
    """Record the configured ordered search prediction, without claiming a graph."""
    directories = list(dict.fromkeys(configured['JAI_RS_MODULE_PATH'].split(os.pathsep)))
    standard = configured.get('JAI_RS_STDLIB')
    if standard and standard not in directories:
        directories.append(standard)
    result = {}
    for key, module in (('JAI_RS_PRELOAD', 'Preload'), ('JAI_RS_RUNTIME_SUPPORT', 'Runtime_Support')):
        if configured[key] == 'off':
            result[module] = None
            continue
        candidates = [Path(directory)/name for directory in directories if directory
                      for name in (f'{module}.jai', f'{module}/module.jai')]
        selected = next((path.resolve() for path in candidates if path.is_file()), None)
        result[module] = {'predicted_source': str(selected) if selected else None,
                          'sha256': digest(selected) if selected else None}
    return result


def freeze_compiler(compiler: Path) -> Path:
    """Snapshot only our already-built CLI, retaining content-addressed identity."""
    compiler = compiler.resolve()
    if not compiler.is_relative_to((ROOT/'target').resolve()) or compiler.name != 'jai-rs' or not compiler.is_file():
        raise ValueError('compiler must be our built target/.../jai-rs')
    if 'frontend-adapter' in str(compiler):
        raise ValueError('syntax-only frontend adapter cannot establish library checks')
    expected = digest(compiler)
    snapshot = ROOT/'target/standard-library-snapshots'/expected/'jai-rs'
    snapshot.parent.mkdir(parents=True, exist_ok=True)
    if snapshot.exists():
        if digest(snapshot) != expected:
            raise ValueError('existing compiler snapshot fingerprint differs')
        return snapshot.resolve()
    with tempfile.NamedTemporaryFile(dir=snapshot.parent, delete=False) as temporary:
        staged = Path(temporary.name)
    try:
        shutil.copyfile(compiler, staged)
        if digest(staged) != expected:
            raise ValueError('compiler changed while snapshotting; rebuild and retry')
        staged.chmod(0o755)
        try:
            os.link(staged, snapshot)
        except FileExistsError:
            if digest(snapshot) != expected:
                raise ValueError('concurrent compiler snapshot fingerprint differs')
    finally:
        staged.unlink(missing_ok=True)
    return snapshot.resolve()


def execute(compiler: Path, source: Source, action: str, configured: dict[str, str],
            timeout: float, directory: Path) -> dict:
    if action not in {'parse', 'check-library'}:
        raise ValueError('standard library harness permits source-only commands')
    command = [str(compiler), action, str(source.path)]
    result = {'command': command, 'environment': configured, 'exit_code': None,
              'status': 'failed', 'diagnostic': '', 'reason': '', 'failure_kind': None}
    try:
        process = subprocess.run(command, cwd=directory, env=controlled_environment(configured),
                                 stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=timeout)
        result.update(exit_code=process.returncode, status='passed' if process.returncode == 0 else 'failed')
        # Keep the first exact diagnostic, never source/IR stdout or line excerpts.
        lines = [line for line in process.stderr.decode('utf-8', errors='replace').splitlines()
                 if line.strip()]
        result['diagnostic'] = (lines[0] if lines else '')[:4096]
        if process.returncode < 0:
            result['failure_kind'] = 'compiler-signal'
            result['reason'] = f'compiler terminated by signal {-process.returncode}'
        elif process.returncode == 101:
            panic = next((index for index, line in enumerate(lines) if 'panicked at ' in line), None)
            if panic is not None:
                result['failure_kind'] = 'compiler-panic'
                site = lines[panic].split('panicked at ', 1)[1].removesuffix(':')
                detail = lines[panic+1] if panic+1 < len(lines) else 'no panic detail'
                result['reason'] = f'compiler panic at {site}: {detail}'[:4096]
    except subprocess.TimeoutExpired:
        result['failure_kind'] = 'compiler-timeout'
        result['reason'] = 'compiler timeout'
    except OSError as error:
        result.update(status='blocked', reason=str(error), failure_kind='invocation-error')
    return result


def failure_groups(results: list[dict], sources: list[Source]) -> list[dict]:
    known = {str(source.path.resolve()): source for source in sources}
    groups = {}
    for result in results:
        for stage, evidence in result['stages'].items():
            if evidence['status'] not in {'failed', 'blocked'}:
                continue
            location = LOCATION.match(evidence['diagnostic'])
            diagnostic_source, line, column, verified = None, None, None, False
            failure_kind = evidence.get('failure_kind')
            message = evidence['reason'] if failure_kind else evidence['diagnostic'] or evidence['reason']
            if location:
                path, line, column, message = location.groups()
                line, column = int(line), int(column)
                dependency = known.get(str(Path(path).resolve()))
                if dependency:
                    diagnostic_source = dependency.id
                    verified = dependency.path.is_file() and digest(dependency.path) == dependency.sha256
            key = (result['profile'], stage, failure_kind, message)
            group = groups.setdefault(key, {'profile': result['profile'], 'stage': stage,
                                           'failure_kind': failure_kind, 'diagnostic': message,
                                           'root_count': 0, 'locations': []})
            group['root_count'] += 1
            group['locations'].append({'root': result['id'], 'diagnostic_source': diagnostic_source,
                                       'line': line, 'column': column, 'source_hash_verified': verified})
    ordered = sorted(groups.values(), key=lambda g: (g['profile'], g['stage'], -g['root_count'], g['diagnostic']))
    for group in ordered:
        group['distinct_locations'] = len({(p['diagnostic_source'], p['line'], p['column'])
                                           for p in group['locations'] if p['source_hash_verified']})
        group['unverified_root_locations'] = sum(not p['source_hash_verified'] for p in group['locations'])
    return ordered


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, default=ROOT/'target/debug/jai-rs')
    parser.add_argument('--manifest', type=Path, default=ROOT/'corpus/standard-library-acceptance.json')
    parser.add_argument('--report', type=Path, default=ROOT/'artifacts/standard-library-acceptance.json')
    parser.add_argument('--inventory-only', action='store_true')
    parser.add_argument('--profile', action='append', choices=PROFILES)
    parser.add_argument('--select', action='append', default=[])
    parser.add_argument('--timeout', type=float, default=10)
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error('timeout must be positive')
    for destination in (args.manifest.resolve(), args.report.resolve()):
        if any(destination.is_relative_to(path.resolve()) for path in (ROOT/'reference', ROOT/'corpus/upstream')):
            parser.error('outputs must be outside the source corpus')
    try:
        sources, rows, manifest, metadata = source_inventory(ROOT)
    except ValueError as error:
        parser.error(str(error))
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    args.manifest.write_text(json.dumps(manifest, indent=2)+'\n')
    library_ids = {case['id'] for case in manifest['cases']}
    selected = set(args.select) if args.select else library_ids
    if selected - library_ids:
        parser.error('selection must identify inventoried genuine library entrypoints')
    profiles = list(dict.fromkeys(args.profile or ('preload', 'runtime-library')))
    rows_by_id = {row['id']: row for row in rows}
    cases_by_id = {case['id']: case for case in manifest['cases']}
    payload = {'format': 1, 'method': 'pinned-source-library-checks', 'inventory': rows,
               'host': {'system': platform.system(), 'machine': platform.machine()},
               'role_counts': dict(Counter(row['role'] for row in rows)), 'selected': len(selected),
               'manifest_sha256': digest(args.manifest), 'profiles': profiles,
               'static_summary': metadata['summary'], 'negative_references': metadata['negative_references'],
               'core_original_entrypoints': metadata['core_original_entrypoints'],
               'scanner_sha256': digest(ROOT/'tools/library_source_inventory.py'),
               'runner_sha256': digest(Path(__file__)),
               'input_manifests': {name: digest(ROOT/name) for name in ('corpus/reference-inputs.json', 'corpus/upstreams.json')},
               'results': [], 'failure_groups': [], 'totals': {}, 'totals_by_role': {},
               'compiler_failures': {}, 'compiler': None,
               'limitations': ['Inventory observations are lexical, not parsed dependencies or target support.',
                               'Nested comments, quoted examples, and custom here-string bodies are excluded from directive matching.',
                               'Negative expectations come only from reviewed acceptance cases; unsupported inputs are not expected negatives.',
                               'Library checks require no synthetic main, replacement modules, SDK stubs or corpus source modifications.',
                               'No original native library, object, compiler, project script, or upstream program is executed or loaded.',
                               'CLI checking uses its compiler effects policy; checks do not establish runtime or foreign OS behavior.',
                               'Library checking does not enumerate every possible generic specialization or module parameter combination.',
                               'Host target is selected by the immutable CLI; other platform source branches are not separately verified.',
                               'Required module parameters other than automatic Runtime_Support are not synthesized.',
                               'Compiler binary and observed current Rust-source fingerprints are independent; no build provenance is inferred.',
                               'Reports remain local and contain no source/IR stdout or source excerpts.']}
    if not args.inventory_only:
        try:
            compiler = freeze_compiler(args.compiler)
        except ValueError as error:
            parser.error(str(error))
        payload['compiler'] = compiler_fingerprint(ROOT, compiler)
        payload['compiler']['evidence_kind'] = 'integrated-cli'
        payload['compiler']['input_binary'] = str(args.compiler.resolve())
        with tempfile.TemporaryDirectory(prefix='jai-standard-library-') as temporary:
            for profile in profiles:
                for source in sources:
                    if source.id not in selected:
                        continue
                    configured = profile_environment(source, profile)
                    stages = {'parse': execute(compiler, source, 'parse', configured, args.timeout, Path(temporary)),
                              'check': {'status': 'not-run', 'reason': 'parse did not pass'}}
                    if stages['parse']['status'] == 'passed':
                        stages['check'] = execute(compiler, source, 'check-library', configured, args.timeout, Path(temporary))
                    payload['results'].append({'id': source.id, 'profile': profile, 'sha256': source.sha256,
                                               'kind': cases_by_id[source.id]['kind'],
                                               'role': rows_by_id[source.id]['role'], 'purpose': rows_by_id[source.id]['purpose'],
                                               'provenance': rows_by_id[source.id]['provenance'],
                                               'configured_bootstrap_search': bootstrap_search_sources(configured),
                                               'stages': stages})
                print(json.dumps({'profile': profile, 'completed': len(payload['results'])}), flush=True)
        payload['compiler']['binary_sha256_after'] = digest(compiler)
        payload['compiler']['binary_unchanged'] = payload['compiler']['binary_sha256_after'] == payload['compiler']['binary_sha256']
        payload['failure_groups'] = failure_groups(payload['results'], sources)
        payload['totals'] = {profile: {stage: dict(Counter(result['stages'][stage]['status']
                                               for result in payload['results'] if result['profile'] == profile))
                                     for stage in ('parse', 'check')} for profile in profiles}
        payload['totals_by_role'] = {profile: {
            role: {stage: dict(Counter(result['stages'][stage]['status'] for result in payload['results']
                                      if result['profile'] == profile and result['role'] == role))
                   for stage in ('parse', 'check')}
            for role in sorted({result['role'] for result in payload['results']})} for profile in profiles}
        payload['compiler_failures'] = {profile: dict(Counter(
            stage['failure_kind'] for result in payload['results'] if result['profile'] == profile
            for stage in result['stages'].values() if stage.get('failure_kind')))
            for profile in profiles}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(payload, indent=2)+'\n')
    print(json.dumps({'selected': len(selected), 'role_counts': payload['role_counts'], 'totals': payload['totals'],
                      'failure_groups': len(payload['failure_groups']), 'report': str(args.report.resolve())}))
    if args.inventory_only:
        return 0
    return int(not payload['compiler']['binary_unchanged'] or any(
        stage['status'] in {'failed', 'blocked'} for result in payload['results'] for stage in result['stages'].values()))


if __name__ == '__main__':
    raise SystemExit(main())
