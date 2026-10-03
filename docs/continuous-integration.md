# Continuous integration

## What it is

The public compiler workflow checks the repository on native macOS and Linux runners. Formatting, strict Clippy, correctness tests, policy checks and benchmark smoke are separate observations within each runner's job.

## How it works

Each job verifies public source inputs and dependency ages, installs the pinned Rust toolchain and LLVM 22, and checks its actual host architecture. Formatting and Clippy record their original outcomes while subsequent test steps run. The final enforcement step fails the job if either failed, even when correctness tests pass. This keeps both style gates required while collecting compiler and runtime evidence from the same checkout. A formatting-scope error is also retained as a failure; the workflow does not bypass the scope guard.

A correctness-test failure still fails the job and prevents subsequent ordinary steps. Cancellation does not start additional enforcement work. Original reference lexer inputs are absent from public checkouts, so that one source-dependent test is explicitly skipped; protected-tool rejection tests remain active. These jobs execute the repository's compiler and generated test code, never the supplied reference binaries.

## How to change it


Add new target checks with their actual SDK/toolchain requirements. Corpus, native graphics and original-reference execution have separate input and environment requirements; this workflow does not establish their acceptance. Change source/age policy in the owning Python tools and documentation rather than bypassing them in the workflow.

## Configuration

The current matrix uses `macos-15` (ARM64), `macos-15-intel`, `ubuntu-24.04` and `ubuntu-24.04-arm`, with a 30-minute job timeout and `fail-fast: false`. `CARGO_INCREMENTAL=0` controls build storage. The workflow installs `nightly-2026-08-29`, sets `LLVM_SYS_221_PREFIX` per platform, uses `--locked`, and treats Clippy warnings as errors. Benchmark smoke uses `--test` without a timing threshold on shared runners.

## Dependencies

GitHub Actions, the pinned checkout/Python actions, Rust/rustfmt/Clippy, LLVM 22 and the platform package manager. Internal dependencies include `tools/check_dependency_age.py`, `tools/check_ci_sources.py`, `tools/install_ci_llvm_linux.sh`, workspace tests, Python policy tests and `jai-bench`.
