#!/usr/bin/env python3
"""Focused verification for this worker's supported paths, using source-built tools.

This does not claim integrated renderer, physical-device or cross-host conformance.
The macOS clipboard case writes and empties the actual host clipboard.
"""
import argparse
import os
from pathlib import Path
import platform
import subprocess
from stdlib_runtime import runtime_prefix

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jaic", type=Path, required=True, help="explicit authorized source-built compiler")
    parser.add_argument("--stdlib", type=Path, default=ROOT / "target/window-integration-stdlib")
    parser.add_argument("--native-libs", type=Path, default=ROOT / "target/window-native")
    parser.add_argument("--font", type=Path, help="caller-provided scalable font asset; never copied into the repository")
    args = parser.parse_args()
    compiler = args.jaic.resolve(strict=True)
    native_search = list(dict.fromkeys([str(args.native_libs.resolve(strict=True)),
                                       *filter(None, os.environ.get("JAIC_NATIVE_LIBS", "").split(os.pathsep))]))
    env = dict(os.environ, JAIC_STDLIB=str(args.stdlib.resolve()),
               JAIC_NATIVE_LIBS=os.pathsep.join(native_search))
    output = ROOT / "target/window-test-binaries"
    output.mkdir(parents=True, exist_ok=True)
    count = 0
    failures = []

    def test_source(name):
        module = {"signed-native-fields": "Input", "texture-packing": "Simp",
                  "glyph-atlas": "Simp", "macos-events": "Window_Creation",
                  "macos-window": "Window_Creation", "windows-device-decoding": "Input"}.get(name)
        return ROOT / (f"stdlib/{module}/tests/{name}.jai" if module else f"tests/window-render/{name}.jai")

    def run(command, expected=None, restore_clipboard=False):
        nonlocal count
        prefix = runtime_prefix({"clipboard_guard": restore_clipboard}, env)
        child_env = dict(env, JAIC_TEST_CHILD_TIMEOUT_MS="60000") if prefix else env
        result = subprocess.run([*prefix, *map(str, command)], cwd=ROOT, env=child_env,
                                capture_output=True, text=True, timeout=90 if prefix else 60)
        if result.returncode or result.stderr or (expected is not None and result.stdout != expected):
            failures.append(f'{command}\nexit {result.returncode}\n{result.stdout}{result.stderr}')
            return False
        count += 1
        return True

    host = ["srgb", "mapping", "bitmap-codec", "signed-native-fields", "keymap-behavior", "mipmap", "font-effects"]
    if platform.system() in {"Darwin", "Windows"}:
        host.append("clipboard-host")
    if args.font:
        host.append("font-public")
        host.append("glyph-atlas")
        host.append("dynamic-font")
        if platform.system() == "Darwin":
            host.append("render-driver")
    if platform.system() == "Windows":
        host.append("windows-events")
        host.append("windows-committed-text")
        host.append("windows-render-events")
        host.append("windows-class-parameters")
        host.append("windows-layout-keys")
        host.append("windows-render-lifecycle")
        host.append("windows-render-public")
        host.append("windows-device-notifications")
        host.append("windows-pointer")
        host.append("public-window")
    if platform.system() == "Darwin":
        host.append("macos-window")
        host.append("macos-events")
        host.append("public-window")
        host.append("texture-driver")
        host.append("texture-colorspace")
        host.append("shader-driver")
        host.append("depth-driver")
        host.append("window-render-host")
    for name in host:
        source = test_source(name)
        parameters = [args.font.resolve()] if name in {"font-public", "glyph-atlas", "dynamic-font", "render-driver"} else []
        program_args = ["--", *parameters] if parameters else []
        before = len(failures)
        run([compiler, source, "-interp", "-I", args.stdlib, *program_args], "ok\n", restore_clipboard=name == "clipboard-host")
        for optimization in ["O0", "O2"]:
            binary = output / f"{name}-{optimization}"
            if run([compiler, source, f"-{optimization}", "-exe", binary, "-I", args.stdlib]):
                run([binary, *parameters], "ok\n", restore_clipboard=name == "clipboard-host")
        outcome = "passed" if len(failures) == before else "FAILED"
        print(f"{outcome} host interpreter and native O0/O2: {name}", flush=True)

    linux = ["types", "portable", "callback-types", "keycodes", "window-invalid", "bitmap-codec", "gamepad-virtual", "gamepad-shared-sdk", "texture-packing", "simp-none", "font-effects"]
    for name in linux:
        source = test_source(name)
        passed = run([compiler, source, "-interp", "-os", "linux", "-I", args.stdlib], "ok\n")
        print(f"{'passed' if passed else 'FAILED'} Linux-target interpreter: {name}", flush=True)

    if args.font:
        passed = run([compiler, ROOT / "tests/window-render/font-public.jai", "-interp", "-os", "linux",
                      "-I", args.stdlib, "--", args.font.resolve()], "ok\n")
        print(f"{'passed' if passed else 'FAILED'} public font facade with native FreeType (Linux-target interpreter)", flush=True)

    for os_name in ["linux", "windows"]:
        for cpu in ["x64", "arm64"]:
            cases = ["types", "callback-types", "keycodes", "window-types", "texture-signatures", "simp-none", "public-window", "keymap-behavior"]
            cases += ["evdev-signatures", "window-invalid", "font-effects"] if os_name == "linux" else ["clipboard-host", "windows-queries", "windows-events", "windows-committed-text", "windows-render-events", "windows-class-parameters", "windows-layout-keys", "windows-render-lifecycle", "windows-render-public", "windows-wgl-driver", "windows-device-notifications", "windows-device-decoding", "windows-pointer", "font-public", "font-effects"]
            for name in cases:
                run([compiler, test_source(name), "-check", "-os", os_name,
                     "-cpu", cpu, "-I", args.stdlib])
            if os_name == "windows":
                run([compiler, ROOT / "tests/window-render/font-public.jai", "-check", "-no_dce",
                     "-os", os_name, "-cpu", cpu, "-I", args.stdlib])
    for cpu in ["x64", "arm64"]:
        # Execute pure packet assertions with the target's layouts at compile
        # time; native Windows startup APIs cannot execute on a foreign host.
        run([compiler, test_source("windows-device-decoding"), "-check", "-add", "#run main();",
             "-os", "windows", "-cpu", cpu, "-I", args.stdlib], "ok\n")
    for case, member in [("private-helper-negative", "pack_bitmap"), ("private-atlas-negative", "atlas_create_page")]:
        visibility = subprocess.run([str(compiler), str(ROOT / f"tests/window-render/{case}.jai"),
                                     "-check", "-os", "linux", "-I", str(args.stdlib)],
                                    cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
        if visibility.returncode != 1 or visibility.stdout or f"has no exported member `{member}`" not in visibility.stderr:
            failures.append(f"private-helper visibility regression: {visibility.stdout}{visibility.stderr}")
        else:
            count += 1
    checked_cast = subprocess.run([str(compiler), str(ROOT / "tests/window-render/checked-narrow-negative.jai"),
                                   "-interp", "-I", str(args.stdlib)],
                                  cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
    if checked_cast.returncode != 1 or checked_cast.stdout or "cast of 65416 to `s16` overflows" not in checked_cast.stderr:
        failures.append(f"checked narrowing regression: {checked_cast.stdout}{checked_cast.stderr}")
    else:
        count += 1
    unsupported = subprocess.run([str(compiler), str(ROOT / "tests/window-render/keymap-meta-persistence-negative.jai"),
                                  "-interp", "-I", str(args.stdlib)], cwd=ROOT, env=env,
                                 capture_output=True, text=True, timeout=60)
    if unsupported.returncode == 0 or unsupported.stdout or "reviewed keymap persistence grammar has no Meta modifier prefix" not in unsupported.stderr:
        failures.append(f"unsupported Meta persistence diagnostic regression: {unsupported.stdout}{unsupported.stderr}")
    else:
        count += 1
    print(f"{count} focused compiler/program invocations passed; {len(failures)} failed", flush=True)
    if failures:
        raise RuntimeError("\n\n".join(failures))


if __name__ == "__main__":
    main()
