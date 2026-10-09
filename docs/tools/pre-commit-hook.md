# Pre-commit hook

## What it is

`.githooks/pre-commit` is a tracked POSIX `sh` hook that checks the files staged for a commit, so formatting and lint failures show up before CI does. It works on macOS, Linux and Git Bash.

Enable it once per clone with `just hooks` (the same as setting `core.hooksPath` to `.githooks`; the hook itself does not need just).

`git commit --no-verify` skips it for one commit. `git config --unset core.hooksPath` turns it off.

## How it works

The hook lists the staged files that survive the commit with `tools/staged-files.sh` (`--diff-filter=ACMR`, so deletions are skipped; `just fmt|lint --staged` use the same script) and exports their staged contents (`git checkout-index`) into a temporary directory together with `jaifmt.toml`, `rustfmt.toml` and `rust-toolchain.toml`. Formatters therefore see what will be committed, not the working tree, and a partly staged file is checked as staged. No `git stash` is involved, so nothing in the working tree is touched.

| Staged files | Check |
| --- | --- |
| `*.jai` | `jaifmt --check` on the export (honours `jaifmt.toml` `ignore`) |
| `*.jai` | `jailint -D warnings` on the listed files in the working tree |
| `crates/**/*.rs` | `rustfmt --check --edition 2024` on each staged blob (stdin mode, so `mod` children are not followed; `rustfmt.toml` and the pinned nightly from `rust-toolchain.toml` apply) |
| `crates/**/*.rs` | `python3 tools/rust_item_spacing.py --check` on the export |

`jailint` compiles programs and needs the files around the one it checks (loaders, modules, the stdlib), so it reads the working tree. When a Jai file is partly staged the hook says so. A file another program `#load`s is linted through that program, with only the listed files reported (see [jailint](jailint.md)), so passing only the touched file is enough.

Everything is quiet on success. On failure the hook prints each tool's findings and the commands that fix them:

```text
pre-commit: checks failed. Fix, re-stage (git add) and commit again:
  jaifmt <files>
  jailint --fix <files>   (then fix the rest by hand)
  cargo fmt --all
  python3 tools/rust_item_spacing.py <files>
```

The same checks on the working tree: `just fmt --check --staged` and `just lint --staged` ([Justfile](justfile.md)). Clippy is not run by the hook: even `cargo clippy -p <crate>` rebuilds a crate and its dependants and takes minutes. CI runs `cargo clippy --workspace --all-targets -- -D warnings`; run it yourself before pushing larger Rust changes.

### Finding the tools

- `jaifmt`: `target/jaifmt`. When its sources (`jaifmt/`, `stdlib/Extensions/Jai_Format`, `stdlib/Extensions/Args`) are newer, and `target/release|debug/jaic` exists, it is rebuilt with `jaic build jaifmt/main.jai -O2 -o target/jaifmt` (the CI command). Otherwise `jaifmt` on `PATH`.
- `jailint`: `target/release/jailint` or `target/debug/jailint`, whichever is not older than `crates/jailint`, `crates/jaic`, `crates/jaic-cli` or `Cargo.lock`. If both are stale, `cargo build -p jailint` (debug, incremental) runs. Otherwise `jailint` on `PATH`.
- `rustfmt`, `python3`: from `PATH` (rustup picks the pinned nightly).

A tool that cannot be found gets a one-line `pre-commit: warning:` and its check is skipped; the commit is not blocked. A fresh clone needs `cargo build -p jaic-cli -p jailint` once for both Jai tools to be available, after which `jaifmt` builds on demand.

## How to change it

The hook is one file; each tool has a block in it. To add a check, add the file filter near the top and a block that appends to `fixes`. Keep it fast: it runs on every commit.

Both Jai tools take a file list as arguments, print nothing when clean and exit 1 only on findings, so no `--files-from` or stdin mode was needed. A very large commit could hit the argument-length limit; use `--no-verify` and let CI check it.

`rustfmt --check` in stdin mode prints a diff but exits 0, so the hook treats any output as a failure. It checks `crates/` only, matching CI's `cargo fmt --all`.

## Configuration

| Variable | Effect |
| --- | --- |
| `JAI_HOOK_SKIP` | Space-separated checks to skip: `jaifmt jailint rustfmt spacing` |
| `JAI_HOOK_NO_BUILD=1` | Never rebuild a stale tool |
| `JAI_HOOK_VERBOSE=1` | Say which checks run and when a tool is rebuilt |

## Dependencies

`git`, `tools/staged-files.sh`, a POSIX `sh` with `mktemp` and `find`, `python3`, the pinned nightly `rustfmt`, and the repo-built `jaifmt` and `jailint`. The test `python3 -m unittest tools/test_pre_commit_hook.py` (part of CI's `unittest discover -s tools`) runs the hook on a temporary repository with stand-in Jai tools.
