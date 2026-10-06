// File_Watcher on a real directory tree: additions, modifications and removals (recursively),
// event filtering, the merge window, repeated and missing roots, and deinit.
#import "Basic";
#import "File";
#import "File_Watcher";
#import "String";

ROOT :: ".scratch/file-watcher-scan";

path :: (name: string) -> string {
    return tprint("%/%", ROOT, name);
}

Seen :: struct {
    names: [..] string;
    events: [..] File_Change_Event;
}

// Records each change as "<name relative to the root>" with its events.
on_change :: (watcher: *File_Watcher(Seen), change: *File_Change, seen: *Seen) {
    // Roots are absolute (with backslashes on Windows); names below them use "/".
    marker :: "file-watcher-scan";
    at := find_index_from_left(change.full_path, marker);
    assert(at >= 0, "change outside the root: %", change.full_path);
    start := at + marker.count + 1;
    name := slice(change.full_path, start, change.full_path.count - start);
    array_add(*seen.names, copy_string(name));
    array_add(*seen.events, change.events);
}

// The events reported for `name`, or 0 when it was not reported.
events_for :: (seen: *Seen, name: string) -> File_Change_Event {
    for seen.names if it == name return seen.events[it_index];
    return 0;
}

forget :: (seen: *Seen) {
    for seen.names free(it);
    array_reset(*seen.names);
    array_reset(*seen.events);
}

main :: () {
    delete_directory(ROOT);
    assert(make_directory_if_it_does_not_exist(path("sub"), recursive = true));
    assert(write_entire_file(path("keep.txt"), "keep"));
    assert(write_entire_file(path("change.txt"), "short"));
    assert(write_entire_file(path("remove.txt"), "remove"));

    seen: Seen;
    watcher: File_Watcher(Seen);
    // Bad arguments, and use before init.
    assert(!add_directories(*watcher, ROOT));
    assert(!init(*watcher, null, *seen));
    assert(!init(*watcher, on_change, *seen, merge_window_seconds = -1));
    assert(!init(*watcher, on_change, *seen, event_buffer_size = 0));

    assert(init(*watcher, on_change, *seen, merge_window_seconds = 0));
    // Initializing twice is refused.
    assert(!init(*watcher, on_change, *seen));
    assert(add_directories(*watcher, ROOT));
    // The same root again is not watched twice; a missing one fails.
    assert(add_directories(*watcher, ROOT, tprint("%/sub/..", ROOT)));
    assert(watcher.roots.count == 1, "% roots", watcher.roots.count);
    assert(!add_directories(*watcher, path("missing")));

    // Nothing happened yet.
    changed, needs_wait := process_changes(*watcher);
    assert(!changed && !needs_wait && seen.names.count == 0, "quiet start reported % changes", seen.names.count);

    // Sizes change so a coarse modification clock cannot hide the edit.
    assert(write_entire_file(path("change.txt"), "much longer contents"));
    assert(write_entire_file(path("added.txt"), "new"));
    assert(write_entire_file(path("sub/nested.txt"), "nested"));
    assert(file_delete(path("remove.txt")));
    changed, needs_wait = process_changes(*watcher);
    assert(changed && !needs_wait);
    assert(events_for(*seen, "change.txt") == .MODIFIED, "change.txt: %", events_for(*seen, "change.txt"));
    assert(events_for(*seen, "added.txt") == .ADDED, "added.txt: %", events_for(*seen, "added.txt"));
    assert(events_for(*seen, "sub/nested.txt") == .ADDED, "sub/nested.txt: %", events_for(*seen, "sub/nested.txt"));
    assert(events_for(*seen, "remove.txt") == .REMOVED, "remove.txt: %", events_for(*seen, "remove.txt"));
    assert(events_for(*seen, "keep.txt") == 0, "keep.txt was not touched");
    // The directory "sub" changed too, where the file system dates a directory by its entries.
    sub_events := events_for(*seen, "sub");
    assert(seen.names.count == 4 + xx (sub_events != 0) && (sub_events == 0 || sub_events == .MODIFIED), "% changes, sub: %", seen.names.count, sub_events);
    forget(*seen);

    // A second pass with no edits reports nothing.
    changed = process_changes(*watcher);
    assert(!changed && seen.names.count == 0);

    // Removing a directory reports what was in it.
    assert(delete_directory(path("sub")));
    changed = process_changes(*watcher);
    assert(changed && events_for(*seen, "sub/nested.txt") == .REMOVED && events_for(*seen, "sub") == .REMOVED, "directory removal: % names", seen.names.count);
    forget(*seen);
    deinit(*watcher);
    assert(!watcher.initialized && watcher.roots.count == 0);

    {
        // Only additions are wanted; nothing is recursive; a long merge window holds changes back.
        held: Seen;
        filtered: File_Watcher(Seen);
        assert(init(*filtered, on_change, *held, events_to_watch = .ADDED, merge_window_seconds = 30, watch_recursively = false));
        assert(make_directory_if_it_does_not_exist(path("sub2")));
        assert(add_directories(*filtered, ROOT));
        assert(write_entire_file(path("sub2/deep.txt"), "not watched"));
        assert(write_entire_file(path("keep.txt"), "kept, but longer now"));
        assert(write_entire_file(path("late.txt"), "late"));
        changed, needs_wait, wait_seconds := process_changes(*filtered);
        assert(!changed && needs_wait && wait_seconds > 0 && wait_seconds <= 30, "held back: % % %", changed, needs_wait, wait_seconds);
        assert(held.names.count == 0);
        assert(filtered.changes.count == 1 && ends_with(filtered.changes[0].full_path, "late.txt"), "% pending", filtered.changes.count);
        // With the window closed, emit_changes delivers it.
        filtered.merge_window_seconds = 0;
        changed, needs_wait = emit_changes(*filtered);
        assert(changed && !needs_wait && held.names.count == 1 && events_for(*held, "late.txt") == .ADDED);
        forget(*held);
        // deinit on the value releases pending changes too.
        assert(write_entire_file(path("later.txt"), "later"));
        filtered.merge_window_seconds = 30;
        process_changes(*filtered);
        assert(filtered.changes.count == 1);
        deinit(filtered);
    }

    assert(delete_directory(ROOT));
    print("ok\n");
}
