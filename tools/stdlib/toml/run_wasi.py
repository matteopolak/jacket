"""Run the complete pinned decoder/encoder inventory under WASI, without exclusions.

    python3 run_wasi.py --jaic /path/jaic --cache /tmp/toml-wasi --report /tmp/wasi.json

The released native Go controller launches the owned WASI adapter through Node. Current
wrappers require a POSIX host; the parser itself is the compiled wasm64 program.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
from run import ROOT, HERE


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--jaic', type=Path, required=True)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--record-statements', action='store_true')
    args = parser.parse_args()
    if sys.platform not in ('darwin', 'linux'):
        parser.error('WASI conformance controller wrappers currently require a POSIX host')
    if args.report.exists():
        raise RuntimeError('report already exists; retain it and choose a fresh report path')
    cache = args.cache.resolve()
    cache.mkdir(parents=True, exist_ok=True)
    source_files = [*sorted((ROOT / 'stdlib').rglob('*.jai')), HERE / 'adapter.jai', HERE / 'wasi.mjs']
    sources = {str(path.relative_to(ROOT)): sha(path) for path in source_files}
    wasm = cache / 'adapter.wasm'
    command = [str(args.jaic.resolve()), str(HERE / 'adapter.jai'), '-os', 'wasm', '-O0', '-exe', str(wasm), '-output_path', str(cache / 'build')]
    subprocess.run(command, cwd=ROOT, env=dict(os.environ, JAIC_STDLIB=str(ROOT / 'stdlib')), check=True)
    if sources != {str(path.relative_to(ROOT)): sha(path) for path in source_files}:
        raise RuntimeError('sources changed during WASI adapter compilation')
    decoder = cache / 'wasi-decoder'
    node = ['node', '--no-warnings', str(HERE / 'wasi.mjs'), str(wasm)]
    decoder.write_text('#!/bin/sh\ncase "$0" in\n  *encoder) exec ' + shlex.join([*node, 'encode']) + ' ;;\n  *) exec ' + shlex.join(node) + ' ;;\nesac\n')
    decoder.chmod(0o755)
    run = [sys.executable, str(HERE / 'run.py'), '--adapter', str(decoder), '--cache', str(cache), '--report', str(args.report.resolve())]
    if args.record_statements:
        run.append('--record-statements')
    result = subprocess.run(run, cwd=ROOT)
    if args.report.exists():
        report = json.loads(args.report.read_text())
        report['backend'] = 'wasm64-wasi via Node'
        report['wasm_adapter_sha256'] = sha(wasm)
        report['wasm_adapter_bytes'] = wasm.stat().st_size
        report['source_sha256'] = sources
        report['compiler_sha256'] = sha(args.jaic)
        report['node'] = subprocess.check_output(['node', '--version'], text=True).strip()
        args.report.write_text(json.dumps(report, indent=2) + '\n')
    return result.returncode


if __name__ == '__main__':
    raise SystemExit(main())
