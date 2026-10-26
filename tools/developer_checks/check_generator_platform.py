#!/usr/bin/env python3
"""Original host effects and official SDK object-format evidence for owned APIs."""
import json
import os
import platform
import re
import subprocess
import build_generator_platform
from build_developer_bindings import llvm_prefix
from dev_common import JAIFMT, ROOT, run_checked, sha256_bytes, sha256_file, stdlib_env, cargo_binary, output_dir

ENV = stdlib_env(TMPDIR=str(output_dir()))
PREFIX = llvm_prefix()


def run(command, expected=None, env=None):
    return run_checked(command, label="Generator platform", expected=expected, env=env or ENV, timeout=60).stdout


def fixture_lint_config():
    """The repository's lint settings without `exclude`: it lists `tests/driven/`, and a path named on
    the command line that the configuration excludes is skipped, but these fixtures are the point."""
    text = (ROOT / "jacket.toml").read_text()
    text = re.sub(r"^exclude = \[.*?^\]\n", "", text, flags=re.DOTALL | re.MULTILINE)
    config = output_dir("generator-platform-lint.toml")
    config.write_text(text)
    return config


def configured_paths_environment():
    base = output_dir("generator-platform-configured-roots")
    for triple in ["x86_64-linux-gnu", "aarch64-linux-gnu"]:
        (base / "linux/usr/include" / triple).mkdir(parents=True, exist_ok=True)
    for triple in ["x86_64-linux-android", "aarch64-linux-android"]:
        (base / "ndk/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/include" / triple).mkdir(parents=True, exist_ok=True)
    (base / "apple/usr/include").mkdir(parents=True, exist_ok=True)
    return dict(ENV, DEVELOPER_TOOLS_LINUX_SYSROOT=str(base / "linux"),
                ANDROID_NDK_HOME=str(base / "ndk"), DEVELOPER_TOOLS_APPLE_SDK_ROOT=str(base / "apple"))


def check_copy_behavior():
    source = output_dir("generator-platform-copy-source", "original.bin")
    work = output_dir("generator-platform-copy-work")
    source.parent.mkdir(parents=True, exist_ok=True)
    work.mkdir(parents=True, exist_ok=True)
    destination = work / source.name
    fixture = ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform-copy.jai"
    modes = [[cargo_binary("jaic"), fixture, "-interp"]]
    for optimization in [0, 2]:
        executable = output_dir(f"generator-platform-copy-o{optimization}")
        run([cargo_binary("jaic"), fixture, f"-O{optimization}", "-exe", executable], "")
        modes.append([executable])
    timestamp = 1700000000000000000
    cases = [("absent", None, 0, b"abcd"),
             ("older", b"WXYZ", timestamp - 1000000000, b"abcd"),
             ("newer", b"WXYZ", timestamp + 1000000000, b"WXYZ"),
             ("equal", b"WXYZ", timestamp, b"WXYZ"),
             ("different-size", b"!", timestamp + 1000000000, b"abcd"),
             ("older-nanoseconds", b"WXYZ", timestamp - 1, b"abcd")]
    for mode in modes:
        for label, local, local_time, expected in cases:
            source.write_bytes(b"abcd")
            os.utime(source, ns=(timestamp, timestamp))
            destination.unlink(missing_ok=True)
            if local is not None:
                destination.write_bytes(local)
                os.utime(destination, ns=(local_time, local_time))
            run(mode, "ok\n")
            if destination.read_bytes() != expected:
                raise RuntimeError(f"Reviewed libclang copy/update rule failed: {label}, {mode}")
        source.unlink()
        result = subprocess.run([str(x) for x in mode], cwd=ROOT, env=ENV,
                                capture_output=True, text=True, timeout=60)
        if result.returncode == 0 or "missing libclang source" not in result.stderr + result.stdout:
            raise RuntimeError("Missing source did not trigger the reviewed assertion")
    bundled = ROOT / "stdlib/internal/developer-tools/clang-bindings/libclang.dylib"
    copied_hash = sha256_file((work / bundled.name))
    bundled_hash = sha256_file(bundled)
    if copied_hash != bundled_hash:
        raise RuntimeError("No-argument helper did not copy the real configured SDK library")
    return {"cases": [case[0] for case in cases], "modes": 3,
            "missing_source_assertions": 3, "bundled_sdk_sha256": bundled_hash,
            "copied_sdk_sha256": copied_hash}


def main():
    if platform.system() != "Darwin":
        raise RuntimeError("This execution evidence requires a real macOS host")
    (output_dir()).mkdir(exist_ok=True)
    build_generator_platform.main()
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform.c"
    evidence = {"scope": "original objects; official SDK symbol tables; host interpreter/native O0/O2",
                "objects": {}, "sdk": run([PREFIX / "bin/clang", "--version"]).splitlines()[0]}
    variants = [("macho.o", None, 0), ("macho-o2.o", None, 2),
                ("elf.o", "x86_64-unknown-linux-gnu", 0),
                ("coff.o", "x86_64-pc-windows-msvc", 0)]
    for suffix, triple, optimization in variants:
        output = output_dir(f"generator-platform-{suffix}")
        flags = ["-target", triple] if triple else []
        run([PREFIX / "bin/clang", *flags, f"-O{optimization}", "-Wall", "-Wextra", "-Werror",
             "-c", source, "-o", output])
        oracle = run([PREFIX / "bin/llvm-readobj", "--symbols", output])
        for symbol in ["gp_add", "gp_value", "gp_unresolved", "gp_external"]:
            if symbol not in oracle:
                raise RuntimeError(f"SDK oracle lacks original symbol {symbol}")
        (output_dir(f"generator-platform-{suffix}.symbols.txt")).write_text(oracle)
        evidence["objects"][suffix] = {"sha256": sha256_file(output),
                                       "oracle_sha256": sha256_bytes(oracle.encode())}
    archive = output_dir("generator-platform-a.a")
    archive.unlink(missing_ok=True)
    run([PREFIX / "bin/llvm-ar", "rcs", archive, output_dir("generator-platform-macho.o"),
         output_dir("generator-platform-macho-o2.o")])
    fixture = ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform.jai"
    diagnostic_file = output_dir("diagnostic-only.o")
    diagnostic_file.write_text("preserve diagnostic path content")
    owned = [fixture, ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform-target.jai",
             ROOT / "stdlib/Bindings_Generator/platform-helpers.jai",
             ROOT / "stdlib/internal/developer-tools/generator-platform.jai",
             ROOT / "stdlib/Bindings_Generator/generator-state.jai",
             ROOT / "stdlib/internal/developer-tools/generator-active-state.jai",
             ROOT / "stdlib/internal/developer-tools/generator-session/module.jai",
             ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform-apple-names.jai",
             ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform-configured-paths.jai",
             ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform-reviewed.jai",
             ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform-copy.jai",
             ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform-session-probe.jai",
             ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform-canonical.jai"]
    for path in owned:
        run([JAIFMT, "--config", ROOT / "jacket.toml", path])
        run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", path])
    checked = output_dir("generator-platform-linted.txt")
    programs = [path for path in owned if path.parent.name == "driven"]
    run([cargo_binary("jailint"), "-D", "warnings", "--config", fixture_lint_config(), "--checked-files", checked,
         *programs])
    linted = checked.read_text().splitlines()
    if any(str(path) not in linted for path in programs):
        raise RuntimeError("Strict contextual fixture lint omitted an owned program: " + ", ".join(str(path) for path in programs if str(path) not in linted))
    for optimization in [0, 2]:
        run([PREFIX / "bin/clang", "-std=c11", f"-O{optimization}", "-Wall", "-Wextra", "-Werror", "-dynamiclib",
             ROOT / "stdlib/Bindings_Generator/tests/driven/generator-platform-shared.c",
             "-o", output_dir("libgenerator_platform_fixture.dylib")])
        run([cargo_binary("jaic"), fixture, "-interp"], "ok\n")
        executable = output_dir(f"generator-platform-o{optimization}")
        run([cargo_binary("jaic"), fixture, f"-O{optimization}", "-exe", executable], "")
        run([executable], "ok\n")
        for support in [owned[-6], owned[-5], owned[-4], *owned[-2:]]:
            environment = configured_paths_environment() if support.stem == "generator-platform-configured-paths" else ENV
            executable = output_dir(f"{support.stem}-o{optimization}")
            run([cargo_binary("jaic"), support, "-interp"], "ok\n", env=environment)
            run([cargo_binary("jaic"), support, f"-O{optimization}", "-exe", executable], "")
            run([executable], "ok\n", env=environment)
    for os_name in ["macos", "linux", "windows"]:
        for cpu in ["x64", "arm64"]:
            run([cargo_binary("jaic"), owned[1], "-check", "-os", os_name, "-cpu", cpu], "")
            for support in owned[-2:]:
                run([cargo_binary("jaic"), support, "-check", "-os", os_name, "-cpu", cpu], "")
    evidence["libclang_copy"] = check_copy_behavior()
    if list((output_dir()).glob("jai-symbols-*")):
        raise RuntimeError("Object staging leaked temporary files")
    evidence["executed"] = ["interpreter", "Mac native O0", "Mac native O2"]
    if diagnostic_file.read_text() != "preserve diagnostic path content":
        raise RuntimeError("Content enumeration overwrote its diagnostic-only path")
    evidence["source_checked_only"] = ["Linux x64/arm64", "Windows x64/arm64", "Mac x64"]
    evidence["canonical_source_commit"] = "4675c90a"
    evidence["support_probes"] = ["canonical record storage and native cursor hash callbacks",
                                  "test-only private context session save/restore, nested deferred return and push_context isolation",
                                  "reviewed active Generator first-match lookup and persistent exact-key symbol cache ownership",
                                  "actual Mac SDK libc root and original configured Linux/Android/Apple directory-selection fixtures",
                                  "reviewed libclang copy/update/missing-source effects, including actual bundled SDK library bytes"]
    evidence["contextual_fixture_lint"] = "passed with warnings denied"
    try:
        run([cargo_binary("jailint"), "-D", "warnings", "--config", fixture_lint_config(), "--checked-files", checked,
             ROOT / "stdlib/Bindings_Generator/module.jai", *owned])
    except RuntimeError as error:
        evidence["module_root_lint"] = "failed; independent #add_context-after-bootstrap driver ordering reduction reported"
        (output_dir("generator-platform-evidence.json")).write_text(json.dumps(evidence, indent=2) + "\n")
        raise error
    evidence["module_root_lint"] = "passed with warnings denied"
    (output_dir("generator-platform-evidence.json")).write_text(json.dumps(evidence, indent=2) + "\n")
    print("Generator platform original-object/library effects passed interpreter/native Mac O0/O2; six desktop source checks passed")


if __name__ == "__main__":
    main()
