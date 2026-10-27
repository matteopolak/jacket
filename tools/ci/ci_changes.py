#!/usr/bin/env python3
"""Decide which path-gated CI jobs run, and write `<area>=true|false` lines to $GITHUB_OUTPUT.

Pushes diff the pushed range, pull requests diff against their base, anything else (manual
runs, a new branch, a range that cannot be diffed) runs every area. Areas live in AREAS: to gate
a job on more paths, add a pattern there. A change to ci.yml or .github/actions runs every area
except those in OWN_PATHS_ONLY. `*` matches within a directory, `**` across them.
`code` is false when every changed file is prose (PROSE, minus the docs some test reads), so a
docs-only push skips the build and test jobs.

It also writes `full=true|false` (the tier, see test_matrix) and the `test_matrix` and
`stdlib_matrix` JSON the `test` and `stdlib-runtime` jobs expand, and `jobs=<JSON array>`: the ids in JOBS that run, which are the jobs whose area is true
(or that have none), narrowed to the comma-separated `$JOBS` of a manual run. Every job of ci.yml
tests `contains(fromJSON(needs.changes.outputs.jobs), '<id>')`; a new job gets a row in JOBS.

    python3 tools/ci/ci_changes.py          # reads GITHUB_EVENT_NAME, JOBS, EVENT_BEFORE, BASE_SHA, SHA
"""

import json
import os
import re
import subprocess
import sys

SELF = r".github/(workflows/ci\.yml|actions/.*)"

AREAS = {
    # Only the extension and its own tooling: the release workflow still builds and tests SDL3 with
    # the shared native-library builders, so a change to those is caught before shipping.
    "sdl3": [
        "stdlib/Extensions/SDL3/**",
        "tools/bindings/*sdl*.py",
        "tools/native/sdl3/**",
    ],
    "clipboard": [
        "stdlib/Clipboard/**",
        "stdlib/internal/clipboard/**",
        "stdlib/internal/window/connection.jai",
        "stdlib/Basic/**",
        "stdlib/internal/basic/**",
        "crates/**",
        "Cargo.*",
        "rust-toolchain.toml",
        "tools/portable/*portable_clipboard*.py",
        "tools/portable/check_clipboard_linux_package.py",
        "tools/portable/*clipboard_linux_*.py",
        "tools/ci/ci_test_scratch.py",
        "tools/ci/test_ci_test_scratch.py",
    ],
    "nix": ["flake.nix", "flake.lock", "nix/**", "Cargo.lock", "rust-toolchain.toml"],
    # Every shell script (tools/dev/staged-files.sh lists them, shebang files included) is linted.
    "shell": ["**.sh", ".githooks/**", "tools/dev/staged-files.sh"],
    "packaging": [
        "install.sh",
        "install.ps1",
        "packaging/**",
        "tools/dev/render_packages.py",
        "tools/ci/published_release.py",
        "tools/ci/test_install_sh.py",
        "tools/ci/test_install_ps1.ps1",
        "tests/installers/**",
    ],
    # macOS ASan is slot-limited and the slowest sanitizer leg: on pushes it runs only for the code
    # that decides what the sanitized binaries are (code generation, the driver, the sweep and
    # its inputs, the toolchain); Linux ASan runs on every push, and nightly/manual runs do both.
    "sanitizers-macos": [
        "crates/jaic-llvm/**",
        "crates/jaic-cli/src/**",
        "crates/jaic/src/**",
        "tools/differential/**",
        "tools/native/**",
        "corpus/**",
        "Cargo.lock",
        "rust-toolchain.toml",
    ],
    # Windows runs on pushes (main, staging) only for the paths its coverage is about, never on
    # pull requests, and always on manual runs.
    "windows": [
        "stdlib/Bindings_Generator/**",
        "stdlib/internal/developer-tools/**",
        "tools/developer_checks/check_developer_vtables.py",
        "stdlib/Extensions/Io/**",
        "stdlib/Extensions/Io_Uring/**",
        "stdlib/Extensions/Http/**",
        "stdlib/Extensions/Json/**",
        "stdlib/Extensions/Variant/**",
        "stdlib/Extensions/Tls/**",
        "tools/native/mbedtls/**",
        "stdlib/Windows/**",
        "stdlib/File_Watcher/**",
        "stdlib/internal/watching/**",
        "tools/stdlib/check_file_watcher_platforms.py",
        "tools/native/check_windows_time_zone.py",
        "tools/native/build_windows_resource_fixture.py",
        "tools/native/test_windows_resource_fixture.py",
        "tools/ci/install_ci_native_linux.sh",
        "tools/ci/install_ci_llvm_linux.sh",
        "tools/ci/apt_lib.sh",
        "tools/ci/apt_install.sh",
        "tools/ci/test_apt_lib.py",
        "tools/native/upstream_native.py",
        "tools/corpus/upstream_smoke.py",
        "tools/upstream-cases.json",
        "corpus/upstreams.json",
        "tools/corpus/select_upstream_cases.py",
        "tools/corpus/upstream_sandbox.py",
        "tools/differential/jaic_sweep.py",
        "tools/ci/stdlib_jacket_sanitize.sh",
        "tools/ci/enter_vs_dev_shell.ps1",
        "tools/portable/build_portable.py",
        "tools/native/audio_build_native.py",
        "tools/native/build_native_libs.py",
        "tools/stdlib/stdlib_runtime.py",
        "tools/native/test_windows_dynamic_package.py",
        "tools/native/windows_cross.py",
        "crates/jaic-llvm/**",
        "crates/jaic/src/interp/**",
        "crates/jaic-cli/src/**",
        "crates/jaic-cli/tests/native.rs",
        "tests/compiler/native/**",
    ],
}

# Every job of ci.yml with the area that gates it (None: always, subject to a manual run's list).
JOBS = {
    "docs": None,
    "fuzz-regressions": "code",
    "scripting-wasm": "code",
    "test": "code",
    "vscode-extension": "code",
    "neovim-plugin": "code",
    "stdlib-runtime": "code",
    "upstream-smoke": "code",
    "sanitizers": "code",
    # Not a job of its own: the macOS leg of `sanitizers`.
    "sanitizers-macos": "sanitizers-macos",
    "sdl3": "sdl3",
    "clipboard-linux": "clipboard",
    "nix": "nix",
    "shellcheck": "shell",
    "install-sh": "packaging",
    "install-ps1": "packaging",
    "homebrew": "packaging",
    "windows-cross": "windows",
    "windows-run-cross": None,
    "windows-native": "windows",
    "windows-stdlib": "windows",
    "windows-smoke": "windows",
}

# The two tiers. Pushes and pull requests run the fast tier, which aims to finish in ten minutes:
# platform-neutral work runs once, on Linux x64, and the long suites are spread over several jobs.
# The nightly run (and a manual run, unless its `tier` input is `fast`) is the full tier, which adds
# everything the fast tier leaves out: Linux arm64 and macOS Intel, -O2 sweeps, sanitizer -O2, the
# macOS tooling and language-server tests, and the native tests built in four parts.
X64 = {"runner": "ubuntu-24.04", "architecture": "x86_64"}
ARM = {"runner": "ubuntu-24.04-arm", "architecture": "aarch64"}
MAC = {"runner": "macos-15", "architecture": "arm64"}
INTEL = {"runner": "macos-15-intel", "architecture": "x86_64"}

# On macOS the fast tier runs only the crates whose tests depend on the host: the compiler, its
# backend and the test runner.
HOST_CRATES = "package(jaic-cli) | package(jaic-llvm) | package(jaic) | package(jacket)"
# The WebAssembly target tests run node on whatever the host is and build the same modules everywhere,
# so only the Linux x64 legs run them.
NOT_WASM = "not test(wasm_target)"


def shards(count):
    return [f"{k}/{count}" for k in range(1, count + 1)]


def test_matrix(full):
    """The legs of the `test` job: `part` picks the steps, `shard` the share of them. The other
    hosts leave out what does not depend on the host: the WebAssembly target tests (NOT_WASM), the
    policy tests and Jai format check (the `jai` leg) and the native tests built in four parts
    (`split`, on Linux arm64 only)."""
    legs = [dict(X64, part="rust", shard=s) for s in shards(6)]
    legs += [dict(X64, part="targets", shard=s) for s in shards(3)]
    parts = ("jai", "lint", "portability", "lint-windows", "lint-macos", "diff")
    legs += [dict(X64, part=part, shard="1/1") for part in parts]
    if not full:
        return legs + [
            dict(MAC, part="rust", shard=s, expr=f"({HOST_CRATES}) & {NOT_WASM}") for s in shards(2)
        ]
    for host in (ARM, MAC, INTEL):
        legs += [dict(host, part="rust", shard=s, expr=NOT_WASM) for s in shards(3)]
        legs += [dict(host, part=part, shard="1/1") for part in ("targets", "lint", "diff")]
    legs.append(dict(ARM, part="split", shard="1/1"))
    return legs


# The producer suites (tools/stdlib/check_stdlib_suites.py) but `tooling` and `bindings`, which
# take longest and have legs of their own.
SUITES = "foundations portable platform audio window numerics numerics-native"
# The tooling suite runs in this many parts, each about five minutes on CI.
TOOLING_PARTS = 4


def stdlib_matrix(full):
    """The legs of `stdlib-runtime`: `shard` is a share of the catalogue, with `modes` its
    stdlib_runtime.py modes and `o2` whether it reruns them at -O2, or names a leg that runs the
    producer suites in `suites` (the `suites` leg also runs the WebGPU checks). The WebAssembly modes and
    the `jacket test` wasm runs do not depend on the host, so only Linux x64 has them. `jacket_test`
    is the share of `jacket test` a leg runs: `tooling` legs take their part of the tests, and the
    `suites` leg runs them all on a platform with no `tooling` legs."""
    everything = "interp,native,wasm-interp,wasm-native"

    def suite_legs(platform, names):
        wasm = platform == "linux-x64"
        legs = [
            dict(platform=platform, shard="suites", suites=SUITES, wasm=wasm, jacket_test="" if "tooling" in names else "all")
        ]
        for name in names:
            if name == "tooling":
                # check_developer_tools.py splits its tests across these legs (JACKET_TOOLING_PART).
                legs += [
                    dict(
                        platform=platform,
                        shard=f"tooling {k} of {TOOLING_PARTS}",
                        suites=name,
                        tooling_part=f"{k}/{TOOLING_PARTS}",
                        jacket_test=f"hash:{k}/{TOOLING_PARTS}",
                        wasm=wasm,
                    )
                    for k in range(1, TOOLING_PARTS + 1)
                ]
            else:
                legs.append(dict(platform=platform, shard=name, suites=name))
        return legs

    if not full:
        legs = [dict(platform="linux-x64", shard=s, modes=everything, o2=False) for s in shards(3)]
        legs += [dict(platform="macos-arm64", shard=s, modes="interp,native", o2=False) for s in shards(2)]
        return legs + suite_legs("linux-x64", ("bindings", "tooling")) + suite_legs("macos-arm64", ())
    legs = []
    for platform in ("linux-x64", "linux-arm64", "macos-arm64", "macos-x64"):
        modes = everything if platform == "linux-x64" else "interp,native"
        legs += [dict(platform=platform, shard=s, modes=modes, o2=True) for s in shards(2)]
        legs += suite_legs(platform, ("bindings", "tooling"))
    return legs


def sanitizer_matrix(full, macos):
    """The legs of `sanitizers`: the fast tier runs -O0 only, so Linux takes three shards there."""
    linux = shards(2 if full else 3)
    legs = [dict(runner="ubuntu-24.04", shard=s) for s in linux]
    return legs + ([dict(runner="macos-15", shard="1/1")] if full or macos else [])


def windows_stdlib_matrix(full):
    """The legs of `windows-stdlib`: the fast tier skips the -O2 rerun and arm64, and splits x64
    more finely."""
    cpus = {
        "x64": dict(runner="windows-2025", triple="x86_64-pc-windows-msvc", timeout=40),
        "arm64": dict(runner="windows-11-arm", triple="aarch64-pc-windows-msvc", timeout=80),
    }
    counts = {"x64": 2, "arm64": 2} if full else {"x64": 5}
    return [dict(cpu=cpu, shard=s, o2=full, **cpus[cpu]) for cpu in windows_cpus(full) for s in shards(counts[cpu])]


WINDOWS = {
    "x64": dict(runner="windows-2025", triple="x86_64-pc-windows-msvc", vs_arch="amd64", mingw="MinGW-w64"),
    "arm64": dict(runner="windows-11-arm", triple="aarch64-pc-windows-msvc", vs_arch="arm64", mingw="llvm-mingw"),
}


def windows_cpus(full):
    """Windows arm64 runs only in the full tier: the fast tier is bound by how many jobs GitHub
    runs at once, and x64 covers the Windows code paths."""
    return ["x64", "arm64"] if full else ["x64"]


def windows_native_matrix(full):
    """The legs of `windows-native`: `programs` and `graphics` per CPU."""
    timeouts = {"x64": 30, "arm64": 60}
    return [
        dict(cpu=cpu, part=part, mesa_dir="target/mesa-driver", timeout=timeouts[cpu], **WINDOWS[cpu])
        for cpu in windows_cpus(full)
        for part in ("programs", "graphics")
    ]


def windows_cross_matrix(full):
    """The legs of `windows-cross` and `windows-run-cross`: a third of the programs each
    (windows_cross.py --shard; the first also builds the shared fixtures)."""
    return [dict(cpu=cpu, shard=str(k), **WINDOWS[cpu]) for cpu in windows_cpus(full) for k in (1, 2, 3)]


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
    result = subprocess.run(
        ["git", "diff", "--name-only", spec], capture_output=True, text=True
    )
    return result.stdout.split() if result.returncode == 0 else None


def selected_jobs(results, requested):
    """The ids of JOBS to run: those whose area is true, limited to the comma-separated `requested` when it is not empty."""
    wanted = {name.strip() for name in requested.split(",") if name.strip()}
    return [
        job
        for job, area in JOBS.items()
        if (not wanted or job in wanted) and (area is None or results[area])
    ]


def main():
    env = os.environ
    files = changed_files(env)
    event = env.get("GITHUB_EVENT_NAME", "")
    results = {}
    for area, patterns in AREAS.items():
        results[area] = files is None or matches(
            patterns, files, area not in OWN_PATHS_ONLY
        )
    results["code"] = files is None or code_changed(files)
    if event == "pull_request":
        results["windows"] = False
    lines = [f"{area}={str(value).lower()}" for area, value in results.items()]
    lines.append("jobs=" + json.dumps(selected_jobs(results, env.get("JOBS", ""))))
    full = event == "schedule" or (event == "workflow_dispatch" and env.get("TIER", "full") != "fast")
    lines.append(f"full={str(full).lower()}")
    lines.append("test_matrix=" + json.dumps(test_matrix(full)))
    lines.append("stdlib_matrix=" + json.dumps(stdlib_matrix(full)))
    macos_asan = "sanitizers-macos" in selected_jobs(results, env.get("JOBS", ""))
    lines.append("sanitizer_matrix=" + json.dumps(sanitizer_matrix(full, macos_asan)))
    lines.append("windows_stdlib_matrix=" + json.dumps(windows_stdlib_matrix(full)))
    lines.append("windows_native_matrix=" + json.dumps(windows_native_matrix(full)))
    lines.append("windows_cross_matrix=" + json.dumps(windows_cross_matrix(full)))
    print("\n".join(lines))
    if env.get("GITHUB_OUTPUT"):
        with open(env["GITHUB_OUTPUT"], "a") as out:
            out.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
