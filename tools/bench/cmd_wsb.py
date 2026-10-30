r"""Write a Windows Sandbox configuration that runs a benchmark command in a disposable, offline VM.

    python tools\bench\main.py wsb --map ref=C:\jai --map jaic=C:\jacket\target\release --out-dir C:\bench-vm ^
        -- compare --baseline {ref}\bin\jai.exe --candidate {jaic}\jaic.exe --out-dir {out}

Mapped folders are read-only inside the VM at `C:\bench\<name>`; `{name}`, `{repo}`, `{python}` and `{out}`
in the command are replaced by their folder inside the VM. The VM has no network, so the harness runs with
`--no-sandbox`; the VM is the wall. See docs/tools/benchmarks.md.
"""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

from bench.winsandbox import windows_sandbox_config
from toolkit.repo import ROOT

GUEST = 'C:\\bench'


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument('--map', action='append', metavar='NAME=FOLDER', default=[],
                        help='a host folder to make readable in the VM as C:\\bench\\NAME (repeatable): the compilers, '
                             'and the toolchain the compilers link with')
    parser.add_argument('--out-dir', type=Path, required=True,
                        help='host folder for the results; writable in the VM as C:\\bench\\out')
    parser.add_argument('--python', type=Path, default=Path(sys.base_prefix),
                        help='the Python installation to run the harness with (default: the one running this)')
    parser.add_argument('--memory-mb', type=int, default=8192, help='memory of the VM (default: %(default)s)')
    parser.add_argument('--launch', action='store_true', help='start Windows Sandbox on the configuration')
    parser.add_argument('benchmark', nargs=argparse.REMAINDER, metavar='-- COMMAND...',
                        help='the harness subcommand and its arguments, e.g. compare --baseline ...')


def guest_folders(maps: list[str]) -> dict[str, Path]:
    folders = {}
    for spec in maps:
        name, separator, host = spec.partition('=')
        if not separator or not name.isidentifier() or not host:
            sys.exit(f'--map wants NAME=FOLDER with NAME a plain word, got {spec!r}')
        if name in ('repo', 'python', 'out'):
            sys.exit(f'--map name {name!r} is taken')
        if not Path(host).is_dir():
            sys.exit(f'--map {name}: {host} is not a folder')
        folders[name] = Path(host).resolve()
    return folders


def guest_path(name: str) -> str:
    return f'{GUEST}\\{name}'


def fill(arguments: list[str], names: list[str]) -> list[str]:
    """`arguments` with `{name}` replaced by the folder inside the VM."""
    values = {name: guest_path(name) for name in (*names, 'repo', 'python', 'out')}
    out = []
    for argument in arguments:
        for name, path in values.items():
            argument = argument.replace('{' + name + '}', path)
        out.append(argument)
    return out


def run_script(arguments: list[str]) -> str:
    """The batch file the VM runs at logon: the harness, its output kept in the results folder, then shutdown."""
    command = subprocess.list2cmdline([guest_path('python') + '\\python.exe',
                                       guest_path('repo') + '\\tools\\bench\\main.py', *arguments, '--no-sandbox'])
    log = guest_path('out') + '\\log.txt'
    return ('@echo off\r\n'
            f'set PYTHONPATH={guest_path("repo")}\\tools\r\n'
            'set PYTHONUTF8=1\r\n'
            f'cd /d {guest_path("out")}\r\n'
            f'{command} > {log} 2>&1\r\n'
            f'echo exit code %ERRORLEVEL% >> {log}\r\n'
            'shutdown /s /t 5\r\n')


def build(args: argparse.Namespace) -> tuple[str, str]:
    """(the `.wsb` text, the batch file text)."""
    folders = guest_folders(args.map)
    command = [a for a in args.benchmark if a != '--']
    if not command:
        sys.exit('give the benchmark command after --, e.g. -- compare --baseline ... --candidate ...')
    mapping = [(ROOT, guest_path('repo'), True), (args.python, guest_path('python'), True),
               (args.out_dir.resolve(), guest_path('out'), False),
               *((host, guest_path(name), True) for name, host in folders.items())]
    script = run_script(fill(command, list(folders)))
    logon = 'cmd.exe /c ' + guest_path('out') + '\\run.cmd'
    return windows_sandbox_config(mapping, logon, args.memory_mb), script


def run(args: argparse.Namespace) -> int:
    config, script = build(args)
    args.out_dir.mkdir(parents=True, exist_ok=True)
    # Both are ASCII unless a host path is not: cmd.exe would read such a path in the OEM code page either way,
    # and the .wsb is XML, whose default encoding is UTF-8. Write UTF-8 without a BOM (cmd mishandles a BOM).
    (args.out_dir / 'run.cmd').write_text(script, encoding='utf-8', newline='')
    path = args.out_dir / 'bench.wsb'
    path.write_text(config, encoding='utf-8')
    print(f'wrote {path}; double-click it (or run with --launch) to start the offline VM; '
          f'results and log.txt land in {args.out_dir}')
    if args.launch:
        if sys.platform != 'win32':
            sys.exit('--launch needs Windows')
        subprocess.Popen(['WindowsSandbox.exe', str(path)])
    return 0
