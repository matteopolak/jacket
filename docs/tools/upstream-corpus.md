# Upstream corpus

## What it is

Pinned source snapshots of recently maintained open-source Jai projects (focus-editor/focus, Ivo-Balbaert/The_Way_to_Jai, SogoCZE/Jails, rluba/jaison, withlang-dev/open-jai, ostef/Vk-Engine, roeyb1/sgpu, plus the dependencies SogoCZE/jai_parser, ostef/Linalg and ostef/Jolt-Jai). They are the real-world programs `jaic` is measured against. They live in the gitignored `corpus/upstream/`; the committed `corpus/upstreams.json` records exact commits and file hashes.

## How it works

`tools/fetch_upstreams.py` downloads the pinned `.jai` files, READMEs, licenses, C-family sources (`NATIVE_SOURCE_SUFFIXES`, needed by native-library builds) and compile-time data files (`RESOURCE_PREFIXES`, e.g. focus's `config/` and `fonts/`) into `corpus/upstream/<owner>--<repo>/`. Existing pins in the manifest are kept; `--since YYYY-MM-DD` (default 2025-10-01) is the recency cutoff for newly added repositories, and `DEPENDENCIES` (submodules) are exempt. `MODULE_LINKS` symlinks dependencies into the consumers' `modules/` directories. Nothing from a project is built or executed by the fetcher.

Dependencies are pinned like projects: jai_parser for Jails; Linalg, Jolt-Jai and Jolt-Jai's JoltC submodule for Vk-Engine (JoltC has no Jai files; the fetcher takes its `CMakeLists.txt` and `Examples/`). `corpus/upstream` and the Git cache live in the main checkout (found through the Git common dir), so worktrees share them.

`tools/verify_upstreams.py` re-hashes the fetched tree against the manifest and rejects modified or missing files. Unlisted files are reported, not rejected: building the projects (sweep `build` cases, `build_native_libs.py`, `build_vk_engine_libs.py`) leaves libraries, executables and generated files next to the sources.

### Project status

- **focus-editor**: `jaic build first.jai` produces a working native editor on macOS (renders, takes input).
  Needs `python3 tools/build_native_libs.py` (stb libraries) and its own
  `modules/Objective_C/LightweightRenderingView/build.jai` run once (`jaic build build.jai` there). Debug builds
  need `~/Library/Application Support/dev.focus-editor` to exist (upstream creates `.../debug` non-recursively).
- **Vk-Engine** (+ Linalg, Jolt-Jai): `jaic check Build.jai -I Modules -I Source -os linux - Core|Renderer|Game|Editor`
  passes (the ImGui and Vulkan generators run on the real headers). Native macOS stops in about 3 s: the upstream
  `Vulkan`, `ImGui` and `JoltPhysics` modules and their `generate.jai` have no macOS branch (editing the upstream
  project is out of scope). `python3 tools/build_vk_engine_libs.py` builds `libImGui`, `libVkMemAlloc` and `libJoltC`
  for macOS. See [Vk-Engine](../native/vk-engine.md).
- **jaison**: tests and example run, also natively (`jaic build tests.jai`). **sgpu**: all examples check (host, linux, windows) and build natively on macOS
  after `python3 tools/build_slang.py` (Slang 2025.24.2, VMA and the Vulkan loader built from source); with MoltenVK
  all run except 04_mesh_shaders (MoltenVK has no `VK_EXT_mesh_shader`). Run commands: [native libraries](native-libs.md).
- **Jails**: `jaic build build.jai` produces a native `bin/jails` that answers LSP requests.
  Jails `-os windows` needs a Windows host (compile-time `MultiByteToWideChar`).
- **The_Way_to_Jai**: 343 entry points are sweep cases (`tools/upstream-cases.json`): 316 run to completion,
  the rest check (windowed Simp programs, interactive or endless ones, deliberate crashes, user-built libraries).
  The files that fail `check` are not compiler bugs: Windows-only APIs (19.8, 33.2C, 33.6, 50.1), the Windows-only
  raylib module (35.1, 52.2, 30/jai_raylib), intentional failures (20.2, 30.9, exercises/22), APIs older Jai versions
  had (6.6 `random_seed` result, 26.27 and exercises/30 `builder_to_string(allocator=)`, 33.10 `Sound_Player`
  struct, 51.2 GetRect `dropdown`), missing command-line arguments (30.14, 8.2, 12.8), a missing
  `cpp_library.cpp`, and 31.2, which calls GL at compile time without a context (crashes in libGL). 19.5 frees an
  advanced pointer and 27/foldera writes through null (both upstream bugs).

Vk-Engine needs a `--release` build (about 45 s per module):
`cd corpus/upstream/ostef--Vk-Engine && jaic check Build.jai -I Modules -I Source -os linux - Core|Renderer|Game|Editor`.

## How to change it

Add a repository to `REPOSITORIES`/`DEPENDENCIES` in `fetch_upstreams.py`, run it, review the manifest diff and commit `corpus/upstreams.json` (never `corpus/upstream/`). Add entry points that compile to `tools/upstream-cases.json` so [jaic-sweep](jaic-sweep.md) keeps them green. Current per-project status is under [Project status](#project-status).

## Configuration

`--since YYYY-MM-DD`. Network access to GitHub. Tests: `tools/test_upstreams.py`.

## Dependencies

Git, Python 3.11+. Native libraries the projects need are built by [native-libs](native-libs.md).
