"""Find `jacket`, read the `// test:` notes of a golden test, and type-check the tests for another target.

A golden test is a program `name.jai` with `name.stdout`, `name.stderr` and `name.exit` beside it and
`// test:` notes on top (docs/tools/jacket-test.md).
"""

import re
import subprocess
from pathlib import Path

from toolkit.cli import default_jaic
from toolkit.host import exe_name
from toolkit.repo import ROOT

NOTE = re.compile(r"^//\s*test:(.*)$")


def header_notes(source):
    """The `// test:` notes on top of `source`, as the words after `test:` on each line."""
    notes = []
    for line in Path(source).read_text(encoding="utf-8").splitlines():
        if not line.startswith("//"):
            break
        match = NOTE.match(line)
        if match:
            notes.extend(re.findall(r'\w+(?:\((?:"(?:[^"\\]|\\.)*"|[^)"])*\))?', match.group(1)))
    return notes


def jacket_binary(compiler=None, jacket=None):
    """The `jacket` to run: `jacket` when given (the `--jacket` flag), else the one beside `compiler` (it starts
    the jaic beside itself; default compiler: the debug jaic of the cargo target directory). Exits nowhere
    else: PATH is not searched, so the tests never run an installed jacket by accident.

    For example `jacket_binary(Path("target/release/jaic"))` is `target/release/jacket`."""
    if jacket is not None:
        path = Path(jacket)
    else:
        path = Path(compiler or default_jaic()).resolve().with_name(exe_name("jacket"))
    if not path.is_file():
        raise FileNotFoundError(
            f"{path} does not exist: build it with `cargo build -p jacket -p jaic-cli`, or pass --jacket"
        )
    return path


OS_NAMES = ("linux", "macos", "windows", "wasm")
CPU_NAMES = ("x64", "arm64", "wasm")


def note_calls(notes, name):
    """The `name(...)` calls among `notes` as (names, mode, cpu): the lowercase positional
    targets, and the `mode =` and `cpu =` values (None when absent)."""
    calls = []
    for note in notes:
        match = re.fullmatch(name + r"\((.*)\)", note, re.DOTALL)
        if not match:
            continue
        names, narrow = [], {"mode": None, "cpu": None}
        # A `reason = "text"` explains the call; its text may hold commas and parentheses.
        arguments = re.sub(r'reason\s*=\s*"(?:[^"\\]|\\.)*"', "", match.group(1))
        for part in arguments.split(","):
            part = part.strip()
            if "=" in part:
                key, _, value = part.partition("=")
                narrow[key.strip()] = value.strip().lstrip(".").lower()
            elif part:
                names.append(part.lstrip(".").lower())
        calls.append((names, narrow["mode"], narrow["cpu"]))
    return calls


def runs_on(notes, os_name, cpu):
    """Whether a native run of a test with these `// test:` notes is allowed on `os_name` and
    `cpu`: every `only` call must pass (each kind of target it names must match, and its `mode =`
    and `cpu =`), and no `skip_on` call may match (any name, and its `mode =` and `cpu =`)."""

    def named(target):
        return target in (os_name, cpu, "native")

    def narrowed(mode, wanted_cpu):
        return mode in (None, "native") and wanted_cpu in (None, cpu)

    for names, mode, wanted_cpu in note_calls(notes, "only"):
        for kind in (OS_NAMES, CPU_NAMES, ("interp", "native")):
            of_kind = [t for t in names if t in kind]
            if of_kind and not any(named(t) for t in of_kind):
                return False
        if not narrowed(mode, wanted_cpu):
            return False
    for names, mode, wanted_cpu in note_calls(notes, "skip_on"):
        if (not names or any(named(t) for t in names)) and narrowed(mode, wanted_cpu):
            return False
    return True


# Programs that build and run other programs while compiling, which a type check cannot do.
COMPILE_TIME_DRIVERS = frozenset({"stdlib/Autorun/tests/basics.jai"})


def jacket_test_roots(root=ROOT, compiler=None, jacket=None):
    """The folders `jacket test` runs in the project at `root` (`[test] paths` in jacket.toml,
    patterns expanded), as absolute paths."""
    result = subprocess.run(
        [str(jacket_binary(compiler, jacket)), "test", "--roots"],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    )
    return [Path(line) for line in result.stdout.splitlines() if line]


def golden_sources(root=ROOT, compiler=None, jacket=None):
    """The golden test programs of every test root: the `tests/*.jai` files with an
    expected-output file beside them that are not compile-fail tests."""
    for folder in sorted(jacket_test_roots(root, compiler, jacket)):
        for source in sorted((folder / "tests").glob("*.jai")):
            expected = any(
                source.with_suffix(suffix).is_file()
                for suffix in (".stdout", ".stderr", ".exit")
            )
            notes = " ".join(header_notes(source))
            # Tests that expect a compile-time failure or only check themselves say so in a note.
            driver = source.relative_to(root).as_posix() in COMPILE_TIME_DRIVERS
            if expected and not driver and not re.search(r"compile_fail|should_fail|check_only", notes):
                yield source


def cross_check(compiler, os_name, cpu, env=None, jacket=None):
    """Type-check (`jaic -check`) every golden test that runs on `os_name`/`cpu`, without running
    anything. Returns (checked count, failure texts)."""
    flags = ["-target", "wasm64-unknown-wasi"] if os_name == "wasm" else ["-os", os_name, "-cpu", cpu]
    checked, failures = 0, []
    for source in golden_sources(compiler=compiler, jacket=jacket):
        if not runs_on(header_notes(source), os_name, "wasm" if os_name == "wasm" else cpu):
            continue
        result = subprocess.run(
            [str(compiler), source.name, "-check", "-quiet", "-no_color", *flags],
            cwd=source.parent,
            env=env,
            capture_output=True,
            text=True,
        )
        checked += 1
        if result.returncode:
            failures.append(f"{source.relative_to(ROOT)}\n{result.stdout}{result.stderr}")
    return checked, failures


__all__ = [
    "cross_check",
    "golden_sources",
    "header_notes",
    "jacket_binary",
    "runs_on",
    "jacket_test_roots",
]
