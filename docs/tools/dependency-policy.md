# Dependency policy

## What it is

Cargo may only select registry crate versions that were published at least 14 days ago, for direct and transitive dependencies alike. `jaic` itself has no external dependencies; the registry crates are `serde`/`serde_json` (language server) and `inkwell`/`llvm-sys` (LLVM backend).

## How it works

`.cargo/config.toml` enables the nightly `min-publish-age` feature (`registry.global-min-publish-age = "14 days"`, resolver `incompatible-publish-age = "deny"`); `rust-toolchain.toml` pins a nightly that supports it. Cargo permits existing lockfile entries even if they are young, so `python3 tools/check_dependency_age.py` independently checks every locked crate against the crates.io API and fails closed on unknown sources, missing metadata, yanked releases and versions younger than 14 days. Internal path crates are skipped.

CI runs the checker before any build (`ci.yml`, `browser-release.yml`).

## How to change it

Add shared dependencies in `[workspace.dependencies]` of the root `Cargo.toml` and pin exact versions (`=x.y.z`, as the existing crates do). Update with the pinned nightly Cargo, then run the checker before building. Never weaken the age policy to resolve a conflict. The checker's logic is covered by `tools/test_dependency_age.py`.

## Configuration

`--lockfile PATH` (default `Cargo.lock`). Policy settings live in `.cargo/config.toml`.

## Dependencies

Python 3.11+ standard library and network access to crates.io's version API (missing metadata is a failure).
