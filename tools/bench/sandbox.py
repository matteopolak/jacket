"""Runs every timed command without network access and with file access limited to what it needs.

A compiler under test runs compile-time code from the workloads, and a downloaded build is a binary
we did not just compile: neither gets the network, the home directory or the
repository. Reads are allowed from the system, the developer tools, the compiler's own install and
the paths given; writes only to the given paths (the run's scratch directory). Every compiler gets
the same sandbox, so its small start-up cost is shared equally.

macOS uses `sandbox-exec` (Seatbelt), Linux `bwrap` (bubblewrap). Elsewhere, or with `--no-sandbox`,
commands run as they are.
"""
from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

# Readable on macOS besides the paths a run is given: the OS, the dynamic linker cache, the developer
# tools the system linker comes from, Homebrew (projects link its libraries; its clang reads its own
# config) and the device files a process opens at start-up.
MACOS_SYSTEM_READS = [
    '/System', '/usr', '/bin', '/sbin', '/Library/Developer', '/Library/Apple', '/Library/Preferences',
    '/Applications/Xcode.app', '/private/etc', '/private/var/db/dyld', '/private/var/db/timezone',
    '/private/var/select', '/dev', '/opt/homebrew', '/usr/local',
]

# Mounted read-only on Linux.
LINUX_SYSTEM_READS = ['/usr', '/bin', '/sbin', '/lib', '/lib64', '/etc']


@dataclass
class Sandbox:
    reads: list[Path] = field(default_factory=list)
    writes: list[Path] = field(default_factory=list)
    # A server under test and its client share the loopback interface: on macOS only loopback
    # traffic is allowed; Linux shares the host network namespace, since the client runs outside.
    loopback: bool = False

    def wrap(self, command: list[str]) -> list[str]:
        if sys.platform == 'darwin':
            return ['/usr/bin/sandbox-exec', '-p', self.seatbelt_profile(), *command]
        if sys.platform.startswith('linux'):
            return [*self.bwrap_arguments(), *command]
        return command

    def seatbelt_profile(self) -> str:
        # Path traversal (stat of every parent folder) needs metadata reads everywhere; the contents of
        # a file can only be read under an allowed path.
        def subpaths(paths) -> str:
            return ' '.join(f'(subpath {quote(p)})' for p in paths)

        # Seatbelt matches real paths: /var/folders is /private/var/folders.
        reads = [*MACOS_SYSTEM_READS, *developer_app(), *real(self.reads), *real(self.writes)]
        writes = real(self.writes)
        # xcrun (behind `cc` and `xcrun ld`) caches its tool lookups in the per-user temporary folder,
        # whatever TMPDIR says; refused, every lookup takes a second and the link stage looks slow.
        user_temp = str(Path(subprocess.run(['getconf', 'DARWIN_USER_TEMP_DIR'], capture_output=True,
                                            text=True).stdout.strip()).resolve())
        xcrun_cache = f'(regex #"^{re.escape(user_temp)}/xcrun_db")' if user_temp != '.' else ''
        return '\n'.join([
            '(version 1)',
            '(allow default)',
            '(deny network*)',
            *(['(allow network* (local ip "localhost:*") (remote ip "localhost:*"))'] if self.loopback else []),
            '(deny file-read-data)',
            f'(allow file-read-data (literal "/") {xcrun_cache} {subpaths(reads)})',
            '(deny file-write*)',
            f'(allow file-write* (literal "/dev/null") (regex #"^/dev/tty") (regex #"^/dev/fd/") {xcrun_cache} {subpaths(writes)})',
        ])

    def bwrap_arguments(self) -> list[str]:
        arguments = ['bwrap', *([] if self.loopback else ['--unshare-net']), '--die-with-parent',
                     '--dev', '/dev', '--proc', '/proc', '--tmpfs', '/tmp']
        for path in LINUX_SYSTEM_READS:
            if Path(path).exists():
                arguments += ['--ro-bind', path, path]
        for path in self.reads:
            arguments += ['--ro-bind', str(path), str(path)]
        for path in self.writes:
            arguments += ['--bind', str(path), str(path)]
        return arguments

    @staticmethod
    def available() -> str:
        """'' when this platform can sandbox, otherwise why not."""
        if sys.platform == 'darwin':
            return '' if Path('/usr/bin/sandbox-exec').exists() else '/usr/bin/sandbox-exec is missing'
        if sys.platform.startswith('linux'):
            return '' if shutil.which('bwrap') else 'bwrap (bubblewrap) is not installed'
        return f'no sandbox on {sys.platform}'


def developer_app() -> list[str]:
    """The selected Xcode (`/Applications/Xcode_16.4.app` on CI runners, not only `Xcode.app`),
    which the system linker and xcrun load libraries from."""
    try:
        selected = subprocess.run(['xcode-select', '-p'], capture_output=True, text=True).stdout.strip()
    except OSError:
        return []
    parts = Path(selected).parts
    for index, part in enumerate(parts):
        if part.endswith('.app'):
            return [str(Path(*parts[:index + 1]))]
    return []


def real(paths: list[Path]) -> list[str]:
    return sorted({str(p) for path in paths for p in (Path(path), Path(path).resolve())})


def quote(path: str) -> str:
    return '"' + path.replace('\\', '\\\\').replace('"', '\\"') + '"'


def environment(scratch: Path) -> dict[str, str]:
    """The environment for sandboxed runs: the caller's, with temporary files and jaic's cache (the
    helpers it unpacks before linking) inside the scratch folder, since the home directory is unreadable."""
    tmp = scratch / 'tmp'
    cache = scratch / 'cache'
    tmp.mkdir(parents=True, exist_ok=True)
    cache.mkdir(parents=True, exist_ok=True)
    return {**os.environ, 'TMPDIR': str(tmp), 'JAIC_CACHE_DIR': str(cache)}
