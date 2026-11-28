# `jacket.toml` in the editor

`jailsp` checks an open `jacket.toml`, completes its keys and paths, describes them on hover and shows where each dependency is pinned.

## What the editor shows

- **Problems.** Everything `jacket_config::issues` finds: TOML that does not read, a value of the wrong type, a key the schema does not know (with the nearest known one), and what the `[lint]` check rejects. Each is an error on the key or value, source `jacket`, code `config`. They update as the file is typed.
- **Completion.** Table names in a `[header]`, the keys of the table the cursor is in (those already written are left out; a key on an empty line gets ` = `), and the keys of an inline table. Inside a string, paths: `[project] entry` offers `.jai` files, `import-path` and `[test] paths` offer folders, `[package] module` offers folders that hold a `module.jai`, and `[dependencies.*] path` offers folders.
- **Hover.** The description of the key or table under the cursor, from `jacket_config::schema::describe`.
- **Dependency hints.** At the end of each `[dependencies]` entry: `pinned v1.2.0` (the tag of the checked-out submodule commit, else its hash), `v1.4.0 available` (the newest release the entry's range and `min-release-age` accept, when newer than the pin) and `v1.5.0 in 3d` (a newer release still inside the release-age window, with the wait left). A failed lookup shows `can't check for updates` with the reason (`git` or `curl` missing, the remote not answering).

## A file that does not read

A `jacket.toml` with a problem gives no settings. `jailsp` publishes the problem and stops publishing what depends on the settings for the Jai files under it: type errors, unreachable code and lints. Syntax errors stay. Hover, completion and navigation keep working with the entries the folder would infer, as if `[project]` were empty. Fixing the file and saving it brings the diagnostics back. A file that is not open in the editor cannot show its problem, so the server sends it once as a `window/showMessage`.

Edits to an open file update its own diagnostics at once; what the Jai files show waits for the save (or the watcher), as before.

## How it works

- `jacket-config` owns the checks. `issues(text, dir, checks)` returns each problem with its byte span, and `schema::{known_keys, tables, describe, describe_table}` describe the schema. The server adds no validation of its own.
- `manifest.rs` answers the requests; `manifest/lines.rs` reads the text a line at a time, because a file being typed rarely parses.
- `config.rs` loads the settings for the Jai side and returns `Broken` for a file with a problem. `Session::config_broken` is what `add_semantic_diagnostics` and `lint_config` ask.
- `manifest/dependencies.rs` defines `DependencySource`. A request never waits: `lookup` answers from what it has and starts a fetch when it has nothing or the answer is ten minutes old. `manifest/live.rs` runs `jacket_dep::dep::standing::lookup` (git for the remote tags, the forges for release times, the release-time cache of `jacket dep`) on at most four background threads. When one finishes, the server sends `workspace/inlayHint/refresh`.

## How to change it

- **A new key:** add it to the struct, to `known_keys` and to `docs.rs` in `jacket-config`; a test fails until it has a description. Path-valued keys are listed in `path_kind` in `manifest/lines.rs`.
- **A new check:** pass it in `jailint::config::CHECKS`; name the offending key in backticks in its message so the problem lands on it.
- **How "newer" is decided:** `crates/jacket/src/dep/standing.rs`, which shares the release-age gate with `jacket dep update`.

## Configuration

`dependency-hints` (a default feature of `jai-language-server`) builds the live source; `jai-wasm` turns it off because the browser has neither git nor `curl`. `JACKET_FORGE_API`, `GITHUB_TOKEN` and `GITLAB_TOKEN` are read as `jacket dep` reads them. The editors send `jacket.toml` buffers: the VS Code extension through its document selector, Neovim through `filetypes` and `root_dir` in `lsp/jailsp.lua`.

## Dependencies

`jacket-config`, `jailint` (the lint check), and `jacket` as a library (`jacket_dep`: `dep`, `git`, `net`, `prerequisite`).
