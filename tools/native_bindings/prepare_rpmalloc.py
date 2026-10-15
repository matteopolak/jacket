"""Build-only, hash-checked fixes to official pinned C sources. Never edit vendor."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
PATCH = ROOT / 'stdlib/internal/bindings/rpmalloc-heaps/sdk-page-granularity.patch'


def prepare(source, destination):
    pin = json.loads((Path(__file__).with_name('dependencies.json')).read_text())['rpmalloc']
    destination.mkdir(parents=True, exist_ok=True)
    hashes = {}
    for name, expected in pin['files'].items():
        original = source / name
        actual = hashlib.sha256(original.read_bytes()).hexdigest()
        if actual != expected:
            raise ValueError('Pinned rpmalloc source mismatch: ' + str(original))
        hashes[name] = actual
        shutil.copy2(original, destination / Path(name).name)
    subprocess.run(['patch', '--batch', '--forward', '-p1', '-i', str(PATCH.resolve())],
                   cwd=destination, check=True, capture_output=True, text=True)
    manifest = dict(repository=pin['repository'], revision=pin['revision'],
                    original_sha256=hashes,
                    patch_sha256=hashlib.sha256(PATCH.read_bytes()).hexdigest(),
                    patched_source_sha256=hashlib.sha256((destination/'rpmalloc.c').read_bytes()).hexdigest(),
                    reason='Round heap-pool mappings to whole configured pages; retain excess spans as native reserve')
    (destination/'provenance.json').write_text(json.dumps(manifest, indent=2)+'\n')
    return destination


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=ROOT/'.work/vendor/rpmalloc')
    parser.add_argument('--output', type=Path, default=ROOT/'.work/patched-rpmalloc')
    args = parser.parse_args()
    print(prepare(args.source, args.output))
