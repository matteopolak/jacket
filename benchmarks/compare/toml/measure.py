"""Fixed paired TOML samples, using the repository sandbox, process metrics and statistics.

Build/verify through the shared heavy gate; pilot/run require its exclusive quiet gate:
    python3 measure.py --scratch /tmp/toml --jaic /path/jaic --community-dir /opaque/import/root --community-archive /opaque/source.tar.gz --build
    python3 measure.py --scratch /tmp/toml --rust /path/oracle --rust-receipt /tmp/oracle.json --workloads /tmp/inputs --pilot
    python3 measure.py --scratch /tmp/toml --freeze-plan
    python3 measure.py --scratch /tmp/toml --rust /path/oracle --rust-receipt /tmp/oracle.json --workloads /tmp/inputs --run --batch-plan /tmp/toml/batch-plan.json

Each child warms three operations before its clock. The measured run retains three unscored
warmup batches plus twelve rotated paired rounds, without retakes or outlier removal. UTF-8
validation/decoding is inside parser tasks; input loading and process startup are outside their
internal clocks. Whole-process wall/CPU/RSS are separate. The append-only journal retains every
sample without repeatedly rewriting the full source receipt between measurements.
"""
from __future__ import annotations
import argparse
from dataclasses import dataclass
import ctypes
import math
import time
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sys
import tarfile
import tomllib

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / 'tools'))
sys.path.insert(0, str(Path(__file__).resolve().parents[3] / 'tools/stdlib/toml'))
from bench.measure import run_measured
from bench.sandbox import Sandbox, environment
from bench.stats import Stat
from bench.results import machine_info, checkout_info
from semantic import semantic_equal, settings_projection

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def clock_facts():
    """Record the native Basic clock's OS resolution; first operations may be sub-tick."""
    class Timespec(ctypes.Structure):
        _fields_ = [('seconds', ctypes.c_long), ('nanoseconds', ctypes.c_long)]
    libc = ctypes.CDLL(None)
    libc.clock_getres.argtypes = [ctypes.c_int, ctypes.POINTER(Timespec)]
    libc.clock_getres.restype = ctypes.c_int
    value = Timespec()
    # These are the owned Posix module's CLOCK_MONOTONIC values on supported hosts.
    if libc.clock_getres(6 if sys.platform == 'darwin' else 1, ctypes.byref(value)):
        raise RuntimeError('cannot query native monotonic clock resolution')
    return dict(jai=dict(api='Basic.current_time_monotonic / POSIX CLOCK_MONOTONIC', resolution_ns=value.seconds * 1000000000 + value.nanoseconds), python=dict(api='time.perf_counter_ns', resolution_ns=math.ceil(time.get_clock_info('perf_counter').resolution * 1000000000)), rust=dict(api='std::time::Instant', resolution_ns=None))


def snapshot():
    # Include transitive owned dependencies, not only Toml and formatter entry files.
    files = [*HERE.glob('*.jai'), HERE / 'measure.py', HERE / 'python_batch.py', ROOT / 'tools/stdlib/toml/semantic.py', *sorted((ROOT / 'stdlib').rglob('*.jai')), *sorted((ROOT / 'tools').rglob('*.py'))]
    return {str(path.relative_to(ROOT)): digest(path) for path in files if not path.name.startswith('._')}


def opaque_community(args):
    """Authenticate opaque checkout bytes against the pinned archive; expose no implementation."""
    pin = json.loads((ROOT / 'tools/stdlib/toml/community-pin.json').read_text())
    if digest(args.community_archive) != pin['archive_sha256']:
        raise RuntimeError('community archive checksum differs from pin')
    expected = {}
    with tarfile.open(args.community_archive) as archive:
        for entry in archive.getmembers():
            if entry.isfile():
                relative = Path(*Path(entry.name).parts[1:]).as_posix()
                expected[relative] = hashlib.sha256(archive.extractfile(entry).read()).hexdigest()
    package = args.community_dir.resolve() / 'toml-jai'
    actual = {path.relative_to(package).as_posix(): digest(path) for path in package.rglob('*') if path.is_file() and not path.name.startswith('._')}
    if actual != expected:
        raise RuntimeError('opaque community checkout differs from the pinned archive')
    return hashlib.sha256(json.dumps(expected, sort_keys=True).encode()).hexdigest()


def checked_receipt(path, binary):
    receipt = json.loads(path.read_text())
    if receipt['binary_sha256'] != digest(binary):
        raise RuntimeError('build receipt does not match binary')
    if any(digest(ROOT / name) != value for name, value in receipt['source_sha256'].items()):
        raise RuntimeError('build receipt does not match current owned sources')
    return receipt


def previous_receipt(path):
    """Authenticate the preserved general-library client and its exact owned-source archive."""
    if path is None:
        return None
    path = path.resolve()
    receipt = json.loads(path.read_text())
    binary = path.parent / 'jai-batch'
    formatter = path.parent / 'formatter-candidate'
    if digest(binary) != receipt['binary_sha256']:
        raise RuntimeError('previous client does not match its build receipt')
    if digest(formatter) != receipt['consumers']['candidate']['binary_sha256']:
        raise RuntimeError('previous formatter does not match its build receipt')
    archive = path.parent / 'frozen-owned-sources.tar.gz'
    actual = {}
    with tarfile.open(archive) as tar:
        for entry in tar.getmembers():
            if not entry.isfile() or entry.name in actual:
                raise RuntimeError('previous source archive has non-file or duplicate entries')
            actual[entry.name] = hashlib.sha256(tar.extractfile(entry).read()).hexdigest()
    if actual != receipt['sources']:
        raise RuntimeError('previous source archive does not match its build receipt')
    return dict(build=receipt, receipt_sha256=digest(path), archive_sha256=digest(archive), binary=str(binary), formatter=str(formatter))


def build(args, scratch):
    if not args.jaic or not args.community_dir or not args.community_archive:
        raise RuntimeError('--build requires --jaic, --community-dir and --community-archive')
    sources = snapshot()
    opaque = opaque_community(args)
    sandbox = Sandbox(reads=[ROOT, args.jaic.resolve().parent, args.community_dir.resolve()], writes=[scratch])
    env = environment(scratch)
    env['JAIC_STDLIB'] = str(ROOT / 'stdlib')
    binary = scratch / 'jai-batch'
    command = [str(args.jaic.resolve()), str(HERE / 'batch.jai'), '-import_dir', str(args.community_dir.resolve()), '-O2', '-no_debug_info', '-exe', str(binary), '-output_path', str(scratch / 'build')]
    result = subprocess.run(sandbox.wrap(command), env=env, capture_output=True, timeout=300)
    # Third-party compiler diagnostics may contain source excerpts. Preserve only opaque files.
    (scratch / 'compile.stdout.opaque').write_bytes(result.stdout)
    (scratch / 'compile.stderr.opaque').write_bytes(result.stderr)
    if result.returncode:
        raise RuntimeError(f'opaque benchmark compilation failed: exit {result.returncode}')
    receipt = dict(community_float_format=dict(mode='SCIENTIFIC', trailing_width=16, zero_removal='NO'), sources=sources, compiler_sha256=digest(args.jaic), binary_sha256=digest(binary), command=command, community_tree_sha256=opaque, consumers={})
    if args.baseline_stdlib:
        for name, stdlib in [('baseline', args.baseline_stdlib.resolve()), ('candidate', ROOT / 'stdlib')]:
            baseline_sources = {str(p.relative_to(stdlib)): digest(p) for p in stdlib.rglob('*.jai') if not p.name.startswith('._')}
            consumer = scratch / ('formatter-' + name)
            command = [str(args.jaic.resolve()), str(HERE / 'formatter_batch.jai'), '-O2', '-no_debug_info', '-exe', str(consumer), '-output_path', str(scratch / ('consumer-build-' + name))]
            consumer_sandbox = Sandbox(reads=[ROOT, stdlib, args.jaic.resolve().parent], writes=[scratch])
            subprocess.run(consumer_sandbox.wrap(command), env=dict(env, JAIC_STDLIB=str(stdlib)), check=True)
            if baseline_sources != {str(p.relative_to(stdlib)): digest(p) for p in stdlib.rglob('*.jai') if not p.name.startswith('._')}:
                raise RuntimeError('consumer dependencies changed during compilation')
            source_file = scratch / (name + '-stdlib-sources.json')
            source_file.write_text(json.dumps(baseline_sources, sort_keys=True) + '\n')
            receipt['consumers'][name] = dict(binary_sha256=digest(consumer), command=command, stdlib_source_receipt_sha256=digest(source_file))
    if sources != snapshot() or opaque != opaque_community(args):
        raise RuntimeError('benchmark inputs changed during compilation')
    (scratch / 'build.json').write_text(json.dumps(receipt, indent=2) + '\n')


def probe(sandbox, env, command, stdin, kind, expected, rust=False):
    """Check the actual executable's output, distinguishing types, signs and civil offsets."""
    row = dict(command=command, kind=kind)
    try:
        run = subprocess.run(sandbox.wrap(command), input=stdin, capture_output=True, env=env, timeout=120)
        row.update(exit=run.returncode, output_bytes=len(run.stdout), output_checksum=sum(run.stdout))
        if run.returncode:
            row.update(pass_check=False, reason=f'verification exited {run.returncode}')
            return row, None
        if kind == 'reject':
            value = json.loads(run.stdout)['rejected']
        elif kind == 'settings':
            value = settings_projection(json.loads(run.stdout), rust=rust)
            row['projection'] = value
        else:
            value = tomllib.loads(run.stdout.decode())
        row['pass_check'] = True if expected is None and kind == 'settings' else semantic_equal(value, expected)
        if not row['pass_check']:
            row['reason'] = 'accepted invalid input' if kind == 'reject' else 'semantic output mismatch'
        return row, value
    except subprocess.TimeoutExpired:
        row.update(pass_check=False, reason='untimed verification exceeded 120 seconds')
    except (ValueError, KeyError, TypeError) as error:
        row.update(pass_check=False, reason='invalid verification output: ' + type(error).__name__)
    return row, None


@dataclass
class Job:
    name: str
    command: list[str]
    stdin: bytes | None
    signature: tuple[int, int] | None = None
    checksum: int | None = None

    @property
    def iterations_index(self):
        return 2 if self.name.startswith('formatter-') else 3

    @property
    def family(self):
        return 'formatter' if self.name.startswith('formatter-') else self.name.split('-', 1)[1]


def validate_manifest(inputs, plan):
    """Require unique workload identities and the exact frozen inventory before any run."""
    names = [path.name for path in inputs]
    if len(names) != len(set(names)):
        raise RuntimeError('duplicate workload basenames across input directories')
    current = {path.name: digest(path) for path in inputs}
    if plan is not None and current != plan['input_sha256']:
        raise RuntimeError('workload manifest differs from the complete frozen pilot inventory')


def apply_metrics(row, run, job, count):
    """Turn every child result into a journalable success or failure row."""
    if not run.ok:
        row['failure'] = dict(exit=run.returncode, timeout=run.timed_out)
        return
    try:
        metrics = json.loads(run.stdout)
        # Consumer tasks resolve settings and do not serialize an output buffer.
        if job.name.startswith('formatter-'):
            metrics['output_bytes'] = 0
        row.update(batch=metrics, process=dict(wall=run.wall, cpu=run.cpu, rss_mib=run.rss_mib, returncode=run.returncode))
        if any(type(metrics[name]) is not int for name in ('iterations', 'nanoseconds', 'output_bytes', 'checksum')) or metrics['iterations'] != count or (metrics['nanoseconds'] < 0 or metrics['nanoseconds'] == 0 and row.get('phase') != 'cold') or metrics['output_bytes'] < 0 or metrics['checksum'] < 0:
            row['failure'] = 'invalid batch metadata'
        elif job.signature and (metrics['output_bytes'], metrics['checksum']) != (job.signature[0] * count, job.signature[1] * count):
            row['failure'] = 'serialized byte consumption changed from verified output'
        elif job.checksum is not None and metrics['checksum'] != job.checksum * count:
            row['failure'] = 'lookup/rejection/consumer checksum changed from verified output'
        else:
            row['ns_per_iteration'] = metrics['nanoseconds'] / count
            if metrics['nanoseconds'] == 0:
                row['below_clock_resolution'] = True
    except (ValueError, KeyError, TypeError, OverflowError) as error:
        row['failure'] = 'malformed child metrics: ' + type(error).__name__


def freeze(scratch):
    pilot_file = scratch / 'pilot.json'
    pilot = json.loads(pilot_file.read_text())
    groups = {}
    for row in pilot['samples']:
        family = 'formatter' if row['task'].startswith('formatter-') else row['task'].split('-', 1)[1]
        groups.setdefault(row['input'] + '/' + family, []).append(row['ns_per_iteration'])
    plan = dict(input_sha256={row['input']: row['input_sha256'] for row in pilot['samples']}, pilot_sha256=digest(pilot_file), build=pilot['build'], rust_binary_sha256=pilot['rust_binary_sha256'], python=pilot['python'], previous=pilot.get('previous'), policy='50ms for fastest pilot implementation in task family, bounded1..1000000', iterations={key: max(1, min(1000000, int(50000000 / min(values)) + 1)) for key, values in groups.items()})
    (scratch / 'batch-plan.json').write_text(json.dumps(plan, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('jaic', 'community-dir', 'community-archive', 'baseline-stdlib', 'rust', 'rust-receipt', 'workloads', 'invalid-workloads', 'batch-plan', 'previous-build'):
        parser.add_argument('--' + name, type=Path)
    parser.add_argument('--scratch', type=Path, required=True)
    modes = parser.add_mutually_exclusive_group(required=True)
    for mode in ('build', 'verify-only', 'pilot', 'run', 'freeze-plan'):
        modes.add_argument('--' + mode, action='store_true')
    args = parser.parse_args()
    if sys.platform not in ('darwin', 'linux'):
        parser.error('community subprocess benchmarks require the macOS/Linux bench Sandbox launcher')
    scratch = args.scratch.resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    if args.freeze_plan:
        freeze(scratch)
        return
    if args.build:
        build(args, scratch)
        return
    if not args.rust or not args.rust_receipt or not args.workloads:
        parser.error('verification/timing requires --rust, --rust-receipt and --workloads')
    rust_receipt = checked_receipt(args.rust_receipt, args.rust)
    receipt = json.loads((scratch / 'build.json').read_text())
    binary = scratch / 'jai-batch'
    if receipt['binary_sha256'] != digest(binary) or receipt['sources'] != snapshot():
        raise RuntimeError('stale benchmark binary/source snapshot')
    previous = previous_receipt(args.previous_build)
    plan = None
    if args.run:
        if not args.batch_plan:
            parser.error('--run requires --batch-plan')
        plan = json.loads(args.batch_plan.read_text())
        if plan['pilot_sha256'] != digest(scratch / 'pilot.json') or plan['build'] != receipt or plan['rust_binary_sha256'] != digest(args.rust) or plan['python'] != platform.python_version() or plan.get('previous') != previous:
            raise RuntimeError('frozen pilot plan does not match executables/runtime')
    inputs = sorted(args.workloads.resolve().glob('*.toml'))
    if args.invalid_workloads:
        inputs += sorted(args.invalid_workloads.resolve().glob('*.toml'))
    if not inputs:
        raise RuntimeError('no workloads')
    validate_manifest(inputs, plan)
    sandbox = Sandbox(reads=[ROOT, scratch, args.rust.resolve().parent, *([Path(previous['binary']).parent] if previous else []), args.workloads.resolve(), *([args.invalid_workloads.resolve()] if args.invalid_workloads else []), Path(sys.executable).resolve().parent], writes=[scratch])
    env = environment(scratch)
    mode = 'verification' if args.verify_only else 'pilot' if args.pilot else 'results'
    output = scratch / (mode + '.json')
    journal = scratch / (mode + '.jsonl')
    if output.exists() or journal.exists():
        raise RuntimeError('report already exists; retain it and use a fresh scratch/report set')
    report = dict(previous=previous, clocks=clock_facts(), machine=machine_info(), checkout=checkout_info(), build=receipt, rust_build=rust_receipt, community_pin=json.loads((ROOT / 'tools/stdlib/toml/community-pin.json').read_text()), python=platform.python_version(), rust_binary_sha256=digest(args.rust), batch_plan_sha256=digest(args.batch_plan) if plan else None, task='preloaded UTF8 bytes; general materialization; Jai common pool; ours calls free_toml before reset, community reset only; Rust/Python normal owned teardown', samples=[], cold_samples=[], validation=[], unavailable=[], summaries={}, cold_summaries={})
    for path in inputs:
        data = path.read_bytes()
        input_hash = hashlib.sha256(data).hexdigest()
        if plan and plan['input_sha256'][path.name] != input_hash:
            raise RuntimeError('workload changed after pilot freeze')
        invalid = bool(args.invalid_workloads and path.parent == args.invalid_workloads.resolve())
        if invalid:
            try:
                tomllib.loads(data.decode())
            except tomllib.TOMLDecodeError:
                expected = True
            else:
                raise RuntimeError('invalid workload is valid TOML')
        else:
            expected = tomllib.loads(data.decode())
        jobs = []
        for library in (['previous'] if previous else []) + ['own', 'community', 'rust']:
            prefix = [str(args.rust.resolve()), 'batch-'] if library == 'rust' else [str(binary), library + '-']
            if library == 'previous':
                prefix = [previous['binary'], 'own-']
            command = [prefix[0], prefix[1] + ('verify-error' if invalid else 'verify'), str(path), '1']
            row, _ = probe(sandbox, env, command, None, 'reject' if invalid else 'toml', expected)
            row.update(input=path.name, library=library)
            report['validation'].append(row)
            if not row['pass_check']:
                report['unavailable'].append(dict(input=path.name, library=library, tasks='reject' if invalid else 'read/read-write/write/lookup', reason=row['reason']))
                if library in ('own', 'previous'):
                    output.write_text(json.dumps(report, indent=2) + '\n')
                    raise RuntimeError('own benchmark verification failed')
                continue
            if not invalid:
                cold, _ = probe(sandbox, env, [*command, 'cold'], None, 'toml', expected)
                cold.update(input=path.name, library=library, cold=True)
                report['validation'].append(cold)
                if not cold['pass_check']:
                    output.write_text(json.dumps(report, indent=2) + '\n')
                    raise RuntimeError('actual cold executable verification failed')
            def command_for(task):
                return [prefix[0], prefix[1] + task, str(path), '1']
            if invalid:
                jobs.append(Job(library + '-reject', command_for('reject'), None, checksum=1))
                continue
            jobs.append(Job(library + '-read', command_for('read'), None))
            jobs.append(Job(library + '-read-write', command_for('read-write'), None, signature=(row['output_bytes'], row['output_checksum'])))
            prepared, _ = probe(sandbox, env, command_for('write-verify'), None, 'toml', expected)
            prepared.update(input=path.name, library=library, prepared=True)
            report['validation'].append(prepared)
            cold_prepared, _ = probe(sandbox, env, [*command_for('write-verify'), 'cold'], None, 'toml', expected)
            cold_prepared.update(input=path.name, library=library, prepared=True, cold=True)
            report['validation'].append(cold_prepared)
            if not cold_prepared['pass_check']:
                output.write_text(json.dumps(report, indent=2) + '\n')
                raise RuntimeError('actual cold prepared writer verification failed')
            if prepared['pass_check']:
                jobs.append(Job(library + '-write', command_for('write'), None, signature=(prepared['output_bytes'], prepared['output_checksum'])))
            else:
                report['unavailable'].append(dict(input=path.name, library=library, tasks='prebuilt write', reason=prepared['reason']))
                if library in ('own', 'previous'):
                    raise RuntimeError('own prepared writer verification failed')
            if library != 'community':
                jobs.append(Job(library + '-lookup', command_for('lookup'), None, checksum=len(expected)))
        jobs.append(Job('python-reject' if invalid else 'python-read', [sys.executable, str(HERE / 'python_batch.py'), str(path), '1', *(['reject'] if invalid else [])], None))
        if not invalid and 'config' in path.stem and receipt['consumers']:
            oracle, expected_settings = probe(sandbox, env, [str(args.rust.resolve()), 'settings'], data, 'settings', None, rust=True)
            oracle.update(input=path.name, library='rust-settings')
            report['validation'].append(oracle)
            if not oracle['pass_check']:
                raise RuntimeError('Rust consumer oracle rejected shared valid configuration')
            checksum = expected_settings['resolved']['indent_width'] + expected_settings['resolved']['max_width'] + len(expected_settings['ignore']) + len(expected_settings['overrides']) + len(expected_settings['unknown'])
            consumers = [(name, consumer, scratch / ('formatter-' + name)) for name, consumer in receipt['consumers'].items()]
            if previous:
                consumers.append(('previous', previous['build']['consumers']['candidate'], Path(previous['formatter'])))
            for name, consumer, executable in consumers:
                if digest(executable) != consumer['binary_sha256']:
                    raise RuntimeError('stale actual consumer binary')
                row, _ = probe(sandbox, env, [str(executable), str(path), '1', 'verify'], None, 'settings', expected_settings)
                row.update(input=path.name, library='formatter-' + name)
                report['validation'].append(row)
                if not row['pass_check']:
                    output.write_text(json.dumps(report, indent=2) + '\n')
                    raise RuntimeError('actual consumer full schema projection differs from oracle')
                jobs.append(Job('formatter-' + name, [str(executable), str(path), '1'], None, checksum=checksum))
        output.write_text(json.dumps(report, indent=2) + '\n')
        if args.verify_only:
            print(path.name + ': all available actual executable checks retained', flush=True)
            continue
        for phase in (['warm'] if args.pilot else ['warm', 'cold']):
            rounds = [-1] if args.pilot else range(-3, 12) if phase == 'warm' else range(12)
            collection = report['samples'] if phase == 'warm' else report['cold_samples']
            for round_number in rounds:
                offset = round_number % len(jobs)
                ordered = jobs[offset:] + jobs[:offset]
                if round_number % 2:
                    ordered = list(reversed(ordered))
                for job in ordered:
                    count = 1 if phase == 'cold' else plan['iterations'][path.name + '/' + job.family] if plan else max(1, min(20000, (8 << 20) // len(data)))
                    command = list(job.command)
                    command[job.iterations_index] = str(count)
                    if phase == 'cold':
                        command.append('cold')
                    run = run_measured(sandbox.wrap(command), env=env, stdin_bytes=job.stdin, timeout=300)
                    row = dict(input=path.name, input_sha256=input_hash, bytes=len(data), task=job.name, phase=phase, round=round_number, warmup=round_number < 0)
                    apply_metrics(row, run, job, count)
                    collection.append(row)
                    with journal.open('a') as stream:
                        stream.write(json.dumps(row) + '\n')
                    if 'failure' in row:
                        output.write_text(json.dumps(report, indent=2) + '\n')
                        raise RuntimeError(f'{job.name}/{path.name}/{phase} failed; raw report retained')
            if not args.pilot:
                for job in jobs:
                    rows = [row for row in collection if row['input'] == path.name and row['task'] == job.name and not row['warmup']]
                    if phase == 'warm':
                        report['summaries'][path.name + '/' + job.name] = Stat.of([row['ns_per_iteration'] for row in rows]).to_json()
                    else:
                        report['cold_summaries'][path.name + '/' + job.name] = dict(below_clock_resolution_samples=sum(bool(row.get('below_clock_resolution')) for row in rows), operation_ns=Stat.of([row['ns_per_iteration'] for row in rows]).to_json(), process_wall_s=Stat.of([row['process']['wall'] for row in rows]).to_json(), process_cpu_s=Stat.of([row['process']['cpu'] for row in rows]).to_json(), rss_mib=Stat.of([row['process']['rss_mib'] for row in rows]).to_json())
            output.write_text(json.dumps(report, indent=2) + '\n')
        print(path.name + ': all fixed samples retained', flush=True)


if __name__ == '__main__':
    main()
