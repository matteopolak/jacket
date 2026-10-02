# Dependency policy

## What it is

Cargo must select the newest usable registry versions published at least 14 days earlier. The policy applies to direct and transitive dependencies.

## How it works

`rust-toolchain.toml` pins nightly 2026-08-29, whose Cargo supports `min-publish-age`. `.cargo/config.toml` enables the feature with `registry.global-min-publish-age = "14 days"` and resolver denial. Commit `Cargo.lock` and use `--locked` for repeatable builds.

Cargo's native resolver permits existing lockfile entries even if they are too young, and registries without publication metadata may ignore the filter. Therefore `python3 tools/check_dependency_age.py` independently checks every external locked crate against crates.io publication records and fails closed for unsupported sources, unavailable metadata, young versions or yanked releases. Run it before dependency build scripts execute, and in CI. Never weaken the age policy automatically to resolve a conflict.

The backend uses Inkwell 0.10.0 and llvm-sys 221.1.0 for independently installed LLVM 22. The benchmark crate uses Divan 0.1.21. The current lockfile contains 33 external packages, all checked before compilation. As packages are introduced, select the latest eligible major version, resolve with the native filter, verify the whole lockfile, then record explicit dependency versions in workspace dependencies.

## How to change it

Add shared dependencies at the workspace level. Update with pinned nightly Cargo, inspect the resulting lockfile and run the publication-date checker before building. A stable Cargo invocation can ignore unstable settings and is not an approved dependency-resolution path. A new dependency must have an identified need; do not add packages merely to populate the graph.

## Configuration

Policy settings live in `.cargo/config.toml`. `--lockfile` selects the lockfile to verify. Tests run with `python3 -m unittest discover -s tools -p 'test_*.py'`. Offline metadata absence is a failure for external packages; the zero-external-package graph requires no network access.

## Dependencies

