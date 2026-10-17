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

Expected values must come from the project (docs, tests, golden outputs written by people who ran Jai), never from jaic's own output, which only freezes current behaviour. A missing expected line is a jaic bug until shown otherwise; reject an annotation only when the book is wrong about current Jai. Per-example outputs from open-jai are not used: it produced them with its own reimplementation.

## Smoke tests

`tools/upstream_smoke.py` builds each project with its own entry point and runs it unattended. Checks live in `tools/upstream-smoke.json` (validated by `tools/test_upstream_smoke.py`); CI runs them in the `upstream-smoke` job of `ci.yml` on linux-x64, linux-arm64, macos-arm64 and windows-x64.

```sh
python3 tools/fetch_upstreams.py --only $(python3 tools/upstream_smoke.py --jaic x --projects)
python3 tools/upstream_smoke.py --jaic target/release/jaic              # every check for this machine
python3 tools/upstream_smoke.py --jaic target/release/jaic --filter focus -v
```

Each check copies its project into a scratch directory and runs steps in order. Step `kind`: `jaic` (`outputs` must exist, `contains`/`ordered`/`not_contains` check output), `run` (default; `stdin`, `exit`), `gui` (must still run after `seconds`, and the screen must change between screenshots taken before and after start), `serve` (free port via `{port}`, fetch `get`), `lsp` (`initialize` over stdio). `needs` runs a step only if a program was built, `only`/`skip` limit platforms, and a check with `only`/`skip` or no steps is a documented skip with a reason. Servers, editors and games need `serve` or `gui`, never a bare `run`. GUI programs run under `xvfb-run` on Linux; on Windows Mesa's software `opengl32.dll` is copied beside them.

Known deviations: Focus builds with `first.jai - release` and, on Windows, is built but not run (Mesa reports a GL error its Simp backend turns into a breakpoint); reflector's `build.jai` also builds a Windows-only benchmark, so the check builds `tools/upstream-drivers/reflector-tests.jai` instead, and its case patches uninitialised arrays upstream leaves undefined; jai-tracy compiles libtracy from the pinned client instead of running `generate.jai`; toml-jai's `tests.jai` spawns a `jai` per example, so the check runs each itself.

x64-only projects: some cases need an x86-64 host, so a Mac or Arm machine cannot show them passing. They run as `upstream` sweep cases (`tools/upstream-cases.json`) in the `third-party projects` steps of `ci.yml`: the Linux x64 job runs `ttwj-28-28.5-get-cpu-info-feature-flags` (CPUID), `ttwj-28-28.7-asm-and-macros` (`#asm` with `__reg` macro parameters), `ttwj-30-jai-raylib-build`, `chess-jai-ui-check`, `chess-jai-engine-check` and the four `vk-engine-*-check` cases; the Windows x64 job runs the same without Vk-Engine. All pass on both hosts. The Vk-Engine cases pass `-os linux -cpu x64` because the non-x64 `Default_Allocator` has no `initialize_if_needed`, which Vk-Engine calls.

Cross-target compile-time code: `#run` always executes on the machine running `jaic`, so a project built with `-os linux` on a Mac runs Linux-only calls against the Mac libc. Monotonic clocks are covered (`current_time_monotonic` reads the host clock when the target OS differs from the compiler's), but code that forks, pipes or locks files in a plain `#run` (Vk-Engine's `Build.jai` uses `run_command`) fails with a Process error on a foreign host. Use `#run,host` for host-only work (`docs/specs/jai/polymorphism-and-metaprogramming.md`), or check such a project on a host of its own OS. The four `vk-engine-*-check` cases are therefore real checks only on Linux; they still fail on macOS with "process result read failed".

Projects that do not fully work, and why: Vk-Engine (checks for Linux only, no macOS support upstream), jai-postgres and jai-redis (need a database or server), Photon (Windows-only), KodaJai (needs unpinned modules by the same author), no_api (its entry point loads a file missing upstream), ui_builder (link needs an unpinned prebuilt `libslang`), sgpu (examples build on macOS after `tools/build_slang.py`; mesh shaders need a driver MoltenVK lacks), wait_group on Linux (upstream typo: `signal` where `it` was meant), The Way to Jai (examples that check but do not run are Windows-only, written for an older Jai, interactive, or deliberately failing).

## How to change it

- **Add a project:** add it to `REPOSITORIES`, `DEPENDENCIES` or `LIBRARIES`, run `fetch_upstreams.py`, review and commit `corpus/upstreams.json` (never `corpus/upstream/`), add entry points to `tools/upstream-cases.json`, and a smoke check if it can run. Prefer projects that check themselves. A project importing unpinned modules from the same author can only be tested once those are pinned (`LIBRARIES` plus `MODULE_LINKS`).
- **Find candidates:** `gh search repos --language=Jai --stars=">=20" --updated=">=<date of last pass>" --limit 100 --json fullName,stargazersCount,pushedAt`; "pushed" in the REST query (`language:Jai pushed:>=...`) means code changed, while "updated" also counts stars and issues.
- **Try it:** entry point is usually `first.jai` or `build.jai` (arguments after `-` select targets). Try `jaic -check`, then a build, then a run, with a release `jaic`. Classify: passes; jaic or stdlib bug (reduce to a test under `tests/stdlib/` and fix); needs native libraries or another platform (`-os linux`/`-os windows` still type-checks); outdated or broken upstream (note the line).
- **Verify on every platform** before trusting a smoke check: `-os` flags only type-check. Push a `tmp/...` branch, run CI there, then delete it.
- The largest projects can also be [benchmark](benchmarks.md) workloads.

## Dependencies

Git, Python 3.11+, network access to GitHub; `gh` (authenticated) for searching; clang for cases that compile vendored C (forbear). Native libraries the projects need come from [native libraries](native-libs.md); `JAIC_NATIVE_LIBS` points builds at them.
