#import "Basic";
W :: #import "File_Watcher";
F :: #import "File";
S :: #import "String";
P :: #import "Process";
N :: #import "POSIX";

libc :: #system_library "libc";
c_free :: (pointer: *void) #foreign libc "free";

observed: W.File_Change_Event;
callbacks: s64;
expected: string;

Allocations :: struct {
    base: Allocator;
    live: s64;
}

track :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    state := cast(*Allocations) data;
    result := state.base.proc(mode, size, old_size, old, state.base.data);

    if mode == .ALLOCATE && result state.live += 1;
    if mode == .RESIZE && !old && result state.live += 1;
    if mode == .FREE && old state.live -= 1;

    return result;
}

Tracker :: struct {
    path: string;
    alternate: string;
    events: W.File_Change_Event;
    calls: s64;
}

typed_changed :: (watcher: *W.File_Watcher(Tracker), change: *W.File_Change, data: *Tracker) {
    assert(watcher != null && data != null);

    if change.full_path == data.path || change.full_path == data.alternate {
        data.events |= change.events;
        data.calls += 1;
    }
}

changed :: (watcher: *W.File_Watcher(void), change: *W.File_Change, data: *void) {
    assert(watcher != null && data == null && change.full_path.count > 0);

    if change.full_path == expected {
        observed |= change.events;
        callbacks += 1;
    }
}

command :: (args: ..string) {
    result := P.run_command(..args);
    assert(result.type == .EXITED && result.exit_code == 0);
}

wait_for :: (watcher: *W.File_Watcher(void), event: W.File_Change_Event) {
    for 0..149 {
        delivered, needs_wait, seconds := W.process_changes(watcher);

        assert(needs_wait && seconds >= 0);

        if (observed & event) != 0 {
            assert(delivered);
            return;
        }

        assert(N.usleep(20000) == 0);
    }

    assert(false, "actual FSEvents notification was not delivered");
}

main :: () {
    allocations := Allocations.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ track, *allocations };
    push_context scoped run_cases();
    assert(allocations.live == 0, "watcher/test leaked Jai storage: %", allocations.live);
    print("ok\n");
}

run_cases :: () {
    #assert !#exists(W.State);
    #assert !#exists(W.c_pop);
    invalid: W.File_Watcher;

    assert(!W.init(*invalid, changed, merge_window_seconds = -1));
    assert(!W.init(*invalid, changed, event_buffer_size = 0));
    nonfinite_bits: u32 = 0x7f800000;
    nonfinite: float;

    memcpy(*nonfinite, *nonfinite_bits, 4);
    assert(!W.init(*invalid, changed, merge_window_seconds = nonfinite));
    assert(invalid.pool == null && invalid.callback == null);
    W.deinit(invalid);
    root := sprint("%/../../../target/portable-tests/watch fixtures", S.path_strip_filename(#file));

    defer free(root);
    command("/bin/rm", "-rf", root);
    command("/bin/mkdir", "-p", root);
    name := to_c_string(root);

    defer free(name);
    canonical := N.realpath(name, null);

    assert(canonical != null);
    defer c_free(canonical);
    length: s64;
    while canonical[length] != 0 length += 1;

    normalized := copy_string(to_string(canonical, length));

    defer free(normalized);
    watcher: W.File_Watcher;

    assert(W.init(*watcher, changed, merge_window_seconds = 0.03));
    assert(W.add_directories(*watcher, normalized));
    assert(watcher.event_streams.count == 1 && watcher.event_streams[0] != null);
    expected = sprint("%/original.txt", normalized);
    defer free(expected);
    assert(F.write_entire_file(expected, "first"));
    wait_for(*watcher, .ADDED);
    observed = .{};
    assert(F.write_entire_file(expected, "modified"));
    wait_for(*watcher, .MODIFIED);
    observed = .{};
    command("/bin/rm", expected);
    wait_for(*watcher, .REMOVED);
    assert(callbacks >= 3);
    W.deinit(watcher);
    tracker := Tracker.{ path = sprint("%/typed.txt", normalized) };

    defer free(tracker.path);
    typed: W.File_Watcher(Tracker);

    assert(W.init(*typed, typed_changed, *tracker, merge_window_seconds = 10));
    polling_context := context;
    polling_context.allocator = (cast(*Allocations) context.allocator.data).base;
    push_context polling_context {
        empty_changed, empty_wait, empty_seconds := W.process_changes(*typed);

        assert(!empty_changed && !empty_wait && empty_seconds == 0 && tracker.calls == 0);
        assert(W.add_directories(*typed, normalized));
        assert(F.write_entire_file(tracker.path, "pending"));
        queued := false;

        for 0..149 {
            delivered, waiting, seconds := W.process_changes(*typed);

            assert(!delivered && waiting && seconds > 0 && tracker.calls == 0);
            for change: typed.changes if change.full_path == tracker.path queued = true;
            if queued break;

            assert(N.usleep(20000) == 0);
        }

        assert(queued, "real event did not enter coalescing queue");
        typed.merge_window_seconds = 0;
        assert(F.write_entire_file(tracker.path, "updated pending content"));

        for 0..149 {
            delivered := W.process_changes(*typed);

            if tracker.calls > 0 {
                assert(delivered);
                break;
            }

            assert(N.usleep(20000) == 0);
        }

        assert(tracker.calls == 1 && (tracker.events & .ADDED) != 0 && (tracker.events & .MODIFIED) != 0);
        W.deinit(typed);
    }

    nested := sprint("%/nested", normalized);

    defer free(nested);
    command("/bin/mkdir", "-p", nested);
    deep := sprint("%/deep.txt", nested);

    defer free(deep);
    excluded := Tracker.{ path = deep };
    included := Tracker.{ path = deep };
    shallow: W.File_Watcher(Tracker);
    recursive: W.File_Watcher(Tracker);

    assert(W.init(*shallow, typed_changed, *excluded, watch_recursively = false, merge_window_seconds = 0));
    assert(W.init(*recursive, typed_changed, *included, merge_window_seconds = 0));
    assert(W.add_directories(*shallow, normalized));
    assert(W.add_directories(*recursive, normalized));
    assert(F.write_entire_file(deep, "nested file"));

    for 0..149 {
        W.process_changes(*shallow);
        W.process_changes(*recursive);
        if included.calls > 0 break;
        assert(N.usleep(20000) == 0);
    }

    assert(included.calls > 0 && (included.events & .ADDED) != 0);

    for 0..14 {
        W.process_changes(*shallow);
        assert(N.usleep(20000) == 0);
    }

    assert(excluded.calls == 0, "nonrecursive watcher delivered a nested file");
    flat := sprint("%/flat.txt", normalized);

    defer free(flat);
    excluded.path = flat;
    assert(F.write_entire_file(flat, "direct child"));

    for 0..149 {
        W.process_changes(*shallow);
        if excluded.calls > 0 break;
        assert(N.usleep(20000) == 0);
    }

    assert(excluded.calls > 0 && (excluded.events & .ADDED) != 0);
    renamed := sprint("%/renamed.txt", normalized);

    defer free(renamed);
    excluded.alternate = renamed;
    excluded.events = .{};
    command("/bin/mv", flat, renamed);

    for 0..149 {
        W.process_changes(*shallow);
        if (excluded.events & .MOVED) != 0 break;
        assert(N.usleep(20000) == 0);
    }

    assert((excluded.events & .MOVED) != 0);
    W.deinit(shallow);
    W.deinit(recursive);
    overflow_tracker := Tracker.{ path = normalized };
    overflow: W.File_Watcher(Tracker);

    assert(W.init(*overflow, typed_changed, *overflow_tracker, merge_window_seconds = 0, event_buffer_size = 1));
    assert(W.add_directories(*overflow, normalized));
    assert(F.write_entire_file(renamed, "overflow event"));

    for 0..149 {
        W.process_changes(*overflow);
        if (overflow_tracker.events & .SCAN_CHILDREN) != 0 break;
        assert(N.usleep(20000) == 0);
    }

    assert((overflow_tracker.events & .SCAN_CHILDREN) != 0, "bounded native overflow did not request a rescan");
    W.deinit(overflow);
    command("/bin/rm", "-rf", root);
    assert((cast(*Type_Info_Struct) type_info(W.File_Watcher)).name == "File_Watcher");
}
