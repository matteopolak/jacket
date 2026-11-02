"""Runs the library comparison benchmarks (`benchmarks/compare/<name>/benches`) with `jacket bench`.

Each comparison measures a Jacket library next to the community libraries it replaces. The community
sources are third-party and unreviewed, so the whole `jacket bench` run (the build, the compile-time code
and the benchmark program) goes inside `bench.sandbox.Sandbox`: no network, writes only to a scratch
copy of the benchmark folder, reads limited to the system, the jacket build, the stdlib and the upstream
sources. Benchmark files import the community libraries through `-import_dir` (`corpus/upstream`);
nothing from there is copied.

A benchmark is named `WORKLOAD__LIBRARY` (for example `decode_api__jacket`); the report groups the
results by workload, one column per library.

    PYTHONPATH=tools python3 tools/bench/library_compare.py json --out target/compare/json.md
    PYTHONPATH=tools python3 tools/bench/library_compare.py regex -- -E 'log_'
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from bench import quiet
from bench.cmd_compile import install_reads
from bench.dialects import parse_compiler
from bench.sandbox import Sandbox, environment
from repo_output import host_platform, shared_output_dir
from toolkit.repo import ROOT, cargo_binary

COMPARE = ROOT / 'benchmarks' / 'compare'
SEPARATOR = '__'


def default_jacket() -> Path:
    return cargo_binary('jacket', 'release')


def jacket_stdlib(jacket: Path) -> list[Path]:
    """The stdlib the jaic next to `jacket` uses. A build reads the stdlib of the checkout it was built
    in, which need not be this one. Only our own jaic is asked: it is a build of this project, never the
    reference compiler or community code."""
    jaic = jacket.with_name('jaic')
    if not jaic.is_file():
        return []
    result = subprocess.run([str(jaic), '-print_stdlib'], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
    folder = Path(result.stdout.strip()) if result.returncode == 0 else None
    return [folder.resolve()] if folder and folder.is_dir() else []


def sandbox_for(jacket: Path, upstream: Path, scratch: Path, extra_reads: list[Path], loopback: bool) -> Sandbox:
    if sys.platform == 'win32':
        # The library benchmarks start `jacket bench` and servers whose output the harness reads through plain
        # `subprocess` calls, which would run unsandboxed; only `run_measured` launches into the AppContainer.
        sys.exit('the library and HTTP comparisons are not supported on Windows (use compile or compare)')
    reads = [*install_reads(parse_compiler(f'jacket={jacket}')), *jacket_stdlib(jacket), upstream, Path(sys.base_prefix), *extra_reads]
    return Sandbox(reads=reads, writes=[scratch], loopback=loopback)


def machine() -> dict[str, str]:
    """CPU, memory and system of this machine, for the report header."""
    info = {'os': platform.platform(), 'cpu': platform.processor() or platform.machine()}
    if sys.platform == 'darwin':
        text = subprocess.run(['system_profiler', 'SPHardwareDataType'], capture_output=True, text=True).stdout
        for line in text.splitlines():
            key, _, value = line.strip().partition(': ')
            if key in ('Chip', 'Total Number of Cores', 'Memory'):
                info[key.lower().replace(' ', '_')] = value
    load = quiet.load_average()
    info['load_average'] = ' '.join(f'{v:.2f}' for v in load)
    return info


def native_libraries(env: dict[str, str]) -> dict[str, str]:
    """`env` with JAIC_NATIVE_LIBS set to the shared native libraries (mbedtls, ...) when they were built."""
    natives = shared_output_dir('native-libs') / host_platform()
    if natives.is_dir():
        env.setdefault('JAIC_NATIVE_LIBS', str(natives))
    return env


def run(name: str, args: argparse.Namespace) -> dict:
    """Run `benchmarks/compare/NAME` and return the `jacket bench --format json` document."""
    source = COMPARE / name
    if not (source / 'benches').is_dir():
        sys.exit(f'library_compare: {source}/benches does not exist')
    jacket = Path(args.jacket).resolve()
    upstream = Path(args.upstream).resolve()
    with tempfile.TemporaryDirectory(prefix=f'library-compare-{name}-') as folder:
        scratch = Path(folder).resolve()
        project = scratch / 'project'
        shutil.copytree(source, project)
        shims = write_shims(scratch, source)
        sandbox = sandbox_for(jacket, upstream, scratch, [source], loopback=name in LOOPBACK)
        command = [str(jacket), 'bench', '--path', str(project), '--format', 'json',
                   *args.bench_arguments, '--',
                   *(['-import_dir', str(shims)] if shims else []), '-import_dir', str(upstream), *args.compiler_flags]
        env = native_libraries(environment(scratch))
        if uses_libpq(source):
            env['LIBRARY_PATH'] = libpq_directory()
        result = subprocess.run(sandbox.wrap(command), cwd=project, env=env,
                                stdout=subprocess.PIPE, stderr=None, text=True)
        if result.returncode not in (0, 1):
            sys.exit(f'library_compare: jacket bench exited with {result.returncode}')
        document = json.loads(result.stdout)
        collect_side_files(project, document)
        return document


def collect_side_files(project: Path, document: dict) -> None:
    """A benchmark process runs in the project folder and the runner keeps only its timing record, so a
    benchmark that has more to say writes files there: `allocs-NAME.tsv` (`NAME<TAB>allocations<TAB>bytes` for one
    call, counted in a pass of its own) and `notes.txt` (what its pre-timing checks found)."""
    allocations = {}
    for path in sorted(project.glob('allocs-*.tsv')):
        name, count, size = path.read_text(encoding='utf-8').split()
        allocations[name] = {'allocs': int(count), 'bytes': int(size)}
    if allocations:
        document['allocations'] = allocations
    notes = project / 'notes.txt'
    if notes.is_file():
        document['notes'] = notes.read_text(encoding='utf-8').strip()


def write_shims(scratch: Path, source: Path) -> Path | None:
    """Modules that exist only in the scratch folder, in an import dir searched before the stdlib:
    the relaxed `Unicode` below for a benchmark that uses Rematch."""
    text = '\n'.join(path.read_text(encoding='utf-8') for path in (source / 'benches').glob('*.jai'))
    folder = scratch / 'shims'
    if 'Lunatoid--Rematch' in text:
        relaxed_unicode(folder)
    return folder if folder.exists() else None


# Rematch hands `Unicode.character_utf32_to_utf8` a string of count 0 to fill, and the stdlib's version
# asserts that the string has room for the character. The comparison measures Rematch with that
# check left out: a `Unicode` module in a scratch import dir, searched before the stdlib, is the stdlib's
# own source minus the four capacity asserts. Nothing else of it changes.
def relaxed_unicode(folder: Path) -> None:
    source = (ROOT / 'stdlib' / 'internal' / 'unicode' / 'module.jai').read_text(encoding='utf-8')
    kept = [line for line in source.splitlines() if 'preallocated_string.count >=' not in line]
    module = folder / 'Unicode'
    module.mkdir(parents=True)
    (module / 'module.jai').write_text('\n'.join(kept) + '\n', encoding='utf-8')


def uses_libpq(source: Path) -> bool:
    return any('libpq' in path.read_text(encoding='utf-8') for path in (source / 'benches').glob('*.jai'))


def libpq_directory() -> str:
    """The folder with libpq (the linker finds `-lpq` through LIBRARY_PATH): `LIBPQ_DIR`, or Homebrew's."""
    candidates = [os.environ.get('LIBPQ_DIR', ''), '/opt/homebrew/lib/postgresql@17', '/opt/homebrew/opt/libpq/lib',
                  '/usr/local/opt/libpq/lib', '/usr/lib/x86_64-linux-gnu', '/usr/lib/aarch64-linux-gnu']
    for candidate in candidates:
        if candidate and any(Path(candidate).glob('libpq.*')):
            return candidate
    sys.exit('library_compare: libpq not found; set LIBPQ_DIR to the folder holding it')


# Comparisons that talk over the loopback interface (the sandbox allows nothing else).
LOOPBACK = {'postgres', 'http'}


def scaled(ns: float) -> str:
    for limit, unit, divisor in ((1e3, 'ns', 1), (1e6, 'us', 1e3), (1e9, 'ms', 1e6)):
        if ns < limit:
            return f'{ns / divisor:.3g} {unit}'
    return f'{ns / 1e9:.3g} s'


def throughput(result: dict) -> str:
    """MB/s from `set_bytes` (or items/s from `set_items`) of one result."""
    if result.get('mb_per_s'):
        return f"{result['mb_per_s']:.0f} MB/s"
    if result.get('items_per_s'):
        return f"{result['items_per_s']:.3g} items/s"
    return ''


def allocation_report(document: dict, libraries: list[str]) -> str:
    """Allocations (and bytes) of one call, a row per workload, a column per library."""
    rows: dict[str, dict[str, dict]] = {}
    for name, value in document.get('allocations', {}).items():
        workload, _, library = name.rpartition(SEPARATOR)
        rows.setdefault(workload, {})[library] = value
    lines = ['| Allocations per call | ' + ' | '.join(libraries) + ' |', '| --- |' + ' --- |' * len(libraries)]
    for workload, by_library in rows.items():
        cells = [f"{v['allocs']:,} ({v['bytes']:,} B)" if (v := by_library.get(library)) else 'n/a' for library in libraries]
        lines.append(f'| {workload} | ' + ' | '.join(cells) + ' |')
    return '\n'.join(lines)


def report(document: dict, libraries: list[str], per_item: bool = False, with_throughput: bool = False) -> str:
    """One table: a row per workload, a column per library (time per iteration with its 95% interval,
    and the ratio to the first library)."""
    rows: dict[str, dict[str, dict]] = {}
    for result in document['results']:
        workload, _, library = result['name'].rpartition(SEPARATOR)
        rows.setdefault(workload, {})[library] = result
    lines = ['| Workload | ' + ' | '.join(libraries) + ' |', '| --- |' + ' --- |' * len(libraries)]
    for workload, by_library in rows.items():
        base = by_library.get(libraries[0])
        cells = []
        for library in libraries:
            result = by_library.get(library)
            if result is None:
                cells.append('n/a')
                continue
            t = result['time_ns']
            items = result.get('items_per_iteration') or 0
            if per_item and items:
                t = {key: value / items for key, value in t.items()}
            cell = f"{scaled(t['estimate'])} ({scaled(t['low'])} to {scaled(t['high'])})"
            if with_throughput and throughput(result):
                cell += f' {throughput(result)}'
            if base is not None and library != libraries[0]:
                cell += f" {result['time_ns']['estimate'] / base['time_ns']['estimate']:.2f}x"
            cells.append(cell)
        lines.append(f'| {workload} | ' + ' | '.join(cells) + ' |')
    return '\n'.join(lines)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('name', help='a folder of benchmarks/compare')
    parser.add_argument('--jacket', default=str(default_jacket()), help='the jacket binary (default: the release build)')
    parser.add_argument('--upstream', default=str(ROOT / 'corpus' / 'upstream'), help='the community sources')
    parser.add_argument('--libraries', default='jacket', help='comma-separated, first is the baseline of the ratios')
    parser.add_argument('--per-item', action='store_true', help='divide each time by the items one iteration handles')
    parser.add_argument('--throughput', action='store_true', help='add MB/s (benchmarks that call set_bytes) to each cell')
    parser.add_argument('--json', help='also write the jacket bench document here')
    parser.add_argument('--out', help='write the Markdown table here')
    parser.add_argument('--bench-arguments', default='', help='more arguments for jacket bench, as one quoted string (a filter, --samples)')
    parser.add_argument('--compiler-flags', default='', help='more flags for jaic, as one quoted string')
    args = parser.parse_args(argv)
    args.bench_arguments = shlex.split(args.bench_arguments)
    args.compiler_flags = shlex.split(args.compiler_flags)
    document = run(args.name, args)
    if args.json:
        Path(args.json).write_text(json.dumps(document, indent=1), encoding='utf-8')
    libraries = args.libraries.split(',')
    text = report(document, libraries, args.per_item, args.throughput)
    if document.get('allocations'):
        text += '\n\n' + allocation_report(document, libraries)
    if document.get('notes'):
        text += '\n\n```\n' + document['notes'] + '\n```'
    header = '\n'.join(f'- {k}: {v}' for k, v in machine().items())
    output = f'{header}\n\n{text}\n'
    if args.out:
        Path(args.out).write_text(output, encoding='utf-8')
    print(output)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
