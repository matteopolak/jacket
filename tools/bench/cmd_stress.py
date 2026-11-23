"""`stress`: a long seeded editing session against a language server, watching its memory and latency."""
from __future__ import annotations

import argparse
import json
import os
import random
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path

from bench import common, procmem
from bench.lsp_client import Client, FrameError, MemoryLimit, ServerExited, Timeout, capabilities, file_uri
from bench.lsp_positions import delete_lines_edit, insert_edit, pick_positions
from bench.results import ROOT
from bench.lsp_workloads import WORKLOADS
from bench.stats import percentile
from repo_output import output_dir

WORKSPACES = ('repo', 'gen-60k', 'gen-240k')
TOLERATED = {-32800, -32801, -32802}  # cancelled, content modified, server cancelled: normal for a busy editor
MAX_OPEN = 6
OS_BLOCK = '''
#if OS == .LINUX {
    stress_linux_%(n)d :: (a: int) -> int {
        linux_value := a + 1;
        return linux_value;
    }
}

#if OS == .WINDOWS {
    stress_windows_%(n)d :: (a: int) -> int {
        windows_value := a * 2;
        return windows_value;
    }
}
'''
NEW_FILE = '''stress_file_%(n)d :: (a: int) -> int {
    local := a + %(n)d;
    return local;
}
'''


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument('--server', default=str(common.default_binary('jailsp')),
                        help='the language server command (default: jailsp from the build tree)')
    parser.add_argument('--workspace', choices=WORKSPACES, default='repo',
                        help='repo: a temp copy of this repository\'s Jai sources; gen-*: a generated program (default: %(default)s)')
    parser.add_argument('--seed', type=int, default=1, help='the session is a function of this (default: %(default)s)')
    parser.add_argument('--steps', type=int, default=300, help='actions in the session (default: %(default)s)')
    parser.add_argument('--idle-every', type=int, default=50, help='an idle gap after this many actions (default: %(default)s)')
    parser.add_argument('--idle-seconds', type=float, default=8, help='length of an idle gap (default: %(default)s)')
    parser.add_argument('--sample-seconds', type=float, default=1, help='memory sampling interval (default: %(default)s)')
    parser.add_argument('--timeout', type=float, default=120, help='seconds before a request counts as unanswered')
    parser.add_argument('--max-rss', type=float, default=8192, help='MiB of server RSS at which the run is abandoned')
    parser.add_argument('--max-peak', type=float, default=0, help='fail when the peak private memory passes this many MiB')
    parser.add_argument('--max-slope', type=float, default=0,
                        help='fail when memory after the idle gaps grows by more than this many MiB per gap')
    parser.add_argument('--keep', action='store_true', help='keep the temporary workspace')
    parser.add_argument('--out', type=Path, default=output_dir('bench', 'stress.json'), help='JSON results (default: %(default)s)')


# ---------------------------------------------------------------------------------------- workspace

def make_workspace(name: str, scratch: Path) -> tuple[Path, list[Path]]:
    """A writable workspace in `scratch` and the Jai files an editor could open in it."""
    if name == 'repo':
        files = subprocess.run(['git', 'ls-files', '-z', 'stdlib/Extensions', 'stdlib/Basic', 'tests/compiler'],
                               cwd=ROOT, capture_output=True, check=True).stdout.decode().split('\0')
        root = scratch / 'repo'
        for rel in files:
            if rel.endswith(('.jai', '.toml')):
                (root / rel).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / rel, root / rel)
        pool = [p for p in sorted(root.glob('stdlib/Extensions/**/*.jai')) if 2_000 < p.stat().st_size < 150_000]
        return root, pool
    workload = next(w for w in WORKLOADS if w.name == name)
    prepared = workload.prepare(scratch)
    return prepared.workspace, sorted(prepared.workspace.glob('*.jai'))


# ---------------------------------------------------------------------------------------- the model

@dataclass
class Doc:
    path: Path
    lines: list[str]
    version: int = 1
    bad_line: int | None = None  # a line the session broke on purpose and has not fixed yet
    os_lines: dict[str, int] = field(default_factory=dict)  # 'linux'/'windows' -> line of the body's first statement

    @property
    def text(self) -> str:
        return '\n'.join(self.lines)

    @property
    def uri(self) -> str:
        return file_uri(self.path)


class Stats:
    def __init__(self) -> None:
        self.latency: dict[str, list[float]] = {}
        self.tolerated = 0
        self.actions: dict[str, int] = {}

    def add(self, kind: str, seconds: float) -> None:
        self.latency.setdefault(kind, []).append(seconds)


class Failure(Exception):
    pass


class Sampler(threading.Thread):
    """Reads the server's memory every `interval` seconds until `stop` is set."""

    def __init__(self, client: Client, interval: float, limit_mib: float):
        super().__init__(daemon=True)
        self.client, self.interval, self.limit = client, interval, limit_mib
        self.series: list[tuple[float, float, float]] = []
        self.stop = threading.Event()
        self.exceeded = ''
        self.began = time.perf_counter()

    def now(self) -> tuple[float, float] | None:
        handle = int(self.client.process._handle) if sys.platform == 'win32' else None
        return procmem.read(self.client.process.pid, handle)

    def run(self) -> None:
        while not self.stop.wait(self.interval):
            if (reading := self.now()) is None:
                continue
            self.series.append((time.perf_counter() - self.began, *reading))
            if self.limit and reading[0] > self.limit:
                self.exceeded = f'resident set {reading[0]:.0f} MiB passed {self.limit:g} MiB'


class Run:
    def __init__(self, args: argparse.Namespace, workspace: Path, pool: list[Path]):
        self.args = args
        self.rng = random.Random(args.seed)
        self.workspace = workspace
        self.pool = pool
        self.client = Client(common.split_command(args.server))
        self.stats = Stats()
        self.docs: dict[Path, Doc] = {}
        self.created: list[Path] = []
        self.counter = 0
        self.gaps: list[dict] = []
        self.sampler = Sampler(self.client, args.sample_seconds, args.max_rss)

    # ------------------------------------------------------------------ plumbing

    def send(self, method: str, params: dict) -> None:
        self.client.notify(method, params)

    def ask(self, kind: str, method: str, params: dict):
        if self.sampler.exceeded:
            raise Failure(self.sampler.exceeded)
        try:
            seconds, response = self.client.request(method, params, self.args.timeout)
        except Timeout:
            raise Failure(f'{kind}: no answer within {self.args.timeout:g} s') from None
        except ServerExited as error:
            raise Failure(f'server exited during {kind}: {error}') from None
        except MemoryLimit as error:
            raise Failure(str(error)) from None
        error = response.get('error')
        if error:
            if error.get('code') in TOLERATED:
                self.stats.tolerated += 1
            else:
                raise Failure(f"{kind}: protocol error {error.get('code')}: {error.get('message')}")
        self.stats.add(kind, seconds)
        with self.client.condition:
            self.client.notifications.clear()
        return response.get('result')

    def count(self, action: str) -> None:
        self.stats.actions[action] = self.stats.actions.get(action, 0) + 1

    # ------------------------------------------------------------------ documents

    def open_doc(self) -> None:
        candidates = [p for p in self.pool if p not in self.docs]
        if not candidates:
            return
        if len(self.docs) >= MAX_OPEN:
            self.close_doc()
        path = self.rng.choice(candidates)
        lines = path.read_text(encoding='utf-8', errors='replace').split('\n')
        doc = Doc(path, lines)
        if self.rng.random() < 0.5:
            self.counter += 1
            first = len(lines)
            lines.extend((OS_BLOCK % {'n': self.counter}).split('\n'))
            doc.os_lines = {'linux': first + 3, 'windows': first + 10}
        self.docs[path] = doc
        self.send('textDocument/didOpen', {'textDocument': {'uri': doc.uri, 'languageId': 'jai', 'version': 1, 'text': doc.text}})
        self.count('open')

    def close_doc(self) -> None:
        if not self.docs:
            return
        path = self.rng.choice(list(self.docs))
        doc = self.docs.pop(path)
        self.send('textDocument/didClose', {'textDocument': {'uri': doc.uri}})
        self.count('close')

    def change(self, doc: Doc, edit: dict) -> None:
        doc.version += 1
        self.send('textDocument/didChange', {'textDocument': {'uri': doc.uri, 'version': doc.version},
                                            'contentChanges': [edit]})

    def some_doc(self) -> Doc | None:
        if not self.docs:
            self.open_doc()
        return self.rng.choice(list(self.docs.values())) if self.docs else None

    def body_position(self, doc: Doc, fraction: float | None = None) -> dict | None:
        return pick_positions(doc.text, self.rng.random() if fraction is None else fraction)

    # ------------------------------------------------------------------ actions

    def type_burst(self) -> None:
        """Keystrokes into a procedure body: a comment, or a line that does not compile and is fixed later."""
        doc = self.some_doc()
        pick = doc and self.body_position(doc)
        if not pick:
            return
        self.counter += 1
        line, indent = pick['insert_line'], pick['indent']
        broken = doc.bad_line is None and self.rng.random() < 0.5
        text = f'stress_{self.counter} := missing_{self.counter} + ;' if broken else f'// stress {self.counter}'
        doc.lines.insert(line, indent)
        self.change(doc, insert_edit(line, 0, indent + '\n'))
        for column, character in enumerate(text, len(indent)):
            doc.lines[line] = doc.lines[line][:column] + character + doc.lines[line][column:]
            self.change(doc, insert_edit(line, column, character))
            if self.rng.random() < 0.2:
                time.sleep(0.01)
        if broken:
            doc.bad_line = line
        self.count('type_burst')
        self.query_in(doc)

    def fix_errors(self) -> None:
        """Delete a broken line a past burst left, a few characters at a time then the whole line."""
        for doc in self.docs.values():
            if doc.bad_line is not None:
                line = doc.bad_line
                doc.bad_line = None
                del doc.lines[line]
                self.change(doc, delete_lines_edit(line))
                self.count('fix_errors')
                self.query_in(doc)
                return

    def query_in(self, doc: Doc) -> None:
        """One of hover, completion, definition, references at a random identifier of `doc`."""
        pick = self.body_position(doc)
        if not pick:
            return
        where = {'textDocument': {'uri': doc.uri}, 'position': {'line': pick['hover']['line'], 'character': pick['hover']['character']}}
        kind = self.rng.choice(['hover', 'completion', 'definition', 'references'])
        method = {'hover': 'textDocument/hover', 'completion': 'textDocument/completion',
                  'definition': 'textDocument/definition', 'references': 'textDocument/references'}[kind]
        params = dict(where)
        if kind == 'references':
            params['context'] = {'includeDeclaration': True}
        self.ask(kind, method, params)
        self.count(kind)

    def query(self) -> None:
        doc = self.some_doc()
        if doc:
            self.query_in(doc)

    def os_branch(self) -> None:
        """Edit and hover inside the `#if OS == .LINUX` and `#if OS == .WINDOWS` branches of a document."""
        docs = [d for d in self.docs.values() if d.os_lines]
        if not docs:
            self.open_doc()
            docs = [d for d in self.docs.values() if d.os_lines]
            if not docs:
                return
        doc = self.rng.choice(docs)
        branch = self.rng.choice(['linux', 'windows'])
        line = next(i for i, text in enumerate(doc.lines) if f'{branch}_value :=' in text)
        if self.rng.random() < 0.5:
            column = len(doc.lines[line]) - 1
            doc.lines[line] = doc.lines[line][:column] + ' ' + doc.lines[line][column:]
            self.change(doc, insert_edit(line, column, ' '))
        column = doc.lines[line].index(f'{branch}_value') + 2
        self.ask('hover_os_branch', 'textDocument/hover', {'textDocument': {'uri': doc.uri},
                                                            'position': {'line': line, 'character': column}})
        self.count('os_branch')

    def watch(self, *changes: tuple[Path, int]) -> None:
        self.send('workspace/didChangeWatchedFiles', {'changes': [{'uri': file_uri(p), 'type': t} for p, t in changes]})

    def file_event(self) -> None:
        """Create, rename or delete a file in the workspace (written for real), or change one on disk."""
        roll = self.rng.random()
        folder = (self.pool[0].parent if self.pool else self.workspace)
        if roll < 0.4 or not self.created:
            self.counter += 1
            path = folder / f'stress_new_{self.counter}.jai'
            path.write_text(NEW_FILE % {'n': self.counter}, encoding='utf-8')
            self.created.append(path)
            self.watch((path, 1))
            self.count('file_create')
        elif roll < 0.7:
            old = self.created.pop(self.rng.randrange(len(self.created)))
            new = old.with_name(old.stem + '_renamed.jai')
            old.rename(new)
            self.created.append(new)
            self.watch((old, 3), (new, 1))
            self.count('file_rename')
        elif roll < 0.9:
            path = self.created.pop(self.rng.randrange(len(self.created)))
            path.unlink()
            self.watch((path, 3))
            self.count('file_delete')
        else:
            path = self.rng.choice(self.created)
            path.write_text(path.read_text(encoding='utf-8') + f'// touched {self.counter}\n', encoding='utf-8')
            self.watch((path, 2))
            self.count('file_change')
        self.ask('workspace_symbol', 'workspace/symbol', {'query': 'stress_file'})

    def idle(self) -> None:
        """Do nothing for the gap, then record the memory the server settled at."""
        time.sleep(self.args.idle_seconds)
        reading = self.sampler.now()
        if reading:
            self.gaps.append({'at_action': sum(self.stats.actions.values()), 'rss_mib': round(reading[0], 1),
                              'private_mib': round(reading[1], 1)})
        self.count('idle')

    # ------------------------------------------------------------------ the session

    def initialize(self) -> None:
        uri = file_uri(self.workspace)
        self.ask('initialize', 'initialize', {
            'processId': None, 'clientInfo': {'name': 'jacket-stress'}, 'rootUri': uri,
            'workspaceFolders': [{'uri': uri, 'name': self.workspace.name}], 'capabilities': capabilities(False)})
        self.client.notify('initialized')

    def run(self) -> None:
        self.sampler.start()
        self.initialize()
        self.open_doc()
        actions = [(self.open_doc, 3), (self.close_doc, 2), (self.type_burst, 8), (self.fix_errors, 4),
                   (self.query, 8), (self.os_branch, 4), (self.file_event, 4)]
        weights = [w for _, w in actions]
        for step in range(1, self.args.steps + 1):
            self.rng.choices([a for a, _ in actions], weights)[0]()
            if step % self.args.idle_every == 0:
                self.idle()
        self.idle()

    def finish(self, failure: str | None) -> dict:
        self.sampler.stop.set()
        self.sampler.join(5)
        try:
            if failure is None:
                self.client.request('shutdown', None, 10)
                self.client.notify('exit')
        except (Timeout, ServerExited, MemoryLimit):
            pass
        peak = self.client.finish(False)
        crashed = self.client.process.returncode not in (0, None, -9, -15) and failure is None
        stderr = self.client.tail()
        if crashed or 'panicked' in stderr:
            failure = failure or f'server exited {self.client.process.returncode}: {stderr}'
        return summarize(self, failure, peak)


def percentile_ms(samples: list[float], q: float) -> float:
    return round(percentile(sorted(samples), q) * 1000, 1) if samples else 0.0


def slope(points: list[tuple[float, float]]) -> float:
    """Least-squares slope of y per x."""
    n = len(points)
    if n < 2:
        return 0.0
    mx = sum(x for x, _ in points) / n
    my = sum(y for _, y in points) / n
    denominator = sum((x - mx) ** 2 for x, _ in points)
    return sum((x - mx) * (y - my) for x, y in points) / denominator if denominator else 0.0


def summarize(run: Run, failure: str | None, peak_rss: float | None) -> dict:
    series = run.sampler.series
    gaps = run.gaps
    private = [g['private_mib'] for g in gaps]
    every = [s for kind in run.stats.latency.values() for s in kind]
    result = {
        'failure': failure,
        'seed': run.args.seed, 'steps': run.args.steps, 'workspace': run.args.workspace,
        'actions': run.stats.actions,
        'tolerated_cancellations': run.stats.tolerated,
        'peak_rss_mib': round(max([r for _, r, _ in series] + [peak_rss or 0]), 1),
        'peak_private_mib': round(max([p for _, _, p in series] + [0]), 1),
        'after_idle': gaps,
        'slope_per_minute_mib': round(slope([(t / 60, p) for t, _, p in series]), 1),
        # The first gap is the server warming up; the trend is judged from the gaps after it.
        'slope_per_gap_mib': round(slope(list(enumerate(private[1:] if len(private) > 3 else private))), 1),
        'latency_ms': {kind: {'n': len(s), 'p50': percentile_ms(s, 0.5), 'p99': percentile_ms(s, 0.99)}
                       for kind, s in sorted(run.stats.latency.items())},
        'latency_all_ms': {'n': len(every), 'p50': percentile_ms(every, 0.5), 'p99': percentile_ms(every, 0.99)},
        'samples': [[round(t, 1), round(r, 1), round(p, 1)] for t, r, p in series],
    }
    if not failure and run.args.max_peak and result['peak_private_mib'] > run.args.max_peak:
        result['failure'] = f"peak private memory {result['peak_private_mib']} MiB passed {run.args.max_peak:g} MiB"
    if not result['failure'] and run.args.max_slope and result['slope_per_gap_mib'] > run.args.max_slope:
        result['failure'] = f"memory after idle gaps grows {result['slope_per_gap_mib']} MiB per gap"
    return result


def render(result: dict) -> str:
    lines = [f"# Language server stress session (seed {result['seed']}, {result['steps']} steps, {result['workspace']})", '']
    lines.append(f"- result: {'FAILED: ' + result['failure'] if result['failure'] else 'ok'}")
    lines.append(f"- peak: {result['peak_rss_mib']} MiB resident, {result['peak_private_mib']} MiB private")
    lines.append(f"- slope: {result['slope_per_minute_mib']} MiB/min over the run, {result['slope_per_gap_mib']} MiB per idle gap")
    lines.append(f"- actions: {', '.join(f'{k} {v}' for k, v in sorted(result['actions'].items()))}")
    lines += ['', 'After each idle gap (MiB private): ' + ', '.join(str(g['private_mib']) for g in result['after_idle']), '']
    rows = [[kind, str(v['n']), f"{v['p50']:.1f}", f"{v['p99']:.1f}"] for kind, v in result['latency_ms'].items()]
    a = result['latency_all_ms']
    rows.append(['all', str(a['n']), f"{a['p50']:.1f}", f"{a['p99']:.1f}"])
    lines.append(common.table(['request', 'n', 'p50 ms', 'p99 ms'], rows))
    return '\n'.join(lines) + '\n'


def run(args: argparse.Namespace) -> int:
    command = common.split_command(args.server)
    if not shutil.which(command[0]) and not Path(command[0]).exists():
        sys.exit(f'{command[0]} not found; build it with cargo build --release -p jai-language-server or pass --server')
    os.environ.setdefault('JAIC_STDLIB', str(ROOT / 'stdlib'))
    scratch = Path(tempfile.mkdtemp(prefix='bench-stress-'))
    try:
        workspace, pool = make_workspace(args.workspace, scratch)
        session = Run(args, workspace, pool)
        failure = None
        try:
            session.run()
        except (Failure, FrameError, ServerExited, Timeout, MemoryLimit) as error:
            failure = str(error) or type(error).__name__
        result = session.finish(failure)
    finally:
        if args.keep:
            print(f'workspace kept at {scratch}', file=sys.stderr)
        else:
            shutil.rmtree(scratch, ignore_errors=True)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, indent=1), encoding='utf-8')
    print(render(result))
    print(f'wrote {args.out}', file=sys.stderr)
    return 1 if result['failure'] else 0
