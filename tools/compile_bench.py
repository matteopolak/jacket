#!/usr/bin/env python3
"""Measure how long jaic takes to compile real projects from the upstream corpus.

    python3 tools/compile_bench.py [--jaic PATH] [--repeat 5] [--only TEXT] [--modes check,build-O0,build-O2]
                                   [--out results.json] [--markdown results.md]
                                   [--compare old.json] [--threshold 0.10]

Each workload (project x mode) runs --repeat times. The first run is reported as "cold" (jaic keeps no
module cache on disk, so cold only means the first run of the session: binary pages, stdlib and project
sources not yet in the OS file cache); the median of the remaining runs is "warm". Peak RSS is the
compiler process's own maximum resident set (child processes such as the linker are not included).
With jaic's --timings, every run also reports wall time per compiler phase.

--compare flags workloads whose warm wall time or peak RSS grew by more than --threshold (and by more
than --min-seconds / --min-mib, so tiny workloads do not trip on noise) and exits 1 when any did.
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def data_root() -> Path:
    # Worktrees share the gitignored corpus/upstream with the main checkout.
    common = subprocess.run(['git', 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                            cwd=ROOT, capture_output=True, text=True)
    return Path(common.stdout.strip()).parent if common.returncode == 0 else ROOT


UPSTREAM = data_root() / 'corpus' / 'upstream'
MODES = ('check', 'build-O0', 'build-O2')

# name -> (project directory under corpus/upstream, {mode: jaic arguments}, upstream-cases ids whose
# `setup` commands must run first). Projects driven by a metaprogram choose their own optimization
# level, so their build-O2 row passes the metaprogram's release switch instead of -O2.
WORKLOADS: dict[str, tuple[str, dict[str, list[str]], list[str]]] = {
    'focus': ('focus-editor--focus', {
        'check': ['check', 'first.jai'],
        'build-O0': ['build', 'first.jai'],
        'build-O2': ['build', 'first.jai', '-', 'release'],
    }, []),
    'jails': ('SogoCZE--Jails', {
        'check': ['check', 'build.jai'],
        'build-O0': ['build', 'build.jai'],
        # Jails' -release is VERY_OPTIMIZED (O3).
        'build-O2': ['build', 'build.jai', '-', '-release'],
    }, []),
    'sgpu-examples': ('roeyb1--sgpu/examples', {
        'check': ['check', 'build.jai'],
    }, []),
    'jaison-tests': ('rluba--jaison', {
        'check': ['check', 'tests.jai'],
        'build-O0': ['build', 'tests.jai', '-O0', '-o', '{tmp}/jaison-tests'],
        'build-O2': ['build', 'tests.jai', '-O2', '-o', '{tmp}/jaison-tests'],
    }, []),
    # open-jai's examples are small; this one pulls in GetRect, Simp and the windowing modules.
    'open-jai-getrect': ('withlang-dev--open-jai/examples/33', {
        'check': ['check', '33.3_getrect_buttons.jai'],
        'build-O0': ['build', '33.3_getrect_buttons.jai', '-O0', '-o', '{tmp}/getrect-buttons'],
        'build-O2': ['build', '33.3_getrect_buttons.jai', '-O2', '-o', '{tmp}/getrect-buttons'],
    }, []),
    'chess-jai': ('danieltan1517--chess-jai', {
        # Engine and UI; the engine reads its 21 MB network at compile time.
        'check': ['check', 'build.jai'],
        'build-O0': ['build', 'build.jai', '-', 'ui', 'ai', 'debug'],
        'build-O2': ['build', 'build.jai', '-', 'ui', 'ai', 'release'],
    }, []),
    'forbear': ('gabrielmfern--forbear', {
        'check': ['check', 'build.jai'],
        'build-O0': ['build', 'build.jai'],
    }, ['forbear-build']),
}


def default_jaic() -> Path:
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
    return target / 'release' / 'jaic'


def sh(*command: str) -> str:
    try:
        return subprocess.run(command, capture_output=True, text=True, timeout=30).stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        return ''


def machine_info() -> dict:
    info = {'system': platform.system(), 'release': platform.release(), 'machine': platform.machine(),
            'python': platform.python_version(), 'cpus': os.cpu_count()}
    if platform.system() == 'Darwin':
        info['cpu'] = sh('sysctl', '-n', 'machdep.cpu.brand_string')
        memory = sh('sysctl', '-n', 'hw.memsize')
        info['macos'] = sh('sw_vers', '-productVersion')
    else:
        info['cpu'] = next((line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines()
                            if line.startswith('model name')), '') if Path('/proc/cpuinfo').exists() else ''
        memory = next((str(int(line.split()[1]) * 1024) for line in Path('/proc/meminfo').read_text().splitlines()
                       if line.startswith('MemTotal')), '') if Path('/proc/meminfo').exists() else ''
    info['memory_gib'] = round(int(memory) / 2**30, 1) if memory.isdigit() else None
    llvm = shutil.which('llvm-config') or '/opt/homebrew/opt/llvm/bin/llvm-config'
    info['llvm'] = sh(llvm, '--version')
    return info


def jaic_info(jaic: Path) -> dict:
    commit = sh('git', '-C', str(ROOT), 'rev-parse', 'HEAD')
    dirty = bool(sh('git', '-C', str(ROOT), 'status', '--porcelain', '--untracked-files=no'))
    return {'path': str(jaic), 'commit': commit, 'dirty': dirty, 'rustc': sh('rustc', '--version')}


def run_setup(case_ids: list[str]) -> None:
    cases = json.loads((ROOT / 'tools/upstream-cases.json').read_text())
    for case in cases:
        if case['id'] in case_ids:
            for command in case.get('setup', []):
                subprocess.run(command, cwd=(UPSTREAM / case['path']).parent, capture_output=True,
                               stdin=subprocess.DEVNULL)


def parse_timings(stderr: str) -> dict[str, float]:
    phases = {}
    for line in stderr.splitlines():
        if line.startswith('jaic-timing: '):
            name, seconds, _calls = line[len('jaic-timing: '):].rsplit(' ', 2)
            phases[name] = float(seconds)
    # Phases nest: a metaprogram's workspaces are compiled (and code-generated) inside the front end.
    backend = sum(phases.get(p, 0.0) for p in ('prepare output', 'codegen', 'link', 'debug info'))
    if 'total' in phases:
        phases['front end only'] = max(0.0, phases['total'] - backend - phases.get('run', 0.0))
    return phases


def run_once(jaic: Path, cwd: Path, args: list[str], timeout: float) -> dict:
    """One compiler run: wall seconds, peak RSS (MiB) and phase timings."""
    start = time.perf_counter()
    with tempfile.TemporaryFile() as err:
        # `--timings` goes before a `-`: everything after it belongs to the metaprogram.
        command = [str(jaic), *args[:2], '--timings', *args[2:]]
        process = subprocess.Popen(command, cwd=cwd, stdout=subprocess.DEVNULL,
                                   stderr=err, stdin=subprocess.DEVNULL)
        deadline = start + timeout
        while True:
            pid, status, usage = os.wait4(process.pid, os.WNOHANG)
            if pid:
                break
            if time.perf_counter() > deadline:
                process.kill()
                _, status, usage = os.wait4(process.pid, 0)
                raise RuntimeError(f'timed out after {timeout:.0f} s')
            time.sleep(0.005)
        wall = time.perf_counter() - start
        process.returncode = os.waitstatus_to_exitcode(status)
        err.seek(0)
        stderr = err.read().decode(errors='replace')
    if process.returncode != 0:
        tail = [line for line in stderr.splitlines() if not line.startswith('jaic-timing: ')][-3:]
        raise RuntimeError(f'exit {process.returncode}: ' + ' | '.join(tail))
    # ru_maxrss is in bytes on macOS and KiB on Linux.
    rss = usage.ru_maxrss / (2**20 if sys.platform == 'darwin' else 2**10)
    return {'wall': wall, 'rss_mib': rss, 'phases': parse_timings(stderr)}


def summarize(runs: list[dict]) -> dict:
    walls = [r['wall'] for r in runs]
    warm = runs[1:] or runs
    phase_names = list(dict.fromkeys(name for r in warm for name in r['phases']))
    return {
        'cold_seconds': round(walls[0], 4),
        'warm_seconds': round(statistics.median(r['wall'] for r in warm), 4),
        'min_seconds': round(min(walls), 4),
        'max_seconds': round(max(walls), 4),
        'runs_seconds': [round(w, 4) for w in walls],
        'peak_rss_mib': round(max(r['rss_mib'] for r in runs), 1),
        'warm_rss_mib': round(statistics.median(r['rss_mib'] for r in warm), 1),
        'phases_seconds': {name: round(statistics.median(r['phases'].get(name, 0.0) for r in warm), 4)
                           for name in phase_names},
    }


def compare(old: dict, new: dict, threshold: float, min_seconds: float, min_mib: float) -> list[str]:
    """Lines describing every regression of `new` against `old` (same workload keys only)."""
    problems = []
    for key, row in new.get('results', {}).items():
        before = old.get('results', {}).get(key)
        if not before:
            continue
        for field, unit, floor in (('warm_seconds', 's', min_seconds), ('peak_rss_mib', 'MiB', min_mib)):
            a, b = before.get(field), row.get(field)
            if not a or b is None:
                continue
            if b > a * (1 + threshold) and b - a > floor:
                problems.append(f'{key}: {field} {a:g} -> {b:g} {unit} (+{(b / a - 1) * 100:.0f}%)')
    return problems


def markdown(report: dict, baseline: dict | None = None) -> str:
    m, j = report['machine'], report['jaic']
    lines = [
        f"# jaic compile-time benchmark ({report['date'][:10]})",
        '',
        f"Machine: {m.get('cpu') or m['machine']}, {m['cpus']} cores, {m.get('memory_gib')} GiB, "
        f"{m['system']} {m.get('macos') or m['release']}, LLVM {m.get('llvm') or '?'}.  ",
        f"jaic: `{j['commit'][:12]}`{' (dirty)' if j['dirty'] else ''}, {j['rustc']}. "
        f"Runs per workload: {report['settings']['repeat']} (cold = first run, warm = median of the rest).",
        '',
        '| workload | mode | cold s | warm s | min-max s | peak RSS MiB | front end s | codegen s | link s |'
        + (' vs baseline |' if baseline else ''),
        '|---|---|---:|---:|---:|---:|---:|---:|---:|' + ('---|' if baseline else ''),
    ]
    for key, row in report['results'].items():
        name, mode = key.split('/', 1)
        p = row['phases_seconds']
        cells = [name, mode, f"{row['cold_seconds']:.2f}", f"{row['warm_seconds']:.2f}",
                 f"{row['min_seconds']:.2f}-{row['max_seconds']:.2f}", f"{row['peak_rss_mib']:.0f}",
                 f"{p.get('front end only', 0):.2f}", f"{p.get('codegen', 0):.2f}", f"{p.get('link', 0):.2f}"]
        if baseline:
            old = baseline.get('results', {}).get(key)
            cells.append(f"{row['warm_seconds'] / old['warm_seconds']:.2f}x time, "
                         f"{row['peak_rss_mib'] / old['peak_rss_mib']:.2f}x RSS" if old else 'new')
        lines.append('| ' + ' | '.join(cells) + ' |')
    if report.get('failures'):
        lines += ['', 'Failed workloads:', '']
        lines += [f'- `{key}`: {message}' for key, message in report['failures'].items()]
    return '\n'.join(lines) + '\n'


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--jaic', type=Path, default=default_jaic())
    ap.add_argument('--repeat', type=int, default=5, help='runs per workload (the first is the cold run)')
    ap.add_argument('--only', default='', help='substring of "workload/mode" keys to run')
    ap.add_argument('--modes', default=','.join(MODES), help='comma-separated subset of ' + ', '.join(MODES))
    ap.add_argument('--timeout', type=float, default=900)
    ap.add_argument('--out', type=Path, help='write the results as JSON')
    ap.add_argument('--markdown', type=Path, help='write the results as a Markdown table')
    ap.add_argument('--compare', type=Path, help='JSON from an earlier --out to check for regressions')
    ap.add_argument('--threshold', type=float, default=0.10, help='relative growth that counts as a regression')
    ap.add_argument('--min-seconds', type=float, default=0.10, help='ignore smaller absolute time growth')
    ap.add_argument('--min-mib', type=float, default=16, help='ignore smaller absolute RSS growth')
    ap.add_argument('--list', action='store_true', help='print the workloads and exit')
    a = ap.parse_args()
    modes = [m for m in a.modes.split(',') if m]
    todo = [(f'{name}/{mode}', UPSTREAM / project, args, setup)
            for name, (project, variants, setup) in WORKLOADS.items()
            for mode, args in variants.items() if mode in modes and a.only in f'{name}/{mode}']
    if a.list:
        for key, cwd, args, _ in todo:
            print(f'{key:32} {cwd.relative_to(UPSTREAM)}: jaic {" ".join(args)}')
        return
    jaic = a.jaic.resolve()
    if not jaic.exists():
        sys.exit(f'{jaic} not found; build it with cargo build --release -p jaic-cli or pass --jaic')
    if not UPSTREAM.is_dir():
        sys.exit(f'{UPSTREAM} is missing; run python3 tools/fetch_upstreams.py')
    baseline = json.loads(a.compare.read_text()) if a.compare else None
    report = {'format': 1, 'date': datetime.now(timezone.utc).isoformat(timespec='seconds'),
              'machine': machine_info(), 'jaic': jaic_info(jaic), 'settings': {'repeat': a.repeat},
              'results': {}, 'failures': {}}
    with tempfile.TemporaryDirectory(prefix='compile-bench-') as tmp:
        for key, cwd, args, setup in todo:
            if not cwd.is_dir():
                report['failures'][key] = f'{cwd} is missing'
                continue
            run_setup(setup)
            args = [arg.replace('{tmp}', tmp) for arg in args]
            try:
                runs = [run_once(jaic, cwd, args, a.timeout) for _ in range(max(1, a.repeat))]
            except RuntimeError as error:
                report['failures'][key] = str(error)
                print(f'{key:32} FAILED {error}', flush=True)
                continue
            row = report['results'][key] = summarize(runs)
            print(f"{key:32} cold {row['cold_seconds']:7.2f} s  warm {row['warm_seconds']:7.2f} s  "
                  f"peak {row['peak_rss_mib']:7.0f} MiB", flush=True)
    text = markdown(report, baseline)
    if a.out:
        a.out.write_text(json.dumps(report, indent=2) + '\n')
    if a.markdown:
        a.markdown.write_text(text)
    print()
    print(text)
    regressions = compare(baseline, report, a.threshold, a.min_seconds, a.min_mib) if baseline else []
    for line in regressions:
        print(f'REGRESSION {line}')
    sys.exit(1 if regressions or report['failures'] else 0)


if __name__ == '__main__':
    main()
