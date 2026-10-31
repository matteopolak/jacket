# Workspace trust

Opening a folder in an editor must not run that folder's code. Jai runs code at compile time (`#run`, build metaprograms, computed `#insert`, `#modify`) and the language server compiles the open project, so `jailsp` has a `trusted` mode (default on) and an untrusted mode that never starts compile-time execution.

## How it works

**One choke point.** Every way of running compile-time code ends in `Compiler::call_thunk` (`crates/jaic/src/sema/consteval.rs`). When `IdeFacts::untrusted` is set (only the language server sets it), `call_thunk` returns a `DiagnosticKind::Untrusted` error before touching the interpreter. Constant folding that doesn't need the interpreter (`#if` on literals, `size_of`) is unaffected.

**What the server does with the refusal.**

- `semantic::known` drops `Untrusted` errors, so a `#run` result is unknown, never an error. A file with a refused `#run` and an `#insert` also drops "unknown identifier" errors, since the inserted code could declare any name.
- A statement that needed a refused value keeps its names declared with a stand-in type (`stmt.rs`), so later statements don't cascade. Hover on such a name shows `?`.
- A file a build metaprogram compiles is normally checked through the program the metaprogram builds. Untrusted, `Session::compile_inputs` compiles it as its own root instead; names only the metaprogram defines are then reported unknown.
- Parsing, type checking that doesn't depend on compile-time results, hover, go to definition, references, completion, formatting and lints all work as before.

**Switching.** `Session::set_trusted` clears the semantic compile cache and bumps the generation; diagnostics are republished so open files re-analyse at once.

**Trusted mode still has a budget.** Trusted compile-time code runs in the editor's `SandboxHost` with a block budget per analysis and no real file or process access. Trust removes the refusal, not the sandbox. It adds read-only `getenv`, and lets [compile-time services](../extensions/service.md) answer from separate `jaic -interp` workers.

## How each editor sets it

| Client | Starts untrusted | Switches later |
| --- | --- | --- |
| any | `jailsp --untrusted`, or `initializationOptions: {"trusted": false}` | notification `jai/setTrusted` with `{"trusted": true}`, or `workspace/didChangeConfiguration` with `settings.jai.trusted` |
| VS Code | automatic: `trusted: vscode.workspace.isTrusted` | `onDidGrantWorkspaceTrust` restarts the client |
| Neovim | `trusted = false` in `setup()` or `vim.g.jai` | `require('jai').setup({ trusted = true })` |

`--untrusted` is a floor: `initializationOptions.trusted = true` can't lift it.

**VS Code** declares `capabilities.untrustedWorkspaces` as `limited`, with `restrictedConfigurations` listing the settings that name a program or what it runs (`jai.server.path`, `jai.compiler.path`, `jai.formatter.path`, `jai.stdlib.path`, `jai.toolchain.autoDownload`, `jai.run.arguments`), so workspace values are ignored. Run/Build/Check File refuse, the test explorer does nothing, and `autoDownload: always` acts as `prompt`. Formatting keeps working. `test/unit/manifest.test.ts` fails if a new `*.path` setting isn't restricted.

**Neovim**'s `trusted` also makes `:Jai run`, `build` and `check` refuse.

## How to change it

- A new way to run compile-time code must go through `call_thunk` (or check `ide.untrusted` itself); don't add a second interpreter entry point.
- A new editor client sends `initializationOptions.trusted` from its host's trust state and `jai/setTrusted` on a grant.
- A new setting that names an executable, library or download source goes in `restrictedConfigurations`.
- Tests: `crates/jai-language-server/tests/workspace_trust.rs`, `editors/vscode/test/unit/manifest.test.ts`, `editors/nvim/tests/config_spec.lua`.

## Dependencies

`jaic` (`sema/consteval.rs`, `ide.rs`, `sema/stmt.rs`), `crates/jai-language-server` (`semantic.rs`, `session.rs`, `protocol.rs`), VS Code's workspace trust API.
