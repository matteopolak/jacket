# UI example editor

A native text editor in `examples/editor`, built on the [UI library](ui.md), the [piece-table buffer](text-buffer.md) and [Desktop](desktop.md): tabs, a watched file tree, plain and regex search, native menus and file dialogs, IME input, kerned UI text, and diagnostics, hover, definition, completion and formatting from `jailsp`. Editing, search, saving and formatting keep working when the server can't start. Native only, not in CI. Full key table and internals: [examples/editor/README.md](../../examples/editor/README.md).

```sh
cargo build -j 3 -p jaic-cli -p jai-language-server
jaic examples/editor/main.jai -exe "$T/editor"
"$T/editor" path/to/folder-or-file
```

## How it works

- `editor.jai` holds documents, the tree, search, settings and the `build` procedure; `native.jai` the menu model, dialog outcomes and Save As; `main.jai` picks the native loop or the headless session in `smoke-session.jai`.
- **Language server.** `lsp/module.jai` runs `jailsp` as a child over stdio with byte-counted JSON-RPC frames, a writer thread so a frame never blocks on a pipe, and UTF-8/UTF-16 position conversion. A frame reads at most 32 messages. `language.jai` keeps open documents in sync with incremental changes and drops replies whose document, version or caret moved on. The server starts with `trusted=false`.
- **Menus and dialogs.** File/Edit/View menus send the same keys as the shortcuts, so one `handle_shortcuts` serves both. A cancelled dialog does nothing; an unsupported or failed one (always on Linux) opens the in-app path prompt. Save As writes before the tab takes the new path, refuses a path another tab has open, and reopens the document under its new URI.
- **Config.** `~/.jai-editor.json` (or `JAI_EDITOR_CONFIG`): font size, theme, tab width, line numbers, sidebar width, last root.

## How to change it

- Language features go in `language.jai`: check the version before applying a reply and free the parsed JSON. Framing cases go in `tests/protocol.jai`, live-server cases in `smoke-session.jai`.
- New menu entries go in `editor_menus` with a unique id and a letter or Shift+letter shortcut; the action itself goes in `handle_shortcuts`.
- Platform work belongs in `stdlib/Extensions/Desktop`, not here.

## Configuration

- `JAI_EDITOR_SERVER`: path to `jailsp`, or `off`. Defaults to `jailsp` on `PATH`. `JAIC_STDLIB` is passed through to it.
- `JAI_UI_FONT`: the proportional face. Helvetica may have no kerning pairs; `/System/Library/Fonts/Supplemental/Arial.ttf` shows them.
- Smoke: `T=<build dir> sh examples/editor/tools/smoke.sh`. `EDITOR_REQUIRE_LSP=1` fails when `jailsp` is missing instead of skipping the live checks; `EDITOR_SMOKE_GPU=1` adds a screenshot.

## Tests

```sh
(cd examples/editor && jacket test -j 3)
T=$TMPDIR/editor sh examples/editor/tools/smoke.sh
```

Unit tests cover the protocol, menu model, Save As paths and synthetic composition without a window. The smoke runs the built binary's `--smoke`: editing, search, clipboard, config, menu routing, dialog fallback, Save As, composition and undo, real-font kerning, and the live language features.
