#!/usr/bin/env python3
"""Original host effects and official SDK object-format evidence for owned APIs."""
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import build_generator_platform

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, JAIC_STDLIB=str(ROOT / "stdlib"), TMPDIR=str(ROOT / "target"))
PREFIX = Path(os.environ.get("DEVELOPER_TOOLS_LLVM_PREFIX", "/opt/homebrew/opt/llvm"))


def run(command, expected=None):
    result = subprocess.run([str(x) for x in command], cwd=ROOT, env=ENV,
                            text=True, capture_output=True, timeout=60)
    if result.returncode or result.stderr or (expected is not None and result.stdout != expected):
        raise RuntimeError(f"{command}: status={result.returncode}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def main():
    if platform.system() != "Darwin":
        raise RuntimeError("This execution evidence requires a real macOS host")
    (ROOT / "target").mkdir(exist_ok=True)
    build_generator_platform.main()
    source = ROOT / "tests/developer-tools/generator-platform.c"
    evidence = {"scope": "original objects; official SDK symbol tables; host interpreter/native O0/O2",
                "objects": {}, "sdk": run([PREFIX / "bin/clang", "--version"]).splitlines()[0]}
    variants = [("macho.o", None, 0), ("macho-o2.o", None, 2),
                ("elf.o", "x86_64-unknown-linux-gnu", 0),
                ("coff.o", "x86_64-pc-windows-msvc", 0)]
    for suffix, triple, optimization in variants:
        output = ROOT / f"target/generator-platform-{suffix}"
        flags = ["-target", triple] if triple else []
        run([PREFIX / "bin/clang", *flags, f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
             "-c", source, "-o", output])
        oracle = run([PREFIX / "bin/llvm-readobj", "--symbols", output])
        for symbol in ["gp_add", "gp_value", "gp_unresolved", "gp_external"]:
            if symbol not in oracle:
                raise RuntimeError(f"SDK oracle lacks original symbol {symbol}")
        (ROOT / f"target/generator-platform-{suffix}.symbols.txt").write_text(oracle)
        evidence["objects"][suffix] = {"sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
                                       "oracle_sha256": hashlib.sha256(oracle.encode()).hexdigest()}
    archive = ROOT / "target/generator-platform-a.a"
    archive.unlink(missing_ok=True)
    run([PREFIX / "bin/llvm-ar", "rcs", archive, ROOT / "target/generator-platform-macho.o",
         ROOT / "target/generator-platform-macho-o2.o"])
    fixture = ROOT / "tests/developer-tools/generator-platform.jai"
    diagnostic_file = ROOT / "target/diagnostic-only.o"
    diagnostic_file.write_text("preserve diagnostic path content")
    owned = [fixture, ROOT / "tests/developer-tools/generator-platform-target.jai",
             ROOT / "stdlib/Bindings_Generator/platform-helpers.jai",
             ROOT / "stdlib/internal/developer-tools/generator-platform.jai",
             ROOT / "stdlib/Bindings_Generator/generator-state.jai",
             ROOT / "tests/developer-tools/generator-platform-canonical.jai"]
    for path in owned:
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", path])
        run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", "--check", path])
    checked = ROOT / "target/generator-platform-linted.txt"
    run([ROOT / "target/debug/jailint", "-D", "warnings", "--checked-files", checked,
         ROOT / "stdlib/Bindings_Generator/module.jai", fixture, owned[1], owned[-1]])
    linted = checked.read_text().splitlines()
    if any(str(path) not in linted for path in owned):
        raise RuntimeError("Integrated strict lint omitted an owned Jai source")
    for optimization in [0, 2]:
        run([PREFIX / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", "-dynamiclib",
             ROOT / "tests/developer-tools/generator-platform-shared.c",
             "-o", ROOT / "target/libgenerator_platform_fixture.dylib"])
        run([ROOT / "target/debug/jaic", fixture, "-interp"], "ok\n")
        executable = ROOT / f"target/generator-platform-o{optimization}"
        run([ROOT / "target/debug/jaic", fixture, f"-O{optimization}", "-exe", executable], "")
        run([executable], "ok\n")
        canonical = ROOT / f"target/generator-platform-canonical-o{optimization}"
        run([ROOT / "target/debug/jaic", owned[-1], "-interp"], "ok\n")
        run([ROOT / "target/debug/jaic", owned[-1], f"-O{optimization}", "-exe", canonical], "")
        run([canonical], "ok\n")
    # Exercise actual discovery with the local link absent, and restore precisely
    # the source-built tooling configuration even when the check fails.
    link = ROOT / "stdlib/internal/developer-tools/clang-bindings/libclang.dylib"
    if not link.is_symlink():
        raise RuntimeError("Discovery check requires the source-built SDK symlink")
    original = link.readlink()
    previous = ENV.get("DEVELOPER_TOOLS_LLVM_PREFIX")
    try:
        link.unlink()
        ENV["DEVELOPER_TOOLS_LLVM_PREFIX"] = str(PREFIX)
        run([ROOT / "target/debug/jaic", fixture, "-interp"], "ok\n")
        if not link.is_symlink() or link.resolve() != original.resolve():
            raise RuntimeError("SDK discovery did not configure the actual official libclang")
    finally:
        link.unlink(missing_ok=True)
        link.symlink_to(original)
        if previous is None:
            ENV.pop("DEVELOPER_TOOLS_LLVM_PREFIX", None)
        else:
            ENV["DEVELOPER_TOOLS_LLVM_PREFIX"] = previous
    for os_name in ["macos", "linux", "windows"]:
        for cpu in ["x64", "arm64"]:
            run([ROOT / "target/debug/jaic", owned[1], "-check", "-os", os_name, "-cpu", cpu], "")
            run([ROOT / "target/debug/jaic", owned[-1], "-check", "-os", os_name, "-cpu", cpu], "")
    if list((ROOT / "target").glob("jai-symbols-*")):
        raise RuntimeError("Object staging leaked temporary files")
    evidence["executed"] = ["interpreter", "Mac native O0", "Mac native O2"]
    if diagnostic_file.read_text() != "preserve diagnostic path content":
        raise RuntimeError("Content enumeration overwrote its diagnostic-only path")
    evidence["source_checked_only"] = ["Linux x64/arm64", "Windows x64/arm64", "Mac x64"]
    (ROOT / "docs/native/generator-platform-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
    print("Generator platform original-object/library effects passed interpreter/native Mac O0/O2; six desktop source checks passed")


if __name__ == "__main__":
    main()
