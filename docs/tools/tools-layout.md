# tools/ layout

`tools/` holds the repo's build, check, generation and CI scripts (Python and Node, standard library only). One folder per subsystem; the files at the top (`repo_output.py`, `benchgen.py`, `jaistats.py`, ...) are shared helpers every folder imports by bare name, and newer shared helpers live in `toolkit/`.

## Finding a script

[`tools/README.md`](../../tools/README.md) lists every Python entry point with its one-line purpose and where it is called from. It is generated, so after adding, renaming or deleting a script, or changing which workflow runs it, run `PYTHONPATH=tools python3 tools/generators/generate_tools_index.py`. A script's purpose is the first line of its module docstring, so write one.

There is no single front-door script: [CI job selection](ci-job-selection.md) reads script paths out of the workflows' `run:` lines. Keep calling scripts by path.

## Folders

| Folder | What lives there |
|---|---|
| `bench/` | Compile, lsp, lint and fmt benchmark harness ([benchmarks](benchmarks.md)) |
| `bindings/` | SDL3 and WebGPU binding generators and tests |
| `browser/` | Wasm engine build, playground policy, browser release packaging |
| `ci/` | Change detection (`ci_changes.py`), source and dependency-age checks, runner install scripts |
| `corpus/` | Upstream smoke runs, conformance corpus generators |
| `dev/` | The workflow behind the Justfile (`dev.py`), format checks, staged-file lists, `linux_x64.sh` ([container](linux-container.md)) |
| `developer_checks/` | Checks of the Bindings_Generator tooling; `check_developer_tools.py` runs them as one unittest module. A new `check_developer_*.py` needs a `CHECKS` entry. Target triples come from `toolkit.platforms` (re-exported as `dev_common.TRIPLES`); `dev_common.WINDOWS_ARCHES` and `LINUX_ARCHES` are the per-architecture loop lists |
| `differential/` | `jaic_diff.py`, `jaic_reduce.py`, `jaic_sweep.py`, `jaigen.py` ([testing](testing.md)) |
| `fuzz/` | The stdlib fuzzer ([stdlib fuzzing](stdlib-fuzzing.md)) |
| `generators/` | Generators of checked-in files |
| `lsp/` | Language-server oracle checks ([oracle checks](lsp-oracle.md)) |
| `native/` | Native dependency stages: `build_native_libs.py`, `ci_native_deps.sh` |
| `native_bindings/` | The native binding generator and its verifiers |
| `numerics/` | The numeric oracle for `stdlib/Math` tests |
| `portable/` | Portable stdlib fixtures and checks |
| `stdlib/` | The stdlib test engine, suite runners, conformance runners |
| `toolkit/` | Shared helpers (below) |
| `window_render/` | Window/render suite |

## Shared helpers (`toolkit/`)

Import them folder-qualified: `from toolkit.hashing import sha256_file`.

| Module | What it gives |
|---|---|
| `toolkit.repo` | `ROOT` (never recompute it from `__file__`), `git_files(*patterns)` (the index's paths), `cargo_target_dir()`, `cargo_binary()`, `jaifmt_path()`, `stdlib_env()` |
| `toolkit.host` | `host_os()`, `host_cpu()`, `exe_name()`, `brew_prefix()` |
| `toolkit.platforms` | `PLATFORMS`, `TRIPLES`, `WINDOWS_TRIPLES`, `MINGW_PREFIXES`: the `<os>-<cpu>` names and their compiler targets; `toolkit/test_platforms.py` fails when they disagree with `crates/jaic/src/target.rs` (OS and CPU names, Windows and Linux triples) |
| `toolkit.hashing` | `sha256_file`, `sha256_bytes` |
| `toolkit.download` | `fetch(url, ...)` with retries (the one download path; don't call `urllib` directly); `pinned(url, sha256, name)` caches under `~/.cache/jaic/` and exits on a digest mismatch |
| `toolkit.process` | `run(command, ...)` (checked, echoes the command); `run_expect(...)` raises unless stdout, status and stderr match |
| `toolkit.cli` | `add_jaic_argument(parser)`: the shared `--jaic` option |

Add a helper to `toolkit/` when a third script needs it, with a test in `toolkit/test_toolkit.py`. Folder-specific helpers stay in the folder. Scratch output goes through `repo_output.output_dir()`, never `ROOT / "target"`; `check_source_style.py` enforces this. Script file names are `snake_case`.

## Generated files

Every generator of a checked-in file takes `--check` (exit 1 if the file differs). Edit a generator, rerun it without `--check`, commit both. `PYTHONPATH=tools python3 tools/generators/check_generated.py` runs them all (`--offline` skips the ones that download input). It discovers generators by module-level literals:

```python
OUTPUTS = ["stdlib/internal/basic/powers-of-five.jai"]   # required: repo-relative files it writes
FORMATTED = True   # formats through the repository jaifmt
DOWNLOADS = True   # downloads pinned input, so --offline skips it
```

Any script under `tools/` that defines `OUTPUTS` is run, and a script in `tools/generators/` with a `__main__` guard must define a non-empty `OUTPUTS`. A new generator needs no edit to the checker.

`generate_reference_docs.py` writes `docs/reference/` from the tools themselves (their `--help`, the environment-variable registry and the lint rules). Edit the code that defines a variable, rule or flag, not the pages.

## Tests

`tools/` has no `__init__.py`, so plain `unittest discover` skips subfolders. Use `python3 tools/run_tests.py [FOLDER ...] [-p PATTERN]... [-v] [-j N] [--timeout S]`: one `discover` per folder, each in its own process, run in parallel. Folders run at the same time, so tests must not share scratch paths. A folder that needs prepared inputs has an entry in `FIXTURES`. A script that only runs a suite by hand must not be named `test_*.py`. The SDL tests in `bindings/` need `SDL_TEST_TMPDIR` and prepared sources: `run_tests.py` prepares them, and run by hand without them they skip with the command to prepare them. When `CI` is set a missing input is an error instead, so CI cannot skip them silently. Node tests (`tools/browser/test_*.mjs`) run with `node --test`.

## Adding a tool

1. Put it in its subsystem's folder; name tests `test_<thing>.py` next to it.
2. Import across folders qualified (`from toolkit.download import fetch`); the top-level helpers stay bare. A module other folders import is imported qualified by its siblings too, so there is one copy in `sys.modules`. No `sys.path` code.
3. Find the repo root with `Path(__file__).resolve().parents[2]` for `tools/<folder>/x.py`.
4. Reference it by full path in workflows, the Justfile, docs and `ci_changes.py` filters.

## Module path

Scripts import folder-qualified, which needs `tools/` on `PYTHONPATH`. The `justfile`, every workflow (`env:`), both hooks and `run_tests.py` set it. By hand:

```sh
PYTHONPATH=tools python3 tools/<folder>/x.py
```

Without it the first import fails with `No module named 'toolkit'`.

## Configuration

`JACKET_OUTPUT_DIR` relocates `target/`. Python 3.11+ and Node 24 are the only requirements beyond each subsystem's toolchain. `toolkit.repo.stdlib_env()` keeps `JAIC_PRELUDE` from the caller ([prelude selection](../compiler/prelude-selection.md)).
