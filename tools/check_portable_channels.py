#!/usr/bin/env python3
"""Exercise native mappings across source-built native/interpreted processes."""
import argparse
import ctypes
import ctypes.util
import errno
import json
import os
from pathlib import Path
import re
import queue
import subprocess
import threading

from portable_toolchain import default_compiler, checked_compiler

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "target/portable-tests"
JOURNAL = OUTPUT / "shared-owned-names.txt"


def unlink_owned(name):
    if not re.fullmatch(r"(?:portable-[0-9]+-channel|jai-portable-[0-9]+-[0-9]+)", name):
        raise RuntimeError("invalid name in our shared-memory ownership journal")
    if os.name == "nt":
        # Named paging-file mappings disappear after every owning process exits.
        # Verify that fact; Windows has no POSIX unlink-by-name operation.
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        opened = kernel.OpenFileMappingW
        opened.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_wchar_p]
        opened.restype = ctypes.c_void_p
        handle = opened(4, 0, name)
        if handle:
            close = kernel.CloseHandle
            close.argtypes = [ctypes.c_void_p]
            close.restype = ctypes.c_int
            if not close(handle):
                raise ctypes.WinError(ctypes.get_last_error())
            raise RuntimeError(f"owned named mapping still exists after child teardown: {name}")
        if ctypes.get_last_error() != 2:
            raise ctypes.WinError(ctypes.get_last_error())
        return
    native = ctypes.CDLL(None, use_errno=True)
    if not hasattr(native, "shm_unlink"):
        native = ctypes.CDLL(ctypes.util.find_library("rt"), use_errno=True)
    unlink = native.shm_unlink
    unlink.argtypes = [ctypes.c_char_p]
    unlink.restype = ctypes.c_int
    if unlink(("/" + name).encode()) != 0 and ctypes.get_errno() != errno.ENOENT:
        raise OSError(ctypes.get_errno(), "owned shared-memory cleanup failed", name)


def checked(command, env):
    result = subprocess.run(command, cwd=ROOT, env=env, text=True,
                            capture_output=True, timeout=40)
    if result.returncode or result.stdout or result.stderr:
        raise RuntimeError(f"source build failed: {result.returncode}: {result.stdout}{result.stderr}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, default=default_compiler())
    parser.add_argument("--stdlib", type=Path, required=True)
    parser.add_argument("--native", action="store_true")
    args = parser.parse_args()
    compiler = checked_compiler(args.jaic, parser)
    OUTPUT.mkdir(parents=True, exist_ok=True)
    if JOURNAL.exists():
        for name in JOURNAL.read_text().splitlines():
            unlink_owned(name)
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve()))
    modes = [(False, False)]
    if args.native:
        modes += [(True, True), (True, False), (False, True)]
    for index, (reader_native, writer_native) in enumerate(modes):
        name = f"jai-portable-{os.getpid()}-{index}"
        with JOURNAL.open("a") as journal:
            journal.write(name + "\n")
            journal.flush()
        definition = "CHANNEL_NAME :: " + json.dumps(name) + ";"
        commands = []
        for role, native in [("reader", reader_native), ("writer", writer_native)]:
            source = ROOT / f"tests/portable-stdlib/ipc/{role}.jai"
            if native:
                executable = OUTPUT / f"ipc-{role}-{index}"
                if os.name == "nt":
                    executable = executable.with_suffix(".exe")
                checked([str(compiler), str(source), "-add", definition,
                         "-exe", str(executable)], env)
                commands.append([str(executable)])
            else:
                commands.append([str(compiler), "-interp", str(source), "-add", definition, "--"])
        reader = subprocess.Popen(commands[0], cwd=ROOT, env=env, text=True,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            ready = queue.Queue()
            handshake = threading.Thread(target=lambda: ready.put(reader.stdout.readline()), daemon=True)
            handshake.start()
            try:
                line = ready.get(timeout=20)
            except queue.Empty:
                reader.terminate()
                handshake.join(timeout=10)
                raise RuntimeError("reader failed to initialize shared mapping") from None
            if line != "ready\n":
                out, err = reader.communicate(timeout=10)
                raise RuntimeError(f"reader startup failed: {reader.returncode}: {out}{err}")
            writer = subprocess.run(commands[1], cwd=ROOT, env=env, text=True,
                                    capture_output=True, timeout=30)
            if writer.returncode or writer.stdout != "ok\n" or writer.stderr:
                raise RuntimeError(f"writer failed: {writer.returncode}: {writer.stdout}{writer.stderr}")
            out, err = reader.communicate(timeout=30)
            if reader.returncode or out != "ok\n" or err:
                raise RuntimeError(f"reader failed: {reader.returncode}: {out}{err}")
            print(f"passed 2048 interprocess packets: reader_native={reader_native}, writer_native={writer_native}", flush=True)
        finally:
            if reader.poll() is None:
                reader.terminate()
                reader.communicate(timeout=10)
            unlink_owned(name)


if __name__ == "__main__":
    main()
