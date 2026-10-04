# Upstream corpus

## What it is

Pinned source snapshots of recently maintained open-source Jai projects (focus-editor/focus, Ivo-Balbaert/The_Way_to_Jai, SogoCZE/Jails, rluba/jaison, withlang-dev/open-jai, ostef/Vk-Engine, roeyb1/sgpu, plus the dependencies SogoCZE/jai_parser, ostef/Linalg and ostef/Jolt-Jai). They are the real-world programs `jaic` is measured against. They live in the gitignored `corpus/upstream/`; the committed `corpus/upstreams.json` records exact commits and file hashes.

## How it works

`tools/fetch_upstreams.py` downloads the pinned `.jai` files, READMEs, licenses, C-family sources (`NATIVE_SOURCE_SUFFIXES`, needed by native-library builds) and compile-time data files (`RESOURCE_PREFIXES`, e.g. focus's `config/` and `fonts/`) into `corpus/upstream/<owner>--<repo>/`. Existing pins in the manifest are kept; `--since YYYY-MM-DD` (default 2025-10-01) is the recency cutoff for newly added repositories, and `DEPENDENCIES` (submodules) are exempt. `MODULE_LINKS` symlinks dependencies into the consumers' `modules/` directories. Nothing from a project is built or executed by the fetcher.

Dependencies are pinned like projects: jai_parser for Jails; Linalg, Jolt-Jai and Jolt-Jai's JoltC submodule for Vk-Engine (JoltC has no Jai files; the fetcher takes its `CMakeLists.txt` and `Examples/`). `corpus/upstream` and the Git cache live in the main checkout (found through the Git common dir), so worktrees share them.

`tools/verify_upstreams.py` re-hashes the fetched tree against the manifest and rejects modified or missing files. Unlisted files are reported, not rejected: building the projects (sweep `build` cases, `build_native_libs.py`, `build_vk_engine_libs.py`) leaves libraries, executables and generated files next to the sources.

## How to change it

Add a repository to `REPOSITORIES`/`DEPENDENCIES` in `fetch_upstreams.py`, run it, review the manifest diff and commit `corpus/upstreams.json` (never `corpus/upstream/`). Add entry points that compile to `tools/upstream-cases.json` so [jaic-sweep](jaic-sweep.md) keeps them green. Current per-project status is in `HANDOFF.md`.

## Configuration

`--since YYYY-MM-DD`. Network access to GitHub. Tests: `tools/test_upstreams.py`.

## Dependencies

Git, Python 3.11+. Native libraries the projects need are built by [native-libs](native-libs.md).
