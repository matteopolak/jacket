#!/usr/bin/env python3
"""Focused developer tooling evidence from this checkout's source-built tools."""
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
JAIC = ROOT / "target/debug/jaic"


def run(command, *, output=None, error=""):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, env=ENV,
                            text=True, capture_output=True, timeout=60)
    if result.returncode or (output is not None and result.stdout != output) or result.stderr != error:
        raise RuntimeError(f"{command}: exit {result.returncode}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def main():
    cases = ["c-header", "c-header-roundtrip", "c-header-reject"]
    run([JAIC, "tests/developer-tools/c-header.jai", "-interp"], output="ok\n")
    run(["clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-dynamiclib", "-I", "target",
         "tests/developer-tools/point.c", "-o", "target/libdeveloper_tools_point.dylib"])
    for case in cases:
        source = ROOT / f"tests/developer-tools/{case}.jai"
        error = "C header: procedure must have the C calling convention" if case == "c-header-reject" else ""
        run([JAIC, source, "-interp"], output="ok\n", error=error)
        for optimization in [0, 2]:
            exe = ROOT / f"target/developer-tools-{case}-o{optimization}"
            run([JAIC, source, f"-O{optimization}", "-exe", exe])
            run([exe], output="ok\n", error=error)
    for optimization in [0, 2]:
        exe = ROOT / f"target/developer-tools-performance-o{optimization}"
        report = run([JAIC, "tests/developer-tools/performance-target.jai", f"-O{optimization}",
                      "-exe", exe, "+Performance_Report"])
        for required in ["Compiler performance report", "#run time:", "Bytecode instructions:",
                         "Calls:", "Polymorph solves:", "work:"]:
            if required not in report:
                raise RuntimeError(f"Missing measured report section {required}")
        rows = [float(x) for x in re.findall(r"^  .*: ([0-9.e+-]+) seconds,", report, re.MULTILINE)]
        if not rows or rows != sorted(rows, reverse=True) or not any(x > 0 for x in rows):
            raise RuntimeError("Run records must contain actual positive timings in descending order")
        run([exe], output="ok\n")
    missing = subprocess.run([str(JAIC), "tests/developer-tools/performance-missing.jai", "-check"],
                             cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=60)
    if missing.returncode or "warning: Performance_Report: compiler did not provide a performance report" not in missing.stderr:
        raise RuntimeError("Missing performance message did not produce its diagnostic")
    paths = [ROOT / "stdlib/Generate_C_Header/module.jai",
             ROOT / "stdlib/Performance_Report/module.jai",
             *sorted((ROOT / "stdlib/internal/developer-tools").glob("*.jai")),
             *sorted((ROOT / "tests/developer-tools").glob("*.jai"))]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths])
    run([ROOT / "target/debug/jailint", "-D", "warnings", *paths])
    for os_name, cpu in [("linux", "x64"), ("windows", "x64"), ("wasm", "x64")]:
        run([JAIC, "tests/developer-tools/c-header.jai", "-check", "-os", os_name, "-cpu", cpu])
    print("Header round trips/rejection and real performance plugin reports passed; interpreter/native O0/O2, scoped formatting/lint and three target checks")


if __name__ == "__main__":
    main()
