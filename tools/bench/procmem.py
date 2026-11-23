"""Resident and private memory of a process by pid, on macOS, Linux and Windows.

`read(pid, handle)` gives `(resident MiB, private MiB)`. Private is what the process owns alone and is the number
to watch for leaks: physical footprint on macOS (`proc_pid_rusage`), `Private_Clean + Private_Dirty` of
`/proc/PID/smaps_rollup` on Linux, committed private bytes on Windows. psutil is used when it is installed.
"""
from __future__ import annotations

import sys
from pathlib import Path

try:
    import psutil
except ImportError:
    psutil = None

MIB = 2**20


def _darwin_reader():
    import ctypes
    import ctypes.util

    libproc = ctypes.CDLL(ctypes.util.find_library('proc') or '/usr/lib/libproc.dylib')

    class RusageV2(ctypes.Structure):  # struct rusage_info_v2, up to the physical footprint
        _fields_ = [('uuid', ctypes.c_uint8 * 16), ('user_time', ctypes.c_uint64), ('system_time', ctypes.c_uint64),
                    ('pkg_idle_wkups', ctypes.c_uint64), ('interrupt_wkups', ctypes.c_uint64),
                    ('pageins', ctypes.c_uint64), ('wired_size', ctypes.c_uint64), ('resident_size', ctypes.c_uint64),
                    ('phys_footprint', ctypes.c_uint64), ('proc_start_abstime', ctypes.c_uint64),
                    ('proc_exit_abstime', ctypes.c_uint64), ('child_user_time', ctypes.c_uint64),
                    ('child_system_time', ctypes.c_uint64), ('child_pkg_idle_wkups', ctypes.c_uint64),
                    ('child_interrupt_wkups', ctypes.c_uint64), ('child_pageins', ctypes.c_uint64),
                    ('child_elapsed_abstime', ctypes.c_uint64), ('diskio_bytesread', ctypes.c_uint64),
                    ('diskio_byteswritten', ctypes.c_uint64)]

    libproc.proc_pid_rusage.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_void_p]

    def read(pid, handle):
        info = RusageV2()
        if libproc.proc_pid_rusage(pid, 2, ctypes.byref(info)) != 0:  # RUSAGE_INFO_V2
            return None
        return info.resident_size / MIB, info.phys_footprint / MIB

    return read


def _linux_read(pid, handle):
    try:
        rollup = Path(f'/proc/{pid}/smaps_rollup').read_text()
        statm = Path(f'/proc/{pid}/statm').read_text().split()
    except OSError:
        return None
    kib = {name: int(rest.split()[0]) for name, _, rest in (line.partition(':') for line in rollup.splitlines())
           if rest.strip().endswith('kB')}
    resident = int(statm[1]) * 4096 / MIB
    return resident, (kib.get('Private_Clean', 0) + kib.get('Private_Dirty', 0)) / 1024


def _windows_read(pid, handle):
    from bench import winapi
    memory = winapi.process_memory_private(handle)
    return None if memory is None else (memory[0] / MIB, memory[1] / MIB)


def _psutil_read(pid, handle):
    try:
        full = psutil.Process(pid).memory_full_info()
    except psutil.Error:
        return None
    return full.rss / MIB, getattr(full, 'uss', full.rss) / MIB


def _reader():
    if psutil is not None:
        return _psutil_read
    if sys.platform == 'darwin':
        return _darwin_reader()
    if sys.platform == 'win32':
        return _windows_read
    return _linux_read


read = _reader()
