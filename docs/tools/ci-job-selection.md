# CI job selection

`tools/ci/ci_changes.py` decides which path-gated jobs of `ci.yml` run for a push or pull request. It keeps no path list per job: `tools/ci/job_inputs.py` computes each job's input files from the job itself, so a new `#import`, a new crate dependency or a new script called from a step selects the right jobs with no edit under `tools/ci/`.

## How it works

For a gated job, `job_inputs.inputs()` collects:

- every path in the job's `run:` lines and `with:` values, and in the composite actions it uses;
- the closure of those scripts (Python imports and path strings; paths named in shell, PowerShell and Nix files);
- for each `cargo build|test -p NAME`, the crate and its path dependencies, plus `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` (dev-dependencies only for jobs that run tests);
- for each `.jai` file a step compiles, the stdlib modules it reads (from `jaic -deps` if a binary exists, else a static scan).

`ci_changes.py` matches the changed files against each area's patterns and writes the `jobs=` and `<area>=` outputs `ci.yml` reads. If the computation fails, everything runs.

Hand-written policy in `ci_changes.py`:

| Table | Meaning |
| --- | --- |
| `NARROW` | The macOS sanitizer leg runs on pushes only for codegen, driver, sweep and native tooling paths. |
| `OWN_PATHS_ONLY` | `sdl3` does not run for a workflow edit. |
| `PROSE`, `READ_BY_TESTS` | Docs-only changes skip the code jobs, except docs a test reads. |

## How to change it

- Gate another job: add its id to `GATED` in `job_inputs.py` and to an area in `ci_changes.py`; `ci.yml` gates it with `contains(fromJSON(needs.changes.outputs.jobs), 'ID')`.
- A path a step reaches only at run time: add it to `EXTRA`.
- A new `cargo -p` crate in a gated job: add it to `CRATES` (the drift test says which).
- Inspect a job's inputs: `python3 tools/ci/job_inputs.py --print [JOB ...]`.
- Tests: `test_job_inputs.py` and `test_ci_changes.py`.

## Configuration

`ci_changes.py` reads the GitHub event variables (`GITHUB_EVENT_NAME`, `EVENT_BEFORE`, `SHA`, the pull request base). `JAIC_DEPS_JAIC` names a `jaic` for the stdlib closure.
