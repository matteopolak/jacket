#!/usr/bin/env python3
"""Check the foundations cases against explicitly selected clean-room source builds."""
import argparse
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CASES = ("builder-format", "integer-format", "string-patterns", "hash-floats", "basic-filesystem",
         "file-utilities-shape", "foundation-filesystem", "hash-scalars", "file-visitation", "basic-output", "formatter-records", "basic-sleep",
         "format-grammar", "format-floats", "format-aggregates", "format-types", "assert-message")


def owned_sources():
    """Exclude inherited runtime/allocation/time/File/MD5 implementation ownership."""
    paths = [ROOT / f"stdlib/{module}/module.jai"
             for module in ("Basic", "String", "Hash", "File_Utilities")]
    paths.extend((ROOT / "stdlib/internal/basic").glob("format*.jai"))
    paths.extend(ROOT / f"stdlib/internal/basic/{name}.jai"
                 for name in ("builder", "scalar", "integer128", "parsing", "module"))
    paths.extend((ROOT / "stdlib/internal/string").glob("*.jai"))
    paths.append(ROOT / "stdlib/internal/hash/module.jai")
    paths.extend(path for path in (ROOT / "stdlib/internal/files").glob("*.jai") if path.name != "file.jai")
    paths.extend(ROOT / f"tests/portable-stdlib/{name}.jai" for name in CASES)
    paths.extend((ROOT / "tests/portable-stdlib/negative").glob("format-*.jai"))
    paths.append(ROOT / "tests/portable-stdlib/pending/formatter-parameter-defaults.jai")
    return sorted(set(paths))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, type=Path)
    parser.add_argument("--stdlib", required=True, type=Path)
    parser.add_argument("--jaifmt", type=Path, help="unchanged repository CLI built with the selected library")
    parser.add_argument("--jailint", type=Path, help="linter built from the selected independent compiler sources")
    parser.add_argument("--cross", action="store_true")
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
        run([linter, "-D", "warnings", *owned_sources()], "")
        print("passed source-built jailint -D warnings on owned sources and tests", flush=True)

    for name, diagnostic in (("hash-unsupported", "get_hash requires an integer, boolean, enum, pointer, or float"),
                             ("hash-seed-visibility", "no exported member `HASH_INIT`"),
                             ("builder-buffer-visibility", "no exported member `Buffer`")):
        source = ROOT / f"tests/portable-stdlib/negative/{name}.jai"
        rejected = subprocess.run([compiler, "-check", source], cwd=ROOT, env=env,
                                  capture_output=True, text=True, timeout=60)
        if rejected.returncode != 1 or diagnostic not in rejected.stderr:
            raise RuntimeError(f"expected contract rejection {name}: {rejected}")
    print("passed unsupported-hash and facade-visibility rejection checks", flush=True)

    for name, diagnostic in (("format-int-kind", "FormatInt requires an integer value"),
                             ("format-float-kind", "FormatFloat requires an IEEE float32 or float64 value"),
                             ("format-int-base", "assertion failed"),
                             ("format-int-minimum", "assertion failed")):
        source = ROOT / f"tests/portable-stdlib/negative/{name}.jai"
        commands = [[compiler, "-interp", source]]
        for level in ("O0", "O2"):
            binary = output / f"{name}-{level}"
            run([compiler, source, f"-{level}", "-exe", binary])
            commands.append([binary])
        for command in commands:
            rejected = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                      capture_output=True, text=True, timeout=60)
            if rejected.returncode == 0 or diagnostic not in rejected.stderr:
                raise RuntimeError(f"expected formatter assertion {name}: {rejected}")
        print(f"passed interpreter/native O0/O2 assertion: {name}", flush=True)

    for name in CASES:
        source = ROOT / f"tests/portable-stdlib/{name}.jai"
        expected = ("out 7++", "err true", 17) if name == "basic-output" else ("ok\n", "", 0)
        run([compiler, "-interp", source], *expected)
        for level in ("O0", "O2"):
            binary = output / f"{name}-{level}"
            run([compiler, source, f"-{level}", "-exe", binary])
            run([binary], *expected)
        print(f"passed interpreter/native O0/O2: {name}", flush=True)
    if args.cross:
        failures = []
        for os_name, cpu in (("linux", "x64"), ("linux", "arm64"), ("macos", "arm64"), ("macos", "x64"), ("windows", "x64"), ("windows", "arm64"), ("wasm", "wasm64")):
            try:
                for name in CASES:
                    if name in ("basic-filesystem", "foundation-filesystem", "file-visitation", "basic-sleep") and os_name in ("windows", "wasm"):
                        continue
                    if name == "basic-output" and os_name == "wasm":
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
        if failures:
            raise RuntimeError("\n".join(failures))


if __name__ == "__main__":
    main()
