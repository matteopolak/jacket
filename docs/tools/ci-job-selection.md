# CI job selection

`tools/ci/ci_changes.py` decides which path-gated jobs of `ci.yml` run for a push or pull request. It keeps no path list per job: `tools/ci/job_inputs.py` computes each job's input files from the job itself, so a new `#import`, a new crate dependency or a new script called from a step selects the right jobs with no edit under `tools/ci/`.

## How it works

For a gated job, `job_inputs.inputs()` collects:

- every path in the job's `run:` lines and `with:` values, and in the composite actions it uses;
- root and nested `.gitattributes` policies when an actual expanded step uses `actions/checkout`, because checkout can change exact file bytes. This dependency also matches new or deleted policies and conditional checkout steps; jobs without checkout do not gain it. Script literals and nested `env`/`with` keys named `uses` are not action references;
- the closure of those scripts (Python imports, path strings and exact sibling data/license/entry-source filenames; paths named in shell, PowerShell and Nix files);
- for each `cargo build|test -p NAME`, the crate and its path dependencies, plus `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` (dev-dependencies only for jobs that run tests);
- for each `.jai` file a step compiles, the stdlib modules it reads (from `jaic -deps` if a binary exists, else a static scan).

A Python literal naming an existing sibling file also follows that file, including `HERE / "pin.json"`, a license or a Jai entry source. The graph uses indexed filenames; add new inputs to the index (or use `git add -N`) before inspecting a local graph.

`ci_changes.py` matches the changed files against each area's patterns and writes the `jobs=` and `<area>=` outputs `ci.yml` reads. If the computation fails, all default groups run. Explicit `MANUAL_ONLY` groups remain excluded unless named in the manual `jobs` selector.

Hand-written policy in `ci_changes.py`:

| Table | Meaning |
| --- | --- |
| `NARROW` | The macOS sanitizer leg runs on pushes only for codegen, driver, sweep, native tooling and shared checkout-policy inputs. |
| `MANUAL_ONLY` | Focused acceptance jobs such as `images-ui-intel`, `image-conformance` and `docs-producer` run only when explicitly named in a manual `jobs=` request. The `docs-producer` caller additionally requires `workflow_dispatch`. |
| `OWN_PATHS_ONLY` | `sdl3` does not run for a workflow edit. |
| `PROSE`, `READ_BY_TESTS` | Docs-only changes skip the code jobs, except docs a test reads. Checkout policies are code inputs even under `docs/`. |

## Compiler target work

Full-tier Intel macOS divides the stdlib module/target checks and cross-OS documentation checks between its existing `rust` and `checks` jobs. The jobs use `targets_part=2/2` and `1/2` respectively. This distributes a large aggregate check without changing nextest's per-test limits, job budgets or case inventory. Objective-C selectors run only in the first target share.

`matrix.shard` still selects the complete native Rust suite on each full-tier Mac Rust job. The target steps use `matrix.targets_part` when present, otherwise `matrix.shard`; fast Mac, ARM Mac and Linux keep their existing assignments. When changing the matrix, update both target-step environment values together and keep a complete, disjoint set of shares per platform. `test_ci_changes.py` checks those shares and guards against accidentally splitting native Rust coverage.

The owning Rust test emits reserved `STDLIB_TARGET_INVENTORY` JSON records for the canonical full and assigned pair lists, returned compiler processes, and a final marker after its existing expectation assertions. A compiler exit failure can be expected for an unsupported target; completion alone does not prove verification. Successful nextest output is normally hidden; `NEXTEST_SUCCESS_OUTPUT=final` exposes these records without changing the test limit.

## Producer work

Full-tier Intel macOS runs `tooling` and `bindings` on its existing catalogue job. The other producer suites stay with the complete native/interpreted/O2 Jacket run. This keeps the long tooling run from leaving bindings only the remainder of the same job budget. Both jobs retain their 150-minute limit, existing test commands and complete case inventories; other platforms and the fast tier keep their assignments.

The `suites` matrix field selects these existing drivers. Keep tooling and bindings together: tooling rebuilds native tools that bindings reads, so `check_stdlib_suites.py` runs them in one resource group. Each CI job has its own workspace. When changing assignments, preserve each complete producer suite exactly once and retain the independent catalogue, Jacket and WebGPU coverage; update `test_ci_changes.py` alongside the matrix.

## Narrowing the stdlib tests

The `stdlib-runtime` legs run `jacket test` over the whole standard library (`tools/ci/stdlib_jacket_test.sh`). When a pull request or a push to a branch other than `main` or `staging` changes only stdlib module sources (`stdlib/**.jai`) and golden outputs (`tests/**.stdout|stderr|exit`), at most 100 files, `ci_changes.py` writes them as the JSON `stdlib_changed` output. The leg passes it as `JACKET_TEST_CHANGED`, and `tools/ci/select_tests.py` asks `jacket test --list --list-target --changed FILE...` with each mode's exact arguments. The dependency graph is what `jaic -deps` says each test program read, so no module map is kept in CI. Each mode uses its own names as `--exact` filters before `--partition`. A native empty selection skips only that mode: target-only WASM declarations may still be affected. Any failed query runs the complete mode instead of narrowing.

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

The first Linux x64 TOML tooling leg runs `test_pins.py`, `test_job_inputs.py` and `test_ci_changes.py` once with its pinned Python. This keeps pin isolation, input closure and matrix assignment regressions in a selectable relevant job group, without requiring the compiler test matrix. Extend those modules when changing the corresponding CI behavior; the TOML conformance predicates are covered in `test_ci_changes.py`.

## Jacket partition evidence

`jacket_test` assigns a complete, disjoint set of `hash:K/N` shares to each platform.
`all` is a single full share. Catalogue modes/O2, producer suites, WebGPU and the Linux
coverage baseline are independent inventory and must remain when changing assignments.
The fast/full job-count limits still apply; splitting jacket work does not add runner slots.

For a manual matrix experiment, set `jacket_inventory=true` on `ci.yml`. The jacket step
records unpartitioned discovery, assigned discovery and actual JSON outcomes per mode in
`$RUNNER_TEMP/jacket-inventory` (outside restored caches), then uploads one artifact per platform/job. The artifact schema is allowlisted: relative case
name/kind/event, child name/status, normalized skip category, run counts, and public source revision, partition,
OS/architecture and Rust version. Reporter paths, arbitrary stdout/stderr, diagnostics,
notes, temporary paths, hostnames and environment variables are excluded. This additional
listing work is opt-in and must be enabled on both sides of a timing comparison. Full and assigned catalogues use `--list-target`, so declarations excluded by the selected target's `#if` conditions are not mistaken for missing executed cases. Default host listings remain available for browsing. The display wrapper preserves Jacket’s stderr separately from its JSON stdout, including with `xvfb-run` versions that merge client streams. Root/probe diagnostics stay in the job log and the JSON filter remains strict.

```sh
gh workflow run ci.yml --ref YOUR_BRANCH -f jobs=stdlib-runtime -f tier=full -F jacket_inventory=true
gh run download BASELINE_RUN --pattern 'jacket-inventory-*' --dir baseline-inventories
gh run download CANDIDATE_RUN --pattern 'jacket-inventory-*' --dir candidate-inventories
python3 tools/ci/verify_jacket_inventory.py baseline-inventories candidate-inventories \
  --baseline-revision BASELINE_SHA --candidate-revision CANDIDATE_SHA \
  --platform linux-x64 --platform linux-arm64 \
  --platform macos-arm64 --platform macos-x64
```

The verifier requires disjoint discovery union equality and identical completed per-case
outcomes/skips and completed child identities/statuses in each mode. Child multiplicity is
preserved, and passed parents must include every discovered child. A listing alone cannot
prove execution. It rejects missing
platforms/modes, stale requested revisions, empty catalogues, overlapping shares, incomplete runs and failed cases. Declare every expected platform with `--platform`; this prevents two equally incomplete downloads from proving coverage. Keep source revisions,
runner/toolchain/cache states, queue delays, step durations and total runner minutes with
the downloaded evidence before accepting a scheduling improvement.

## Compiler target identity proof

The same `jacket_inventory=true` input also captures module/target records and per-OS check-only documentation discovery, assignment and completed outcomes from the `test` job. The source matrix determines the required platforms and shares; native Rust keeps its independent shard. All existing compiler commands and assertions run. Additional listings and success-output capture are opt-in.

```sh
gh run download RUN --pattern 'compiler-target-inventory-*' --dir compiler-inventories
python3 tools/ci/compiler_target_inventory.py verify compiler-inventories --tier full --revision EXPECTED_SHA
```

The verifier requires identical ordered canonical module/target catalogues, the exact modulo assignment for each declared share, complete disjoint shares and exactly one returned compiler per assigned identity, followed by the verified marker. It rejects missing, duplicate, truncated, stale, empty, wrong-share or failed command/capture evidence. For each of Linux, macOS, Windows and wasm documentation, each job's completed outcomes must equal its assigned identities and the shares must form the full discovery set. Equal counts alone are insufficient. Doc checks reject flaky outcomes, missing-prerequisite or unexplained runtime skips, and all-skipped OS inventories. Explicit target/mode filters, ignored snippets and declared capabilities remain visible as distinct skip categories. No additional unpartitioned execution is needed because discovery comes from the owning harness and doc driver.

`tools/ci/compiler_doc_inventory.sh` runs the existing doc arguments with JSON reporting. `compiler_target_inventory.py` collects only the fixed module/target schema and checks artifacts against `ci_changes.test_matrix()`. Neither collector copies raw environment or arbitrary reporter diagnostics into its artifact. Both collectors reject malformed input after draining the producer pipe, preserving the original test command’s outcome. The documented `slow` progress notice is excluded from coverage records; a completed outcome is still required for every assigned case. Unknown events remain errors. Its original command, tee and capture exit codes are retained separately. This proof does not replace required green CI or establish a benchmark improvement.

For hosted opt-in inventory only, `--diagnostics` retains normal failed/timed-out/unexpected-pass/flaky case details in the masked CI log while the uploaded coverage artifact stays allowlisted. The two CI scripts enable it only under `GITHUB_ACTIONS=true`; the filter refuses it locally before reading input. Passed-case output and arbitrary environment fields are not copied. Public hosted workspace/temp paths in failure details are expected; no local evidence is copied into CI.

The verifier requires the explicit expected SHA and reads the matrix from this checkout. Use the reviewed source matching that claim. A Mac-only comparison declares both Mac platforms; a full runtime comparison also declares `linux-x64` and `linux-arm64`. Update the doc OS loop and `DOC_OSES` contract together if supported targets change. The scripts depend on the normal pinned Python, Bash, Rust/nextest and checkout-built compiler tools; no additional service is required.

## Focused Intel image acceptance

`images-ui-intel` is manual-only: it is never added to ordinary push, pull request or nightly runs unless explicitly requested. It runs on `macos-15-intel` with a focused 60-minute job budget. It reuses the existing production tool/native-library setup, then checks runtime SIMD dispatch and cross-target Web macros, the portable foundation producer, filename-to-ICO native/WASI execution and UI image readback. These are separate steps so a failed check does not hide the remaining results. It adds no artifact uploads and does not replace or change the full matrix.

```sh
gh workflow run ci.yml --ref BRANCH -f tier=fast -f jobs=test,stdlib-runtime,windows-native,scripting-wasm,docs,images-ui-intel
```

`tools/ci/check_images_ui.py` runs the focused CPU/docs checks locally (`--jaic`, `--jacket`, `--output-root`). It uses Jacket's existing case budgets and the whole-job bound, without killing a direct parent process on a separate timer. It records the host architecture, checks Linux/macOS/Windows x64 and ARM64 macros, runs dispatch in interpreted/native O0/O2 modes, and checks the three Web documentation examples for each native OS. Extend its test list when changing the stage boundary. `job_inputs.py` still records its real input closure for drift checks. The job explicitly uses its isolated checkout's `target` directory and the existing Rust/native cache actions.

The UI runner still reports an explicit skip if a hosted Metal adapter is unavailable. That skip is build evidence, not native GPU acceptance; actual local Metal and required Linux/Windows software-adapter checks remain distinct evidence. A focused green job does not declare the separate whole-tooling producer job complete.

## Complete image corpus acceptance

`image-conformance` is also manual-only. It adds no automatic fast/full work and changes no existing matrix or timeout. Its separate Linux job runs complete pinned image inventories in native sanitizer, optimized native and actual WASI modes, plus independent encoder checks. It builds the pinned JPEG reference instead of substituting the runner's older system library. Corpora, reference builds and results stay in the job's temporary directory; no new artifacts are uploaded. See [image codec coverage](image-conformance.md) for profiles, pins and commands.

```sh
gh workflow run ci.yml --ref BRANCH -f tier=fast -f jobs=test,stdlib-runtime,windows-native,scripting-wasm,docs,images-ui-intel,image-conformance
```
