#!/usr/bin/env python3
"""Original POSIX SDK signal/assertion lifecycle effects, separate from Windows."""
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))
BUILD = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
JAIC = BUILD / "debug/jaic"


def invoke(command, *, code=0, output=None, diagnostic=None):
    result = subprocess.run([str(item) for item in command], cwd=ROOT, env=ENV,
                            capture_output=True, text=True, timeout=90)
    if (result.returncode != code or (output is not None and result.stdout != output)
            or (diagnostic is not None and diagnostic not in result.stderr)):
        raise RuntimeError(f"Debug lifecycle failure: {command}; status={result.returncode}; "
                           f"diagnostic={result.stderr.splitlines()[:2]}")
    return result


def main():
    if platform.system() not in ("Darwin", "Linux"):
        raise RuntimeError("POSIX Debug lifecycle fixture requires a supported native host")
    prefix = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    invoke([prefix / "bin/clang", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", *flags,
            ROOT / "tests/developer-tools/debug-lifecycle-native.c", "-o",
            ROOT / f"target/libdeveloper_tools_debug_lifecycle.{suffix}"])
    for name in ["debug-lifecycle", "debug-native", "debug-default-assert", "debug-ignored-locations", "debug-abort"]:
        source = ROOT / f"tests/developer-tools/{name}.jai"
        for mode in ["interp", "0", "2"]:
            command = [JAIC, source, "-interp"]
            if mode != "interp":
                executable = ROOT / f"target/developer-tools-{name}-o{mode}"
                invoke([JAIC, source, f"-O{mode}", "-exe", executable], output="")
                command = [executable]
            if name == "debug-abort":
                invoke(command, code=2, output="")
            elif name == "debug-default-assert":
                invoke(command, code=3, output="", diagnostic="original failing assertion")
            else:
                expected = {"debug-lifecycle": "original trace", "debug-native": "native debug output"}.get(name)
                result = invoke(command, output="ok\n", diagnostic=expected)
                if name == "debug-lifecycle":
                    lines = result.stderr.splitlines()
                    if len(lines) < 3 or "original assertion" not in lines[0]:
                        raise RuntimeError("Assertion diagnostic or actual trace is missing")
    # Fatal native signal delivery uses the actual compiled Jai callback. Interpreter
    # callback execution is not async-signal-safe and is deliberately not asserted.
    for name, diagnostic in [("debug-crash-callback", "original crash callback"),
                             ("debug-owned-context", "original owned context callback"),
                             ("debug-lifecycle-concurrent", "original concurrent context callback")]:
        source = ROOT / f"tests/developer-tools/{name}.jai"
        for mode in [0, 2]:
            executable = ROOT / f"target/developer-tools-{name}-o{mode}"
            invoke([JAIC, source, f"-O{mode}", "-exe", executable], output="")
            result = invoke([executable], code=4, output="", diagnostic=diagnostic)
            if result.stderr != diagnostic + "\n":
                raise RuntimeError("Unexpected crash callback output")
    paths = [ROOT / "stdlib/Debug/module.jai", ROOT / "stdlib/internal/developer-tools/debug-lifecycle.jai",
             ROOT / "stdlib/internal/developer-tools/debug-assert-storage.jai",
             *[ROOT / f"tests/developer-tools/{name}.jai" for name in
               ["debug-lifecycle", "debug-crash-callback", "debug-default-assert",
                "debug-owned-context", "debug-lifecycle-concurrent", "debug-ignored-locations", "debug-abort"]]]
    invoke([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", *paths], output="")
    invoke([BUILD / "debug/jailint", "-D", "warnings", *paths], output="")
    print("Debug assertion ownership/diagnostics and POSIX lifecycle passed interpreter/native Mac O0/O2; fatal callbacks passed native O0/O2")


if __name__ == "__main__":
    main()
