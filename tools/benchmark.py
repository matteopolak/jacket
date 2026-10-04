#!/usr/bin/env python3
"""Save checked smoke runs and allocation-instrumented benchmark baselines."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tarfile
from cargo_build_paths import configured_target_directory, pinned_cargo_command

ROOT = Path(__file__).resolve().parents[1]
SUITES = ('compiler', 'vm', 'discovery')


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def version(*args: str) -> str:
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def host_hardware() -> dict:
    result = {'logical_cpus': os.cpu_count(), 'processor': platform.processor()}
    if platform.system() == 'Darwin':
        for name in ('hw.model', 'machdep.cpu.brand_string', 'hw.physicalcpu', 'hw.memsize'):
            try:
                result[name] = subprocess.check_output(['sysctl', '-n', name], text=True, stderr=subprocess.DEVNULL).strip()
            except (OSError, subprocess.CalledProcessError):
                result.setdefault('unavailable', []).append(name)
    return result


def source_files(root: Path) -> list[Path]:
    files = [path for path in (root / 'crates').rglob('*')
             if path.is_file() and (path.suffix in {
                 '.rs', '.toml', '.jai', '.json', '.c', '.cc', '.cpp', '.h', '.hh', '.hpp', '.s', '.S', '.ll', '.inc',
             } or path.name.endswith('.jai.pending'))]
    files.extend(path for path in (root / 'prelude').rglob('*.jai') if path.is_file())
    files.extend(root / name for name in (
        'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml',
        'tools/benchmark.py', 'tools/benchmark_resources.py', 'tools/check_dependency_age.py',
        'tools/cargo_build_paths.py',
    ))
    return sorted(files)


def source_hashes(root: Path) -> dict[str, str]:
    return {str(path.relative_to(root)): digest(path) for path in source_files(root)}


def copy_snapshot(root: Path, destination: Path, inputs: dict[str, str]) -> None:
    for name, expected in inputs.items():
        content = (root / name).read_bytes()
        if hashlib.sha256(content).hexdigest() != expected:
            raise RuntimeError(f'source changed while capturing snapshot: {name}')
        path = destination / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        if digest(path) != expected:
            raise RuntimeError(f'snapshot input changed while copying: {name}')


def changed_inputs(before: dict[str, str], after: dict[str, str]) -> list[str]:
    return sorted(name for name in before.keys() | after.keys()
                  if before.get(name) != after.get(name))


def native_artifact_hashes(workloads: list[dict], root: Path) -> dict[str, str]:
    """Only retain own generated outputs inside this run's output directory."""
    result = {}
    permitted = {'own-harness.c', 'own-generated.o', 'own-generated-program'}
    for workload in workloads:
        for name in workload.get('native_artifacts', []):
            path = Path(name).resolve()
            if not path.is_relative_to(root.resolve()) or path.name not in permitted or not path.is_file():
                raise ValueError('native benchmark output is outside its retained directory or missing')
            result[str(path)] = digest(path)
    return result


def native_tool_candidate(environment: dict[str, str], repository: Path) -> Path:
    """Mirror configured path selection only; the benchmark verifies Clang 22."""
    explicit = environment.get('JAI_RS_CLANG')
    prefix = environment.get('LLVM_SYS_221_PREFIX')
    if explicit is not None:
        if not explicit:
            raise ValueError('JAI_RS_CLANG is empty')
        candidate = explicit if '/' in explicit else shutil.which(explicit, path=environment.get('PATH'))
    elif prefix is not None:
        if not prefix:
            raise ValueError('LLVM_SYS_221_PREFIX is empty')
        candidate = str(Path(prefix) / 'bin/clang')
    else:
        candidate = shutil.which('clang-22', path=environment.get('PATH')) or shutil.which('clang', path=environment.get('PATH'))
    if not candidate:
        raise ValueError('independent native benchmark Clang was not found')
    selected = Path(candidate)
    tool = (selected if selected.is_absolute() else repository / selected).resolve(strict=True)
    for protected in ('reference', 'corpus', 'vendor', '.git'):
        if tool.is_relative_to((repository / protected).resolve()):
            raise ValueError('native benchmark tool is inside protected inputs')
    return tool


def save_metadata(destination: Path, metadata: dict) -> None:
    temporary = destination / 'metadata.tmp'
    try:
        temporary.write_text(json.dumps(metadata, indent=2) + '\n')
        temporary.replace(destination / 'metadata.json')
    finally:
        temporary.unlink(missing_ok=True)


def corpus_hashes(root: Path, upstream: bool, reference: bool) -> dict:
    result = {}
    for name, path, enabled in (
        ('reference', root / 'reference', reference),
        ('upstream', root / 'corpus/upstream', upstream),
    ):
        if not enabled:
            continue
        sources = sorted(path.rglob('*.jai'))
        combined = hashlib.sha256()
        size = 0
        for source in sources:
            content = source.read_bytes()
            size += len(content)
            combined.update(str(source.relative_to(path)).encode() + b'\0')
            combined.update(hashlib.sha256(content).digest())
        result[name] = {'files': len(sources), 'bytes': size, 'sha256': combined.hexdigest()}
    return result


def build_command(suites: list[str], offline: bool, manifest: Path | None = None,
                  cargo: list[str] | None = None, target: Path | None = None) -> list[str]:
    command = [*(cargo or ['cargo']), 'bench', '--locked', '-j1', '-p', 'jai-bench',
               '--no-run', '--message-format=json']
    if offline:
        command.append('--offline')
    if manifest is not None:
        command.extend(['--manifest-path', str(manifest)])
    if target is not None:
        command.extend(['--target-dir', str(target)])
    for suite in suites:
        command.extend(['--bench', suite])
    return command


def benchmark_executables(output: str, suites: list[str], target: Path) -> dict[str, Path]:
    executables = {}
    for line in output.splitlines():
        record = json.loads(line)
        if record.get('reason') != 'compiler-artifact':
            continue
        name = record['target']['name']
        if name not in suites or 'bench' not in record['target']['kind'] or not record.get('executable'):
            continue
        path = Path(record['executable']).resolve()
        if not path.is_relative_to(target.resolve()) or not path.is_file():
            raise ValueError(f'benchmark executable outside the build directory or missing: {path}')
        if name in executables and executables[name] != path:
            raise ValueError(f'ambiguous benchmark executable: {name}')
        executables[name] = path
    if set(executables) != set(suites):
        raise ValueError(f'missing benchmark executables: {sorted(set(suites) - executables.keys())}')
    return executables


def measurement_arguments(samples: int, smoke: bool, upstream: bool, reference: bool) -> list[str]:
    arguments = ['--color', 'never']
    if smoke:
        arguments.append('--test')
    else:
        arguments.extend(['--bench', '--sample-count', str(samples), '--sample-size', '1'])
    if upstream:
        arguments.append('--include-ignored')
    if not reference:
        arguments.extend(['--skip', 'reference_lex'])
    return arguments


def measured_rows(output: str, samples: int) -> int:
    """Require real Divan timing/allocation output, not its successful test tree."""
    lines = output.splitlines()
    if not lines or not all(word in lines[0] for word in ('median', 'samples', 'iters')):
        raise ValueError('benchmark output lacks the Divan measurement header')
    time = re.compile(r'\s*(?:\d+(?:\.\d+)?|\.\d+)\s+(?:ps|ns|µs|us|ms|s)\s*')
    rows = 0
    for line in lines[1:]:
        columns = line.split('│')
        if len(columns) < 6 or not columns[-2].strip().isdigit() or not columns[-1].strip().isdigit():
            continue
        columns = columns[-6:]
        if not all(time.fullmatch(column) for column in columns[1:4]):
            raise ValueError('benchmark timing row has missing or invalid time columns')
        if int(columns[-2]) != samples or int(columns[-1]) < samples:
            raise ValueError('benchmark timing row does not match the requested sample count')
        rows += 1
    if not rows or 'alloc:' not in output or 'max alloc:' not in output:
        raise ValueError('benchmark output lacks measured timing rows or Rust allocation statistics')
    return rows


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', type=Path, help='Override CARGO_TARGET_DIR or Cargo target configuration')
    parser.add_argument('--samples', type=int, default=100)
    parser.add_argument('--upstream', action='store_true')
    parser.add_argument('--bench', choices=['all', *SUITES], default='all')
    parser.add_argument('--smoke', action='store_true', help='validate fixtures without timing measurements')
    parser.add_argument('--offline', action='store_true', help='build with Cargo --offline; dependency-age verification still queries crates.io')
    parser.add_argument('--skip-reference', action='store_true', help='exclude the optional local reference lexer corpus')
    parser.add_argument('--timeout-seconds', type=int, default=600, help='maximum runtime of each smoke or measurement suite')
    parser.add_argument('--context-note', help='record relevant measurement conditions, such as concurrent development builds')
    args = parser.parse_args()
    if sys.version_info < (3, 11):
        parser.error('Python 3.11 or newer is required by the dependency-age checker')
    if args.samples < 2:
        parser.error('--samples must be at least 2')
    if args.timeout_seconds < 1:
        parser.error('--timeout-seconds must be positive')
    environment = os.environ.copy()
    environment['RUSTC_WRAPPER'] = ''
    cargo = pinned_cargo_command(ROOT, environment)
    target_selection = configured_target_directory(args.target_dir, environment, cargo, ROOT)
    target_root = target_selection.path
    environment['CARGO_TARGET_DIR'] = str(target_root)
    environment['JAI_BENCH_CORPUS_ROOT'] = str(ROOT)
    environment['JAI_BENCH_TIMEOUT_SECONDS'] = str(args.timeout_seconds)
    suites = list(SUITES) if args.bench == 'all' else [args.bench]
    reference = not args.skip_reference and 'compiler' in suites
    upstream = args.upstream and 'compiler' in suites
    destination = ROOT / 'artifacts/benchmarks' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    destination.mkdir(parents=True)
    environment['JAI_BENCH_NATIVE_ARTIFACTS'] = str(destination / 'native-artifacts')
    build_source = destination / 'source'
    command = build_command(suites, args.offline, build_source / 'Cargo.toml', cargo, target_root)
    llvm_prefix = environment.get('LLVM_SYS_221_PREFIX')
    llvm_config = str(Path(llvm_prefix) / 'bin/llvm-config') if llvm_prefix else 'llvm-config'
    inputs = {}
    corpora = {}
    metadata = {
        'format': 3, 'timestamp': datetime.now(timezone.utc).isoformat(),
        'mode': 'smoke' if args.smoke else 'baseline', 'build_command': command,
        'build_source_directory': str(build_source),
        'target_directory': target_selection.receipt(),
        'benchmark_suites': suites, 'sample_count': None if args.smoke else args.samples,
        'sample_size': None if args.smoke else 1,
        'timeout_seconds': args.timeout_seconds,
        'platform': platform.platform(), 'architecture': platform.machine(),
        'load_average_start': os.getloadavg(),
        'allocation_instrumented': True,
        'allocation_scope': 'Rust allocator only; LLVM native allocations excluded',
        'shared_host': True,
        'context_note': args.context_note,
        'environment': {key: value for key, value in environment.items()
                        if key in {'RUSTC_WRAPPER', 'CARGO_TARGET_DIR', 'JAI_BENCH_CORPUS_ROOT', 'JAI_BENCH_NATIVE_ARTIFACTS', 'JAI_BENCH_TIMEOUT_SECONDS', 'LLVM_SYS_221_PREFIX', 'JAI_RS_CLANG', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS'}
                        or key.startswith('CARGO_PROFILE_BENCH_')},
        'input_hashes': inputs, 'corpora': corpora, 'runs': [], 'valid': False,
        'native_tools': {},
    }
    # Preserve an invalid record even if a later build exhausts disk space.
    try:
        save_metadata(destination, {**metadata, 'error': 'run did not complete'})
    except OSError as error:
        print(destination, flush=True)
        raise SystemExit(f'cannot save initial invalid benchmark metadata: {error}') from error
    exit_code = 1
    try:
        metadata['hardware'] = host_hardware()
        metadata['rustc'] = version('rustc', '-vV')
        metadata['cargo'] = version(*cargo, '-V')
        metadata['llvm'] = {'version': version(llvm_config, '--version'), 'prefix': version(llvm_config, '--prefix')}
        # Freeze the current source before a potentially slow network preflight.
        inputs = source_hashes(ROOT)
        corpora = corpus_hashes(ROOT, upstream, reference)
        metadata['input_hashes'] = inputs
        metadata['corpora'] = corpora
        copy_snapshot(ROOT, build_source, inputs)
        changed = changed_inputs(inputs, source_hashes(ROOT))
        if changed:
            metadata['changed_capture_inputs'] = changed
            raise RuntimeError('source inputs changed while capturing snapshot; retry the run')
        archive_path = destination / 'compiler-source.tar.gz'
        with tarfile.open(archive_path, 'w:gz') as archive:
            for name in inputs:
                archive.add(build_source / name, arcname=name, recursive=False)
        metadata['source_archive_sha256'] = digest(archive_path)
        # Verify the exact lockfile and policy that belong to this snapshot.
        age_command = [sys.executable, str(build_source / 'tools/check_dependency_age.py'),
                       '--lockfile', str(build_source / 'Cargo.lock')]
        age_inputs = {name: digest(build_source / name) for name in ('Cargo.lock', 'tools/check_dependency_age.py')}
        with (destination / 'dependency-age.txt').open('w') as output:
            age = subprocess.run(age_command, cwd=build_source, env=environment, stdout=output, stderr=subprocess.STDOUT)
        metadata['dependency_age'] = {'command': age_command, 'exit_code': age.returncode, 'input_hashes': age_inputs}
        if age.returncode:
            raise RuntimeError('dependency-age verification failed; see dependency-age.txt')
        if any(digest(build_source / name) != expected for name, expected in age_inputs.items()):
            raise RuntimeError('dependency-age inputs changed during verification')
        with (destination / 'build.jsonl').open('w') as output, (destination / 'build.txt').open('w') as errors:
            result = subprocess.run(command, cwd=build_source, env=environment, stdout=output, stderr=errors)
        if result.returncode:
            exit_code = result.returncode
            raise RuntimeError('benchmark build failed; see build.txt and build.jsonl')
        changed = changed_inputs(inputs, source_hashes(build_source))
        if changed:
            metadata['changed_build_inputs'] = changed
            raise RuntimeError('source inputs changed during the build; baseline is not valid')
        built = benchmark_executables((destination / 'build.jsonl').read_text(), suites, target_root)
        (destination / 'executables').mkdir()
        binaries = {}
        metadata['executables'] = {}
        for name, build_path in built.items():
            expected_hash = digest(build_path)
            retained = destination / 'executables' / name
            shutil.copy2(build_path, retained)
            if digest(retained) != expected_hash or digest(build_path) != expected_hash:
                raise RuntimeError(f'benchmark executable changed while retaining it: {name}')
            binaries[name] = retained
            metadata['executables'][name] = {'path': str(retained), 'build_path': str(build_path), 'sha256': expected_hash}
        modes = [True] if args.smoke else [True, False]
        with (destination / 'results.txt').open('w') as combined:
            for smoke in modes:
                for suite in suites:
                    if corpus_hashes(ROOT, upstream, reference) != corpora:
                        raise RuntimeError('source corpus changed during the run; baseline is not valid')
                    path = binaries[suite]
                    expected_hash = metadata['executables'][suite]['sha256']
                    if digest(path) != expected_hash:
                        raise RuntimeError(f'benchmark executable changed: {suite}')
                    if suite == 'compiler':
                        tool = native_tool_candidate(environment, ROOT)
                        expected = metadata['native_tools'].setdefault(str(tool), digest(tool))
                        if digest(tool) != expected:
                            raise RuntimeError('native benchmark tool changed before measurement')
                    label = f'{"smoke" if smoke else "baseline"}-{suite}'
                    benchmark_command = [str(path), *measurement_arguments(args.samples, smoke, upstream, reference)]
                    resource_path = destination / f'{label}-resources.json'
                    run_command = [sys.executable, str(build_source / 'tools/benchmark_resources.py'),
                                   str(resource_path), '--', *benchmark_command]
                    run = {'suite': suite, 'smoke': smoke, 'command': run_command, 'exit_code': None}
                    metadata['runs'].append(run)
                    with (destination / f'{label}.txt').open('w') as output, (destination / f'{label}-stderr.txt').open('w') as errors:
                        try:
                            result = subprocess.run(run_command, cwd=ROOT, env=environment, stdout=output, stderr=errors, timeout=args.timeout_seconds + 10)
                        except subprocess.TimeoutExpired as error:
                            run['exit_code'] = 124
                            run['timed_out'] = True
                            exit_code = 124
                            raise RuntimeError(f'{label} exceeded {args.timeout_seconds} seconds; see its output') from error
                    run['exit_code'] = result.returncode
                    if resource_path.exists():
                        run['process_resources'] = json.loads(resource_path.read_text())
                    run['workloads'] = [json.loads(line.removeprefix('JAI_BENCH_WORKLOAD '))
                                        for line in (destination / f'{label}-stderr.txt').read_text().splitlines()
                                        if line.startswith('JAI_BENCH_WORKLOAD ')]
                    run['native_artifacts'] = native_artifact_hashes(
                        run['workloads'], destination / 'native-artifacts')
                    for workload in run['workloads']:
                        if 'clang' in workload:
                            tool = Path(workload['clang']).resolve()
                            for protected in ('reference', 'corpus', 'vendor', '.git'):
                                if tool.is_relative_to((ROOT / protected).resolve()):
                                    raise ValueError('native benchmark tool is inside protected inputs')
                            if str(tool) not in metadata['native_tools'] or digest(tool) != metadata['native_tools'][str(tool)]:
                                raise RuntimeError('native benchmark tool does not match the captured tool')
                    combined.write(f'=== {label} ===\n')
                    combined.write((destination / f'{label}.txt').read_text())
                    combined.flush()
                    if result.returncode:
                        exit_code = result.returncode
                        raise RuntimeError(f'{label} failed; see its output')
                    if not smoke:
                        run['measured_rows'] = measured_rows((destination / f'{label}.txt').read_text(), args.samples)
                    if digest(path) != expected_hash:
                        raise RuntimeError(f'benchmark executable changed during measurement: {suite}')
        if corpus_hashes(ROOT, upstream, reference) != corpora:
            raise RuntimeError('source corpus changed during the run; baseline is not valid')
        changed = changed_inputs(inputs, source_hashes(build_source))
        if changed:
            metadata['changed_measurement_inputs'] = changed
            raise RuntimeError('isolated source inputs changed during measurement; baseline is not valid')
        if digest(archive_path) != metadata['source_archive_sha256']:
            raise RuntimeError('compiler-source archive changed during measurement; baseline is not valid')
        for run in metadata['runs']:
            for name, expected in run.get('native_artifacts', {}).items():
                if digest(Path(name)) != expected:
                    raise RuntimeError('generated native benchmark artifact changed during measurement')
        for name, expected in metadata['native_tools'].items():
            if digest(Path(name)) != expected:
                raise RuntimeError('native benchmark tool changed during measurement')
        # Live edits cannot change the isolated source or its hashed executables.
        metadata['source_changes_after_build'] = changed_inputs(inputs, source_hashes(ROOT))
        metadata['valid'] = True
        exit_code = 0
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        metadata['error'] = str(error)
    finally:
        metadata['exit_code'] = exit_code
        metadata['finished_at'] = datetime.now(timezone.utc).isoformat()
        metadata['load_average_finish'] = os.getloadavg()
        metadata['output_hashes'] = {path.name: digest(path) for path in sorted(destination.iterdir()) if path.is_file() and path.name != 'metadata.json'}
        try:
            save_metadata(destination, metadata)
        except OSError as error:
            exit_code = 1
            metadata['error'] = f'cannot save final metadata; initial record remains invalid: {error}'
        print(destination, flush=True)
    if exit_code:
        print(metadata.get('error', 'benchmark failed'), file=sys.stderr)
        raise SystemExit(exit_code)


if __name__ == '__main__':
    main()
