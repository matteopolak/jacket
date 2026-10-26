#!/usr/bin/env python3
"""SDK-derived mangled signatures through staged Windows-only Check on the Mac host."""
import json
from pathlib import Path
import platform
import re
from developer_checks import build_developer_check
from developer_checks.dev_common import JAIFMT, ROOT, run_checked, sha256_file, cargo_binary, output_dir
from developer_checks.build_developer_bindings import llvm_prefix


def run(command, *, code=0, expected=None):
    return run_checked(command, label="Check fixture", code=code, expected=expected, stderr=None)


def prepare():
    prefix = llvm_prefix()
    # Stage the actual private engine with its reviewed parameter constants;
    # production public visibility is checked separately under Windows.
    facade = (ROOT / "stdlib/internal/developer-tools/check-engine/module.jai").read_text()
    first, body = facade.split("#module_parameters(", 1)
    parameters, body = body.split(");", 1)
    if "CHECK_BINDINGS := true" not in parameters or "DEBUG_BINDINGS := false" not in parameters:
        raise RuntimeError("Unexpected canonical checker engine parameters")
    body = "CHECK_BINDINGS :: true;\nDEBUG_BINDINGS :: false;\n" + body
    for name in ["porting-check.jai", "check-bindings.jai"]:
        original = ROOT / "stdlib/internal/developer-tools" / name
        if original.exists():
            copied = output_dir(f"developer-tools-check-staged-{name}")
            copied.write_bytes(original.read_bytes())
            body = body.replace(f'"../{name}"', json.dumps(str(copied)))
    native_directory = output_dir("check-native")
    native_directory.mkdir(exist_ok=True)
    for library in (ROOT / "stdlib/internal/developer-tools/check-native").glob("*developer_tools_check.*"):
        link = native_directory / library.name
        if link.is_symlink():
            link.unlink()
        if not link.exists():
            link.symlink_to(library)
    staged = output_dir("developer-tools-check-staged.jai")
    staged.write_text("// Test staging of NEW production code; private engine parameters fixed for staged host validation.\n" + body)
    debug_staged = output_dir("developer-tools-check-debug-staged.jai")
    if body.count("DEBUG_BINDINGS :: false;") != 1:
        raise RuntimeError("Debug accounting staging must change only the reviewed debug constant")
    debug_staged.write_text("// Test-only debug accounting configuration.\n" + body.replace("DEBUG_BINDINGS :: false;", "DEBUG_BINDINGS :: true;"))
    fields = ["scalars", "zero", "reference", "callback", "array", "record", "method"]
    rows = []
    for target in ["arm64-apple-macos13", "x86_64-pc-windows-msvc"]:
        native = output_dir(f"developer-tools-check-{target}.o")
        run([prefix / "bin/clang++", "-std=c++17", "-target", target, "-O0", "-c",
             ROOT / "stdlib/Bindings_Generator/tests/driven/check-binding-native.cpp", "-o", native])
        symbols = run([prefix / "bin/llvm-nm", "--defined-only", "--just-symbol-name", native]).stdout.splitlines()
        selected = {}
        for field in fields:
            marker = "original_check_" + field if field != "method" else "method"
            matches = [symbol for symbol in symbols if marker in symbol]
            if len(matches) != 1:
                raise RuntimeError(f"Original native signature must have one symbol: {target}/{field}")
            selected[field] = matches[0]
        rows.append(selected)
    oracle = output_dir("developer-tools-check-names.jai")
    oracle.write_text("Entry :: struct {\n" + "\n".join(f"{field}: string;" for field in fields) +
                      "\n}\nsymbols :: Entry.[\n" + ",\n".join(".{ " + ", ".join(
                          f"{field} = {json.dumps(row[field])}" for field in fields) + " }" for row in rows) + "];\n")
    run([JAIFMT, "--config", ROOT / "jacket.toml", staged, debug_staged, oracle], expected="")
    return staged, oracle


def main():
    build_developer_check.main()
    staged, oracle = prepare()
    jaic = cargo_binary("jaic")
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/check-bindings.jai"
    run([jaic, source, "-interp", "-I", output_dir()], expected="ok\n")
    for optimization in [0, 2]:
        executable = output_dir(f"developer-tools-check-bindings-o{optimization}")
        run([jaic, source, f"-O{optimization}", "-exe", executable, "-I", output_dir()], expected="")
        run([executable], expected="ok\n")
    rejected = run([jaic, ROOT / "tests/compiler/rejections/check-binding-types.jai",
                    "-check", "-I", output_dir()], code=1)
    if (rejected.stderr.count("error: Check:") != 12 or "assertion failed" in rejected.stderr
            or "encoded foreign return type" not in rejected.stderr):
        raise RuntimeError("Expected twelve genuine continuable binding diagnostics and false results")
    stats_source = ROOT / "stdlib/Bindings_Generator/tests/driven/check-binding-stats.jai"
    stats_measurements = []
    for mode in ["interp", "0", "2"]:
        command = [jaic, stats_source, "-interp", "-I", output_dir()]
        if mode != "interp":
            executable = output_dir(f"developer-tools-check-binding-stats-o{mode}")
            run([jaic, stats_source, f"-O{mode}", "-exe", executable, "-I", output_dir()], expected="")
            command = [executable]
        result = run(command)
        measurements = re.findall(r"pool peak=(\d+) bytes cumulative=(\d+) bytes", result.stdout)
        if (len(measurements) != 2 or measurements[0] != measurements[1]
                or int(measurements[0][0]) <= 0 or int(measurements[0][1]) != 2 * int(measurements[0][0])
                or not result.stdout.endswith("ok\n") or result.stderr):
            raise RuntimeError("Real parsing-pool peak/cumulative accounting or debug-only finish report is incorrect")
        stats_measurements.append({"mode": mode, "peak_bytes": int(measurements[0][0]),
                                   "cumulative_bytes": int(measurements[0][1])})
    skip_source = ROOT / "stdlib/Bindings_Generator/tests/driven/check-binding-skip.jai"
    for mode in ["interp", "0", "2"]:
        command = [jaic, skip_source, "-interp", "-I", output_dir()]
        if mode != "interp":
            executable = output_dir(f"developer-tools-check-binding-skip-o{mode}")
            run([jaic, skip_source, f"-O{mode}", "-exe", executable, "-I", output_dir()], expected="")
            command = [executable]
        result = run(command)
        if result.stdout.count("Check:") != 1 or not result.stdout.endswith("ok\n") or result.stderr:
            raise RuntimeError("Unsupported target must log once, skip parsing and return true")
    paths = [source, stats_source, skip_source, staged,
             output_dir("developer-tools-check-debug-staged.jai"), oracle]
    run([JAIFMT, "--config", ROOT / "jacket.toml", "--check", *paths,
         ROOT / "stdlib/internal/developer-tools/check-bindings.jai"], expected="")
    run([cargo_binary("jailint"), "-D", "warnings", "-I", output_dir(), *paths], expected="")
    run([cargo_binary("jailint"), "-D", "warnings", "-os", "windows", "-cpu", "x64",
         ROOT / "stdlib/Check/module.jai", ROOT / "stdlib/internal/developer-tools/check-engine/module.jai"], expected="")
    if platform.system() == "Darwin":
        library = ROOT / "stdlib/internal/developer-tools/check-native/libdeveloper_tools_check.dylib"
        dependencies = run(["otool", "-L", library]).stdout
        prefix = llvm_prefix()
        sdk_links = [line.strip().split(" (", 1)[0] for line in dependencies.splitlines() if "libLLVM" in line]
        if len(sdk_links) != 1 or not sdk_links[0].startswith("@loader_path/../clang-bindings/"):
            raise RuntimeError("Production Check helper must select its distribution LLVM runtime")
        runtime = ROOT / "stdlib/internal/developer-tools/clang-bindings"
        manifest = json.loads((runtime / "runtime-sdk.json").read_text())
        name = Path(sdk_links[0]).name
        sdk_image = prefix / "lib" / name
        sdk_hash = sha256_file(sdk_image)
        if manifest[name]["sdk_sha256"] != sdk_hash:
            raise RuntimeError("Bundled Check LLVM runtime must originate from configured SDK")
        evidence = {"host": platform.platform(), "configured_llvm_prefix": str(prefix),
                    "native_helper_sha256": sha256_file((ROOT / "stdlib/internal/developer-tools/check-native/signatures.cpp")),
                    "llvm_library": sdk_links[0], "configured_sdk_runtime_sha256": sdk_hash, "parsing_pool_effects": stats_measurements,
                    "continuable_diagnostics": 12, "module_availability": "Windows only; NEW code staged on Mac",
                    "windows_native_execution": False, "actual_compiler_header_integration": "pending producer ELSEWHERE metadata fix"}
        (output_dir("developer-tools-check-evidence.json")).write_text(json.dumps(evidence, indent=2) + "\n")
    print("Check staged public mangled-signature effects passed interpreter/native Mac O0/O2; Windows native execution remains unverified")


if __name__ == "__main__":
    main()
