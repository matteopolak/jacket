# tools/ layout

## What it is

`tools/` holds the repo's build, check, generation and CI scripts (Python and Node, standard library only). Scripts are grouped into one folder per subsystem; the files left at the top (`repo_output.py`, `benchgen.py`, `jaistats.py`, ...) are shared helpers every folder imports by bare name; the newer shared helpers live in `toolkit/`.

## Folders

| Folder | What lives there |
|---|---|
| `bench/` | The compile, lsp, lint and fmt benchmark harness ([benchmarks](benchmarks.md)). Needs `benchgen.py`, `jaistats.py`, `repo_output.py`, `wasi_run.mjs`, `fetch_upstreams.py`, `upstream-cases.json` to stay at the top. |
| `bindings/` | SDL3 and WebGPU binding generators, their native fetch/build steps and tests. |
| `browser/` | The wasm engine build, playground policy, browser release packaging and their checks (`check_jaifmt_wasm.mjs engine\|wasi`). |
| `ci/` | CI plumbing: change detection (`ci_changes.py`), source and dependency-age checks, runner install scripts, `windows-llvm/`, pre-commit hook test. |
| `corpus/` | Upstream project smoke runs (`upstream_smoke.py`), conformance corpus generators (`gen_*_corpus.py`), `regex_oracle/`, `upstream-drivers/`. |
| `dev/` | Developer workflow behind the Justfile (`dev.py`), Rust format checks, staged-file lists, PGO and packaging helpers, rule-coverage check. |
| `developer_checks/` | The `check_developer_*` / `build_developer_*` checks of the Bindings_Generator developer tooling; `check_developer_tools.py` runs them all, in the order of `developer_checks.json` (one entry per check module, `hosts` for those that need a particular OS; `--list`, `--only MODULE`). A new `check_developer_*.py` needs an entry there (a test fails otherwise). |
| `differential/` | `jaic_diff.py` (backend differential testing), `jaic_reduce.py`, `jaic_sweep.py`, the random program generator `jaigen.py`. |
| `fuzz/` | The stdlib fuzzer: driver, discovery metaprogram, per-module config and property hooks ([stdlib fuzzing](stdlib-fuzzing.md)). |
| `generators/` | Generators of checked-in files (below). The generated file's header names its generator. |
| `native/` | Native dependency stages (below), the Windows cross/provider builders, `mbedtls/` and `sdl3/` sources. |
| `native_bindings/` | The native binding generator and its verifiers (own test discovery and module names). |
| `numerics/` | Numerics suite runners and capability tables. |
| `portable/` | Portable stdlib fixtures, crash/clipboard/watcher/mail checks, `portable_data/`. |
| `stdlib/` | The stdlib test engine and the suite runners layered on it (below), platform I/O and Http conformance, extension runners. |
| `toolkit/` | Shared helpers (below): repo root, file hashing, pinned downloads, checked subprocess runs. |
| `window_render/` | Window/render suite and its WGL lint. |

Top level: `repo_output.py|mjs` (where output goes, `host_platform()`), `run_tests.py`, `benchgen.py`, `jaistats.py`, `corpus-shape.json`, `regex_bench.py`, `profile_report.py`, `fetch_upstreams.py`, `upstream-cases.json`, `wasi_run.mjs`, `test_benchgen.py`.

## Layered pieces worth knowing

- **Native dependency stages** (`native/`): `build_native_libs.py` builds the pinned third-party C libraries from `native-libs.json`; `build_ci_native_bindings.py` builds the rpmalloc/SDK probe providers for the bindings checks; `upstream_native.py` prepares the helpers the upstream smoke run needs; `ci_native_deps.sh` is the CI entry that calls them in order. They overlap only in host detection, now shared as `repo_output.host_platform()`.
- **Stdlib runners** (`stdlib/`, `portable/`): `stdlib_runtime.py` runs the catalogued `stdlib/**/tests` cases; `check_portable_stdlib.py` builds fixtures and runs what a catalogued case cannot; `check_stdlib_suites.py` is the orchestrator that invokes every suite. Orchestrator, suite and engine are separate layers, so they are not merged.

## Shared helpers (`toolkit/`)

`toolkit/` is the one place for helpers that were copied between scripts. Import them folder-qualified (`from toolkit.hashing import sha256_file`), after the `sys.path` lines of "How to add a tool".

| Module | What it gives |
|---|---|
| `toolkit.repo` | `ROOT` (the checkout; never recompute it from `__file__`), `cargo_target_dir()` (`$CARGO_TARGET_DIR` or `target/`), `stdlib_env()` (the environment with `JAIC_STDLIB` set). |
| `toolkit.hashing` | `sha256_file(path)` (chunked), `sha256_bytes(data)`. |
| `toolkit.download` | `fetch(url, timeout=, attempts=, headers=, max_bytes=)`: the body of a URL, retrying rate limits, 5xx answers and dropped connections (the one download path; do not call `urllib` directly). `pinned(url, sha256, name)`: the file cached as `$XDG_CACHE_HOME/jaic/<name>` (default `~/.cache/jaic/`), downloaded first if missing and retried with backoff on 408, 429 and 5xx; exits when the digest differs. The generators that read upstream data (`gen_io_uring.py`, `gen_regex_unicode.py`) use it. |
| `toolkit.process` | `run(command, check=True, capture=False, echo=True, **subprocess_kwargs)`: a checked `subprocess.run` that `str()`s the items and echoes `+ command`. `run_expect(command, label=..., expected=..., code=..., stderr=..., env=..., timeout=...)`: run, then raise `RuntimeError` unless stdout, status and stderr match. |
| `toolkit.cli` | `add_jaic_argument(parser, required=False, profile="debug", type=Path)`: the shared `--jaic` option, defaulting to the target directory's build; `default_jaic()`. |

`developer_checks/dev_common.py` is that folder's single entry point: importing it puts `tools/` on `sys.path`, so the checks there carry no path lines of their own. It re-exports `ROOT`, `ENV` (the compiler environment), `JAIC`, `sha256_file` and `run_checked` (`run_expect` with the checks' defaults). The `run` function each check still defines only fixes its failure label and defaults.

Script file names are `snake_case` (they are imported as modules). Add a helper to `toolkit/` when a third script needs it, with a test in `toolkit/test_toolkit.py`. A helper that is specific to one folder (`bench/common.py`, `native_bindings/verify.py`) stays in that folder.

## Generated files

Every generator of a checked-in file takes `--check` (exit 1 if the file differs from what it would write) and parses its arguments, so `--help` writes nothing. Generators of Jai format their output through the repository formatter (`--formatter`, default `target/jaifmt`, built by `just fmt --check --lang jai` or `jaic jaifmt/main.jai -O2 -exe target/jaifmt`) via `jai_codegen.emit_generated`; edit a generator, rerun it without `--check`, commit both.

`python3 tools/generators/check_generated.py` runs them all with `--check` (CI step "Generated sources check"; `--offline` skips `gen_io_uring.py` and `gen_regex_unicode.py`, which download their pinned input). It covers `tools/generators/*`, `tools/corpus/gen_json_corpus.py`, `gen_http_corpus.py` and `tools/developer_checks/generate_developer_binding_model.py`; add a new generator to its lists. The native bindings (`native_bindings/generate.py`) and SDL3 bindings (`bindings/sdl3_gen.py`) have their own CI steps. Not checked in CI: `corpus/gen_regex_corpus.py` (needs the Rust oracle), the rpmalloc and AppKit generators in `native_bindings/` (need vendored SDK inputs) and `bindings/webgpu_gen.py` (`--check-header` needs a `webgpu.h`).

`generate_socket_abi.py` reads `docs/specs/stdlib/modules/socket.md` with the spec parser of `native_bindings/spec.py`, so a parser change can move the Socket ABI files too.

## How tests are discovered

`tools/` has no `__init__.py`, so `unittest discover -s tools` skips subfolders. `python3 tools/run_tests.py [-p PATTERN] [-v]` runs one `discover` per folder at any depth that holds a matching file (including `native_bindings/`, `numerics/` and `bench/`) with `tools/` on `PYTHONPATH` (and unittest putting the folder under test on `sys.path`, so a test imports its sibling modules by bare name). CI, `just test` and the workflows that run a single test file all go through it. The SDL tests in `bindings/` need hash-pinned header inputs; CI prepares them with `bindings/prepare_sdl_python_tests.py` (which sets `SDL_TEST_TMPDIR`), and `run_tests.py` does it itself when the variable is unset (a download, cached under `target/sdl-python-cache`). `generators/` tests need a built `target/jaifmt`; `run_tests.py` builds it (as `dev.py fmt` does, `ensure_jaifmt`) when it is missing and skips that folder with a message if it cannot. An import that only exists so tests can patch through the module (`urllib.request` in the native builders) carries `# noqa: F401`; `ruff check --select F401 .` is clean. A script that only runs a suite by hand must not be named `test_*.py` (`bindings/run_sdl3.py`, `bindings/run_webgpu_native.py`), or discovery imports it. The Node tests (`tools/browser/test_*.mjs`) run with `node --test tools/browser/test_*.mjs` in the wasm CI job and in `browser-release.yml`.

## How to add a tool

1. Put it in the folder of its subsystem; name tests `test_<thing>.py` next to what they test.
2. Imports across folders are qualified by folder: `from toolkit.download import fetch`, `from stdlib import stdlib_runtime`. The helpers at the top of `tools/` stay bare (`import repo_output`). A script that imports either way starts with the same three lines, before any other project import, so `tools/` is on `sys.path` when it runs directly (`python3 tools/<folder>/x.py` only puts `tools/<folder>/` there; the test runner and CI add nothing):

   ```python
   import sys as _sys
   from pathlib import Path as _Path

   # tools/, for folder-qualified imports
   _sys.path.insert(0, str(_Path(__file__).resolve().parents[1]))
   ```

   Use `parents[2]` in a sub-sub-folder (`tools/portable/portable_data/`). Imports between modules of the same folder may stay bare, except for a module that other folders import (`stdlib/stdlib_runtime.py`, `native/build_native_libs.py`, ...): it and everything that touches it import its siblings qualified too, so there is only one copy of it in `sys.modules`. That includes `unittest.mock.patch("stdlib.stdlib_runtime.run_command")` target strings. `native_bindings/` keeps generic module names (`config`, `spec`), so it is never put on `sys.path` as a folder; import it as `native_bindings.config`.
3. A script at `tools/<folder>/x.py` finds the repo root with `Path(__file__).resolve().parents[2]`.
4. Reference it by its full path (`tools/<folder>/x.py`) in workflows, the Justfile, docs and `tools/ci/ci_changes.py` path filters.

## Import check

`python3 tools/ci/check_tool_imports.py` (the `rule coverage` job in `ci.yml`) loads every `tools/**/*.py` in a fresh interpreter twice, the way CI runs a script (only its own folder on `sys.path`) and the way `run_tests.py` imports it, and fails on any project module that does not resolve. It runs the module body only up to its last top-level import, checks imports nested in functions statically, and skips optional third-party modules. Run it after moving or renaming anything under `tools/`.

## Configuration and dependencies

`JACKET_OUTPUT_DIR` relocates `target/` (see `repo_output.py`). Python 3.11+ and Node 24 are the only requirements beyond the toolchains each subsystem drives.
