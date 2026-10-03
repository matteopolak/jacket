#!/usr/bin/env python3
"""Build, execute and package a clean exact-commit browser compiler release."""
import argparse
from datetime import date, datetime, timezone
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import tomllib
import zipfile
from cargo_build_paths import checked_directory
from build_scripting_wasm import storage_directory

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = 'jai-playground.zip'
MANIFEST = 'jai-playground.manifest.json'
MAX_FILES = 512
MAX_FILE_BYTES = 64 * 1024**2
MAX_TOTAL_BYTES = 128 * 1024**2
SUFFIXES = {'.html', '.js', '.mjs', '.css', '.wasm', '.json', '.txt', '.md', '.jai', '.svg', '.woff', '.woff2'}
FORBIDDEN = {'reference', 'corpus', 'artifacts', 'target', '.git', 'node_modules'}
REQUIRED = {'index.html', 'worker.mjs', 'engine.mjs', 'editor.mjs', 'workspace.mjs', 'style.css', 'jai_wasm.wasm', 'release.json'}


def sha256(contents):
    return hashlib.sha256(contents).hexdigest()


def pinned_channel(root, today=None):
    channel = tomllib.loads((root / 'rust-toolchain.toml').read_text())['toolchain']['channel']
    if not isinstance(channel, str) or not re.fullmatch(r'nightly-\d{4}-\d{2}-\d{2}', channel):
        raise ValueError('browser releases require an exact dated nightly')
    published = date.fromisoformat(channel.removeprefix('nightly-'))
    if ((today or datetime.now(timezone.utc).date()) - published).days < 14:
        raise ValueError('pinned nightly must be at least 14 days old')
    return channel


def clean_revision(root):
    revision = subprocess.check_output([browser_tool('git', root), 'rev-parse', '--verify', 'HEAD^{commit}'], cwd=root, text=True).strip()
    if not re.fullmatch(r'[0-9a-f]{40}', revision):
        raise ValueError('release commit must be a full SHA-1 commit')
    status = subprocess.check_output([browser_tool('git', root), 'status', '--porcelain=v1', '--untracked-files=all'], cwd=root, text=True)
    if status.strip():
        raise ValueError('browser releases require a clean checkout, including untracked source')
    return revision


def asset_path(name):
    path = PurePosixPath(name)
    if not name or len(name.encode('utf-8')) > 240 or '\\' in name or ':' in name or any(ord(char) < 32 or ord(char) == 127 for char in name):
        raise ValueError('invalid browser asset path')
    if path.is_absolute() or path.as_posix() != name or any(part in {'', '.', '..'} | FORBIDDEN for part in path.parts):
        raise ValueError('unsafe browser asset path: ' + name)
    if path.suffix not in SUFFIXES or path.name == 'build-metadata.json':
        raise ValueError('unsupported browser release asset: ' + name)
    return name


def assets(stage):
    records = []
    total = 0
    for path in sorted(stage.rglob('*')):
        if path.is_symlink():
            raise ValueError('browser release assets cannot be symlinks')
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValueError('browser release assets must be regular files')
        name = asset_path(path.relative_to(stage).as_posix())
        size = path.stat().st_size
        if size > MAX_FILE_BYTES:
            raise ValueError('browser release asset exceeds the size limit')
        total += size
        if total > MAX_TOTAL_BYTES or len(records) >= MAX_FILES:
            raise ValueError('browser release exceeds its bounded inventory')
        records.append({'path': name, 'size': size, 'sha256': sha256(path.read_bytes())})
    if not REQUIRED <= {record['path'] for record in records}:
        raise ValueError('browser release is missing required runtime/frontend assets')
    return records


def write_archive(stage, destination, records):
    # Fixed timestamps and modes make identical source/artifact inputs reproducible.
    with zipfile.ZipFile(destination, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for record in records:
            contents = (stage / record['path']).read_bytes()
            if len(contents) != record['size'] or sha256(contents) != record['sha256']:
                raise ValueError('asset changed before archiving: ' + record['path'])
            info = zipfile.ZipInfo(record['path'], date_time=(1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | 0o644) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, contents)
    with zipfile.ZipFile(destination) as archive:
        if archive.namelist() != [record['path'] for record in records]:
            raise ValueError('archive inventory differs from the manifest')
        for record in records:
            info = archive.getinfo(record['path'])
            if info.file_size != record['size'] or sha256(archive.read(info)) != record['sha256']:
                raise ValueError('archive contents failed verification')


def browser_tool(name, root):
    executable = shutil.which(name)
    if executable is None:
        raise ValueError('independently installed ' + name + ' is required')
    return str(checked_directory(Path(executable).resolve(strict=True), root))


def build_environment():
    environment = dict(os.environ)
    for key in ('RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'RUSTFLAGS',
                'CARGO_ENCODED_RUSTFLAGS', 'NODE_OPTIONS', 'NODE_PATH', 'ESBUILD_BINARY_PATH',
                'npm_config_script_shell', 'NPM_CONFIG_SCRIPT_SHELL', 'CC', 'CXX', 'AR'):
        environment.pop(key, None)
    for key in list(environment):
        if key.startswith('CARGO_TARGET_') and key.endswith('_LINKER'):
            environment.pop(key)
    environment['CARGO_INCREMENTAL'] = '0'
    return environment


def build_frontend(root):
    inputs = [root / name for name in ('package.json', 'package-lock.json',
              'tools/check_editor_dependencies.mjs', 'tools/build_browser_editor.mjs')]
    present = [path.is_file() for path in inputs]
    if not any(present):
        return
    if not all(present):
        raise ValueError('frontend dependency/build inventory is incomplete')
    subprocess.run([browser_tool('node', root), str(inputs[2])], cwd=root, env=build_environment(), check=True)
    subprocess.run([browser_tool('npm', root), 'ci', '--ignore-scripts'], cwd=root, env=build_environment(), check=True)
    subprocess.run([browser_tool('npm', root), 'run', 'build'], cwd=root, env=build_environment(), check=True)
    subprocess.run([browser_tool('npm', root), 'test'], cwd=root, env=build_environment(), check=True)


def require_output_space(path, reserve=0):
    if shutil.disk_usage(storage_directory(path)).free < 2 * 1024**3 + reserve:
        raise ValueError('release output requires the 2 GiB free-space floor plus staging/archive headroom')


def package(root, output, target_dir=None):
    output = checked_directory(output, root)
    pinned_channel(root)
    revision = clean_revision(root)
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ValueError('release output must be absent or an empty directory; assets are never overwritten')
    if output.is_relative_to(root):
        relative = output.relative_to(root)
        if not relative.parts:
            raise ValueError('release output cannot replace the checkout')
        ignored = subprocess.run([browser_tool('git', root), 'check-ignore', '--quiet', str(relative / 'release-output')], cwd=root)
        if ignored.returncode != 0:
            raise ValueError('in-checkout release output must be gitignored')
    require_output_space(output, 2 * MAX_TOTAL_BYTES)
    subprocess.run([sys.executable, str(root / 'tools/check_ci_sources.py')], cwd=root, env=build_environment(), check=True)
    subprocess.run([sys.executable, str(root / 'tools/check_dependency_age.py')], cwd=root, env=build_environment(), check=True)
    build_frontend(root)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.jai-browser-release-', dir=output.parent) as temporary:
        working = Path(temporary)
        stage = working / 'stage'
        command = [sys.executable, str(root / 'tools/build_scripting_wasm.py'), '--release', '--output', str(stage)]
        if target_dir is not None:
            command.extend(['--target-dir', str(target_dir)])
        subprocess.run(command, cwd=root, env=build_environment(), check=True)
        # The build receipt is local evidence, never a relocatable public asset.
        receipt_path = stage / 'build-metadata.json'
        receipt = json.loads(receipt_path.read_text())
        if receipt.get('wasm_sha256') != sha256((stage / 'jai_wasm.wasm').read_bytes()):
            raise ValueError('staged Wasm does not match its successful build receipt')
        receipt_path.unlink()
        (stage / 'release.json').write_text(json.dumps({'schema_version': 1, 'commit': revision}, sort_keys=True) + '\n')
        records = assets(stage)
        # Probe the staged wrapper and real module; failures leave no release assets.
        proof_path = working / 'probe.json'
        subprocess.run([browser_tool('node', root), str(root / 'tools/check_browser_release.mjs'), str(stage), '--report', str(proof_path)], cwd=root, env=build_environment(), check=True)
        proof = json.loads(proof_path.read_text())
        if proof.get('commit') != revision or proof.get('runtime') is not True or type(proof.get('lsp')) is not bool:
            raise ValueError('staged compiler probe identity/capabilities are invalid')
        if clean_revision(root) != revision:
            raise ValueError('compiler source changed while building its release')
        require_output_space(output, MAX_TOTAL_BYTES)
        ready = working / 'ready'; ready.mkdir()
        archive = ready / ARCHIVE
        write_archive(stage, archive, records)
        manifest = {'schema_version': 1, 'commit': revision, 'dirty_checkout': False,
                    'entrypoint': 'index.html', 'files': records,
                    'capabilities': {'runtime': True, 'lsp': proof['lsp']},
                    'archive': {'name': ARCHIVE, 'size': archive.stat().st_size, 'sha256': sha256(archive.read_bytes())}}
        (ready / MANIFEST).write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
        if clean_revision(root) != revision:
            raise ValueError('compiler source changed before publication')
        require_output_space(output)
        os.replace(ready, output)
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--target-dir', type=Path, help='preserve the explicit Cargo target override')
    parser.add_argument('--toolchain', action='store_true', help='validate age and print the pinned nightly, without building')
    args = parser.parse_args()
    if args.toolchain:
        print(pinned_channel(ROOT)); return
    if args.output is None:
        parser.error('--output is required unless --toolchain is selected')
    result = package(ROOT, args.output, args.target_dir)
    print(f"Verified browser assets for {result['commit']}: {args.output}")


if __name__ == '__main__':
    main()
