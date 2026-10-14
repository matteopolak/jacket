# Neovim plugin

## What it is

`editors/nvim/` is the Neovim plugin for Jai (Neovim 0.11 or newer): an `lsp/jailsp.lua` configuration for [`jailsp`](../compiler/language-server.md), `jaifmt` formatting, a Vim syntax file, a `:Jai` command and a toolchain downloader. It is the counterpart of the [VS Code extension](vscode-extension.md) and follows its settings. User-facing install steps and settings are in [`editors/nvim/README.md`](../../editors/nvim/README.md); this page is for changing the plugin.

## How it works

```
lsp/jailsp.lua        the vim.lsp.config('jailsp') defaults: cmd, filetypes, root markers, settings
plugin/jai.lua        :Jai, the formatSpecifier highlight, vim.lsp.enable('jailsp')
ftdetect/jai.lua      registers *.jai (Neovim up to 0.12 has no rule)
ftplugin/jai.lua      commentstring, four-space indent (jaifmt's default), format on save
syntax/jai.vim        fallback highlighting, under the semantic tokens
lua/jai/init.lua      setup(), the server `cmd`, the download offer, install, restart
lua/jai/config.lua    defaults < vim.g.jai < setup(); the settings jailsp reads
lua/jai/toolchain.lua finding jailsp, jaic, jaifmt and jailint; installed versions
lua/jai/install.lua   archive name per platform, SHA-256, download, unpack (curl and tar)
lua/jai/format.lua    jaifmt --stdin, jaifmt.toml ignore globs, line-wise edits
lua/jai/commands.lua  :Jai subcommands
lua/jai/health.lua    :checkhealth jai
lua/jai/version.lua   the toolchain version to download (stamped by the release)
lua/jai/checksums.lua the release archives' SHA-256 (stamped by the release)
tests/                headless specs
```

**Starting the server.** `lsp/jailsp.lua` sets `cmd` to a function, so the binary is looked up when a Jai buffer needs the server, not when the file loads. `plugin/jai.lua` calls `vim.lsp.enable('jailsp')` unless `lsp.enable = false`. Without the plugin's `lua/` directory the file does nothing useful; users who want only the server copy the snippet from the README. `JAIC_STDLIB` (from `stdlib.path`) goes into the server's environment. Settings reach `jailsp` as `initializationOptions` and as `settings.jai` (`before_init`), and `setup()` sends `workspace/didChangeConfiguration` to running servers; the only one the server reads is `completion.autoImport`.

**Finding tools** (`toolchain.find`): the setting, `PATH` (`vim.fn.exepath`), next to `jaic`, then `stdpath('data')/jai/<version>` (the wanted version, else the newest installed). A configured path that does not exist is an error. The order is the VS Code extension's.

**Installing** (`install.install`, run by `:Jai install` and, after a prompt, when no `jailsp` exists): pick `jai-<macos|linux|windows>-<arm64|x64>.<tar.gz|zip>` from `uv.os_uname()`; get the expected SHA-256 from `lua/jai/checksums.lua` (written into the published plugin by the release, so it comes from the tag, not from the download site) or, in a development checkout, from the release's `SHA256SUMS`; download with `curl` into `.staging-*`; hash with `sha256sum`, `shasum` or `openssl` (`certutil` on Windows); unpack with `tar --strip-components=1`; require all four executables; rename into `<data>/jai/<version>` and remove older versions. Nothing is left behind on failure. Windows uses `%SystemRoot%\System32\tar.exe`, which reads zip files, so Git's GNU `tar` is never picked up. Archives are always named `jai-<platform>` (releases up to 0.4.0 used `jaic-`); the plugin does not support those.

**Formatting.** `jailsp` has no `textDocument/formatting`, so `lua/jai/format.lua` runs `jaifmt --stdin` in the file's folder (so `jaifmt.toml` applies), skips files that the config's `ignore` globs cover (the VS Code extension does the same, because `--stdin` has no file name), and applies the result as changed lines (`vim.text.diff`), which keeps marks and folds elsewhere. If `jaifmt` rejects the text (unbalanced brackets), the buffer stays as it is and the reason is shown.

**Commands** are in `lua/jai/commands.lua`, one table entry each. `fix` asks for the `source.fixAll.jailint` code action; `lint` runs `jailint --color never` over the project root and parses its `--> file:line:col` lines into the quickfix list; `expand` sends `jai/expansion` and shows the text in a scratch buffer (the server's `jai-expansion:` document, without VS Code's content provider); `polymorphs` runs `workspace/executeCommand` `jai.showPolymorphs` with a `{uri, position}` argument (not a text document identifier).

## How to change it

- **A new setting**: add it to the defaults and the `jai.Config` annotation in `config.lua`, use it where needed, and add a row to the README table. If `jailsp` should receive it, add it to `config.server_settings()` after the server reads it (`JsonSession::configure` in `crates/jai-language-server/src/protocol.rs`).
- **A new `:Jai` subcommand**: add it to `subcommands` in `commands.lua`; completion and the help message follow.
- **A new platform**: add a row to `install.asset_for` when `release.yml` builds it, and to the VS Code extension's `RELEASE_ASSETS` too.
- **The archive layout**: `install.install` assumes one top folder holding the four executables and `stdlib/`. If `release.yml` changes that, change it here, in the install scripts and in the VS Code extension together.
- **Syntax words**: `syntax/jai.vim` copies the keyword and directive lists of `editors/vscode/scripts/build-grammar.ts`. Nothing checks them against each other, so update both.
- Gotchas: `vim.lsp.enable()` attaches to already open buffers only after startup, so `restart` re-triggers `FileType` itself; `lsp/*.lua` is resolved lazily, so a `setup()` later than startup still applies; keep `vim.uv`, never `vim.loop`.

## Configuration

Settings are listed in the [README](../../editors/nvim/README.md#settings). The `vim.g.jai` table and `require('jai').setup()` take the same nested keys.

Environment: `JAILSP` (tests only) names the `jailsp` for the end-to-end spec; `JAIC_STDLIB` is set for the server from `stdlib.path`.

### Tests

`editors/nvim/tests/` holds plain-Lua specs run by `tests/run.lua` with a tiny harness (`harness.lua`), no dependencies:

```sh
cd editors/nvim && JAILSP=target/debug/jailsp nvim -n --headless --clean -u tests/minimal_init.lua -c 'luafile tests/run.lua'
just nvim-test          # builds a debug jailsp, then runs everything
just nvim-test --unit   # without the end-to-end spec (what `just check` runs)
```

The test runner disables swap files (`-n`) so isolated temporary buffers do not depend on the user's swap directory.

| Spec | Covers |
| --- | --- |
| `filetype_spec` | `*.jai` detection, ftplugin options, syntax groups (nested comments, here-strings) |
| `toolchain_spec` | lookup order and the resolved `cmd`, with shell scripts standing in for tools |
| `install_spec` | `SHA256SUMS` parsing, hash output parsing, verification; a full install from a fake `file://` release (right and wrong checksum, missing tool, cleanup) |
| `format_spec` | ignore globs, line-wise edits, `format_on_save`, quickfix parsing; a stand-in `jaifmt` |
| `server_spec` | with `JAILSP`: attach, diagnostics, hover docs, semantic tokens, go to definition, restart |

Specs that use shell scripts skip on Windows. `just check` runs the unit specs only when `nvim` 0.11+ is installed and says so when it skips. CI (`neovim-plugin` job in `ci.yml`) installs a pinned Neovim release (checksum verified) and runs `just nvim-test`.

### Publishing

`vim.pack` and plugin managers need a repository whose root is the plugin, so the `neovim` job in `release.yml` copies `editors/nvim/` to `matteopolak/jai.nvim` (`main`, replaced each time) and tags it with the editor version. The plugin manifest keeps the compiler version and its archive checksums; the optional `editor_version` workflow input can advance the mirror tag independently. Existing tags are never moved: an identical rerun is allowed, while different content under an existing tag fails and requires a new editor version. The installer records the verified archive hash and will not select a cached same-version toolchain whose hash differs from the new pin.

Setup, once: create the empty repository `matteopolak/jai.nvim`, generate a key pair (`ssh-keygen -t ed25519 -N '' -f nvim-mirror`), add the public key as a deploy key of `jai.nvim` with write access, and store the private key as the `NVIM_MIRROR_DEPLOY_KEY` secret of this repository. Without the secret the job says it skipped. The job checks GitHub's published ed25519 host key fingerprint before pushing. A dry run (workflow started without a tag) stamps and stops.

## Dependencies

Neovim 0.11+ (`vim.lsp.config`, `vim.lsp.enable`, `vim.text.diff`, `vim.fs.relpath`; `vim.pack` is only one way to install). `curl` and `tar` for `:Jai install`; `sha256sum`, `shasum` or `openssl` on Unix. `jailsp`, and `jaifmt`, `jailint` and `jaic` for their commands. No Lua dependencies; it does not need nvim-lspconfig or plenary.
