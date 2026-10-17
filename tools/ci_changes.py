#!/usr/bin/env python3
"""Decide which path-gated CI jobs run, and write `<area>=true|false` lines to $GITHUB_OUTPUT.

Pushes diff the pushed range, pull requests diff against their base, anything else (manual
runs, a new branch, a range that cannot be diffed) runs every area. Areas live in AREAS: to gate
a job on more paths, add a pattern there. `*` matches within a directory, `**` across them.

    python3 tools/ci_changes.py          # reads GITHUB_EVENT_NAME, GITHUB_REF, EVENT_BEFORE, BASE_SHA, SHA
"""
import os
import re
import subprocess
import sys

SELF = r".github/(workflows/ci\.yml|actions/.*)"

AREAS = {
    "sdl3": [
        "stdlib/Extensions/SDL3/**", "tools/sdl3*.py", "tools/sdl_test_support.py",
        "tools/prepare_sdl_python_tests.py", "tools/test_sdl*.py", "tools/native/sdl3/**",
        "tools/native-libs.json", "tools/build_native_libs.py", "tools/repo_output.py", "tests/sdl3/**", ".gitattributes",
    ],
    "clipboard": [
        "stdlib/Clipboard/**", "stdlib/internal/clipboard/**", "stdlib/internal/window/connection.jai",
        "stdlib/Basic/**", "stdlib/internal/basic/**", "crates/**", "Cargo.*", "rust-toolchain.toml",
        "tests/portable-stdlib/clipboard-*.jai", "tests/portable-stdlib/native/clipboard-*.c",
        "tools/*portable_clipboard.py", "tools/check_clipboard_linux_package.py",
        "tools/*clipboard_linux_*.py", "tools/ci_test_scratch.py", "tools/test_ci_test_scratch.py",
    ],
    "nix": ["flake.nix", "flake.lock", "nix/**", "Cargo.lock", "rust-toolchain.toml"],
    "packaging": ["install.sh", "install.ps1", "packaging/**", "tools/render_packages.py"],
    # Windows runs on pushes (main, staging) only for the paths its coverage is about, never on
    # pull requests, and always on manual runs.
    "windows": [
        "stdlib/Extensions/Io/**", "stdlib/Extensions/Io_Uring/**", "stdlib/Extensions/Http/**",
        "stdlib/Extensions/Json/**", "stdlib/Extensions/Tls/**", "tools/native/mbedtls/**",
        "stdlib/Windows/**", "stdlib/File_Watcher/**", "stdlib/internal/watching/**",
        "tools/check_file_watcher_platforms.py", "tools/check_windows_time_zone.py",
        "tools/build_windows_resource_fixture.py", "tools/test_windows_resource_fixture.py",
        "tools/install_ci_native_linux.sh", "tools/upstream_native.py", "tools/upstream_smoke.py",
        "tools/build_portable_runtime.py", "tools/audio_build_native.py", "tools/build_native_libs.py",
        "tools/stdlib_runtime.py", "tools/test_windows_dynamic_package.py", "tools/windows_cross.py",
        "crates/jaic-llvm/**", "crates/jaic/src/interp/**", "crates/jaic-cli/src/**",
        "crates/jaic-cli/tests/native.rs", "tests/native/**",
    ],
}


def regex(pattern):
    out = ""
    i = 0
    while i < len(pattern):
        if pattern.startswith("**", i):
            out += ".*"
            i += 2
        elif pattern[i] == "*":
            out += "[^/]*"
            i += 1
        else:
            out += re.escape(pattern[i])
            i += 1
    return out


def matches(patterns, files):
    rx = re.compile("^(" + "|".join(regex(p) for p in patterns) + "|" + SELF + ")$")
    return any(rx.match(f) for f in files)


def changed_files(env):
    event = env.get("GITHUB_EVENT_NAME", "")
    if event == "push":
        base, head = env.get("EVENT_BEFORE", ""), env.get("SHA", "HEAD")
        if not base or set(base) == {"0"}:
            return None
        spec = f"{base}..{head}"
    elif event == "pull_request":
        spec = f"{env.get('BASE_SHA', '')}...{env.get('SHA', 'HEAD')}"
    else:
        return None
    result = subprocess.run(["git", "diff", "--name-only", spec], capture_output=True, text=True)
    return result.stdout.split() if result.returncode == 0 else None


def main():
    env = os.environ
    files = changed_files(env)
    event = env.get("GITHUB_EVENT_NAME", "")
    results = {}
    for area, patterns in AREAS.items():
        results[area] = files is None or matches(patterns, files)
    if event == "pull_request":
        results["windows"] = False
    lines = [f"{area}={str(value).lower()}" for area, value in results.items()]
    print("\n".join(lines))
    if env.get("GITHUB_OUTPUT"):
        with open(env["GITHUB_OUTPUT"], "a") as out:
            out.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
