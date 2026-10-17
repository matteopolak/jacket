#scope_module
L :: #import "Linux";
P :: #import "POSIX";
HT :: #import "Hash_Table";
U :: #import "File_Utilities";
R :: #import "internal/watching/event-records";
Strings :: #import "String";

inotify_libc :: #library,system "c";
system_free :: (memory: *void) #foreign inotify_libc "free";

Pending_Move :: struct {
    cookie: u32;
    path: string;
    deadline: float64;
}

backend_init :: (watcher: *$T/File_Watcher, state: *State) -> bool {
    descriptor := L.inotify_init(L.IN_NONBLOCK | L.IN_CLOEXEC);
    if descriptor < 0 {
        report_errno("inotify_init1");
        return false;
    }

    buffer := state.owner.proc(.ALLOCATE, watcher.buffer_size, 0, null, state.owner.data);
    if !buffer {
        if P.close(descriptor) != 0 report_errno("close after inotify allocation failure");
        return false;
    }

    paths := B.New(Pool.Pool);
    if !paths {
        state.owner.proc(.FREE, 0, watcher.buffer_size, buffer, state.owner.data);
        if P.close(descriptor) != 0 report_errno("close after path pool allocation failure");
        return false;
    }

    Pool.set_allocators(paths, state.owner);
    state.descriptor = descriptor;
    state.read_buffer = buffer;
    state.read_capacity = watcher.buffer_size;
    state.path_storage = paths;
    state.moves.allocator = state.owner;
    watcher.inotify_instance = descriptor;
    watcher.inotify_data = buffer;
    watcher.path_pool = paths;
    watcher.watch_paths.allocator = state.owner;

    return true;
}

report_errno :: (operation: string) {
    message := B.sprint("File_Watcher % failed (errno %)", operation, P.errno());
    defer B.free(message);
    report_error(message);
}

backend_add_directory :: (watcher: *$T/File_Watcher, state: *State, path: string) -> bool {
    for byte: path if byte == 0 return false;
    if !path.count return false;

    native := B.to_c_string(path);

    defer B.free(native);
    absolute := P.realpath(native, null);
    if !absolute {
        report_errno("realpath");
        return false;
    }

    defer system_free(absolute);
    count: int;
    while absolute[count] != 0 count += 1;

    root: string = .{ count, absolute };
    if !register_path(watcher, state, root) return false;

    found := false;
    for existing: state.roots if existing == root found = true;

    if !found {
        stored := pooled_path(watcher.path_pool, root);
        if !stored return false;
        B.array_add(*state.roots, stored);
    }

    if !watcher.watch_recursively return true;

    return register_tree(watcher, state, root);
}

register_path :: (watcher: *$T/File_Watcher, state: *State, path: string) -> bool {
    native := B.to_c_string(path);

    defer B.free(native);
    mask: u32 = L.IN_CREATE | L.IN_MODIFY | L.IN_CLOSE_WRITE | L.IN_ATTRIB | L.IN_MOVED_FROM | L.IN_MOVED_TO | L.IN_DELETE | L.IN_DELETE_SELF | L.IN_MOVE_SELF | L.IN_ONLYDIR | L.IN_DONT_FOLLOW;
    wd := L.inotify_add_watch(state.descriptor, native, mask);

    if wd < 0 {
        report_errno("inotify_add_watch");
        return false;
    }

    found, _ := HT.table_find(*watcher.watch_paths, wd);
    if found return true;

    stored := pooled_path(watcher.path_pool, path);
    if stored && HT.table_add(*watcher.watch_paths, wd, stored) != null return true;

    if L.inotify_rm_watch(state.descriptor, wd) != 0 {
        report_errno("inotify_rm_watch after registration allocation failure");
        state.failed = true;
    }

    return false;
}

register_tree :: (watcher: *$T/File_Watcher, state: *State, path: string) -> bool {
    Registration :: struct {
        watcher: *T;
        state: *State;
        success: bool = true;
    }

    request := Registration.{ watcher = watcher, state = state };

    visit :: (info: *U.File_Visit_Info, request: *Registration) {
        if info.had_error {
            request.success = false;
            return;
        }

        if info.is_directory && !info.is_symlink && !register_path(request.watcher, request.state, info.full_name) request.success = false;
    }

    walked := U.visit_files(path, true, *request, visit, visit_files = false, visit_directories = true, visit_symlinks = false, follow_directory_symlinks = false);

    return walked && request.success;
}

classify_inotify :: (flags: u32) -> File_Change_Event {
    result: File_Change_Event;
    if flags & L.IN_CREATE result |= .ADDED;
    if flags & (L.IN_MODIFY | L.IN_CLOSE_WRITE) result |= .MODIFIED;
    if flags & L.IN_ATTRIB result |= .ATTRIBUTES_CHANGED;
    if flags & L.IN_MOVED_FROM result |= .MOVED | .MOVED_FROM;
    if flags & L.IN_MOVED_TO result |= .MOVED | .MOVED_TO;
    if flags & L.IN_MOVE_SELF result |= .MOVED;
    if flags & (L.IN_DELETE | L.IN_DELETE_SELF) result |= .REMOVED;

    return result;
}

path_below :: (path: string, root: string) -> bool {
    return path == root || (path.count > root.count && Strings.begins_with(path, root) && (root == "/" || path[root.count] == #char "/"));
}

remove_tree :: (watcher: *$T/File_Watcher, state: *State, path: string) -> bool {
    descriptors: [..] s32;

    defer B.array_free(descriptors);
    for name, wd: watcher.watch_paths if path_below(name, path) B.array_add(*descriptors, wd);

    for wd: descriptors {
        if L.inotify_rm_watch(state.descriptor, wd) != 0 && cast(s32) P.errno() != P.EINVAL {
            report_errno("inotify_rm_watch");
            return false;
        }

        HT.table_remove(*watcher.watch_paths, wd);
    }

    return true;
}

move_paths :: (watcher: *$T/File_Watcher, from: string, to: string) -> bool {
    for *path: watcher.watch_paths {
        if !path_below(path.*, from) continue;

        renamed := B.sprint("%%", to, string.{ path.count - from.count, path.data + from.count });

        defer B.free(renamed);
        replacement := pooled_path(watcher.path_pool, renamed);
        if !replacement return false;

        path.* = replacement;
    }

    return true;
}

handle_inotify :: (watcher: *$T/File_Watcher, state: *State, record: R.Record) -> bool {
    if record.flags & L.IN_Q_OVERFLOW {
        for root: state.roots if !queue_event(state, root, .SCAN_CHILDREN | .MODIFIED) return false;
        return true;
    }

    found, root := HT.table_find(*watcher.watch_paths, record.descriptor);
    if !found return true; // Retired descriptors can still have queued IN_IGNORED.

    if record.flags & L.IN_IGNORED {
        HT.table_remove(*watcher.watch_paths, record.descriptor);
        return true;
    }

    path := ifx record.name.count then join_path(root, string.{ record.name.count, record.name.data }) else B.copy_string(root);

    defer B.free(path);
    events := classify_inotify(record.flags);
    if record.flags & L.IN_UNMOUNT events |= .SCAN_CHILDREN | .MODIFIED;
    if !queue_event(state, path, events) return false;
    if !watcher.watch_recursively return true;

    if record.flags & L.IN_MOVED_FROM && record.flags & L.IN_ISDIR {
        stored := pooled_path(watcher.path_pool, path);
        if !stored return false;
        B.array_add(*state.moves, Pending_Move.{ cookie = record.cookie, path = stored, deadline = B.seconds_since_init() + 0.1 });
    }

    if record.flags & L.IN_MOVED_TO && record.flags & L.IN_ISDIR {
        for move, index: state.moves {
            if move.cookie != record.cookie continue;
            if !move_paths(watcher, move.path, path) return false;
            B.array_ordered_remove_by_index(*state.moves, index);
            break;
        }
    }

    if record.flags & (L.IN_CREATE | L.IN_MOVED_TO) && record.flags & L.IN_ISDIR {
        if !register_path(watcher, state, path) {
            // The directory can disappear before registration; report rescan and
            // keep watching the surviving parent rather than inventing a watch.
            return queue_event(state, root, .SCAN_CHILDREN | .MODIFIED);
        }

        if !register_tree(watcher, state, path) return queue_event(state, path, .SCAN_CHILDREN | .MODIFIED);
    }

    return true;
}

backend_drain :: (watcher: *$T/File_Watcher, state: *State) -> bool {
    if watcher.inotify_instance != state.descriptor || watcher.inotify_data != state.read_buffer || watcher.buffer_size != state.read_capacity || watcher.path_pool != state.path_storage {
        report_error("File_Watcher native input storage changed externally");
        return false;
    }

    if watcher.inotify_cursor < 0 || watcher.inotify_bytes_read < 0 || watcher.inotify_bytes_read > state.read_capacity || watcher.inotify_cursor > watcher.inotify_bytes_read {
        report_error("File_Watcher invalid public notification buffer extent");
        return false;
    }

    while true {
        while watcher.inotify_cursor < watcher.inotify_bytes_read {
            bytes: [] u8 = .{ watcher.inotify_bytes_read, cast(*u8) watcher.inotify_data };
            record, next, valid := R.inotify_record(bytes, watcher.inotify_cursor);

            if !valid {
                report_error("File_Watcher malformed inotify record");
                return false;
            }

            watcher.inotify_cursor = cast(s32) next;
            if !handle_inotify(watcher, state, record) return false;
        }

        count := P.read(state.descriptor, watcher.inotify_data, cast(u64) watcher.buffer_size);
        if count < 0 {
            error := cast(s32) P.errno();
            if error == P.EINTR continue;
            if error == P.EAGAIN break;

            report_errno("inotify read");

            return false;
        }

        if count == 0 {
            report_error("File_Watcher unexpected inotify EOF");
            return false;
        }

        watcher.inotify_bytes_read = count;
        watcher.inotify_cursor = 0;
    }

    now := B.seconds_since_init();
    index: int;

    while index < state.moves.count {
        move := state.moves[index];
        if now < move.deadline {
            index += 1;
            continue;
        }

        if !remove_tree(watcher, state, move.path) return false;

        B.array_ordered_remove_by_index(*state.moves, index);
    }

    return true;
}

backend_active :: (watcher: *$T/File_Watcher) -> bool {
    return watcher.watch_paths.count > 0;
}

backend_deinit :: (watcher: *$T/File_Watcher, state: *State) -> bool {
    if state.descriptor >= 0 {
        status := P.close(state.descriptor);
        error := cast(s32) P.errno();

        // Linux closes the fd even on EINTR; retrying could close a reused fd.
        state.descriptor = -1;

        if status != 0 {
            report_errno("inotify close");
            if error != P.EINTR state.failed = true;
        }
    }

    state.owner.proc(.FREE, 0, state.read_capacity, state.read_buffer, state.owner.data);
    HT.deinit(*watcher.watch_paths);
    Pool.release(state.path_storage);
    state.owner.proc(.FREE, 0, size_of(Pool.Pool), state.path_storage, state.owner.data);
    B.array_free(state.moves);

    return true;
} @Releases
