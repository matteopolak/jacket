#!/usr/bin/env python3
"""Build and validate the private, staged libclang C binding pipeline."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import build_developer_bindings

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"))


def run(command, expected="", returncode=0, diagnostic=None):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, env=ENV,
                            text=True, capture_output=True, timeout=60)
    if result.returncode != returncode or (expected is not None and result.stdout != expected):
        raise RuntimeError(f"{command}: exit {result.returncode}\n{result.stdout}\n{result.stderr}")
    if diagnostic is None and result.stderr:
        raise RuntimeError(result.stderr)
    if diagnostic is not None and diagnostic not in result.stderr:
        raise RuntimeError(f"Missing diagnostic {diagnostic}: {result.stderr}")
    return result.stdout


def main():
    prefix = build_developer_bindings.llvm_prefix()
    tool = build_developer_bindings.build(prefix)
    source = ROOT / "stdlib/internal/developer-tools/clang-bindings/main.c"
    output = ROOT / "target/developer-tools-generated.jai"
    header = ROOT / "tests/developer-tools/bindings-input.h"
    run([tool, header, output, "developer_tools_bindings"])
    before = output.read_bytes()
    run([tool, header, output, "developer_tools_bindings"])
    if output.read_bytes() != before:
        raise RuntimeError("Binding generation is not deterministic")
    run([prefix / "bin/clang", "-std=c11", "-Wall", "-Wextra", "-Werror", *build_developer_bindings.shared_flags(),
         ROOT / "tests/developer-tools/bindings-input.c", "-o", build_developer_bindings.shared_library("developer_tools_bindings")])
    jaic = ROOT / "target/debug/jaic"
    fixture = ROOT / "tests/developer-tools/bindings-roundtrip.jai"
    run([jaic, fixture, "-interp", "-I", ROOT / "target"], "ok\n")
    for optimization in [0, 2]:
        exe = ROOT / f"target/developer-tools-bindings-o{optimization}"
        run([jaic, fixture, f"-O{optimization}", "-exe", exe, "-I", ROOT / "target"])
        run([exe], "ok\n")
    rejected = ROOT / "target/developer-tools-rejected.jai"
    run([tool, ROOT / "tests/developer-tools/bindings-macro.h", output, "native"])
    if "NUMERIC_MACRO :: cast(s32) 17;" not in output.read_text():
        raise RuntimeError("Original numeric macro must now generate an actual typed constant")
    for case, reason in [("bitfield", "bitfield member"), ("malformed", "error:"),
                         ("macro-function", "function-like macro"),
                         ("macro-string", "not a supported numeric constant expression"),
                         ("macro-nonfinite", "nonfinite floating macro")]:
        rejected.write_text("existing output must survive")
        run([tool, ROOT / f"tests/developer-tools/bindings-{case}.h", rejected, "native"],
            returncode=1, diagnostic=reason)
        if rejected.read_text() != "existing output must survive":
            raise RuntimeError(f"{case} replaced output despite generation failure")
    cpp_header = ROOT / "tests/developer-tools/bindings-cpp-input.h"
    run([prefix / "bin/clang++", "-x", "c++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-fsyntax-only", cpp_header])
    for triple in ["aarch64-apple-macos", "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"]:
        rejected.write_text("existing output must survive")
        run([tool, cpp_header, rejected, "native", f"--target={triple}", "-x", "c++", "-std=c++17"],
            returncode=1, diagnostic="duplicate C bridge output name")
        if rejected.read_text() != "existing output must survive":
            raise RuntimeError("Colliding C++ adapter names replaced the destination")
    for triple, os_name, spelling in [("x86_64-unknown-linux-gnu", "linux", "NativeLong :: s64;"),
                                      ("x86_64-pc-windows-msvc", "windows", "NativeLong :: s32;")]:
        run([tool, header, output, "developer_tools_bindings", f"--target={triple}"])
        if spelling not in output.read_text():
            raise RuntimeError(f"Target-dependent C long ABI was not preserved: {triple}")
        run([jaic, fixture, "-check", "-os", os_name, "-cpu", "x64", "-I", ROOT / "target"])
    run([tool, header, output, "developer_tools_bindings"])
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", output, fixture])
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", output, fixture])
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", output, fixture])
    macro_header = ROOT / "tests/developer-tools/bindings-macro-values.h"
    macro_output = ROOT / "target/developer-tools-macros-generated.jai"
    macro_fixture = ROOT / "tests/developer-tools/bindings-macros-roundtrip.jai"
    for triple, os_name, width in [("x86_64-unknown-linux-gnu", "linux", 8),
                                    ("x86_64-pc-windows-msvc", "windows", 4)]:
        run([tool, macro_header, macro_output, "developer_tools_macros", f"--target={triple}"])
        if f"ORIGINAL_LONG_BYTES :: cast(u64) {width};" not in macro_output.read_text():
            raise RuntimeError("Macro sizeof must use the requested target ABI")
        run([jaic, macro_fixture, "-check", "-os", os_name, "-cpu", "x64", "-I", ROOT / "target"])
    run([tool, macro_header, macro_output, "developer_tools_macros"])
    run([tool, macro_header, macro_output, "developer_tools_macros", "-pedantic-errors", "-Wall", "-Wextra", "-Werror"])
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", macro_output])
    run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", macro_output])
    run([ROOT / "target/debug/jailint", "-D", "warnings", "-I", ROOT / "target", macro_output])
    inputs = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
              for path in [prefix / "include/clang-c/Index.h", prefix / "include/clang-c/CXString.h",
                           prefix / "include/clang-c/BuildSystem.h"]}
    metadata = {"scope": "staged libclang backend used by the supported public Bindings_Generator facade; full compatibility remains pending",
                "sdk_headers_sha256": inputs, "clang_version": run([prefix / "bin/clang", "--version"], expected=None),
                "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                "tests": "interpreter/native O0/O2 call and mutation, callbacks, enum and record layout; rejection preserves output; Linux/Windows target-dependent long"}
    (ROOT / "docs/native/developer-tools-clang-evidence.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print("Private libclang pipeline: C/Jai effects and callback round trips, rejection preservation, deterministic output and two target ABI checks passed")


if __name__ == "__main__":
    main()
