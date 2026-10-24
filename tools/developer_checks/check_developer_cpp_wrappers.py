#!/usr/bin/env python3
"""Generated wrappers for C++ virtual methods, called against objects built by a C++ library."""
import os
import platform
import shutil

from dev_common import ENV, JAIC, JAIFMT, ROOT, labeled_runner
from driven_sources import driven
from repo_output import output_dir
import build_developer_bindings


def main():
    prefix = build_developer_bindings.llvm_prefix()
    work = output_dir("cpp-wrappers")
    shutil.rmtree(work, ignore_errors=True)
    work.mkdir(parents=True, exist_ok=True)
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    library = work / f"libcppwrappers.{suffix}"
    env = dict(ENV, JAIC_NATIVE_LIBS=os.pathsep.join(filter(None, [str(work), str(prefix / "lib"), ENV.get("JAIC_NATIVE_LIBS")])))
    run = labeled_runner("C++ wrappers", env=env, timeout=180)

    emitter = driven("bindings-cpp-wrappers.jai")
    native = driven("bindings-cpp-wrappers-native.cpp")
    consumer = work / "roundtrip.jai"
    consumer.write_text(driven("bindings-cpp-wrappers-roundtrip.jai").read_text())
    generated = [work / "developer-tools-cpp-wrappers-shapes.jai", work / "developer-tools-cpp-wrappers-sdk.jai"]

    for mode in ["interp", "2"]:
        if mode == "interp":
            run([JAIC, emitter, "-interp", "--", work], "ok\n")
        else:
            emitter_exe = work / "emitter-o2"
            run([JAIC, emitter, "-O2", "-exe", emitter_exe])
            run([emitter_exe, work], "ok\n")
        run([JAIFMT, "--config", ROOT / "jacket.toml", *generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang++", "-std=c++17", "-Wall", "-Wextra", "-Werror", f"-O{optimization}", *flags,
                 "-I", native.parent, native, "-o", library])
            run([JAIC, consumer, "-interp", "-I", work], "ok\n")
            executable = work / f"roundtrip-{mode}-o{optimization}"
            run([JAIC, consumer, f"-O{optimization}", "-I", work, "-exe", executable])
            run([executable], "ok\n")

    paths = [emitter, driven("bindings-cpp-wrappers-roundtrip.jai"), *generated]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([JAIC.parent / "jailint", "-D", "warnings", "-I", work, *paths])
    print("C++ virtual method wrappers called interface, derived-pointer, overload, reference, aggregate and "
          "COM-style SDK methods on a C++ library (interpreter, native O0/O2)")


if __name__ == "__main__":
    main()
