# Continuous integration

## What it is

`.github/workflows/ci.yml` checks every push and pull request; `.github/workflows/browser-release.yml` builds and checks the browser release bundle.

## How it works

`ci.yml` has two kinds of job:

- `scripting-wasm` (ubuntu): dependency-age and include checks, `cargo test -p jai-language-server -p jai-wasm`, a `wasm32-unknown-unknown` build of `jai-wasm`, then `node tools/check_scripting_wasm.mjs` executes the real module and `node tools/check_jai_format_wasm.mjs` formats the Jai_Format golden cases through it ([jaifmt](jaifmt.md#browser-playground)).
- `test` (macOS arm64/x86-64, Linux x86-64/arm64): installs the pinned nightly and LLVM 22 ([llvm-setup](llvm-setup.md)), runs `tools/check_dependency_age.py` and `tools/check_ci_sources.py` (literal `include_str!`/`include_bytes!` inputs must exist in the checkout), then format ([code-formatting](code-formatting.md)), `cargo clippy -D warnings`, `cargo test --workspace --no-fail-fast`, and the Python tool tests (`python3 -m unittest discover -s tools -p 'test_*.py'`). Each check records its outcome independently and a final step fails the job if any failed, so one run reports every failure. A `jai-format` step then builds [jaifmt](jaifmt.md) with the debug `jaic` and runs `jaifmt --check` over the Jai trees; it is advisory (not in the enforcement step) until the repository's Jai code has been formatted.

The corpus sweep (`tools/jaic-sweep.py`) is not part of CI because the upstream corpus is fetched, not committed; run it locally ([jaic-sweep](jaic-sweep.md)).

## How to change it

Edit the workflow files. Keep the `format`/`lint`/`correctness`/`policy` step ids paired with the final enforcement step's `steps.<id>.outcome` checks, otherwise failures are silently ignored. Pin third-party actions by commit SHA and keep `permissions: contents: read`. A Python or Node test under `tools/` named `test_*` is picked up by the policy step (Python) or must be added to `browser-release.yml` (Node).

## Configuration

Runners: `macos-15`, `macos-15-intel`, `ubuntu-24.04`, `ubuntu-24.04-arm`; 30 minute timeouts; `CARGO_INCREMENTAL=0`; toolchain `nightly-2026-08-29`; Python 3.14.

## Dependencies

GitHub Actions, rustup, LLVM 22, Node (for the wasm check) and the scripts under `tools/` named above. See [browser compiler releases](../browser/compiler-releases.md) for the other workflow.
