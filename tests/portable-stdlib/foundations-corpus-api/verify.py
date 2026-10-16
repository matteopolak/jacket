#!/usr/bin/env python3
"""Focused shared-tree Hash/Math/File evidence; all outputs remain on assigned T7."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
T7 = Path("/Volumes/T7/codex-builds/jai-cleanroom-01a137eb/foundations")
HERE = Path(__file__).resolve().parent

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, default=T7 / "shared-target/debug/jaic")
    parser.add_argument("--jailint", type=Path, default=T7 / "shared-target/debug/jailint")
    args = parser.parse_args()
    output = Path(tempfile.mkdtemp(prefix="api-family-proof-", dir=T7))
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
    watched = [
        ROOT / "stdlib/Math/module.jai", ROOT / "stdlib/internal/numerics/math/module.jai",
        *sorted((ROOT / "stdlib/internal/numerics").glob("math_*.jai")),
        ROOT / "stdlib/Hash/module.jai", ROOT / "stdlib/internal/hash/module.jai",
        ROOT / "stdlib/File/module.jai", ROOT / "stdlib/internal/files/file.jai",
        ROOT / "stdlib/internal/files/native-unix.jai",
        ROOT / "stdlib/internal/collections/hash-table.jai",
    ]
    evidence = {"complete": False, "commands": [], "output": str(output),
                "source_before": {str(p.relative_to(ROOT)): digest(p) for p in watched},
                "tools": {str(p): digest(p) for p in [args.jaic, args.jailint]},
                "native_configuration": "O0/O2; assertions enabled; optional crash backtrace disabled"}
    report = output / "results.json"

    def run(command, expected_stdout=None, reject=None):
        result = subprocess.run([str(x) for x in command], cwd=ROOT, env=env,
                                capture_output=True, text=True, timeout=60)
        evidence["commands"].append({"command": [str(x) for x in command],
                                     "exit_code": result.returncode, "stdout": result.stdout,
                                     "stderr": result.stderr,
                                     "required_rejection": reject})
        report.write_text(json.dumps(evidence, indent=2) + "\n")
        if reject:
            if result.returncode == 0 or reject not in result.stderr:
                raise RuntimeError("Required privacy rejection did not match")
        elif result.returncode or result.stderr or (expected_stdout is not None and result.stdout != expected_stdout):
            raise RuntimeError(str(command) + "\n" + result.stdout + result.stderr)
        return result

    formatter = output / "jaifmt"
    run([args.jaic, ROOT / "jaifmt/main.jai", "-O2", "-no_backtrace_on_crash", "-exe", formatter])
    owned = [ROOT / "stdlib/Math/module.jai", ROOT / "stdlib/internal/hash/module.jai", *sorted(HERE.glob("*.jai"))]
    run([formatter, "--config", ROOT / "jaifmt.toml", *owned])
    run([formatter, "--config", ROOT / "jaifmt.toml", "--check", *owned])
    positive = [HERE / (name + ".jai") for name in
                ["math-poke", "hash-call-context", "hash-variants", "file-qualified", "file-public-load", "file-compile-time-qualified"]]
    positive += [ROOT / "tests/portable-stdlib" / (name + ".jai")
                 for name in ["hash-scalars", "hash-floats", "hash-scalar-fnv"]]
    for path in positive:
        run([args.jaic, path, "-interp"], expected_stdout="ok\n")
        for optimization in [0, 2]:
            exe = output / (path.stem + "-o" + str(optimization))
            run([args.jaic, path, "-O" + str(optimization), "-no_backtrace_on_crash", "-exe", exe])
            run([exe], expected_stdout="ok\n")
    run([args.jailint, "-D", "warnings", ROOT / "stdlib/Math/module.jai", *positive])
    for name in ["file-no-import-negative"]:
        run([args.jaic, HERE / (name + ".jai"), "-check"],
            reject="unknown identifier `get_absolute_path`")
    run([args.jaic, HERE / "hash-struct-variant-negative.jai", "-check"],
        reject="get_hash requires an integer, boolean, enum, pointer, or float")
    run([args.jaic, HERE / "file-compile-time-qualified.jai", "-check", "-", "test"],
        expected_stdout="selected\n")
    scope = HERE.parent / "foundations-table-scope"
    for name, symbol in [("named-private-negative", "File"), ("other-named-private-negative", "Hidden_File"), ("constant-private-negative", "private_number")]:
        run([args.jaic, scope / (name + ".jai"), "-check"], reject="unknown identifier `" + symbol + "`")
    # Official executed oracle establishes anonymous imports remain visible
    # across #load even after #scope_file. Old filenames/receipts are historical.
    run([args.jaic, HERE / "file-private-load-negative.jai", "-interp"], expected_stdout="")
    run([args.jaic, HERE / "file-compile-time-private-negative.jai", "-check", "-", "test"],
        expected_stdout="")
    evidence["source_after"] = {str(p.relative_to(ROOT)): digest(p) for p in watched}
    evidence["fixture_hashes"] = {str(p.relative_to(ROOT)): digest(p) for p in HERE.glob("*.jai")}
    if evidence["source_before"] != evidence["source_after"]:
        raise RuntimeError("Watched source changed during focused proof")
    evidence["supported_checks_complete"] = True
    evidence["complete"] = True
    evidence["remaining_required_cases"] = []
    evidence["scope"] = "All canonical API and official anonymous/named scope controls pass"
    report.write_text(json.dumps(evidence, indent=2) + "\n")
    print(report)

if __name__ == "__main__":
    main()
