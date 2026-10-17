# Development workflow

The Justfile, the pre-commit hook, formatting, the dependency policy and continuous integration.

## Justfile

The root `Justfile` gives each dev command one name. CI's format and lint steps call the same recipes, so local and CI runs cannot drift. `just --list` shows them. Recipes are thin and take options (`just fmt --check --lang jai a.jai`); the logic lives in `tools/dev.py`, which finds or builds the tools (`jaifmt` is rebuilt when `jaic` or its sources are newer; `jailint` via `cargo build -p jailint`).

| Recipe | Does |
| --- | --- |
| `build` | cargo build of `jaic-cli`, `jai-language-server`, `jailint`, then `target/jaifmt` (`--release`, `-j`) |
| `fmt` | `jaifmt` on Jai; `cargo fmt` plus `tools/rust_item_spacing.py` on Rust (`--check`, `--lang jai\|rust\|all`, `--staged`, paths) |
| `lint` | `jailint -D warnings` on Jai; `cargo clippy --workspace --all-targets -- -D warnings` on Rust (`--fix`, `--lang`, `--staged`) |
| `check` | `fmt --check`, `lint`, `editor`, `nvim-test --unit` |
| `test` | `cargo test` (CI's skips) and the `tools/` unittests (`--suite cargo\|tools\|all`, `--crate NAME`) |
| `nvim-test`, `editor` | Neovim plugin tests; VS Code generated-file checks (skipped with a note without nvim 0.11 / node 24) |
| `hooks` | `git config core.hooksPath .githooks` |
| `fetch-upstreams`, `wasm`, `vscode` | `tools/fetch_upstreams.py`, `tools/build_scripting_wasm.py --release`, `pnpm run <script>` in `editors/vscode` |

`--staged` takes files from `tools/staged-files.sh` (also used by the hook); clippy cannot run on single files, so for Rust it only decides whether the workspace is linted.

To add a recipe: give it a doc comment (it becomes the `--list` text) and a `[group(...)]`, keep the body to one command and put logic in `tools/dev.py`. Options use `[arg("name", long)]`; a flag is `[arg("check", long, value="true")]` on a parameter defaulting to `"false"`. Needs just 1.58 (CI pins `just-version`). Variadic `*paths` are joined by spaces, so paths with spaces fail. Windows CI does not call the recipes. `JAI_JOBS` sets the default `--jobs` (3, for a 16 GiB machine).

## Pre-commit hook

`.githooks/pre-commit` (POSIX `sh`) checks staged files so failures show before CI. Enable with `just hooks`; skip once with `git commit --no-verify`. It exports the staged contents (`git checkout-index`) into a temp directory, so a partly staged file is checked as staged and nothing in the working tree is touched.

| Staged files | Check |
| --- | --- |
| `*.jai` | `jaifmt --check` on the export; `jailint -D warnings` on the working tree files (it needs the surrounding loaders and modules; a file another program `#load`s is linted through that program) |
| `crates/**/*.rs` | `rustfmt --check --edition 2024` per staged blob (stdin mode, pinned nightly); `tools/rust_item_spacing.py --check` |

Tools are found as follows: `target/jaifmt` (rebuilt with `jaic jaifmt/main.jai -O2 -exe target/jaifmt` when sources are newer and a `jaic` exists), `target/{release,debug}/jailint` (else `cargo build -p jailint`), then `PATH`. A missing tool prints a warning and skips its check. Clippy is not run (minutes per crate); CI runs it. `rustfmt --check` in stdin mode exits 0 on a diff, so any output counts as failure. To add a check, add a file filter and a block appending to `fixes`; keep it fast.

| Variable | Effect |
| --- | --- |
| `JAI_HOOK_SKIP` | space-separated checks to skip: `jaifmt jailint rustfmt spacing` |
| `JAI_HOOK_NO_BUILD=1` | never rebuild a stale tool |
| `JAI_HOOK_VERBOSE=1` | say which checks run |

Tested by `tools/test_pre_commit_hook.py` on a temporary repository with stand-in tools.

## Code formatting

Rust uses the pinned nightly `rustfmt` and `rustfmt.toml` (edition 2024, `max_width = 100`, compact items expanded, at most two blank lines). `tools/check_rust_format.py` first checks that every manifest under `crates/` is a workspace member (an unregistered crate would escape formatting), then runs `cargo fmt --all -- --check` and `tools/rust_item_spacing.py --check` (rustfmt does not put blank lines between multi-line items; the script does). Jai uses [jaifmt](jaifmt.md) with `jaifmt.toml`; CI runs `jaifmt --check stdlib tests benchmarks tools jaifmt examples`. A new crate must be added to `members` in the root `Cargo.toml`. Change style options in `rustfmt.toml` and land the reformat as its own commit.

**Format-only commits** (repo-wide `jaifmt` or `cargo fmt`, or a style option change) contain nothing else, use the message `style: format with jaifmt` (or `rustfmt`), and are listed in `.git-blame-ignore-revs`. The hash is only known after committing, so add it in a second commit, and only after the commit has landed on `main` (a rebase changes the hash). Locally opt in once with `git config blame.ignoreRevsFile .git-blame-ignore-revs`.

## Output directory

Tools and test scripts write build products, downloads and scratch output under `target/` (ignored; shared with cargo). `tools/repo_output.py` (`output_dir`, `shared_output_dir`) and `tools/repo_output.mjs` (`outputDir`) are the single helpers; never hard-code a path. `JACKET_OUTPUT_DIR` relocates the whole tree (`jaic` honours it when looking for `native-libs/<platform>`). `shared_output_dir` resolves to the main checkout's directory so worktrees reuse native builds and downloads.

| Directory under the output dir | Contents |
|---|---|
| `native-libs/<platform>`, `native-libs/sources` | `tools/build_native_libs.py` output and downloaded sources |
| `sdl3/` | SDL3 source cache, builds, generator, lint and test scratch |
| `thirdparty/` | Slang, JoltPhysics clones and builds |
| `cache/upstream/` | bare clones for `tools/fetch_upstreams.py` |
| `webgpu/<rev>/` | pinned `webgpu.yml` |
| `scripting-runtime/`, `ci-wasm/`, `wasm/`, `browser-release/` | browser bundles |
| `lsp-edits/`, `numerics/`, `extensions-*`, `http-client-*`, `core-public-verification/` | test and verification evidence |

## Dependency policy

Cargo only selects crate versions published at least 14 days ago, direct and transitive. A git dependency must pin a full commit hash that landed upstream at least 14 days ago. `jaic` has no external dependencies; the external crates are `serde`/`serde_json` (language server) and `inkwell`/`llvm-sys` (LLVM backend). The same age rule applies to npm packages in `editors/vscode` ([VS Code extension](vscode-extension.md)).

`.cargo/config.toml` enables nightly `min-publish-age` (`registry.global-min-publish-age = "14 days"`, `incompatible-publish-age = "deny"`). Cargo allows young entries already in a lockfile, so `python3 tools/check_dependency_age.py [--lockfile PATH]` checks every locked crate against the crates.io API and fails closed on unknown sources, missing metadata, yanked releases and young versions. Git sources are accepted only as `rev = "<40 hex>"` on GitHub, whose committer date is checked via the GitHub API (`GITHUB_TOKEN` authenticates it). It runs before any build in `ci.yml`, `release.yml`, `browser-release.yml` and `fuzz.yml` (for `fuzz/Cargo.lock`). Tests: `tools/test_dependency_age.py`.

`inkwell` currently comes from git: release 0.10.0 supports LLVM up to 22, while LLVM 23 support (`llvm23-1`) is on its main branch. The root `Cargo.toml` pins an aged commit for `inkwell` and `inkwell_internals`. Once a crates.io release with `llvm23-1` is 14 days old, switch back to `version = "..."` and drop `allowBuiltinFetchGit` in `nix/jaic.nix`. Put shared dependencies in `[workspace.dependencies]`, prefer exact versions, and never weaken the age policy to resolve a conflict.

## Continuous integration

Workflows in `.github/workflows/`:

| Workflow | Runs |
| --- | --- |
| `ci.yml` | pushes to `main` and `staging`, pull requests and manual runs: all checks below, including the fuzz crash-input replay |
| `fuzz.yml` | nightly fuzzing and manual runs ([testing](testing.md#fuzzing)) |
| `http-conformance.yml` | only when `stdlib/Extensions/Http/**` or its harness (`tools/http_conformance.py`, `http_native.py`, `gen_http_corpus.py`, `http_deviations.json`) changes, and by hand |
| `release.yml` | tags ([releases](releases.md)) |
| `compile-bench.yml`, `browser-release.yml` | manual only ([benchmarks](benchmarks.md), [browser releases](../browser/playground.md)) |

Shared setup lives in composite actions: `.github/actions/setup-unix` (pinned nightly Rust, Python, LLVM 23, Linux native libraries), `setup-windows` and `windows-provider`. `tools/ci_native_deps.sh {developer|clipboard|full}` builds the native helper libraries the jobs link.

`ci.yml` starts with a `changes` job that runs `tools/ci_changes.py`, which diffs the pushed range (or PR base; anything undiffable runs everything) and sets `<area>=true|false` outputs for the path-gated jobs: `sdl3`, `clipboard-linux`, `nix`, packaging (`shellcheck`, `install-sh`, `install-ps1`, `homebrew`) and the `windows-*` jobs. To gate a job on more paths, add a pattern to `AREAS` in that script (tests: `tools/test_ci_changes.py`). Editing `ci.yml` or `.github/actions/` runs every area.

Always-on jobs:

- `scripting-wasm` (ubuntu): builds the browser engine and runs `tools/check_scripting_wasm.mjs`, `check_jai_format_wasm.mjs`, `check_playground_stdlib.mjs` (every `tests/stdlib` program in the browser engine) and `test_webgpu_host.mjs`, then differential testing `interp,wasm` with an LLVM-free `jaic`.
- `test` (macOS arm64, Linux x86-64/arm64; macOS Intel only on pushes to `main`): dependency-age and `tools/check_ci_sources.py` (literal `include_str!` inputs must exist in the checkout) checks, `just fmt --check --lang rust`, `just lint --lang rust`, `cargo test --workspace --no-fail-fast`, the Python tool tests (`unittest discover -s tools -p 'test_*.py'`), `just fmt --check --lang jai`, `just lint --lang jai`, a `JAIC_CODEGEN_UNITS=4` rerun of the native tests, differential testing, the wasm target (`jaifmt.wasm` against native, `interp,wasm-native`), and the [stdlib target](testing.md#stdlib-target-check) and [Objective-C selector](testing.md#objective-c-selector-check) checks. Each check records its outcome and a final step fails the job if any failed, so one run reports every failure.
- `vscode-extension`, `neovim-plugin`: see [VS Code extension](vscode-extension.md), [Neovim plugin](neovim-plugin.md).
- `stdlib-runtime` (macOS arm64, Linux x86-64/arm64): every [stdlib runtime test](testing.md#stdlib-runtime-tests) in all modes (`--strict`, Xvfb and Mesa on Linux), headless and window WebGPU tests, and the per-module coverage ratchet against `tests/stdlib-coverage.txt`.
- `upstream-smoke`: builds and runs pinned third-party projects ([upstream corpus](upstream-corpus.md#smoke-tests)).
- `sanitizers` (Linux x86-64, macOS arm64): `tools/jaic-sweep.py --sanitize address,undefined corpus stdlib modules` at `-O0` and `-O2` ([sanitizers](../native/sanitizers.md)).

Path-gated jobs: `sdl3` (native SDL3 artifacts), `clipboard-linux` (X11 clipboard on Xvfb), `nix` (flake check and build; [releases](releases.md#nix)), the packaging jobs (install scripts and Homebrew formula on macOS, Linux, Windows), and the Windows jobs: `windows-cross` (MinGW cross-builds corpus and stdlib programs, x64 and arm64), `windows-run-cross` (runs them on Windows), `windows-native` (builds `jaic` against the official LLVM and runs the stdlib runtime tests, `cargo test --test native` and the WebGPU tests on D3D12 WARP) and `windows-smoke` (third-party projects). The x64 Windows jobs install Mesa's llvmpipe `opengl32.dll` from a pinned, checksum-verified mesa-dist-win (`tools/install_ci_mesa_windows.sh`) because runners only have GDI OpenGL 1.1. See [Windows](../native/windows.md).

The full corpus sweep with the `upstream` set is not in CI (the corpus is fetched, not committed); CI fetches only what the smoke checks use.

**Concurrency.** A push cancels that branch's older `ci.yml` run. On `main` every push has its own group so the light jobs finish for each commit; heavy jobs (`stdlib-runtime`, `upstream-smoke`, sanitizers, the Intel leg) have job-level `concurrency` keyed by job, ref and platform, so only the newest `main` commit runs them to the end. Add or remove that block to change which jobs are heavy. Nightly fuzz runs never cancel.

**To change it:** edit the workflows. Keep each check's step id paired with the `steps.<id>.outcome` test in the job's final enforcement step, or failures are silently ignored. Pin third-party actions by commit SHA and keep `permissions: contents: read`. A `test_*` Python file under `tools/` is picked up by the Python test step; Node tests must be added to the relevant job by hand. Runners: `macos-15`, `macos-15-intel`, `ubuntu-24.04`, `ubuntu-24.04-arm`, `windows-2025`, `windows-11-arm`; 30 minute timeouts (45 for sanitizers); `CARGO_INCREMENTAL=0`; toolchain `nightly-2026-08-29`; Python 3.14; node 24.

## Dependencies

[just](https://github.com/casey/just) 1.58+, `python3`, the pinned Rust toolchain, `pnpm` (for `just vscode`), `git`, and the repo-built `jaifmt` and `jailint`.
