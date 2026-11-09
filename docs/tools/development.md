# Development workflow

The Justfile, the hooks, formatting, the dependency policy and CI.

## Justfile

Each dev command has one name, and CI's format and lint steps call the same recipes. `just --list` shows them; the logic lives in `tools/dev/dev.py`.

| Recipe | Does |
| --- | --- |
| `build` | Builds `jaic-cli`, `jai-language-server`, `jailint`, `target/jaifmt` and the native pieces a release archive ships (`--release`, `--native=false`) |
| `fmt` | `jaifmt` on Jai; `cargo fmt` plus `tools/dev/rust_item_spacing.py` on Rust (`--check`, `--lang`, `--staged`) |
| `lint` | `jailint -D warnings` on Jai; clippy on Rust (`--fix`, `--lang`, `--staged`) |
| `check` | `fmt --check`, `lint`, `editor`, `nvim-test --unit` |
| `test` | `cargo nextest run --profile ci` (needs cargo-nextest) plus doctests and the `tools/` unittests (`--suite cargo\|tools\|all`, `--crate`, `--partition slice:K/N`) |
| `repo-checks` | CI's text checks: source style, repo paths, CI sources, stdlib docs |
| `hooks` | Points `core.hooksPath` at `.githooks` |
| `bump-version X.Y.Z` | The version step of [releases](releases.md) |
| `docs` | Builds the stdlib docs site into `target/doc-site` (`--wasm` for working Run buttons, `--serve --port N` to serve it like GitHub Pages, `--no-test-examples` to skip running the examples) |
| `run-example [NAME] [-- ARGS]` | Builds and runs a program from `examples/` (`chip8`, `http/crud`, `game2d` for `game2d/game`), passing ARGS to it; with no name, lists them |
| `fetch-upstreams`, `wasm`, `vscode` | Fetch the corpus, build the browser engine, build the extension |
| `vscode-try [FOLDER]` | Opens VS Code in a throwaway profile with only the built extension and tools |
| `profile-report PROFILE` | Summarizes a samply profile ([profiling](benchmarks.md#profiling)) |

The recipes export `PYTHONPATH=<checkout>/tools`. Outside `just`, run scripts as `PYTHONPATH=tools python3 tools/<folder>/x.py` ([module path](tools-layout.md#module-path)).

`just build` gives a checkout what a release archive has beside `stdlib/`: it builds the third-party libraries the stdlib binds into `target/native-libs/<platform>` ([native libraries](native-libs.md); the first run is slow, SDL3 alone takes minutes) and the `Bindings_Generator` helpers. Both need the LLVM SDK ([LLVM setup](llvm-setup.md)). A plain `cargo build` only makes the Rust binaries.

To add a recipe, give it a doc comment and a `[group(...)]`, keep the body to one command, and put logic in `dev.py`. Needs just 1.58. `JAI_JOBS` sets the default `--jobs` (3).

## Tests

`cargo nextest run --profile ci`. Settings are in `.config/nextest.toml`, which lists tests left out because they need separately supplied sources; add to `default-filter` to skip another. Nextest does not run doctests, so `just test` runs `cargo test --doc` too. The compiler and stdlib test tools are in [testing](testing.md).

## Pre-commit hook

`.githooks/pre-commit` runs `python3 tools/dev/dev.py precommit`. Enable it with `just hooks`; skip once with `--no-verify`. It checks the staged contents exported to a temp directory, so a partly staged file is checked as staged. The hook path is relative, so each worktree runs its own hooks. It fails with an install hint when `python3` is missing; use `--no-verify` to skip it.

| Staged files | Check |
| --- | --- |
| `*.jai` | `jaifmt --check`; `jailint -D warnings` |
| `crates/**/*.rs` | `rustfmt --check`; `rust_item_spacing.py --check` |
| shell scripts | `shellcheck` (skipped if not installed) |
| `.github/workflows/*.yml` | `actionlint` (skipped if not installed) |
| anything | `just repo-checks` |
| `Cargo.toml`, `Cargo.lock` | `cargo metadata --locked --offline` for the workspace and `fuzz/` |
| `crates/**`, `editors/vscode/**` | the VS Code grammar and schema checks (needs node 24) |

Tools are built from this checkout (`jailint` through cargo, `jaifmt` rebuilt when `jaic` or its sources are newer), since a shared target directory can hold another checkout's binaries. It checks an export of the staged files, with every tracked `jacket.toml` beside them, so a folder's own `[format]` (the doc theme's) applies. When the hook can't get a `jaifmt` built from this checkout, it warns and skips the format check. It never uses an installed `jaifmt`, whose rules may be older.

| Variable | Effect |
| --- | --- |
| `JAI_HOOK_SKIP` | Space-separated checks to skip: `jaifmt jailint rustfmt spacing shellcheck actionlint repo locks editor` (pre-commit); `actions clippy` (pre-push) |
| `JAI_HOOK_NO_BUILD=1` | Never build a tool; use what exists |
| `JAI_HOOK_VERBOSE=1` | Say which checks run |
| `JACKET_HOOKS_NO_CARGO=1` | For jobs with only a prebuilt toolchain and no Rust: skip the cargo steps (below) |

Cargo is needed to build `jaic` and `jailint`, run `rustfmt` and check lock files. When it is missing the hook fails with an install hint. Set `JACKET_HOOKS_NO_CARGO=1` to opt out: those steps are skipped, nothing is built, and `jaifmt` and `jailint` must already exist (otherwise the hook warns and skips their checks, as above). Everything else still runs: item spacing, shellcheck, actionlint, repo checks, the editor check. No CI job runs the hooks today; a job that does and has no Rust should set it.

A new per-system file under `stdlib/internal` (imports `windows-api`, `posix-api`, X11, or `#assert`s an OS) must be listed in `[lint.portability.platforms]` in `jacket.toml`, or the `platform_leak` check fails. Tests: `tools/ci/test_pre_commit_hook.py`.

## Pre-push hook

`.githooks/pre-push` runs two checks. Skip either with `--no-verify` or `JAI_HOOK_SKIP`.

- `actions`: `tools/ci/check_action_pins.py` ([action pins](action-pins.md)). A broken or unreachable check blocks the push; available updates are only printed.
- `clippy`: CI's clippy on the packages whose Rust or `Cargo.toml` files differ from the remote, via `tools/dev/pre_push_clippy.py`. It fails, rather than skipping, when Rust files changed and `cargo` is missing or `cargo metadata` fails.

With `JACKET_HOOKS_NO_CARGO=1` the clippy check is skipped; without it, missing `cargo` fails as above.

The hook fails when `python3` is missing, unless both checks are skipped.

## Formatting

Rust uses the pinned nightly `rustfmt` and `rustfmt.toml`. A new crate must be added to `members` in the root `Cargo.toml`, or it escapes formatting. Jai uses [jaifmt](jaifmt.md) with `[format]` in `jacket.toml`. Format-only commits contain nothing else and use the message `style: format with jaifmt` (or `rustfmt`).

Rust lints live in the root `[workspace.lints]`, thresholds in `clippy.toml`. `std::collections::{HashMap, HashSet}` are banned; use `jaic::fxhash`.

## Output directory

Build products, downloads and scratch output go under `target/`. `playground/` is also ignored, for local experiments; never commit it. Use `tools/repo_output.py` (`output_dir`, `shared_output_dir`) or `tools/repo_output.ts` instead of hard-coding a path. `JACKET_OUTPUT_DIR` relocates the tree; `shared_output_dir` resolves to the main checkout so worktrees reuse native builds.

## Dependency policy

Cargo only selects crate versions published at least 14 days ago, direct and transitive. A git dependency must pin a full commit hash that is at least 14 days old. The same rule applies to npm packages in `editors/vscode` (pnpm's `minimumReleaseAge`).

`.cargo/config.toml` enables `min-publish-age`, but cargo accepts young entries already in a lockfile, so `PYTHONPATH=tools python3 tools/ci/check_dependency_age.py [--lockfile PATH]` checks every locked crate against crates.io and fails on unknown sources, yanked releases and young versions. It runs before any build in the workflows. `--cache FILE` keeps answers between runs.

`inkwell` comes from a pinned git commit until a crates.io release supports the LLVM version we build against; then switch back to `version = "..."` and drop `allowBuiltinFetchGit` in `nix/jaic.nix`. Put shared dependencies in `[workspace.dependencies]`, prefer exact versions, and never weaken the age policy to resolve a conflict.

`cargo machete` and `cargo deny` run in the `dependency-hygiene` job; `deny.toml` holds the license and source policy.

## Fixing reported bugs

A fix is done when a test fails before it and passes after it, on a machine CI has.

1. Explain every line of the reporter's output from the cause you name. A line that doesn't fit means the cause is wrong.
2. Reproduce first. For hardware- or driver-specific bugs, make the code take those calls as replaceable procedures and write a stand-in that behaves like the reporter's environment.
3. Check the new path ran (assert, or print and check).
4. Before adding a compile error that rejects code jaic used to accept, run the upstream corpus smoke checks ([upstream corpus](upstream-corpus.md)).
5. For bugs only seen on setups we can't run, label the issue `needs confirmation` and leave it open.

## Continuous integration

| Workflow | Runs |
| --- | --- |
| `ci.yml` | pushes to `main`, pull requests, nightly and manual runs |
| `fuzz.yml` | nightly fuzzing ([testing](testing.md#fuzzing)) |
| `http-conformance.yml` | when `stdlib/Extensions/Http/**` or its harness changes |
| `release.yml` | tags ([releases](releases.md)) |
| `compile-bench.yml`, `browser-release.yml` | manual only |

`ci.yml` starts with a `changes` job running `tools/ci/ci_changes.py`, which emits the list of job ids to run from the changed paths ([CI job selection](ci-job-selection.md)). It also picks the tier: pushes and pull requests run the fast tier (aiming for ten minutes), nightly and manual runs the full tier. The test matrices live in that script and `test_ci_changes.py` checks them. A change that only touches prose sets `code=false` and skips build and test jobs; docs that a test or generator reads count as code (`READ_BY_TESTS`).

Main jobs: `test` (Rust tests in nextest shards, format, lint, repo checks, differential testing, stdlib target checks), `stdlib-runtime` ([stdlib runtime tests](testing.md#stdlib-runtime-tests)), `sanitizers` ([sanitizers](../native/sanitizers.md)), `scripting-wasm`, `upstream-smoke` ([upstream corpus](upstream-corpus.md#smoke-tests)), `vscode-extension`, `neovim-plugin`, `dependency-hygiene`, path-gated jobs (`sdl3`, `clipboard-linux`, `nix`, packaging, and the `windows-*` jobs) and `docs`.

Things to know when editing workflows:

- Every action is pinned by commit SHA with a version comment; keep `permissions: contents: read`. `tools/ci/check_action_pins.py` verifies the pins and applies updates ([action pins](action-pins.md)).
- Keep each check's step id paired with its `steps.<id>.outcome` test in the job's final enforcement step, or failures are silently ignored.
- Tool versions the workflows download live in `.github/pins.json`; `PYTHONPATH=tools python3 tools/ci/pins.py check` fails when a workflow disagrees, `pins.py sync` rewrites them.
- Workflows call scripts in `tools/ci/` instead of carrying shell logic. A `test_*.py` under `tools/` is picked up by the Python test step; Node tests are added to the job by hand.
- Apt setup goes through `apt_configure` in `tools/ci/apt_lib.sh`, which isolates the runner's vendor feeds. Register a new required repository there; don't ignore update errors.
- GitHub runs only five macOS jobs at a time per account, so macOS jobs are merged and the fast tier has exactly five.
- A push to `main` or a pull request cancels its older run, so only the newest commit finishes. Nightly and manual runs are never cancelled by a push.
- Long sweeps split with `--shard K/N`; Rust tests with nextest `--partition slice:K/N`. To change the count edit `shards(N)` in `test_matrix`/`stdlib_matrix`.
- `release.yml` has one non-cancelling concurrency group per tag.

### Caches

The expensive cache is `target/native-libs` (`.github/actions/native-libs-cache`), keyed on OS, CPU, LLVM version and the hash of `tools/native/native-libs.json`, `tools/native/**` and the build scripts, and saved from `main` only. A build input outside that key must be added to the action's `hashFiles` list. Cargo builds use `rust-cache`, the upstream mirrors `upstream-mirrors-cache`. Keep tool output out of `target/<profile>/`, which is cached.

## Dependencies

[just](https://github.com/casey/just) 1.58+, `python3`, the pinned Rust toolchain, `pnpm` (for `just vscode`, `just site-assets` and `just tools-scripts`), and the repo-built `jaifmt` and `jailint`.

### Investigating jacket CI partitions

`tools/ci/stdlib_jacket_test.sh` runs native, interpreted and O2 tests on each native
platform; `WASM=true` additionally runs interpreted and built wasm on Linux x64.
`JACKET_TEST_CHANGED` narrows discovery first and fails open to all tests if selection
fails; `JACKET_TEST_PARTITION` then selects `all` or `hash:K/N`. Never substitute one mode
or producer suite for another when moving work between jobs.

Matrix assignments live in `tools/ci/ci_changes.py`; coverage invariants are tested in
`test_ci_changes.py`. The shell runner owns execution, and the workflow owns
runner setup. Prefer existing jobs when testing new assignments.
