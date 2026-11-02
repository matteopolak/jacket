# Rust test harness and snapshots

`crates/test-support` is a dev-only crate with the helpers the workspace's Rust tests share: scratch directories, toolchain lookup, child processes, IR assertions, compile and run helpers, language-server sessions, and `insta` snapshots with machine-independent paths.

## Modules

| Module | Provides |
| --- | --- |
| `tmp` | `TempDir::new(label)`: a unique directory removed on drop (kept when the test panics or `JAIC_KEEP_TEST_TMP` is set). |
| `process` | `succeeded`, `checked`, `success_with_stdout`: run a command, fail with its output. |
| `toolchain` | `command`, `host_cc`, `clang`, `wasm_ld`, `mingw_gcc`, `node`, `require`, `optional`. |
| `repo` | `root`, `path`, `fixture`, `stdlib`. |
| `ir` | assertions over textual LLVM IR. |
| `snapshot` | `assert_snapshot!`, `assert_snapshot_in!`, `Redactor`, `assert_generated_file`. |
| `compiler` (feature) | compile a source string or file with `jaic`. |
| `backend` (feature) | native or wasm64 at O0/O2: emit, link, run. |
| `lsp` (feature) | language-server sessions. |

`jaic` and `jaic-llvm` dev-depend on `test-support`, which depends on them only behind features. A test inside a crate's `src/` sees a different build of that crate and may use only the feature-less modules.

`require(name)` fails the test on CI (or when `JAIC_REQUIRE_TOOLCHAIN` names the tool); otherwise the test skips with a note. `optional(name)` skips unless the variable names it.

## Snapshots

- `tests/lint/<rule>/bad.snap`: lint output for the case's `bad.jai`.
- `crates/jaic/tests/snapshots/ide_facts_golden__<name>.snap`: IDE facts per fixture. Stdlib locations are written as the file alone, so editing the stdlib never rewrites them.

`Redactor` turns absolute and Windows-style paths into repository-relative forward-slash paths. `bad.fixed.jai` files and `editors/lexicon.json` are read by other tools, so `assert_generated_file` compares and rewrites them under the same switch.

## Changing snapshots

- Review: `INSTA_UPDATE=always cargo test -p jailint -p jaic`, then read the diff (or `cargo insta review`).
- New lint case: add `bad.jai`, run with `INSTA_UPDATE=always`, review `bad.snap`.
- A helper used by one test belongs in that test. `crates/jacket/tests/support/installed.rs` holds the scratch directory and unpacked-archive helpers its installed-binary tests share.
- `jacket test` goldens (`--accept`) are a separate workflow.

## Configuration

| Variable | Effect |
| --- | --- |
| `INSTA_UPDATE` | `always` rewrites snapshots; `no` (CI default) only compares. |
| `JAIC_REQUIRE_TOOLCHAIN` | `1` or a list like `node,wasm-ld`: missing tools fail instead of skipping. |
| `JAIC_ABI_CROSS` | `;`-separated `os-cpu[:clang args]` targets whose C headers the stdlib's C layouts are checked against. |
| `JAIC_TEST_TMPDIR` | Where `TempDir` creates directories. |
| `JAIC_KEEP_TEST_TMP` | Keep scratch directories. |
