# VS Code extension

## What it is

`editors/vscode/` is the Visual Studio Code extension (ID `matteopolak.jacket`, AGPL-3.0-or-later): a TextMate grammar, language configuration and snippets, a client for [`jailsp`](../compiler/language-server.md) (which also carries [jailint](jailint.md) findings and fixes), [`jaifmt`](jaifmt.md) as the document formatter, *Run/Build/Check File* commands, and a downloader for the compiler release named in its checksum manifest. The extension version and compiler version are independent, so a checksum repin can ship without a compiler bump. It ships as a `.vsix` on GitHub releases and is published to the Marketplace and Open VSX.

## How it works

```
src/extension.ts         activation, LanguageClient, restart, jailint.toml sync, formatter
src/discovery.ts         finding jailsp, jaic, jaifmt (pure; unit-tested)
src/toolchain.ts         archive per platform, checksums, download, versioned install (pure)
src/toolchainManager.ts  prompts, progress, remembered answers
src/jaifmt.ts            jaifmt.toml lookup, ignore globs, minimal edit, jaifmt --stdin
src/commands.ts          Run/Build/Check File as tasks
src/expansion.ts         jai.showExpansion / jai.showPolymorphs, jai-expansion: documents
scripts/build-grammar.ts writes syntaxes/jai.tmLanguage.json
scripts/build-schemas.ts writes schemas/jailint.schema.json (from jailint's RULES) and jaifmt.schema.json
```

- **Highlighting.** The grammar is generated from the keyword and directive lists at the top of `build-grammar.ts` and written from the compiler's lexer and parser, not another editor's files. It is the fast baseline; `jailsp` semantic tokens refine it. Doc comments get Markdown styling from a hand-written injection grammar (`syntaxes/jai-doc-comments.tmLanguage.json`, tests in `test/grammar/doc-comments.jai`).
- **Embedded languages.** A here-string whose terminator names a language (`#string WGSL ... WGSL`; also GLSL, HLSL, MSL, SQL, JSON, HTML, CSS, XML, Lua, JS/TS, Python, shell, C/C++/ObjC, Rust, YAML, TOML, Markdown, JAI; matched ignoring case) is highlighted as that language. Each language is one rule in front of the plain here-string rule: a `begin` lookahead captures the tag, and a `begin`/`while` rule continues while lines do not start with the tag, so an unclosed construct cannot swallow the file. `package.json`'s `embeddedLanguages` (generated) maps the scopes to VS Code language IDs for comments and brackets. If a language's grammar is not installed, the rule is dropped and the string stays plain. `jailsp` skips known tags in semantic tokens (`crates/jai-language-server/src/here_string.rs`).
- **Language client.** Starts `jailsp` over stdio for `file:` documents. `JAIC_STDLIB` comes from `jai.stdlib.path`; entry files and module folders from the nearest `jai.toml` ([project settings](../compiler/language-server.md#project-settings-jaitoml)); lint levels from `jailint.toml`. Settings go as `initializationOptions` and `workspace/didChangeConfiguration`; a watcher on `**/{*.jai,jai.toml}` keeps the auto-import index current. The extension sends `didOpen/didChange/didClose` for `jailint.toml` and `jai.toml` itself (`SETTINGS_FILES`) so unsaved edits apply at once.
- **Binary discovery.** `jailsp`: `jai.server.path`, `PATH`, next to `jaic`, downloaded toolchain. `jaic`: `jai.compiler.path`, `PATH`, download. `jaifmt`: `jai.formatter.path`, `PATH`, next to `jaic`/`jailsp`, download. A configured path that does not exist is an error naming the setting, never a silent fallback.
- **Toolchain download.** With nothing found, `jai.toolchain.autoDownload` (`prompt`/`always`/`never`) decides. The archive name comes from `RELEASE_ASSETS` (`${platform}-${arch}` to `jacket-<platform>.tar.gz`, `.zip` on Windows). It is checked against `toolchain-checksums.json` (compiler version and SHA-256 of each archive, written at packaging time by `scripts/pin-toolchain.ts`; development builds fall back to the release's `SHA256SUMS`), unpacked with system `tar` into staging, then renamed to `<globalStorage>/toolchain/<version>/`. A cached folder whose recorded hash differs from the current pin is replaced; older versions are deleted only after the new one is installed.
- **Formatting.** `jailsp` has no `textDocument/formatting`, so the extension runs `jaifmt --stdin` in the document's folder, first skipping files the nearest `jaifmt.toml` `ignore` globs match (`--stdin` does not apply them). It returns one edit covering the changed span; on failure it leaves the text alone and shows jaifmt's message.
- **Commands.** *Run/Build/Check File* save and run `jaic <file> [-interp|-check] <jai.run.arguments>` as a task. Refactorings, call hierarchy and expand selection are plain `jailsp` capabilities with no extension code. `source.fixAll.jailint` works with `editor.codeActionsOnSave`.

## How to change it

```sh
cd editors/vscode
corepack enable && pnpm install
pnpm run build           # grammar, schemas, dist/extension.cjs
pnpm run lint            # oxlint, tsc, generated files up to date
pnpm test                # unit and grammar tests
JAILSP=../../target/debug/jailsp [JAIFMT=...] pnpm run test:e2e
pnpm run package         # jacket-vscode-<version>.vsix
```

- **Grammar:** edit `build-grammar.ts`, rebuild, add assertions to `test/grammar/*.jai` (vscode-tmgrammar-test: `^` under tested columns). `scripts/tokenize-corpus.ts` tokenizes `examples/` and `stdlib/` and fails on a token left open; run it on the upstream corpus after big changes.
- **New here-string language:** add to `EMBEDDED_LANGUAGES` in `build-grammar.ts` and the same tags in the same order to `LANGUAGE_TAGS` in `here_string.rs` (the build fails while they differ), and to the playground's table if supported. Avoid common generic terminators.
- **New jailint rule:** nothing; the schema regenerates and lint fails while stale. **New jaifmt.toml key:** `build-schemas.ts`. **New release platform:** one row in `RELEASE_ASSETS` (a unit test compares it with the `release.yml` matrix).
- **Icons:** `pnpm run icons` regenerates SVG/PNG icons from the playground glyph; lint fails when stale. **Screenshots** for the README: `pnpm run screenshots` on macOS (`test/screenshots/`).
- **Dependencies:** pnpm is pinned by `packageManager`. Every npm package must be at least 14 days old ([development](development.md#dependency-policy)); `pnpm-workspace.yaml` sets `minimumReleaseAge` and `scripts/check-dependency-age.ts` checks the lockfile in CI. `@types/vscode` must not be newer than `engines.vscode`.

Gotchas:

- Tooling is Rolldown (`rolldown.config.ts`): `dist/extension.cjs` for the extension; `--environment TESTS` transpiles `src/` and `test/` to `out/**/*.cjs` for tests. Scripts run as `node scripts/x.ts` (erasable TypeScript only).
- `vsce package` needs `--no-dependencies` (everything is bundled) and a `LICENSE` beside `package.json` (the package script copies it).
- Integration tests use a short `--user-data-dir` (macOS socket path limit); `VSCODE_EXECUTABLE` uses an installed VS Code.
- A store version cannot be replaced, so a checksum repin needs a new editor version (`editor_version` input of `release.yml`).
- The extension was `matteopolak.jai` (to 0.4.1) then `matteopolak.jai-toolchain`; `activate` checks `OLD_EXTENSION_IDS` and, if one is installed, registers nothing and offers to uninstall it (duplicate command registration would throw). Drop an ID once few installs remain. The Marketplace requires a globally unique name, which is why it is `jacket`.

## Configuration

| Setting | Default | Effect |
| --- | --- | --- |
| `jai.server.path`, `jai.compiler.path`, `jai.formatter.path` | `""` | tool locations (accept `~`, `${env:NAME}`) |
| `jai.stdlib.path` | `""` | `JAIC_STDLIB` for jailsp and jaic |
| `jai.toolchain.autoDownload` | `prompt` | `prompt`, `always`, `never` |
| `jai.completion.autoImport` | `true` | complete names from unimported modules and add the import |
| `jai.run.arguments` | `[]` | extra arguments for the commands |
| `jai.trace.server` | `off` | `messages`/`verbose` log LSP traffic |

Test environment: `JAILSP`, `JAIFMT`, `VSCODE_EXECUTABLE`.

CI (`vscode-extension` job in `ci.yml`): dependency ages, `pnpm run lint`, `pnpm test`, integration tests under `xvfb-run` with a stand-in formatter (`test/e2e/fake-jaifmt.ts`), and the `.vsix` as an artifact. In `release.yml` the `vscode` job packages the VSIX, attaches it to the release and `SHA256SUMS`, and publishes the same package to both stores with `--skip-duplicate` using secrets `VSCE_PAT` (Azure DevOps token, scope Marketplace Manage, publisher `matteopolak`) and `OVSX_PAT` (Open VSX token, Eclipse publisher agreement signed); without them the steps skip. Manual publish: `gh release download <tag> -p 'jacket-vscode-*.vsix'`, then `pnpm exec vsce publish --packagePath ...` and `pnpm exec ovsx publish ... -p "$OVSX_PAT"`. Deprecating old IDs: Open VSX needs a PR to `open-vsx/publish-extensions` (`extension-control/extensions.json`, `disallowInstall` with replacement `matteopolak.jacket`); on the Marketplace use *Manage Publishers, Deprecate*.

## Dependencies

VS Code 1.91+, `vscode-languageclient` (bundled), system `tar`, and the toolchain (`jailsp`, `jaic`, `jaifmt`). Development: pnpm, TypeScript, Rolldown, oxlint, `@vscode/test-electron`, `vscode-tmgrammar-test`, `@vscode/vsce`, `ovsx`, `@resvg/resvg-js`. Internal: the `jailsp` protocol (`crates/jai-language-server/src/protocol.rs`), jailint's rule table, jaifmt's `--stdin` contract and release archive names.
