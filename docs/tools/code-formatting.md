# Code formatting

## What it is

Rust code is formatted with the pinned nightly `rustfmt` and the shared `rustfmt.toml`. `tools/check_rust_format.py` is the CI entry point.

## How it works

`python3 tools/check_rust_format.py` first checks that every package manifest under `crates/` is a member of the root workspace (an unregistered crate would silently escape formatting), then runs `cargo fmt --all -- --check`. Packages that live outside the workspace would be listed in `STANDALONE_MANIFESTS` in the script; it is empty today.

`rustfmt.toml` expands compact items (`empty_item_single_line`, `fn_single_line` and `struct_lit_single_line` are off) and keeps up to two consecutive blank lines (`blank_lines_upper_bound = 2`). Blank lines between methods and logical steps are authored, not enforced.

## How to change it

Format with `rustup run nightly-2026-08-29 cargo fmt --all` before committing. Change style options in `rustfmt.toml` (several need nightly) and land the resulting reformat as its own commit. A new crate must be added to `members` in the root `Cargo.toml`, otherwise the guard fails.

## Configuration

`rust-toolchain.toml` pins `nightly-2026-08-29` with the `rustfmt` and `clippy` components. `rustfmt.toml`: edition and style edition 2024, `max_width = 100`, Unix newlines.

## Dependencies

Python 3.11+ (`tomllib`), Cargo and the pinned toolchain's rustfmt. CI runs the script from [continuous integration](continuous-integration.md).
