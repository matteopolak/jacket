#!/usr/bin/env python3
"""Put the native libraries a release archive needs into it, as native-libs.json and the stdlib's `@native` notes declare.

    python3 tools/native/package_native_libs.py --platform linux-x64 --package jacket-linux-x64 --llvm-prefix DIR [--dry-run]

For the unpacked archive `--package` (its `stdlib/` and `native-libs/<platform>/`), in order:

1. `runtime` libraries: C the standard library binds that is built into `stdlib/` (the Windows crash runtime).
2. `fetched` libraries: prebuilt, pinned and checksum-verified downloads (wgpu-native).
3. every library of `libraries`: built from pinned sources by tools/native/build_native_libs.py.
4. `packaged` libraries: built by their own script (Bindings_Generator's helper).
5. tools/ci/check_native_libs.py: every library the packaged stdlib binds is in the archive, or declared
   by a note (embedded, developer or external) on purpose.

A `runtime`, `fetched` or `packaged` library has a recipe in tools/native/provided_libs.py: its `script` runs with
its `args`, in which `{platform}`, `{cpu}`, `{stdlib}`, `{out}` and `{llvm_prefix}` are replaced. `--dry-run` prints
the commands.
"""

import argparse
import subprocess
import sys
from pathlib import Path

from native.provided_libs import on_platform, providers
from toolkit.repo import ROOT


def expand(args, values):
    return [argument.format(**values) for argument in args]


def commands(platform, package, llvm_prefix):
    """The command lines that fill the archive directory `package`, in order."""
    declared, problems = providers(ROOT / "stdlib")
    if problems:
        raise SystemExit("\n".join(problems))
    out = package / "native-libs" / platform
    values = {
        "platform": platform,
        "cpu": "arm64" if platform.endswith("arm64") else "x64",
        "stdlib": str(package / "stdlib"),
        "out": str(out),
        "llvm_prefix": str(llvm_prefix),
    }

    def declared_steps(kind):
        # One command per script: several libraries can share a script.
        scripts = {}
        for name, provider in declared.items():
            if provider.kind == kind and on_platform(provider, platform):
                scripts.setdefault(provider.detail["script"], provider.detail.get("args", []))
        return [[sys.executable, script, *expand(args, values)] for script, args in sorted(scripts.items())]

    build = [sys.executable, "tools/native/build_native_libs.py", "--platform", platform, "--out", str(out)]
    check = [sys.executable, "tools/ci/check_native_libs.py", "--stdlib", values["stdlib"], "--platform", platform,
             "--native", str(out)]
    return [*declared_steps("runtime"), *declared_steps("fetched"), build, *declared_steps("packaged"), check]


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--platform", required=True, help="the archive's platform, e.g. linux-x64 or windows-arm64")
    parser.add_argument("--package", type=Path, required=True, help="the unpacked archive directory, with its stdlib/")
    parser.add_argument("--llvm-prefix", type=Path, required=True)
    parser.add_argument("--dry-run", action="store_true", help="print the commands instead of running them")
    args = parser.parse_args()
    for command in commands(args.platform, args.package.resolve(), args.llvm_prefix):
        print("+ " + " ".join(command), flush=True)
        if not args.dry_run:
            subprocess.run(command, cwd=ROOT, check=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
