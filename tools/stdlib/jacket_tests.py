"""Read the inputs of tests that `jacket test` runs, for suites that build such a test themselves.

A module listed under `[test] paths` in jacket.toml has no manifest. Its golden
test is a program `name.jai` with `name.stdout`, `name.stderr` and `name.exit` beside it and
`// test:` notes on top (docs/tools/jacket-test.md). A suite that builds that program with its own
flags uses `golden_row` for what the catalogue row used to say, and `run_exact` to run an
`@test` file (no `main`) through `jacket test` in the suite's own modes.
"""

import json
import os
import re
import shutil
import subprocess
import tomllib
from pathlib import Path

from toolkit.repo import ROOT, cargo_target_dir

NOTE = re.compile(r"^//\s*test:(.*)$")


def header_notes(source):
    """The `// test:` notes on top of `source`, as the words after `test:` on each line."""
    notes = []
    for line in Path(source).read_text(encoding="utf-8").splitlines():
        if not line.startswith("//"):
            break
        match = NOTE.match(line)
        if match:
            notes.extend(re.findall(r"\w+(?:\([^)]*\))?", match.group(1)))
    return notes


def golden_row(source):
    """`{"stdout", "stderr", "exit_code"}` of the golden test `source`, each key only when its
    file exists beside the program, like the manifest row it replaced."""
    source = Path(source)
    row = {}
    for key, suffix in (("stdout", ".stdout"), ("stderr", ".stderr")):
        file = source.with_suffix(suffix)
        if file.is_file():
            row[key] = file.read_text(encoding="utf-8")
    exit_file = source.with_suffix(".exit")
    if exit_file.is_file():
        row["exit_code"] = int(exit_file.read_text(encoding="utf-8").strip())
    return row


def expected_output(source):
    """(stdout, stderr, exit code) a golden program must end with; a missing file means empty
    output (or 0), which is stricter than `jacket test`, so a program that prints more fails here."""
    row = golden_row(source)
    return row.get("stdout", ""), row.get("stderr", ""), row.get("exit_code", 0)


def jacket_binary(compiler=None):
    """The `jacket` next to `compiler` (it starts the jaic beside itself), else on PATH, else in
    the cargo target directory."""
    if compiler is not None:
        beside = (
            Path(compiler)
            .resolve()
            .with_name("jacket" + (".exe" if os.name == "nt" else ""))
        )
        if beside.is_file():
            return beside
    found = shutil.which("jacket")
    if found:
        return Path(found)
    for profile in ("release", "debug"):
        candidate = (
            cargo_target_dir()
            / profile
            / ("jacket.exe" if os.name == "nt" else "jacket")
        )
        if candidate.is_file():
            return candidate
    raise FileNotFoundError(
        "no `jacket` next to the compiler, on PATH or in the cargo target directory"
    )


def test_names(source, jacket=None, env=None):
    """Names of the `@test` procedures in `source`'s file, as `jacket test --list` prints them."""
    source = Path(source).resolve()
    listing = subprocess.run(
        [
            str(jacket or jacket_binary()),
            "test",
            "--path",
            str(source.parent.parent),
            "--list",
            "--format",
            "json",
        ],
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    names = []
    for line in listing.splitlines():
        event = json.loads(line)
        if event.get("kind") == "test" and Path(event["file"]).resolve() == source:
            names.append(event["name"])
    if not names:
        raise ValueError(f"no @test procedures in {source}")
    return names


def exact_command(
    source, compiler, env, *, mode=(), jaic_flags=(), runner=None, jacket=None
):
    """The `jacket test` command that runs the `@test`s of `source` and nothing else.

    `mode` are `jacket test` options such as `["--interp"]` or `["--os", "wasm"]`; flags for jaic
    (`-O2`) go after `--`. `runner` runs programs built for another target. `env` must carry
    `JAIC_STDLIB`; `jacket` starts the `jaic` beside it, so pass the compiler under test."""
    jacket = jacket or jacket_binary(compiler)
    source = Path(source).resolve()
    command = [
        str(jacket),
        "test",
        "--path",
        str(source.parent.parent),
        "--exact",
        *test_names(source, jacket, env),
        *mode,
    ]
    if runner:
        command += ["--runner", runner]
    if jaic_flags:
        command += ["--", *jaic_flags]
    return command


OS_NAMES = ("linux", "macos", "windows", "wasm")
CPU_NAMES = ("x64", "arm64", "wasm")


def note_targets(notes, name):
    """The lowercase targets of the `name(...)` note among `notes` (`only(.LINUX,.X64)`)."""
    targets = []
    for note in notes:
        match = re.fullmatch(name + r"\(([^)]*)\)", note)
        if match:
            targets += [
                part.strip().lstrip(".").lower()
                for part in match.group(1).split(",")
                if part.strip()
            ]
    return targets


def runs_on(notes, os_name, cpu):
    """Whether a native run of a test with these `// test:` notes is allowed on `os_name` and
    `cpu`: every kind of target `only` names must match, and no `skip_on` target may."""

    def matches(target):
        system, dash, rest = target.partition("-")
        if dash:
            return system == os_name and rest in (cpu, "native")
        return target in (os_name, cpu, "native")

    only = note_targets(notes, "only")
    for names in (OS_NAMES, CPU_NAMES, ("interp", "native")):
        named = [t for t in only if t in names or (names is OS_NAMES and "-" in t)]
        if named and not any(matches(t) for t in named):
            return False
    return not any(matches(t) for t in note_targets(notes, "skip_on"))


# Programs that build and run other programs while compiling, which a type check cannot do.
COMPILE_TIME_DRIVERS = frozenset({"stdlib/Autorun/tests/basics.jai"})


def golden_sources(root=ROOT):
    """The golden test programs of every folder under `[test] paths` in jacket.toml: the
    `tests/*.jai` files with an expected-output file beside them that are not compile-fail tests."""
    config = root / "jacket.toml"
    paths = tomllib.loads(config.read_text(encoding="utf-8")).get("test", {}).get("paths", [])
    for path in sorted(paths):
        for source in sorted((root / path / "tests").glob("*.jai")):
            expected = any(
                source.with_suffix(suffix).is_file()
                for suffix in (".stdout", ".stderr", ".exit")
            )
            notes = " ".join(header_notes(source))
            # Tests that expect a compile-time failure or only check themselves say so in a note.
            driver = source.relative_to(root).as_posix() in COMPILE_TIME_DRIVERS
            if expected and not driver and not re.search(r"compile_fail|should_fail|check_only", notes):
                yield source


def cross_check(compiler, os_name, cpu, env=None):
    """Type-check (`jaic -check`) every golden test that runs on `os_name`/`cpu`, without running
    anything. Returns (checked count, failure texts)."""
    flags = ["-target", "wasm64-unknown-wasi"] if os_name == "wasm" else ["-os", os_name, "-cpu", cpu]
    checked, failures = 0, []
    for source in golden_sources():
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
    "expected_output",
    "exact_command",
    "golden_row",
    "golden_sources",
    "header_notes",
    "jacket_binary",
    "runs_on",
    "test_names",
]
