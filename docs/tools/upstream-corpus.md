# Upstream corpus

Pinned snapshots of open-source Jai projects that `jaic` is measured against: applications, their dependencies and libraries (some with their own tests). The authoritative lists are `REPOSITORIES`, `DEPENDENCIES`, `LIBRARIES` and `STAR_REPOSITORIES` in `tools/fetch_upstreams.py`.

Files live in the gitignored `corpus/upstream/`. The committed `corpus/upstreams.json` records each repository's commit, selection and file count; the fetcher recomputes the file list from the repository tree and checks each file against its object id. The corpus and cache live in the main checkout, so worktrees share them. The projects are unreviewed third-party code: run them only through the sandbox below.

## Fetching

`tools/fetch_upstreams.py` downloads the pinned `.jai` files, READMEs, licences, C-family sources and compile-time data into `corpus/upstream/<owner>--<repo>/`. It never builds or runs a project.

- Existing pins are kept. `--since YYYY-MM-DD` is the recency cutoff for new repositories.
- Binaries are never pinned (`is_forbidden_file`, `BINARY_SUFFIXES`, `BINARY_MAGIC`).
- Every file must hash to its object id, so a truncated download fails instead of being pinned.
- `MODULE_LINKS` symlinks dependencies into consumers' `modules/` and libraries into `corpus/upstream/_modules/` (cases pass `-I ../../_modules`). `SOURCE_PREFIXES` limits a repository to some directories; `MAX_FILE_BYTES` raises the size cap for a file.
- A build that writes into the corpus leaves a hash mismatch; restore with `fetch_upstreams.py` and give the case a `setup` step that avoids the write.

## Sweep cases

Working entry points are cases in `tools/upstream-cases.json`, run by the sweep ([testing](testing.md)). Heavy ones use `check` mode, with `setup` commands for prerequisites.

```json
{"id": "ttwj-16-16.3-enum-specified", "mode": "run",
 "path": "Ivo-Balbaert--The_Way_to_Jai/examples/16/16.3_enum_specified.jai",
 "expect": {"stdout_ordered": ["enum 'Direction' is", "specified"],
            "source": "The_Way_to_Jai `// =>` annotations, lines 20, 22, 26"}}
```

- `expect` values must come from the project (docs, tests, golden outputs), never from jaic's output, which only freezes current behaviour.
- `only` (platform names) and `only_reason`: reported as unavailable elsewhere instead of failing. `tools/corpus/select_upstream_cases.py --platform linux-x64|windows-x64` selects them for CI, so a new platform-only case needs no workflow edit.
- `disabled` with a reason: the case's own source has a genuine error. It is skipped and counted separately.
- `why_check`: one line on why a `check` case isn't promoted to `run`.
- `JAIC_UPSTREAM_CASES=<file>` reads cases from another file, for triage.
- Some cases pass although the official compiler rejects them, because jaic is deliberately more lenient and warns ([leniencies](../language/leniencies.md)).

## Smoke tests

`tools/corpus/upstream_smoke.py` builds each project with its own entry point and runs it unattended. Checks live in `tools/corpus/upstream-smoke.json` (validated by `test_upstream_smoke.py`); CI runs them in the `upstream-smoke` job.

```sh
PYTHONPATH=tools python3 tools/fetch_upstreams.py --only $(PYTHONPATH=tools python3 tools/corpus/upstream_smoke.py --jaic x --projects)
PYTHONPATH=tools python3 tools/corpus/upstream_smoke.py --jaic target/release/jaic
PYTHONPATH=tools python3 tools/corpus/upstream_smoke.py --jaic target/release/jaic --filter focus -v
```

Each check copies its project into a scratch directory and runs steps in order. Step `kind`: `jaic` (`outputs` must exist, `contains`/`ordered`/`not_contains`), `run` (default; `stdin`, `exit`), `gui` (still running after `seconds`, and the screen changed), `serve` (free port via `{port}`, fetch `get`), `lsp` (`initialize` over stdio). `needs`, `only` and `skip` limit steps; a check with no steps is a documented skip. Servers, editors and games need `serve` or `gui`, never a bare `run`. GUI programs run under `xvfb-run` on Linux, with Mesa's software `opengl32.dll` beside them on Windows.

Some projects only type-check (they need a database, a GPU, prebuilt binaries or another OS, or the upstream is broken); the reason is recorded in the check or case.

## Sandbox

Compiling a project runs its compile-time code, so `jaic_sweep.py upstream` and `upstream_smoke.py` run every `jaic` invocation, setup command and executable through `tools/bench/sandbox.py`: no network, writes only to the case's scratch copy, reads limited to the system, the `jaic` build tree, `stdlib/`, `JAIC_NATIVE_LIBS` and `corpus/upstream`. macOS uses `sandbox-exec`, Linux `bwrap`. Servers and X11 clients get loopback (`Sandbox(loopback=True)`).

Where no sandbox exists (Windows), runs are unconfined on CI runners only: `tools/corpus/upstream_sandbox.py` refuses outside `CI` unless `--no-sandbox` is passed. Add extra reads or writes as exact paths, never by opening the sandbox wholesale. Tests: `test_upstream_sandbox.py`.

## Star-qualified repositories

`STAR_REPOSITORIES` holds every `language:Jai` repository with at least `STAR_THRESHOLD` (5) stars, plus repositories found by code search whose language GitHub reports differently. They are unreviewed:

- **Scan first.** `tools/corpus/scan_upstream.py [owner--repo ...]` reads the pinned files (never runs anything) and prints a per-repository risk: process spawning, networking, deletes, credential paths, environment access, unusual `#foreign_library`, `#asm`, encoded blobs, binaries. A finding in a `#run` or build program is build-context and rated higher. Exit 1 means a `high` finding.
- **High risk is read, not skipped.** A repository with a `high` finding is pinned only after a person has read those lines.
- **Excluded** (`EXCLUDED_REPOSITORIES`): jaithon (its own language), this repository, repositories whose `.jai` files are incidental, vendored copies of official modules, forks of pinned upstreams, and mostly-binary repositories.
- **Review gate.** A new star repository's cases carry `"review": "pending"` until someone has read the code they reach. Pending cases are never selected for unconfined jobs and run only inside the sandbox. `test_star_corpus.py` enforces this.

## How to change it

- **Add a project:** add it to `REPOSITORIES`, `DEPENDENCIES` or `LIBRARIES`, run `fetch_upstreams.py`, commit `corpus/upstreams.json` (never `corpus/upstream/`), add entry points to `upstream-cases.json`, and a smoke check if it can run. Prefer projects that check themselves.
- **Add a star repository:** append to `STAR_REPOSITORIES`, fetch, run `scan_upstream.py`, read any `high` finding, then add `"review": "pending"` cases.
- **Find candidates:** `gh search repos --language=Jai --stars=">=20" --updated=">=<date>" --limit 100`, or `gh search code` with `extension:jai size:<bucket>` queries for repositories GitHub labels differently.
- **Try it:** the entry point is usually `first.jai` or `build.jai`. Try `jaic -check`, then build, then run. Classify: passes; jaic or stdlib bug (reduce to a test under `tests/compiler/regressions/` and fix); needs native libraries or another platform; broken upstream.
- Verify on every platform before trusting a smoke check: `-os` flags only type-check. Push a `tmp/...` branch, run CI, delete it.

Needs Python 3.11+ and network access to GitHub; `gh` for searching; clang for cases that compile vendored C. Native libraries come from [native libraries](native-libs.md) (`JAIC_NATIVE_LIBS`).
