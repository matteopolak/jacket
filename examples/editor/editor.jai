// The editor: documents in tabs, a file tree that follows the disk, search with regular
// expressions, a quick-open prompt, keyboard shortcuts and a status line. Everything here is one
// `build` procedure over an `Editor` value; main.jai opens a window for it and smoke.jai drives it
// without one.

#import "Basic";
#import "Input";
#import "File";
#import "File_Utilities";
#import "Sort";
#import "File_Watcher";
#import "Extensions/UI";
#import "Extensions/Json";
#import "Extensions/Environment";
#import "Extensions/Text_Buffer";

Re :: #import "Extensions/Regex";

#load "config.jai";
#load "highlight.jai";

// One open file.
Document :: struct {
    path: string; // empty for a new unsaved document
    name: string;
    edit: Text_Edit;
    saved_version: int; // the buffer version that is on disk
    close_armed: bool; // closing a document with unsaved changes needs a second request
}

// One row of the file tree.
Tree_Entry :: struct {
    path: string;
    name: string;
    depth: int;
    is_directory: bool;
}

// Everything the editor shows and remembers.
Editor :: struct {
    config: Config;
    config_file: string;
    root: string;
    docs: [..] *Document;
    active: int;
    tree: [..] Tree_Entry;
    expanded: [..] string;
    tree_dirty: bool = true;
    watcher: File_Watcher(Editor);
    watching: bool;
    sidebar: float = 240;

    search_open: bool;
    search_query: Text_Edit;
    search_regex: bool;
    search_marks: [..] Text_Mark;
    search_current: int = -1;
    search_error: string;
    search_key: u64; // what the marks were computed for: document, version and query

    quick_open_open: bool;
    quick_open_path: Text_Edit;

    status: string;
}

// Start the editor on `root` (a directory) and open `file` if it is not empty.
open_editor :: (ed: *Editor, root: string, file: string = "") {
    ed.config_file = config_path();
    ed.config = load_config(ed.config_file);
    ed.root = copy_string(root);
    ed.sidebar = ed.config.sidebar_width;
    ed.search_query.single_line = true;
    ed.quick_open_path.single_line = true;

    if init(*ed.watcher, on_file_change, ed) {
        ed.watching = add_directories(*ed.watcher, root);
    }

    if file.count open_document(ed, file);
}

// Save the configuration and release everything.
close_editor :: (ed: *Editor) {
    ed.config.sidebar_width = ed.sidebar;
    ed.config.last_root = ed.root;
    save_config(ed.config, ed.config_file);

    if ed.watching deinit(*ed.watcher);

    for ed.docs close_document_data(it);

    array_free(ed.docs);
    clear_tree(ed);
    array_free(ed.tree);
    for ed.expanded free(it.data);

    array_free(ed.expanded);
    array_free(ed.search_marks);
    free_text_edit(*ed.search_query);
    free_text_edit(*ed.quick_open_path);
    free(ed.root.data);
    free(ed.status.data);
    free(ed.search_error.data);
    free_json(*ed.config);
}

// The document being edited, or null when none is open.
active_document :: (ed: *Editor) -> *Document {
    if ed.active < 0 || ed.active >= ed.docs.count return null;
    return ed.docs[ed.active];
}

// Whether the document has changes that are not on disk.
is_dirty :: (doc: *Document) -> bool {
    return doc.edit.buffer.version != doc.saved_version;
}

// Open `path` in a tab (or switch to it). A path that does not exist opens an empty document that
// is created when saved. Returns the document.
open_document :: (ed: *Editor, path: string) -> *Document {
    full := path;
    if !is_absolute(path) full = tprint("%/%", ed.root, path);

    for doc, i: ed.docs {
        if doc.path == full {
            ed.active = i;
            return doc;
        }
    }

    doc := New(Document);
    doc.path = copy_string(full);
    doc.name = copy_string(file_name_of(full));

    text, ok := read_entire_file(full);

    if ok {
        set_contents(*doc.edit, text);
        free(text.data);
    } else {
        set_status(ed, tprint("New file %", full));
    }

    doc.saved_version = doc.edit.buffer.version;
    array_add(*ed.docs, doc);
    ed.active = ed.docs.count - 1;

    return doc;
}

// Write the active document to disk.
save_document :: (ed: *Editor) -> bool {
    doc := active_document(ed);
    if !doc return false;

    if !doc.path.count {
        set_status(ed, "Open a path with Cmd+O first");
        return false;
    }

    text := contents(*doc.edit);

    if !write_entire_file(doc.path, text) {
        set_status(ed, tprint("Could not save %", doc.path));
        return false;
    }

    doc.saved_version = doc.edit.buffer.version;
    set_status(ed, tprint("Saved %", doc.name));
    ed.tree_dirty = true;

    return true;
}

// Close the active tab; a document with unsaved changes asks for a second request.
close_active :: (ed: *Editor) {
    doc := active_document(ed);
    if !doc return;

    if is_dirty(doc) && !doc.close_armed {
        doc.close_armed = true;
        set_status(ed, tprint("% has unsaved changes: close again to discard them", doc.name));
        return;
    }

    close_document_data(doc);
    array_ordered_remove_by_index(*ed.docs, ed.active);
    if ed.active >= ed.docs.count ed.active = ed.docs.count - 1;

    ed.active = max(ed.active, 0);
}

// Show `text` in the status line.
set_status :: (ed: *Editor, text: string) {
    free(ed.status.data);
    ed.status = copy_string(text);
}

// Draw the whole editor for one frame.
build :: (ed: *Editor) {
    ui := current_ui();

    handle_shortcuts(ed, ui);
    process_watcher(ed, ui);

    if ed.tree_dirty rebuild_tree(ed);

    if ed.config.theme == "light" && ui.theme.background.x < 0.5 set_theme(default_theme(.LIGHT));

    for column() {
        for pane: split(.HORIZONTAL, *ed.sidebar, .{ min = 140 }) {
            if pane == 0 file_tree(ed);
            else editor_pane(ed);
        }
    }

    if ed.quick_open_open quick_open_overlay(ed);
}

// Shortcuts that work wherever the focus is: Cmd or Ctrl with S, W, O, F.
handle_shortcuts :: (ed: *Editor, ui: *UI_State) {
    for event: ui.input.keys {
        if event.kind != .KEY || !primary(event.mods) continue;

        if event.key == {
            case cast(Key_Code) #char "S"; save_document(ed);
            case cast(Key_Code) #char "W"; close_active(ed);
            case cast(Key_Code) #char "O"; ed.quick_open_open = !ed.quick_open_open;
            case cast(Key_Code) #char "F"; ed.search_open = !ed.search_open;
        }
    }
}

// The path prompt that stands in for a native open dialog.
quick_open_overlay :: (ed: *Editor) {
    ui := current_ui();
    t := *ui.theme;

    for column(.{ padding = 60, align = .CENTER }) {
        for panel(.{ padding = 12, gap = 8, width = 520, radius = t.radius, border = 1, border_color = t.outline }) {
            label("Open file (absolute, or relative to the folder)", .{ muted = true });
            _, submitted := text_field(*ed.quick_open_path, .{ placeholder = "path/to/file.jai" });

            if submitted {
                path := contents(*ed.quick_open_path);

                if path.count {
                    open_document(ed, path);
                    ed.quick_open_open = false;
                    set_contents(*ed.quick_open_path, "");
                }
            }

            for key: ui.input.keys {
                if key.kind == .KEY && key.key == .ESCAPE ed.quick_open_open = false;
            }
        }
    }
}

// The file tree, one row per entry.
file_tree :: (ed: *Editor) {
    for panel(.{ grow = 1 }) {
        label(file_name_of(ed.root), .{ muted = true, padding = 8 });

        for scroll(.{ grow = 1, padding = 4 }) {
            for entry, i: ed.tree {
                key(entry.path);
                prefix := ifx entry.is_directory then (ifx is_expanded(ed, entry.path) then "v " else "> ") else "  ";
                doc := active_document(ed);
                selected := doc != null && doc.path == entry.path;

                if list_item(tprint("%%", prefix, entry.name), selected, entry.depth * 14.0) {
                    if entry.is_directory toggle_directory(ed, entry.path);
                    else open_document(ed, entry.path);
                }
            }
        }
    }

}

// Tabs, the search bar, the text and the status line.
editor_pane :: (ed: *Editor) {
    doc := active_document(ed);

    for column(.{ grow = 1 }) {
        for tab_bar() {
            for d, i: ed.docs {
                result := tab(d.name, i == ed.active, .{ dirty = is_dirty(d), closable = true });

                if result == .SELECT ed.active = i;

                if result == .CLOSE {
                    ed.active = i;
                    close_active(ed);
                    break;
                }
            }
        }

        if !doc {
            for column(.{ grow = 1, align = .CENTER, justify = .CENTER, gap = 6 }) {
                label("No file open", .{ size = 18 });
                label("Pick one in the tree or press Cmd+O (Ctrl+O)", .{ muted = true });
            }
        } else {
            if ed.search_open search_bar(ed, doc);

            options := Editor_Options.{ line_numbers = ed.config.line_numbers, size = ed.config.font_size, highlight = highlight_jai, marks = ed.search_marks, tab_width = ed.config.tab_width };
            text_editor(*doc.edit, options);
            doc.close_armed = doc.close_armed && is_dirty(doc);
        }

        status_line(ed, doc);
    }
}

// The search bar: a query field, regex toggle and next/previous.
search_bar :: (ed: *Editor, doc: *Document) {
    ui := current_ui();
    t := *ui.theme;

    for row(.{ padding = 6, gap = 8, align = .CENTER, background = t.surface }) {
        label("Find", .{ muted = true });
        _, submitted := text_field(*ed.search_query, .{ placeholder = "text or regex", grow = 1 });

        checkbox(*ed.search_regex, "Regex");

        update_search(ed, doc);

        if button("Prev") step_match(ed, doc, -1);
        if button("Next") || submitted step_match(ed, doc, 1);

        count := ed.search_marks.count;

        if ed.search_error.count {
            label(ed.search_error, .{ color = t.error });
        } else {
            label(tprint("% matches", count), .{ muted = true });
        }

        for event: ui.input.keys {
            if event.kind == .KEY && event.key == .ESCAPE ed.search_open = false;
        }
    }

    if !ed.search_open array_reset(*ed.search_marks);
}

// The line below the text: path, caret position and the last message.
status_line :: (ed: *Editor, doc: *Document) {
    ui := current_ui();
    t := *ui.theme;

    for row(.{ padding_x = 10, padding_y = 4, gap = 16, background = t.surface }) {
        if doc {
            line, column := caret_position(*doc.edit);
            label(ifx doc.path.count then doc.path else "(untitled)", .{ muted = true });
            label(tprint("Ln %, Col %", line + 1, column + 1), .{ muted = true });
            if is_dirty(doc) label("modified", .{ color = t.warning });
        }

        spacer();
        label(ed.status, .{ muted = true });
    }
}

#scope_module

// Recompute the marks when the document, its text or the query changed.
update_search :: (ed: *Editor, doc: *Document) {
    query := contents(*ed.search_query);
    key := cast(u64) doc.edit.buffer.version;
    key = key * SEARCH_KEY_MULTIPLIER + cast(u64) ed.active;
    key = key * SEARCH_KEY_MULTIPLIER + cast(u64) ed.search_query.buffer.version;
    key = key * SEARCH_KEY_MULTIPLIER + cast(u64) (ifx ed.search_regex then 1 else 0);
    key = key * SEARCH_KEY_MULTIPLIER + cast(u64) query.count + 1;

    if key == ed.search_key return;

    ed.search_key = key;

    array_reset(*ed.search_marks);
    free(ed.search_error.data);
    ed.search_error = "";
    ed.search_current = -1;
    if !query.count return;

    text := contents(*doc.edit);
    mark_color := with_alpha(current_ui().theme.warning, 0.35);

    if ed.search_regex {
        re, ok := Re.compile(query);

        if !ok {
            ed.search_error = copy_string("invalid regex");
            return;
        }

        defer Re.free_regex(*re);
        found := Re.find_all(*re, text);

        defer array_free(found);

        for found {
            if it.end > it.start && ed.search_marks.count < MAX_MATCHES array_add(*ed.search_marks, .{ start = it.start, end = it.end, color = mark_color });
        }
    } else {
        at := 0;

        while at >= 0 && ed.search_marks.count < MAX_MATCHES {
            at = find_text(*doc.edit.buffer, query, at);
            if at < 0 break;

            array_add(*ed.search_marks, .{ start = at, end = at + query.count, color = mark_color });
            at += query.count;
        }
    }
}

MAX_MATCHES :: 5000;
SEARCH_KEY_MULTIPLIER :: 1000003;

// Select the next or previous match relative to the current selection, wrapping at the ends.
step_match :: (ed: *Editor, doc: *Document, direction: int) {
    update_search(ed, doc);
    if !ed.search_marks.count return;

    start, end := selection_range(*doc.edit);
    chosen := -1;

    if direction > 0 {
        for ed.search_marks if it.start >= end && it.start != start {
            chosen = it_index;
            break;
        }

        if chosen < 0 chosen = 0;
    } else {
        for < ed.search_marks if it.start < start {
            chosen = it_index;
            break;
        }

        if chosen < 0 chosen = ed.search_marks.count - 1;
    }

    mark := ed.search_marks[chosen];

    doc.edit.anchor = mark.start;
    doc.edit.cursor = mark.end;
    ed.search_current = chosen;
}

close_document_data :: (doc: *Document) {
    free_text_edit(*doc.edit);
    free(doc.path.data);
    free(doc.name.data);
    free(doc);
}

is_absolute :: (path: string) -> bool {
    if !path.count return false;
    if path.data[0] == #char "/" return true;
    return path.count > 2 && path.data[1] == #char ":";
}

file_name_of :: (path: string) -> string {
    i := path.count - 1;
    while i >= 0 && path.data[i] != #char "/" && path.data[i] != #char "\\" i -= 1;
    return string.{ path.count - i - 1, path.data + i + 1 };
}

is_expanded :: (ed: *Editor, path: string) -> bool {
    for ed.expanded if it == path return true;
    return false;
}

toggle_directory :: (ed: *Editor, path: string) {
    for ed.expanded {
        if it == path {
            free(it.data);
            remove it;
            ed.tree_dirty = true;
            return;
        }
    }

    array_add(*ed.expanded, copy_string(path));
    ed.tree_dirty = true;
}

clear_tree :: (ed: *Editor) {
    for ed.tree {
        free(it.path.data);
        free(it.name.data);
    }

    array_reset(*ed.tree);
}

rebuild_tree :: (ed: *Editor) {
    ed.tree_dirty = false;
    clear_tree(ed);
    list_directory(ed, ed.root, 0);
}

list_directory :: (ed: *Editor, directory: string, depth: int) {
    found: [..] Tree_Entry;
    found.allocator = temp;
    visit_files(directory, false, *found, collect_entry, true, true, false, false);
    quick_sort(found, compare_entries);

    for found {
        entry := it;
        entry.path = copy_string(entry.path);
        entry.name = copy_string(entry.name);
        entry.depth = depth;
        array_add(*ed.tree, entry);

        if entry.is_directory && is_expanded(ed, entry.path) list_directory(ed, entry.path, depth + 1);
    }
}

collect_entry :: (info: *File_Visit_Info, found: *[..] Tree_Entry) {
    if info.short_name.count == 0 || info.short_name.data[0] == #char "." return;

    entry: Tree_Entry;
    entry.path = copy_string(info.full_name,, temp);
    entry.name = copy_string(info.short_name,, temp);
    entry.is_directory = info.is_directory;
    array_add(found, entry);
}

// Directories first, then by name.
compare_entries :: (a: Tree_Entry, b: Tree_Entry) -> int {
    if a.is_directory != b.is_directory return ifx a.is_directory then -1 else 1;
    return compare_names(a.name, b.name);
}

compare_names :: (a: string, b: string) -> int {
    n := min(a.count, b.count);

    for 0..n - 1 {
        x := to_lower(a.data[it]);
        y := to_lower(b.data[it]);
        if x != y return ifx x < y then -1 else 1;
    }

    return a.count - b.count;
}

on_file_change :: (watcher: *File_Watcher(Editor), change: *File_Change, ed: *Editor) {
    ed.tree_dirty = true;

    // A document that is open and unchanged follows the file on disk.
    for ed.docs {
        if it.path != change.full_path || is_dirty(it) continue;

        text, ok := read_entire_file(it.path);
        if !ok continue;

        defer free(text.data);

        if text != contents(*it.edit) {
            cursor := it.edit.cursor;

            set_contents(*it.edit, text);
            it.edit.cursor = min(cursor, it.edit.buffer.length);
            it.edit.anchor = it.edit.cursor;
            it.saved_version = it.edit.buffer.version;
        }
    }
}

process_watcher :: (ed: *Editor, ui: *UI_State) {
    if !ed.watching return;

    changed, needs_wait, wait_seconds := process_changes(*ed.watcher);
    if changed ui.wants_frame = true;

    // Poll again soon: there is no operating-system wakeup for the watcher.
    wake_in(ui, ifx needs_wait then max(wait_seconds, 0.05) else WATCH_POLL_SECONDS);
}

WATCH_POLL_SECONDS :: 0.5;

// `path` without its last component ("." when it has none).
parent_directory :: (path: string) -> string {
    i := path.count - 1;
    while i >= 0 && path.data[i] != #char "/" && path.data[i] != #char "\\" i -= 1;
    if i < 0 return ".";
    if i == 0 return "/";

    return string.{ i, path.data };
}

// `path` made absolute against the working directory.
absolute_path :: (path: string) -> string {
    if is_absolute(path) return path;

    cwd := get_working_directory();
    if path == "." return cwd;
    return tprint("%/%", cwd, path);
}
