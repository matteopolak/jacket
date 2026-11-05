# VS Code extension

`editors/vscode/` is the Visual Studio Code extension (ID `matteopolak.jacket`): a TextMate grammar, snippets, a client for [`jailsp`](../compiler/language-server.md) (which also carries [jailint](jailint.md) findings and fixes), [`jaifmt`](jaifmt.md) as the formatter, a test explorer, *Run/Build/Check File* commands, and a downloader for the compiler release named in its checksum manifest. The extension and compiler versions are independent. It ships as a `.vsix` on GitHub releases and on the Marketplace and Open VSX.

## How it works

```
src/extension.ts         activation, LanguageClient, restart, jacket.toml sync
src/discovery.ts         finding jailsp, jaic, jaifmt
src/toolchain.ts         archive per platform, checksums, download, versioned install
src/toolchainManager.ts  prompts, progress, remembered answers
src/commands.ts          Run/Build/Check File as tasks
src/testing.ts           test explorer and Run/Debug Test lens commands
src/testEvents.ts, testTree.ts, testProcess.ts   `jacket test --format json` handling
scripts/build-grammar.ts writes syntaxes/jai.tmLanguage.json
scripts/build.ts schemas writes the jailint, jaifmt and jacket.toml JSON schemas
```

- **Highlighting.** The grammar is generated from the keyword lists at the top of `build-grammar.ts`; `jailsp` semantic tokens refine it. A here-string whose terminator names a language (`#string WGSL ... WGSL`, also GLSL, SQL, JSON, HTML, ...) is highlighted as that language if its grammar is installed.
- **Schemas.** `schemas/jacket.schema.json` is contributed through `tomlValidation`; completion and validation need Even Better TOML (`tamasfe.even-better-toml`).
- **Language client.** Starts `jailsp` over stdio. `JAIC_STDLIB` comes from `jai.stdlib.path`, `JAIC_PRELUDE` from `jai.stdlib.prelude`; entry files, module folders and lint levels from the nearest `jacket.toml`, whose edits apply on save. Tool-path and prelude settings restart the server; live settings update it in place.
- **Binary discovery.** `jailsp`: `jai.server.path`, `PATH`, next to `jaic`, downloaded toolchain. `jaic` and `jaifmt` likewise. A configured path that doesn't exist is an error naming the setting. With `jai.toolchain.preferDownloaded`, the download comes before `PATH`. If the `jailsp` on `PATH` is older than the extension's, it warns once per `path@version`.
- **Toolchain download.** With nothing found, `jai.toolchain.autoDownload` decides. The archive (`RELEASE_ASSETS`) is checked against `toolchain-manifest.json` ([releases](releases.md#toolchain-manifest)), unpacked with system `tar`, and installed under `<globalStorage>/toolchain/<version>/`; old versions are deleted after the new one lands.
- **Formatting.** Format Document, Format Selection and format on save all go to `jailsp`'s formatting providers, which run `jaifmt` (see [the language server](../compiler/language-server.md)). The extension only finds `jaifmt` (`jai.formatter.path`, `PATH`, next to `jaic`, the downloaded toolchain) and sends the path as the server's `formatter.path` setting. A format that fails is logged in the *Jai Language Server* output, not shown as a popup.
- **Test explorer.** Lists with `jacket test --list --format json` (files, tests, `@cases` cases), relists on save, and runs `jacket test --format json --exact NAME...`, mapping events as they stream. A failed check becomes a message at its file and line. Needs a trusted workspace. `jailsp` also puts *Run test* and *Debug test* lenses over each `@test`; *Debug* only shows a message, since there is no Jai debug configuration.
- **Commands.** *Run/Build/Check File* run `jaic <file> [-interp|-check] <jai.run.arguments>` as a task. `source.fixAll.jailint` works with `editor.codeActionsOnSave`.

The pinned language client (11) must release an in-flight diagnostic request when `jailsp` answers a pull with `ServerCancelled` (`retriggerRequest: true`, sent when the check was stopped or did not finish), so the next pull is not blocked. Check this when updating the client dependency.

## Building and testing

```sh
cd editors/vscode
corepack enable && pnpm install
pnpm run build           # grammar, schemas, dist/extension.cjs
pnpm run lint            # oxlint, tsc, generated files up to date
pnpm test                # unit and grammar tests
JAILSP=../../target/debug/jailsp [JAIFMT=...] pnpm run test:e2e
pnpm run package         # jacket-vscode-<version>.vsix
```

To try a change by hand, `just vscode-try [FOLDER]` builds the tools and the extension and opens VS Code in a throwaway profile pointing at the built binaries (and `JAIC_STDLIB` at this checkout; open a new terminal after switching profiles). The setup is `vscode_settings` and `cmd_vscode_try` in `tools/dev/dev.py`; keep its PATH, editor paths and package list in step when adding a tool. Debug builds ship source maps; startup failures log a stack to the Jai output channel.

## How to change it

- **Grammar:** edit `build-grammar.ts`, rebuild, add assertions to `test/grammar/*.jai` (`^` under tested columns). `scripts/tokenize-corpus.ts` fails on a token left open; run it after big changes.
- **A here-string language:** add the tags to `LANGUAGE_TAGS` in `crates/jaic-syntax/src/lexer/here.rs`, regenerate `editors/lexicon.json`, and add it to `EMBEDDED_LANGUAGES` in `build-grammar.ts` (the build fails while they differ).
- **Test explorer:** a new JSON field is read in `parseEvent` and mapped in `outcomeOf` (`testEvents.ts`); layout is `buildTree`; VS Code calls are in `testing.ts`. The lens argument is built in `crates/jai-language-server/src/test_lenses.rs`; keep `LensArgument` in step.
- **A jailint rule:** nothing; the schema regenerates and lint fails while stale. **A `[format]` key:** `scripts/build.ts schemas`. **A release platform:** one row in `RELEASE_ASSETS`.
- **Icons:** `pnpm run icons`. **Screenshots:** `pnpm run screenshots` on macOS.
- **Dependencies:** pnpm is pinned; every npm package must be at least 14 days old ([development](development.md#dependency-policy)). `@types/vscode` must not be newer than `engines.vscode`.

Gotchas:

- Bundling is Rolldown; scripts run as `node scripts/x.ts` (erasable TypeScript only).
- `vsce package` needs `--no-dependencies` and a `LICENSE` beside `package.json` (the package script copies it).
- A store version can't be replaced, so a checksum repin needs a new editor version (`editor_version` input of `release.yml`).

## Configuration

| Setting | Default | Effect |
| --- | --- | --- |
| `jai.server.path`, `jai.compiler.path`, `jai.formatter.path`, `jai.jacket.path` | `""` | tool locations (accept `~`, `${env:NAME}`) |
| `jai.stdlib.path`, `jai.stdlib.prelude` | `""` | `JAIC_STDLIB`, `JAIC_PRELUDE` |
| `jai.toolchain.autoDownload` | `prompt` | `prompt`, `always`, `never` |
| `jai.toolchain.preferDownloaded` | `false` | use the download before `PATH` |
| `jai.completion.autoImport` | `true` | complete unimported names and add the import |
| `jai.completion.callSnippets` | `addParentheses` | `addParentheses`, `fillArguments`, `none` |
| `jai.inlayHints.closingBraceMinLines` | `25` | name closing braces of blocks this long (`0` off) |
| `jai.compileTime.evaluationBudget` | `100000000` | steps one compile-time evaluation may run (`0` no limit); machine-scoped |
| `jai.compileTime.checkTimeLimit` | `10` | seconds a check may run compile-time code (`0` no limit); machine-scoped |
| `jai.compileTime.costLensBlocks` | `1000000` | evaluations this costly show their cost |
| `jai.run.arguments` | `[]` | extra arguments for Run/Build/Check File |
| `jai.tests.codeLens` | `true` | Run/Debug Test lenses |
| `jai.tests.arguments` | `[]` | extra `jacket test` arguments (`--interp`, `--`, `-O2`) |
| `jai.trace.server` | `off` | `messages`/`verbose` log LSP traffic |

In an untrusted workspace the path, download and argument settings are ignored when the workspace sets them, `jailsp` runs without `#run` or build metaprograms, Run/Build/Check refuse, and the test explorer does nothing. See [workspace trust](workspace-trust.md).

CI (`vscode-extension` job): dependency ages, lint, tests, and integration tests under `xvfb-run` with a stand-in formatter. In `release.yml` the `vscode` job packages the VSIX, attaches it to the release and publishes to both stores with `--skip-duplicate` using `VSCE_PAT` and `OVSX_PAT` (steps skip without them). Manual publish: `gh release download <tag> -p 'jacket-vscode-*.vsix'`, then `pnpm exec vsce publish --packagePath ...` and `pnpm exec ovsx publish ... -p "$OVSX_PAT"`.

## Dependencies

VS Code 1.91+, `vscode-languageclient` (bundled), system `tar`, and the toolchain (`jailsp`, `jaic`, `jaifmt`, `jacket`). The `jailsp` protocol is `crates/jai-language-server/src/protocol.rs`.
