"""Discovery and per-case metadata for the compiler regression tests the Python runner executes.

The stdlib modules' tests run under `jacket test` (`[test] paths` in jacket.toml, see
docs/tools/jacket-test.md) and are skipped here. What is left is `tests/compiler/regressions/`
and any `stdlib/**/tests/` folder that is not under `[test] paths`.
Every top-level `*.jai` file of a test directory is a case that must build and exit 0 in each
mode; sub-directories hold helpers and fixtures and are never discovered.

A test directory may carry a `manifest.json`: a list of rows `{"name": ..., ...}` that refine
a case with the keys in ROW_KEYS (expected output, required platform, unsupported modes,
...). A row for a name that is not a top-level file adds a case from a sub-directory:
`"name": "negative/private-name"` runs `negative/private-name.jai`, and `"source"` names a
different entry file (`"name": "scoped", "source": "scoped/main.jai"`). The same rows drive
`stdlib_runtime.py`, `jaic_sweep.py`, `windows_cross.py` and the browser runner.

This holds no expected-failure baseline. Every selected supported case runs; compiler/API
errors remain failures.
"""

import json
import os
import tomllib
from pathlib import Path

from stdlib.case_capabilities import user_deferred_reason
from toolkit.repo import ROOT

COMPILER_REGRESSIONS = "tests/compiler/regressions"

ROW_KEYS = frozenset(
    {
        "name",
        "source",
        # What the program must print and how it must end.
        "stdout",
        "stderr",
        # A compiler warning the case must produce (on stderr when interpreted, in the build
        # output when compiled); the program's own stderr must otherwise be empty.
        "warning_contains",
        "exit_code",
        "error_contains",
        # `mode` "check-fail": a compile-time rejection (`-check` must fail with
        # `diagnostic_contains`); `positive_baseline` names the case in the same test directory
        # that must still check clean. `mode` "fault": the program builds and ends with
        # `exit_code` after printing `error_contains` (a checked runtime failure); a native
        # executable aborts instead, so for it any nonzero end counts.
        # Any other `error_contains` row is a compile-time error met while interpreting.
        "mode",
        "diagnostic_contains",
        "positive_baseline",
        # A reason the case is switched off: the source has a genuine error.
        "disabled",
        # Where the case can run.
        "required_os",
        "required_cpu",
        # `<os>-<cpu>` platforms where a required native SDK has no build (`required_capabilities` says which).
        "unsupported_platforms",
        "unsupported_modes",
        "required_capabilities",
        "required_browser_host_capabilities",
        "requires_display",
        "source_check_only",
        # The program does its work at compile time (a metaprogram that builds and checks other
        # workspaces): the interpreter runs all of it, and the native, wasm and cross builds would
        # only repeat that, so only the `interp` mode and the sweep's interpreted run include it.
        "interpreter_only",
    }
)


def disabled_reason(case):
    """Why a case is switched off (a manifest row's or upstream case's `disabled` text), or None."""
    return case.get("disabled")


def visible(path):
    return not any(part.startswith("._") for part in path.parts)


def jacket_test_folders(root=ROOT):
    """The folders `jacket test` runs (`[test] paths` in jacket.toml), as absolute paths."""
    config = root / "jacket.toml"
    if not config.is_file():
        return set()
    paths = tomllib.loads(config.read_text(encoding="utf-8")).get("test", {}).get("paths", [])
    return {(root / path).resolve() for path in paths}


def test_directories(root=ROOT):
    """Every `tests` directory below stdlib/, outermost only, then the compiler regressions.
    Folders `jacket test` runs are left out: their tests are not manifest cases."""
    stdlib = root / "stdlib"
    migrated = jacket_test_folders(root)
    for directory, children, _ in os.walk(stdlib):
        children.sort()
        if "tests" in children:
            children.remove("tests")
            if Path(directory).resolve() not in migrated:
                yield Path(directory) / "tests"
    yield root / COMPILER_REGRESSIONS


def load_rows(directory):
    manifest = directory / "manifest.json"
    if not manifest.is_file():
        return []
    rows = json.loads(manifest.read_text(encoding="utf-8"))
    names = set()
    for row in rows:
        unknown = set(row) - ROW_KEYS
        if unknown:
            raise ValueError(
                f"{manifest}: unknown keys {sorted(unknown)} in {row.get('name')!r}"
            )
        if "name" not in row or row["name"] in names:
            raise ValueError(
                f"{manifest}: missing or duplicate name {row.get('name')!r}"
            )
        names.add(row["name"])
    return rows


def case_id(root, directory, name):
    if directory == root / COMPILER_REGRESSIONS:
        return "compiler/" + name
    return (directory.relative_to(root / "stdlib") / name).as_posix()


def entries(root=ROOT):
    for directory in test_directories(root):
        if not directory.is_dir():
            continue
        rows = {row["name"]: row for row in load_rows(directory)}
        names = {
            path.stem
            for path in directory.glob("*.jai")
            if visible(path.relative_to(root))
        } | set(rows)
        family = "compiler" if directory == root / COMPILER_REGRESSIONS else "modules"
        for name in sorted(names):
            row = rows.get(name, {})
            source = row.get("source", name + ".jai")
            case = {
                **{k: v for k, v in row.items() if k not in ("name", "source")},
                "id": case_id(root, directory, name),
                "path": directory / source,
                "family": family,
            }
            case["suite"] = directory.relative_to(root).as_posix()
            case["row_name"] = name
            yield case


def validated_entries(root=ROOT):
    result = list(entries(root))
    by_id = {}
    for case in result:
        if case["id"] in by_id:
            raise ValueError("duplicate test ID: " + case["id"])
        by_id[case["id"]] = case
        if not case["path"].is_file():
            raise ValueError("missing test source: " + str(case["path"]))
        if not case["path"].resolve().is_relative_to(root.resolve()):
            raise ValueError("test source leaves repository: " + str(case["path"]))
    for case in result:
        baseline = case.get("positive_baseline")
        if baseline:
            target = case_id(root, root / case["suite"], baseline)
            other = by_id.get(target)
            if (
                other is None
                or other.get("mode") == "check-fail"
                or "error_contains" in other
            ):
                raise ValueError(
                    "positive_baseline must name a positive case: " + baseline
                )
            case["positive_baseline_id"] = target
    if not result:
        raise ValueError("empty test catalogue")
    return result


def unavailable_reason(case, mode, actual_platform=None):
    if mode in case.get("unsupported_modes", []):
        required = ", ".join(case["required_capabilities"])
        return f"{mode} platform unavailable: requires {required}; actual native execution remains required, not passed"
    if "-".join((actual_platform or "").split("-")[:2]) in case.get(
        "unsupported_platforms", []
    ):
        required = ", ".join(case["required_capabilities"])
        return f"{actual_platform} platform unavailable: requires {required}; actual execution remains required, not passed"
    target_os = (
        "wasm" if mode.startswith("wasm") else (actual_platform or "").split("-")[0]
    )
    deferred = user_deferred_reason(case, target_os)
    if deferred:
        return deferred
    required = case.get("required_os")
    required = [required] if isinstance(required, str) else required
    if required and target_os and target_os not in required:
        return f"requires actual {'/'.join(required)} host; native execution remains required, not passed on {target_os}"
    target_cpu = (
        "wasm64"
        if mode.startswith("wasm")
        else (actual_platform or "").partition("-")[2].partition("-")[0]
    )
    required_cpu = case.get("required_cpu")
    if required_cpu and target_cpu and target_cpu != required_cpu:
        return f"requires actual {required_cpu} CPU; execution remains required, not passed on {target_cpu}"
    return None


def cases(root=ROOT):
    for case in validated_entries(root):
        yield case["id"], case["path"]


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="Project the canonical catalogue onto an explicit execution mode"
    )
    parser.add_argument(
        "--mode", choices=("interp", "native", "wasm-interp", "wasm-native")
    )
    parser.add_argument("--platform")
    arguments = parser.parse_args()
    entries = validated_entries()
    if arguments.mode:
        entries = [
            {
                **case,
                "selected_unavailable_reason": unavailable_reason(
                    case, arguments.mode, arguments.platform
                ),
            }
            for case in entries
        ]
    # The browser runner consumes the same manifests and source discovery.
    print(
        json.dumps(
            [{**case, "path": str(case["path"].relative_to(ROOT))} for case in entries]
        )
    )
