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

## Narrowing the stdlib tests

The `stdlib-runtime` legs run `jacket test` over the whole standard library (`tools/ci/stdlib_jacket_test.sh`). When a pull request or a push to a branch other than `main` or `staging` changes only stdlib module sources (`stdlib/**.jai`) and golden outputs (`tests/**.stdout|stderr|exit`), at most 100 files, `ci_changes.py` writes them as the JSON `stdlib_changed` output. The leg passes it as `JACKET_TEST_CHANGED`, and `tools/ci/select_tests.py` asks `jacket test --list --changed FILE...` once which tests can be affected: the dependency graph is what `jaic -deps` says each test program read, so no module map is kept in CI. The leg then runs every platform mode (native, `--interp`, `-O2`, wasm) with those names as `--exact` filters; `--partition` splits the selection, so the legs stay balanced. If nothing is affected the step ends early.

Everything else runs every test: any file outside those patterns (compiler, runtime, tools, CI config, a stdlib `.c`/`.h`/data file), pushes to `main`/`staging`, scheduled and manual runs, an undiffable range, and any failure of `jacket` or the script (exit 3). To force a full run on a branch, include a non-stdlib file in the change or run the workflow by hand. `Basic` and other widely imported modules select nearly everything on their own. The Windows and sanitizer `jacket test` steps and the `sdl3` jobs are not narrowed this way (`sdl3` stays own-paths-only).

## How to change it

- Gate another job: add its id to `GATED` in `job_inputs.py` and to an area in `ci_changes.py`; `ci.yml` gates it with `contains(fromJSON(needs.changes.outputs.jobs), 'ID')`.
- A path a step reaches only at run time: add it to `EXTRA`.
- A new `cargo -p` crate in a gated job: add it to `CRATES` (the drift test says which).
- Inspect a job's inputs: `python3 tools/ci/job_inputs.py --print [JOB ...]`.
- Which diffs narrow the stdlib tests: `NARROW_STDLIB`, `FULL_REFS` and `NARROW_STDLIB_LIMIT` in `ci_changes.py`. How a test depends on a file is `crates/jacket/src/test/deps.rs`.
- Tests: `test_job_inputs.py`, `test_ci_changes.py` and `test_select_tests.py`.

## Configuration

`ci_changes.py` reads the GitHub event variables (`GITHUB_EVENT_NAME`, `EVENT_BEFORE`, `SHA`, the pull request base). `JAIC_DEPS_JAIC` names a `jaic` for the stdlib closure. `GITHUB_REF` decides the branch rule above; `JACKET_TEST_CHANGED` is read by `stdlib_jacket_test.sh`.
