#!/usr/bin/env python3
"""Fuzzes the exported procedures of stdlib modules with arguments generated from their types.

    python3 tools/fuzz/stdlib_fuzz.py String Unicode Base64 --jaic target/release/jaic

For each module: `discover.jai` (a metaprogram) lists the exported procedures and prints a harness
that calls them with `Fuzz.gen` values; the harness runs under `jaic -interp`; a crash is attributed
to the `CASE <proc> <seed> <max_len>` line printed before it, minimized by lowering the length bound,
and written to `fuzz/regressions/stdlib/` unless the module's `known` list says it is an expected
precondition. Everything runs in the bench sandbox (no network, writes only to the scratch folder).
See docs/tools/stdlib-fuzzing.md.
"""
from __future__ import annotations

import sys as _sys
from pathlib import Path as _Path

# tools/, for folder-qualified imports
_sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
from bench.sandbox import Sandbox, environment  # noqa: E402

LLVM_READS = [Path('/opt/homebrew/opt/llvm'), Path('/usr/lib/llvm-23')]  # clang and the libFuzzer archive
MAX_RESTARTS = 40
MAX_AUTO_DENY = 20
LENGTH_BOUNDS = [0, 1, 2, 4, 8, 16, 32, 64, 128]


@dataclass
class Finding:
    module: str
    proc: str
    seed: int
    max_len: int
    signature: str
    known: bool = False
    stderr: str = ''
    input: str = ''


@dataclass
class Report:
    module: str
    skipped: list[str] = field(default_factory=list)
    procs: list[str] = field(default_factory=list)
    calls: int = 0
    findings: list[Finding] = field(default_factory=list)
    coverage: str = ''


class HarnessError(Exception):
    def __init__(self, message: str, stderr: str):
        super().__init__(message)
        self.stderr = stderr

    def proc(self, harness: Path) -> str | None:
        """The procedure whose generated call the compiler rejected, from the error's line number."""
        match = re.search(rf'{re.escape(harness.name)}:(\d+):', self.stderr)
        if not match:
            return None
        lines = harness.read_text().splitlines()
        for line in lines[int(match.group(1)) - 1:int(match.group(1)) + 1]:
            call = re.search(r'Target\.(\w+)\(', line)
            if call:
                return call.group(1)
        return None


class Runner:
    def __init__(self, args: argparse.Namespace):
        self.args = args
        self.jaic = Path(args.jaic).resolve()
        self.stdlib = Path(args.stdlib).resolve()
        self.scratch = Path(args.scratch).resolve() if args.scratch else Path(tempfile.mkdtemp(prefix='stdlib-fuzz-'))
        self.scratch.mkdir(parents=True, exist_ok=True)
        self.sandbox = None
        if not args.no_sandbox:
            why = Sandbox.available()
            if why:
                print(f'warning: running without a sandbox ({why})', file=sys.stderr)
            else:
                self.sandbox = Sandbox(reads=[ROOT, self.stdlib, self.jaic.parent, *LLVM_READS],
                                       writes=[self.scratch, *(Path(p) for p in (args.corpus, args.artifacts))])
        self.env = environment(self.scratch) if self.sandbox else None
        self.config = json.loads((ROOT / 'tools/fuzz/targets.json').read_text())

    def run(self, command: list[str], timeout: float) -> tuple[int | None, str, str]:
        """Exit code (None on timeout), stdout, stderr."""
        if self.sandbox:
            command = self.sandbox.wrap(command)
        try:
            done = subprocess.run(command, capture_output=True, timeout=timeout, cwd=self.scratch, env=self.env)
        except subprocess.TimeoutExpired as expired:
            return None, (expired.stdout or b'').decode('utf-8', 'replace'), (expired.stderr or b'').decode('utf-8', 'replace')
        return done.returncode, done.stdout.decode('utf-8', 'replace'), done.stderr.decode('utf-8', 'replace')

    def jaic_command(self, mode: str, source: Path, tail: list[str] | None = None) -> list[str]:
        command = [str(self.jaic), mode, str(source), '-import_dir', str(self.stdlib)]
        return command + (['--'] + tail if tail else [])

    def discover(self, module: str, settings: dict) -> tuple[Path | None, list[str]]:
        """Compile the discovery metaprogram for `module`; returns the harness path and the SKIP lines."""
        def lines(key: str) -> str:
            return '\n'.join(settings.get(key, []))

        arity = '\n'.join(f'{k}={v}' for k, v in settings.get('arity', {}).items())
        hooks = settings.get('hooks')
        hooks_path = str((ROOT / 'tools/fuzz' / hooks).resolve()) if hooks else ''
        names = '\n'.join(module_names(module))
        wrapper = self.scratch / f'discover-{module}.jai'
        wrapper.write_text('\n'.join([
            f'MODULE :: "{module}";',
            f'NAMES :: #string END\n{names}\nEND;',
            f'DENY :: #string END\n{lines("deny")}\nEND;',
            f'ALLOW :: #string END\n{lines("allow")}\nEND;',
            f'POINTERS :: #string END\n{lines("pointers")}\nEND;',
            f'NULLABLE :: #string END\n{lines("nullable")}\nEND;',
            f'ARITY :: #string END\n{arity}\nEND;',
            f'HOOKS :: "{hooks_path}";',
            f'#load "{ROOT / "tools/fuzz/discover.jai"}";',
            'main :: () {}',
            '',
        ]))
        code, out, err = self.run(self.jaic_command('-check', wrapper), timeout=120)
        if code != 0 or 'HARNESS BEGIN' not in out:
            print(f'discovery for {module} failed:\n{err or out}', file=sys.stderr)
            return None, []
        skipped = [line[5:] for line in out.splitlines() if line.startswith('SKIP ')]
        body = out.split('HARNESS BEGIN\n', 1)[1].split('HARNESS END', 1)[0]
        harness = self.scratch / f'harness-{module}.jai'
        harness.write_text('ENGINE_BUILD :: false;\n' + body)
        return harness, skipped

    def build(self, module: str, settings: dict) -> tuple[Path | None, list[str], list[str]]:
        """Harness path, its case names and the skipped procedures, dropping any the compiler rejects."""
        settings = {**settings, 'deny': list(settings.get('deny', []))}
        denied: list[str] = []
        for _ in range(MAX_AUTO_DENY):
            harness, skipped = self.discover(module, settings)
            if harness is None:
                return None, [], []
            try:
                return harness, self.list_procs(harness), skipped + denied
            except HarnessError as error:
                # A signature the type info cannot tell apart (variadic, constant parameter): drop the
                # procedure whose call failed to compile and try again.
                proc = error.proc(harness)
                if proc is None:
                    print(error, file=sys.stderr)
                    return None, [], []
                settings['deny'].append(proc)
                denied.append(f'{proc} does not compile when called with generated arguments')
        return None, [], []

    def run_harness(self, harness: Path, proc: str, seed: int, count: int, max_len: int):
        tail = ['--only', proc, '--seed', str(seed), '--count', str(count), '--max-len', str(max_len)]
        return self.run(self.jaic_command('-interp', harness, tail), timeout=self.args.timeout)

    def list_procs(self, harness: Path) -> list[str]:
        code, out, err = self.run(self.jaic_command('-interp', harness, ['--list']), timeout=120)
        if code != 0:
            raise HarnessError(f'cannot list {harness}:\n{err}', err)
        return [line for line in out.splitlines() if line]


def module_names(module: str) -> list[str]:
    """Top-level constant names declared in the module's `module.jai` (its exports and private helpers)."""
    text = (ROOT / 'stdlib' / module / 'module.jai').read_text()
    names: list[str] = []
    for line in text.splitlines():
        if line.startswith('#scope_file'):
            break
        match = re.match(r'(?:#if [^{;]*?\s)?([A-Za-z_]\w*)\s*::', line)
        if match and match.group(1) not in names:
            names.append(match.group(1))
    return names


def signature_of(code: int | None, stderr: str) -> str:
    if code is None:
        return 'timeout'
    for line in stderr.splitlines():
        if 'error:' in line:
            where, _, message = line.partition('error: ')
            location = '/'.join(where.split('/')[-1:]).rstrip(': ')
            location = re.sub(r':\d+$', lambda m: m.group(0), location)
            return re.sub(r'\d{3,}', 'N', f'{location}: {message}'.strip())
    return f'exit {code}'


def last_case(stdout: str) -> tuple[str, int, int] | None:
    for line in reversed(stdout.splitlines()):
        if line.startswith('CASE '):
            _, name, seed, max_len = line.split(' ')
            return name, int(seed), int(max_len)
    return None


def is_known(settings: dict, proc: str, signature: str) -> bool:
    short = proc.split('.', 1)[-1]
    return any(text in signature for text in settings.get('known', {}).get(short, []))


def fuzz_module(runner: Runner, module: str) -> Report:
    args = runner.args
    settings = runner.config.get(module)
    report = Report(module)
    if settings is None:
        print(f'{module}: no entry in tools/fuzz/targets.json', file=sys.stderr)
        return report
    harness, report.procs, report.skipped = runner.build(module, settings)
    if harness is None:
        return report
    seen: set[tuple[str, str]] = set()

    for proc in report.procs:
        if args.only and not proc.endswith(args.only):
            continue
        seed, remaining, restarts = args.seed, args.count, 0
        while remaining > 0 and restarts <= MAX_RESTARTS:
            code, out, err = runner.run_harness(harness, proc, seed, remaining, args.max_len)
            if code == 0 and 'DONE ' in out:
                report.calls += remaining
                break
            case = last_case(out)
            if case is None:
                print(f'{proc}: died before its first case (exit {code}):\n{err}', file=sys.stderr)
                break
            _, crash_seed, _ = case
            report.calls += crash_seed - seed + 1
            signature = signature_of(code, err)
            if (proc, signature) not in seen:
                seen.add((proc, signature))
                finding = minimize(runner, harness, module, proc, crash_seed, args.max_len, signature)
                finding.known = is_known(settings, proc, signature)
                finding.stderr = err
                report.findings.append(finding)
            remaining -= crash_seed - seed + 1
            seed = crash_seed + 1
            restarts += 1
    return report


def minimize(runner: Runner, harness: Path, module: str, proc: str, seed: int, max_len: int, signature: str) -> Finding:
    """The smallest length bound that still fails with the same signature at this seed."""
    best = max_len
    for bound in LENGTH_BOUNDS:
        if bound >= max_len:
            break
        code, _, err = runner.run_harness(harness, proc, seed, 1, bound)
        if (code != 0) and signature_of(code, err) == signature:
            best = bound
            break
    return Finding(module, proc, seed, best, signature)


def engine_signature(stderr: str) -> str:
    """One line naming a native crash: a Jai runtime error, an ASan report or a libFuzzer stop."""
    for line in stderr.splitlines():
        if 'error:' in line and 'runtime error' in line:
            return signature_of(1, line)
        if 'ERROR: AddressSanitizer' in line:
            kind = re.sub(r'on (unknown )?address 0x[0-9a-f]+.*', '', line.split('ERROR: AddressSanitizer: ', 1)[-1])
            frame = re.search(r'#0 0x[0-9a-f]+ in (\S+?)(?:\.\d+)? (\S+)', stderr)
            return f'AddressSanitizer: {kind.strip()} in {frame.group(1)} {frame.group(2)}' if frame else 'AddressSanitizer: ' + kind.strip()
        if 'timeout after' in line or 'ERROR: libFuzzer: timeout' in line:
            return 'timeout'
        if 'ERROR: libFuzzer: out-of-memory' in line:
            return 'out-of-memory'
        if 'ERROR: libFuzzer: fuzz target exited' in line:
            return 'fuzz target exited'
        found = re.match(r'(\S+)\.jai:(\d+): (assertion failed.*)', line.strip().rsplit('/', 1)[-1])
        if found:
            return f'{found.group(1)}.jai:{found.group(2)}: {found.group(3)}'.strip()
    return 'unknown crash'


def build_engine(runner: Runner, module: str, harness: Path, opt: str) -> Path | None:
    """Compile the harness with coverage instrumentation and the libFuzzer engine, ASan on."""
    source = runner.scratch / f'engine-{module}.jai'
    source.write_text(harness.read_text().replace('ENGINE_BUILD :: false;', 'ENGINE_BUILD :: true;', 1))
    binary = runner.scratch / f'engine-{module}'
    command = runner.jaic_command(f'-{opt}', source) + ['-sanitize', 'address,fuzzer', '-exe', str(binary)]
    code, out, err = runner.run(command, timeout=600)
    if code != 0:
        print(f'engine build for {module} failed:\n{err or out}', file=sys.stderr)
        return None
    return binary


def engine_module(runner: Runner, module: str) -> Report:
    """Coverage-guided run of one module: libFuzzer over the generated harness."""
    args = runner.args
    settings = runner.config.get(module)
    report = Report(module)
    if settings is None:
        return report
    harness, report.procs, report.skipped = runner.build(module, settings)
    if harness is None:
        return report
    binary = build_engine(runner, module, harness, args.opt)
    if binary is None:
        return report
    corpus = Path(args.corpus) / module
    artifacts = Path(args.artifacts) / module
    corpus.mkdir(parents=True, exist_ok=True)
    artifacts.mkdir(parents=True, exist_ok=True)
    only = ['--only', f'{module}.{args.only}'] if args.only else []
    env = {**(runner.env or os.environ), 'ASAN_OPTIONS': 'detect_leaks=0:halt_on_error=1:abort_on_error=0:handle_abort=1'}
    flags = [f'-max_total_time={args.time}', f'-max_len={args.engine_max_len}', f'-timeout={int(args.timeout)}',
             f'-artifact_prefix={artifacts}/', '-print_final_stats=1', '-fork=1', '-ignore_crashes=1']
    before = set(artifacts.iterdir())
    done = subprocess.run(runner.sandbox.wrap([str(binary), str(corpus), *flags, *only]) if runner.sandbox
                          else [str(binary), str(corpus), *flags, *only],
                          capture_output=True, env=env, cwd=runner.scratch)
    log = done.stderr.decode('utf-8', 'replace')
    stats = re.search(r'stat::number_of_executed_units:\s+(\d+)', log)
    report.calls = int(stats.group(1)) if stats else 0
    coverage = re.findall(r'cov: (\d+) ft: (\d+) corp: (\d+)', log)
    if coverage:
        report.coverage = f'{coverage[-1][0]} edges, {coverage[-1][2]} corpus entries'
    seen: set[str] = set()
    for crash in sorted(set(artifacts.iterdir()) - before):
        if not crash.name.startswith(('crash-', 'timeout-', 'oom-', 'leak-')):
            continue
        repro = subprocess.run(runner.sandbox.wrap([str(binary), '--trace', *only, str(crash)]) if runner.sandbox
                               else [str(binary), '--trace', *only, str(crash)],
                               capture_output=True, env=env, cwd=runner.scratch, timeout=args.timeout + 30)
        text = repro.stderr.decode('utf-8', 'replace')
        cases = re.findall(r'^CASE (\S+)', repro.stdout.decode('utf-8', 'replace'), re.M)
        proc = cases[-1] if cases else '?'
        signature = engine_signature(text)
        if (proc, signature) in seen:
            continue
        seen.add((proc, signature))
        finding = Finding(module, proc, 0, args.engine_max_len, signature, stderr=text)
        finding.known = is_known(settings, proc, signature)
        finding.input = str(crash)
        report.findings.append(finding)
    return report


def write_regression(finding: Finding, directory: Path) -> Path:
    directory.mkdir(parents=True, exist_ok=True)
    name = re.sub(r'[^A-Za-z0-9_.-]', '_', f'{finding.proc}.{finding.seed}') + '.json'
    path = directory / name
    path.write_text(json.dumps({
        'module': finding.module, 'proc': finding.proc, 'seed': finding.seed, 'max_len': finding.max_len,
        'signature': finding.signature,
    }, indent=2) + '\n')
    return path


def replay(runner: Runner, directory: Path) -> int:
    """Re-run every stored case; returns the number that still fail."""
    failures = 0
    harnesses: dict[str, Path | None] = {}
    for path in sorted(directory.glob('*.json')):
        case = json.loads(path.read_text())
        module = case['module']
        if module not in harnesses:
            harnesses[module], _, _ = runner.build(module, runner.config[module])
        harness = harnesses[module]
        if harness is None:
            failures += 1
            continue
        code, _, err = runner.run_harness(harness, case['proc'], case['seed'], 1, case['max_len'])
        if code != 0:
            failures += 1
            print(f'STILL FAILS {path.name}: {signature_of(code, err)}')
        else:
            print(f'fixed       {path.name}')
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('modules', nargs='*', help='stdlib modules (default: every one in targets.json)')
    parser.add_argument('--jaic', default=str(ROOT / 'target/release/jaic'))
    parser.add_argument('--count', type=int, default=200, help='seeds per procedure')
    parser.add_argument('--seed', type=int, default=1, help='first seed')
    parser.add_argument('--max-len', type=int, default=256, help='bound on generated lengths')
    parser.add_argument('--timeout', type=float, default=60, help='seconds per harness run; a hang is a finding')
    parser.add_argument('--only', help='fuzz only procedures whose name ends with this')
    parser.add_argument('--scratch', help='scratch folder (default: a fresh temporary one)')
    parser.add_argument('--stdlib', default=str(ROOT / 'stdlib'), help='stdlib folder to fuzz')
    parser.add_argument('--engine', action='store_true', help='coverage-guided: build with -sanitize address,fuzzer and run libFuzzer')
    parser.add_argument('--time', type=int, default=60, help='engine seconds per module')
    parser.add_argument('--engine-max-len', type=int, default=512, help='engine input size bound in bytes')
    parser.add_argument('--opt', default='O0', help='optimization level of the engine build')
    parser.add_argument('--corpus', default=str(ROOT / 'fuzz/corpus/stdlib'))
    parser.add_argument('--artifacts', default=str(ROOT / 'fuzz/artifacts/stdlib'))
    parser.add_argument('--no-sandbox', action='store_true')
    parser.add_argument('--write-regressions', action='store_true', help='store new findings under fuzz/regressions/stdlib')
    parser.add_argument('--replay', action='store_true', help='re-run the stored cases instead of fuzzing')
    parser.add_argument('--regressions', default=str(ROOT / 'fuzz/regressions/stdlib'))
    args = parser.parse_args()

    runner = Runner(args)
    if args.replay:
        return 1 if replay(runner, Path(args.regressions)) else 0

    modules = args.modules or list(runner.config)
    new = 0
    for module in modules:
        report = engine_module(runner, module) if args.engine else fuzz_module(runner, module)
        print(f'{module}: {len(report.procs)} cases, {report.calls} calls, {len(report.skipped)} skipped'
              + (f', {report.coverage}' if report.coverage else ''))
        for line in report.skipped:
            print(f'  skip {line}')
        for f in report.findings:
            label = 'known' if f.known else 'FOUND'
            where = f'input={f.input}' if f.input else f'seed={f.seed} max_len={f.max_len}'
            print(f'  {label} {f.proc} {where}: {f.signature}')
            if not f.known:
                new += 1
                if args.write_regressions:
                    print(f'    stored {write_regression(f, Path(args.regressions)).relative_to(ROOT)}')
    print(f'{new} new finding(s)')
    return 1 if new else 0


if __name__ == '__main__':
    sys.exit(main())
