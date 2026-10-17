#!/usr/bin/env python3
"""Build and execute the real FSEvents watcher tests against the production stdlib (macOS)."""
import argparse
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, type=Path)
    parser.add_argument("--stdlib", required=True, type=Path)
    args = parser.parse_args()
    if platform.system() != "Darwin":
        raise RuntimeError("the implemented watcher backend currently requires macOS")
    compiler = args.jaic.resolve(strict=True)
    stdlib = args.stdlib.resolve(strict=True)
    output = ROOT / "target/portable-tests"
    output.mkdir(parents=True, exist_ok=True)
    native = output / "watcher-c-check"
    subprocess.run(
        ["clang", "-std=c11", "-O1", "-g", "-fblocks", "-Wall", "-Wextra", "-Werror",
         "-fsanitize=address,undefined", "-fno-omit-frame-pointer",
         str(ROOT / "tests/portable-stdlib/native/watcher-c-check.c"),
         "-framework", "CoreServices", "-pthread", "-o", str(native)],
        cwd=ROOT, check=True)
    fixture = output / "native-watch-fixtures"
    if fixture.exists():
        shutil.rmtree(fixture)
    fixture.mkdir()
    result = subprocess.run([str(native), str(fixture.resolve())], cwd=ROOT, check=True,
                            text=True, capture_output=True, timeout=30)
    assert result.stdout == "native watcher queue/overflow/lifetime checks passed\n" and result.stderr == "", result
    shutil.rmtree(fixture)
    env = dict(os.environ, JAIC_STDLIB=str(stdlib))
    commands = []
    targets = [([], "host")]
    if platform.machine() == "arm64":
        targets.append((["-target", "x86_64-apple-darwin", "-cpu", "x64"], "translated-x64"))
    for fixture_name in ["main", "diagnostics"]:
        source = ROOT / f"tests/portable-stdlib/file-watcher/{fixture_name}.jai"
        commands.append([str(compiler), "-interp", str(source), "--"])
        for flags, name in targets:
            for optimization in ["-O0", "-O2"]:
                program = output / f"file-watcher-{fixture_name}-{name}{optimization}"
                subprocess.run([str(compiler), str(source), *flags, optimization, "-exe", str(program)],
                               cwd=ROOT, env=env, check=True, timeout=60)
                commands.append([str(program)])
    for command in commands:
        result = subprocess.run(command, cwd=ROOT, env=env, check=True,
                                text=True, capture_output=True, timeout=30)
        assert result.stdout == "ok\n" and result.stderr == "", result
    print("Real FSEvents event/callback/recursion/overflow/captured cleanup passes interpreter and "
          + ",".join(name for _, name in targets) + " native O0/O2; actual SDK mutex close/retry "
          "ownership checks pass ASan/UBSan and allocation tracking")


if __name__ == "__main__":
    sys.exit(main())
