#!/usr/bin/env python3
"""Build the independently authored generator host operations from this checkout."""

import argparse
from pathlib import Path
import platform
import subprocess
from toolkit.host import exe_name, require_tool
from developer_checks import build_developer_bindings
from developer_checks.dev_common import ROOT


def sdk_tool(prefix, name, xcode_name):
    """`prefix/bin/name` of the SDK; on macOS `xcode_name` from PATH (Xcode's tool) instead.

    macOS always uses Xcode's tools: the x64 SDK has no clang and its llvm-ar cannot load libLLVM.
    Either way a missing tool exits naming it; there is no second choice.
    """
    if platform.system() == "Darwin":
        return require_tool(xcode_name, "install Xcode's command line tools (`xcode-select --install`)")
    tool = Path(prefix) / "bin" / exe_name(name)
    if not tool.is_file():
        raise SystemExit(f"{tool} does not exist; point --llvm-prefix at an LLVM install that has bin/{name}")
    return str(tool)


def build(prefix=None, output_dir=None, record_nm=True):
    """Build the helper library into `output_dir`.

    `record_nm` also writes the path of the SDK's llvm-nm beside it, which a checkout uses; a
    release archive does not (the build machine's paths mean nothing there, tools/ci/package_bindings_helpers.py).
    """
    directory = ROOT / "stdlib/internal/developer-tools"
    output = Path(output_dir or directory).resolve()
    output.mkdir(parents=True, exist_ok=True)
    prefix = Path(prefix) if prefix and not record_nm else build_developer_bindings.llvm_prefix(prefix)
    system = platform.system()
    suffix = {"Darwin": "dylib", "Linux": "so", "Windows": "dll"}[system]
    flags = ["-dynamiclib"] if system == "Darwin" else ["-shared"]
    if system == "Darwin":
        flags += ["-Wl,-install_name,@rpath/libgenerator_platform.dylib"]
    libraries = []
    if system == "Linux":
        flags += ["-fPIC"]
        libraries += ["-ldl"]
    subprocess.run(
        [
            sdk_tool(prefix, "clang", "clang"),
            "-std=c11",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            *flags,
            str(directory / "generator-platform-native.c"),
            "-o",
            str(output / f"libgenerator_platform.{suffix}"),
            *libraries,
        ],
        check=True,
        cwd=ROOT,
    )
    if record_nm:
        (output / "generator-platform-nm.txt").write_text(str(prefix / "bin/llvm-nm"))
    return output / f"libgenerator_platform.{suffix}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--llvm-prefix", type=Path)
    args = parser.parse_args()
    build(args.llvm_prefix, args.output_dir)


if __name__ == "__main__":
    main()
