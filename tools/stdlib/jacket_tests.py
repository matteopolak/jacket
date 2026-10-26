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


__all__ = [
    "expected_output",
    "exact_command",
    "golden_row",
    "header_notes",
    "jacket_binary",
    "test_names",
]
