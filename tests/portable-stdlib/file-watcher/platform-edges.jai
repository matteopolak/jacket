#import "Basic";
W :: #import "File_Watcher";
F :: #import "File";

Seen :: struct {
    count: int;
    nested: bool;
    forbidden: string;
    flags: W.File_Change_Event;
}

notice :: (_watcher: *W.File_Watcher(Seen), event: *W.File_Change, seen: *Seen) {
    seen.count += 1;
    seen.flags |= event.events;
    assert(event.time_of_last_change >= 0 && event.time_of_last_change <= seconds_since_init() + 1);
    if event.full_path == seen.forbidden seen.nested = true;
}

poll :: (watcher: *W.File_Watcher(Seen)) {
    for 0..29 {
        W.process_changes(watcher);
        sleep_milliseconds(10);
    }
}

main :: () {
    #assert OS == .LINUX || OS == .WINDOWS;
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    root, valid := F.get_absolute_path(args[1]);

    assert(valid);
    defer free(root);
    for *byte: root if byte.* == #char "\\" byte.* = #char "/";

    watched := sprint("%/watched", root);
    outside := sprint("%/outside", root);
    child := sprint("%/child", watched);
    moved := sprint("%/moved", outside);
    nested := sprint("%/nested.txt", child);
    relocated := sprint("%/nested.txt", moved);
    direct := sprint("%/direct.txt", watched);
    absent := sprint("%/absent", root);

    defer {
        free(watched);
        free(outside);
        free(child);
        free(moved);
        free(nested);
        free(relocated);
        free(direct);
        free(absent);
    }

    assert(F.make_directory_if_it_does_not_exist(watched));
    assert(F.make_directory_if_it_does_not_exist(outside));
    assert(F.make_directory_if_it_does_not_exist(child));
    seen := Seen.{ forbidden = nested };
    watcher: W.File_Watcher(Seen);

    assert(!W.init(*watcher, notice, *seen, event_buffer_size = 0));
    assert(watcher.pool == null);
    assert(!W.init(*watcher, notice, *seen, merge_window_seconds = -1));
    assert(watcher.pool == null);
    assert(W.init(*watcher, notice, *seen, merge_window_seconds = 0, watch_recursively = false));
    // A failed registration must preserve existing watches and allow retry.
    assert(!W.add_directories(*watcher, absent));
    assert(W.add_directories(*watcher, watched));
    assert(!W.add_directories(*watcher, absent));
    assert(F.write_entire_file(nested, "nonrecursive"));
    poll(*watcher);
    assert(!seen.nested && seen.count == 0);
    assert(F.write_entire_file(direct, "direct"));
    poll(*watcher);
    assert(seen.count > 0);
    W.deinit(watcher);
    watcher = .{};
    seen = .{ forbidden = relocated };
    assert(W.init(*watcher, notice, *seen, merge_window_seconds = 0));
    assert(W.add_directories(*watcher, watched));
    #if OS == .LINUX assert(watcher.watch_paths.count == 2);
    assert(F.file_move(child, moved));
    poll(*watcher);
    #if OS == .LINUX assert(watcher.watch_paths.count == 1, "moved-out subtree retains inotify descriptors");
    seen.count = 0;
    assert(F.write_entire_file(relocated, "outside the watch"));
    poll(*watcher);
    assert(!seen.nested && seen.count == 0);
    W.deinit(watcher);
    // A new lifecycle verifies filter/coalescing without retaining observations
    // from the earlier watches. Two writes must merge to one owned path record.
    assert(F.file_delete(direct));
    watcher = .{};
    seen = .{};
    assert(W.init(*watcher, notice, *seen, events_to_watch = .ADDED | .MODIFIED, merge_window_seconds = 0.2));
    assert(W.add_directories(*watcher, watched));
    assert(F.write_entire_file(direct, "one"));
    assert(F.write_entire_file(direct, "two"));
    poll(*watcher);
    assert(seen.count == 1 && (seen.flags & .ADDED) != 0 && (seen.flags & .MODIFIED) != 0);
    assert(F.file_delete(direct));
    poll(*watcher);
    assert(seen.count == 1, "removed notifications bypassed the requested mask");
    W.deinit(watcher);
    assert(F.file_delete(relocated));
    assert(F.delete_directory(moved));
    assert(F.delete_directory(watched));
    assert(F.delete_directory(outside));
    print("ok\n");
}
