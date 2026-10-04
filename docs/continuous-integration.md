# Continuous integration

## What it is

The public compiler workflow checks the repository on native macOS and Linux runners. Formatting, strict Clippy, correctness tests, policy checks and benchmark smoke are separate observations within each runner's job.

## How it works

Each job verifies public source inputs and dependency ages, installs the pinned Rust toolchain and LLVM 22, and checks its actual host architecture. Formatting, Clippy, correctness tests, policy tests and benchmark smoke record their original outcomes independently. Cargo uses `--no-fail-fast` to execute all compiled test binaries even when one fails. Later independent steps still run, and the final enforcement step fails the job if any of the five checks failed. This collects actionable failures from one checkout while keeping every check required. A formatting-scope error is also retained as a failure; the workflow does not bypass the scope guard.

A correctness-test failure is retained while policy and benchmark checks continue. A compilation failure can prevent dependent tests from running; their absence is not a pass. Dependency-age and source-input admission failures stop before building. Cancellation does not start additional enforcement work. Original reference lexer inputs are absent from public checkouts, so that one source-dependent test is explicitly skipped; protected-tool rejection tests remain active. These jobs execute the repository's compiler and generated test code, never the supplied reference binaries.

## How to change it


Add new target checks with their actual SDK/toolchain requirements. Corpus, native graphics and original-reference execution have separate input and environment requirements; this workflow does not establish their acceptance. Change source/age policy in the owning Python tools and documentation rather than bypassing them in the workflow.

## Configuration

The current matrix uses `macos-15` (ARM64), `macos-15-intel`, `ubuntu-24.04` and `ubuntu-24.04-arm`, with a 30-minute job timeout and `fail-fast: false`. `CARGO_INCREMENTAL=0` controls build storage. The workflow installs `nightly-2026-08-29`, sets `LLVM_SYS_221_PREFIX` per platform, uses `--locked` and Cargo test `--no-fail-fast`, and treats Clippy warnings as errors. Benchmark smoke uses `--test` without a timing threshold on shared runners.

## Dependencies

GitHub Actions, the pinned checkout/Python actions, Rust/rustfmt/Clippy, LLVM 22 and the platform package manager. Internal dependencies include `tools/check_dependency_age.py`, `tools/check_ci_sources.py`, `tools/install_ci_llvm_linux.sh`, workspace tests, Python policy tests and `jai-bench`.
