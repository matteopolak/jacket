# Neovim plugin

## What it is

`editors/nvim/` is the Neovim 0.11+ plugin for Jai: an `lsp/jailsp.lua` config for [`jailsp`](../compiler/language-server.md), `jaifmt` formatting, a Vim syntax file, a `:Jai` command and a toolchain downloader. It mirrors the [VS Code extension](vscode-extension.md). User install steps and settings are in [`editors/nvim/README.md`](../../editors/nvim/README.md).

## How it works

- `lsp/jailsp.lua` sets `cmd` to a function so the binary is looked up when a Jai buffer needs it. `plugin/jai.lua` adds `:Jai` and calls `vim.lsp.enable('jailsp')` unless `lsp.enable = false`. `ftdetect/` and `ftplugin/` handle `*.jai` and format on save; `syntax/jai.vim` is the fallback under semantic tokens.
- `lua/jai/`: `config.lua` (defaults < `vim.g.jai` < `setup()`; `JAIC_STDLIB` comes from `stdlib.path`), `toolchain.lua` (find tools: setting, `PATH`, next to `jaic`, `stdpath('data')/jai/<version>`), `install.lua` (archive per platform, SHA-256, `curl` download, `tar` unpack, rename into place, older versions removed; nothing left behind on failure; Windows uses `System32\tar.exe`), `format.lua` (`jaifmt --stdin` in the file's folder, `jaifmt.toml` ignore globs, line-wise edits via `vim.text.diff`), `commands.lua` (`:Jai` subcommands), `health.lua`, and `version.lua` / `checksums.lua` (stamped by the release, so checksums come from the tag, not the download site).
- Subcommands: `fix` asks for `source.fixAll.jailint`; `lint` runs `jailint --color never` and fills quickfix; `expand` sends `jai/expansion`; `polymorphs` runs `jai.showPolymorphs` with a `{uri, position}` argument.

## How to change it

- **Workspace trust:** `trusted` (default `true`) is sent to `jailsp` and gates `:Jai run/build/check`; see [workspace trust](workspace-trust.md).
- **Setting:** add it to the defaults and `jai.Config` annotation in `config.lua` and the README table; if `jailsp` should receive it, add it to `config.server_settings()` after the server reads it (`JsonSession::configure` in `protocol.rs`).
- **`:Jai` subcommand:** add to `subcommands` in `commands.lua`. **Platform:** a row in `install.asset_for` and the VS Code `RELEASE_ASSETS`. **Archive layout** (one top folder with the four executables and `stdlib/`): change it here, in the install scripts and in VS Code together.
- **Syntax words** in `syntax/jai.vim` copy `editors/vscode/scripts/build-grammar.ts`; nothing checks them against each other.
- Gotchas: `vim.lsp.enable()` does not attach to already open buffers after startup, so `restart` re-triggers `FileType`; use `vim.uv`, never `vim.loop`.

Tests are plain-Lua specs in `editors/nvim/tests/` (`filetype`, `toolchain`, `install`, `format`, `server`):

```sh
just nvim-test            # builds a debug jailsp, runs everything
just nvim-test --unit     # without the end-to-end spec (what `just check` runs)
```

`JAILSP` names the server binary for the end-to-end spec. CI's `neovim-plugin` job installs a pinned Neovim and runs `just nvim-test`.

## Publishing

Plugin managers need a repository whose root is the plugin, so the `neovim` job in `release.yml` copies `editors/nvim/` to `matteopolak/jacket.nvim` (`main`) and tags it with the editor version. Existing tags are never moved: an identical rerun is fine, different content under a tag fails and needs a new `editor_version`. Setup once: create the empty repository, generate an ed25519 key, add the public key as a write deploy key there, and store the private key as the `NVIM_MIRROR_DEPLOY_KEY` secret here (without it the job skips). A run without a tag stamps and stops.

## Dependencies

Neovim 0.11+ (`vim.lsp.config`, `vim.lsp.enable`, `vim.text.diff`, `vim.fs.relpath`), `curl` and `tar` for `:Jai install`, `sha256sum`/`shasum`/`openssl` on Unix, the toolchain binaries. No Lua dependencies.
