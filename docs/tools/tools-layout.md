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
| `dev/` | Developer workflow behind the Justfile (`dev.py`), Rust format checks, staged-file lists, PGO and packaging helpers, rule-coverage check, `linux_x64.sh` ([container](linux-container.md)). |
| `developer_checks/` | The `check_developer_*` / `build_developer_*` checks of the Bindings_Generator developer tooling. `check_developer_tools.py` is one unittest module over them: `ManifestChecks` runs each check module's `main()` (`CHECKS` lists them, with the `platform.system()` hosts of the ones that need a particular OS; elsewhere the test is skipped, never passed), `DrivenCases` runs each driven case interpreted and native at O0 and O2, and `Inline` holds the format/lint step, the Windows target checks and the rejections. A new `check_developer_*.py` needs a `CHECKS` entry (`test_developer_tools.py` fails otherwise). Run all of it, one test (`ManifestChecks.test_check_developer_vtables`) or a subset (`-k dwarf`) like any unittest module; a test that reads another check's output under `target/` names it in `NEEDS` and the module runs it first, once per process. `JACKET_TOOLING_PART=K/N` keeps every N-th test from the K-th, which is how CI splits the suite into legs (`check_stdlib_suites.py --tooling-part`); each part rebuilds the shared fixtures it needs. The module is not named `test_*.py` because it needs the source-built tools, so `run_tests.py` leaves it to the `tooling` stdlib suite. |
| `differential/` | `jaic_diff.py` (backend differential testing), `jaic_reduce.py`, `jaic_sweep.py`, the random program generator `jaigen.py`. |
| `fuzz/` | The stdlib fuzzer: driver, discovery metaprogram, per-module config and property hooks ([stdlib fuzzing](stdlib-fuzzing.md)). |
| `generators/` | Generators of checked-in files (below). The generated file's header names its generator. |
| `lsp/` | The language-server oracle checks driver `oracle.py` and its known findings ([oracle checks](lsp-oracle.md)). |
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
- **Stdlib runners** (`stdlib/`, `portable/`): `stdlib_runtime.py` runs the compiler regressions (the stdlib modules' tests run under `jacket test`); `check_portable_stdlib.py` builds fixtures and runs what a test cannot; `check_stdlib_suites.py` is the orchestrator that invokes every suite. Orchestrator, suite and engine are separate layers, so they are not merged.

## Shared helpers (`toolkit/`)

`toolkit/` is the one place for helpers that were copied between scripts. Import them folder-qualified (`from toolkit.hashing import sha256_file`); `tools/` is on the module path ([Module path](#module-path)).

| Module | What it gives |
|---|---|
| `toolkit.repo` | `ROOT` (the checkout; never recompute it from `__file__`), `cargo_target_dir(resolve=False)` (`$CARGO_TARGET_DIR`, else cargo's configured target directory, else `target/`), `cargo_binary(name, profile="debug", triple=None)` (a cargo-built executable, `.exe` on Windows), `jaifmt_path()`, `stdlib_env()` (the environment with `JAIC_STDLIB` set). Scratch output goes through `repo_output.output_dir()`, never `ROOT / "target"`. |
| `toolkit.host` | `host_os()` (`macos`/`linux`/`windows`), `host_cpu()` (`arm64`/`x64`), `exe_name(name)`, `brew_prefix()` (Homebrew's prefix on macOS, else `None`; use it instead of `/opt/homebrew`). |
| `toolkit.hashing` | `sha256_file(path)` (chunked), `sha256_bytes(data)`. |
| `toolkit.download` | `fetch(url, timeout=, attempts=, headers=, max_bytes=)`: the body of a URL, retrying rate limits, 5xx answers and dropped connections (the one download path; do not call `urllib` directly). `pinned(url, sha256, name)`: the file cached as `$XDG_CACHE_HOME/jaic/<name>` (default `~/.cache/jaic/`), downloaded first if missing and retried with backoff on 408, 429 and 5xx; exits when the digest differs. The generators that read upstream data (`gen_io_uring.py`, `gen_regex_unicode.py`) use it. |
| `toolkit.process` | `run(command, check=True, capture=False, echo=True, **subprocess_kwargs)`: a checked `subprocess.run` that `str()`s the items and echoes `+ command`. `run_expect(command, label=..., expected=..., code=..., stderr=..., env=..., timeout=...)`: run, then raise `RuntimeError` unless stdout, status and stderr match. |
| `toolkit.cli` | `add_jaic_argument(parser, required=False, profile="debug", type=Path)`: the shared `--jaic` option, defaulting to the target directory's build; `default_jaic()`. |

`developer_checks/dev_common.py` is that folder's shared entry point (its modules import each other as `developer_checks.<module>`). It re-exports `ROOT`, `ENV` (the compiler environment), `JAIC`, `sha256_file` and `run_checked` (`run_expect` with the checks' defaults). The `run` function each check still defines only fixes its failure label and defaults.

Script file names are `snake_case` (they are imported as modules). `tools/ci/check_source_style.py` also fails a script that reads `CARGO_TARGET_DIR` or builds `ROOT / "target"` itself; the few files that must (listed with reasons in `BUILD_PATH_ALLOWED`) are the exceptions. Add a helper to `toolkit/` when a third script needs it, with a test in `toolkit/test_toolkit.py`. A helper that is specific to one folder (`bench/common.py`, `native_bindings/verify.py`) stays in that folder.

## Generated files

Every generator of a checked-in file takes `--check` (exit 1 if the file differs from what it would write) and parses its arguments, so `--help` writes nothing. Generators of Jai format their output through the repository formatter (`--formatter`, default `jaifmt_path()` from `toolkit.repo`, i.e. `<cargo target dir>/jaifmt`, built by `just fmt --check --lang jai` or `jaic jaifmt/main.jai -O2 -exe <cargo target dir>/jaifmt`) via `jai_codegen.emit_generated`; edit a generator, rerun it without `--check`, commit both.

`PYTHONPATH=tools python3 tools/generators/check_generated.py` runs them all with `--check` (CI step "Generated sources check"; `--offline` skips the generators that download their pinned input). It discovers them: a generator declares module-level literals, which the checker reads from the source without importing it.

```python
OUTPUTS = ["stdlib/internal/basic/powers-of-five.jai"]   # required: the repo-relative files it writes
FORMATTED = True   # it formats through the repository jaifmt, so the checker passes --formatter
DOWNLOADS = True   # it downloads its pinned input, so --offline skips it
```

Any script under `tools/` that defines `OUTPUTS` is run (that is how `tools/corpus/gen_*_corpus.py` and `tools/developer_checks/generate_developer_binding_model.py` take part), and a script in `tools/generators/` with a `__main__` guard (not `check_generated.py` or a test) is a generator that must define a non-empty `OUTPUTS`: the check fails for one that declares none, for an output that does not exist and for an output two generators claim. A new generator needs no edit to the checker; declare its outputs and flags. The native bindings (`native_bindings/generate.py`) and SDL3 bindings (`bindings/sdl3_gen.py`) have their own CI steps. Not checked in CI: `corpus/gen_regex_corpus.py` (needs the Rust oracle), the rpmalloc and AppKit generators in `native_bindings/` (need vendored SDK inputs) and `bindings/webgpu_gen.py` (`--check-header` needs a `webgpu.h`).

`generate_version_files.py` writes `jaifmt/version.jai` and `editors/nvim/lua/jai/version.lua` from the `Cargo.toml` version; `tools/bump_version.py` runs it.

`generate_socket_abi.py` reads `docs/specs/stdlib/modules/socket.md` with the spec parser of `native_bindings/spec.py`, so a parser change can move the Socket ABI files too.

## How tests are discovered

`tools/` has no `__init__.py`, so `unittest discover -s tools` skips subfolders. `python3 tools/run_tests.py [FOLDER ...] [-p PATTERN]... [-v] [-j N] [--timeout S]` is the one entry point: it runs one `discover` per folder (at any depth, `native_bindings/`, `numerics/` and `bench/` included) and pattern that holds a matching file, each in its own process with `tools/` on `PYTHONPATH` (and unittest putting the folder under test on `sys.path`, so a test imports its sibling modules by bare name). CI, `just test` and the workflows that run a few test files all go through it; a workflow passes `-p` once per file instead of calling it repeatedly.

- **Parallel and bounded.** Folders run `-j` at a time (default: the CPU count, at most 4) and a folder's process group is killed after `--timeout` seconds (default 900; `FOLDER_TIMEOUTS` in the script gives the slow folders more). A folder's output is printed whole when it finishes, always for a failure and otherwise with `-v`; a passing folder prints one line. Tests must not share scratch paths across folders (use `tempfile` or `repo_output.output_dir()` with a name of their own), since folders run at the same time.
- **Fixtures.** Inputs a folder's tests need are prepared by an entry of `FIXTURES` (folder name, the file patterns that need it, a `prepare()` that returns environment variables or raises `Skip`). It runs once per run, only when a selected file uses it, and a `Skip` skips the folder with its message instead of failing the run. `bindings/` has the SDL one: the tests need hash-pinned header inputs, CI prepares them with `bindings/prepare_sdl_python_tests.py` (which sets `SDL_TEST_TMPDIR`), and the fixture does it itself when the variable is unset (a download, cached under `target/sdl-python-cache`). `generators/` needs a built `jaifmt_path()`; its fixture builds it (as `dev.py fmt` does, `ensure_jaifmt`) when it is missing.

An import that only exists so tests can patch through the module (`urllib.request` in the native builders) carries `# noqa: F401`; `ruff check tools .github` is clean under ruff's default rules (`ruff.toml`; CI's rule coverage job runs it). A script that only runs a suite by hand must not be named `test_*.py` (`bindings/run_sdl3.py`, `bindings/run_webgpu_native.py`), or discovery imports it. The Node tests (`tools/browser/test_*.mjs`) run with `node --test tools/browser/test_*.mjs` in the wasm CI job and in `browser-release.yml`.

## How to add a tool

1. Put it in the folder of its subsystem; name tests `test_<thing>.py` next to what they test.
2. Imports across folders are qualified by folder: `from toolkit.download import fetch`, `from stdlib import stdlib_runtime`. The helpers at the top of `tools/` stay bare (`import repo_output`). A script carries no `sys.path` code of its own: `tools/` is on `PYTHONPATH` ([Module path](#module-path)), and `PYTHONPATH=tools python3 tools/<folder>/x.py` adds `tools/<folder>/` itself. Imports between modules of the same folder may stay bare, except for a module that other folders import (`stdlib/stdlib_runtime.py`, `native/build_native_libs.py`, `bench/sandbox.py`, ...): it and everything that touches it import its siblings qualified too (`from bench import common`), so there is only one copy of it in `sys.modules`. That includes `unittest.mock.patch("stdlib.stdlib_runtime.run_command")` target strings. `native_bindings/` keeps generic module names (`config`, `spec`), so it is never put on `sys.path` as a folder; import it as `native_bindings.config`.
3. A script at `tools/<folder>/x.py` finds the repo root with `Path(__file__).resolve().parents[2]`.
4. Reference it by its full path (`tools/<folder>/x.py`) in workflows, the Justfile, docs and `tools/ci/ci_changes.py` path filters.

## Module path

Scripts import each other folder-qualified (`from toolkit.repo import ROOT`), which needs `tools/` on the module search path. Every entry point sets `PYTHONPATH` to the absolute path of `tools/` once instead of each script patching `sys.path`:

- the `justfile` exports it (so `just <recipe>` and everything a recipe spawns have it),
- each workflow in `.github/workflows/` sets it in its top-level `env:` to `${{ github.workspace }}/tools`,
- `.githooks/pre-commit` and `.githooks/pre-push` set it for the script they exec,
- `run_tests.py` sets it for the test processes it starts.

A script run by hand needs it too: `PYTHONPATH=tools python3 tools/<folder>/x.py` from the repository root, or `just` where a recipe exists. Without it the first folder-qualified import fails with `No module named 'toolkit'`.

## Configuration and dependencies

`JACKET_OUTPUT_DIR` relocates `target/` (see `repo_output.py`). Python 3.11+ and Node 24 are the only requirements beyond the toolchains each subsystem drives.

`toolkit.repo.stdlib_env()` preserves `JAIC_PRELUDE` from the caller while setting the module folder. [Prelude selection](../compiler/prelude-selection.md) describes `PRELUDE_ENTRIES`, custom root files and how to extend detection.
