#!/usr/bin/env python3
"""Verify the pinned official cargo-fuzz archive and every embedded locked dependency."""
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import urllib.request

VERSION = '0.13.2'


def main():
    request = urllib.request.Request(f'https://crates.io/api/v1/crates/cargo-fuzz/{VERSION}',
                                     headers={'User-Agent': 'jai-rs-fuzz-dependency-policy/0.1'})
    with urllib.request.urlopen(request, timeout=30) as response:
        record = json.load(response)['version']
    # The existing checker applies the same age/yanked rules to the tool itself.
    import check_dependency_age
    from datetime import datetime, timezone
    errors = check_dependency_age.check([{'name': 'cargo-fuzz', 'version': VERSION,
        'source': 'registry+https://github.com/rust-lang/crates.io-index'}],
        lambda _name, _version: record, datetime.now(timezone.utc))
    if errors: raise SystemExit('\n'.join(errors))
    with urllib.request.urlopen(f'https://static.crates.io/crates/cargo-fuzz/cargo-fuzz-{VERSION}.crate', timeout=30) as response:
        data = response.read()
    if hashlib.sha256(data).hexdigest() != record['checksum']:
        raise SystemExit('official cargo-fuzz archive checksum mismatch')
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        # Read a single member; never extract or execute archive paths.
        member = archive.extractfile(f'cargo-fuzz-{VERSION}/Cargo.lock')
        if member is None: raise SystemExit('official cargo-fuzz archive has no lockfile')
        content = member.read()
    with tempfile.TemporaryDirectory(prefix='jai-cargo-fuzz-age-') as directory:
        path = Path(directory) / 'Cargo.lock'
        path.write_bytes(content)
        subprocess.run([sys.executable, str(Path(__file__).with_name('check_dependency_age.py')),
                        '--lockfile', str(path)], check=True)
    print(f'cargo-fuzz {VERSION} and its official locked dependency graph passed publication-age verification')


if __name__ == '__main__':
    main()
