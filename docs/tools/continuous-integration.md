# Continuous integration

## What it is

`.github/workflows/ci.yml` checks every push and pull request; `.github/workflows/browser-release.yml` builds and checks the browser release bundle; `.github/workflows/windows-native.yml` tests native Windows executables. `.github/workflows/compile-bench.yml` runs the [compile-time benchmark](compile-time-benchmark.md) on demand (`workflow_dispatch` only; it never gates a PR).

## How it works

`ci.yml` has three jobs:

- `scripting-wasm` (ubuntu): dependency-age and include checks, `cargo test -p jai-language-server -p jai-wasm`, a release `wasm32-unknown-unknown` build staged by `tools/build_scripting_wasm.py`, then `node tools/check_scripting_wasm.mjs` executes the real module, `node tools/check_jai_format_wasm.mjs` formats the Jai_Format golden cases through it ([jaifmt](jaifmt.md#browser-playground)), and `node tools/check_playground_stdlib.mjs` runs every `tests/stdlib` program in the browser engine; any failure fails the job. The browser release (`tools/package_browser_release.py`, which the portfolio's publish workflow runs) makes the same check before it writes any asset, so a broken bundle is never published. A new stdlib test that needs processes, native libraries or windows must skip that part itself under `OS == .WASM`. Last, it builds an LLVM-free host `jaic` (`--no-default-features`) and runs [differential testing](differential-testing.md) of the interpreter against the browser engine (`--backends interp,wasm corpus gen:1:40`).
- `test` (macOS arm64/x86-64, Linux x86-64/arm64): installs the pinned nightly and LLVM 23 ([llvm-setup](llvm-setup.md)), runs `tools/check_dependency_age.py` and `tools/check_ci_sources.py` (literal `include_str!`/`include_bytes!` inputs must exist in the checkout), then format ([code-formatting](code-formatting.md)), `cargo clippy -D warnings`, `cargo test --workspace --no-fail-fast`, and the Python tool tests (`python3 -m unittest discover -s tools -p 'test_*.py'`). Each check records its outcome independently and a final step fails the job if any failed, so one run reports every failure. A `jai-format` step then builds [jaifmt](jaifmt.md) with the debug `jaic` and runs `jaifmt --check` over the Jai trees. A `jai-lint` step builds [jailint](jailint.md) and runs `jailint -D warnings` over `stdlib`, `examples`, `tools/jaifmt` and `tests/corpus/positive`, with the repository's `jailint.toml`. Any finding fails it. A `backends` step runs [differential testing](differential-testing.md) of the interpreter against `-O0` and `-O2` native builds (`--backends interp,native,native-O2 corpus gen:1:40`). All three are in the enforcement step.
- `sanitizers` (Linux x86-64, macOS arm64): installs LLVM 23 with its compiler-rt sanitizer runtimes (`libclang-rt-23-dev` on Linux, part of Homebrew's `llvm@23`), builds the debug `jaic`, and runs `tools/jaic-sweep.py --sanitize address,undefined corpus stdlib modules` at `-O0` and again at `-O2`: every corpus runtime case, `tests/stdlib` program and stdlib module test is built natively under ASan and the IR-level UBSan checks and must give the interpreter's expected result with no sanitizer report. The sweep exits non-zero on any failure. See [sanitizers](../native/sanitizers.md).

`windows-native.yml` runs on pushes to `wip/windows-native` and `ci/windows-arm64` (and by hand) that touch the compiler, stdlib or tests, once per CPU. x64: a Linux job cross-builds the corpus runtime cases and stdlib test programs with MinGW-w64 (`tools/windows_cross.py build --stdlib`), a `windows-2025` job runs them and compares output and exit codes, and a second `windows-2025` job builds `jaic` against the official LLVM, builds and runs the same programs natively (`--host`), then runs `cargo test --test native`. arm64: the same three jobs with `--cpu arm64` cross builds through llvm-mingw (a pinned release tarball, `LLVM_MINGW_VERSION`, unpacked into the runner's temp directory) and GitHub's `windows-11-arm` runner, with the official `aarch64-pc-windows-msvc` LLVM archive for the native build. See [Windows](../native/windows.md).

`nix.yml` runs on ubuntu and macOS when the flake, `nix/`, `Cargo.lock` or `rust-toolchain.toml` change: it checks `flake.lock` is current, runs `nix flake check` and `nix build`, and smoke-tests the result. See [Nix flake](nix.md).

`fuzz.yml` replays the saved fuzz crash inputs (`fuzz/regressions/`) on every push and pull request, and fuzzes every cargo-fuzz target nightly for 10 minutes each with a cached corpus. See [fuzzing](fuzzing.md).

The full corpus sweep (`tools/jaic-sweep.py` with the `upstream` set) is not part of CI because the upstream corpus is fetched, not committed; run it locally ([jaic-sweep](jaic-sweep.md)).

## How to change it

Edit the workflow files. Keep the `format`/`jai-format`/`jai-lint`/`lint`/`correctness`/`policy` step ids paired with the final enforcement step's `steps.<id>.outcome` checks, otherwise failures are silently ignored. Pin third-party actions by commit SHA and keep `permissions: contents: read`. A Python or Node test under `tools/` named `test_*` is picked up by the policy step (Python) or must be added to `browser-release.yml` (Node).

## Configuration

Runners: `macos-15`, `macos-15-intel`, `ubuntu-24.04`, `ubuntu-24.04-arm` (and `windows-2025`, `windows-11-arm` in `windows-native.yml`); 30 minute timeouts (45 for `sanitizers`); `CARGO_INCREMENTAL=0`; toolchain `nightly-2026-08-29`; Python 3.14.

## Dependencies

GitHub Actions, rustup, LLVM 23, Node (for the wasm check) and the scripts under `tools/` named above. See [browser compiler releases](../browser/compiler-releases.md) for the other workflow.
