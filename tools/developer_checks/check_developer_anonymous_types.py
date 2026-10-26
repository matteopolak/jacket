#!/usr/bin/env python3
"""Reviewed anonymous typedef normalization and real C ABI consumers."""
import platform
from developer_checks.driven_sources import driven
from developer_checks.dev_common import JAIFMT, ROOT, labeled_runner, cargo_binary, output_dir
from developer_checks.build_developer_bindings import llvm_prefix


run = labeled_runner('Anonymous typedef workflow', stdout=True)


def main():
    prefix = llvm_prefix()
    native = ROOT / "stdlib/internal/developer-tools/clang-bindings"
    oracle = output_dir("developer-tools-anonymous-oracle")
    run([prefix / "bin/clang", "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
         "-I", prefix / "include", driven("bindings-anonymous-oracle.c"), "-L", native,
         "-Wl,-rpath," + str(native), "-lclang", "-o", oracle])
    observations = [line.split("|") for line in run([oracle, driven("bindings-anonymous.h")], None).splitlines()]
    # SDK inferred tag spelling/anonymous-query values are observations, not
    # compatibility oracles. Source location and canonical identity are tested.
    if [(row[0], row[2], row[5]) for row in observations] != [
            ("Anonymous_Record", "2", "struct"), ("Anonymous_Union", "3", "union"),
            ("Anonymous_Mode", "5", "enum"), ("Record_Alias", "2", "struct"),
            ("Mode_Alias", "5", "enum"), ("Forward_Record", "2", "Forward_Record")]:
        raise RuntimeError("SDK tag location/identity did not match original source")
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    jaic = cargo_binary("jaic")
    emitter = driven("bindings-anonymous.jai")
    consumer = driven("bindings-anonymous-roundtrip.jai")
    generated = output_dir("developer-tools-anonymous-generated.jai")
    default_generated = output_dir("developer-tools-anonymous-default.jai")
    # The generator loads the configured library, so it must exist before the first generation.
    run([prefix / "bin/clang", "-std=c11", "-O0", *flags, driven("bindings-anonymous-native.c"),
         "-o", output_dir(f"libdeveloper_tools_anonymous.{suffix}")])
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([jaic, emitter, "-interp"], "ok\n")
        else:
            executable = output_dir(f"developer-tools-anonymous-emitter-o{mode}")
            run([jaic, emitter, f"-O{mode}", "-exe", executable])
            run([executable], "ok\n")
        run([JAIFMT, "--config", ROOT / "jacket.toml", generated, default_generated])
        for optimization in [0, 2]:
            run([prefix / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
                 *flags, driven("bindings-anonymous-native.c"), "-o", output_dir(f"libdeveloper_tools_anonymous.{suffix}")])
            run([jaic, consumer, "-interp", "-I", output_dir()], "ok\n")
            executable = output_dir(f"developer-tools-anonymous-consumer-{mode}-o{optimization}")
            run([jaic, consumer, f"-O{optimization}", "-exe", executable, "-I", output_dir()])
            run([executable], "ok\n")
    paths = [emitter, consumer, generated, default_generated, ROOT / "stdlib/Bindings_Generator/module.jai",
             ROOT / "stdlib/internal/developer-tools/binding-native-model.jai"]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), *paths])
    for triple, os_tag, cpu in [("x86_64-pc-windows-msvc", "windows", "x64"),
                                ("aarch64-unknown-linux-gnu", "linux", "arm64")]:
        run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-target", triple,
             "-c", driven("bindings-anonymous-native.c"), "-o", output_dir(f"developer-tools-anonymous-{triple}.o")])
        run([jaic, consumer, "-check", "-os", os_tag, "-cpu", cpu, "-I", output_dir()])
    print("Anonymous record/union/enum naming, omitted naming aliases, retained secondary aliases and forward identity passed interpreter/native Mac O0/O2, C O0/O2, format and strict lint; Windows/Linux target object/source checks only")


if __name__ == "__main__":
    main()
