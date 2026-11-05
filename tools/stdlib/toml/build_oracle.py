"""Build the independent release Rust oracle with a checked source/binary receipt.

    python3 build_oracle.py --target-dir /tmp/toml-target --receipt /tmp/oracle-build.json

Invoke through the shared heavy gate locally. This builds only an example; production Rust
configuration parsing and dependencies are unchanged.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from toolkit.repo import ROOT


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot():
    files = [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / 'rust-toolchain.toml', ROOT / 'crates/jacket-config/Cargo.toml', *sorted((ROOT / 'crates/jacket-config').rglob('*.rs'))]
    if (ROOT / '.cargo/config.toml').is_file():
        files.append(ROOT / '.cargo/config.toml')
    return {str(path.relative_to(ROOT)): digest(path) for path in files}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--receipt', type=Path, required=True)
    args = parser.parse_args()
    sources = snapshot()
    target = args.target_dir.resolve()
    command = ['cargo', 'build', '--locked', '--release', '-p', 'jacket-config', '--example', 'toml-oracle', '--target-dir', str(target), '-j', '2']
    subprocess.run(command, cwd=ROOT, env=dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='2'), check=True)
    if snapshot() != sources:
        raise RuntimeError('oracle sources changed during compilation')
    binary = target / 'release/examples' / ('toml-oracle.exe' if os.name == 'nt' else 'toml-oracle')
    report = dict(command=command, source_sha256=sources, binary_sha256=digest(binary), rustc=subprocess.check_output(['rustc', '--version'], cwd=ROOT, text=True).strip(), rustflags=os.environ.get('RUSTFLAGS', ''))
    args.receipt.parent.mkdir(parents=True, exist_ok=True)
    args.receipt.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
