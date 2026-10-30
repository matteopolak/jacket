"""What a comparison needs recorded so it can be repeated and audited: binary hashes and versions, the
machine, a hash of every input, and the raw timings."""
from __future__ import annotations

import csv
import hashlib
import platform
import subprocess
from pathlib import Path

from bench.results import machine_info, sh


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for block in iter(lambda: handle.read(1 << 20), b''):
            digest.update(block)
    return digest.hexdigest()


def tree_sha256(directory: Path) -> str:
    """One hash over every file under `directory`: relative names and contents, in sorted order."""
    digest = hashlib.sha256()
    for path in sorted(p for p in directory.rglob('*') if p.is_file()):
        digest.update(path.relative_to(directory).as_posix().encode() + b'\0')
        try:
            digest.update(path.read_bytes())
        except OSError:
            digest.update(b'<unreadable>')
        digest.update(b'\0')
    return digest.hexdigest()


def version_of(binary: Path, env: dict[str, str] | None = None) -> str:
    """The first line `binary --version` prints ('' when it prints none or cannot run)."""
    try:
        done = subprocess.run([str(binary), '--version'], capture_output=True, text=True, timeout=30,
                              stdin=subprocess.DEVNULL, env=env)
    except (OSError, subprocess.TimeoutExpired):
        return ''
    lines = (done.stdout or done.stderr).strip().splitlines()
    return lines[0] if lines else ''


def binary_facts(binary: Path, env: dict[str, str] | None = None) -> dict:
    if not binary.exists():
        return {}
    return {'sha256': sha256_file(binary), 'size_bytes': binary.stat().st_size, 'version': version_of(binary, env)}


def machine_specs() -> dict:
    """CPU model, cores, RAM, OS and its version."""
    info = machine_info()
    if platform.system() == 'Darwin':
        info['os_version'] = sh('sw_vers', '-productVersion')
        info['os_build'] = sh('sw_vers', '-buildVersion')
    elif platform.system() == 'Windows':
        from bench import winapi
        facts = winapi.machine_facts()
        info['os_version'] = facts['os_version'] or platform.version()
        info['os_build'] = facts['os_build'] or platform.version()
    else:
        info['os_version'] = platform.version()
    return info


RAW_COLUMNS = ['workload', 'mode', 'phase', 'round', 'position', 'compiler', 'wall_s', 'cpu_s', 'rss_mib',
               'exit', 'stages']


def write_raw_csv(path: Path, samples: list[dict]) -> None:
    """One row per timed run, in the order they ran (`position` is its place within its round)."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('w', newline='') as handle:
        writer = csv.DictWriter(handle, fieldnames=RAW_COLUMNS, extrasaction='ignore')
        writer.writeheader()
        for sample in samples:
            row = {**sample, 'stages': ';'.join(f'{k}={v:.6f}' for k, v in sample.get('stages', {}).items())}
            writer.writerow(row)
