"""The Windows system calls the harness needs, through `ctypes` (no third-party packages): job objects,
process creation with an attribute list, process memory and CPU, machine load, memory and power.

Everything that touches the OS goes through `api()`, which binds the functions once and only on Windows;
the structures, the unit conversions and the text builders in this module work on any OS, so the tests
can check them anywhere.
"""
from __future__ import annotations

import ctypes
import functools
import subprocess
import sys
import time
from ctypes import wintypes
from types import SimpleNamespace

IS_WINDOWS = sys.platform == 'win32'

# Constants from the Windows headers.
STARTF_USESTDHANDLES = 0x100
CREATE_SUSPENDED = 0x4
CREATE_UNICODE_ENVIRONMENT = 0x400
EXTENDED_STARTUPINFO_PRESENT = 0x80000
CREATE_NO_WINDOW = 0x08000000
HANDLE_FLAG_INHERIT = 1
WAIT_OBJECT_0 = 0
WAIT_TIMEOUT = 0x102
INFINITE = 0xFFFFFFFF
JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE = 0x2000
JOB_OBJECT_BASIC_ACCOUNTING_INFORMATION = 1
JOB_OBJECT_BASIC_PROCESS_ID_LIST = 3
JOB_OBJECT_EXTENDED_LIMIT_INFORMATION = 9
PROC_THREAD_ATTRIBUTE_HANDLE_LIST = 0x00020002
PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES = 0x00020009
PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
PROCESS_QUERY_INFORMATION = 0x400
PROCESS_VM_READ = 0x10
HRESULT_ALREADY_EXISTS = 0x800700B7


class FILETIME(ctypes.Structure):
    _fields_ = [('low', wintypes.DWORD), ('high', wintypes.DWORD)]

    @property
    def ticks(self) -> int:
        return (self.high << 32) | self.low


class IO_COUNTERS(ctypes.Structure):
    _fields_ = [(name, ctypes.c_uint64) for name in
                ('read_ops', 'write_ops', 'other_ops', 'read_bytes', 'write_bytes', 'other_bytes')]


class JOBOBJECT_BASIC_ACCOUNTING_INFORMATION(ctypes.Structure):
    _fields_ = [('TotalUserTime', ctypes.c_int64), ('TotalKernelTime', ctypes.c_int64),
                ('ThisPeriodTotalUserTime', ctypes.c_int64), ('ThisPeriodTotalKernelTime', ctypes.c_int64),
                ('TotalPageFaultCount', wintypes.DWORD), ('TotalProcesses', wintypes.DWORD),
                ('ActiveProcesses', wintypes.DWORD), ('TotalTerminatedProcesses', wintypes.DWORD)]


class JOBOBJECT_BASIC_LIMIT_INFORMATION(ctypes.Structure):
    _fields_ = [('PerProcessUserTimeLimit', ctypes.c_int64), ('PerJobUserTimeLimit', ctypes.c_int64),
                ('LimitFlags', wintypes.DWORD), ('MinimumWorkingSetSize', ctypes.c_size_t),
                ('MaximumWorkingSetSize', ctypes.c_size_t), ('ActiveProcessLimit', wintypes.DWORD),
                ('Affinity', ctypes.c_size_t), ('PriorityClass', wintypes.DWORD),
                ('SchedulingClass', wintypes.DWORD)]


class JOBOBJECT_EXTENDED_LIMIT_INFORMATION(ctypes.Structure):
    _fields_ = [('BasicLimitInformation', JOBOBJECT_BASIC_LIMIT_INFORMATION), ('IoInfo', IO_COUNTERS),
                ('ProcessMemoryLimit', ctypes.c_size_t), ('JobMemoryLimit', ctypes.c_size_t),
                ('PeakProcessMemoryUsed', ctypes.c_size_t), ('PeakJobMemoryUsed', ctypes.c_size_t)]


class PROCESS_MEMORY_COUNTERS(ctypes.Structure):
    _fields_ = [('cb', wintypes.DWORD), ('PageFaultCount', wintypes.DWORD),
                ('PeakWorkingSetSize', ctypes.c_size_t), ('WorkingSetSize', ctypes.c_size_t),
                ('QuotaPeakPagedPoolUsage', ctypes.c_size_t), ('QuotaPagedPoolUsage', ctypes.c_size_t),
                ('QuotaPeakNonPagedPoolUsage', ctypes.c_size_t), ('QuotaNonPagedPoolUsage', ctypes.c_size_t),
                ('PagefileUsage', ctypes.c_size_t), ('PeakPagefileUsage', ctypes.c_size_t)]


class MEMORYSTATUSEX(ctypes.Structure):
    _fields_ = [('dwLength', wintypes.DWORD), ('dwMemoryLoad', wintypes.DWORD),
                ('ullTotalPhys', ctypes.c_uint64), ('ullAvailPhys', ctypes.c_uint64),
                ('ullTotalPageFile', ctypes.c_uint64), ('ullAvailPageFile', ctypes.c_uint64),
                ('ullTotalVirtual', ctypes.c_uint64), ('ullAvailVirtual', ctypes.c_uint64),
                ('ullAvailExtendedVirtual', ctypes.c_uint64)]


class SYSTEM_POWER_STATUS(ctypes.Structure):
    _fields_ = [('ACLineStatus', ctypes.c_ubyte), ('BatteryFlag', ctypes.c_ubyte),
                ('BatteryLifePercent', ctypes.c_ubyte), ('SystemStatusFlag', ctypes.c_ubyte),
                ('BatteryLifeTime', wintypes.DWORD), ('BatteryFullLifeTime', wintypes.DWORD)]


class STARTUPINFOW(ctypes.Structure):
    _fields_ = [('cb', wintypes.DWORD), ('lpReserved', wintypes.LPWSTR), ('lpDesktop', wintypes.LPWSTR),
                ('lpTitle', wintypes.LPWSTR), ('dwX', wintypes.DWORD), ('dwY', wintypes.DWORD),
                ('dwXSize', wintypes.DWORD), ('dwYSize', wintypes.DWORD), ('dwXCountChars', wintypes.DWORD),
                ('dwYCountChars', wintypes.DWORD), ('dwFillAttribute', wintypes.DWORD),
                ('dwFlags', wintypes.DWORD), ('wShowWindow', wintypes.WORD), ('cbReserved2', wintypes.WORD),
                ('lpReserved2', ctypes.c_void_p), ('hStdInput', wintypes.HANDLE),
                ('hStdOutput', wintypes.HANDLE), ('hStdError', wintypes.HANDLE)]


class STARTUPINFOEXW(ctypes.Structure):
    _fields_ = [('StartupInfo', STARTUPINFOW), ('lpAttributeList', ctypes.c_void_p)]


class PROCESS_INFORMATION(ctypes.Structure):
    _fields_ = [('hProcess', wintypes.HANDLE), ('hThread', wintypes.HANDLE),
                ('dwProcessId', wintypes.DWORD), ('dwThreadId', wintypes.DWORD)]


class SID_AND_ATTRIBUTES(ctypes.Structure):
    _fields_ = [('Sid', ctypes.c_void_p), ('Attributes', wintypes.DWORD)]


class SECURITY_CAPABILITIES(ctypes.Structure):
    _fields_ = [('AppContainerSid', ctypes.c_void_p), ('Capabilities', ctypes.c_void_p),
                ('CapabilityCount', wintypes.DWORD), ('Reserved', wintypes.DWORD)]


# ----------------------------------------------------------------------------------- pure helpers

def seconds_from_100ns(ticks: int) -> float:
    """FILETIME and job accounting durations count 100 ns."""
    return ticks / 1e7


def mib(byte_count: int) -> float:
    return byte_count / 2**20


def environment_block(env: dict[str, str]) -> str:
    """The Unicode environment block `CreateProcessW` takes: `KEY=value` entries separated by NUL, sorted
    case-insensitively as Windows requires, and ended by an empty entry."""
    entries = sorted((f'{key}={value}' for key, value in env.items()), key=lambda entry: entry.split('=', 1)[0].upper())
    return '\0'.join(entries) + '\0\0'


def busy_cores(before: tuple[int, int, int], after: tuple[int, int, int], cpus: int) -> float:
    """How many cores were busy between two `GetSystemTimes` readings (idle, kernel, user ticks). Kernel
    time includes idle time. This plays the part of the load average on the other systems: the number of
    cores that had work."""
    idle = after[0] - before[0]
    total = (after[1] - before[1]) + (after[2] - before[2])
    if total <= 0:
        return 0.0
    return max(0.0, min(1.0, (total - idle) / total)) * cpus


def power_source_from_line_status(line_status: int) -> str:
    """`SYSTEM_POWER_STATUS.ACLineStatus`: 0 battery, 1 mains, 255 unknown."""
    return {0: 'battery', 1: 'ac'}.get(line_status, 'unknown')


def parse_power_scheme(text: str) -> str:
    """The name from `powercfg /getactivescheme`: `Power Scheme GUID: <guid>  (Balanced)`."""
    start, end = text.rfind('('), text.rfind(')')
    return text[start + 1:end].strip() if 0 <= start < end else ''


# ---------------------------------------------------------------------------------------- the calls

@functools.cache
def api() -> SimpleNamespace:
    """The Win32 functions the harness calls, bound with exact argument types (handles are 64 bits wide).
    Raises OSError off Windows."""
    if not IS_WINDOWS:
        raise OSError('the Win32 API is only available on Windows')
    k32 = ctypes.WinDLL('kernel32', use_last_error=True)
    advapi = ctypes.WinDLL('advapi32', use_last_error=True)
    userenv = ctypes.WinDLL('userenv', use_last_error=True)

    def bind(dll, name, restype, *argtypes):
        function = getattr(dll, name)
        function.restype = restype
        function.argtypes = list(argtypes)
        return function

    H, D, P = wintypes.HANDLE, wintypes.DWORD, ctypes.c_void_p
    return SimpleNamespace(
        CloseHandle=bind(k32, 'CloseHandle', wintypes.BOOL, H),
        CreateJobObjectW=bind(k32, 'CreateJobObjectW', H, P, wintypes.LPCWSTR),
        SetInformationJobObject=bind(k32, 'SetInformationJobObject', wintypes.BOOL, H, ctypes.c_int, P, D),
        QueryInformationJobObject=bind(k32, 'QueryInformationJobObject', wintypes.BOOL, H, ctypes.c_int, P, D, P),
        AssignProcessToJobObject=bind(k32, 'AssignProcessToJobObject', wintypes.BOOL, H, H),
        TerminateJobObject=bind(k32, 'TerminateJobObject', wintypes.BOOL, H, wintypes.UINT),
        CreateProcessW=bind(k32, 'CreateProcessW', wintypes.BOOL, wintypes.LPCWSTR, wintypes.LPWSTR, P, P,
                            wintypes.BOOL, D, P, wintypes.LPCWSTR, P, P),
        ResumeThread=bind(k32, 'ResumeThread', D, H),
        TerminateProcess=bind(k32, 'TerminateProcess', wintypes.BOOL, H, wintypes.UINT),
        WaitForSingleObject=bind(k32, 'WaitForSingleObject', D, H, D),
        GetExitCodeProcess=bind(k32, 'GetExitCodeProcess', wintypes.BOOL, H, ctypes.POINTER(D)),
        OpenProcess=bind(k32, 'OpenProcess', H, D, wintypes.BOOL, D),
        SetHandleInformation=bind(k32, 'SetHandleInformation', wintypes.BOOL, H, D, D),
        GetProcessTimes=bind(k32, 'GetProcessTimes', wintypes.BOOL, H, P, P, P, P),
        GetProcessMemoryInfo=bind(k32, 'K32GetProcessMemoryInfo', wintypes.BOOL, H, P, D),
        InitializeProcThreadAttributeList=bind(k32, 'InitializeProcThreadAttributeList', wintypes.BOOL, P, D, D,
                                               ctypes.POINTER(ctypes.c_size_t)),
        UpdateProcThreadAttribute=bind(k32, 'UpdateProcThreadAttribute', wintypes.BOOL, P, D, ctypes.c_size_t, P,
                                       ctypes.c_size_t, P, P),
        DeleteProcThreadAttributeList=bind(k32, 'DeleteProcThreadAttributeList', None, P),
        GetSystemTimes=bind(k32, 'GetSystemTimes', wintypes.BOOL, P, P, P),
        GlobalMemoryStatusEx=bind(k32, 'GlobalMemoryStatusEx', wintypes.BOOL, P),
        GetSystemPowerStatus=bind(k32, 'GetSystemPowerStatus', wintypes.BOOL, P),
        LocalFree=bind(k32, 'LocalFree', P, P),
        FreeSid=bind(advapi, 'FreeSid', P, P),
        ConvertSidToStringSidW=bind(advapi, 'ConvertSidToStringSidW', wintypes.BOOL, P, ctypes.POINTER(wintypes.LPWSTR)),
        CreateAppContainerProfile=bind(userenv, 'CreateAppContainerProfile', ctypes.c_long, wintypes.LPCWSTR,
                                       wintypes.LPCWSTR, wintypes.LPCWSTR, P, D, ctypes.POINTER(P)),
        DeriveAppContainerSidFromAppContainerName=bind(userenv, 'DeriveAppContainerSidFromAppContainerName',
                                                       ctypes.c_long, wintypes.LPCWSTR, ctypes.POINTER(P)),
        DeleteAppContainerProfile=bind(userenv, 'DeleteAppContainerProfile', ctypes.c_long, wintypes.LPCWSTR),
    )


def raise_last_error(what: str) -> None:
    raise ctypes.WinError(ctypes.get_last_error(), what)


def process_memory(handle) -> tuple[int, int] | None:
    """(working set, peak working set) of the process in bytes; works on an exited process while its handle is open."""
    counters = PROCESS_MEMORY_COUNTERS()
    counters.cb = ctypes.sizeof(counters)
    if not api().GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
        return None
    return counters.WorkingSetSize, counters.PeakWorkingSetSize


def process_cpu_seconds(handle) -> tuple[float, float] | None:
    """(user, kernel) CPU seconds of the process alone."""
    created, exited, kernel, user = FILETIME(), FILETIME(), FILETIME(), FILETIME()
    if not api().GetProcessTimes(handle, *(ctypes.byref(t) for t in (created, exited, kernel, user))):
        return None
    return seconds_from_100ns(user.ticks), seconds_from_100ns(kernel.ticks)


def system_times() -> tuple[int, int, int]:
    """(idle, kernel, user) ticks since boot, all cores together."""
    idle, kernel, user = FILETIME(), FILETIME(), FILETIME()
    if not api().GetSystemTimes(ctypes.byref(idle), ctypes.byref(kernel), ctypes.byref(user)):
        raise_last_error('GetSystemTimes')
    return idle.ticks, kernel.ticks, user.ticks


def sample_busy_cores(cpus: int, window: float = 0.5) -> float:
    """Busy cores over a short window (this blocks for `window` seconds)."""
    before = system_times()
    time.sleep(window)
    return busy_cores(before, system_times(), cpus)


def memory_status() -> MEMORYSTATUSEX:
    status = MEMORYSTATUSEX()
    status.dwLength = ctypes.sizeof(status)
    if not api().GlobalMemoryStatusEx(ctypes.byref(status)):
        raise_last_error('GlobalMemoryStatusEx')
    return status


def power_source() -> str:
    status = SYSTEM_POWER_STATUS()
    if not api().GetSystemPowerStatus(ctypes.byref(status)):
        return 'unknown'
    return power_source_from_line_status(status.ACLineStatus)


def sid_string(sid) -> str:
    """`S-1-15-2-...` for a SID pointer."""
    text = wintypes.LPWSTR()
    if not api().ConvertSidToStringSidW(sid, ctypes.byref(text)):
        raise_last_error('ConvertSidToStringSidW')
    try:
        return text.value
    finally:
        api().LocalFree(ctypes.cast(text, ctypes.c_void_p))


# ------------------------------------------------------------------------------------ machine facts

def registry_value(path: str, name: str) -> str:
    """A string or number under HKEY_LOCAL_MACHINE, or ''."""
    if not IS_WINDOWS:
        return ''
    import winreg
    try:
        with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, path) as key:
            return str(winreg.QueryValueEx(key, name)[0]).strip()
    except OSError:
        return ''


def machine_facts() -> dict:
    """CPU name, installed RAM, OS build and active power plan."""
    cpu = registry_value(r'HARDWARE\DESCRIPTION\System\CentralProcessor\0', 'ProcessorNameString')
    windows = r'SOFTWARE\Microsoft\Windows NT\CurrentVersion'
    product = registry_value(windows, 'ProductName')
    display = registry_value(windows, 'DisplayVersion')
    build = registry_value(windows, 'CurrentBuildNumber')
    revision = registry_value(windows, 'UBR')
    try:
        memory = memory_status().ullTotalPhys
    except OSError:
        memory = 0
    try:
        scheme = parse_power_scheme(subprocess.run(['powercfg', '/getactivescheme'], capture_output=True,
                                                   text=True, timeout=30).stdout)
    except (OSError, subprocess.TimeoutExpired):
        scheme = ''
    return {'cpu': cpu, 'memory_bytes': memory,
            'os_version': ' '.join(part for part in (product, display) if part),
            'os_build': f'{build}.{revision}' if build and revision else build, 'power_plan': scheme}
