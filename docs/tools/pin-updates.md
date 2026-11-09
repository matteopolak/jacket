# Pin updates

A weekly workflow (`.github/workflows/pin-updates.yml`, also runnable by hand) compares the tool pins with the latest upstream stable release and keeps one issue, "Tool pins are out of date", listing the ones that are behind. Nothing is changed in the repository.

| Pin | Where | Compared with |
| --- | --- | --- |
| rust | `channel` in `rust-toolchain.toml` | the stable channel manifest (`static.rust-lang.org/dist/channel-rust-stable.toml`) |
| llvm | `llvm` in `.github/pins.json` | releases of `llvm/llvm-project` (`llvmorg-X.Y.Z`) |
| just | `just` in `.github/pins.json` | releases of `casey/just` |
| nextest | `nextest` in `.github/pins.json` | releases of `nextest-rs/nextest` (`cargo-nextest-X.Y.Z`) |

The Rust pin is a nightly, so it is compared by date: it is behind when it predates the current stable release. Versions are compared as numbers (`0.9.100` is after `0.9.99`). Prereleases and drafts are ignored, and a release counts only once it is 14 days old, as for [action pins](action-pins.md). A new LLVM major or minor needs more than the pin; see [LLVM setup](llvm-setup.md).

## The issue

Found by a hidden marker in its body. It is created when a pin is behind, edited in place when the list changes (untouched when it does not), reopened if it was closed, and closed when every pin is current. The workflow uses `gh` with the workflow token (`issues: write`).

## Run it locally

```
PYTHONPATH=tools python3 tools/ci/check_pin_updates.py --dry-run
```

Prints each pin that is behind and the issue body, and touches no issue (no `gh` needed; `GITHUB_TOKEN` raises GitHub's rate limit). Without `--dry-run` it needs `gh` and `GITHUB_REPOSITORY`, and fails with an install hint when either is missing. Exit status 3 means an upstream answer could not be read.

## Change it

Add a pin to `RELEASES` in `tools/ci/check_pin_updates.py` (repository, tag prefix, how many releases to read) for a GitHub-released tool. Only the newest few releases of a repository are read (the count is in `RELEASES`). Tests: `tools/ci/test_check_pin_updates.py` (recorded data, no network).
