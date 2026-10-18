#!/usr/bin/env python3
"""Decide which path-gated CI jobs run, and write `<area>=true|false` lines to $GITHUB_OUTPUT.

Pushes diff the pushed range, pull requests diff against their base, anything else (manual
runs, a new branch, a range that cannot be diffed) runs every area. Areas live in AREAS: to gate
a job on more paths, add a pattern there. A change to ci.yml or .github/actions runs every area
except those in OWN_PATHS_ONLY. `*` matches within a directory, `**` across them.
`code` is false when every changed file is prose (PROSE, minus the docs some test reads), so a
docs-only push skips the build and test jobs.

    python3 tools/ci/ci_changes.py          # reads GITHUB_EVENT_NAME, GITHUB_REF, EVENT_BEFORE, BASE_SHA, SHA
"""
import os
import re
import subprocess
import sys

SELF = r".github/(workflows/ci\.yml|actions/.*)"

AREAS = {
    # Only the extension and its own tooling: the release workflow still builds and tests SDL3 with
    # the shared native-library builders, so a change to those is caught before shipping.
    "sdl3": ["stdlib/Extensions/SDL3/**", "tools/bindings/*sdl*.py", "tools/native/sdl3/**"],
    "clipboard": [
        "stdlib/Clipboard/**", "stdlib/internal/clipboard/**", "stdlib/internal/window/connection.jai",
        "stdlib/Basic/**", "stdlib/internal/basic/**", "crates/**", "Cargo.*", "rust-toolchain.toml",
        "tools/portable/*portable_clipboard*.py", "tools/portable/check_clipboard_linux_package.py",
        "tools/portable/*clipboard_linux_*.py", "tools/ci/ci_test_scratch.py", "tools/ci/test_ci_test_scratch.py",
    ],
    "nix": ["flake.nix", "flake.lock", "nix/**", "Cargo.lock", "rust-toolchain.toml"],
    "packaging": ["install.sh", "install.ps1", "packaging/**", "tools/dev/render_packages.py"],
    # Windows runs on pushes (main, staging) only for the paths its coverage is about, never on
    # pull requests, and always on manual runs.
    "windows": [
        "stdlib/Extensions/Io/**", "stdlib/Extensions/Io_Uring/**", "stdlib/Extensions/Http/**",
        "stdlib/Extensions/Json/**", "stdlib/Extensions/Tls/**", "tools/native/mbedtls/**",
        "stdlib/Windows/**", "stdlib/File_Watcher/**", "stdlib/internal/watching/**",
        "tools/stdlib/check_file_watcher_platforms.py", "tools/native/check_windows_time_zone.py",
        "tools/native/build_windows_resource_fixture.py", "tools/native/test_windows_resource_fixture.py",
        "tools/ci/install_ci_native_linux.sh", "tools/native/upstream_native.py", "tools/corpus/upstream_smoke.py",
        "tools/portable/build_portable_runtime.py", "tools/native/audio_build_native.py", "tools/native/build_native_libs.py",
        "tools/stdlib/stdlib_runtime.py", "tools/native/test_windows_dynamic_package.py", "tools/native/windows_cross.py",
        "crates/jaic-llvm/**", "crates/jaic/src/interp/**", "crates/jaic-cli/src/**",
        "crates/jaic-cli/tests/native.rs", "tests/compiler/native/**",
    ],
}

# Areas too costly to run whenever the workflow itself changes.
OWN_PATHS_ONLY = {"sdl3"}

# Prose no build or test reads. The specs feed generators and checks, jailint's rules test reads
# its doc, and the rule IDs in docs/language and docs/metaprogramming are checked by the `docs`
# job, which always runs.
PROSE = ["docs/**", "*.md", ".github/*.md"]
READ_BY_TESTS = ["docs/specs/**", "docs/tools/jailint.md"]


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


def compile_patterns(patterns, extra=None):
    alternatives = [regex(p) for p in patterns] + ([extra] if extra else [])
    return re.compile("^(" + "|".join(alternatives) + ")$")


def matches(patterns, files, workflow=True):
    rx = compile_patterns(patterns, SELF if workflow else None)
    return any(rx.match(f) for f in files)


def code_changed(files):
    prose, read = compile_patterns(PROSE), compile_patterns(READ_BY_TESTS)
    return any(not prose.match(f) or read.match(f) for f in files)


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
        results[area] = files is None or matches(patterns, files, area not in OWN_PATHS_ONLY)
    results["code"] = files is None or code_changed(files)
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
