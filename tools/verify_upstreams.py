#!/usr/bin/env python3
"""Verify the inert upstream corpus against its pinned source manifest."""
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
from fetch_upstreams import ROOT, REPOSITORIES, DEPENDENCIES, safe_path

def verify(root: Path = ROOT) -> tuple[int, int]:
    manifest = json.loads((root / 'corpus/upstreams.json').read_text())
    if manifest['format'] != 1: raise ValueError('unsupported manifest format')
    cutoff = datetime.fromisoformat(manifest['minimum_source_date'])
    projects = manifest['projects']
    expected = REPOSITORIES + DEPENDENCIES
    if {p['repository'] for p in projects} != set(expected) or len(projects) != len(expected):
        raise ValueError('repository set does not match compatibility targets')
    sources = 0
    for project in projects:
        if not re.fullmatch(r'[0-9a-f]{40}', project['revision']): raise ValueError('invalid commit SHA')
        if project['selection'] == 'stale-excluded':
            if datetime.fromisoformat(project['latest_jai_change']) >= cutoff or project['files']: raise ValueError('invalid stale exclusion')
            continue
        if project['selection'] != 'recent-source' or datetime.fromisoformat(project['latest_jai_change']) < cutoff:
            raise ValueError('invalid recent-source selection')
        destination = root / 'corpus/upstream' / project['repository'].replace('/', '--')
        listed = set()
        for record in project['files']:
            path = safe_path(record['path'])
            if path in listed: raise ValueError('duplicate source path')
            listed.add(path)
            data = destination.joinpath(*path.parts).read_bytes()
            if len(data) != record['bytes'] or hashlib.sha256(data).hexdigest() != record['sha256']:
                raise ValueError(f'changed source: {destination}/{path}')
            if path.suffix == '.jai': sources += 1
        actual = {p.relative_to(destination).as_posix() for p in destination.rglob('*') if p.is_file()}
        if actual != {str(p) for p in listed}: raise ValueError(f'unlisted or missing files in {destination}')
    return len(projects), sources
if __name__ == '__main__':
    projects, sources = verify()
    print(f'Verified {projects} pinned projects, {sources} Jai sources; no code executed')
