"""Is the machine quiet enough to trust a timing? Snapshots of load, thermal state, power source, free
memory and known noisy processes, taken around every measurement block, and the rules that turn two
snapshots into flags.

The parsers take text so tests can feed them captured output; `snapshot()` is the only part that touches
the machine.
"""
from __future__ import annotations

import os
import re
import subprocess
import sys
import time
from pathlib import Path

# Process names (basename of the command) that compete for CPU, memory or disk with a compiler: other
# compilers and build tools, the indexer and the backup.
NOISY_NAMES = {
    'cargo', 'rustc', 'rustdoc', 'sccache', 'rust-analyzer', 'clang', 'clang++', 'cc', 'gcc', 'g++', 'cc1',
    'ld', 'lld', 'ld64', 'swiftc', 'swift-frontend', 'jaic', 'jailsp', 'mds', 'mds_stores', 'mdworker',
    'mdworker_shared', 'backupd', 'backupd-helper', 'tmutil',
}

# A free-memory drop that counts as a state change: at least this fraction and this many MiB.
MEMORY_DROP_FRACTION = 0.2
MEMORY_DROP_MIB = 512
# A load average rise that counts as a state change, besides being above the limit.
LOAD_RISE = 1.0


def sh(*command: str) -> str:
    try:
        return subprocess.run(command, capture_output=True, text=True, timeout=30).stdout
    except (OSError, subprocess.TimeoutExpired):
        return ''


# ------------------------------------------------------------------------------------------ parsers

def parse_thermal_macos(text: str) -> dict:
    """`pmset -g therm`: `CPU_Speed_Limit = 80` style lines appear only when the OS is limiting the CPU."""
    limits = {key: int(value) for key, value in re.findall(r'(\w*Speed_Limit|\w*Scheduler_Limit)\s*=\s*(\d+)', text)}
    throttled = any(value < 100 for key, value in limits.items() if 'Speed' in key)
    return {'source': 'pmset', 'throttled': throttled, 'limits': limits,
            'raw': ' '.join(line.strip() for line in text.splitlines() if line.strip())[:200]}


def parse_power_macos(text: str) -> str:
    """`pmset -g batt`: `Now drawing from 'AC Power'` or `'Battery Power'`."""
    match = re.search(r"drawing from '([^']+)'", text)
    if not match:
        return 'unknown'
    return 'ac' if 'AC' in match[1] else 'battery'


def parse_vm_stat(text: str) -> float | None:
    """Free plus inactive plus speculative pages from `vm_stat`, in MiB (what the OS can hand out)."""
    size = re.search(r'page size of (\d+) bytes', text)
    if not size:
        return None
    pages = 0
    for key in ('Pages free', 'Pages inactive', 'Pages speculative'):
        match = re.search(rf'{key}:\s+(\d+)', text)
        pages += int(match[1]) if match else 0
    return round(pages * int(size[1]) / 2**20, 1)


def parse_meminfo(text: str) -> float | None:
    match = re.search(r'MemAvailable:\s+(\d+) kB', text)
    return round(int(match[1]) / 1024, 1) if match else None


def noisy_processes(ps_output: str, ignore_pids: set[int] | None = None) -> list[str]:
    """Names of the known noisy processes in `ps -axo pid=,comm=` output (each name once, sorted)."""
    found = set()
    for line in ps_output.splitlines():
        parts = line.strip().split(None, 1)
        if len(parts) != 2 or not parts[0].isdigit() or int(parts[0]) in (ignore_pids or ()):
            continue
        name = parts[1].rsplit('/', 1)[-1].strip()
        if name in NOISY_NAMES or name.startswith(('rustc', 'mdworker')):
            found.add(name)
    return sorted(found)


# ---------------------------------------------------------------------------------------- the machine

def thermal_state() -> dict:
    if sys.platform == 'darwin':
        return parse_thermal_macos(sh('pmset', '-g', 'therm'))
    zones = sorted(Path('/sys/class/thermal').glob('thermal_zone*/temp'))
    temps = []
    for zone in zones:
        try:
            temps.append(int(zone.read_text()) / 1000)
        except (OSError, ValueError):
            continue
    return {'source': 'sysfs', 'throttled': None, 'max_temp_c': max(temps) if temps else None}


def power_source() -> str:
    if sys.platform == 'darwin':
        return parse_power_macos(sh('pmset', '-g', 'batt'))
    supplies = sorted(Path('/sys/class/power_supply').glob('*/online'))
    for supply in supplies:
        try:
            if supply.read_text().strip() == '1':
                return 'ac'
        except OSError:
            continue
    return 'battery' if supplies else 'unknown'


def free_memory_mib() -> float | None:
    if sys.platform == 'darwin':
        return parse_vm_stat(sh('vm_stat'))
    try:
        return parse_meminfo(Path('/proc/meminfo').read_text())
    except OSError:
        return None


def snapshot() -> dict:
    load = os.getloadavg()
    return {'time': time.time(), 'cpus': os.cpu_count(), 'load': [round(v, 2) for v in load],
            'thermal': thermal_state(), 'power': power_source(), 'free_mib': free_memory_mib(),
            'noisy': noisy_processes(sh('ps', '-axo', 'pid=,comm='), {os.getpid(), os.getppid()})}


# --------------------------------------------------------------------------------------------- rules

def default_max_load(cpus: int | None) -> float:
    """A quarter of the cores: a compiler run uses a few, and a busier machine skews the comparison."""
    return max(1.0, (cpus or 4) * 0.25)


def start_problems(snap: dict, max_load: float) -> tuple[list[str], list[str]]:
    """(errors, warnings) before the first measurement. Errors stop the run unless forced."""
    errors, warnings = [], []
    if snap['load'][0] > max_load:
        errors.append(f"1-minute load average {snap['load'][0]} is above {max_load:g}")
    if snap['thermal'].get('throttled'):
        errors.append(f"the CPU is thermally limited ({snap['thermal'].get('limits')})")
    if snap['noisy']:
        warnings.append('noisy processes running: ' + ', '.join(snap['noisy']))
    if snap['power'] == 'battery':
        warnings.append('running on battery power')
    return errors, warnings


def block_flags(before: dict, after: dict, max_load: float) -> list[str]:
    """Why a measurement block between these two snapshots is suspect (empty when it looks quiet)."""
    flags = []
    for label, snap in (('before', before), ('after', after)):
        if snap['load'][0] > max_load:
            flags.append(f"load {snap['load'][0]} above {max_load:g} {label}")
        if snap['thermal'].get('throttled'):
            flags.append(f'CPU thermally limited {label}')
    if after['load'][0] - before['load'][0] > LOAD_RISE:
        flags.append(f"load rose from {before['load'][0]} to {after['load'][0]}")
    if before['thermal'].get('throttled') != after['thermal'].get('throttled'):
        flags.append('thermal state changed')
    if before['power'] != after['power']:
        flags.append(f"power source changed ({before['power']} to {after['power']})")
    free_before, free_after = before.get('free_mib'), after.get('free_mib')
    if free_before and free_after is not None:
        drop = free_before - free_after
        if drop >= MEMORY_DROP_MIB and drop / free_before >= MEMORY_DROP_FRACTION:
            flags.append(f'free memory fell from {free_before:.0f} to {free_after:.0f} MiB')
    noisy = sorted(set(before['noisy']) | set(after['noisy']))
    if noisy:
        flags.append('noisy processes: ' + ', '.join(noisy))
    return flags


def purge_caches() -> tuple[bool, str]:
    """Drop the OS file cache (`purge` on macOS). (done, note): the note says why not when it could not."""
    if sys.platform != 'darwin':
        return False, 'no purge command on this platform'
    try:
        done = subprocess.run(['purge'], capture_output=True, text=True, timeout=120)
    except (OSError, subprocess.TimeoutExpired) as error:
        return False, f'purge unavailable: {error}'
    if done.returncode != 0:
        return False, f'purge failed (exit {done.returncode}; it usually needs root): {done.stderr.strip()[:100]}'
    return True, ''
