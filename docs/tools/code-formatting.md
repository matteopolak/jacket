# Code formatting

## What it is

Rust code is formatted with the pinned nightly `rustfmt` and the shared `rustfmt.toml`. `tools/check_rust_format.py` is the CI entry point. Jai code is formatted with `jaifmt` and the root `jaifmt.toml`; see [jaifmt](jaifmt.md).

## How it works

`python3 tools/check_rust_format.py` first checks that every package manifest under `crates/` is a member of the root workspace (an unregistered crate would silently escape formatting), then runs `cargo fmt --all -- --check` and `tools/rust_item_spacing.py --check`. rustfmt does not insert blank lines between items; the spacing script puts one before and after every multi-line item (`python3 tools/rust_item_spacing.py` fixes them). Packages that live outside the workspace would be listed in `STANDALONE_MANIFESTS` in the script; it is empty today.

CI runs the Jai check on every runner too: it builds `jaic`, compiles `tools/jaifmt/main.jai` with `-O2` to `target/jaifmt`, and runs `target/jaifmt --check prelude stdlib tests benchmarks tools examples`. Exit 1 lists each file with the first line that would change, and "Enforce all recorded checks" fails the job. Format with `target/jaifmt prelude stdlib tests benchmarks tools examples` before committing. That is every Jai file the repository owns; `jaifmt.toml` lists the exceptions and why (pinned corpus fixtures, the formatter's golden inputs, the debugger test's fixed line numbers, fuzzer seeds).

`rustfmt.toml` expands compact items (`empty_item_single_line`, `fn_single_line` and `struct_lit_single_line` are off, and single-line `if`/`else` and `let`/`else` widths are 0) and keeps up to two consecutive blank lines (`blank_lines_upper_bound = 2`). Blank lines between methods and logical steps are authored, not enforced.

## How to change it

Format with `rustup run nightly-2026-08-29 cargo fmt --all` and `python3 tools/rust_item_spacing.py` before committing. Change style options in `rustfmt.toml` (several need nightly) and land the resulting reformat as its own commit. A new crate must be added to `members` in the root `Cargo.toml`, otherwise the guard fails.

### Format-only commits and blame

Any commit that only reformats code must contain nothing else. That covers a repo-wide `jaifmt` or `cargo fmt` run, or a reformat after a style option changes. Use the message `style: format with jaifmt` (or `style: format with rustfmt`). The commit must also be listed in `.git-blame-ignore-revs` at the repository root, so that `git blame` attributes lines to the commits that actually wrote them. The hash is only known after committing, so add the listing in a second commit:

```text
# .git-blame-ignore-revs
# style: format with jaifmt
<full 40-character hash of the formatting commit>
```

GitHub's blame view reads this file automatically. Locally, opt in once per clone with `git config blame.ignoreRevsFile .git-blame-ignore-revs`. A rebase or squash changes the hash, so add the entry only after the formatting commit has landed on `main`.

## Configuration

`rust-toolchain.toml` pins `nightly-2026-08-29` with the `rustfmt` and `clippy` components. `.git-blame-ignore-revs` lists format-only commits. `rustfmt.toml`: edition and style edition 2024, `max_width = 100`, Unix newlines.

## Dependencies

Python 3.11+ (`tomllib`), Cargo and the pinned toolchain's rustfmt. CI runs the script from [continuous integration](continuous-integration.md).
