# CI job selection

`tools/ci/ci_changes.py` decides which path-gated jobs of `ci.yml` run for a push or pull request. It keeps no path list per job: `tools/ci/job_inputs.py` computes each job's inputs from the job itself, so a new `#import`, crate dependency or script selects the right jobs with no edit under `tools/ci/`.

## How it works

For a gated job, `job_inputs.inputs()` collects:

- every path in the job's `run:` lines and `with:` values, and in the composite actions it uses;
- `.gitattributes` policies (root and nested), when a step uses `actions/checkout`, because checkout can change file bytes. Script literals and `env`/`with` keys named `uses` are not action references;
- the closure of those scripts (Python imports, path strings and exact sibling data/license/entry-source filenames; paths named in shell, PowerShell and Nix files);
- for each `cargo build|test -p NAME`, the crate and its path dependencies, plus `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` (dev-dependencies only for jobs that run tests);
- for each `.jai` file a step compiles, the stdlib modules it reads (from `jaic -deps` if a binary exists, else a static scan).

A Python literal naming an existing sibling file follows that file too (`HERE / "pin.json"`, a license, a Jai entry source). The graph uses indexed filenames: `git add -N` new inputs before inspecting a local graph.

`ci_changes.py` matches the changed files against each area's patterns and writes the `jobs=` and `<area>=` outputs `ci.yml` reads. If the computation fails, everything runs.

Hand-written policy in `ci_changes.py`:

| Table | Meaning |
| --- | --- |
| `NARROW` | The macOS sanitizer leg runs on pushes only for codegen, driver, sweep, native tooling and shared checkout-policy inputs. |
| `MANUAL_ONLY` | Focused acceptance jobs such as `images-ui-intel` run only when explicitly named in a manual `jobs=` request. |
| `OWN_PATHS_ONLY` | `sdl3` does not run for a workflow edit. |
| `PROSE`, `READ_BY_TESTS` | Docs-only changes skip the code jobs, except docs a test reads. Checkout policies are code inputs even under `docs/`. |

## Splitting work across jobs

**Compiler target checks.** Full-tier Intel macOS divides the stdlib module/target checks and the cross-OS doc checks between its `rust` and `checks` jobs (`targets_part=2/2` and `1/2`). Objective-C selectors run only in the first share. `matrix.shard` still selects the whole native Rust suite on each full-tier Mac Rust job; the target steps use `matrix.targets_part` when set, else `matrix.shard`. When changing the matrix, update both target-step environment values together and keep a complete, disjoint set of shares per platform. `test_ci_changes.py` checks the shares.

**Producer suites.** Full-tier Intel macOS runs `tooling` and `bindings` on its catalogue job; the other producer suites stay with the native/interpreted/O2 Jacket run. Tooling rebuilds native tools that bindings reads, so keep them together (`check_stdlib_suites.py` runs them in one resource group). The `suites` matrix field selects the drivers. Keep each producer suite exactly once, keep the independent catalogue, Jacket and WebGPU coverage, and update `test_ci_changes.py` with the matrix.

**Jacket partitions.** `jacket_test` gives each platform a complete, disjoint set of `hash:K/N` shares; `all` is one full share. The job-count limits of the fast and full tiers still apply.

## Narrowing the stdlib tests

The `stdlib-runtime` legs run `jacket test` over the whole standard library (`tools/ci/stdlib_jacket_test.sh`). When a pull request, or a push to a branch other than `main` or `staging`, changes only stdlib module sources (`stdlib/**.jai`) and golden outputs (`tests/**.stdout|stderr|exit`), at most 100 files, `ci_changes.py` writes them as the JSON `stdlib_changed` output. The leg passes it as `JACKET_TEST_CHANGED`, and `tools/ci/select_tests.py` asks `jacket test --list --list-target --changed FILE...` with each mode's exact arguments; the dependency graph is what `jaic -deps` says each test program read, so CI keeps no module map. Each mode uses its own names as `--exact` filters before `--partition`. An empty native selection skips only that mode, since target-only WASM declarations may still be affected. A failed query runs the whole mode.

Everything else runs every test: any file outside those patterns (compiler, runtime, tools, CI config, a stdlib `.c`/`.h`/data file), pushes to `main`/`staging`, scheduled and manual runs, an undiffable range, and any failure of `jacket` or the script (exit 3). To force a full run on a branch, include a non-stdlib file in the change or run the workflow by hand. `Basic` and other widely imported modules select nearly everything on their own. The Windows and sanitizer `jacket test` steps and the `sdl3` jobs are not narrowed this way (`sdl3` stays own-paths-only).

## How to change it

- Gate another job: add its id to `GATED` in `job_inputs.py` and to an area in `ci_changes.py`; `ci.yml` gates it with `contains(fromJSON(needs.changes.outputs.jobs), 'ID')`.
- A path a step reaches only at run time: add it to `EXTRA`. Checkout policies belong to the shared `CHECKOUT_INPUTS` dependency instead of a fixture-specific exception. The action scanner follows block-style step mappings and local composite actions (`action.yml` or `action.yaml`), including nested composites; extend its focused controls if workflow step syntax changes.
- A new `cargo -p` crate in a gated job: add it to `CRATES` (the drift test says which).
- Inspect a job's inputs: `python3 tools/ci/job_inputs.py --print [JOB ...]`.
- Which diffs narrow the stdlib tests: `NARROW_STDLIB`, `FULL_REFS` and `NARROW_STDLIB_LIMIT` in `ci_changes.py`. How a test depends on a file is `crates/jacket/src/test/deps.rs`.
- Tests: `test_job_inputs.py`, `test_ci_changes.py` and `test_select_tests.py`.

## Configuration

`ci_changes.py` reads the GitHub event variables (`GITHUB_EVENT_NAME`, `EVENT_BEFORE`, `SHA`, the pull request base). `JAIC_DEPS_JAIC` names a `jaic` for the stdlib closure. `GITHUB_REF` decides the branch rule above; `JACKET_TEST_CHANGED` is read by `stdlib_jacket_test.sh`.

The first Linux x64 TOML tooling leg runs `test_pins.py`, `test_job_inputs.py` and `test_ci_changes.py` once with its pinned Python, so pin, input-closure and matrix regressions have a job that needs no compiler matrix. Extend those modules when changing the matching CI behavior.

## Manual image jobs

`images-ui-intel` and `image-conformance` are manual-only: they run only when named in a `jobs=` request, never on push, pull request or nightly, and upload no artifacts.

```sh
gh workflow run ci.yml --ref BRANCH -f tier=fast -f jobs=test,stdlib-runtime,windows-native,scripting-wasm,docs,images-ui-intel,image-conformance
```

`images-ui-intel` runs on `macos-15-intel` (60 minutes) with the usual tool and native-library setup, in separate steps: runtime SIMD dispatch and cross-target Web macros, the portable foundation producer, filename-to-ICO native/WASI execution and UI image readback. `tools/ci/check_images_ui.py` runs the CPU and doc checks locally (`--jaic`, `--jacket`, `--output-root`): it records the host architecture, checks Linux/macOS/Windows x64 and ARM64 macros, runs dispatch interpreted and native O0/O2, and checks the three Web doc examples for each native OS. Extend its test list when the stage boundary changes. The UI runner skips explicitly when no hosted Metal adapter exists; that is build evidence, not GPU acceptance.

`image-conformance` runs the complete pinned image inventories on Linux in sanitized native, optimized native and WASI modes, plus independent encoder checks, with the pinned JPEG reference built from source. See [image codec coverage](image-conformance.md).
