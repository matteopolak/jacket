#!/usr/bin/env python3
"""Pin recent upstream Jai sources as inert compatibility input, never run them."""
from __future__ import annotations
import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
import re
from pathlib import Path, PurePosixPath
import subprocess
import urllib.parse
import urllib.request

REPOSITORIES = (
    'focus-editor/focus', 'Ivo-Balbaert/The_Way_to_Jai', 'SogoCZE/Jails',
    'rluba/jaison', 'withlang-dev/open-jai', 'ostef/Vk-Engine', 'roeyb1/sgpu',
)
ROOT = Path(__file__).resolve().parents[1]

def git(cache: Path, *args: str) -> str:
    return subprocess.check_output(['git', '--git-dir', str(cache), *args], text=True).strip()

def safe_path(path: str) -> PurePosixPath:
    p = PurePosixPath(path)
    if p.is_absolute() or '..' in p.parts or not p.parts:
        raise ValueError(f'unsafe upstream path: {path}')
    return p

def download(repo: str, revision: str, path: str) -> bytes:
    safe_path(path)
    url = f'https://raw.githubusercontent.com/{repo}/{revision}/{urllib.parse.quote(path)}'
    request = urllib.request.Request(url, headers={'User-Agent': 'jai-rust-compatibility-research'})
    with urllib.request.urlopen(request, timeout=60) as response:
        data = response.read(8 * 1024 * 1024 + 1)
    if len(data) > 8 * 1024 * 1024:
        raise ValueError(f'oversize source: {repo}/{path}')
    return data

def fetch(repo: str, since: datetime, pinned: dict | None = None) -> dict:
    key = repo.replace('/', '--')
    cache = ROOT / 'artifacts/upstream-cache' / f'{key}.git'
    cache.parent.mkdir(parents=True, exist_ok=True)
    if not cache.exists():
        subprocess.run(['git', 'clone', '--bare', '--filter=blob:none', '--depth=128', f'https://github.com/{repo}.git', str(cache)], check=True)
    # Existing manifest pins win even on a machine with no cache.
    revision = pinned['revision'] if pinned else git(cache, 'rev-parse', 'HEAD')
    if not re.fullmatch(r'[0-9a-f]{40}', revision):
        raise ValueError(f'invalid revision: {revision}')
    if subprocess.run(['git', '--git-dir', str(cache), 'cat-file', '-e', revision + '^{commit}'], capture_output=True).returncode:
        subprocess.run(['git', '--git-dir', str(cache), 'fetch', '--filter=blob:none', '--depth=128', 'origin', revision], check=True)
    stamp = git(cache, 'log', '-1', '--format=%cI', revision, '--', '*.jai')
    if not stamp:
        raise ValueError(f'no Jai source changes in 128 commits: {repo}')
    source_date = datetime.fromisoformat(stamp)
    record = {'repository': repo, 'revision': revision, 'latest_jai_change': stamp,
              'source_of_truth': f'https://github.com/{repo}/tree/{revision}'}
    if source_date < since:
        return record | {'selection': 'stale-excluded', 'files': []}
    paths = git(cache, 'ls-tree', '-r', '--name-only', revision).splitlines()
    selected = [p for p in paths if p.endswith('.jai') or PurePosixPath(p).name.lower() in {'copying', 'readme.md'} or PurePosixPath(p).name.lower().startswith('license')]
    destination = ROOT / 'corpus/upstream' / key
    def one(path: str) -> dict:
        expected = next((f for f in pinned['files'] if f['path'] == path), None) if pinned else None
        target = destination.joinpath(*safe_path(path).parts)
        if expected and target.exists() and hashlib.sha256(target.read_bytes()).hexdigest() == expected['sha256']:
            return expected
        data = download(repo, revision, path)
        if expected and hashlib.sha256(data).hexdigest() != expected['sha256']:
            raise ValueError(f'source hash changed at immutable revision: {repo}/{path}')
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        return {'path': path, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
    with ThreadPoolExecutor(max_workers=8) as pool:
        files = list(pool.map(one, sorted(selected)))
    return record | {'selection': 'recent-source', 'files': files}

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--since', default='2025-10-01', help='minimum Jai source modification date (UTC)')
    args = parser.parse_args()
    since = datetime.fromisoformat(args.since).replace(tzinfo=timezone.utc)
    manifest = ROOT / 'corpus/upstreams.json'
    old = json.loads(manifest.read_text()) if manifest.exists() else {'projects': []}
    pins = {p['repository']: p for p in old['projects']}
    projects = []
    for repo in REPOSITORIES:
        record = fetch(repo, since, pins.get(repo)); projects.append(record)
        print(f"{repo}: {record['selection']}, {len(record['files'])} text files at {record['revision']}", flush=True)
    temporary = manifest.with_suffix('.json.tmp')
    temporary.write_text(json.dumps({'format': 1, 'minimum_source_date': since.isoformat(), 'projects': projects}, indent=2) + '\n')
    temporary.replace(manifest)
if __name__ == '__main__': main()
