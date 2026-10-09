# Justfile

## What it is

The `Justfile` at the repository root gives the dev commands one name each: build, format, lint, test and set-up. CI's format and lint steps call the same recipes, so a local run and CI cannot drift apart. `just --list` shows them.

## How it works

Recipes are thin. Each takes a few options (`just fmt --check --lang jai a.jai`, not one recipe per variant) and hands them to a script:

| Recipe | Does | Options |
| --- | --- | --- |
| `build` | `cargo build` of `jaic-cli`, `jai-language-server`, `jailint`, then `target/jaifmt` compiled by `jaic` | `--release`, `-j/--jobs` |
| `fmt` | `jaifmt` on Jai, `cargo fmt` plus `tools/rust_item_spacing.py` on Rust | `--check`, `--lang jai\|rust\|all`, `--staged`, `-j`, paths |
| `lint` | `jailint -D warnings` on Jai, `cargo clippy --workspace --all-targets -- -D warnings` on Rust | `--fix`, `--lang`, `--staged`, `-j`, paths |
| `check` | `fmt --check` then `lint`, everything CI's format and lint steps run | `-j` |
| `test` | `cargo test` (CI's skips) and the `tools/` unittests | `--suite cargo\|tools\|all`, `--crate NAME`, `-j`, test-name filter |
| `hooks` | `git config core.hooksPath .githooks` ([pre-commit hook](pre-commit-hook.md)) | |
| `fetch-upstreams` | `tools/fetch_upstreams.py` ([upstream corpus](upstream-corpus.md)) | arguments pass through |
| `wasm` | `tools/build_scripting_wasm.py --release` into `artifacts/wasm` | arguments pass through |
| `vscode` | `pnpm run <script>` in `editors/vscode` (default `lint`) | script name |

`fmt`, `lint`, `build` and `test` run `tools/dev.py`, which finds or builds the tools: `jaifmt` is rebuilt when `jaic` or its sources are newer, `jailint` through `cargo build -p jailint`. With no paths the whole tree is processed (the directories CI uses). Paths narrow it to files or directories. `--staged` takes the staged files from `tools/staged-files.sh`, the script the pre-commit hook also uses; clippy cannot work on single files, so for Rust it only decides whether the workspace is linted.

`dev.py` runs on Python 3.9 (macOS `/usr/bin/python3`). `fmt --check` over all Rust runs `tools/check_rust_format.py` (which needs 3.11 for `tomllib`) when it can, and plain `cargo fmt --check` plus item spacing otherwise.

Examples:

```sh
just fmt                      # format everything
just fmt --check --lang jai   # what CI's Jai format step runs
just lint --staged --fix      # fix lints in staged files
just test --crate jaic-cli    # one crate
just test --suite tools       # only the tools/ unittests
```

## How to change it

Add a recipe with a doc comment (it becomes the `just --list` text) and a `[group(...)]`. Keep the body to one command; put logic in `tools/dev.py` or a script under `tools/`. Options come from `[arg("name", long)]`; a flag is `[arg("check", long, value="true")]` on a parameter defaulting to `"false"`. Add a recipe CI should run to the workflow instead of repeating its commands.

Gotchas: recipe options need just 1.58 (CI pins `just-version`); variadic `*paths` are joined by spaces, so paths with spaces are not supported; the `vscode` recipe only names package scripts, so the scripts can change without touching the Justfile. The recipes are tested on macOS and Linux; Windows CI does not call them (`windows-shell` is PowerShell, `python` instead of `python3`).

## Configuration

- `JAI_JOBS`: default for `--jobs` (3, which suits a 16 GiB machine; cargo and jailint compile at that width).
- `CARGO_TARGET_DIR` or Cargo's configured target directory decides where `dev.py` looks for binaries.

## Dependencies

[just](https://github.com/casey/just) 1.58+, `python3`, the pinned Rust toolchain, `pnpm` for `vscode`. CI installs just with `extractions/setup-just` (SHA-pinned) in the `compiler` job.
