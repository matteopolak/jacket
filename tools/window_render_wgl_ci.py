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
import re
import shutil
import struct
import subprocess
from cleanroom_lint_contexts import canonical, checked_coverage
from window_render_tests import WINDOW_RENDER_TESTS

ROOT = Path(__file__).resolve().parents[1]
CASES = (
    "windows-wgl-driver",
    "windows-render-events",
    "windows-render-public",
    "windows-render-lifecycle",
)


def lint_jobs(linter, library, output):
    """Private requested files retain the provenance of their public importer."""
    # Entry files include their transitive #loads in jailint's report. Recursive
    # directory roots would also compile Mac-only tests in this Windows gate.
    public = [
        library / name / "module.jai"
        for name in [
            "Window_Type",
            "Window_Creation",
            "Input",
            "Keymap",
            "Gamepad",
            "Gamepad/SDL_Input_Map",
            "Simp",
            "Srgb",
            "Clipboard",
        ]
    ]
    public += [
        library / name
        for name in [
            "Input/tests/windows-device-decoding.jai",
            "Input/tests/signed-native-fields.jai",
            "Simp/tests/glyph-atlas.jai",
            "Simp/tests/texture-packing.jai",
        ]
    ]
    programs = [ROOT / WINDOW_RENDER_TESTS[case] for case in CASES]
    private = [
        library / "internal/rendering/window-contexts.jai",
        library / "internal/window/lifecycle.jai",
        library / "internal/input/native-fields.jai",
        library / "internal/input/windows-devices/module.jai",
    ]
    jobs = []
    for name, paths, context, required in [
        ("public", public + programs, None, public + programs),
        ("private", private, programs[1], private),
    ]:
        manifest = output / f"owned-WGL-lint-{name}.json"
        command = [linter, "-D", "warnings", "-I", library, "--manifest", manifest]
        if context:
            command += ["--context", context]
        jobs.append(
            {
                "name": name,
                "command": command + paths,
                "manifest": manifest,
                "required": required,
            }
        )
    return jobs


def lint_coverage(manifest, status, required):
    try:
        coverage = checked_coverage(json.loads(manifest.read_text()), status)
        coverage["missing_required"] = sorted(
            canonical(path)
            for path in required
            if canonical(path) not in coverage["visited_complete"]
        )
        coverage["clean"] = coverage["clean"] and not coverage["missing_required"]
        return coverage
    except (OSError, ValueError, KeyError, TypeError) as error:
        return {
            "clean": False,
            "error": f"invalid or missing lint coverage manifest: {error}",
        }
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


def probe_loaded_expected_driver(stdout, expected, alternatives=()):
    paths = [line.removeprefix("opengl32=") for line in stdout.splitlines() if line.startswith("opengl32=")]
    if len(paths) != 1:
        return False
    for candidate in [expected, *alternatives]:
        try:
            if Path(paths[0]).samefile(candidate):
                return True
        except OSError:
            continue
    return False



def deploy_driver(driver, machine, directories):
    """Deploy the same curated driver closure beside every actual consumer.

    The x64 installer supplies the checksum-verified frontend/backend pair;
    the native Mesa builder supplies its SDK-validated transitive closure.
    Only DLLs in that driver output are selected, never arbitrary package files.
    Validate the whole closure before changing any destination.
    """
    libraries = sorted(path for path in driver.iterdir() if path.suffix.casefold() == ".dll")
    names = [path.name.casefold() for path in libraries]
    if not {"opengl32.dll", "libgallium_wgl.dll"}.issubset(names):
        raise ValueError("required Mesa frontend/backend DLL missing from curated driver output")
    if len(set(names)) != len(names):
        raise ValueError("ambiguous case-insensitive driver DLL names")
    records = []
    for library in libraries:
        if library.is_symlink() or not library.is_file():
            raise ValueError(f"driver DLL must be a regular file: {library}")
        if pe_machine(library) != machine:
            raise ValueError(f"driver/host architecture mismatch: {library}")
        records.append({"name": library.name, "machine": machine,
                        "sha256": hashlib.sha256(library.read_bytes()).hexdigest()})
    for directory in dict.fromkeys(directories):
        directory.mkdir(parents=True, exist_ok=True)
        for library, record in zip(libraries, records):
            destination = directory / library.name
            if destination.is_symlink():
                raise ValueError(f"app-local driver must not be a symbolic link: {destination}")
            if library.resolve() != destination.resolve():
                shutil.copy2(library, destination)
            if hashlib.sha256(destination.read_bytes()).hexdigest() != record["sha256"]:
                raise ValueError(f"app-local driver copy integrity mismatch: {destination}")
    return records


def wgl_lint_paths(library, cases):
    # Entry files retain loaded implementation lint in its real module context.
    # Directory inputs would recursively promote unrelated platform tests to roots.
    paths = [library / name / "module.jai" for name in
             ["Window_Type", "Window_Creation", "Input", "Keymap", "Gamepad", "Gamepad/SDL_Input_Map", "Simp", "Srgb", "Clipboard"]]
    paths += [library / "internal/rendering/window-contexts.jai",
              library / "internal/window/lifecycle.jai",
              library / "internal/input/native-fields.jai",
              library / "internal/input/windows-devices/module.jai"]
    # Retain the existing valid Windows/portable tests previously found inside
    # these module directories; unrelated Cocoa/Linux roots are not WGL inputs.
    paths += [library / name for name in
              ["Input/tests/windows-device-decoding.jai", "Input/tests/signed-native-fields.jai",
               "Simp/tests/glyph-atlas.jai", "Simp/tests/texture-packing.jai"]]
    return paths + [ROOT / WINDOW_RENDER_TESTS[case] for case in cases]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", required=True, type=Path)
    parser.add_argument("--jailint", required=True, type=Path)
    parser.add_argument("--stdlib", type=Path, default=ROOT / "target/window-integration-stdlib")
    parser.add_argument("--native-libs", type=Path, required=True)
    parser.add_argument("--driver-dir", type=Path)
    parser.add_argument("--consumer-dir", action="append", type=Path, default=[],
                        help="also deploy the verified driver beside these checkout-local consumers")
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
    consumers = [path.resolve() for path in args.consumer_dir]
    if consumers and not args.driver_dir:
        parser.error("consumer deployment requires the real --driver-dir")
    for path in consumers:
        if not path.is_relative_to(ROOT):
            parser.error(f"consumer output must stay under this checkout: {path}")
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
        deployed = deploy_driver(driver, machine, [compiler.parent, output, *consumers])
        search.insert(0, driver)
        env["GALLIUM_DRIVER"] = "llvmpipe"
    env["JAIC_STDLIB"] = str(args.stdlib.resolve(strict=True))
    env["JAIC_NATIVE_LIBS"] = os.pathsep.join(map(str, search))
    cases = CASES
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
                expected_driver = (compiler.parent if mode == "interp" else output) / "opengl32.dll"
                # Interpreter foreign-library search explicitly honors the
                # supplied JAIC_NATIVE_LIBS driver directory. That fresh file
                # and its app-local copy are both valid candidate identities.
                ok = ok and probe_loaded_expected_driver(
                    result.stdout, expected_driver, [args.driver_dir / "opengl32.dll"])
        records.append({"case": case, "mode": mode, "passed": ok,
                        "exit": result.returncode, "stdout": result.stdout, "stderr": result.stderr})
        return ok

    for case in cases:
        source = ROOT / WINDOW_RENDER_TESTS[case]
        run(case, "interp", [compiler, source, "-interp"])
        for optimization in ["O0", "O2"]:
            binary = output / f"{case}-{optimization}.exe"
            if run(case, "build", [compiler, source, f"-{optimization}", "-exe", binary]):
                run(case, optimization, [binary])
    # Compile/lint the actual integrated library roots, not local facade copies
    # whose parent directory could prepend inherited dependencies to imports.
    library = args.stdlib.resolve(strict=True)
    for job in lint_jobs(linter, library, output):
        # A previous successful run's manifest cannot cover this invocation.
        job["manifest"].unlink(missing_ok=True)
        run("owned-WGL-lint", "lint-" + job["name"], job["command"])
        record = records[-1]
        record["coverage"] = lint_coverage(
            job["manifest"], record["exit"], job["required"]
        )
        record["passed"] = record["passed"] and record["coverage"]["clean"]
    report = {"machine": machine, "driver_files": deployed, "consumer_dirs": list(map(str, consumers)), "records": records,
              "passed": all(item["passed"] for item in records)}
    (output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    # Coordinator can transfer this filtered native-SDK metadata without exposing
    # raw CI logs, dependency source diagnostics or implementation transcripts.
    prefixes = ("PFD:", "legacy GL_VENDOR=", "GL_RENDERER=", "GL_VERSION=",
                "WGL_EXTENSIONS=", "opengl32=", "canonical WGL3.3:",
                "modern GL_VERSION=", "unsupported version:", "WGL native failure:")
    summary = {"machine": machine, "driver_files": deployed, "consumer_dirs": list(map(str, consumers)), "passed": report["passed"],
               "cases": [{"case": item["case"], "mode": item["mode"],
                          "exit": item["exit"], "passed": item["passed"]} for item in records],
               "driver_metadata": [{"mode": item["mode"],
                                    "lines": [line for line in item["stdout"].splitlines()
                                              if line.startswith(prefixes)]}
                                   for item in records if item["case"] == "windows-wgl-driver"]}
    summary["native_failures"] = [
        {"case": item["case"], "mode": item["mode"],
         "line": line[line.index("WGL native failure:"):]}
        for item in records for line in (item["stdout"] + item["stderr"]).splitlines()
        if "WGL native failure:" in line]
    # Transfer diagnostic headers/locations, never source excerpts or raw logs.
    # This supplies the missing rule and location for the current native lint
    # failure without exposing any dependency/corpus implementation body.
    summary["lint_diagnostics"] = []
    summary["lint_coverage"] = [
        {"mode": item["mode"], **item["coverage"]}
        for item in records
        if "coverage" in item
    ]
    for item in records:
        if item["case"] != "owned-WGL-lint":
            continue
        for raw in (item["stdout"] + item["stderr"]).splitlines():
            line = re.sub(r"\x1b\[[0-9;]*m", "", raw).strip()
            if re.match(r"^(error|warning|note)(\[[a-z_]+\])?:", line) or line.startswith("--> "):
                summary["lint_diagnostics"].append(line)
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    if not report["passed"]:
        raise SystemExit("WGL diagnostics/production renderer failed; retain this failure evidence")


if __name__ == "__main__":
    main()
