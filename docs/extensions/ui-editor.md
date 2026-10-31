# UI example editor

A small text editor built on the [UI library](ui.md) and the [piece-table buffer](text-buffer.md): a file tree, tabs, syntax colouring, regex search, a quick-open prompt, a status line and a JSON config. It lives in `examples/editor` and is not part of CI. Native only.

```
jaic examples/editor/main.jai -interp -- path/to/folder-or-file
```

Shortcuts (Cmd on macOS, Ctrl elsewhere): `S` save, `W` close tab (a modified tab asks twice), `O` open a path, `F` find, `Z`/`Shift+Z` undo and redo, `A`/`C`/`X`/`V` select all and clipboard. Esc closes the find bar and the open prompt.

## How it works

One `Editor` value holds everything and `build(ed)` (`editor.jai`) declares a frame from it.

- **Documents.** `Document` owns a `Text_Edit` and the buffer version on disk; `is_dirty` compares them. `open_document` reuses a tab for an open path.
- **File tree.** `rebuild_tree` lists the root through `visit_files`, keeping `ed.expanded` directories open. A `File_Watcher` marks it dirty; `process_watcher` polls from `build` and `wake_in` schedules the next poll, so an idle editor still sleeps.
- **Search.** `update_search` recomputes marks only when the document version, query or mode changed. Plain queries use `find_text`, regex mode `Extensions/Regex`. Marks cap at 5000.
- **Highlighting.** `highlight_jai` colours one visible line at a time and doesn't carry a multi-line comment across lines.
- **Config.** `Config` (`config.jai`) is read by `open_editor` and written by `close_editor` at `~/.jai-editor.json` (or `JAI_EDITOR_CONFIG`): `font_size`, `theme`, `tab_width`, `line_numbers`, `sidebar_width`, `last_root`.

`smoke.jai` drives the same `build` headlessly, then renders one frame on the GPU. `just ui-editor-smoke` runs it; the GPU step is skipped without an adapter.

## How to change it

- New shortcut: `handle_shortcuts`. New panel: another `for` block in `build`.
- New language: a `Highlight_Proc` `(line: string, spans: *[..] Highlight_Span)` chosen per extension in `editor_pane`.
- Settings are `Config` fields (`@json(default)`), so a new field reads its default when the file lacks it.
- Gotchas: a column needs `grow = 1` to give a `text_editor` height; scripted input needs two `frame` calls before asserting on what a click changed.
