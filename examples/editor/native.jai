// The menu model is also a headless fixture. App loop clicks enter the same key path as shortcuts,
// so Edit actions operate on the focused text field, including the search and path prompts.
#import "Extensions/Desktop";

editor_menus := Menu_Item.[
    .{ menu = "File", title = "Open…", action = 1, key = #char "O" },
    .{ menu = "File", title = "Open Folder…", action = 2, key = #char "O", shift = true },
    .{ menu = "File", title = "Save", action = 3, key = #char "S" },
    .{ menu = "File", title = "Save As…", action = 4, key = #char "S", shift = true },
    .{ menu = "File", title = "Close Tab", action = 5, key = #char "W" },
    .{ menu = "Edit", title = "Undo", action = 6, key = #char "Z" },
    .{ menu = "Edit", title = "Redo", action = 7, key = #char "Z", shift = true },
    .{ menu = "Edit", title = "Cut", action = 8, key = #char "X" },
    .{ menu = "Edit", title = "Copy", action = 9, key = #char "C" },
    .{ menu = "Edit", title = "Paste", action = 10, key = #char "V" },
    .{ menu = "Edit", title = "Select All", action = 11, key = #char "A" },
    .{ menu = "View", title = "Find", action = 12, key = #char "F" },
    .{ menu = "View", title = "Toggle Line Numbers", action = 13, key = #char "L" },
];

request_path :: (ed: *Editor, ui: *UI_State, kind: Dialog_Kind) {
    doc := active_document(ed);
    if kind == .SAVE && !doc return;

    name := ifx doc then doc.name else "";
    result: Dialog_Result;

    // Windowless sessions and Linux keep the path prompt. A cancelled native dialog stays closed.
    if ui.native_window result = native_file_dialog(ui.native_window, kind, ed.root, name);

    defer free(result.path);
    apply_dialog_result(ed, kind, result);
}

apply_dialog_result :: (ed: *Editor, kind: Dialog_Kind, result: Dialog_Result) {
    ed.quick_open_open = false;

    if result.status == .ACCEPTED {
        use_selected_path(ed, kind, result.path);
    } else if result.status == .UNSUPPORTED || result.status == .FAILED {
        ed.prompt_kind = kind;
        ed.quick_open_open = true;
        set_contents(*ed.quick_open_path, "");
    }
}

use_selected_path :: (ed: *Editor, kind: Dialog_Kind, path: string) -> bool {
    if !path.count return false;

    full := path;
    if !is_absolute(full) full = tprint("%/%", ed.root, path);

    if kind == .OPEN {
        open_document(ed, full);
        return true;
    }

    if kind == .SAVE return save_document_as(ed, full);

    is_dir, ok := is_directory(full);
    if !ok || !is_dir {
        set_status(ed, "Not a folder");
        return false;
    }

    if ed.watcher_initialized deinit(*ed.watcher);

    ed.watching = false;
    ed.watcher = .{};
    ed.watcher_initialized = init(*ed.watcher, on_file_change, ed);
    if ed.watcher_initialized ed.watching = add_directories(*ed.watcher, full);

    replacement := copy_string(full);

    free(ed.root);
    ed.root = replacement;
    for ed.expanded free(it);

    array_reset(*ed.expanded);
    ed.tree_dirty = true;

    return true;
}

// Change the document identity only after the new file has been written successfully.
save_document_as :: (ed: *Editor, path: string) -> bool {
    doc := active_document(ed);
    if !doc return false;

    full := absolute_path(path);

    for other: ed.docs if other != doc && other.path == full {
        set_status(ed, "That path is already open in another tab");
        return false;
    }

    if !write_entire_file(full, contents(*doc.edit)) {
        set_status(ed, tprint("Could not save %", path));
        return false;
    }

    replacement := copy_string(full);
    name := copy_string(file_name_of(full));

    // Save As changes protocol identity too: retire requests and diagnostics for the old URI.
    if doc.lsp_version >= 0 {
        Lsp.cancel_document(*ed.language, Lsp.uri(doc.path));
        Lsp.send(*ed.language, "textDocument/didClose", tprint("{\"textDocument\":{\"uri\":%}}", Lsp.quote(Lsp.uri(doc.path))));
    }

    doc.lsp_version = -1;
    doc.diagnostic_version = -1;
    free(doc.lsp_text);
    doc.lsp_text = "";
    free_json(*doc.diagnostics);
    array_reset(*doc.diagnostic_marks);
    clear_language_popup(ed);

    free(doc.path);
    free(doc.name);
    doc.path = replacement;
    doc.name = name;
    doc.saved_version = doc.edit.buffer.version;
    doc.close_armed = false;
    ed.tree_dirty = true;
    sync_language(ed);
    if doc.lsp_version >= 0 Lsp.send(*ed.language, "textDocument/didSave", tprint("{\"textDocument\":{\"uri\":%}}", Lsp.quote(Lsp.uri(doc.path))));

    set_status(ed, tprint("Saved %", name));

    return true;
}
