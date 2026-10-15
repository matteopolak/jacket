#!/usr/bin/env python3
"""Check the foundations cases against explicitly selected clean-room source builds."""
import argparse
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CASES = ("builder-format", "integer-format", "string-patterns", "hash-floats", "basic-filesystem",
         "file-utilities-shape", "foundation-filesystem", "hash-scalars", "file-visitation", "basic-output", "formatter-records", "basic-sleep",
         "format-grammar", "format-floats", "format-aggregates", "format-types", "assert-message",
         "basic-configuration", "temporary-stamping", "float-parsing", "type-parameter-format",
         "formatter-parameter-defaults", "wasm-basic-minimal", "float-roundtrip",
         "memory-debugger-interface", "memory-debugger-default", "filesystem-invalid-paths",
         "basic-logging", "basic-logging-output", "basic-reflection",
         "basic-helpers", "basic-print-bytes", "basic-stack-traces", "basic-stack-output",
         "basic-scalars-ownership", "basic-leak-reports", "basic-leak-graphs",
         "basic-visualization-disabled", "basic-visualization-channel", "basic-visualization-timing",
         "basic-capabilities", "basic-preamble", "basic-array-aliasing")


def expected_output(name):
    return {
        "basic-output": ("out 7++", "err true", 17),
        "basic-logging-output": ("out 7", "err true", 0),
        "basic-print-bytes": ("0 1 7f ff \n[0][1]\n\n", "", 0),
        "basic-stack-output": ("OUT 'walk' at tree.jai:3\n", "ERR 'walk' at tree.jai:3\n", 0),
    }.get(name, ("ok\n", "", 0))


def owned_sources():
    """Keep inherited runtime/time/File/MD5 outside the modified core scope."""
    paths = [ROOT / f"stdlib/{module}/module.jai"
             for module in ("Basic", "String", "Hash", "File_Utilities")]
    paths.extend((ROOT / "stdlib/internal/basic").glob("format*.jai"))
    paths.extend(ROOT / f"stdlib/internal/basic/{name}.jai"
                 for name in ("builder", "scalar", "integer128", "parsing", "numeric-conversion", "temporary", "allocation", "arrays", "memory-debugger", "logging", "reflection", "stream-helpers", "stack-traces", "module"))
    paths.append(ROOT / "stdlib/internal/basic-interface/module.jai")
    paths.append(ROOT / "stdlib/internal/basic-context/module.jai")
    paths.append(ROOT / "stdlib/internal/basic/preamble.jai")
    paths.extend((ROOT / "stdlib/internal/basic").glob("leak-*.jai"))
    paths.extend((ROOT / "stdlib/internal/basic").glob("visualization*.jai"))
    paths.append(ROOT / "stdlib/internal/native-path/module.jai")
    paths.append(ROOT / "stdlib/internal/basic/filesystem.jai")
    paths.extend((ROOT / "stdlib/internal/string").glob("*.jai"))
    paths.append(ROOT / "stdlib/internal/hash/module.jai")
    paths.extend(path for path in (ROOT / "stdlib/internal/files").glob("*.jai") if path.name != "file.jai")
    paths.extend(ROOT / f"tests/portable-stdlib/{name}.jai" for name in CASES)
    paths.extend((ROOT / "tests/portable-stdlib/negative").glob("format-*.jai"))
    paths.append(ROOT / "tests/portable-stdlib/negative/builder-ensured-range.jai")
    paths.append(ROOT / "tests/portable-stdlib/negative/stack-trace-cycle.jai")
    paths.append(ROOT / "tests/portable-stdlib/negative/basic-scalar-bool.jai")
    paths.append(ROOT / "tests/portable-stdlib/negative/leak-unknown-allocation.jai")
    paths.append(ROOT / "tests/portable-stdlib/pending/context-overload-probe.jai")
    paths.append(ROOT / "tests/portable-stdlib/pending/basic-unused-platform-dependency.jai")
    paths.append(ROOT / "tests/portable-stdlib/pending/host-target-predicate.jai")
    paths.append(ROOT / "tests/portable-stdlib/pending/pool-capabilities-name.jai")
    paths.append(ROOT / "tests/portable-stdlib/pending/alias-signature-probe.jai")
    paths.append(ROOT / "tests/portable-stdlib/pending/alias-signature-facade/module.jai")
    paths.append(ROOT / "tests/portable-stdlib/pending/alias-signature-helper/module.jai")
    paths.append(ROOT / "tests/portable-stdlib/pending/interface-parameter-module/module.jai")
    paths.append(ROOT / "tests/portable-stdlib/pending/no-parameter-module/module.jai")
    paths.append(ROOT / "tests/portable-stdlib/support/test-heap.jai")
    paths.append(ROOT / "tests/portable-stdlib/float-power-sweep.jai")
    paths.append(ROOT / "tests/portable-stdlib/windows-foundations.jai")
    return sorted(set(paths))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, type=Path)
    parser.add_argument("--stdlib", required=True, type=Path)
    parser.add_argument("--jaifmt", type=Path, help="unchanged repository CLI built with the selected library")
    parser.add_argument("--jailint", type=Path, help="linter built from the selected independent compiler sources")
    parser.add_argument("--cross", action="store_true")
    parser.add_argument("--wasm", action="store_true", help="execute supported cases under Node WASI at O0/O2")
    args = parser.parse_args()
    compiler = args.jaic.resolve(strict=True)
    if not compiler.is_relative_to(ROOT / "target"):
        parser.error("use a compiler built from independent sources under this checkout's target")
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve(strict=True)))
    output = ROOT / "target/foundations-tests"
    output.mkdir(parents=True, exist_ok=True)

    if args.jaifmt:
        formatter = args.jaifmt.resolve(strict=True)
        if not formatter.is_relative_to(ROOT / "target"):
            parser.error("use a jaifmt source build under this checkout's target")
        directory = output / "jaifmt-cli"
        directory.mkdir(exist_ok=True)
        hidden = directory / ".hidden"
        hidden.mkdir(exist_ok=True)
        (hidden / "ignored.jai").write_text("deliberately malformed {")
        source = directory / "sample.jai"
        source.write_text("main::(){value:=1;value+=2;}\n")
        check = subprocess.run([formatter, "--check", source], cwd=ROOT, env=env,
                               capture_output=True, text=True, timeout=60)
        if check.returncode != 1 or not check.stdout or check.stderr:
            raise RuntimeError(f"jaifmt dirty-file check: {check}")
        rewrite = subprocess.run([formatter, directory], cwd=ROOT, env=env,
                                 capture_output=True, text=True, timeout=60)
        if rewrite.returncode or rewrite.stderr:
            raise RuntimeError(f"jaifmt recursive rewrite: {rewrite}")
        formatted = source.read_text()
        clean = subprocess.run([formatter, "--check", directory], cwd=ROOT, env=env,
                               capture_output=True, text=True, timeout=60)
        if clean.returncode or clean.stdout or clean.stderr or source.read_text() != formatted:
            raise RuntimeError(f"jaifmt idempotence: {clean}")
        stdin = subprocess.run([formatter, "--stdin"], input="main::(){value:=1;value+=2;}\n",
                               cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
        if stdin.returncode or stdin.stderr or stdin.stdout != formatted:
            raise RuntimeError(f"jaifmt stdin: {stdin}")
        if (hidden / "ignored.jai").read_text() != "deliberately malformed {":
            raise RuntimeError("jaifmt modified a pruned hidden directory")
        print("passed unchanged jaifmt CLI: check/rewrite/recursion/pruning/stdin/idempotence", flush=True)

    def run(command, expected=None, expected_stderr="", expected_code=0):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                capture_output=True, text=True, timeout=60)
        if result.returncode != expected_code or result.stderr != expected_stderr or (expected is not None and result.stdout != expected):
            raise RuntimeError(f"{command}\nexit={result.returncode}\n{result.stdout}{result.stderr}")

    if args.jaifmt:
        run([formatter, "--check", *owned_sources()], "")
        print("passed unchanged jaifmt --check on owned sources and tests", flush=True)
    if args.jailint:
        linter = args.jailint.resolve(strict=True)
        if not linter.is_relative_to(ROOT / "target"):
            parser.error("use a jailint source build under this checkout's target")
        files = owned_sources()
        covered = set()
        lint_output = output / "lint"
        lint_output.mkdir(exist_ok=True)
        for target_os, target_cpu in [("macos", "arm64"), ("linux", "x64"), ("windows", "x64"), ("windows", "arm64")]:
            selected = []
            for path in files:
                if "windows" in path.name and target_os != "windows":
                    continue
                if "unix" in path.name and target_os == "windows":
                    continue
                if target_os == "windows" and path in {
                    ROOT / "stdlib/internal/files/filesystem.jai",
                    ROOT / "stdlib/internal/basic/filesystem.jai",
                    ROOT / "tests/portable-stdlib/file-visitation.jai",
                }:
                    continue  # POSIX implementations/oracle; Windows backends have their own roots.
                selected.append(path)
            manifest = lint_output / f"{target_os}-{target_cpu}.txt"
            checked = subprocess.run([str(linter), "-v", "-D", "warnings", "--os", target_os,
                                      "--cpu", target_cpu, "-I", str(ROOT / "tests/portable-stdlib/pending"),
                                      "--checked-files", str(manifest), *map(str, selected)],
                                     cwd=ROOT, env=env, capture_output=True, text=True, timeout=180)
            (lint_output / f"{target_os}-{target_cpu}.log").write_text(checked.stdout + checked.stderr)
            if checked.returncode or checked.stdout or "did not compile completely" in checked.stderr:
                raise RuntimeError(f"foundation lint {target_os}/{target_cpu} incomplete or failed:\n{checked.stdout}{checked.stderr}")
            covered.update(Path(path).resolve() for path in manifest.read_text().splitlines())
            print(f"passed strict lint {target_os}/{target_cpu}", flush=True)
        missing = set(files) - covered
        if missing:
            raise RuntimeError(f"foundation sources not checked: {sorted(map(str, missing))}")
        print(f"passed source-built jailint -D warnings with coverage of {len(files)} owned sources", flush=True)

    for name, diagnostic in (("hash-unsupported", "get_hash requires an integer, boolean, enum, pointer, or float"),
                             ("hash-seed-visibility", "no exported member `HASH_INIT`"),
                             ("builder-buffer-visibility", "no exported member `Buffer`"),
                             ("basic-replacement-interface", "check_"),
                             ("basic-interface-import-visibility", "no exported member `Interface_Definition`"),
                             ("basic-parameter-interface-visibility", "no exported member `Memory_Debugger_Interface`"),
                             ("basic-private-formatter", "standard-library internal sources are private"),
                             ("basic-scalar-bool", "clamp requires a scalar type")):
        source = ROOT / f"tests/portable-stdlib/negative/{name}.jai"
        if args.jaifmt:
            run([formatter,"--check",source], "")
        rejected = subprocess.run([compiler, "-check", source], cwd=ROOT, env=env,
                                  capture_output=True, text=True, timeout=60)
        if rejected.returncode != 1 or diagnostic not in rejected.stderr:
            raise RuntimeError(f"expected contract rejection {name}: {rejected}")
    print("passed unsupported-hash and facade-visibility rejection checks", flush=True)
    run([compiler,"-check",ROOT / "tests/portable-stdlib/pending/context-overload-probe.jai"])
    print("passed general context/overload compiler regression", flush=True)
    run([compiler, "-interp", "-I", ROOT / "tests/portable-stdlib/pending",
         ROOT / "tests/portable-stdlib/pending/alias-signature-probe.jai"], "ok\n")
    print("passed general public alias overload/default execution regression", flush=True)

    for name, diagnostic in (("format-int-kind", "FormatInt requires an integer value"),
                             ("format-float-kind", "FormatFloat requires an IEEE float32 or float64 value"),
                             ("format-int-base", "assertion failed"),
                             ("format-int-minimum", "assertion failed"),
                             ("builder-ensured-range", "assertion failed"),
                             ("stack-trace-cycle", "a stack trace must not contain a cycle"),
                             ("leak-unknown-allocation", "unknown allocation marked as non-leak")):
        source = ROOT / f"tests/portable-stdlib/negative/{name}.jai"
        commands = [[compiler, "-interp", source]]
        for level in ("O0", "O2"):
            binary = output / f"{name}-{level}"
            run([compiler, source, f"-{level}", "-exe", binary])
            commands.append([binary])
        for command in commands:
            rejected = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                      capture_output=True, text=True, timeout=60)
            if rejected.returncode == 0 or diagnostic not in rejected.stderr or (name == "leak-unknown-allocation" and rejected.returncode != 1):
                raise RuntimeError(f"expected formatter assertion {name}: {rejected}")
        print(f"passed interpreter/native O0/O2 assertion: {name}", flush=True)

    host_cases = list(CASES)
    if os.name == "nt":
        host_cases.remove("file-visitation")  # Native POSIX symlink graph oracle.
        host_cases.append("windows-foundations")
    for name in host_cases:
        source = ROOT / f"tests/portable-stdlib/{name}.jai"
        expected = expected_output(name)
        run([compiler, "-interp", source], *expected)
        for level in ("O0", "O2"):
            binary = output / f"{name}-{level}"
            run([compiler, source, f"-{level}", "-exe", binary])
            run([binary], *expected)
        print(f"passed interpreter/native O0/O2: {name}", flush=True)
    if args.wasm:
        for name in CASES:
            if name in ("basic-filesystem", "foundation-filesystem", "file-visitation", "basic-sleep", "basic-output", "filesystem-invalid-paths", "basic-visualization-channel", "basic-visualization-timing"):
                continue
            source = ROOT / f"tests/portable-stdlib/{name}.jai"
            for level in ("O0", "O2"):
                binary = output / f"{name}-{level}.wasm"
                run([compiler, source, "-os", "wasm", f"-{level}", "-exe", binary])
                expected = expected_output(name)
                run(["node", "--no-warnings", ROOT / "tools/run_foundations_wasm.mjs", binary], *expected)
            print(f"passed production WASI O0/O2: {name}", flush=True)
        for level in ("O0", "O2"):
            binary = output / f"leak-unknown-allocation-{level}.wasm"
            run([compiler, ROOT / "tests/portable-stdlib/negative/leak-unknown-allocation.jai",
                 "-os", "wasm", f"-{level}", "-exe", binary])
            run(["node", "--no-warnings", ROOT / "tools/run_foundations_wasm.mjs", binary],
                "", "memory debugger: unknown allocation marked as non-leak", 1)
        print("passed production WASI O0/O2 unknown-allocation misuse status 1", flush=True)
    failures = []
    if args.cross:
        for os_name, cpu in (("linux", "x64"), ("linux", "arm64"), ("macos", "arm64"), ("macos", "x64"), ("windows", "x64"), ("windows", "arm64"), ("wasm", "wasm64")):
            try:
                for name in CASES:
                    if name == "file-visitation" and os_name == "windows":
                        continue  # POSIX symlink oracle; Windows public traversal is checked below.
                    if name in ("basic-filesystem", "foundation-filesystem", "file-visitation", "basic-sleep", "filesystem-invalid-paths") and os_name == "wasm":
                        continue
                    if name == "basic-output" and os_name == "wasm":
                        continue
                    if name in ("basic-visualization-channel", "basic-visualization-timing") and os_name in ("windows", "wasm"):
                        continue
                    target = ["-target", "wasm64-unknown-wasi"] if os_name == "wasm" else ["-os", os_name, "-cpu", cpu]
                    run([compiler, "-check", "-no_dce", *target,
                         ROOT / f"tests/portable-stdlib/{name}.jai"])
                if os_name == "windows":
                    run([compiler, "-check", "-no_dce", "-os", os_name, "-cpu", cpu,
                         ROOT / "tests/portable-stdlib/windows-foundations.jai"])
                    print("checked Windows-specific filesystem specializations (not executed)", flush=True)
                print(f"checked unused code: {os_name}/{cpu}", flush=True)
            except RuntimeError as error:
                failures.append(f"{os_name}/{cpu}: {error}")
                print(f"FAILED target {os_name}/{cpu}; continuing independent targets", flush=True)
    if args.jaifmt:
        run([formatter,"--check",ROOT / "tests/portable-stdlib/pending/module-interface-constraint.jai"], "")
    constraint = subprocess.run([compiler, "-I", ROOT / "tests/portable-stdlib/pending", "-check",
                                 ROOT / "tests/portable-stdlib/pending/module-interface-constraint.jai"],
                                cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
    if constraint.returncode != 1 or "run" not in constraint.stderr:
        failures.append("compiler must reject the invalid unused module interface constraint:\n"
                        f"exit={constraint.returncode}\n{constraint.stdout}{constraint.stderr}")
    else:
        print("passed general unused module-interface constraint rejection", flush=True)
    for source, flag in ((ROOT / "tests/portable-stdlib/negative/basic-unknown-parameter.jai", "FOUNDATIONS_UNDECLARED_PARAMETER"),
                         (ROOT / "tests/portable-stdlib/pending/module-undeclared-parameter.jai", "EXTRA")):
        if args.jaifmt:
            run([formatter,"--check",source], "")
        rejected = subprocess.run([compiler,"-I",ROOT / "tests/portable-stdlib/pending","-check",source],
                                  cwd=ROOT,env=env,capture_output=True,text=True,timeout=60)
        if rejected.returncode != 1 or flag not in rejected.stderr or "parameter" not in rejected.stderr:
            failures.append(f"compiler must reject undeclared module parameter {flag}:\n"
                            f"exit={rejected.returncode}\n{rejected.stdout}{rejected.stderr}")
        else:
            print(f"passed undeclared module parameter rejection: {flag}",flush=True)
    if failures:
        raise RuntimeError("\n".join(failures))


if __name__ == "__main__":
    main()
