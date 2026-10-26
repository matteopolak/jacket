#!/usr/bin/env python3
"""Turns a stdlib module's manifest.json cases into `jacket test` files.

    PYTHONPATH=tools python3 tools/stdlib/migrate_to_jacket_test.py stdlib/Sort [--dry-run]

Each case stays where it is and becomes a golden test (a program with `.stdout`/`.stderr`/`.exit` files
beside it) or, for a check-fail row, a compile-fail test (`// test: compile_fail` and a `.stderr` that
`jacket test --accept --exact NAME` fills in), with the row's gating as `// test:` notes on the file's
first line. The module is added to `[test] paths` in
jacket.toml and its manifest is deleted. The migration section of docs/tools/jacket-test.md
says what every row key becomes.

Cases it cannot express are listed and the module is left untouched: run it again after
converting those by hand (or leaving the module on the old runner).
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

from stdlib.stdlib_test_catalog import ROW_KEYS
from toolkit.repo import ROOT

# Keys with a direct equivalent, or that carry no behaviour of their own.
HANDLED = {
    "name",
    "source",
    "stdout",
    "stderr",
    "exit_code",
    "error_contains",
    "mode",
    "diagnostic_contains",
    "positive_baseline",
    "required_os",
    "required_cpu",
    "unsupported_modes",
    "unsupported_platforms",
    "required_capabilities",
    "required_browser_host_capabilities",
    "arguments",
    "input",
    "requires_display",
    "window_lease",
    "source_check_only",
    "warning_contains",
}
WASM_MODES = {"wasm-interp", "wasm-native"}
SCRATCH_INPUTS = {"scratch-file": "$TEMP/data", "scratch-directory": "$TEMP"}


def as_list(value):
    if value is None:
        return []
    return [value] if isinstance(value, str) else list(value)


def notes_of(row):
    """The `// test:` notes of a row, and the reasons it cannot be expressed."""
    notes, problems = [], []
    for key in sorted(set(row) - HANDLED):
        problems.append(f"row key `{key}` has no jacket test equivalent")
    unsupported = set(row.get("unsupported_modes", []))
    if unsupported == WASM_MODES:
        notes.append("skip_on(.WASM)")
    elif unsupported:
        problems.append(f"unsupported_modes {sorted(unsupported)} is not all of wasm")
    if row.get("required_os"):
        notes.append(
            "only("
            + ",".join("." + os.upper() for os in as_list(row["required_os"]))
            + ")"
        )
    if row.get("required_cpu"):
        notes.append(
            "only("
            + ",".join("." + cpu.upper() for cpu in as_list(row["required_cpu"]))
            + ")"
        )
    for platform in row.get("unsupported_platforms", []):
        notes.append("skip_on(." + platform.upper() + ")")
    if row.get("requires_display"):
        notes.append("needs(.DISPLAY)")
    if row.get("window_lease"):
        notes.append("serial")
    if row.get("source_check_only"):
        notes.append("check_only")
    if "warning_contains" in row:
        notes.append(f'warns("{row["warning_contains"]}")')
    if row.get("mode") == "check-fail":
        notes.append("compile_fail")
    if row.get("mode") == "fault" or (
        row.get("error_contains") and row.get("mode") != "check-fail"
    ):
        notes.append(f'should_fail("{row["error_contains"]}")')
    if "arguments" in row:
        notes.append("args(" + ",".join(f'"{a}"' for a in row["arguments"]) + ")")
    elif "input" in row:
        if row["input"] not in SCRATCH_INPUTS:
            problems.append(
                f"input `{row['input']}` needs fixtures only the old runner makes"
            )
        else:
            notes.append(f'args("{SCRATCH_INPUTS[row["input"]]}")')
    if "fixture_libraries" in row:
        problems.append("fixture_libraries")
    if row.get("mode") == "check-fail" and not row.get("diagnostic_contains"):
        problems.append("check-fail without a diagnostic")
    return notes, problems


def plan(directory):
    """(cases, problems): cases are (name, row, source path); sources stay where they are."""
    tests = directory / "tests"
    rows = {}
    manifest = tests / "manifest.json"
    if manifest.is_file():
        for row in json.loads(manifest.read_text()):
            rows[row["name"]] = row
    names = {p.stem for p in tests.glob("*.jai")} | set(rows)
    cases, problems = [], []
    for name in sorted(names):
        row = rows.get(name, {})
        unknown = set(row) - ROW_KEYS
        if unknown:
            problems.append(f"{name}: unknown keys {sorted(unknown)}")
        source = tests / row.get("source", name + ".jai")
        if "disabled" in row:
            problems.append(f"{name}: disabled")
            continue
        if not source.is_file():
            problems.append(f"{name}: missing {source}")
            continue
        _, why = notes_of(row)
        problems.extend(f"{name}: {w}" for w in why)
        cases.append((name, row, source))
    return cases, problems


def write_expected(row, source):
    """The files beside `source` that make it a golden (or compile-fail) test."""
    wrote = False
    if row.get("mode") == "check-fail":
        # `jacket test --accept` fills in the diagnostics; an empty file makes the test discoverable.
        source.with_suffix(".stderr").write_text("")
        return
    if "stdout" in row:
        source.with_suffix(".stdout").write_text(row["stdout"])
        wrote = True
    if "stderr" in row:
        source.with_suffix(".stderr").write_text(row["stderr"])
        wrote = True
    code = row.get("exit_code")
    if code not in (None, 0) and row.get("mode") != "fault":
        source.with_suffix(".exit").write_text(f"{code}\n")
        wrote = True
    if not wrote:
        source.with_suffix(".exit").write_text("0\n")


def migrate(directory, dry):
    cases, problems = plan(directory)
    if problems:
        print(f"{directory}: not migrated")
        for p in problems:
            print("  -", p)
        return False
    for name, row, source in cases:
        notes, _ = notes_of(row)
        print(f"  {name}" + (f"  [{' '.join(notes)}]" if notes else ""))
        if dry:
            continue
        if notes:
            source.write_text("// test: " + " ".join(notes) + "\n" + source.read_text())
        write_expected(row, source)
    if dry:
        return True
    manifest = directory / "tests" / "manifest.json"
    subprocess.run(["git", "rm", "-q", "-f", str(manifest)], check=True, cwd=ROOT)
    config = ROOT / "jacket.toml"
    relative = directory.resolve().relative_to(ROOT).as_posix()
    text = config.read_text()
    match = re.search(r"(paths\s*=\s*\[)(.*?)(\])", text[text.index("[test]") :], re.S)
    start = text.index("[test]") + match.start(2)
    end = text.index("[test]") + match.end(2)
    items = re.findall(r'"([^"]+)"', match.group(2)) + [relative]
    items = sorted(set(items))
    body = "\n" + "".join(f'    "{i}",\n' for i in items)
    config.write_text(text[:start] + body + text[end:])
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("modules", nargs="*", type=Path)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument(
        "--under",
        type=int,
        metavar="N",
        help="every module still on the old runner with at most N test files",
    )
    parser.add_argument(
        "--skip",
        action="append",
        default=[],
        metavar="MODULE",
        help="with --under: leave this module (a stdlib path) out",
    )
    args = parser.parse_args()
    modules = list(args.modules)
    if args.under:
        for manifest in sorted((ROOT / "stdlib").glob("**/manifest.json")):
            if manifest.parent.name != "tests":
                continue
            module = manifest.parent.parent
            if module.relative_to(ROOT).as_posix() in args.skip:
                continue
            if len(list(manifest.parent.glob("*.jai"))) <= args.under:
                modules.append(module.relative_to(ROOT))
    ok = True
    for module in modules:
        print(module)
        ok &= migrate((ROOT / module).resolve(), args.dry_run)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
