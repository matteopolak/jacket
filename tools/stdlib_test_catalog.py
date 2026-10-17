"""Discovery and per-case metadata for every runnable stdlib and compiler regression test.

A test directory is `stdlib/**/tests/` (module tests) or `tests/compiler/regressions/`.
Every top-level `*.jai` file of a test directory is a case that must build and exit 0 in each
mode; sub-directories hold helpers and fixtures and are never discovered.

A test directory may carry a `manifest.json`: a list of rows `{"name": ..., ...}` that refine
a case with the keys in ROW_KEYS (expected output, required platform, native-only services,
...). A row for a name that is not a top-level file adds a case from a sub-directory:
`"name": "negative/private-name"` runs `negative/private-name.jai`, and `"source"` names a
different entry file (`"name": "scoped", "source": "scoped/main.jai"`). The same rows drive
`stdlib_runtime.py`, `jaic-sweep.py`, `windows_cross.py` and the browser runner.

This holds no expected-failure baseline. Every selected supported case runs; compiler/API
errors remain failures.
"""

import hashlib
import json
import os
from pathlib import Path

from case_capabilities import user_deferred_reason
from numerics_case_capabilities import (
    apply_numerics_capabilities,
    numerics_unavailable_reason,
)

ROOT = Path(__file__).resolve().parents[1]
COMPILER_REGRESSIONS = "tests/compiler/regressions"

ROW_KEYS = frozenset(
    {
        "name",
        "source",
        # What the program must print and how it must end.
        "stdout",
        # For output that depends on the host (a sound device that is or is not present): the
        # exact stdout must be one of these, with the same empty stderr and exit code as `stdout`.
        "stdout_one_of",
        "stderr",
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
        "mode_contracts",
        # Where the case can run.
        "required_os",
        "required_cpu",
        "unsupported_modes",
        "required_capabilities",
        "required_browser_host_capabilities",
        # How the runner starts it.
        "arguments",
        "input",
        "fixture_libraries",
        "requires_display",
        # macOS GUI programs run one at a time (stdlib_runtime.py WINDOW_LEASE).
        "window_lease",
        "clipboard_guard",
        "source_check_only",
        "numerics_target_source",
    }
)


def reference_rejected_reason(case):
    if (
        case["id"] == "compiler/proc-type-defaults-ifx"
        and hashlib.sha256(case["path"].read_bytes()).hexdigest()
        == "e2aaa5f6c468e62fb635f48ca5ada3532abeba02f8f7e2388cfec634f5108968"
    ):
        return "root official compiler rejects the original overloaded/concrete ifx mismatch; not a supported positive, regardless of raw local acceptance"
    return None


def visible(path):
    return not any(part.startswith("._") for part in path.parts)


def test_directories(root=ROOT):
    """Every `tests` directory below stdlib/, outermost only, then the compiler regressions."""
    stdlib = root / "stdlib"
    for directory, children, _ in os.walk(stdlib):
        children.sort()
        if "tests" in children:
            children.remove("tests")
            yield Path(directory) / "tests"
    yield root / COMPILER_REGRESSIONS


def load_rows(directory):
    manifest = directory / "manifest.json"
    if not manifest.is_file():
        return []
    rows = json.loads(manifest.read_text())
    names = set()
    for row in rows:
        unknown = set(row) - ROW_KEYS
        if unknown:
            raise ValueError(f"{manifest}: unknown keys {sorted(unknown)} in {row.get('name')!r}")
        if "name" not in row or row["name"] in names:
            raise ValueError(f"{manifest}: missing or duplicate name {row.get('name')!r}")
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
        apply_numerics_capabilities(case)
        if "macos" in (case.get("required_os") or ()):
            case["window_lease"] = True
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
                raise ValueError("positive_baseline must name a positive case: " + baseline)
            case["positive_baseline_id"] = target
    if not result:
        raise ValueError("empty test catalogue")
    return result


def case_for_mode(case, mode):
    """Return the explicit mode contract without mutating the discovered source.

    Selectors must use this result for dispatch AND expected-result accounting.
    A required negative is a real check with a precise diagnostic, never a pass
    inferred from target unavailability or an arbitrary compilation failure.
    """
    contract = case.get("mode_contracts", {}).get(mode, {})
    if contract and (
        set(contract) != {"mode", "diagnostic_contains"}
        or contract.get("mode") != "check-fail"
        or not isinstance(contract.get("diagnostic_contains"), str)
        or not contract["diagnostic_contains"].strip()
    ):
        raise ValueError("mode contract requires check-fail and a precise diagnostic")
    if contract and mode in case.get("unsupported_modes", ()):
        raise ValueError("required mode rejection conflicts with unavailable metadata")
    return {**case, **contract}


def unavailable_reason(case, mode, actual_platform=None):
    case = case_for_mode(case, mode)
    numerics_reason = numerics_unavailable_reason(case, mode, actual_platform)
    if numerics_reason:
        return numerics_reason
    if mode in case.get("unsupported_modes", []):
        required = ", ".join(case["required_capabilities"])
        return f"{mode} platform unavailable: requires {required}; actual native execution remains required, not passed"
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

    parser = argparse.ArgumentParser(description="Project the canonical catalogue onto an explicit execution mode")
    parser.add_argument("--mode", choices=("interp", "native", "wasm-interp", "wasm-native"))
    parser.add_argument("--platform")
    arguments = parser.parse_args()
    entries = validated_entries()
    if arguments.mode:
        entries = [
            {**case_for_mode(case, arguments.mode),
             "selected_unavailable_reason": unavailable_reason(case, arguments.mode, arguments.platform)}
            for case in entries
        ]
    # The browser runner consumes the same manifests and source discovery.
    print(
        json.dumps(
            [
                {**case, "path": str(case["path"].relative_to(ROOT))}
                for case in entries
            ]
        )
    )
