# CI job selection

## What it is

`tools/ci/ci_changes.py` decides which path-gated jobs of `ci.yml` run for a push or pull request. It does not keep a list of paths per job: `tools/ci/job_inputs.py` computes each job's input files from the job itself, so a new `#import` in the standard library, a new path dependency between crates or a new script called from a step selects the right jobs with no edit under `tools/ci/`.

## How it works

For a gated job, `job_inputs.inputs()` collects:

- every path named in the job's `run:` lines and `with:` values, and, recursively, in the composite actions it uses (`.github/actions/*`);
- the closure of those scripts: for Python, the modules they import and the path strings they build; for shell, PowerShell and Nix files, the paths they name. `tools/run_tests.py` is treated as opaque (only the suites a step names count);
- for each `cargo build|test -p NAME` in a step, the crate and its path dependencies from the `Cargo.toml` files, plus `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml`. Dev-dependencies count only for jobs that run `cargo test`, nextest, `llvm-cov` or Nix;
- for each `.jai` file a step compiles, the stdlib modules it reads, from `jaic -deps` when a `jaic` binary exists and otherwise from a static scan of `#import`/`#load` plus the prelude. Only the modules (directories under `stdlib/`) are expanded; other directories stay as globs.

`ci_changes.py` then matches the changed files against each area's patterns (`*` stays within a directory, `**` crosses them) and writes the same `jobs=` and `<area>=` outputs as before, so `ci.yml` is unchanged. If the computation fails, every area runs.

What stays hand-written in `ci_changes.py` is policy, not inputs:

| Table | Meaning |
| --- | --- |
| `NARROW` | The macOS sanitizer leg runs on pushes only for the code generation, driver, sweep and native tooling paths. |
| `OWN_PATHS_ONLY` | `sdl3` does not run for a workflow edit, only for its own inputs. |
| `PROSE`, `READ_BY_TESTS` | Documentation-only changes skip the code jobs, except the docs a test reads. |

In `job_inputs.py`, `GATED` lists the jobs that are computed, `CRATES` names the crates behind `cargo -p` lines, and `EXTRA` holds paths a step builds at run time (for example a directory a script walks). A drift check fails the test suite when a run line names a crate missing from `CRATES`.

Inspect a job's inputs with `python3 tools/ci/job_inputs.py --print [JOB ...]`.

## How to change it

- Gate another job: add its id to `GATED` and to an area in `ci_changes.py`; `ci.yml` gates it with `contains(fromJSON(needs.changes.outputs.jobs), 'ID')`.
- A path a step reaches only at run time: add it to `EXTRA`.
- A new `cargo -p` crate in a gated job: add it to `CRATES` (the drift test says which).
- Tests: `tools/ci/test_job_inputs.py` (computation) and `tools/ci/test_ci_changes.py` (selection, built from fake workflows). `tools/release.py` watches the same outputs.

Gotcha: a dev-dependency edge (for example `jaic-cli` to `test-support` to `jai-language-server`) widens only the test jobs; build-only jobs ignore it.

## Configuration

`ci_changes.py` reads the GitHub event variables (`GITHUB_EVENT_NAME`, `EVENT_BEFORE`, `SHA` and the pull request base) to find the diff. A `jaic` binary named by `JAIC_DEPS_JAIC` improves the stdlib closure; without one, the static scan is used.

## Dependencies

`ci.yml` and the composite actions, the `Cargo.toml` files (read with `tomllib`, not `cargo metadata`, so a bare checkout needs no toolchain), and optionally `jaic -deps` ([the command line](cli.md)).
