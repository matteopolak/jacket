#!/usr/bin/env python3
"""Real persistent Check factory and compiler-message callback effects."""
import os
from pathlib import Path
import subprocess
import build_developer_check

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
JAIC = ROOT / "target/debug/jaic"


def run(command, *, expected="", code=0):
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=ENV,
                            capture_output=True, text=True, timeout=90)
    if result.returncode != code or (expected is not None and result.stdout != expected) or (code == 0 and result.stderr):
        raise RuntimeError(f"Check plugin failure: {command}; status={result.returncode}; "
                           f"diagnostic={(result.stderr or result.stdout).splitlines()[:2]}")
    return result


def main():
    build_developer_check.main()
    for name in ["check-plugin", "check-plugin-workspace"]:
        source = ROOT / f"tests/developer-tools/{name}.jai"
        run([JAIC, source, "-interp"], expected="ok\n")
        for optimization in [0, 2]:
            executable = ROOT / f"target/developer-tools-{name}-o{optimization}"
            run([JAIC, source, f"-O{optimization}", "-exe", executable])
            run([executable], expected="ok\n")
    for name, diagnostic in [("check-plugin-porting", "small integer addition requires an explicit intermediate integer cast"),
                             ("check-plugin-bindings", "could not parse foreign signature")]:
        result = run([JAIC, ROOT / f"tests/developer-tools/rejections/{name}.jai", "-check"], expected=None, code=1)
        if result.stderr.count("error:") != 1 or diagnostic not in result.stderr:
            raise RuntimeError(f"Expected one genuine compiler diagnostic from {name}")
    paths = [ROOT / "stdlib/Check/module.jai", ROOT / "stdlib/internal/developer-tools/check-engine/module.jai",
             ROOT / "tests/developer-tools/check-plugin.jai", ROOT / "tests/developer-tools/check-plugin-workspace.jai"]
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths,
         ROOT / "stdlib/internal/developer-tools/porting-check.jai", ROOT / "stdlib/internal/developer-tools/check-bindings.jai",
         ROOT / "tests/developer-tools/rejections/check-plugin-porting.jai", ROOT / "tests/developer-tools/rejections/check-plugin-bindings.jai"])
    run([ROOT / "target/debug/jailint", "-D", "warnings", *paths])
    run([ROOT / "target/debug/jailint", "-D", "warnings", "--os", "windows", "--cpu", "x64", *paths])
    print("Check persistent factory and configured real callback effects passed interpreter/native Mac O0/O2, diagnosed rejections, formatting and strict lint")


if __name__ == "__main__":
    main()
