# UI example editor (`examples/editor`)

## What it is

A small text editor built on the [UI library](ui.md) and the [piece-table buffer](text-buffer.md): a file tree, tabs, syntax colouring, search with regular expressions, a quick-open prompt, a status line and a JSON config. It is a jacket-style example and the library's first user; it is not part of CI.

```
jaic examples/editor/main.jai -interp -- path/to/folder-or-file
```

Shortcuts (Cmd on macOS, Ctrl elsewhere): `S` save, `W` close tab (a modified tab asks twice), `O` open a path, `F` find, `Z`/`Shift+Z` undo and redo, `A`/`C`/`X`/`V` select all and clipboard. Esc closes the find bar and the open prompt.

## How it works

One `Editor` value holds everything and `build(ed)` declares a frame from it (`editor.jai`):

- **Documents.** `Document` owns a `Text_Edit` (buffer, caret, undo history) and the buffer version that is on disk; `is_dirty` compares them. `open_document` reuses a tab for a path that is already open.
- **File tree.** `rebuild_tree` lists the root recursively through `visit_files`, keeping the directories in `ed.expanded` open. A `File_Watcher` marks the tree dirty; `process_watcher` polls it from `build` and `wake_in` schedules the next poll, so an idle editor still sleeps.
- **Search.** `update_search` recomputes the `Text_Mark`s only when the document version, query or mode changed (`search_key`). A plain query uses `find_text`; regex mode uses `Extensions/Regex`. Marks are capped at 5000. `step_match` moves the selection.
- **Highlighting.** `highlight_jai` (`highlight.jai`) colours one visible line at a time; it is a lexical approximation and does not carry a multi-line comment across lines.
- **Config.** `Config` (`config.jai`) is read by `open_editor` and written by `close_editor`. The file is `~/.jai-editor.json`, or `JAI_EDITOR_CONFIG`.

`smoke.jai` drives the same `build` headlessly (`headless`, `click`, `send_text`, `send_key`), then renders one frame on the GPU with system fonts.

## How to change it

- A new shortcut goes in `handle_shortcuts`; a new panel is another `for` block in `build`.
- A new language needs a `Highlight_Proc` (`(line: string, spans: *[..] Highlight_Span)`) chosen per file extension in `editor_pane`.
- Settings are fields of `Config` (`@json(default)`), so a field added there is read with its default when the file lacks it.
- Gotchas: containers in a column need `grow = 1` to give a `text_editor` height; hit testing uses the previous frame, so scripted input needs two `frame` calls before asserting on what a click changed.

## Configuration

`JAI_EDITOR_CONFIG` (config file path), `JAI_UI_FONT` and `JAI_UI_MONO_FONT` (fonts), the `Config` fields `font_size`, `theme` (`dark` or `light`), `tab_width`, `line_numbers`, `sidebar_width`, `last_root`.

## Dependencies

`Extensions/UI`, `Extensions/Text_Buffer`, `Extensions/Regex`, `Extensions/Json`, `Extensions/Environment`, `File_Watcher`, `File`, `File_Utilities`, `Sort`, `Input`. Native only: it needs a window and, for the smoke screenshot, a GPU.

## Testing

`just ui-editor-smoke` runs the scripted session (tree, open, type, save, undo, clipboard, search, tabs, config, one GPU frame). It prints one line per check and exits non-zero on a failure; the GPU step is skipped without an adapter. Notes on rough edges found while building it are in `examples/editor/FRICTION.md`.
