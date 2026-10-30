"""The Windows sandbox: an AppContainer with no capabilities, plus the folder grants a run needs.

An AppContainer is a token with a package SID and no capabilities. It has no network (no
`internetClient` capability, and loopback is blocked unless exempted), cannot read the user's profile or
other files, and can read only what its SID or `ALL APPLICATION PACKAGES` is granted: the Windows folder
and Program Files already are. The harness grants the container SID read access to the compilers' installs,
the stdlib and the workload copies, and modify access to the run's scratch folder, with `icacls`, and takes
the grants away when the sandbox closes.

Everything OS-specific is a few `ctypes` calls (`winapi.py`); the `icacls` command lines and the
Windows Sandbox configuration are plain text builders that the tests check on any OS.
"""
from __future__ import annotations

import atexit
import ctypes
import functools
import os
import shutil
import subprocess
import sys
import uuid
from pathlib import Path
from xml.sax.saxutils import escape

from bench import winapi
from bench.winapi import api


class ContainerError(RuntimeError):
    pass


def new_container_name() -> str:
    """A profile name for one run: AppContainer names allow letters, digits, dots, dashes and underscores."""
    return f'jacket.bench.{uuid.uuid4().hex[:16]}'


def grant_command(path: Path, sid: str, write: bool) -> list[str]:
    """`icacls` arguments that give the container read (or modify) access to `path`; a folder's grant is
    inherited by what is inside it, now and later."""
    rights = 'M' if write else 'RX'
    flags = '(OI)(CI)' if path.is_dir() else ''
    return ['icacls', str(path), '/grant', f'*{sid}:{flags}({rights})']


def revoke_command(path: Path, sid: str) -> list[str]:
    return ['icacls', str(path), '/remove:g', f'*{sid}']


def loopback_command(sid: str, allow: bool) -> list[str]:
    """`CheckNetIsolation` arguments that let (or stop letting) the container use the loopback interface.
    Adding the exemption needs an elevated prompt."""
    return ['CheckNetIsolation', 'LoopbackExempt', '-a' if allow else '-d', f'-p={sid}']


def reject_broad(path: Path) -> None:
    """A grant on a drive root or the home folder would undo the sandbox."""
    resolved = path.resolve()
    if resolved == Path(resolved.anchor) or resolved == Path.home().resolve():
        raise ContainerError(f'refusing to grant the container access to {resolved}')


class AppContainer:
    """One AppContainer profile and its grants. `close()` removes both."""

    def __init__(self, reads: list[Path], writes: list[Path], loopback: bool = False):
        self.name = new_container_name()
        self.sid_pointer = None
        self.sid = ''
        self.granted: list[Path] = []
        self.loopback = loopback
        self.loopback_added = False
        self.closed = False
        profile = ctypes.c_void_p()
        code = api().CreateAppContainerProfile(self.name, 'Jacket benchmark', 'Sandbox for timed compiler runs', None,
                                               0, ctypes.byref(profile)) & 0xFFFFFFFF
        if code == winapi.HRESULT_ALREADY_EXISTS:
            code = api().DeriveAppContainerSidFromAppContainerName(self.name, ctypes.byref(profile)) & 0xFFFFFFFF
        if code != 0:
            raise ContainerError(f'cannot create an AppContainer profile (HRESULT 0x{code:08X})')
        self.sid_pointer = profile.value
        self.sid = winapi.sid_string(self.sid_pointer)
        atexit.register(self.close)
        try:
            for path in reads:
                self.grant(path, write=False)
            for path in writes:
                self.grant(path, write=True, required=True)
            if loopback:
                self.allow_loopback()
        except Exception:
            self.close()
            raise

    def grant(self, path: Path, write: bool, required: bool = False) -> None:
        path = Path(path)
        if not path.exists():
            return
        reject_broad(path)
        done = subprocess.run(grant_command(path, self.sid, write), capture_output=True, text=True)
        if done.returncode == 0:
            self.granted.append(path)
        elif required:
            raise ContainerError(f'cannot grant the container access to {path}: {(done.stderr or done.stdout).strip()}')
        # A read grant can fail on a folder everyone already reads (Program Files, a protected install):
        # the container reads it through ALL APPLICATION PACKAGES, so a failed grant is not an error.

    def allow_loopback(self) -> None:
        done = subprocess.run(loopback_command(self.sid, True), capture_output=True, text=True)
        if done.returncode != 0:
            raise ContainerError('cannot exempt the container from the loopback block (run the benchmark from an '
                                 f'elevated prompt, or pass --no-sandbox): {(done.stdout + done.stderr).strip()[:200]}')
        self.loopback_added = True

    def close(self) -> None:
        if self.closed:
            return
        self.closed = True
        for path in self.granted:
            if path.exists():
                subprocess.run(revoke_command(path, self.sid), capture_output=True, text=True)
        if self.loopback_added:
            subprocess.run(loopback_command(self.sid, False), capture_output=True, text=True)
        if self.sid_pointer:
            api().DeleteAppContainerProfile(self.name)
            api().FreeSid(self.sid_pointer)
            self.sid_pointer = None


@functools.cache
def unavailable_reason() -> str:
    """'' when an AppContainer can be created and run a program here, otherwise why not. It really tries:
    a profile with no grants runs `cmd.exe /c exit 0` and the exit code is checked."""
    if sys.platform != 'win32':
        return f'no AppContainer on {sys.platform}'
    if sys.getwindowsversion().major < 10:
        return 'AppContainers need Windows 10 or later'
    if not shutil.which('icacls'):
        return 'icacls is missing'
    try:
        from bench import winjob
        container = AppContainer([], [])
    except (ContainerError, OSError) as error:
        return f'cannot create an AppContainer: {error}'
    try:
        command = [os.path.join(os.environ.get('SystemRoot', r'C:\Windows'), 'System32', 'cmd.exe'), '/c', 'exit 0']
        with open(os.devnull, 'rb') as stdin, open(os.devnull, 'wb') as out:
            result = winjob.run(command, os.environ.get('SystemRoot'), None, stdin, out, out, 30, container.sid_pointer)
        if result.returncode != 0:
            return f'a program in the AppContainer exited with {result.returncode:#x}'
        return ''
    except OSError as error:
        return f'a program cannot start in an AppContainer: {error}'
    finally:
        container.close()


# ------------------------------------------------------------------------------ Windows Sandbox mode

SANDBOX_HOME = r'C:\Users\WDAGUtilityAccount'


def windows_sandbox_config(folders: list[tuple[Path, str, bool]], logon_command: str, memory_mb: int = 8192) -> str:
    """A `.wsb` file for Windows Sandbox: a disposable VM with the network disabled. `folders` are
    (host folder, folder inside the VM, read-only) mappings; `logon_command` runs once the VM is up. The
    VM is a stronger wall than an AppContainer (nothing of the host is visible but the mapped folders),
    at the cost of a slow start and timings taken inside a VM."""
    mapped = ''.join(
        '    <MappedFolder>\n'
        f'      <HostFolder>{escape(str(host))}</HostFolder>\n'
        f'      <SandboxFolder>{escape(inside)}</SandboxFolder>\n'
        f'      <ReadOnly>{"true" if read_only else "false"}</ReadOnly>\n'
        '    </MappedFolder>\n'
        for host, inside, read_only in folders)
    return ('<Configuration>\n'
            '  <Networking>Disable</Networking>\n'
            '  <vGPU>Disable</vGPU>\n'
            '  <ClipboardRedirection>Disable</ClipboardRedirection>\n'
            '  <PrinterRedirection>Disable</PrinterRedirection>\n'
            '  <AudioInput>Disable</AudioInput>\n'
            '  <VideoInput>Disable</VideoInput>\n'
            f'  <MemoryInMB>{memory_mb}</MemoryInMB>\n'
            '  <MappedFolders>\n' + mapped + '  </MappedFolders>\n'
            '  <LogonCommand>\n'
            f'    <Command>{escape(logon_command)}</Command>\n'
            '  </LogonCommand>\n'
            '</Configuration>\n')
