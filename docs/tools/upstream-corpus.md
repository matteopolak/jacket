# Upstream corpus

## What it is

Pinned snapshots of open-source Jai projects that `jaic` is measured against: applications (Focus, Jails, The Way to Jai, Vk-Engine, sgpu, chess-jai, forbear, ...), their dependencies (jai_parser, Linalg, Jolt-Jai, Tracy's client) and libraries (metaprogramming libraries such as match-jai and yield-jai, rluba's family such as jaison, uniform and hyperserve, and libraries with their own tests: reflector, jai-format, jai-protobuf, toml-jai, jai-xml). The authoritative lists are `REPOSITORIES`, `DEPENDENCIES` and `LIBRARIES` in `tools/fetch_upstreams.py`. jaithon is excluded: it is its own language in `.jai` files.

Files live in the gitignored `corpus/upstream/`; the committed `corpus/upstreams.json` records exact commits and file hashes. Both the corpus and the Git cache live in the main checkout (via the Git common dir), so worktrees share them.

## How it works

`tools/fetch_upstreams.py` downloads the pinned `.jai` files, READMEs, licences, C-family sources (`NATIVE_SOURCE_SUFFIXES`, for native-library builds) and compile-time data files (`RESOURCE_PREFIXES`) into `corpus/upstream/<owner>--<repo>/`. It never builds or runs a project.

- Existing pins are kept. `--since YYYY-MM-DD` (default 2025-10-01) is the recency cutoff for new repositories; `DEPENDENCIES` and `LIBRARIES` are exempt.
- `MODULE_LINKS` symlinks dependencies into consumers' `modules/` and libraries imported by name into `corpus/upstream/_modules/` (cases pass `-I ../../_modules`).
- `SOURCE_PREFIXES` limits a repository to some directories (Tracy: only `public/`); `MAX_FILE_BYTES` raises the size cap for a file (chess-jai's network).
- Building the projects leaves libraries and generated files next to the sources, so a hash mismatch or stray file after a build means restore with `fetch_upstreams.py` and give the case a `setup` step that avoids the write.

Entry points that work are sweep cases in `tools/upstream-cases.json` ([testing](testing.md)); `check` mode is used for heavy ones, with `setup` commands for prerequisites. Where a project documents its output, the case carries an `expect` record:

```json
{"id": "ttwj-16-16.3-enum-specified", "mode": "run",
 "path": "Ivo-Balbaert--The_Way_to_Jai/examples/16/16.3_enum_specified.jai",
 "expect": {"stdout_ordered": ["enum 'Direction' is", "specified"],
            "source": "The_Way_to_Jai `// =>` annotations, lines 20, 22, 26"}}
```

A case that cannot run on every host carries `only` (a list of platform names, e.g. `["linux-x64"]`) and `only_reason`; the sweep reports it as platform unavailable elsewhere instead of failing.

A case whose own source has a genuine error (a removed API, a deprecated directive, wrong arity) carries `disabled` with a short reason. The sweep and the smoke run skip it and count it separately; manifest rows of the stdlib and compiler test catalogues accept the same key.

Expected values must come from the project (docs, tests, golden outputs written by people who ran Jai), never from jaic's own output, which only freezes current behaviour. A missing expected line is a jaic bug until shown otherwise; reject an annotation only when the book is wrong about current Jai. Per-example outputs from open-jai are not used: it produced them with its own reimplementation.

## Smoke tests

`tools/corpus/upstream_smoke.py` builds each project with its own entry point and runs it unattended. Checks live in `tools/corpus/upstream-smoke.json` (validated by `tools/corpus/test_upstream_smoke.py`); CI runs them in the `upstream-smoke` job of `ci.yml` on linux-x64, linux-arm64, macos-arm64 and windows-x64.

```sh
python3 tools/fetch_upstreams.py --only $(python3 tools/corpus/upstream_smoke.py --jaic x --projects)
python3 tools/corpus/upstream_smoke.py --jaic target/release/jaic              # every check for this machine
python3 tools/corpus/upstream_smoke.py --jaic target/release/jaic --filter focus -v
```

Each check copies its project into a scratch directory and runs steps in order. Step `kind`: `jaic` (`outputs` must exist, `contains`/`ordered`/`not_contains` check output), `run` (default; `stdin`, `exit`), `gui` (must still run after `seconds`, and the screen must change between screenshots taken before and after start), `serve` (free port via `{port}`, fetch `get`), `lsp` (`initialize` over stdio). `needs` runs a step only if a program was built, `only`/`skip` limit platforms, and a check with `only`/`skip` or no steps is a documented skip with a reason. Servers, editors and games need `serve` or `gui`, never a bare `run`. GUI programs run under `xvfb-run` on Linux; on Windows Mesa's software `opengl32.dll` is copied beside them.

Known deviations: Focus builds with `first.jai - release` and, on Windows, is built but not run (Mesa reports a GL error its Simp backend turns into a breakpoint); reflector's `build.jai` also builds a Windows-only benchmark, so the check builds `tools/corpus/upstream-drivers/reflector-tests.jai` instead, and its case patches uninitialised arrays upstream leaves undefined; jai-tracy compiles libtracy from the pinned client instead of running `generate.jai`; toml-jai's `tests.jai` spawns a `jai` per example, so the check runs each itself.

Platform-restricted cases: a case that can only run on some hosts names them in `only` (with `only_reason`) in `tools/upstream-cases.json`; elsewhere it reports `PLATFORM_UNAVAILABLE`. The x64 steps of `ci.yml` do not list ids: `tools/corpus/select_upstream_cases.py --platform linux-x64|windows-x64` selects every enabled case whose `only` names that platform (`--ids`, or `--repos` for `fetch_upstreams.py --only`), and `jaic_sweep.py upstream --only-platform P` runs them, so a new Windows-only case needs no workflow edit. Cases restricted to x86-64 hosts: the CPUID and `#asm` examples of The Way to Jai, `ttwj-30-jai-raylib-build`, `chess-jai-ui-check`, `chess-jai-engine-check`, and (Linux only) the four `vk-engine-*-check` cases, which pass `-os linux -cpu x64` because the non-x64 `Default_Allocator` has no `initialize_if_needed`, which Vk-Engine calls. `tools/ci/ci_changes.py` runs the Windows job when the case list or the pins change.

Cross-target compile-time code: `#run` always executes on the machine running `jaic`, so a project built with `-os linux` on a Mac runs Linux-only calls against the Mac libc. Monotonic clocks are covered (`current_time_monotonic` reads the host clock when the target OS differs from the compiler's), but code that forks, pipes or locks files in a plain `#run` (Vk-Engine's `Build.jai` uses `run_command`) fails with a Process error on a foreign host. Use `#run,host` for host-only work (`docs/specs/jai/polymorphism-and-metaprogramming.md`), or check such a project on a host of its own OS. The four `vk-engine-*-check` cases are therefore real checks only on Linux; they still fail on macOS with "process result read failed".

Projects that do not fully work, and why: raylib-jai (examples are check-only: its prebuilt raylib and raygui libraries are binaries, not pinned, and `native-libs.md` does not build raylib, so nothing links or runs; five examples are disabled for upstream errors: C-style `(.*)p = x`, an unfinished `ifx mode ==`, an assignment inside `ifx` branches, and two float32 expressions that need float64 precision), Vk-Engine (checks for Linux only, no macOS support upstream), jai-postgres and jai-redis (need a database or server), Photon (Windows-only; `photon-check` type-checks `source/main.jai` with `-os windows -cpu x64` through `tools/corpus/upstream-drivers/photon-check.jai`, and is disabled at a construct in its vendored `Input` module, see the case), KodaJai (its modules and the other kujukuju libraries are pinned and reached as `-I` module paths; `tests/main.jai` and `input_poll_tests.jai` pass, but `module.jai` calls the undefined `create_cubemap` and the deferred example passes `bool` for an enum parameter, so those two are disabled), no_api (its entry point loads a file missing upstream), ui_builder (link needs an unpinned prebuilt `libslang`), sgpu (examples build on macOS after `tools/native/build_slang.py`; mesh shaders need a driver MoltenVK lacks), wait_group on Linux (upstream typo: `signal` where `it` was meant), The Way to Jai (examples that check but do not run are Windows-only, written for an older Jai, interactive, or deliberately failing).

## Sandbox

Everything under `corpus/upstream` is unreviewed third-party code, and compiling it runs its compile-time code (`#run`, build programs). So `jaic_sweep.py upstream` and `upstream_smoke.py` run every `jaic` invocation, setup command and produced executable through `tools/bench/sandbox.py`: no network, writes only to the case's scratch copy, reads limited to the system, the `jaic` build tree, `stdlib/`, `JAIC_NATIVE_LIBS` and `corpus/upstream`. macOS uses `sandbox-exec`, Linux `bwrap` (CI installs bubblewrap and lifts Ubuntu's unprivileged-namespace restriction). Servers and X11 clients get loopback (`Sandbox(loopback=True)`); the GUI step on Linux also reads `/tmp/.X11-unix`, and a Mesa DLL directory is read when given. Build and `run_after` cases work in a scratch copy of their project with the other repositories linked in, so a build that writes into its tree never touches the corpus.

Where no sandbox exists (Windows) the runs are unconfined on CI runners only: `tools/corpus/upstream_sandbox.py` refuses outside `CI` unless `--no-sandbox` is passed explicitly. Extra reads or writes a case needs are added as exact paths, never by opening the sandbox wholesale. `tools/corpus/test_upstream_sandbox.py` covers the policy, the wrapping and the write confinement.

## How to change it

- **Add a project:** add it to `REPOSITORIES`, `DEPENDENCIES` or `LIBRARIES`, run `fetch_upstreams.py`, review and commit `corpus/upstreams.json` (never `corpus/upstream/`), add entry points to `tools/upstream-cases.json`, and a smoke check if it can run. Prefer projects that check themselves. A project importing unpinned modules from the same author can only be tested once those are pinned (`LIBRARIES` plus `MODULE_LINKS`).
- **Find candidates:** `gh search repos --language=Jai --stars=">=20" --updated=">=<date of last pass>" --limit 100 --json fullName,stargazersCount,pushedAt`; "pushed" in the REST query (`language:Jai pushed:>=...`) means code changed, while "updated" also counts stars and issues.
- **Try it:** entry point is usually `first.jai` or `build.jai` (arguments after `-` select targets). Try `jaic -check`, then a build, then a run, with a release `jaic`. Classify: passes; jaic or stdlib bug (reduce to a test under `tests/compiler/regressions/` and fix); needs native libraries or another platform (`-os linux`/`-os windows` still type-checks); outdated or broken upstream (note the line).
- **Verify on every platform** before trusting a smoke check: `-os` flags only type-check. Push a `tmp/...` branch, run CI there, then delete it.
- The largest projects can also be [benchmark](benchmarks.md) workloads.

## Dependencies

Git, Python 3.11+, network access to GitHub; `gh` (authenticated) for searching; clang for cases that compile vendored C (forbear). Native libraries the projects need come from [native libraries](native-libs.md); `JAIC_NATIVE_LIBS` points builds at them.
