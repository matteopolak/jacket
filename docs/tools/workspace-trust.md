# Workspace trust

## What it is

Opening a folder in an editor must not run that folder's code. Jai runs code at compile time (`#run`, build metaprograms that call `compiler_create_workspace`, computed `#insert`, `#modify`), and the language server compiles the open project to check it, so an editor integration needs a switch: `jailsp` has a `trusted` mode (default on) and an untrusted mode that never starts compile-time execution.

## How it works

**One choke point.** Every way of running compile-time code in the compiler ends in `Compiler::call_thunk` (`crates/jaic/src/sema/consteval.rs`): `#run` bodies, computed `#insert` strings, `#modify` blocks and the `build()` of a metaprogram all become a thunk that `call_thunk` executes. When `IdeFacts::untrusted` is set (only the language server sets it, from `semantic::compile`), `call_thunk` returns an error of kind `DiagnosticKind::Untrusted` before any interpreter state is touched. Constant folding that does not need the interpreter (`#if` on literals, `size_of`, ...) is unaffected.

**What the server does with the refusal.**

- `semantic::known` drops `Untrusted` errors, so a `#run` result is *unknown*, never an error. A file that has a refused `#run` and an `#insert` also drops "unknown identifier" errors, because the inserted code could have declared any name.
- A statement that needed a refused value (`local := ANSWER + 1;` with `ANSWER :: #run f();`) is salvaged the way sketches of uncalled polymorphs are (`stmt.rs`): its names stay declared with a stand-in type, so the following statements do not cascade into "unknown identifier" errors. Hover on such a name shows `?`.
- A file a build metaprogram compiles (`add_build_file("src/main.jai", w)` in `first.jai`) is normally checked through the program the metaprogram builds. Untrusted, that program is not built; `Session::compile_inputs` compiles the file as its own root instead (`auto_import.rs: is_build_metaprogram` finds the case from the static `add_build_file` literal scan). Names that only the metaprogram defines (`add_build_string`) are then unknown in it, and reported as such.
- Parsing, type checking of everything that does not depend on compile-time results, hover, go to definition, references, completion, formatting and lints all work as before. Import folders still come from the static discovery (`declared_import_paths`, `jai.toml`).

**Switching.** `Session::set_trusted` clears the semantic compile cache (the compiles were made under the other setting) and bumps the generation; the JSON layer then republishes diagnostics (and sends `workspace/diagnostic/refresh` to pull clients), so open files re-analyse at once.

**Trusted mode still has a budget.** Trusted compile-time code runs in the editor's `SandboxHost` with `BLOCK_BUDGET` interpreter blocks per analysis, no real file or process access beyond what the sandbox provides, and output sent nowhere. Trust removes the refusal, not that sandbox.

## How each editor sets it

| Client | Starts untrusted | Switches later |
| --- | --- | --- |
| any | `jailsp --untrusted`, or `initializationOptions: {"trusted": false}` | notification `jai/setTrusted` with `{"trusted": true}`, or `workspace/didChangeConfiguration` with `settings.jai.trusted` |
| VS Code | automatic: `trusted: vscode.workspace.isTrusted` in the initialization options | `onDidGrantWorkspaceTrust` restarts the client, which starts trusted from the settings now in force |
| Neovim | `trusted = false` in `setup()` or `vim.g.jai` | `require('jai').setup({ trusted = true })` sends `workspace/didChangeConfiguration` |

`--untrusted` is a floor: `initializationOptions.trusted = true` cannot lift it. Only `jai/setTrusted` or a configuration change can.

### VS Code

The manifest declares `capabilities.untrustedWorkspaces` as `limited`, with `restrictedConfigurations` listing every setting that names a program or what it runs, so workspace values are ignored in an untrusted workspace (the configuration API then returns the user value or default):

`jai.server.path`, `jai.compiler.path`, `jai.formatter.path`, `jai.stdlib.path` (passed as `JAIC_STDLIB`, so it chooses the code jailsp loads), `jai.toolchain.autoDownload`, `jai.run.arguments`.

Also, in an untrusted workspace: *Run/Build/Check File* refuse (they execute compile-time code), and `autoDownload: always` behaves as `prompt`. Formatting keeps working with `jaifmt` from the user's settings or `PATH`. The toolchain download source is not a setting: it is the pinned GitHub release with pinned checksums.

`test/unit/manifest.test.ts` fails if a new `*.path` setting is not restricted.

### Neovim

`trusted` (default `true`) is part of `config.server_settings()`; with `false` the `:Jai run`, `build` and `check` commands also refuse.

## How to change it

- A new way to run compile-time code in the compiler must go through `call_thunk` (or call `ide.untrusted` itself); do not add a second interpreter entry point.
- A new editor client: send `initializationOptions.trusted` from its host's trust state and `jai/setTrusted` on a grant.
- A new setting that names an executable, a library or a download source: add it to `restrictedConfigurations`.
- Tests: `crates/jai-language-server/tests/workspace_trust.rs` (a `#run` constant, computed `#insert`, a build metaprogram, the protocol switches), `editors/vscode/test/unit/manifest.test.ts`, `editors/nvim/tests/config_spec.lua`.

## Configuration

- `jailsp --untrusted`; `initializationOptions.trusted`; `jai/setTrusted`; `settings.jai.trusted`.
- VS Code `capabilities.untrustedWorkspaces` in `editors/vscode/package.json`; Neovim `trusted`.

## Dependencies

`jaic` (`sema/consteval.rs`, `sema/ide.rs`, `sema/stmt.rs`, `source.rs`), `crates/jai-language-server` (`semantic.rs`, `session.rs`, `protocol.rs`, `main.rs`), VS Code's workspace trust API (1.57+).
