#!/usr/bin/env python3
"""Real persistent Check factory and compiler-message callback effects."""
import build_developer_check
from driven_sources import driven
from dev_common import JAIC, JAIFMT, ROOT, run_checked



def run(command, *, expected="", code=0):
    return run_checked(command, label="Check plugin", code=code, expected=expected, stderr="" if code == 0 else None)


def main():
    build_developer_check.main()
    for name in ["check-plugin", "check-plugin-workspace"]:
        source = driven(f"{name}.jai")
        run([JAIC, source, "-interp"], expected="ok\n")
        for optimization in [0, 2]:
            executable = ROOT / f"target/developer-tools-{name}-o{optimization}"
            run([JAIC, source, f"-O{optimization}", "-exe", executable])
            run([executable], expected="ok\n")
    for name, diagnostic in [("check-plugin-porting", "small integer addition requires an explicit intermediate integer cast"),
                             ("check-plugin-bindings", "could not parse foreign signature")]:
        result = run([JAIC, ROOT / f"tests/compiler/rejections/{name}.jai", "-check"], expected=None, code=1)
        if result.stderr.count("error:") != 1 or diagnostic not in result.stderr:
            raise RuntimeError(f"Expected one genuine compiler diagnostic from {name}")
    paths = [ROOT / "stdlib/Check/module.jai", ROOT / "stdlib/internal/developer-tools/check-engine/module.jai",
             ROOT / "stdlib/Check/tests/driven/check-plugin.jai", ROOT / "stdlib/Check/tests/driven/check-plugin-workspace.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths,
         ROOT / "stdlib/internal/developer-tools/porting-check.jai", ROOT / "stdlib/internal/developer-tools/check-bindings.jai",
         ROOT / "tests/compiler/rejections/check-plugin-porting.jai", ROOT / "tests/compiler/rejections/check-plugin-bindings.jai"])
    run([ROOT / "target/debug/jailint", "-D", "warnings", *paths])
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-os", "windows", "-cpu", "x64", *paths])
    print("Check persistent factory and configured real callback effects passed interpreter/native Mac O0/O2, diagnosed rejections, formatting and strict lint")


if __name__ == "__main__":
    main()
