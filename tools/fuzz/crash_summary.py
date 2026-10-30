#!/usr/bin/env python3
"""Write what a fuzz run found to the job summary, so nobody downloads the artifact to see it.

    python3 tools/fuzz/crash_summary.py TARGET [--artifacts DIR] [--limit N]

For each file libFuzzer left in fuzz/artifacts/TARGET (crash-*, timeout-*, oom-*, leak-*,
slow-unit-*), replays it through the harness without libFuzzer (`fuzz/harness/examples/replay.rs`,
built with the debug assertions and overflow checks the fuzz build has) and appends a section to
$GITHUB_STEP_SUMMARY (stdout when that is unset): the panic message or how the replay ended, the
start of a backtrace, then the input as a hex dump and as escaped text, both cut to a sensible
length. The first LIMIT files get the full treatment, the rest are listed by name and size.
Always exits 0: the fuzz step has already failed the job.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from toolkit.repo import ROOT

PREFIXES = ("crash-", "timeout-", "oom-", "leak-", "slow-unit-")
HEX_BYTES = 512
TEXT_CHARS = 2000
MESSAGE_LINES = 25
BACKTRACE_LINES = 30
# The replay's address space, a little above libFuzzer's -rss_limit_mb=4096 in fuzz/run.sh.
ADDRESS_SPACE = 6 << 30
HARNESS = "fuzz/harness/Cargo.toml"
REPLAY_ENV = {
    "CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS": "true",
    "CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS": "true",
    "RUST_BACKTRACE": "1",
}


@dataclass
class Replay:
    """How replaying one input ended."""

    status: int | None  # exit status; negative for a signal; None for a timeout
    output: str  # stderr, then stdout


def hex_dump(data: bytes, limit: int = HEX_BYTES) -> str:
    """`xxd`-style lines (offset, 16 hex bytes, printable characters) for the first `limit` bytes."""
    lines = []
    for offset in range(0, min(len(data), limit), 16):
        chunk = data[offset : min(offset + 16, limit)]
        hexes = " ".join(f"{byte:02x}" for byte in chunk)
        text = "".join(chr(b) if 0x20 <= b < 0x7F else "." for b in chunk)
        lines.append(f"{offset:08x}  {hexes:<47}  {text}")
    if len(data) > limit:
        lines.append(f"... {len(data) - limit} more bytes")
    return "\n".join(lines)


def escape_text(data: bytes, limit: int = TEXT_CHARS) -> str:
    """The input as UTF-8 text with every control character and invalid byte escaped
    (`\\n`, `\\t`, `\\xNN`, `\\uNNNN`), cut after `limit` characters."""
    text = data.decode("utf-8", errors="surrogateescape")
    out = []
    for char in text:
        if char == "\n":
            out.append("\\n")
        elif char == "\t":
            out.append("\\t")
        elif char == "\r":
            out.append("\\r")
        elif char == "\\":
            out.append("\\\\")
        elif 0xDC80 <= ord(char) <= 0xDCFF:  # a byte that is not UTF-8
            out.append(f"\\x{ord(char) - 0xDC00:02x}")
        elif ord(char) < 0x20 or ord(char) == 0x7F:
            out.append(f"\\x{ord(char):02x}")
        else:
            out.append(char)
    escaped = "".join(out)
    if len(escaped) > limit:
        return escaped[:limit] + f"... ({len(escaped) - limit} more characters)"
    return escaped


def extract_panic(output: str) -> tuple[str, str]:
    """(the panic message, the start of the backtrace) from a replay's output; both empty when
    there is no panic. The message is the `thread ... panicked at` line and what follows it up to
    the backtrace hint."""
    lines = output.splitlines()
    for index, line in enumerate(lines):
        if re.match(r"thread '.*' (\(\d+\) )?panicked at ", line):
            break
    else:
        return "", ""
    message = []
    rest = index + 1
    while rest < len(lines) and len(message) < MESSAGE_LINES:
        if lines[rest].startswith(("note: run with", "stack backtrace:")):
            break
        message.append(lines[rest])
        rest += 1
    backtrace = []
    for tail in range(rest, len(lines)):
        if lines[tail].startswith("stack backtrace:"):
            backtrace = lines[tail : tail + BACKTRACE_LINES]
            break
    return "\n".join([lines[index], *message]), "\n".join(backtrace)


def describe_end(replay: Replay, unit_timeout: int) -> str:
    """One line on how the replay ended, for the cases without a panic message."""
    if replay.status is None:
        return f"the replay was still running after {unit_timeout} s (a hang)"
    if replay.status == 0:
        return "the replay finished without a panic (not reproduced outside the fuzzer)"
    if replay.status < 0:
        return f"the replay was killed by signal {-replay.status}"
    return f"the replay exited with status {replay.status}"


def fence(text: str) -> str:
    """`text` in a code fence longer than any run of backticks inside it."""
    longest = max((len(run) for run in re.findall(r"`+", text)), default=0)
    ticks = "`" * max(3, longest + 1)
    return f"{ticks}\n{text}\n{ticks}"


def section(path: Path, data: bytes, replay: Replay | None, unit_timeout: int = 0) -> str:
    parts = [f"#### `{path.name}` ({len(data)} bytes)"]
    if replay is not None:
        message, backtrace = extract_panic(replay.output)
        if message:
            parts.append(fence(message))
            if backtrace:
                parts.append(f"<details><summary>backtrace</summary>\n\n{fence(backtrace)}\n\n</details>")
        else:
            tail = "\n".join(replay.output.strip().splitlines()[-20:])
            parts.append(f"No panic message: {describe_end(replay, unit_timeout)}.")
            if tail:
                parts.append(fence(tail))
    parts.append("Input as a hex dump:\n\n" + fence(hex_dump(data) or "(empty)"))
    parts.append("Input as escaped text:\n\n" + fence(escape_text(data) or "(empty)"))
    return "\n\n".join(parts)


def findings(directory: Path) -> list[Path]:
    """The files libFuzzer saved, oldest first."""
    if not directory.is_dir():
        return []
    files = [p for p in directory.iterdir() if p.is_file() and p.name.startswith(PREFIXES)]
    return sorted(files, key=lambda p: (p.stat().st_mtime, p.name))


def limit_memory():
    import resource

    resource.setrlimit(resource.RLIMIT_AS, (ADDRESS_SPACE, ADDRESS_SPACE))


def unit_timeout_of(target: str) -> int:
    """The target's own timeout, from fuzz/targets.json (the one fuzz/run.sh uses)."""
    settings = json.loads((ROOT / "fuzz" / "targets.json").read_text())[target]
    return int(settings.get("timeout", 10))


def replay_input(target: str, path: Path, unit_timeout: int) -> Replay:
    env = {**os.environ, **REPLAY_ENV}
    command = [
        "cargo", "run", "--quiet", "--release", "--manifest-path", HARNESS,
        "--example", "replay", "--", target, str(path),
    ]  # fmt: skip
    try:
        done = subprocess.run(
            command,
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            errors="replace",
            timeout=unit_timeout + 60,
            preexec_fn=limit_memory if sys.platform == "linux" else None,
        )
    except subprocess.TimeoutExpired as error:
        partial = error.stderr or b""
        if isinstance(partial, bytes):
            partial = partial.decode("utf-8", "replace")
        return Replay(None, partial)
    return Replay(done.returncode, done.stderr + done.stdout)


def build_replay() -> bool:
    """Build the replay example once, so the per-input runs only measure the input."""
    env = {**os.environ, **REPLAY_ENV}
    command = ["cargo", "build", "--quiet", "--release", "--manifest-path", HARNESS, "--example", "replay"]
    return subprocess.run(command, cwd=ROOT, env=env).returncode == 0


def summarize(target: str, directory: Path, limit: int, replayer=None) -> str:
    files = findings(directory)
    if not files:
        return ""
    unit_timeout = unit_timeout_of(target)
    replayer = replayer or (lambda path: replay_input(target, path, unit_timeout))
    parts = [f"### fuzz `{target}`: {len(files)} finding(s)"]
    for path in files[:limit]:
        replay = replayer(path)
        parts.append(section(path, path.read_bytes(), replay, unit_timeout))
    if len(files) > limit:
        names = ", ".join(f"`{p.name}` ({p.stat().st_size} bytes)" for p in files[limit:])
        parts.append(f"Not replayed (only the first {limit} are): {names}")
    return "\n\n".join(parts) + "\n"


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("target")
    parser.add_argument("--artifacts", type=Path)
    parser.add_argument("--limit", type=int, default=3)
    args = parser.parse_args(argv)
    directory = args.artifacts or ROOT / "fuzz" / "artifacts" / args.target
    if findings(directory):
        build_replay()
    text = summarize(args.target, directory, args.limit)
    if not text:
        print(f"no findings in {directory}")
        return 0
    destination = os.environ.get("GITHUB_STEP_SUMMARY")
    if destination:
        with open(destination, "a", encoding="utf-8") as out:
            out.write(text)
    else:
        print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
