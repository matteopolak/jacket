#!/usr/bin/env python3
"""Run bounded AddressSanitizer cargo-fuzz campaigns with locked dependencies."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import shutil
import subprocess
import sys
import tempfile
import time
import benchmark
import cargo_build_paths

ROOT = Path(__file__).resolve().parents[1]
TARGETS = ('lexer_utf8', 'parser', 'module_vfs', 'constant_sema', 'checked_ir_vm')
CARGO_FUZZ_VERSION = '0.13.2'
DISK_FLOOR = 2 * 1024**3
PROTECTED = ('reference', 'corpus', 'vendor', '.git')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def trusted_tool(name):
    path = shutil.which(name)
    if not path:
        raise RuntimeError(f'independently installed {name} is required')
    resolved = Path(path).resolve()
    if any(resolved.is_relative_to((ROOT / root).resolve()) for root in PROTECTED):
        raise RuntimeError(f'{name} resolves into protected original inputs')
    return path


def build_environment(rustc, clang, clangxx, ar):
    environment = dict(os.environ)
    for name in tuple(environment):
        if name.startswith(('CC_', 'CXX_', 'AR_', 'CFLAGS_', 'CXXFLAGS_')) or (name.startswith('CARGO_TARGET_') and name.endswith('_LINKER')):
            environment.pop(name)
    for name in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CFLAGS', 'CXXFLAGS',
                 'LDFLAGS', 'CPATH', 'C_INCLUDE_PATH', 'CPLUS_INCLUDE_PATH',
                 'LIBRARY_PATH', 'LD_LIBRARY_PATH', 'LD_PRELOAD', 'LD_AUDIT',
                 'DYLD_LIBRARY_PATH', 'DYLD_FALLBACK_LIBRARY_PATH', 'DYLD_FRAMEWORK_PATH',
                 'DYLD_FALLBACK_FRAMEWORK_PATH', 'DYLD_INSERT_LIBRARIES', 'DYLD_ROOT_PATH',
                 'DYLD_VERSIONED_LIBRARY_PATH', 'DYLD_VERSIONED_FRAMEWORK_PATH',
                 'CCC_OVERRIDE_OPTIONS', 'AR', 'CXXSTDLIB', 'RUSTC_BOOTSTRAP', 'SDKROOT'):
        environment.pop(name, None)
    environment.update(RUSTC=rustc, CC=clang, CXX=clangxx, AR=ar, RUSTC_WRAPPER='',
                       RUSTC_WORKSPACE_WRAPPER='', CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='1')
    return environment


def require_disk_space(path):
    if shutil.disk_usage(path).free < DISK_FLOOR:
        raise RuntimeError('fuzz build/run refused below the 2 GiB free-space floor')


def configured_target_directory(argument, environment, cargo):
    try:
        return cargo_build_paths.configured_target_directory(argument, environment, [cargo], ROOT).path
    except ValueError as error:
        raise RuntimeError(str(error)) from error


def cargo_shim(directory, real_cargo):
    """cargo-fuzz 0.13.2 lacks --locked; constrain its internal Cargo calls."""
    path = directory / 'cargo'
    path.write_text(f'''#!{sys.executable}
import os, sys
real = {str(real_cargo)!r}
args = sys.argv[1:]
if args and args[0] in ('build', 'check', 'test', 'rustc', 'metadata'):
    args += ['--locked', '--offline']
os.execv(real, [real, *args])
''')
    path.chmod(0o755)
    return path


def fuzz_source_hashes(root):
    fuzz_root = root / 'fuzz'
    return {str(path.relative_to(root)): digest(path)
            for path in sorted(fuzz_root.rglob('*'))
            if path.is_file() and not any(part in {'target', 'corpus', 'artifacts'}
                                          for part in path.relative_to(fuzz_root).parts)}


def completed_runs(output):
    matches = re.findall(r'Done (\d+) runs in', output)
    if not matches or int(matches[-1]) < 1:
        raise RuntimeError('libFuzzer did not report completed execution; build success is insufficient')
    return int(matches[-1])


def run_campaign(command, output, environment, timeout=1800, storage_paths=None):
    """A build timeout must also stop the compiler and fuzzer descendants."""
    process = subprocess.Popen(command, cwd=ROOT, env=environment, stdout=output,
                               stderr=subprocess.STDOUT, start_new_session=True)
    try:
        deadline = time.monotonic() + timeout
        while True:
            for path in storage_paths or (ROOT,):
                require_disk_space(path)
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise subprocess.TimeoutExpired(command, timeout)
            try:
                return process.wait(timeout=min(1, remaining))
            except subprocess.TimeoutExpired:
                continue
    except (subprocess.TimeoutExpired, RuntimeError):
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()
        raise


def save(destination, metadata):
    temporary = destination / 'metadata.tmp'
    temporary.write_text(json.dumps(metadata, indent=2) + '\n')
    temporary.replace(destination / 'metadata.json')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', choices=('all', *TARGETS), default='all')
    parser.add_argument('--seconds', type=int, default=15)
    parser.add_argument('--runs', type=int, default=1000)
    parser.add_argument('--rss-mb', type=int, default=1024)
    parser.add_argument('--target-dir', type=Path,
                        help='Override Cargo target configuration or CARGO_TARGET_DIR')
    args = parser.parse_args()
    if min(args.seconds, args.runs, args.rss_mb) < 1:
        parser.error('seconds, runs and RSS must be positive')
    destination = ROOT / 'artifacts/fuzz' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    destination.mkdir(parents=True)
    metadata = {'valid': False, 'sanitizer': 'address', 'mode': 'bounded-smoke', 'runs': [],
                'limits': {'seconds_per_target': args.seconds, 'runs_per_target': args.runs,
                           'rss_mb': args.rss_mb, 'input_bytes': 131072, 'input_timeout_seconds': 3},
                'error': 'run did not complete'}
    save(destination, metadata)
    try:
        require_disk_space(ROOT)
        cargo, fuzz, rustc, clang, clangxx, ar = map(trusted_tool, ('cargo', 'cargo-fuzz', 'rustc', 'clang', 'clang++', 'ar'))
        environment = build_environment(rustc, clang, clangxx, ar)
        version = subprocess.check_output([fuzz, '--version'], text=True, env=environment).strip()
        if version != f'cargo-fuzz {CARGO_FUZZ_VERSION}':
            raise RuntimeError(f'expected cargo-fuzz {CARGO_FUZZ_VERSION}, got {version}')
        metadata['cargo_fuzz'] = {'version': version, 'path': fuzz, 'sha256': digest(Path(fuzz))}
        metadata['rustc'] = subprocess.check_output([rustc, '-vV'], text=True, env=environment).strip()
        metadata['build_tools'] = {name: {'path': path, 'sha256': digest(Path(path))}
                                   for name, path in (('cargo', cargo), ('rustc', rustc), ('clang', clang), ('clang++', clangxx), ('ar', ar))}
        host = next(line.removeprefix('host: ') for line in metadata['rustc'].splitlines() if line.startswith('host: '))
        environment[f'CARGO_TARGET_{host.upper().replace("-", "_")}_LINKER'] = clang
        target_dir = configured_target_directory(args.target_dir, environment, cargo).resolve()
        target_dir.mkdir(parents=True, exist_ok=True)
        storage_paths = (destination, target_dir)
        for path in storage_paths:
            require_disk_space(path)
        metadata['storage_paths'] = [str(path) for path in storage_paths]
        inputs = benchmark.source_hashes(ROOT)
        inputs.update(fuzz_source_hashes(ROOT))
        for name in ('tools/run_fuzz.py', 'tools/check_cargo_fuzz_age.py'):
            inputs[name] = digest(ROOT / name)
        metadata['input_hashes'] = inputs
        build_source = destination / 'source'
        benchmark.copy_snapshot(ROOT, build_source, inputs)
        for name, expected in inputs.items():
            if digest(ROOT / name) != expected:
                raise RuntimeError('source changed during fuzz snapshot capture')
        lock = build_source / 'fuzz/Cargo.lock'
        expected_lock = digest(lock)
        # Verify every external dependency before a dependency build script runs.
        with (destination / 'dependency-age.txt').open('w') as output:
            subprocess.run([sys.executable, str(build_source / 'tools/check_dependency_age.py'),
                            '--lockfile', str(lock)], cwd=ROOT, stdout=output,
                           stderr=subprocess.STDOUT, check=True, env=environment)
        # Fetch uses --locked; all internal cargo-fuzz builds use the same offline lock.
        with (destination / 'fetch.txt').open('w') as output:
            subprocess.run([cargo, 'fetch', '--locked', '--manifest-path', str(build_source / 'fuzz/Cargo.toml')],
                           cwd=ROOT, stdout=output, stderr=subprocess.STDOUT, check=True, env=environment)
        with tempfile.TemporaryDirectory(prefix='jai-fuzz-locked-cargo-') as directory:
            shim = Path(directory)
            cargo_shim(shim, cargo)
            environment['PATH'] = str(shim) + os.pathsep + environment.get('PATH', '')
            targets = TARGETS if args.target == 'all' else (args.target,)
            for target in targets:
                for path in storage_paths:
                    require_disk_space(path)
                corpus = destination / 'corpus' / target
                shutil.copytree(build_source / 'fuzz/seeds' / target, corpus)
                artifacts = destination / 'crashes' / target
                artifacts.mkdir(parents=True)
                command = [fuzz, 'fuzz', 'run', '--fuzz-dir', str(build_source / 'fuzz'), '--features', 'fuzzing',
                           '--dev', '--sanitizer', 'address', '--target', host, '--target-dir', str(target_dir),
                           target, str(corpus), '--', f'-runs={args.runs}', f'-max_total_time={args.seconds}',
                           '-max_len=131072', '-timeout=3', f'-rss_limit_mb={args.rss_mb}',
                           f'-artifact_prefix={artifacts}/']
                run = {'target': target, 'command': command, 'exit_code': None}
                metadata['runs'].append(run)
                with (destination / f'{target}.txt').open('w') as output:
                    run['exit_code'] = run_campaign(command, output, environment,
                                                    storage_paths=storage_paths)
                if digest(lock) != expected_lock:
                    raise RuntimeError('fuzz lockfile changed during a campaign')
                if run['exit_code']:
                    raise RuntimeError(f'{target} failed; inspect its log and retained own crash input')
                run['completed_runs'] = completed_runs((destination / f'{target}.txt').read_text())
                binary = target_dir / host / 'debug' / target
                if not binary.is_file() or not binary.resolve().is_relative_to(target_dir):
                    raise RuntimeError('own built fuzzer executable is missing or outside its cache')
                retained = destination / 'executables' / target
                retained.parent.mkdir(exist_ok=True)
                shutil.copy2(binary, retained)
                run['executable'] = {'path': str(retained), 'sha256': digest(retained)}
                for name, expected in inputs.items():
                    if digest(build_source / name) != expected:
                        raise RuntimeError('captured fuzz source changed during the campaign')
                for path in storage_paths:
                    require_disk_space(path)
                save(destination, metadata)
        if digest(Path(fuzz)) != metadata['cargo_fuzz']['sha256']:
            raise RuntimeError('cargo-fuzz tool changed during the campaign')
        for tool in metadata['build_tools'].values():
            if digest(Path(tool['path'])) != tool['sha256']:
                raise RuntimeError('a compiler build tool changed during the campaign')
        metadata['valid'] = True
        metadata['live_source_changes'] = [name for name, expected in inputs.items()
                                          if not (ROOT / name).is_file() or digest(ROOT / name) != expected]
        metadata.pop('error', None)
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        metadata['error'] = str(error)
    finally:
        metadata['output_hashes'] = {str(path.relative_to(destination)): digest(path)
                                    for path in sorted(destination.rglob('*'))
                                    if path.is_file() and path.name != 'metadata.json'}
        save(destination, metadata)
        print(destination)
    if not metadata['valid']:
        raise SystemExit(metadata['error'])


if __name__ == '__main__':
    main()
