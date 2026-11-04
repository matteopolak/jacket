#!/usr/bin/env python3
"""Build matched Meow binaries, then compare them on a quiet machine.

From the repository root, with a built jaic and a fresh output directory:

    PYTHONPATH=tools python3 tools/meow/benchmark.py prepare --compiler target/debug/jaic --out /tmp/meow-benchmark
    PYTHONPATH=tools python3 tools/meow/benchmark.py run --out /tmp/meow-benchmark --execution-label "native x64 Linux, software AES lowering"

Prepare both binaries before timing. Run measurements on a quiet machine with no
concurrent builds or other benchmarks; see docs/tools/benchmarks.md for general
benchmarking guidance. State the actual host and any emulation in the label.
Default coverage: 16 boundary lengths, offsets 0/1/7/15, two seeds, five modes;
5,000 iterations below 1 KiB, 2,000 at 1 KiB, and four at 1 MiB. `run --full` includes lengths 0-64 plus
127/128/129, 255/256/257, 1024 and 1048576, and every offset 0-15.
Use `--full --timeout 600` to allow the larger matrix more time per binary.
`run --smoke` covers lengths 0/1/15/16/31/32/255/256/257, offsets 0/15,
both seeds and all modes; it is a correctness and runtime check, not a benchmark.
Each run defaults to one validated warmup pair excluded from measured ratios.
Each hash iteration hides the source and seed pointers from optimization. These
barriers preserve bytes but add loop and memory-clobber overhead. Short timings
can be dominated by noise. Disjoint does not mean cache-cold.
Use a fresh output directory and retain it as evidence.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import time

from toolkit.repo import ROOT
BASE = 'd072effb268b18d2b16ebf27d69d36c14859909d'
FINALIZER = Path('internal/hash/meow/finalize.jai')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_digest(path):
    h = hashlib.sha256()
    for f in sorted(path.rglob('*')):
        if f.is_file():
            h.update(str(f.relative_to(path)).encode() + b'\0')
            h.update(bytes.fromhex(digest(f)))
    return h.hexdigest()


def run_checked(args, *, env=None, cwd=ROOT):
    return subprocess.run([str(x) for x in args], cwd=cwd, env=env,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def prepare(args):
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    compiler = args.compiler.resolve(strict=True)
    root_rev = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    baseline = subprocess.check_output(['git', 'show', f'{BASE}:stdlib/{FINALIZER}'], cwd=ROOT)
    metadata = {'baseline_finalizer_revision': BASE, 'candidate_worktree_head': root_rev,
                'compiler': str(compiler), 'compiler_sha256': digest(compiler),
                'harness_sha256': digest(ROOT / 'tools/meow/benchmark.jai'),
                'driver_sha256': digest(Path(__file__)), 'target': args.target,
                'flags': ['-O2'], 'platform': platform.platform(), 'cpu_count': os.cpu_count(),
                'machine': platform.machine(), 'processor': platform.processor(),
                'note': 'Execution architecture and emulation must be independently identified; host machine alone does not establish native execution.'}
    for variant in ('baseline', 'candidate'):
        stdlib = out / variant / 'stdlib'
        shutil.copytree(ROOT / 'stdlib', stdlib, symlinks=True)
        if variant == 'baseline':
            (stdlib / FINALIZER).write_bytes(baseline)
        metadata[variant + '_stdlib_sha256'] = tree_digest(stdlib)
        metadata[variant + '_finalizer_sha256'] = digest(stdlib / FINALIZER)
        cmd = [compiler, ROOT / 'tools/meow/benchmark.jai', '-O2', '-exe', out / variant / 'bench']
        if args.target:
            cmd += ['-target', args.target]
        env = os.environ.copy()
        env['JAIC_STDLIB'] = str(stdlib)
        result = run_checked(cmd, env=env)
        (out / variant / 'build.stdout').write_bytes(result.stdout)
        (out / variant / 'build.stderr').write_bytes(result.stderr)
        metadata[variant + '_build'] = {'command': list(map(str, cmd)), 'exit': result.returncode}
        if result.returncode:
            (out / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
            raise SystemExit(f'{variant} build failed; see {out / variant}')
        metadata[variant + '_binary_sha256'] = digest(out / variant / 'bench')
    (out / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')


def parse(raw):
    rows = {}
    for line in raw.decode().splitlines():
        fields = line.split()
        if len(fields) != 8:
            raise ValueError(f'unexpected output: {line!r}')
        key = tuple(map(int, fields[:5]))
        if key in rows:
            raise ValueError(f'duplicate case {key}')
        rows[key] = (float(fields[5]), int(fields[6]), int(fields[7]))
    if not rows:
        raise ValueError('no benchmark cases')
    return rows


def measure(args):
    out = args.out.resolve(strict=True)
    meta = json.loads((out / 'metadata.json').read_text())
    for name in ('baseline', 'candidate'):
        if digest(out / name / 'bench') != meta[name + '_binary_sha256']:
            raise SystemExit(f'{name} binary changed since preparation')
    meta['execution_label'] = args.execution_label
    meta['load_before'] = list(os.getloadavg()) if hasattr(os, 'getloadavg') else None
    meta['coverage'] = 'full' if args.full else 'smoke' if args.smoke else 'representative'
    meta['warmup_pairs'] = args.warmups
    meta['measured_pairs'] = args.pairs
    meta['timeout_seconds'] = args.timeout
    (out / 'metadata.json').write_text(json.dumps(meta, indent=2) + '\n')
    records = []
    ratios = {}
    failed = False
    for pair in range(-args.warmups, args.pairs):
        warmup = pair < 0
        order = ('baseline', 'candidate') if pair % 2 == 0 else ('candidate', 'baseline')
        pair_rows = {}
        for name in order:
            started = time.time_ns()
            stdout_path = out / f'pair-{pair}-{name}.stdout'
            stderr_path = out / f'pair-{pair}-{name}.stderr'
            try:
                with stdout_path.open('wb') as stdout, stderr_path.open('wb') as stderr:
                    result = subprocess.run([str(out / name / 'bench')] + (['--full'] if args.full else ['--smoke'] if args.smoke else []), cwd=ROOT, stdout=stdout, stderr=stderr, timeout=args.timeout)
                returncode = result.returncode
            except subprocess.TimeoutExpired:
                returncode = None
            raw = stdout_path.read_bytes()
            record = {'pair': pair, 'warmup': warmup, 'variant': name, 'started_unix_ns': started,
                      'returncode': returncode, 'stdout_file': str(stdout_path),
                      'stderr_file': str(stderr_path), 'timeout_seconds': args.timeout}
            records.append(record)
            try:
                if returncode != 0:
                    raise ValueError(f'exit {returncode} (None means timeout)')
                pair_rows[name] = parse(raw)
                expected = 11680 if args.full else 180 if args.smoke else 640
                if len(pair_rows[name]) != expected:
                    raise ValueError(f'expected {expected} cases, got {len(pair_rows[name])}')
            except ValueError as exc:
                pair_rows.pop(name, None)
                record['error'] = str(exc)
                failed = True
            (out / 'raw.json').write_text(json.dumps(records, indent=2) + '\n')
        if len(pair_rows) != 2 or pair_rows['baseline'].keys() != pair_rows['candidate'].keys():
            failed = True
            break
        for key, (base, low, high) in pair_rows['baseline'].items():
            cand, clow, chigh = pair_rows['candidate'][key]
            if (low, high) != (clow, chigh) or base <= 0 or cand <= 0:
                failed = True
                records[-1].setdefault('mismatches', []).append({'case': key, 'baseline': [low, high], 'candidate': [clow, chigh]})
                continue
            if not warmup:
                ratios.setdefault(key, []).append((cand / base, base / key[4] * 1e9, cand / key[4] * 1e9))
        if failed:
            break
    meta['load_after'] = list(os.getloadavg()) if hasattr(os, 'getloadavg') else None
    (out / 'metadata.json').write_text(json.dumps(meta, indent=2) + '\n')
    (out / 'raw.json').write_text(json.dumps(records, indent=2) + '\n')
    summary = [{'case': key, 'pairs': len(values), 'median_ratio': statistics.median(v[0] for v in values),
                'min_ratio': min(v[0] for v in values), 'max_ratio': max(v[0] for v in values),
                'baseline_median_ns_per_hash': statistics.median(v[1] for v in values),
                'candidate_median_ns_per_hash': statistics.median(v[2] for v in values)} for key, values in sorted(ratios.items())]
    (out / 'summary.json').write_text(json.dumps({'failed': failed, 'cases': summary}, indent=2) + '\n')
    if failed:
        raise SystemExit('Benchmark failures or digest mismatches; inspect raw.json')


parser = argparse.ArgumentParser(description=__doc__)
sub = parser.add_subparsers(dest='action', required=True)
p = sub.add_parser('prepare')
p.add_argument('--compiler', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
p.add_argument('--target', default='')
p.set_defaults(func=prepare)
p = sub.add_parser('run')
p.add_argument('--out', type=Path, required=True)
p.add_argument('--pairs', type=int, default=5)
p.add_argument('--warmups', type=int, default=1, help='Validated warmup pairs excluded from timing summary')
p.add_argument('--timeout', type=float, default=120, help='Maximum seconds per binary run')
coverage = p.add_mutually_exclusive_group()
coverage.add_argument('--full', action='store_true', help='Run the full length and alignment matrix')
coverage.add_argument('--smoke', action='store_true', help='Run a small correctness and runtime check')
p.add_argument('--execution-label', required=True, help='Host and native or emulated execution, including Rosetta when applicable')
p.set_defaults(func=measure)
args = parser.parse_args()
if getattr(args, 'pairs', 1) < 1:
    parser.error('--pairs must be positive')
if getattr(args, 'warmups', 0) < 0:
    parser.error('--warmups cannot be negative')
if getattr(args, 'timeout', 1) <= 0:
    parser.error('--timeout must be positive')
args.func(args)
