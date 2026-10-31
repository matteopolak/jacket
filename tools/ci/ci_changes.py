#!/usr/bin/env python3
"""Decide which path-gated CI jobs run, and write `<area>=true|false` lines to $GITHUB_OUTPUT.

Pushes diff the pushed range, pull requests diff against their base, anything else (manual
runs, a new branch, a range that cannot be diffed) runs every area. An area is true when a changed
file is one of the inputs of its jobs, which job_inputs.py computes from ci.yml: the paths the
steps name, the scripts and Jai sources those read, and the crates they build (no path is listed
here). A change to ci.yml runs every area except those in OWN_PATHS_ONLY; a change to a composite
action runs the jobs that use it. `*` matches within a directory, `**` across them.
NARROW holds what is not about inputs: the macOS sanitizer leg runs on pushes only for the code
that decides what the sanitized binaries are. Windows never runs on pull requests (see main).
`code` is false when every changed file is prose (PROSE, minus the docs some test reads), so a
docs-only push skips the build and test jobs.

It also writes `full=true|false` (the tier, see test_matrix) and the `test_matrix` and
`stdlib_matrix` JSON the `test` and `stdlib-runtime` jobs expand, and `jobs=<JSON array>`: the ids in JOBS that run, which are the jobs whose area is true
(or that have none), narrowed to the comma-separated `$JOBS` of a manual run. Every job of ci.yml
tests `contains(fromJSON(needs.changes.outputs.jobs), '<id>')`; a new job gets a row in JOBS, and
a gated one is added to job_inputs.GATED.

    python3 tools/ci/ci_changes.py          # reads GITHUB_EVENT_NAME, JOBS, EVENT_BEFORE, BASE_SHA, SHA
"""

import json
import os
import re
import subprocess
import sys

import job_inputs

# A change to the workflow runs every area but OWN_PATHS_ONLY; the actions a job uses are among
# its inputs.
SELF = r".github/workflows/ci\.yml"

# Every job of ci.yml with the area that gates it (None: always, subject to a manual run's list).
JOBS = {
    "docs": None,
    "fuzz-regressions": "code",
    "dependency-hygiene": "code",
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
HOST_CRATES = "package(jaic-cli) | package(jaic-llvm) | package(jaic-native) | package(jaic-backend) | package(jaic) | package(jaic-syntax) | package(jacket)"
# The WebAssembly target tests run node on whatever the host is and build the same modules everywhere,
# so only the Linux x64 legs run them.
NOT_WASM = "not test(wasm_target)"


def shards(count):
    return [f"{k}/{count}" for k in range(1, count + 1)]


# GitHub runs only a few macOS jobs at once, and every job repeats the setup and the build, so each
# macOS host runs few, long jobs: one job per host takes the work that shares a build (below). The
# timeout is the job's, in minutes.
MACOS_TIMEOUT = 150


def leg(host, part, shard="1/1", parts=None, **extra):
    """A leg of the `test` job. `parts` lists the parts whose steps the leg runs, as ",a,b," (the
    steps test `contains(matrix.parts, ',a,')`); it is the leg's own `part` unless a leg merges several."""
    return dict(host, part=part, shard=shard, parts=f",{parts or part},", **extra)


def test_matrix(full):
    """The legs of the `test` job: `part` names the leg and `parts` picks its steps, `shard` is the
    share of the tests. The other hosts leave out what does not depend on the host: the WebAssembly
    target tests (NOT_WASM), the policy tests and Jai format check (the `jai` leg) and the native
    tests built in four parts (`split`, on Linux arm64 only). A macOS host runs two jobs: `rust`
    (the whole test suite, formatted and linted once) and `checks` (the stdlib target checks, the
    Jai lint and the backend agreement, which share one debug build of jaic)."""
    legs = [leg(X64, "rust", s) for s in shards(6)]
    legs += [leg(X64, "targets", s) for s in shards(3)]
    parts = ("jai", "lint", "portability", "lint-windows", "lint-macos", "diff")
    legs += [leg(X64, part) for part in parts]
    if not full:
        return legs + [
            leg(MAC, "rust", s, expr=f"({HOST_CRATES}) & {NOT_WASM}") for s in shards(2)
        ]
    legs += [leg(ARM, "rust", s, expr=NOT_WASM) for s in shards(3)]
    legs += [leg(ARM, part) for part in ("targets", "lint", "diff", "split")]
    for host in (MAC, INTEL):
        legs.append(leg(host, "rust", expr=NOT_WASM, timeout=MACOS_TIMEOUT))
        legs.append(
            leg(host, "checks", parts="targets,lint,diff", timeout=MACOS_TIMEOUT)
        )
    return legs


# The producer suites (tools/stdlib/check_stdlib_suites.py) but `tooling` and `bindings`, which
# take longest and have legs of their own.
SUITES = "portable platform audio window"
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
            dict(
                platform=platform,
                shard="suites",
                suites=SUITES,
                wasm=wasm,
                jacket_test="" if "tooling" in names else "all",
                # The standard library's coverage is measured once, on the platform that has the most tests.
                coverage=platform == "linux-x64",
            )
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
        legs = [
            dict(platform="linux-x64", shard=s, modes=everything, o2=False)
            for s in shards(3)
        ]
        legs += [
            dict(platform="macos-arm64", shard=s, modes="interp,native", o2=False)
            for s in shards(2)
        ]
        return (
            legs
            + suite_legs("linux-x64", ("bindings", "tooling"))
            + suite_legs("macos-arm64", ())
        )
    legs = []
    for platform in ("linux-x64", "linux-arm64"):
        modes = everything if platform == "linux-x64" else "interp,native"
        legs += [
            dict(platform=platform, shard=s, modes=modes, o2=True) for s in shards(2)
        ]
        legs += suite_legs(platform, ("bindings", "tooling"))
    for platform in ("macos-arm64", "macos-x64"):
        # Two jobs per Mac: the whole catalogue, and every producer suite with all the
        # `jacket test` runs (they share the production build, and their parts only exist to
        # shorten Linux jobs).
        legs.append(
            dict(
                platform=platform,
                shard="1/1",
                modes="interp,native",
                o2=True,
                timeout=MACOS_TIMEOUT,
            )
        )
        legs.append(
            dict(
                platform=platform,
                shard="suites",
                suites=f"{SUITES} bindings tooling",
                jacket_test="all",
                timeout=MACOS_TIMEOUT,
            )
        )
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
        "arm64": dict(
            runner="windows-11-arm", triple="aarch64-pc-windows-msvc", timeout=80
        ),
    }
    counts = {"x64": 2, "arm64": 2} if full else {"x64": 5}
    return [
        dict(cpu=cpu, shard=s, o2=full, **cpus[cpu])
        for cpu in windows_cpus(full)
        for s in shards(counts[cpu])
    ]


WINDOWS = {
    "x64": dict(
        runner="windows-2025",
        triple="x86_64-pc-windows-msvc",
        vs_arch="amd64",
        mingw="MinGW-w64",
    ),
    "arm64": dict(
        runner="windows-11-arm",
        triple="aarch64-pc-windows-msvc",
        vs_arch="arm64",
        mingw="llvm-mingw",
    ),
}


def windows_cpus(full):
    """Windows arm64 runs only in the full tier: the fast tier is bound by how many jobs GitHub
    runs at once, and x64 covers the Windows code paths."""
    return ["x64", "arm64"] if full else ["x64"]


def windows_native_matrix(full):
    """The legs of `windows-native`: `programs` and `graphics` per CPU."""
    timeouts = {"x64": 30, "arm64": 60}
    return [
        dict(
            cpu=cpu,
            part=part,
            mesa_dir="target/mesa-driver",
            timeout=timeouts[cpu],
            **WINDOWS[cpu],
        )
        for cpu in windows_cpus(full)
        for part in ("programs", "graphics")
    ]


def windows_cross_matrix(full):
    """The legs of `windows-cross` and `windows-run-cross`: a third of the programs each
    (windows_cross.py --shard; the first also builds the shared fixtures)."""
    return [
        dict(cpu=cpu, shard=str(k), **WINDOWS[cpu])
        for cpu in windows_cpus(full)
        for k in (1, 2, 3)
    ]


# Areas too costly to run whenever the workflow or an action changes.
OWN_PATHS_ONLY = {"sdl3"}

# The job of ci.yml whose inputs decide a JOBS id that is no job of its own (the macOS leg).
INPUT_JOB = {"sanitizers-macos": "sanitizers"}

# Policy narrowing an area's computed inputs: a changed file selects the area only when it is an
# input and also matches one of these. macOS ASan is slot-limited and the slowest sanitizer leg:
# on pushes it runs only for the code that decides what the sanitized binaries are (code
# generation, the driver, the sweep and its inputs, the toolchain); Linux ASan runs on every push,
# and nightly/manual runs do both.
NARROW = {
    "sanitizers-macos": [
        "crates/jaic-llvm/**",
        "crates/jaic-native/**",
        "crates/jaic-backend/**",
        "crates/jaic-cli/src/**",
        "crates/jaic/src/**",
        "crates/jaic-syntax/src/**",
        "tools/differential/**",
        "tools/native/**",
        "corpus/**",
        "Cargo.lock",
        "rust-toolchain.toml",
    ],
}

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


def area_jobs(area):
    """The ci.yml jobs whose inputs decide `area`."""
    return [INPUT_JOB.get(job, job) for job, owner in JOBS.items() if owner == area]


def input_areas():
    """The areas gated on computed inputs: every JOBS area but `code`."""
    return sorted({area for area in JOBS.values() if area and area != "code"})


def area_patterns(area, repo, workflow_text, graph=None, jaic=None, crates=None):
    """The input patterns of `area`: the union of its jobs' computed inputs (workflow files only
    where the area is not OWN_PATHS_ONLY)."""
    found = set()
    for job in area_jobs(area):
        found |= job_inputs.inputs(
            job, repo, workflow_text, graph, crates=crates, jaic=jaic
        )
    if area in OWN_PATHS_ONLY:
        found = {p for p in found if not p.startswith(".github/")}
    return job_inputs.collapse(found)


def area_changed(area, patterns, files):
    """Whether a changed file selects `area` (its inputs, narrowed by NARROW)."""
    narrow = NARROW.get(area)
    workflow = SELF if area not in OWN_PATHS_ONLY else None
    inputs = compile_patterns(patterns, workflow)
    narrowed = compile_patterns(narrow, workflow) if narrow else None
    return any(
        inputs.match(f) and (narrowed is None or narrowed.match(f)) for f in files
    )


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


def area_results(files):
    """{area: whether it runs} for the changed `files` (`None`: all of them). If the inputs
    cannot be computed every area runs, which only costs minutes."""
    if files is None:
        return {area: True for area in input_areas()}
    try:
        repo = job_inputs.Repo()
        workflow_text = repo.read(job_inputs.WORKFLOW)
        graph = job_inputs.crate_graph(repo)
        jaic = os.environ.get("JAIC_DEPS_JAIC")
        return {
            area: area_changed(
                area, area_patterns(area, repo, workflow_text, graph, jaic), files
            )
            for area in input_areas()
        }
    except Exception as error:
        print(
            f"ci_changes: cannot compute the job inputs ({error!r}); running every area",
            file=sys.stderr,
        )
        return {area: True for area in input_areas()}


def main():
    env = os.environ
    files = changed_files(env)
    event = env.get("GITHUB_EVENT_NAME", "")
    results = area_results(files)
    results["code"] = files is None or code_changed(files)
    if event == "pull_request":
        # Policy, not inputs: the Windows jobs run on pushes to main and staging and on manual runs.
        results["windows"] = False
    lines = [f"{area}={str(value).lower()}" for area, value in results.items()]
    lines.append("jobs=" + json.dumps(selected_jobs(results, env.get("JOBS", ""))))
    full = event == "schedule" or (
        event == "workflow_dispatch" and env.get("TIER", "full") != "fast"
    )
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
