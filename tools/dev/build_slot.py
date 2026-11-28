#!/usr/bin/env python3
"""A machine-wide build slot: at most N heavy cargo jobs run at once, across every checkout and session.

`build_slot.py run -- <command>...` waits for a free slot, runs the command and releases the slot
when it exits, including on Ctrl-C. `tools/dev/cargo-slot <cargo args>` is `run -- cargo <args>`.
Python code takes a slot with `with slot():`. See docs/tools/development.md#build-slot.

A slot is an `flock` on one of `slot-0` .. `slot-<N-1>` under `<cache>/jacket/build-slots/`
(`$XDG_CACHE_HOME`, else `~/.cache`). The kernel drops the lock when its holder dies, so a crashed
job never leaves a slot taken. The holder writes its pid and command into the slot file, which is
what a waiting job prints. Waiting has no timeout.
"""

import contextlib
import fcntl
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

SLOTS = 2
# Set for the children of a job that holds a slot, so a nested job (a hook run by a wrapped
# command) does not wait for a second slot and deadlock when all of them are held.
HELD = "JACKET_BUILD_SLOT_HELD"
POLL_SECONDS = 0.5


def slot_count() -> int:
    """`JACKET_BUILD_SLOTS`, else 2; exits when it is not a positive number."""
    raw = os.environ.get("JACKET_BUILD_SLOTS")
    if raw is None:
        return SLOTS
    if not raw.isdigit() or int(raw) < 1:
        raise SystemExit(f"JACKET_BUILD_SLOTS must be a positive number, not {raw!r}")
    return int(raw)


def slot_dir() -> Path:
    """The folder holding the slot files; `JACKET_BUILD_SLOT_DIR` replaces it (tests use this)."""
    override = os.environ.get("JACKET_BUILD_SLOT_DIR")
    if override:
        return Path(override)
    return Path(os.environ.get("XDG_CACHE_HOME") or Path.home() / ".cache") / "jacket" / "build-slots"


def holder(path: Path) -> str:
    try:
        text = path.read_text().strip()
    except OSError:
        text = ""
    return text or "an unknown job"


def try_take(directory: Path, count: int, who: str):
    """The open file of the first free slot, locked and labelled `who`, or None when all are held."""
    for index in range(count):
        handle = open(directory / f"slot-{index}", "a+")
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError:
            handle.close()
            continue
        handle.seek(0)
        handle.truncate()
        handle.write(who)
        handle.flush()
        return handle
    return None


@contextlib.contextmanager
def slot(who: str | None = None, count: int | None = None, directory: Path | None = None):
    """Holds a build slot for the block, waiting without limit for one to free up.

    `who` labels the holder for waiting jobs (default: pid and command line).
    """
    if os.environ.get(HELD):
        yield
        return
    directory = directory or slot_dir()
    count = count or slot_count()
    directory.mkdir(parents=True, exist_ok=True)
    who = who or f"pid {os.getpid()}: {' '.join(sys.argv)}"
    handle = try_take(directory, count, who)
    if handle is None:
        holders = "; ".join(holder(directory / f"slot-{i}") for i in range(count))
        print(f"waiting for a build slot (all {count} held by {holders})", file=sys.stderr, flush=True)
        while handle is None:
            time.sleep(POLL_SECONDS)
            handle = try_take(directory, count, who)
    os.environ[HELD] = "1"
    try:
        yield
    finally:
        os.environ.pop(HELD, None)
        handle.close()


def run(command: list[str]) -> int:
    """Runs `command` in a build slot and returns its exit status (128 + signal when killed).

    Ctrl-C reaches the command itself, since the terminal signals the whole foreground group;
    SIGTERM and SIGHUP are passed on, so the slot is never freed while its job still runs.
    """
    with slot():
        try:
            child = subprocess.Popen(command)
        except FileNotFoundError:
            raise SystemExit(f"build-slot: {command[0]}: command not found")
        for number in (signal.SIGTERM, signal.SIGHUP):
            signal.signal(number, lambda received, _frame: child.send_signal(received))
        while True:
            try:
                status = child.wait()
                break
            except KeyboardInterrupt:
                continue
    return 128 - status if status < 0 else status


def main() -> int:
    args = sys.argv[1:]
    if args[:1] == ["run"]:
        args = args[1:]
    if args[:1] == ["--"]:
        args = args[1:]
    if not args:
        print("usage: build_slot.py run -- <command>...", file=sys.stderr)
        return 2
    return run(args)


if __name__ == "__main__":
    sys.exit(main())
