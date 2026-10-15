#!/usr/bin/env python3
"""Run original WGL diagnostics and full public renderer regressions on Windows.

Optional --driver-dir deploys a caller-supplied, source-built real Mesa driver.
No downloads, credentials, CI dispatch, version overrides, fallback GL or skips.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def pe_machine(path):
    with path.open("rb") as source:
        header = source.read(64)
        if len(header) != 64 or header[:2] != b"MZ":
            raise ValueError(f"not a native Windows PE image: {path}")
        source.seek(struct.unpack_from("<I", header, 60)[0])
        signature = source.read(6)
        if len(signature) != 6 or signature[:4] != b"PE\0\0":
            raise ValueError(f"invalid PE signature: {path}")
        return struct.unpack_from("<H", signature, 4)[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, type=Path)
    parser.add_argument("--jailint", required=True, type=Path)
    parser.add_argument("--stdlib", type=Path, default=ROOT / "target/window-integration-stdlib")
    parser.add_argument("--native-libs", type=Path, required=True)
    parser.add_argument("--driver-dir", type=Path)
    parser.add_argument("--out", type=Path, default=ROOT / "target/window-wgl-ci")
    args = parser.parse_args()
    if platform.system() != "Windows":
        parser.error("actual Windows execution is required; source checking is not driver verification")
    compiler = args.jaic.resolve(strict=True)
    linter = args.jailint.resolve(strict=True)
    output = args.out.resolve()
    for path in [compiler, linter, output]:
        if not path.is_relative_to(ROOT / "target"):
            parser.error(f"source-built compiler/output must stay under this checkout's target: {path}")
    machine = pe_machine(compiler)
    if machine not in {0x8664, 0xaa64}:
        parser.error("a native x64 or arm64 source-built compiler is required")
    if pe_machine(linter) != machine:
        parser.error("source-built compiler and linter must have the same native CPU")
    output.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ)
    for key in ["MESA_GL_VERSION_OVERRIDE", "MESA_GLSL_VERSION_OVERRIDE"]:
        if env.get(key):
            parser.error(f"{key} would invalidate actual driver capability verification")
    search = [args.native_libs.resolve(strict=True)]
    deployed = []
    if args.driver_dir:
        driver = args.driver_dir.resolve(strict=True)
        if not (driver / "opengl32.dll").is_file():
            parser.error("driver directory must contain the real Mesa opengl32.dll and its dependencies")
        libraries = sorted(driver.glob("*.dll"))
        # Validate every driver dependency before modifying either output directory.
        for library in libraries:
            if pe_machine(library) != machine:
                parser.error(f"driver/host architecture mismatch: {library}")
        for library in libraries:
            digest = hashlib.sha256(library.read_bytes()).hexdigest()
            for destination in [compiler.parent / library.name, output / library.name]:
                if library.resolve() != destination.resolve():
                    shutil.copy2(library, destination)
            deployed.append({"name": library.name, "machine": machine, "sha256": digest})
        search.insert(0, driver)
        env["GALLIUM_DRIVER"] = "llvmpipe"
    env["JAIC_STDLIB"] = str(args.stdlib.resolve(strict=True))
    env["JAIC_NATIVE_LIBS"] = os.pathsep.join(map(str, search))
    cases = ["windows-wgl-driver", "windows-render-events", "windows-render-public", "windows-render-lifecycle"]
    records = []

    def run(case, mode, command):
        try:
            result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                    capture_output=True, text=True, timeout=120)
        except subprocess.TimeoutExpired as failure:
            stdout = failure.stdout or ""
            stderr = failure.stderr or ""
            if isinstance(stdout, bytes):
                stdout = stdout.decode("utf-8", errors="replace")
            if isinstance(stderr, bytes):
                stderr = stderr.decode("utf-8", errors="replace")
            print(f"{case} {mode}: TIMEOUT (failure)", flush=True)
            print(stdout, end="", flush=True)
            print(stderr, end="", flush=True)
            records.append({"case": case, "mode": mode, "passed": False,
                            "exit": None, "stdout": stdout, "stderr": stderr + "\ntimeout"})
            return False
        except OSError as failure:
            print(f"{case} {mode}: process could not start: {failure}", flush=True)
            records.append({"case": case, "mode": mode, "passed": False,
                            "exit": None, "stdout": "", "stderr": str(failure)})
            return False
        print(f"{case} {mode}: exit={result.returncode}", flush=True)
        print(result.stdout, end="", flush=True)
        print(result.stderr, end="", flush=True)
        ok = result.returncode == 0 and not result.stderr
        if mode in {"interp", "O0", "O2"} and case != "windows-wgl-driver":
            ok = ok and result.stdout == "ok\n"
        if case == "windows-wgl-driver" and mode != "build":
            ok = ok and result.stdout.endswith("ok\n")
            if args.driver_dir:
                ok = ok and "llvmpipe" in result.stdout.lower()
        records.append({"case": case, "mode": mode, "passed": ok,
                        "exit": result.returncode, "stdout": result.stdout, "stderr": result.stderr})
        return ok

    for case in cases:
        source = ROOT / f"tests/window-render/{case}.jai"
        run(case, "interp", [compiler, source, "-interp"])
        for optimization in ["O0", "O2"]:
            binary = output / f"{case}-{optimization}.exe"
            if run(case, "build", [compiler, source, f"-{optimization}", "-exe", binary]):
                run(case, optimization, [binary])
    # Compile/lint the actual integrated library roots, not local facade copies
    # whose parent directory could prepend inherited dependencies to imports.
    library = args.stdlib.resolve(strict=True)
    lint_paths = [library / name for name in ["Window_Type", "Window_Creation", "Input", "Keymap",
                                            "Gamepad", "Simp", "Srgb", "Clipboard"]]
    lint_paths += [library / "internal/rendering/window-contexts.jai",
                   library / "internal/window/lifecycle.jai"]
    lint_paths += [ROOT / f"tests/window-render/{case}.jai" for case in cases]
    run("owned-WGL-lint", "lint", [linter, "-D", "warnings", "-I", library, *lint_paths])
    report = {"machine": machine, "driver_files": deployed, "records": records,
              "passed": all(item["passed"] for item in records)}
    (output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    # Coordinator can transfer this filtered native-SDK metadata without exposing
    # raw CI logs, dependency source diagnostics or implementation transcripts.
    prefixes = ("PFD:", "legacy GL_VENDOR=", "GL_RENDERER=", "GL_VERSION=",
                "WGL_EXTENSIONS=", "opengl32=", "canonical WGL3.3:",
                "modern GL_VERSION=", "unsupported version:", "WGL native failure:")
    summary = {"machine": machine, "driver_files": deployed, "passed": report["passed"],
               "cases": [{"case": item["case"], "mode": item["mode"],
                          "exit": item["exit"], "passed": item["passed"]} for item in records],
               "driver_metadata": [{"mode": item["mode"],
                                    "lines": [line for line in item["stdout"].splitlines()
                                              if line.startswith(prefixes)]}
                                   for item in records if item["case"] == "windows-wgl-driver"]}
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    if not report["passed"]:
        raise SystemExit("WGL diagnostics/production renderer failed; retain this failure evidence")


if __name__ == "__main__":
    main()
