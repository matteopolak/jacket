# tools/ layout

## What it is

`tools/` holds the repo's build, check, generation and CI scripts (Python and Node, standard library only). Scripts are grouped into one folder per subsystem; the files left at the top are shared helpers and the ones the benchmark harness (`tools/bench/`) imports by bare name.

## Folders

| Folder | What lives there |
|---|---|
| `bench/` | The compile, lsp, lint and fmt benchmark harness ([benchmarks](benchmarks.md)). Needs `benchgen.py`, `jaistats.py`, `repo_output.py`, `wasi_run.mjs`, `fetch_upstreams.py`, `upstream-cases.json` to stay at the top. |
| `bindings/` | SDL3 and WebGPU binding generators, their native fetch/build steps and tests. |
| `browser/` | The wasm engine build, playground policy, browser release packaging and their checks (`check_jaifmt_wasm.mjs engine\|wasi`). |
| `ci/` | CI plumbing: change detection (`ci_changes.py`), source and dependency-age checks, runner install scripts, `windows-llvm/`, pre-commit hook test. |
| `corpus/` | Upstream project smoke runs (`upstream_smoke.py`), conformance corpus generators (`gen_*_corpus.py`), `regex_oracle/`, `upstream-drivers/`. |
| `dev/` | Developer workflow behind the Justfile (`dev.py`), Rust format checks, staged-file lists, PGO and packaging helpers, rule-coverage check. |
| `developer_checks/` | The `check_developer_*` / `build_developer_*` checks of the Bindings_Generator developer tooling; `check_developer_tools.py` runs them all. |
| `differential/` | `jaic-diff.py` (backend differential testing), `jaic-reduce.py`, `jaic-sweep.py`, the random program generator `jaigen.py`. |
| `generators/` | Generators of checked-in files (below). The generated file's header names its generator. |
| `native/` | Native dependency stages (below), the Windows cross/provider builders, `mbedtls/` and `sdl3/` sources. |
| `native_bindings/` | The native binding generator and its verifiers (own test discovery and module names). |
| `numerics/` | Numerics suite runners and capability tables. |
| `portable/` | Portable stdlib fixtures, crash/clipboard/watcher/mail checks, `portable_data/`. |
| `stdlib/` | The stdlib test engine and the suite runners layered on it (below), platform I/O and Http conformance, extension runners. |
| `window_render/` | Window/render suite and its WGL lint. |

Top level: `repo_output.py|mjs` (where output goes, `host_platform()`), `run_tests.py`, `benchgen.py`, `jaistats.py`, `corpus-shape.json`, `regex_bench.py`, `profile_report.py`, `fetch_upstreams.py`, `upstream-cases.json`, `wasi_run.mjs`, `test_benchgen.py`.

## Layered pieces worth knowing

- **Native dependency stages** (`native/`): `build_native_libs.py` builds the pinned third-party C libraries from `native-libs.json`; `build_ci_native_bindings.py` builds the rpmalloc/SDK probe providers for the bindings checks; `upstream_native.py` prepares the helpers the upstream smoke run needs; `ci_native_deps.sh` is the CI entry that calls them in order. They overlap only in host detection, now shared as `repo_output.host_platform()`.
- **Stdlib runners** (`stdlib/`, `portable/`): `stdlib_runtime.py` runs the catalogued `stdlib/**/tests` cases; `check_portable_stdlib.py` builds fixtures and runs what a catalogued case cannot; `check_stdlib_suites.py` is the orchestrator that invokes every suite. Orchestrator, suite and engine are separate layers, so they are not merged.

## Generated files

Every generator of a checked-in file takes `--check` (exit 1 if the file differs from what it would write) and parses its arguments, so `--help` writes nothing. Generators of Jai format their output through the repository formatter (`--formatter`, default `target/jaifmt`, built by `just fmt --check --lang jai` or `jaic jaifmt/main.jai -O2 -exe target/jaifmt`) via `jai_codegen.emit_generated`; edit a generator, rerun it without `--check`, commit both.

`python3 tools/generators/check_generated.py` runs them all with `--check` (CI step "Generated sources check"; `--offline` skips `gen_io_uring.py` and `gen_regex_unicode.py`, which download their pinned input). It covers `tools/generators/*`, `tools/corpus/gen_json_corpus.py`, `gen_http_corpus.py` and `tools/developer_checks/generate_developer_binding_model.py`; add a new generator to its lists. The native bindings (`native_bindings/generate.py`) and SDL3 bindings (`bindings/sdl3_gen.py`) have their own CI steps. Not checked in CI: `corpus/gen_regex_corpus.py` (needs the Rust oracle), the rpmalloc and AppKit generators in `native_bindings/` (need vendored SDK inputs) and `bindings/webgpu_gen.py` (`--check-header` needs a `webgpu.h`).

`generate_socket_abi.py` reads `docs/specs/stdlib/modules/socket.md` with the spec parser of `native_bindings/spec.py`, so a parser change can move the Socket ABI files too.

## How tests are discovered

`tools/` has no `__init__.py`, so `unittest discover -s tools` skips subfolders. `python3 tools/run_tests.py [-p PATTERN] [-v]` runs one `discover` per folder (including `native_bindings/` and `bench/`) with every folder on `PYTHONPATH`. CI, `just test` and the workflows that run a single test file all go through it. Three Node tests (`tools/browser/test_*.mjs`) run in the wasm CI job with `node --test`.

## How to add a tool

1. Put it in the folder of its subsystem; name tests `test_<thing>.py` next to what they test.
2. Modules are imported by bare name. A script that imports from another folder starts with the `_sys.path[:0] = [...]` block the existing ones use (see `tools/stdlib/http_conformance.py`).
3. A script at `tools/<folder>/x.py` finds the repo root with `Path(__file__).resolve().parents[2]`.
4. Reference it by its full path (`tools/<folder>/x.py`) in workflows, the Justfile, docs and `tools/ci/ci_changes.py` path filters.

## Configuration and dependencies

`JACKET_OUTPUT_DIR` relocates `target/` (see `repo_output.py`). Python 3.11+ and Node 24 are the only requirements beyond the toolchains each subsystem drives.
