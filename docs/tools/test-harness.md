# Rust test harness and snapshots

## What it is

`crates/test-support` is a dev-only crate (`publish = false`) with the helpers the workspace's Rust tests share: scratch directories, toolchain lookup, child processes, IR assertions, compile and run helpers, language-server sessions, and `insta` snapshots with machine-independent paths. Tests no longer carry their own `common/mod.rs`.

## How it works

| Module | Provides |
| --- | --- |
| `tmp` | `TempDir::new(label)`: a unique, canonical directory removed on drop. Kept when the test panics or `JAIC_KEEP_TEST_TMP` is set. |
| `process` | `succeeded`, `checked`, `success_with_stdout`: run a command and fail with its output. |
| `toolchain` | `command(name)`, `host_cc`, `clang`, `wasm_ld`, `mingw_gcc`, `node`, `require`, `require_all`, `optional`. A missing tool skips the test with a note. |
| `repo` | `root`, `path`, `fixture`, `stdlib`: paths inside the repository. |
| `ir` | `function`, `body`, `assert_contains`, `assert_lacks`, `assert_line`, `assert_function_contains` over textual LLVM IR. |
| `plugin` | A small metaprogram plugin and a program that uses it. |
| `snapshot` | `assert_snapshot!`, `assert_snapshot_in!`, `Redactor`, `assert_generated_file`. |
| `compiler` (feature `compiler`) | Compile a source string or file with `jaic`. |
| `backend` (feature `backend`) | `Variant` (native or wasm64 at O0 or O2), emit, link, run, `execute`, `check`. |
| `lsp` (feature `lsp`) | Language-server sessions, `position`, `uri`, `project`. |

`jaic` and `jaic-llvm` dev-depend on `test-support`, which depends on them only behind features, so there is no cycle. A test inside a crate's `src/` sees a different build of that crate and may use only the feature-less modules.

### Toolchain strictness

`require(name)` fails the test on CI (`CI` set) or when `JAIC_REQUIRE_TOOLCHAIN` names the tool; otherwise it prints a note and the test returns early. `optional(name)` skips unless `JAIC_REQUIRE_TOOLCHAIN` names it. CI sets `JAIC_REQUIRE_TOOLCHAIN: node,wasm-ld` for the WebAssembly jobs.

### Snapshots

Goldens are `insta` snapshots:

- `tests/lint/<rule>/bad.snap` (and `platform/<os>/window.snap`): the lint output for the case's `bad.jai`, written by `assert_snapshot_in!`.
- `crates/jaic/tests/snapshots/ide_facts_golden__<name>.snap`: IDE facts for each fixture.

Paths in snapshot text go through `Redactor`, which rewrites the repository root, relative `../..` forms, Windows `\\?\` prefixes and `\` separators to forward-slash repository-relative paths, so snapshots match on every platform.

`bad.fixed.jai` files and `editors/lexicon.json` are read by other tools, so they stay as files; `assert_generated_file` compares them and rewrites them under the same switch.

## How to change it

- Review changed snapshots: `INSTA_UPDATE=always cargo test -p jailint -p jaic`, then read the diff. `cargo insta review` also works.
- New helper: add it to the module that fits. Keep the API small; a helper used by one test belongs in that test.
- New lint case: add `bad.jai`, run with `INSTA_UPDATE=always`, review `bad.snap`.
- Insta trims trailing newlines, so a snapshot never ends with a blank line.
- `jacket test` goldens (`.stdout`, `.stderr`, `--accept`) belong to that feature and are separate from this workflow.

## Configuration

| Variable | Effect |
| --- | --- |
| `INSTA_UPDATE` | `always` rewrites snapshots and generated files; `no` (the default on CI) only compares. |
| `JAIC_REQUIRE_TOOLCHAIN` | `1`, or a comma list such as `node,wasm-ld`: missing tools fail instead of skipping. |
| `JAIC_TEST_TMPDIR` | Where `TempDir` creates directories (default: the system temp dir). |
| `JAIC_KEEP_TEST_TMP` | Keep scratch directories after the test. |
| `CI` | Makes `require` strict. |

## Dependencies

`insta` (pinned exactly; it must satisfy the dependency-age rule), and optionally `jaic`, `jaic-llvm` and `jai-language-server` behind the features above.
