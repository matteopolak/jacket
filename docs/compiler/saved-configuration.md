# Saved editor configuration

The language server applies `jacket.toml` on save. Unfinished typing never changes active settings or produces configuration warnings.

## How it works

- `Session` keeps the editable document apart from its saved snapshot. Project discovery, completion, linting and formatting read the snapshot.
- `textDocument/didSave` commits the buffer, invalidates dependent analysis and refreshes diagnostics. The VS Code extension forwards saves of TOML files too. Embedded clients call `Session::save`.
- An invalid saved configuration warns once per distinct problem and falls back to defaults. Saving a fix resets the warning.
- Closing without saving discards edits. Watched-file events read disk, not the buffer, so there are no duplicate warnings.

## How to change it

Route config readers through the saved snapshot (`config::SessionFiles`, `auto_import::Sources`) and update both when the lifecycle changes. Lookup, defaults and validation live in `jacket-config`. There is no debounce and no per-keystroke application.
