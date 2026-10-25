# Saved editor configuration

## What it is

The language server applies `jacket.toml` on save. Unfinished typing neither changes active settings nor produces configuration warnings.

## How it works

`Session` stores the editable document separately from its saved snapshot. Project discovery, completion, linting and formatting all read that snapshot. Opening a disk-backed file uses its disk contents as the baseline; virtual files use their initial contents. `textDocument/didSave` commits the current buffer, invalidates dependent analysis and refreshes diagnostics. The VS Code extension forwards saves for TOML buffers as well as Jai source files. Embedded clients call `Session::save` or send the same notification.

An invalid saved configuration emits one warning per distinct saved problem and uses the existing defaults fallback. Saving a repair resets the warning state. Closing without saving discards pending edits; watched-file events read disk state rather than the editor buffer, so save and watcher notifications do not produce duplicate warnings. Closing a virtual file removes its session-only configuration.

## How to change it

Keep config readers routed through the saved snapshot in `config::SessionFiles` and `auto_import::Sources`. Update both readers when changing lifecycle behavior. Save synchronization is advertised by the shared JSON-RPC server, including its browser build. Source document synchronization remains incremental.

## Configuration

Applies to `jacket.toml`. Nearest-file lookup, defaults and validation remain in `jacket-config`. Editors must send save notifications; there is no debounce timer or per-keystroke application.

## Dependencies

Uses the existing session document model, `jacket-config`, semantic caches and lint/format configuration readers. No additional dependencies are required.
