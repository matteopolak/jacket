"""What a compiler produced, as opposed to how fast it produced it: the size of a built executable, the
machine instructions in its code, and how many instructions and cycles it retires when it runs.

A compiler that builds faster by emitting worse code shows up here as more code or more retired
instructions for the same program."""
from __future__ import annotations

import platform
import re
import shutil
import statistics
import subprocess
from pathlib import Path

# `llvm-objdump -d` / `objdump -d` instruction lines: an address, a colon, padding that ends in a tab,
# then the mnemonic. Symbol lines (`<address> <name>:`) have no colon straight after the address.
INSTRUCTION = re.compile(r'^\s*[0-9a-f]+:[ \t]*\t\S', re.MULTILINE)
# `/usr/bin/time -l` on macOS reports the process's hardware counters.
TIME_COUNTER = re.compile(r'^\s*(\d+)\s+(instructions retired|cycles elapsed)\s*$', re.MULTILINE)
# `perf stat -x,` on Linux: value,unit,event,...
PERF_COUNTER = re.compile(r'^(\d+),[^,]*,(instructions|cycles)\b', re.MULTILINE)
# A program that does nothing: its counters are the cost of starting a process, subtracted from each count.
NOTHING = '/usr/bin/true'


def disassembler() -> list[str] | None:
    """The command that disassembles an executable's code, or None when there is none."""
    if platform.system() == 'Darwin' and shutil.which('xcrun'):
        return ['xcrun', 'llvm-objdump', '-d', '--no-show-raw-insn']
    if shutil.which('objdump'):
        return ['objdump', '-d', '--no-show-raw-insn']
    return None


def static_facts(executable: Path) -> dict:
    """File size and the number of machine instructions in the executable's code sections."""
    facts: dict = {'file_bytes': executable.stat().st_size}
    command = disassembler()
    if command:
        done = subprocess.run([*command, str(executable)], capture_output=True, text=True, timeout=600)
        if done.returncode == 0:
            facts['instructions'] = len(INSTRUCTION.findall(done.stdout))
    return facts


def counter_command(command: list[str]) -> tuple[list[str], re.Pattern] | None:
    """`command` wrapped so the run reports retired instructions and cycles, or None when the system
    has no way to read them."""
    if platform.system() == 'Darwin' and Path('/usr/bin/time').exists():
        return ['/usr/bin/time', '-l', *command], TIME_COUNTER
    if platform.system() == 'Linux' and shutil.which('perf'):
        return ['perf', 'stat', '-x,', '-e', 'instructions,cycles', '--', *command], PERF_COUNTER
    return None


def parse_counters(stderr: str, pattern: re.Pattern) -> dict[str, int]:
    found = {}
    for value, name in pattern.findall(stderr):
        key = 'retired_instructions' if name.startswith('instructions') else 'cycles'
        found[key] = int(value)
    return found


def median_counters(runs: list[dict[str, int]]) -> dict[str, int]:
    """The median of each counter over the runs that reported it."""
    keys = {key for run in runs for key in run}
    return {key: int(statistics.median(run[key] for run in runs if key in run)) for key in sorted(keys)}


def subtract(counters: dict[str, int], baseline: dict[str, int]) -> dict[str, int]:
    """`counters` less the start-up cost in `baseline`, never below zero."""
    return {key: max(0, value - baseline.get(key, 0)) for key, value in counters.items()}
