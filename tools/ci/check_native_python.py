#!/usr/bin/env python3
"""Fail when Python on a Windows runner is emulated or the runner's CPU is not the expected one.

    python tools/ci/check_native_python.py 0xaa64

`EXPECTED` is the native machine type: 0x8664 for x64, 0xaa64 for ARM64.
"""

import argparse
import ctypes
import json
import platform
import sys
import sysconfig


def machines():
    """(process machine, native machine) from `IsWow64Process2`; the process value is 0 when Python is native."""
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.GetCurrentProcess.restype = ctypes.c_void_p
    kernel.IsWow64Process2.argtypes = [
        ctypes.c_void_p,
        ctypes.POINTER(ctypes.c_ushort),
        ctypes.POINTER(ctypes.c_ushort),
    ]
    kernel.IsWow64Process2.restype = ctypes.c_int
    process, native = ctypes.c_ushort(), ctypes.c_ushort()
    if not kernel.IsWow64Process2(
        kernel.GetCurrentProcess(), ctypes.byref(process), ctypes.byref(native)
    ):
        raise ctypes.WinError(ctypes.get_last_error())
    return process.value, native.value


def problems(process, native, expected):
    """Errors for an emulated Python or an unexpected native machine, for example `problems(0, 0xaa64, 0x8664)`."""
    errors = []
    if process != 0:
        errors.append("Python must run natively rather than under emulation")
    if native != expected:
        errors.append(f"unexpected native machine {native:#x}, wanted {expected:#x}")
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("expected", help="native machine type, such as 0xaa64")
    expected = int(parser.parse_args().expected, 0)
    process, native = machines()
    print(
        json.dumps(
            {
                "python": sys.executable,
                "platform": sysconfig.get_platform(),
                "machine": platform.machine(),
                "process_machine": hex(process),
                "native_machine": hex(native),
            }
        )
    )
    errors = problems(process, native, expected)
    if errors:
        sys.exit("\n".join(errors))


if __name__ == "__main__":
    main()
