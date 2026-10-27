#!/usr/bin/env python3
"""Turns a stdlib module's manifest.json cases into `jacket test` files.

    PYTHONPATH=tools python3 tools/stdlib/migrate_to_jacket_test.py stdlib/Sort [--dry-run]

Each case stays where it is and becomes a golden test (a program with `.stdout`/`.stderr`/`.exit` files
beside it) or, for a check-fail row, a compile-fail test (`// test: compile_fail` and a `.stderr` that
`jacket test --accept --exact NAME` fills in), with the row's gating as `// test:` notes on the file's
first line (a row's `input` becomes `args(...)`; see the docs for the environment and fixture
inputs). The module is added to `[test] paths` in jacket.toml and its manifest is deleted. The
migration section of docs/tools/jacket-test.md says what every row key becomes.

Cases it cannot express are listed and the module is left untouched: run it again after
converting those by hand (or leaving the module on the old runner).
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

from toolkit.repo import ROOT

# Keys with a direct equivalent, or that carry no behaviour of their own.
HANDLED = {
    "name",
    "source",
    "stdout",
    "stdout_one_of",
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
    "clipboard_guard",
    "source_check_only",
    "warning_contains",
}
# Every key a manifest row may carry (the old runner's vocabulary), handled or reported below.
ROW_KEYS = HANDLED | {
    "disabled",
    "fixture_libraries",
    "interpreter_only",
    "mode_contracts",
    "numerics_target_source",
    "unsupported_platforms",
}
WASM_MODES = {"wasm-interp", "wasm-native"}
SCRATCH_INPUTS = {"scratch-file": "$TEMP/data", "scratch-directory": "$TEMP"}
# Inputs the machine provides, named by the environment variable that holds their path.
ENVIRONMENT_INPUTS = {
    "font": "JAIC_TEST_FONT_PATH",
    "windows-resource-directory": "JAIC_TEST_WINDOWS_RESOURCE_DIR",
}
# Inputs that are files beside the tests, in a folder of that name, and a folder to write to.
FIXTURE_INPUTS = {"ico-fixtures": ["$DIR/ico-fixtures", "$TEMP"]}
# A mode the row cannot run in, as the arguments of the `skip_on` call that skips it.
MODE_TARGETS = {
    "interp": ".INTERP",
    "native": ".NATIVE",
    "wasm-interp": ".WASM, mode = .INTERP",
    "wasm-native": ".WASM, mode = .NATIVE",
}


def quoted(argument):
    """An argument as the `args(...)` note spells it: quotes and backslashes escaped."""
    return '"' + argument.replace("\\", "\\\\").replace('"', '\\"') + '"'


def platform_target(platform):
    """`windows-arm64` as the arguments of a `skip_on` call: `.WINDOWS, cpu = .ARM64`."""
    system, dash, cpu = platform.partition("-")
    if not dash:
        return "." + system.upper()
    return f".{system.upper()}, cpu = .{cpu.upper()}"


def as_list(value):
    if value is None:
        return []
    return [value] if isinstance(value, str) else list(value)


def wasm_only_contract(row):
    """The one contract that both wasm modes of `mode_contracts` share, if that is all it says."""
    contracts = row.get("mode_contracts")
    if not contracts or set(contracts) != WASM_MODES:
        return None
    first = contracts["wasm-interp"]
    return first if first == contracts["wasm-native"] else None


def notes_of(row):
    """The `// test:` notes of a row, and the reasons it cannot be expressed."""
    notes, problems = [], []
    only, skip_on = [], []
    contract = wasm_only_contract(row)
    for key in sorted(set(row) - HANDLED):
        if key == "mode_contracts" and contract is not None:
            continue
        problems.append(f"row key `{key}` has no jacket test equivalent")
    if contract is not None:
        if contract.get("mode") != "check-fail" or not contract.get("diagnostic_contains"):
            problems.append("mode_contracts other than a wasm check-fail")
        only.append(".WASM")
        notes.append("compile_fail")
    unsupported = set(row.get("unsupported_modes", []))
    if unsupported >= WASM_MODES:
        skip_on.append(".WASM")
        unsupported -= WASM_MODES
    skip_on.extend(MODE_TARGETS[mode] for mode in sorted(unsupported))
    only.extend("." + os.upper() for os in as_list(row.get("required_os")))
    if row.get("required_cpu"):
        only.extend("." + cpu.upper() for cpu in as_list(row["required_cpu"]))
    skip_on.extend(platform_target(p) for p in row.get("unsupported_platforms", []))
    if only:
        notes.append("only(" + ",".join(only) + ")")
    # One call for each target that names more than an OS, CPU or mode.
    plain = [t for t in skip_on if "=" not in t]
    if plain:
        notes.append("skip_on(" + ",".join(plain) + ")")
    notes.extend(f"skip_on({t})" for t in skip_on if "=" in t)
    if row.get("requires_display"):
        notes.append("needs(.DISPLAY)")
    systems = [o.lower() for o in as_list(row.get("required_os"))]
    # A test that runs only on macOS and Windows hosts opens windows there: one at a time.
    gui = "macos" in systems and "linux" not in systems
    if row.get("window_lease") or row.get("clipboard_guard") or gui:
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
        notes.append("args(" + ",".join(quoted(a) for a in row["arguments"]) + ")")
    elif "input" in row:
        kind = row["input"]
        if kind in SCRATCH_INPUTS:
            notes.append(f'args("{SCRATCH_INPUTS[kind]}")')
        elif kind in ENVIRONMENT_INPUTS:
            notes.append(f'args("${{{ENVIRONMENT_INPUTS[kind]}}}")')
        elif kind in FIXTURE_INPUTS:
            notes.append("args(" + ",".join(f'"{a}"' for a in FIXTURE_INPUTS[kind]) + ")")
        else:
            problems.append(f"input `{kind}` has no jacket test equivalent")
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
    if row.get("mode") == "check-fail" or wasm_only_contract(row) is not None:
        # `jacket test --accept` fills in the diagnostics; an empty file makes the test discoverable.
        source.with_suffix(".stderr").write_text("")
        return
    if "stdout" in row:
        source.with_suffix(".stdout").write_text(row["stdout"])
        wrote = True
    for number, text in enumerate(row.get("stdout_one_of", []), 1):
        # The first output is `name.stdout`; the others are `name.stdout.2`, `name.stdout.3`, ...
        suffix = ".stdout" if number == 1 else f".stdout.{number}"
        source.with_name(source.stem + suffix).write_text(text)
        wrote = True
    if "stderr" in row:
        source.with_suffix(".stderr").write_text(row["stderr"])
        wrote = True
    elif ("stdout" in row or "stdout_one_of" in row) and "warning_contains" not in row:
        # A row that names its output also meant the program printed nothing else.
        source.with_suffix(".stderr").write_text("")
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
    if manifest.is_file():
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
