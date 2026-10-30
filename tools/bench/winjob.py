"""Run and measure one process tree on Windows with a Job Object: the same numbers `measure.py` takes from
`wait4` elsewhere.

- CPU time: the job's accounting, user plus kernel time of the process and every descendant (a linker,
  a helper) that ran in it, including those that already exited.
- Peak memory: the largest peak *working set* (resident memory) of any single process in the job. The
  main process's own peak is read after it exits; descendants are sampled while the job runs, so a child
  that starts and ends between two samples (10 ms apart) can be missed. The job's peak *commit* charge
  (private bytes: not comparable to the working set, which also counts shared pages) is reported separately as `commit_mib`.
  macOS and Linux report `ru_maxrss`, the peak resident set of the largest single process.
- Timeouts and stragglers: `TerminateJobObject` kills the whole tree, on timeout and again once the
  command exits, as `killpg` does elsewhere.

A command can also be launched into an AppContainer (`winsandbox.py`): the process is created suspended
with the container's security capabilities, put in the job, then resumed.
"""
from __future__ import annotations

import ctypes
import os
import subprocess
import time
from dataclasses import dataclass
from typing import IO

from bench import winapi
from bench.winapi import api

POLL_SECONDS = 0.01


@dataclass
class Measured:
    wall: float
    cpu: float
    rss_mib: float
    commit_mib: float
    returncode: int
    timed_out: bool


class Job:
    """A Job Object whose processes die when the last handle to it closes."""

    def __init__(self) -> None:
        self.handle = api().CreateJobObjectW(None, None)
        if not self.handle:
            winapi.raise_last_error('CreateJobObject')
        limits = winapi.JOBOBJECT_EXTENDED_LIMIT_INFORMATION()
        limits.BasicLimitInformation.LimitFlags = winapi.JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        if not api().SetInformationJobObject(self.handle, winapi.JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                                             ctypes.byref(limits), ctypes.sizeof(limits)):
            winapi.raise_last_error('SetInformationJobObject')

    def assign(self, process_handle) -> None:
        if not api().AssignProcessToJobObject(self.handle, process_handle):
            winapi.raise_last_error('AssignProcessToJobObject')

    def cpu_seconds(self) -> tuple[float, float]:
        """(user, kernel) seconds of every process the job has run."""
        info = winapi.JOBOBJECT_BASIC_ACCOUNTING_INFORMATION()
        if not api().QueryInformationJobObject(self.handle, winapi.JOB_OBJECT_BASIC_ACCOUNTING_INFORMATION,
                                               ctypes.byref(info), ctypes.sizeof(info), None):
            return 0.0, 0.0
        return winapi.seconds_from_100ns(info.TotalUserTime), winapi.seconds_from_100ns(info.TotalKernelTime)

    def peak_commit_mib(self) -> float:
        info = winapi.JOBOBJECT_EXTENDED_LIMIT_INFORMATION()
        if not api().QueryInformationJobObject(self.handle, winapi.JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                                               ctypes.byref(info), ctypes.sizeof(info), None):
            return 0.0
        return winapi.mib(info.PeakJobMemoryUsed)

    def process_ids(self) -> list[int]:
        slots = 64
        # JOBOBJECT_BASIC_PROCESS_ID_LIST: two DWORD counts, then the ids as pointer-sized integers.
        buffer = (ctypes.c_byte * (8 + slots * ctypes.sizeof(ctypes.c_size_t)))()
        if not api().QueryInformationJobObject(self.handle, winapi.JOB_OBJECT_BASIC_PROCESS_ID_LIST,
                                               buffer, ctypes.sizeof(buffer), None):
            return []
        count = ctypes.cast(buffer, ctypes.POINTER(ctypes.c_uint32))[1]
        ids = (ctypes.c_size_t * slots).from_buffer(buffer, 8)
        return [ids[i] for i in range(min(count, slots))]

    def peak_working_set_of_members(self) -> int:
        """The largest peak working set, in bytes, among the processes in the job right now."""
        largest = 0
        for pid in self.process_ids():
            handle = api().OpenProcess(winapi.PROCESS_QUERY_LIMITED_INFORMATION | winapi.PROCESS_VM_READ,
                                       False, pid)
            if not handle:
                continue
            try:
                memory = winapi.process_memory(handle)
                if memory:
                    largest = max(largest, memory[1])
            finally:
                api().CloseHandle(handle)
        return largest

    def terminate(self) -> None:
        api().TerminateJobObject(self.handle, 1)

    def close(self) -> None:
        if self.handle:
            api().CloseHandle(self.handle)
            self.handle = None


# ------------------------------------------------------------------------------------- launching

def inheritable_handle(file: IO) -> int:
    """The OS handle of an open file, marked so a child process can inherit it."""
    import msvcrt
    handle = msvcrt.get_osfhandle(file.fileno())
    api().SetHandleInformation(handle, winapi.HANDLE_FLAG_INHERIT, winapi.HANDLE_FLAG_INHERIT)
    return handle


@dataclass
class Spawned:
    process: int
    thread: int
    pid: int


def launch(command: list[str], cwd, env: dict[str, str] | None, stdin: IO, stdout: IO, stderr: IO,
           container_sid=None) -> Spawned:
    """Create `command` suspended, with the three files as its standard streams. With `container_sid` (a SID
    pointer) the process runs in that AppContainer and holds no capabilities. The caller resumes it."""
    lib = api()
    handles = [inheritable_handle(f) for f in (stdin, stdout, stderr)]
    startup = winapi.STARTUPINFOEXW()
    startup.StartupInfo.cb = ctypes.sizeof(startup)
    startup.StartupInfo.dwFlags = winapi.STARTF_USESTDHANDLES
    startup.StartupInfo.hStdInput, startup.StartupInfo.hStdOutput, startup.StartupInfo.hStdError = handles

    # The attribute list: only the three stream handles are inherited, and the container if there is one.
    attribute_count = 2 if container_sid else 1
    size = ctypes.c_size_t(0)
    lib.InitializeProcThreadAttributeList(None, attribute_count, 0, ctypes.byref(size))
    attributes = ctypes.create_string_buffer(size.value)
    if not lib.InitializeProcThreadAttributeList(attributes, attribute_count, 0, ctypes.byref(size)):
        winapi.raise_last_error('InitializeProcThreadAttributeList')
    startup.lpAttributeList = ctypes.cast(attributes, ctypes.c_void_p)
    unique = sorted(set(handles))
    inherited = (ctypes.c_void_p * len(unique))(*unique)
    capabilities = winapi.SECURITY_CAPABILITIES()
    try:
        if not lib.UpdateProcThreadAttribute(attributes, 0, winapi.PROC_THREAD_ATTRIBUTE_HANDLE_LIST, inherited,
                                             ctypes.sizeof(inherited), None, None):
            winapi.raise_last_error('UpdateProcThreadAttribute(handle list)')
        if container_sid:
            capabilities.AppContainerSid = container_sid
            if not lib.UpdateProcThreadAttribute(attributes, 0, winapi.PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
                                                 ctypes.byref(capabilities), ctypes.sizeof(capabilities), None, None):
                winapi.raise_last_error('UpdateProcThreadAttribute(security capabilities)')
        block = None
        if env is not None:
            text = winapi.environment_block(env)
            block = ctypes.create_string_buffer(len(text) * 2)
            ctypes.memmove(block, text.encode('utf-16-le'), len(text) * 2)
        info = winapi.PROCESS_INFORMATION()
        flags = (winapi.CREATE_SUSPENDED | winapi.EXTENDED_STARTUPINFO_PRESENT | winapi.CREATE_NO_WINDOW
                 | winapi.CREATE_UNICODE_ENVIRONMENT)
        line = ctypes.create_unicode_buffer(subprocess.list2cmdline([str(part) for part in command]))
        if not lib.CreateProcessW(None, line, None, None, True, flags, block, str(cwd) if cwd else None,
                                  ctypes.byref(startup), ctypes.byref(info)):
            winapi.raise_last_error(f'CreateProcess({command[0]})')
        return Spawned(info.hProcess, info.hThread, info.dwProcessId)
    finally:
        lib.DeleteProcThreadAttributeList(attributes)


def run(command: list[str], cwd, env: dict[str, str] | None, stdin: IO, stdout: IO, stderr: IO, timeout: float,
        container_sid=None) -> Measured:
    """Run `command` in a job of its own to completion, or kill the tree after `timeout` seconds."""
    lib = api()
    job = Job()
    try:
        start = time.perf_counter()
        spawned = launch(command, cwd, env, stdin, stdout, stderr, container_sid)
        try:
            job.assign(spawned.process)
            lib.ResumeThread(spawned.thread)
            deadline = start + timeout
            timed_out = False
            sampled = 0
            while lib.WaitForSingleObject(spawned.process, int(POLL_SECONDS * 1000)) == winapi.WAIT_TIMEOUT:
                sampled = max(sampled, job.peak_working_set_of_members())
                if time.perf_counter() > deadline:
                    job.terminate()
                    lib.WaitForSingleObject(spawned.process, 10_000)
                    timed_out = True
                    break
            wall = time.perf_counter() - start
            code = ctypes.c_uint32(1)
            lib.GetExitCodeProcess(spawned.process, ctypes.byref(code))
            own = winapi.process_memory(spawned.process)
            peak = max(sampled, own[1] if own else 0, job.peak_working_set_of_members())
            user, kernel = job.cpu_seconds()
            commit = job.peak_commit_mib()
            job.terminate()  # anything the command left behind
            return Measured(wall=wall, cpu=user + kernel, rss_mib=winapi.mib(peak), commit_mib=commit,
                            returncode=code.value, timed_out=timed_out)
        finally:
            lib.CloseHandle(spawned.thread)
            lib.CloseHandle(spawned.process)
    finally:
        job.close()


def open_null() -> IO:
    return open(os.devnull, 'rb')
