"""Run clean-room numerics tests against an explicit locally source-built compiler."""

from pathlib import Path
import argparse
import json
import os
import subprocess
import sys
import time
from target_contracts import ORACLE, target_contract

root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(root / "tools"))
from repo_output import output_dir
parser = argparse.ArgumentParser()
parser.add_argument("--compiler", type=Path, required=True)
parser.add_argument("--stdlib", type=Path, required=True)
parser.add_argument("--quick", action="store_true")
parser.add_argument(
    "--canonical-dependencies",
    action="store_true",
    default=True,
    help="Canonical dependencies are required; this explicit flag is retained for callers",
)
parser.add_argument(
    "--output", type=Path, default=output_dir("numerics", "verification")
)
args = parser.parse_args()
compiler = args.compiler.resolve()
env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve()))
out = args.output.resolve()
if not out.is_relative_to(output_dir().resolve()):
    raise SystemExit("evidence must remain in the repository output directory")
out.mkdir(parents=True, exist_ok=True)
records = []
imports = []


def run(label, cmd, sentinel=None, expect_fail=False):
    started = time.monotonic()
    result = subprocess.run(
        [str(x) for x in cmd],
        cwd=root,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    log = result.stdout
    (out / (label + ".log")).write_text(log)
    markers = (
        []
        if sentinel is None
        else (sentinel if isinstance(sentinel, tuple) else [sentinel])
    )
    passed = (
        (
            result.returncode != 0
            and "overflow assertion was missed" not in log
            and "half float overflow" in log
        )
        if expect_fail
        else result.returncode == 0
        and all(marker in log for marker in markers)
        and "assertion failed" not in log
    )
    records.append(
        dict(
            label=label,
            passed=passed,
            exit=result.returncode,
            seconds=round(time.monotonic() - started, 3),
        )
    )
    print(("PASS " if passed else "FAIL ") + label, flush=True)
    if not passed:
        print(log)
    (out / "results.json").write_text(json.dumps(records, indent=2) + "\n")
    return passed


run("inventory", [sys.executable, "tools/numerics_surface.py"])
run("generate-oracle", [sys.executable, "tools/numerics/oracle.py"])
tests = {
    "core": ("stdlib/Float16/tests/core.jai", "numerics core ok"),
    "color-portable": (
        "stdlib/Color_Interpolation/tests/policies.jai",
        "portable color policies ok",
    ),
    "extended": ("stdlib/Color_Interpolation/tests/extended.jai", "extended numerics ok"),
    "edges": ("stdlib/Float16/tests/edges.jai", "numeric edges ok"),
    "algebra": ("stdlib/Float16/tests/algebra.jai", "algebra overloads ok"),
    "color-policies": ("stdlib/GetRect/tests/color-policy-hash.jai", "color policies ok"),
    "ui": ("stdlib/GetRect/tests/ui.jai", "ui contracts ok"),
    "ui-smoke": ("stdlib/GetRect/tests/ui-smoke.jai", "ui smoke ok"),
    "ui-interaction": ("stdlib/GetRect/tests/ui-interaction.jai", "ui interaction ok"),
    "render-callbacks": ("stdlib/GetRect/tests/render-callbacks.jai", "render callbacks ok"),
    "default-draw-procs": (
        "stdlib/GetRect/tests/default-draw-procs.jai",
        "default draw procs ok",
    ),
    "precision-rounding": (
        "stdlib/Color_Interpolation/tests/precision-rounding.jai",
        "explicit precision rounding ok",
    ),
    "canonical-rendering": (
        "stdlib/Simp/tests/canonical-rendering.jai",
        "canonical rendering types and metrics ok",
    ),
    "type-indicator": (
        "stdlib/GetRect/tests/type-indicator.jai",
        "alternate type indicator ok",
    ),
    "decimal-precision": (
        "stdlib/GetRect/tests/decimal-precision.jai",
        "canonical decimal precision ok",
    ),
    "text-capacity": (
        "stdlib/GetRect/tests/text-capacity.jai",
        ("text capacity rejection ok", "rejected input because capacity was exceeded"),
    ),
    "wheel-events": ("stdlib/GetRect/tests/wheel-events.jai", "canonical wheel events ok"),
    "keyboard-shortcuts": (
        "stdlib/GetRect/tests/keyboard-shortcuts.jai",
        "canonical keyboard shortcuts ok",
    ),
    "normalization-ieee": (
        "stdlib/Math/tests/normalization-ieee.jai",
        "IEEE normalization ok",
    ),
    "abs-ieee": ("stdlib/Math/tests/abs-ieee.jai", "IEEE absolute value ok"),
    "oracle": (ORACLE, "independent oracle ok"),
    "public-contract": ("stdlib/Math/tests/float16-edge-vectors.jai", "ok"),
}
for name, (source, sentinel) in tests.items():
    common = [compiler, str(output_dir("numerics", "numeric_oracle.jai")) if source == ORACLE else source] + imports
    run(name + "-interp", common + ["-interp"], sentinel)
    if args.quick:
        continue
    for level in ["O0", "O2"]:
        exe = out / (name + "-" + level)
        if run(name + "-" + level + "-build", common + ["-" + level, "-exe", exe]):
            run(name + "-" + level, [exe], sentinel)
    contract = target_contract(source, "wasm")
    if not contract.available:
        records.append(
            dict(
                label=name + "-wasm",
                passed=None,
                classification="unavailable target",
                reason=contract.reason,
            )
        )
        (out / "results.json").write_text(json.dumps(records, indent=2) + "\n")
        continue
    wasm = out / (name + ".wasm")
    if run(name + "-wasm-build", common + ["-os", "wasm", "-O2", "-exe", wasm]):
        run(
            name + "-wasm",
            ["node", "--no-warnings", "tools/wasi_run.mjs", wasm],
            sentinel,
        )
for target in ["linux", "windows", "macos", "wasm"]:
    contract = target_contract("stdlib/GetRect/tests/ui-smoke.jai", target)
    if not contract.available:
        records.append(
            dict(
                label=target + "-unused-check",
                passed=None,
                classification="unavailable target",
                reason=contract.reason,
            )
        )
        continue
    run(
        target + "-unused-check",
        [compiler, "stdlib/GetRect/tests/ui-smoke.jai"]
        + imports
        + ["-check", "-no_dce", "-os", target],
    )
for target in ["linux", "windows", "macos"]:
    run(
        target + "-x64-unused-check",
        [compiler, "stdlib/GetRect/tests/ui-smoke.jai"]
        + imports
        + ["-check", "-no_dce", "-os", target, "-cpu", "x64"],
    )
run(
    "overflow-assert",
    [compiler, "stdlib/Float16/tests/half-overflow.jai", "-interp"],
    expect_fail=True,
)
if not args.quick:
    for level in ["O0", "O2"]:
        exe = out / ("overflow-" + level)
        if run(
            "overflow-" + level + "-build",
            [compiler, "stdlib/Float16/tests/half-overflow.jai", "-" + level, "-exe", exe],
        ):
            run("overflow-" + level, [exe], expect_fail=True)
    wasm = out / "overflow.wasm"
    if run(
        "overflow-wasm-build",
        [
            compiler,
            "stdlib/Float16/tests/half-overflow.jai",
            "-os",
            "wasm",
            "-O2",
            "-exe",
            wasm,
        ],
    ):
        run(
            "overflow-wasm",
            ["node", "--no-warnings", "tools/wasi_run.mjs", wasm],
            expect_fail=True,
        )
failures = [r for r in records if r["passed"] is False]
passed = sum(r["passed"] is True for r in records)
unavailable = sum(r["passed"] is None for r in records)
print(
    f"{passed} passed; {len(failures)} failed; {unavailable} unavailable; evidence: {out}"
)
raise SystemExit(bool(failures))
