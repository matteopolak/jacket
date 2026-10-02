#!/usr/bin/env python3
"""Save an allocation-instrumented compiler baseline with reproducible metadata."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
def version(*args: str) -> str:
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()
def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--samples', type=int, default=100)
    parser.add_argument('--upstream', action='store_true')
    args = parser.parse_args()
    if args.samples < 2: parser.error('--samples must be at least 2')
    environment = os.environ.copy()
    environment['RUSTC_WRAPPER'] = ''
    environment['CARGO_TARGET_DIR'] = str(ROOT / 'target')
    command = ['cargo', 'bench', '-p', 'jai-bench', '--bench', 'compiler', '--locked', '--', '--color', 'never', '--sample-count', str(args.samples), '--sample-size', '1']
    if args.upstream: command.append('--include-ignored')
    files = sorted(list((ROOT / 'crates').rglob('*.rs')) + [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / 'rust-toolchain.toml', ROOT / '.cargo/config.toml'])
    if args.upstream: files.append(ROOT / 'corpus/upstreams.json')
    llvm_prefix = environment.get('LLVM_SYS_221_PREFIX')
    llvm_config = str(Path(llvm_prefix) / 'bin/llvm-config') if llvm_prefix else 'llvm-config'
    metadata = {'format': 2, 'timestamp': datetime.now(timezone.utc).isoformat(), 'command': command,
                'platform': platform.platform(), 'architecture': platform.machine(),
                'rustc': version('rustc', '-vV'), 'cargo': version('cargo', '-V'),
                'allocation_instrumented': True, 'allocation_scope': 'Rust allocator only; LLVM native allocations excluded', 'shared_host': True,
                'llvm': {'version': version(llvm_config, '--version'), 'prefix': version(llvm_config, '--prefix')},
                'input_hashes': {str(p.relative_to(ROOT)): digest(p) for p in files}}
    for name, root in [('reference', ROOT / 'reference'), ('upstream', ROOT / 'corpus/upstream')]:
        if name == 'upstream' and not args.upstream: continue
        sources = sorted(root.rglob('*.jai'))
        combined = hashlib.sha256()
        for p in sources:
            combined.update(str(p.relative_to(root)).encode() + b'\0')
            combined.update(bytes.fromhex(digest(p)))
        metadata[name + '_corpus'] = {'files': len(sources), 'bytes': sum(p.stat().st_size for p in sources), 'sha256': combined.hexdigest()}
    destination = ROOT / 'artifacts/benchmarks' / datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    destination.mkdir(parents=True)
    with tarfile.open(destination / 'compiler-source.tar.gz', 'w:gz') as archive:
        for p in files: archive.add(p, arcname=str(p.relative_to(ROOT)), recursive=False)
    with (destination / 'results.txt').open('w') as output, (destination / 'build.txt').open('w') as errors:
        result = subprocess.run(command, cwd=ROOT, env=environment, stdout=output, stderr=errors)
    metadata['exit_code'] = result.returncode
    (destination / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
    print(destination)
    if result.returncode: raise SystemExit(result.returncode)
if __name__ == '__main__': main()
