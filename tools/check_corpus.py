#!/usr/bin/env python3
"""Local staged acceptance using only this repository's Rust compiler."""
from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import asdict, dataclass, field
from enum import Enum
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

class Stage(str, Enum):
    LEX = 'lex'
    PARSE = 'parse'
    CHECK = 'check'
    CODEGEN = 'codegen'
    BUILD = 'build'
    RUN = 'run'

class Status(str, Enum):
    PASSED = 'passed'
    FAILED = 'failed'
    EXPECTED_REJECTION = 'expected-rejection'
    UNEXPECTED_ACCEPTANCE = 'unexpected-acceptance'
    BLOCKED = 'blocked'
    NOT_RUN = 'not-run'

@dataclass
class Evidence:
    status: Status = Status.NOT_RUN
    reason: str = ''
    command: list[str] = field(default_factory=list)
    exit_code: int | None = None
    diagnostic: str = ''
    output_sha256: str | None = None
    environment: dict[str, str] = field(default_factory=dict)

@dataclass
class Source:
    id: str
    path: Path
    sha256: str
    expected_sha256: str
    project: str
    revision: str | None

@dataclass
class Result:
    id: str
    source: str
    sha256: str
    kind: str
    target: str
    dependencies: list[str]
    stages: dict[str, Evidence]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inside(root: Path, relative: str) -> Path:
    p = PurePosixPath(relative)
    if p.is_absolute() or '..' in p.parts or not p.parts or '\\' in relative:
        raise ValueError(f'unsafe source path: {relative}')
    path = root.joinpath(*p.parts).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError(f'source escapes corpus: {relative}')
    return path


def inventory(root: Path) -> list[Source]:
    sources = []
    reference = json.loads((root / 'corpus/reference-inputs.json').read_text())
    upstream = json.loads((root / 'corpus/upstreams.json').read_text())
    if reference['format'] != 1 or upstream['format'] != 1:
        raise ValueError('unsupported input manifest format')
    groups = [('reference', root / 'reference', None, reference['files'])]
    groups += [(p['repository'], root / 'corpus/upstream' / p['repository'].replace('/', '--'),
                p['revision'], p['files']) for p in upstream['projects']]
    for project, base, revision, files in groups:
        for record in files:
            if not record['path'].endswith('.jai'):
                continue
            path = inside(base, record['path'])
            actual = digest(path) if path.is_file() else ''
            sources.append(Source(f"{project}:{record['path']}", path, actual,
                                  record['sha256'], project, revision))
        listed = {r['path'] for r in files if r['path'].endswith('.jai')}
        actual_paths = {p.relative_to(base).as_posix() for p in base.rglob('*.jai')}
        if actual_paths - listed:
            raise ValueError(f'unlisted sources in {project}: {sorted(actual_paths - listed)}')
    ids = [s.id for s in sources]
    if len(set(ids)) != len(ids):
        raise ValueError('duplicate source ids')
    return sources


def validate_cases(manifest: dict) -> dict[str, dict]:
    if manifest.get('format') != 1 or not isinstance(manifest.get('cases'), list):
        raise ValueError('unsupported acceptance manifest format')
    cases = {}
    for case in manifest['cases']:
        if case.get('kind') not in {'program', 'module', 'support-file', 'metaprogram', 'negative'}:
            raise ValueError('invalid acceptance kind')
        if not isinstance(case.get('target'), str) or not isinstance(case.get('dependencies'), list):
            raise ValueError('case requires target and dependencies')
        if not all(isinstance(d, str) for d in case['dependencies']):
            raise ValueError('dependencies must be strings')
        if case.get('id') in cases or not isinstance(case.get('id'), str):
            raise ValueError('duplicate or invalid acceptance case id')
        for stage, expected in case.get('negative', {}).items():
            if stage not in {'lex', 'parse', 'check', 'codegen', 'build'} or not isinstance(expected, str) or not expected.strip():
                raise ValueError('negative requires a nonempty stage diagnostic')
        if case['kind'] == 'negative' and not case.get('negative'):
            raise ValueError('negative case requires expected diagnostic')
        if 'host_build_reviewed' in case and not isinstance(case['host_build_reviewed'], bool):
            raise ValueError('host_build_reviewed must be boolean')
        cases[case['id']] = case
    return cases


def classify(returncode: int, diagnostic: str, expected: str | None) -> Status:
    if expected:
        if returncode == 0:
            return Status.UNEXPECTED_ACCEPTANCE
        # Crashes, signals and unrelated failures are never expected rejections.
        if returncode == 1 and expected in diagnostic:
            return Status.EXPECTED_REJECTION
        return Status.FAILED
    return Status.PASSED if returncode == 0 else Status.FAILED


def module_search_paths(source: Source) -> list[Path]:
    base = ROOT / 'reference' if source.project == 'reference' else ROOT / 'corpus/upstream' / source.project.replace('/', '--')
    candidates = []
    for parent in source.path.resolve().parents:
        if not parent.is_relative_to(base.resolve()):
            break
        actual_names = {child.name for child in parent.iterdir()}
        for name in ('modules', 'Modules'):
            candidate = parent / name
            if name in actual_names and candidate.is_dir():
                candidates.append(candidate.resolve())
    if standard.is_dir():
        candidates.append(standard.resolve())
    return list(dict.fromkeys(candidates))


def fresh_artifact_path(output: Path, source: Source, compiler: Path, artifact_root: Path | None) -> Path:
    """Only remove the exact generated leaf inside the caller's scratch root."""
    root = (artifact_root or output.parent).resolve()
    parent = output.parent.resolve()
    if parent != root or not output.name or output.name in ('.', '..'):
        raise ValueError('artifact must be a direct child of its owned scratch root')
    scratch = Path(tempfile.gettempdir()).resolve()
    target = (ROOT / 'target').resolve()
    if not (root.is_relative_to(scratch) or root.is_relative_to(target)):
        raise ValueError('artifact scratch root must be temporary or repository target storage')
    path = parent / output.name
    aliases_input = path.exists() and any(candidate.exists() and path.samefile(candidate)
                                         for candidate in (source.path, compiler))
    if path.is_symlink() or aliases_input or path.resolve() in (source.path.resolve(), compiler.resolve()):
        raise ValueError('artifact path aliases source/compiler or is a symlink')
    for forbidden in (ROOT / 'reference', ROOT / 'corpus', ROOT / 'vendor'):
        if path.is_relative_to(forbidden.resolve()):
            raise ValueError('artifact cannot replace source corpus')
    if path.exists() and not path.is_file():
        raise ValueError('artifact path must be a regular file')
    path.unlink(missing_ok=True)
    return path


def execute(compiler: Path, source: Source, stage: Stage, timeout: float,
            output: Path, expected: str | None = None, *, library: bool = False, bootstrap: str = 'off', artifact_root: Path | None = None) -> Evidence:
    action = {Stage.LEX: 'lex', Stage.PARSE: 'parse', Stage.CHECK: 'check', Stage.CODEGEN: 'emit-llvm', Stage.BUILD: 'build'}[stage]
    if stage == Stage.CHECK and library:
        action = 'check-library'
    command = [str(compiler), action, str(source.path)]
    if stage in (Stage.CODEGEN, Stage.BUILD):
        command.append(str(output))
    recorded_environment = {'JAI_RS_MODULE_PATH': os.pathsep.join(str(p) for p in module_search_paths(source)),
                            'JAI_RS_PRELOAD': bootstrap, 'JAI_RS_RUNTIME_SUPPORT': 'off'}
    if bootstrap == 'search':
    try:
        if stage in (Stage.CODEGEN, Stage.BUILD):
            output = fresh_artifact_path(output, source, compiler, artifact_root)
            command[-1] = str(output)
        environment = os.environ.copy()
        environment.pop('JAI_RS_STDLIB', None)
        environment.update(recorded_environment)
        if stage == Stage.BUILD:
            # Never inherit a user-supplied backend pointing at corpus tooling.
            environment['JAI_RS_CLANG'] = '/usr/bin/clang'
            recorded_environment['JAI_RS_CLANG'] = '/usr/bin/clang'
        run = subprocess.run(command, capture_output=True, timeout=timeout, cwd=output.parent, env=environment)
        # Reports retain diagnostics only, never compiler stdout containing source/IR.
        diagnostic = run.stderr.decode('utf-8', errors='replace')[:8192]
        status = classify(run.returncode, diagnostic, expected)
        output_hash = digest(output) if stage in (Stage.CODEGEN, Stage.BUILD) and run.returncode == 0 and output.is_file() and not output.is_symlink() else None
        if stage in (Stage.CODEGEN, Stage.BUILD) and status == Status.PASSED and not output_hash:
            return Evidence(Status.FAILED, 'compiler did not produce output', command, run.returncode, diagnostic, environment=recorded_environment)
        if stage == Stage.BUILD and status == Status.PASSED and not os.access(output, os.X_OK):
            return Evidence(Status.FAILED, 'build artifact is not executable', command, run.returncode, diagnostic, output_hash, recorded_environment)
        return Evidence(status, '', command, run.returncode, diagnostic, output_hash, recorded_environment)
    except ValueError as error:
        return Evidence(Status.BLOCKED, str(error), command, environment=recorded_environment)
    except subprocess.TimeoutExpired:
        return Evidence(Status.FAILED, 'compiler timeout', command, environment=recorded_environment)
    except OSError as error:
        return Evidence(Status.BLOCKED, str(error), command, environment=recorded_environment)


def evaluate(source: Source, case: dict, compiler: Path, through: Stage, timeout: float,
             output: Path, *, bootstrap: str = 'off') -> Result:
    stages = {s.value: Evidence() for s in Stage}
    result = Result(source.id, str(source.path), source.sha256, case.get('kind', 'support-file'),
                    case.get('target', 'unspecified'), case.get('dependencies', []), stages)
    if source.sha256 != source.expected_sha256:
        stages['check'] = Evidence(Status.BLOCKED, 'source missing or fingerprint differs from pinned manifest')
        return result
    stages['run'] = Evidence(Status.BLOCKED, 'runtime requires separate isolated execution and behavior assertions')
    for stage in (Stage.LEX, Stage.PARSE, Stage.CHECK, Stage.CODEGEN, Stage.BUILD):
        if list(Stage).index(stage) > list(Stage).index(through):
            break
        if stage == Stage.BUILD and not case.get('host_build_reviewed', False):
            stages['build'] = Evidence(Status.BLOCKED, 'host build not reviewed: SDK, foreign libraries, metaprogram and target requirements')
            break
        expected = case.get('negative', {}).get(stage.value)
        evidence = execute(compiler, source, stage, timeout, output, expected,
                           library=case.get('kind', 'support-file') in ('module', 'support-file'), bootstrap=bootstrap, artifact_root=output.parent)
        stages[stage.value] = evidence
        if evidence.status != Status.PASSED:
            break
    return result


def totals(results: list[Result]) -> dict:
    return {stage.value: dict(Counter(r.stages[stage.value].status.value for r in results)) for stage in Stage}


def compiler_fingerprint(root: Path, compiler: Path) -> dict:
    paths = [root / 'Cargo.lock', root / 'Cargo.toml', root / 'rust-toolchain.toml']
    paths += sorted((root / 'crates').rglob('*.rs'))
    paths += sorted((root / 'crates').rglob('Cargo.toml'))
    records = {str(p.relative_to(root)): digest(p) for p in paths if p.is_file()}
    return {'binary': str(compiler), 'binary_sha256': digest(compiler), 'inputs': records,
            'inputs_sha256': hashlib.sha256(json.dumps(records, sort_keys=True).encode()).hexdigest(),
            'inputs_provenance': 'observed-worktree', 'build_inputs_verified': False}


def annotate_observed_compiler_inputs(report: dict) -> None:
    """Clarify an existing capture without replacing its originally observed hashes."""
    compiler = report['compiler']
    binary = Path(compiler['binary'])
    if not binary.is_file() or digest(binary) != compiler['binary_sha256']:
        raise ValueError('cannot annotate compiler inputs: recorded binary is missing or changed')
    compiler['inputs_provenance'] = 'observed-worktree'
    compiler['build_inputs_verified'] = False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, default=ROOT / 'target/debug/jai-rs')
    parser.add_argument('--frontend-adapter', action='store_true', help='label isolated frontend adapter evidence; only lex/parse allowed')
    parser.add_argument('--manifest', type=Path, default=ROOT / 'corpus/acceptance.json')
    parser.add_argument('--report', type=Path, default=ROOT / 'artifacts/corpus-acceptance.json')
    parser.add_argument('--through', type=Stage, choices=[Stage.LEX, Stage.PARSE, Stage.CHECK, Stage.CODEGEN, Stage.BUILD], default=Stage.CHECK)
    parser.add_argument('--bootstrap', choices=['off', 'search'], default='off', help='off: diagnostic checking without Preload; search: real reference stdlib Preload')
    parser.add_argument('--all', action='store_true', help='attempt every inventoried source, including support files; not project build coverage')
    parser.add_argument('--select', action='append', default=[], help='exact source id; may repeat')
    parser.add_argument('--timeout', type=float, default=10)
    args = parser.parse_args()
    compiler = args.compiler.resolve()
    if args.frontend_adapter and args.through not in (Stage.LEX, Stage.PARSE):
        parser.error('frontend adapter cannot establish check/codegen/build acceptance')
    if not compiler.is_relative_to((ROOT / 'target').resolve()) or compiler.name != 'jai-rs':
        parser.error('compiler must be the repository-built target/.../jai-rs')
    if not compiler.is_file():
        parser.error('build our compiler first: cargo build -p jai-cli --locked')
    if args.timeout <= 0:
        parser.error('timeout must be positive')
    report = args.report.resolve()
    if any(report.is_relative_to(p.resolve()) for p in [ROOT / 'reference', ROOT / 'corpus/upstream']):
        parser.error('report must be outside read-only source corpus')
    manifest = json.loads(args.manifest.read_text())
    try:
        cases = validate_cases(manifest)
    except ValueError as error:
        parser.error(str(error))
    sources = inventory(ROOT)
    known = {s.id for s in sources}
    if (set(cases) | set(args.select)) - known:
        parser.error('acceptance case/selection not in pinned inventory')
    selected = set(args.select) if args.select else (known if args.all else set(cases))
    results = []
    with tempfile.TemporaryDirectory(prefix='jai-corpus-') as directory:
        for index, source in enumerate(sources):
            if source.id in selected:
                results.append(evaluate(source, cases.get(source.id, {}), compiler, args.through,
                                        args.timeout, Path(directory) / f'{index}.output', bootstrap=args.bootstrap))
            else:
                results.append(Result(source.id, str(source.path), source.sha256, cases.get(source.id, {}).get('kind', 'support-file'),
                                      cases.get(source.id, {}).get('target', 'unspecified'), cases.get(source.id, {}).get('dependencies', []),
                                      {s.value: Evidence(Status.NOT_RUN, 'not selected') for s in Stage}))
    fingerprint = compiler_fingerprint(ROOT, compiler)
    fingerprint['evidence_kind'] = 'isolated-frontend-adapter' if args.frontend_adapter else 'integrated-cli'
    if args.frontend_adapter:
        for name in ['artifacts/frontend-adapter/Cargo.toml', 'artifacts/frontend-adapter/Cargo.lock', 'artifacts/frontend-adapter/src/main.rs']:
            path = ROOT / name
            if path.is_file():
                fingerprint['inputs'][name] = digest(path)
        fingerprint['inputs_sha256'] = hashlib.sha256(json.dumps(fingerprint['inputs'], sort_keys=True).encode()).hexdigest()
    payload = {'format': 1, 'compiler': fingerprint,
               'manifest_sha256': digest(args.manifest), 'input_manifests': {p: digest(ROOT / p) for p in ['corpus/reference-inputs.json', 'corpus/upstreams.json']},
               'source_inventory_sha256': hashlib.sha256(json.dumps([(s.id, s.sha256) for s in sources]).encode()).hexdigest(),
               'inventory': len(sources), 'selected': len(selected), 'through': args.through.value, 'bootstrap': args.bootstrap,
               'totals': totals(results), 'results': [asdict(r) for r in results],
               'project_build_successes': 0,
               'limitations': ['support-file checks are not project builds', 'no runtime or upstream build scripts executed',
                               'CLI check uses NoEffects compile-time policy; checking does not establish metaprogram/native effects or runtime behavior', 'target SDK selection is not implemented by CLI', 'manifest host_build_reviewed requires audited pure generated IR and trusted system backend']}
    payload['limitations'].insert(0, 'Bootstrap profile: ' + args.bootstrap + '; off is diagnostic checking only without Preload; search selects actual reference stdlib Preload. Runtime_Support remains disabled and runtime acceptance unverified.')
    if args.frontend_adapter:
        payload['limitations'].insert(0, 'Isolated frontend adapter: syntax evidence only, not integrated CLI acceptance; see compiler evidence_kind and adapter source fingerprints.')
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(payload, indent=2) + '\n')
    print(json.dumps({'inventory': len(sources), 'selected': len(selected), 'totals': payload['totals'], 'report': str(report)}))
    return int(any(e.status in (Status.FAILED, Status.UNEXPECTED_ACCEPTANCE, Status.BLOCKED) for r in results for name, e in r.stages.items() if name != 'run'))

if __name__ == '__main__':
    raise SystemExit(main())
