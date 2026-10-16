#!/usr/bin/env python3
"""Shared canonical Table order and official anonymous-import scope proof."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
T7 = Path("/Volumes/T7/codex-builds/jai-cleanroom-01a137eb/foundations")
JAIC = T7 / "table-order-fresh-target/debug/jaic"
LINT = T7 / "table-order-fresh-target/debug/jailint"
APP = Path("/Users/matthew/projects/jai/artifacts/cleanroom-workers/application-corpus")

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    out = Path(tempfile.mkdtemp(prefix="table-scope-proof-", dir=T7))
    env = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
    evidence = {"complete": False, "table_complete": False, "commands": [], "required_scope_failures": [],
                "tools": {str(p): digest(p) for p in [JAIC, LINT]}}
    report = out / "results.json"

    def run(command, expected=None, reject=None, keep_failure=False, expected_stderr=""):
        result = subprocess.run([str(x) for x in command], cwd=out, env=env,
                                text=True, capture_output=True, timeout=60)
        valid = (result.returncode != 0 and reject in result.stderr) if reject else (
            result.returncode == 0 and result.stderr == expected_stderr and (expected is None or result.stdout == expected))
        record = {"command": [str(x) for x in command], "exit_code": result.returncode,
                  "stdout": result.stdout, "stderr": result.stderr, "required_rejection": reject,
                  "passed": valid}
        evidence["commands"].append(record)
        report.write_text(json.dumps(evidence, indent=2) + "\n")
        if not valid:
            if keep_failure:
                evidence["required_scope_failures"].append(record)
            else:
                raise RuntimeError(str(command) + "\n" + result.stdout + result.stderr)
        return result.stdout

    formatter = out / "jaifmt"
    run([JAIC, ROOT / "jaifmt/main.jai", "-O2", "-no_backtrace_on_crash", "-exe", formatter])
    owned = [ROOT / "stdlib/internal/collections/hash-table.jai", *sorted(HERE.glob("*.jai"))]
    run([formatter, "--config", ROOT / "jaifmt.toml", *owned])
    run([formatter, "--config", ROOT / "jaifmt.toml", "--check", *owned])
    watched = [ROOT / "stdlib/internal/collections/hash-table.jai",
               ROOT / "stdlib/Hash_Table/module.jai", ROOT / "stdlib/Math/module.jai",
               ROOT / "stdlib/internal/hash/module.jai", ROOT / "stdlib/prelude/reflection.jai"]
    evidence["source_before"] = {str(p.relative_to(ROOT)): digest(p) for p in watched}

    cases = [HERE / "table-order.jai", HERE / "table-callbacks.jai",
             ROOT / "tests/portable-stdlib/hash-table-contract/main.jai",
             ROOT / "tests/portable-stdlib/foundations-corpus-api/hash-variants.jai",
             ROOT / "tests/portable-stdlib/foundations-corpus-api/hash-call-context.jai",
             ROOT / "tests/portable-stdlib/foundations-corpus-api/math-poke.jai"]
    for index, path in enumerate(cases):
        run([JAIC, path, "-interp"], expected="ok\n")
        for level in [0, 2]:
            exe = out / ("case-" + str(index) + "-o" + str(level))
            run([JAIC, path, "-O" + str(level), "-no_backtrace_on_crash", "-exe", exe])
            run([exe], expected="ok\n")

    access = json.loads((APP / "ACCESS.json").read_text())
    inputs = [f for f in access["files"] if f["path"].startswith("sjorsdonkers--toml-jai/") and f["path"].endswith(".jai")]
    evidence["application_inputs"] = {}
    for item in inputs:
        actual = digest(APP / item["path"])
        if actual != item["sha256"]:
            raise RuntimeError("Application hash mismatch: " + item["path"])
        evidence["application_inputs"][item["path"]] = actual
    app = APP / "sjorsdonkers--toml-jai/examples/custom_handlers.jai"
    app_stdout = "Magic_Bytes unaltered: [1, 67, 242]"
    app_stderr = "Invalid Magic_Bytes: {[2, 67, 242]}Fixed array unaltered.Magic_Bytes expected 3 items, got 4Superfluous key `b` in .OnlyA"
    # The unchanged consumer deliberately exercises rejected input. Require
    # its exact diagnostics and every original assertion, rather than ignoring stderr.
    output = run([JAIC, app, "-interp"], expected=app_stdout, expected_stderr=app_stderr)
    for level in [0, 2]:
        exe = out / ("actual-toml-o" + str(level))
        run([JAIC, app, "-O" + str(level), "-no_backtrace_on_crash", "-exe", exe])
        run([exe], expected=output, expected_stderr=app_stderr)
    run([LINT, "-D", "warnings", ROOT / "stdlib/Hash_Table/module.jai", *cases])
    evidence["table_complete"] = True

    # Official positives: retain failures until compiler owner fixes general scope.
    run([JAIC, HERE / "anonymous-load.jai", "-interp"], expected="ok\n", keep_failure=True)
    run([JAIC, HERE / "anonymous-compile-time.jai", "-check", "-", "test"],
        expected="selected\n", keep_failure=True)
    for name, symbol in [("named-private-negative", "File"), ("other-named-private-negative", "Hidden_File"),
                         ("constant-private-negative", "private_number")]:
        run([JAIC, HERE / (name + ".jai"), "-check"], reject="unknown identifier `" + symbol + "`")
    run([JAIC, ROOT / "tests/portable-stdlib/foundations-corpus-api/file-no-import-negative.jai", "-check"],
        reject="unknown identifier `get_absolute_path`")
    run([JAIC, ROOT / "tests/portable-stdlib/foundations-corpus-api/file-public-load.jai", "-interp"], expected="ok\n")
    evidence["source_after"] = {str(p.relative_to(ROOT)): digest(p) for p in watched}
    if evidence["source_before"] != evidence["source_after"]:
        raise RuntimeError("Production changed during proof")
    evidence["fixture_sha256"] = {str(p.relative_to(ROOT)): digest(p) for p in HERE.glob("*.jai")}
    evidence["complete"] = not evidence["required_scope_failures"]
    report.write_text(json.dumps(evidence, indent=2) + "\n")
    print(report)
    raise SystemExit(0 if evidence["complete"] else 1)

if __name__ == "__main__":
    main()
