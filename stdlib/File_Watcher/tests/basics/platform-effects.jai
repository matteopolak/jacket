#import "Basic";
W :: #import "File_Watcher";
F :: #import "File";
#if OS == .LINUX P :: #import "POSIX";

#if OS == .WINDOWS {
    Win :: #import "Windows";

    test_kernel32 :: #library,system "kernel32";
    // Official SDK observer, private to this test; not a compatibility export.
    handle_count :: (process: Win.HANDLE, count: *u32) -> Win.BOOL #foreign test_kernel32 "GetProcessHandleCount";
}

Observation :: struct {
    path: string;
    events: W.File_Change_Event;
}

Log :: struct {
    events: [..] Observation;
    owner: Allocator;
}

Allocation_Count :: struct {
    backing: Allocator;
    live: int;
}

track :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    count := cast(*Allocation_Count) data;
    result := count.backing.proc(mode, size, old_size, old, count.backing.data);

    if mode == .ALLOCATE && result count.live += 1;
    else if mode == .RESIZE && !old && result count.live += 1;
    else if mode == .FREE && old count.live -= 1;

    assert(count.live >= 0);

    return result;
}

changed :: (_watcher: *W.File_Watcher(Log), change: *W.File_Change, log: *Log) {
    assert(log != null && change.full_path.count > 0 && change.time_of_last_change >= 0);

    push_context {
        context.allocator = log.owner;
        array_add(*log.events, Observation.{ copy_string(change.full_path), change.events });
    }
}

has :: (log: *Log, path: string, event: W.File_Change_Event) -> bool {
    for record: log.events if record.path == path && (record.events & event) != 0 return true;
    return false;
}

wait_for :: (watcher: *W.File_Watcher(Log), log: *Log, path: string, event: W.File_Change_Event) {
    for 0..499 {
        _, waiting, seconds := W.process_changes(watcher);
        assert(seconds >= 0 && (waiting || has(log, path, event)));
        if has(log, path, event) return;
        sleep_milliseconds(10);
    }

    assert(false, "native directory notification missing: % / %", path, event);
}

free_log :: (log: *Log) {
    push_context {
        context.allocator = log.owner;
        for event: log.events free(event.path);
        array_reset(*log.events);
    }
}

main :: () {
    #assert OS == .LINUX || OS == .WINDOWS;
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    root, absolute := F.get_absolute_path(args[1]);

    assert(absolute);
    defer free(root);
    for *byte: root if byte.* == #char "\\" byte.* = #char "/";

    allocations := Allocation_Count.{ backing = context.allocator };
    owner := Allocator.{ track, *allocations };
    log := Log.{ owner = owner };
    log.events.allocator = owner;
    watcher: W.File_Watcher(Log);

    #if OS == .WINDOWS {
        handles_before: u32;
        assert(handle_count(Win.GetCurrentProcess(), *handles_before) != 0);
    }

    push_context {
        context.allocator = owner;
        assert(W.init(*watcher, changed, *log, merge_window_seconds = 0.02));
        assert(W.add_directories(*watcher, root));
        assert(W.add_directories(*watcher, root));
    }

    // Poll and clean up under a different context: owned allocations must still
    // retire through the allocator captured at initialization.
    #if OS == .LINUX {
        assert(watcher.inotify_instance >= 0 && watcher.inotify_data != null && watcher.path_pool != null);
        assert(watcher.watch_paths.count == 1);
    }

    name := sprint("%/original.txt", root);
    renamed := sprint("%/renamed.txt", root);
    directory := sprint("%/child", root);
    moved := sprint("%/moved", root);
    nested := sprint("%/child/inside.txt", root);
    relocated := sprint("%/moved/inside.txt", root);

    defer {
        free(name);
        free(renamed);
        free(directory);
        free(moved);
        free(nested);
        free(relocated);
    }

    assert(F.write_entire_file(name, "first"));
    wait_for(*watcher, *log, name, .ADDED);
    // Discard creation's MODIFIED event so the next wait proves a new write.
    free_log(*log);
    log.events.allocator = owner;
    assert(F.write_entire_file(name, "second"));
    wait_for(*watcher, *log, name, .MODIFIED);
    assert(F.file_move(name, renamed));
    wait_for(*watcher, *log, name, .MOVED_FROM);
    wait_for(*watcher, *log, renamed, .MOVED_TO);
    assert(F.file_delete(renamed));
    wait_for(*watcher, *log, renamed, .REMOVED);
    assert(F.make_directory_if_it_does_not_exist(directory));
    wait_for(*watcher, *log, directory, .ADDED);
    assert(F.write_entire_file(nested, "nested"));
    wait_for(*watcher, *log, nested, .ADDED);
    assert(F.file_move(directory, moved));
    wait_for(*watcher, *log, moved, .MOVED_TO);
    free_log(*log);
    log.events.allocator = owner;
    assert(F.write_entire_file(relocated, "after move"));
    wait_for(*watcher, *log, relocated, .MODIFIED);
    for event: log.events assert(event.path != nested, "watch descriptor retained stale renamed path");

    assert(F.file_delete(relocated));
    wait_for(*watcher, *log, relocated, .REMOVED);
    assert(F.delete_directory(moved));
    // Keep a notification pending when canceling; deinit must retire the real
    // kernel operation before releasing caller-visible ownership records.
    assert(F.write_entire_file(name, "cancel pending"));
    before := log.events.count;

    #if OS == .LINUX descriptor := watcher.inotify_instance;
    W.deinit(watcher);
    assert(F.file_delete(name));
    sleep_milliseconds(30);
    assert(log.events.count == before);

    #if OS == .LINUX {
        assert(P.fcntl(descriptor, P.F_GETFD) == -1 && cast(s32) P.errno() == P.EBADF);
    }

    #if OS == .WINDOWS {
        handles_after: u32;
        assert(handle_count(Win.GetCurrentProcess(), *handles_after) != 0);
        assert(handles_after == handles_before, "File_Watcher leaked directory/event handles");
    }

    free_log(*log);
    assert(allocations.live == 0, "File_Watcher leaked captured allocator storage");
    print("ok\n");
}
