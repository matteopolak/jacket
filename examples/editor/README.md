# Jacket editor

A small native editor with tabs, a watched file tree, plain/regex search, native menus and dialogs,
IME composition, proportional UI text, and language features from this repository's `jailsp`.
Editing, saving, search and formatting remain available if the server cannot start.

## Run and configuration

```sh
cargo build -j 3 -p jaic-cli -p jai-language-server
jaic examples/editor/main.jai -exe "$T/editor"
"$T/editor" examples/editor
"$T/editor" path/to/file.jai
"$T/editor" --smoke
```

`Extensions/Args` handles the optional path, `--smoke`, and `--help`. Put `jailsp` on `PATH` or set
`JAI_EDITOR_SERVER` to its absolute path (`off` disables it). `JAIC_STDLIB` is inherited by the
server. `JAI_EDITOR_CONFIG` overrides `~/.jai-editor.json`; settings include font size, tab width,
theme, line numbers and sidebar width. The config path is copied into owned storage across frames.
`JAI_UI_FONT` selects the proportional face; code uses the MONO slot.

## Keys and menus

Use Cmd on macOS and Ctrl on Windows/Linux unless stated otherwise.

| Key | Action |
| --- | --- |
| O / Shift+O | Open file / open folder |
| S / Shift+S | Save / Save As |
| W | Close tab; request twice to discard modified text |
| Z / Shift+Z | Undo / redo |
| X / C / V / A | Cut / copy / paste / select all in the focused field |
| F / L | Toggle find / line numbers |
| Shift+F | Format document as one undoable edit |
| F1 / F12 | Hover at caret / go to definition |
| Alt+Left | Return from definition (one saved location) |
| Ctrl+Space | Open completion choices |
| Escape | Dismiss popup or path prompt |

Diagnostics have squiggles and a clickable list. Completion uses a filter field and clickable
items. Snippets and additional edits report a status message. Formatting uses
`Extensions/Jai_Format` and the nearest `jacket.toml`, including its ignore list.

File/Edit/View native menus route through the same key path as physical shortcuts. macOS uses
NSMenu and NSOpenPanel/NSSavePanel; Windows uses menu/accelerator tables, Unicode common file
dialogs and the shell folder browser. Linux has no native menus or dialogs; shortcuts and the
in-app path prompt work there. Unsupported/failed dialogs open the prompt; cancellation leaves
it closed. The fallback Save As prompt writes directly without overwrite confirmation.

Opening a folder replaces the sidebar and watcher, retaining tabs. Save As writes successfully
before changing identity, rejects another tab's canonical path, and closes/reopens its LSP URI.

## How it works

`editor.jai` owns documents, tree, search, settings and widget declarations. `native.jai` owns the
menu model, dialog outcomes and Save As. `main.jai` starts the native loop or the shared headless
session in `smoke-session.jai`; `smoke.jai` retains the standalone interpreted entry point.

`lsp/module.jai` owns the stdio child, byte-counted JSON-RPC frames, request IDs, cancellation,
URI escaping and UTF-8/UTF-16 conversion. A bounded writer queue/thread keeps writes outside UI
frames. Frames poll both output pipes with timeout zero and parse at most 32 messages. Limits:
8 MiB/frame, 16 MiB queued output, 15 seconds to initialize. Shutdown kills/reaps the child and
joins the writer. `language.jai` synchronizes document lifecycle and incremental changes; replies
must match document, version and caret. The server starts with `trusted=false`.

`Extensions/Desktop` owns native menus, dialogs and opt-in text services. It temporarily filters
AppKit event retrieval during its pump and subclasses the existing content view for
NSTextInputClient. Windows subclasses the window procedure and reads IMM32. Other programs keep
the existing Input/Window_Creation API and behavior. Composition messages carry positions among
ordinary Input events; UI merges both streams in order. Callbacks between pumps are retained in
a pending queue for the next frame. Detach restores the original native hook.

UI keeps preedit outside Text_Buffer, leaving dirty flags and undo unchanged until commit.
Commit replaces the selection as one undo group; cancellation/focus loss clears preedit. The
focused widget supplies candidate-window geometry. FreeType pair adjustments are cached by size
and Unicode pair; measuring, drawing, selection and hit-testing share them. MONO remains unkerned.
This does not provide full script shaping or bidirectional layout.

## How to change it

Add language operations in `language.jai`, checking versions before applying replies and freeing
owned JSON. Keep the Client/Process addresses stable until shutdown. Add framing/coordinate cases
in `tests/protocol.jai` and live-server cases in `smoke-session.jai`; never block on pipes in `build`.

Add unique IDs and ASCII-letter/Shift shortcuts to `editor_menus`. Application shortcuts belong
in `handle_shortcuts`; focused text actions already go through UI. Extend platform integration in
`stdlib/Extensions/Desktop/`, with its tests and `UI/tests/native-menus.jai`. Native resources
belong to the window thread; remove menus and detach text services before destroying the window.
Menu strings are borrowed during creation, dialog paths are owned, and App_Options borrows its
menu model until the loop returns. Do not modify Native_Menu's implementation fields.

Keep preedit outside the document buffer. Text layout changes must update width, offsets,
hit-testing and drawing together, using real fonts in tests. On macOS, Arial exposes a legacy
AV kerning pair; the default Helvetica may return zero. For visual checks:

```sh
JAI_UI_FONT='/System/Library/Fonts/Supplemental/Arial.ttf' "$T/editor" .
```

## Checks and dependencies

```sh
(cd examples/editor && jacket test -j 3)
sh examples/editor/tools/smoke.sh
(cd stdlib/Extensions/Desktop && jacket test -j 3)
(cd stdlib/Extensions/UI && jacket test -j 3)
```

The smoke script builds into `T` (a removed temporary directory when unset) and runs the real binary's `--smoke`:
editing, search, clipboard, config, menu routing/dialog fallback, Save As, composition/undo,
real-font kerning, and live diagnostics/hover/definition/completion/synchronization/formatting.
Absent jailsp skips live checks; `EDITOR_REQUIRE_LSP=1` makes it required for a direct run.
`EDITOR_SMOKE_GPU=1` enables optional screenshot coverage. Native UI appearance and interactive
input methods require a desktop session; headless tests cannot verify them.

Dependencies include Process, Thread, Unicode, file/watcher/clipboard modules,
Extensions/{Args,Json,Jai_Format,Text_Buffer,UI,Desktop}, FreeType, native window libraries and
WebGPU. Tests for protocol, layout, menu models and synthetic composition need no GPU or window.
See [FRICTION.md](FRICTION.md) for limitations.
