#!/usr/bin/env python3
"""Pin recent upstream Jai sources as inert compatibility input, never run them."""
from __future__ import annotations
import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
import os
import re
from pathlib import Path, PurePosixPath
import subprocess
import urllib.parse
import urllib.request

REPOSITORIES = (
    'focus-editor/focus', 'Ivo-Balbaert/The_Way_to_Jai', 'SogoCZE/Jails',
    'rluba/jaison', 'withlang-dev/open-jai', 'ostef/Vk-Engine', 'roeyb1/sgpu',
    # Smoke-test set: the language:Jai repositories with 20+ stars pushed since 2026-05-05
    # (docs/tools/third-party-smoke-test.md explains how to refresh it). abhiramasonny/jaithon
    # matched too but is its own language that happens to use the .jai extension.
    'danieltan1517/chess-jai', 'gabrielmfern/forbear', 'rexim/rexim.github.io',
    'kooparse/ui_builder', 'DavidColson/Photon', 'kujukuju/KodaJai', 'UnNabbo/no_api',
)
# Libraries the projects above import from git submodules: pinned like the
# projects but exempt from the recency cutoff.
DEPENDENCIES = ('SogoCZE/jai_parser', 'ostef/Linalg', 'ostef/Jolt-Jai', 'ostef/JoltC', 'wolfpld/tracy',
                'OrangeLightning219/jai_parser')
# Metaprogramming libraries that exercise the Compiler module's node API (typed
# compiler_get_nodes, compiler_get_code, TYPECHECKED bodies). Few change often, so they
# are exempt from the recency cutoff too.
LIBRARIES = (
    'sjorsdonkers/match-jai', 'sjorsdonkers/yield-jai', 'GufNZ/JaiModules-AST_Utils',
    'PixelRifts/Jai-Shader-Transpiler', 'Stuart-Mouse/jai-utils', 'n00bmind/unotest',
    # rluba's library family (jaison is a project above); several import each other.
    'rluba/jai-tracy', 'rluba/jai-redis', 'rluba/uniform', 'rluba/cluster', 'rluba/jai-csv',
    'rluba/jai-postgres', 'rluba/stubborn', 'rluba/hyperserve', 'rluba/wait_group', 'rluba/jai-date',
    # Libraries with their own test suites or self-checking examples: sweep cases check their
    # results (docs/tools/upstream-corpus.md, "Recorded outputs").
    'sjorsdonkers/toml-jai', 'OrangeLightning219/jai-format',
    'segcore/jai-protobuf', 'n00bmind/reflector', 'smari/jai-xml',
)
# Dependencies pinned to the consumer's submodule commit (else the newest commit).
SUBMODULE_REVISIONS = {
    'ostef/Linalg': '5f60c11f057787a804ce9582c5daeccc0e8de89a',
    'ostef/Jolt-Jai': '56d1cd47b92d6c08ecdab5b9057addce35dab41a',
    # Jolt-Jai's Source/JoltC submodule: the C wrapper around Jolt Physics that libJoltC is built from.
    'ostef/JoltC': 'd395f4138e1d29dfd46884f6ba00ff865248af08',
    # rluba/jai-tracy's `tracy` submodule (Tracy v0.11.1): libtracy is built from its client sources.
    'wolfpld/tracy': '30997d5ca6bb632cc10807a1da8a6d3de0aeeb3c',
    # jai-format's `modules/jai_parser` submodule (a fork of SogoCZE/jai_parser).
    'OrangeLightning219/jai_parser': '7745b7960f3af8f090e089fceb89a28ead151c40',
}
# Repositories of which only these path prefixes are taken (plus licenses and READMEs): Tracy's
# profiler GUI, server and bundled libraries are not needed to build its client library.
SOURCE_PREFIXES = {
    'wolfpld/tracy': ('public/',),
}
# Data files a project reads at compile time (`#run read_entire_file`, fonts...),
# by path prefix.
RESOURCE_PREFIXES = {
    'focus-editor/focus': ('config/', 'fonts/', 'images/', 'themes/'),
    # sgpu examples compile their Slang shaders at run time and one loads a sample texture.
    'roeyb1/sgpu': ('examples/shaders/', 'examples/sample.png'),
    # JoltC builds with CMake (tools/build_vk_engine_libs.py); Jolt Physics itself is cloned there.
    'ostef/JoltC': ('CMakeLists.txt', 'Examples/'),
    # chess-jai reads its NNUE network at compile time and its fonts, images and sounds at run time.
    'danieltan1517/chess-jai': ('resources/',),
    'kooparse/ui_builder': ('demo/assets/',),
    'gabrielmfern/forbear': ('apps/Inter.ttf',),
    # 30.15_global_data embeds this image with #run add_global_data and prints its size.
    'Ivo-Balbaert/The_Way_to_Jai': ('examples/30/pixel.png',),
    # jai-xml's test.jai parses every file under test_data/ (the committed subset of its suite).
    'smari/jai-xml': ('test_data/',),
    # jai-protobuf's tests generate Jai code from these schemas.
    'segcore/jai-protobuf': ('examples/protos/',),
}
# Per-repository download cap where a needed data file is bigger than the default 8 MiB
# (chess-jai's network is 21 MB).
MAX_FILE_BYTES = {'danieltan1517/chess-jai': 32 * 1024 * 1024}
# Submodule mount points: (consumer directory link, target relative to corpus/upstream).
MODULE_LINKS = (
    ('SogoCZE--Jails/modules/jaison', 'rluba--jaison'),
    ('SogoCZE--Jails/modules/unicode_utils', 'rluba--jaison/unicode_utils'),
    ('SogoCZE--Jails/modules/jai_parser', 'SogoCZE--jai_parser'),
    ('OrangeLightning219--jai-format/modules/jai_parser', 'OrangeLightning219--jai_parser'),
    ('ostef--Vk-Engine/Modules/Linalg', 'ostef--Linalg'),
    ('ostef--Vk-Engine/Modules/JoltPhysics', 'ostef--Jolt-Jai'),
    ('ostef--Jolt-Jai/Source/JoltC', 'ostef--JoltC'),
    # Libraries that are imported as modules by name (`#import "AST_Utils"`).
    ('_modules/AST_Utils', 'GufNZ--JaiModules-AST_Utils'),
    ('rluba--jai-tracy/tracy', 'wolfpld--tracy'),
    # rluba's modules import each other by these names (`#import "date"`, `"wait_group"`...).
    ('_modules/uniform', 'rluba--uniform'),
    ('_modules/wait_group', 'rluba--wait_group'),
    ('_modules/cluster', 'rluba--cluster'),
    ('_modules/date', 'rluba--jai-date'),
    ('_modules/stubborn', 'rluba--stubborn'),
    ('_modules/tracy', 'rluba--jai-tracy'),
    ('_modules/hyperserve', 'rluba--hyperserve'),
    # uniform's build file loads its test runner from its own `modules/stubborn`.
    ('rluba--uniform/modules/stubborn', 'rluba--stubborn'),
)
ROOT = Path(__file__).resolve().parents[1]

def data_root() -> Path:
    # Worktrees share the gitignored corpus/upstream and artifacts/ with the main checkout.
    common = subprocess.run(['git', 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                            cwd=ROOT, capture_output=True, text=True)
    return Path(common.stdout.strip()).parent if common.returncode == 0 else ROOT
DATA = data_root()

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
    limit = MAX_FILE_BYTES.get(repo, 8 * 1024 * 1024)
    with urllib.request.urlopen(request, timeout=60) as response:
        data = response.read(limit + 1)
    if len(data) > limit:
        raise ValueError(f'oversize source: {repo}/{path}')
    return data

# C-family sources a project builds its own helper libraries from (never prebuilt binaries).
NATIVE_SOURCE_SUFFIXES = ('.c', '.h', '.m', '.mm', '.cpp', '.cc', '.hpp', '.inl')

def fetch(repo: str, since: datetime, pinned: dict | None = None) -> dict:
    key = repo.replace('/', '--')
    cache = DATA / 'artifacts/upstream-cache' / f'{key}.git'
    cache.parent.mkdir(parents=True, exist_ok=True)
    if not cache.exists():
        subprocess.run(['git', 'clone', '--bare', '--filter=blob:none', '--depth=128', f'https://github.com/{repo}.git', str(cache)], check=True)
    # Existing manifest pins win even on a machine with no cache.
    revision = pinned['revision'] if pinned else SUBMODULE_REVISIONS.get(repo) or git(cache, 'rev-parse', 'HEAD')
    if not re.fullmatch(r'[0-9a-f]{40}', revision):
        raise ValueError(f'invalid revision: {revision}')
    if subprocess.run(['git', '--git-dir', str(cache), 'cat-file', '-e', revision + '^{commit}'], capture_output=True).returncode:
        subprocess.run(['git', '--git-dir', str(cache), 'fetch', '--filter=blob:none', '--depth=128', 'origin', revision], check=True)
    stamp = git(cache, 'log', '-1', '--format=%cI', revision, '--', '*.jai')
    if not stamp and repo in DEPENDENCIES + LIBRARIES:
        stamp = git(cache, 'log', '-1', '--format=%cI', revision)  # native-only dependency (no .jai files)
    if not stamp:
        raise ValueError(f'no Jai source changes in 128 commits: {repo}')
    source_date = datetime.fromisoformat(stamp.replace('Z', '+00:00'))
    record = {'repository': repo, 'revision': revision, 'latest_jai_change': stamp,
              'source_of_truth': f'https://github.com/{repo}/tree/{revision}'}
    if source_date < since and repo not in DEPENDENCIES + LIBRARIES:
        return record | {'selection': 'stale-excluded', 'files': []}
    paths = git(cache, 'ls-tree', '-r', '--name-only', revision).splitlines()
    if repo in SOURCE_PREFIXES:
        paths = [p for p in paths if p.startswith(SOURCE_PREFIXES[repo]) or '/' not in p]
    selected = [p for p in paths if p.endswith('.jai') or p.endswith(NATIVE_SOURCE_SUFFIXES) or PurePosixPath(p).name.lower() in {'copying', 'readme.md'} or PurePosixPath(p).name.lower().startswith('license')
                or (p.startswith(RESOURCE_PREFIXES.get(repo, ())) and not p.endswith('.psd'))]
    destination = DATA / 'corpus/upstream' / key
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
    for repo in REPOSITORIES + DEPENDENCIES + LIBRARIES:
        record = fetch(repo, since, pins.get(repo)); projects.append(record)
        print(f"{repo}: {record['selection']}, {len(record['files'])} text files at {record['revision']}", flush=True)
    upstream = DATA / 'corpus/upstream'
    for link, target in MODULE_LINKS:
        path = upstream / link
        path.parent.mkdir(parents=True, exist_ok=True)
        if path.is_symlink() or path.exists():
            path.unlink()
        path.symlink_to(os.path.relpath(upstream / target, path.parent))
    temporary = manifest.with_suffix('.json.tmp')
    temporary.write_text(json.dumps({'format': 1, 'minimum_source_date': since.isoformat(), 'projects': projects}, indent=2) + '\n')
    temporary.replace(manifest)
if __name__ == '__main__': main()
