"""The result format every subcommand writes, and how it is stored and shown."""
from __future__ import annotations

import json
import os
import platform
import shutil
import subprocess
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path

from stats import Stat

FORMAT_VERSION = 3
ROOT = Path(__file__).resolve().parent.parent.parent


class Status(str, Enum):
    OK = 'ok'
    SKIPPED = 'skipped'
    FAILED = 'failed'


@dataclass
class Result:
    """One benchmarked thing: a workload under one compiler and mode, one LSP request kind, one lint run.

    `wall`, `cpu` (user + system) and `rss_mib` each summarize the repeated runs. `stages` holds named
    sub-measurements (compiler stages), `info` plain facts (workload size, binary size, counts)."""
    status: Status = Status.OK
    reason: str = ''
    wall: Stat | None = None
    cpu: Stat | None = None
    rss_mib: Stat | None = None
    stages: dict[str, Stat] = field(default_factory=dict)
    info: dict = field(default_factory=dict)

    @classmethod
    def skipped(cls, reason: str, **info) -> Result:
        return cls(Status.SKIPPED, reason, info=info)

    @classmethod
    def failed(cls, reason: str, **info) -> Result:
        return cls(Status.FAILED, reason, info=info)

    def to_json(self) -> dict:
        data: dict = {'status': self.status.value}
        if self.reason:
            data['reason'] = self.reason
        for name in ('wall', 'cpu', 'rss_mib'):
            stat = getattr(self, name)
            if stat:
                data[name] = stat.to_json()
        if self.stages:
            data['stages'] = {name: stat.to_json() for name, stat in self.stages.items()}
        if self.info:
            data['info'] = self.info
        return data

    @classmethod
    def from_json(cls, data: dict) -> Result:
        def stat(name: str) -> Stat | None:
            return Stat.from_json(data[name]) if name in data else None

        return cls(Status(data['status']), data.get('reason', ''), stat('wall'), stat('cpu'), stat('rss_mib'),
                   {name: Stat.from_json(s) for name, s in data.get('stages', {}).items()}, data.get('info', {}))


@dataclass
class Report:
    tool: str
    settings: dict
    subjects: dict = field(default_factory=dict)  # compilers, servers, tools: name -> description
    results: dict[str, Result] = field(default_factory=dict)
    machine: dict = field(default_factory=lambda: machine_info())
    date: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat(timespec='seconds'))

    def to_json(self) -> dict:
        return {'format': FORMAT_VERSION, 'tool': self.tool, 'date': self.date, 'machine': self.machine,
                'settings': self.settings, 'subjects': self.subjects,
                'results': {key: result.to_json() for key, result in self.results.items()}}

    @classmethod
    def from_json(cls, data: dict) -> Report:
        return cls(data['tool'], data.get('settings', {}), data.get('subjects', {}),
                   {key: Result.from_json(r) for key, r in data['results'].items()},
                   data.get('machine', {}), data.get('date', ''))

    def failures(self) -> dict[str, str]:
        return {key: r.reason for key, r in self.results.items() if r.status is Status.FAILED}

    def write(self, json_path: Path, markdown_path: Path, markdown: str) -> None:
        for path in (json_path, markdown_path):
            path.parent.mkdir(parents=True, exist_ok=True)
        json_path.write_text(json.dumps(self.to_json(), indent=2) + '\n')
        markdown_path.write_text(markdown)


def load_report(path: Path) -> Report:
    return Report.from_json(json.loads(path.read_text()))


# ------------------------------------------------------------------------------------ machine info

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
    else:
        cpuinfo, meminfo = Path('/proc/cpuinfo'), Path('/proc/meminfo')
        info['cpu'] = next((line.split(':', 1)[1].strip() for line in cpuinfo.read_text().splitlines()
                            if line.startswith('model name')), '') if cpuinfo.exists() else ''
        memory = next((str(int(line.split()[1]) * 1024) for line in meminfo.read_text().splitlines()
                       if line.startswith('MemTotal')), '') if meminfo.exists() else ''
    info['memory_gib'] = round(int(memory) / 2**30, 1) if memory.isdigit() else None
    return info


def checkout_info() -> dict:
    """The commit this repository is at and whether tracked files differ from it."""
    commit = sh('git', '-C', str(ROOT), 'rev-parse', 'HEAD')
    dirty = bool(sh('git', '-C', str(ROOT), 'status', '--porcelain', '--untracked-files=no'))
    return {'commit': commit, 'dirty': dirty}


def tool_version(command: list[str]) -> str:
    """A one-line `--version` answer, or '' when the tool prints something else."""
    if not shutil.which(command[0]) and not Path(command[0]).exists():
        return ''
    text = sh(*command, '--version')
    return text if text and '\n' not in text and not text.startswith('Content-Length') else ''
