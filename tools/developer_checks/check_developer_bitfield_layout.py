#!/usr/bin/env python3
"""Generated bitfield storage and accessors against the C compiler's layout of the same header."""
import platform
import re
from driven_sources import driven
from dev_common import JAIC, JAIFMT, ROOT, labeled_runner
from build_developer_bindings import llvm_prefix

run = labeled_runner("Bitfield layout", stdout=True)

# Targets the header is generated for on top of the host: (file suffix, jaic -os, jaic -cpu, clang triple).
# Their records are checked for layout only: the generated #assert lines compare Jai's placement with
# libclang's for that target, and the C compiler for that target must agree with every number.
TARGETS = [
    ("windows", "windows", "x64", "x86_64-pc-windows-msvc"),
    ("linux", "linux", "x64", "x86_64-unknown-linux-gnu"),
    ("linux-arm", "linux", "arm64", "aarch64-unknown-linux-gnu"),
    ("macos-arm", "macos", "arm64", "arm64-apple-macos"),
]

ASSERTION = re.compile(r'#assert\((size_of|align_of|offset_of)\((\w+)(?:, "(\w+)")?\) == (\d+)\)')


def layout_assertions(generated, keywords):
    """C `_Static_assert`s for the sizes, alignments and ordinary member offsets a generated file asserts."""
    lines = ['#include "bindings-bitfield-layout.h"', "#include <stddef.h>"]
    for kind, record, member, value in ASSERTION.findall(generated.read_text()):
        if record not in keywords or member.startswith("__"):
            continue
        record = f"{keywords[record]} {record}"
        subject = {"size_of": f"sizeof({record})", "align_of": f"_Alignof({record})"}.get(kind) or f"offsetof({record}, {member})"
        lines.append(f'_Static_assert({subject} == {value}, "{subject}");')
    if len(lines) < 40:
        raise RuntimeError(f"{generated}: too few layout assertions to check ({len(lines)})")
    return "\n".join(lines) + "\n"


def main():
    prefix = llvm_prefix()
    target = ROOT / "target"
    flags = ["-dynamiclib"] if platform.system() == "Darwin" else ["-shared", "-fPIC"]
    suffix = "dylib" if platform.system() == "Darwin" else "so"
    emitter = driven("bindings-bitfield-layout.jai")
    consumer = driven("bindings-bitfield-layout-roundtrip.jai")
    native = driven("bindings-bitfield-layout-native.c")
    header = driven("bindings-bitfield-layout.h")
    generated = [target / f"developer-tools-bitfield-layout-{name}.jai" for name in ["generated", "plain"] + [t[0] for t in TARGETS]]
    library = target / f"libdeveloper_tools_bitfield_layout.{suffix}"
    clang = [prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-Wno-comment", "-I", header.parent]

    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            run([JAIC, emitter, "-interp", "--", target], "ok\n")
        else:
            executable = target / f"developer-tools-bitfield-layout-emitter-o{mode}"
            run([JAIC, emitter, f"-O{mode}", "-exe", executable])
            run([executable, target], "ok\n")
        run([JAIFMT, "--config", ROOT / "jacket.toml", *generated])
        for optimization in [0, 2]:
            run([*clang, f"-O{optimization}", *flags, native, "-o", library])
            run([JAIC, consumer, "-interp", "-I", target], "ok\n")
            executable = target / f"developer-tools-bitfield-layout-consumer-{mode}-o{optimization}"
            run([JAIC, consumer, f"-O{optimization}", "-exe", executable, "-I", target])
            run([executable], "ok\n")

    keywords = {name: keyword for keyword, name in re.findall(r"^(struct|union)\s+(?:__attribute__\(\(packed\)\)\s+)?(Bf\w+)", header.read_text(), re.M)}
    for name, os_name, cpu, triple in TARGETS:
        check = target / f"developer-tools-bitfield-layout-{name}-check.jai"
        check.write_text(f'G :: #import "developer-tools-bitfield-layout-{name}";\nmain :: () {{}}\n')
        run([JAIC, check, "-check", "-os", os_name, "-cpu", cpu, "-I", target])
        facts = target / f"developer-tools-bitfield-layout-{name}-facts.c"
        facts.write_text(layout_assertions(target / f"developer-tools-bitfield-layout-{name}.jai", keywords))
        run([*clang, "--target=" + triple, "-fsyntax-only", facts])
    plain = target / "developer-tools-bitfield-layout-plain-check.jai"
    plain.write_text('G :: #import "developer-tools-bitfield-layout-plain";\nmain :: () {}\n')
    run([JAIC, plain, "-check", "-I", target])

    paths = [emitter, consumer, *generated]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths])
    run([JAIC.parent / "jailint", "-D", "warnings", "-I", target, *paths])
    print("Bitfield units and accessors matched C sizes, offsets and bytes for the host (interpreter, native O0/O2, C O0/O2); "
          "Windows/Linux/macOS generated layouts passed their assertions and the C compiler's layout for each target; no cross-target execution")


if __name__ == "__main__":
    main()
