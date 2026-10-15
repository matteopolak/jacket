#!/usr/bin/env python3
"""Source-build and execute actual FSEvents tests; no peer checkout is modified."""
import os
from pathlib import Path
import platform
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
if platform.system() != "Darwin":
    raise RuntimeError("the implemented watcher backend currently requires macOS")
subprocess.run(["python3", "-B", str(ROOT / "tools/build_portable_watcher.py")], cwd=ROOT, check=True)
stdlib = ROOT / "target/portable-integration-stdlib"
for module in ["File_Watcher", "internal/watching"]:
    shutil.copytree(ROOT / "stdlib" / module, stdlib / module, dirs_exist_ok=True)
for module in ["macOS", "Objective_C"]:
    shutil.copytree(ROOT.parent / "bindings/stdlib" / module, stdlib / module, dirs_exist_ok=True)
output = ROOT / "target/portable-tests"
output.mkdir(parents=True, exist_ok=True)
native = output / "watcher-c-check"
subprocess.run(["clang", "-std=c11", "-O1", "-g", "-fblocks", "-Wall", "-Wextra", "-Werror",
                "-fsanitize=address,undefined", "-fno-omit-frame-pointer",
                str(ROOT / "tests/portable-stdlib/native/watcher-c-check.c"),
                "-framework", "CoreServices", "-pthread", "-o", str(native)], cwd=ROOT, check=True)
fixture = output / "native-watch-fixtures"
if fixture.exists():
    shutil.rmtree(fixture)
fixture.mkdir()
result = subprocess.run([str(native), str(fixture.resolve())], cwd=ROOT, check=True,
                        text=True, capture_output=True, timeout=30)
assert result.stdout == "native watcher queue/overflow/lifetime checks passed\n" and result.stderr == "", result
shutil.rmtree(fixture)
compiler = ROOT / "target/compiler/debug/jaic"
source = ROOT / "tests/portable-stdlib/file-watcher/main.jai"
program = output / "file-watcher"
env = dict(os.environ, JAIC_STDLIB=str(stdlib))
subprocess.run([str(compiler), str(source), "-exe", str(program)], cwd=ROOT, env=env, check=True, timeout=60)
for command in [[str(compiler), "-interp", str(source), "--"], [str(program)]]:
    result = subprocess.run(command, cwd=ROOT, env=env, check=True,
                            text=True, capture_output=True, timeout=30)
    assert result.stdout == "ok\n" and result.stderr == "", result
print("Real FSEvents create/modify/remove/rename, typed/coalesced callbacks, recursion, overflow and captured allocation cleanup pass both modes; native queue passes ASan/UBSan and allocation tracking")
