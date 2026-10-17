File_Change_Event :: enum_flags {
    ADDED;
    MODIFIED;
    MOVED;
    MOVED_FROM;
    MOVED_TO;
    ATTRIBUTES_CHANGED;
    REMOVED;
    SCAN_CHILDREN;
    ALL_EVENTS :: ADDED | MODIFIED | MOVED | ATTRIBUTES_CHANGED | REMOVED;
}

File_Change :: struct {
    full_path: string;
    events: File_Change_Event;
    time_of_last_change := 0.0;
}

DEFAULT_BUFFER_SIZE :: 8000;

File_Watcher :: struct(User_Data: Type = void) {
    File_Watcher_Callback :: #type (watcher: *File_Watcher(User_Data), change: *File_Change, user_data: *User_Data);
    callback: File_Watcher_Callback;
    user_data: *User_Data;
    events_to_watch: File_Change_Event;
    merge_window_seconds: float;
    watch_recursively: bool;
    verbose: bool;
    buffer_size: s32;
    pool: *Pool.Pool;
    changes: [..] File_Change;
    event_streams: [..] Native.FSEventStreamRef;
}

init :: (watcher: *$T/File_Watcher, callback: T.File_Watcher_Callback, user_data: *T.User_Data = null, events_to_watch: File_Change_Event = .ALL_EVENTS, merge_window_seconds: float = 0.1, watch_recursively := true, verbose := false, event_buffer_size: s32 = DEFAULT_BUFFER_SIZE) -> success: bool {
    if watcher == null || watcher.pool != null {
        B.assert(false, "watcher is null or already initialized");
        return false;
    }

    if !callback || !Math.is_finite(merge_window_seconds) || merge_window_seconds < 0 || event_buffer_size <= 0 return false;

    state := B.New(State);
    state.owner = context.allocator;
    state.watcher = watcher;
    state.changes = *watcher.changes;
    state.events = *watcher.events_to_watch;
    state.window = *watcher.merge_window_seconds;
    state.recursive = *watcher.watch_recursively;
    state.deadlines.allocator = state.owner;
    watcher.pool = B.New(Pool.Pool);
    Pool.set_allocators(watcher.pool, state.owner);
    state.pool = watcher.pool;
    watcher.callback = callback;
    watcher.user_data = user_data;
    watcher.events_to_watch = events_to_watch;
    watcher.merge_window_seconds = merge_window_seconds;
    watcher.watch_recursively = watch_recursively;
    watcher.verbose = verbose;
    watcher.buffer_size = event_buffer_size;
    watcher.changes.allocator = state.owner;
    watcher.event_streams.allocator = state.owner;

    dispatch :: (state: *State, change: *File_Change) {
        target := cast(*T) state.watcher;
        target.callback(target, change, target.user_data);
    }

    state.dispatch = dispatch;
    lock_states();
    state.next = states;
    states = state;
    unlock_states();

    return true;
}

add_directories :: (using watcher: *$T/File_Watcher, dir_paths: ..string) -> bool {
    state := find_state(watcher.pool);

    if !state || state.watcher != watcher || state.stopping {
        B.assert(false, "uninitialized, relocated or stopping watcher");
        return false;
    }

    B.assert(watcher.buffer_size > 0, "watcher buffer size must remain positive");

    for path: dir_paths {
        record := state.spare_roots;
        if record state.spare_roots = record.next;
        else record = cast(*Root) Pool.get(state.pool, size_of(Root));
        if !record return false;

        record.* = .{};
        native := B.to_c_string(path);

        defer B.free(native);
        handle := c_create(native, 0.01, cast(u64) watcher.buffer_size);
        if !handle {
            record.next = state.spare_roots;
            state.spare_roots = record;
            context.logger("File_Watcher directory registration failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            return false;
        }

        record.handle = handle;
        bytes: u64;
        name := c_root(handle, *bytes);

        record.path = .{ cast(s64) bytes, name };
        record.next = state.roots;
        state.roots = record;
        stream := cast(Native.FSEventStreamRef) c_stream(handle);

        B.array_add(*watcher.event_streams, stream);
    }

    return true;
}

process_changes :: (using watcher: *File_Watcher) -> changed: bool, needs_wait: bool, wait_seconds: float64 {
    state := find_state(watcher.pool);

    if !state || state.watcher != watcher || state.delivering {
        B.assert(false, "uninitialized, relocated or recursively polled watcher");
        return false, false, 0;
    }

    if state.stopping {
        B.assert(false, "watcher teardown is incomplete");
        return false, false, 0;
    }

    B.assert(Math.is_finite(watcher.merge_window_seconds) && watcher.merge_window_seconds >= 0);
    root := state.roots;
    while root {
        while true {
            path: *u8;
            bytes: u64;
            flags: u32;
            time: float64;
            result := c_pop(root.handle, *path, *bytes, *flags, *time);

            if result < 0 {
                diagnostic := B.sprint("native watcher queue failed: %", c_error(root.handle));
                defer B.free(diagnostic);
                B.assert(false, diagnostic);
                return false, false, 0;
            }

            if result == 0 break;

            text: string = .{ cast(s64) bytes, path };

            queue_change(state, root.path, text, flags, time);
            c_free(path);
        }

        root = root.next;
    }

    B.assert(state.deadlines.count == watcher.changes.count, "watcher pending arrays were changed externally");
    now := c_time();
    if now < 0 {
        B.assert(false, "native watcher clock failed");
        return false, false, 0;
    }

    retained: s64;
    delivered := false;
    waiting := state.roots != null;
    delay: float64 = 0.1;

    state.delivering = true;
    defer state.delivering = false;

    for index: 0..watcher.changes.count - 1 {
        event := *watcher.changes[index];

        if now >= state.deadlines[index] {
            state.dispatch(state, event);
            B.assert(state.deadlines.count == watcher.changes.count, "callback changed watcher pending arrays");
            state.owner.proc(.FREE, 0, event.full_path.count, event.full_path.data, state.owner.data);
            delivered = true;
        } else {
            delay = B.min(delay, state.deadlines[index] - now);
            if retained != index watcher.changes[retained] = event.*;
            state.deadlines[retained] = state.deadlines[index];
            retained += 1;
        }
    }

    watcher.changes.count = retained;
    state.deadlines.count = retained;
    if !waiting delay = 0;

    return delivered, waiting, delay;
}

deinit :: (using watcher: $T/File_Watcher) {
    if !watcher.pool return;

    state := find_state(watcher.pool);

    if !state || state.delivering || state.closing {
        B.assert(false, "invalid or recursively invoked watcher deinit");
        return;
    }

    state.stopping = true;
    state.closing = true;
    root := state.roots;
    retained := false;
    reported_error := false;

    while root {
        if root.handle {
            released: s32;
            status := c_close(root.handle, *released);

            if released != 0 root.handle = null;

            if status != 0 {
                reported_error = true;
                context.logger("native watcher shutdown failed: ", context.logger_data, Log_Info.{ common_flags = .ERROR });
                description := c_strerror(status);
                if description {
                    count: s64;
                    while description[count] != 0 count += 1;
                    context.logger(.{ count, description }, context.logger_data, Log_Info.{ common_flags = .ERROR });
                }

                context.logger("\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            }

            if root.handle retained = true;
        }

        root = root.next;
    }

    if retained {
        state.closing = false;
        B.assert(false, "native watcher shutdown failed; resources retained for retry");
        return;
    }

    // All native objects were released; unregister before user allocator cleanup.
    lock_states();
    link := *states;
    while link.* && link.* != state link = *link.*.next;

    B.assert(link.* == state);
    link.* = state.next;
    unlock_states();
    for event: watcher.changes state.owner.proc(.FREE, 0, event.full_path.count, event.full_path.data, state.owner.data);

    B.array_free(watcher.changes);
    B.array_free(watcher.event_streams);
    B.array_free(state.deadlines);
    Pool.release(state.pool);
    state.owner.proc(.FREE, 0, size_of(Pool.Pool), state.pool, state.owner.data);
    owner := state.owner;

    owner.proc(.FREE, 0, size_of(State), state, owner.data);
    if reported_error B.assert(false, "native watcher shutdown reported a backend error");
} @Releases

#scope_file
B :: #import "Basic";
Pool :: #import "Pool";
Native :: #import "macOS";
Math :: #import "Math";

backend :: #library "native/portable_watch";
c_create :: (path: *u8, latency: float64, limit: u64) -> *void #foreign backend "portable_watch_create";
c_stream :: (handle: *void) -> *void #foreign backend "portable_watch_stream";
c_root :: (handle: *void, bytes: *u64) -> *u8 #foreign backend "portable_watch_root";
c_pop :: (handle: *void, path: **u8, bytes: *u64, flags: *u32, time: *float64) -> s32 #foreign backend "portable_watch_pop";
c_close :: (handle: *void, released: *s32) -> s32 #foreign backend "portable_watch_close_v2";
c_free :: (path: *void) #foreign backend "portable_watch_free";
c_time :: () -> float64 #foreign backend "portable_watch_time";
c_error :: (handle: *void) -> s32 #foreign backend "portable_watch_error";
c_classify :: (flags: u32) -> u32 #foreign backend "portable_watch_classify";
watcher_libc :: #system_library "libc";
c_strerror :: (error: s32) -> *u8 #foreign watcher_libc "strerror";

#assert cast(u64) File_Change_Event.ADDED == 1;
#assert cast(u64) File_Change_Event.MODIFIED == 2;
#assert cast(u64) File_Change_Event.MOVED == 4;
#assert cast(u64) File_Change_Event.ATTRIBUTES_CHANGED == 32;
#assert cast(u64) File_Change_Event.REMOVED == 64;
#assert cast(u64) File_Change_Event.SCAN_CHILDREN == 128;

Root :: struct {
    next: *Root;
    handle: *void;
    path: string;
}

State :: struct {
    next: *State;
    owner: Allocator;
    watcher: *void;
    pool: *Pool.Pool;
    changes: *[..] File_Change;
    events: *File_Change_Event;
    window: *float;
    recursive: *bool;
    deadlines: [..] float64;
    roots: *Root;
    spare_roots: *Root;
    dispatch: (state: *State, change: *File_Change);
    delivering: bool;
    stopping: bool;
    closing: bool;
}

states: *State;
gate: s32;

lock_states :: () {
    while !compare_and_swap(*gate, cast(s32) 0, cast(s32) 1) {}
}

unlock_states :: () {
    released := compare_and_swap(*gate, cast(s32) 1, cast(s32) 0);
    B.assert(released);
}

find_state :: (pool: *Pool.Pool) -> *State {
    lock_states();
    defer unlock_states();
    state := states;
    while state {
        if state.pool == pool return state;
        state = state.next;
    }

    return null;
}

queue_change :: (state: *State, root: string, path: string, flags: u32, time: float64) {
    if !state.recursive.* && path != root {
        prefix := root.count;
        if prefix > 0 && root[prefix - 1] != #char "/" prefix += 1;
        if path.count < prefix return;
        for i: prefix..path.count - 1 if path[i] == #char "/" return;
    }

    events := cast(File_Change_Event) c_classify(flags);
    if (events & state.events.*) == 0 return;

    B.assert(state.changes.count == state.deadlines.count);

    for *change, index: state.changes.* {
        if change.full_path == path {
            change.events |= events;
            change.time_of_last_change = cast(float) time;
            state.deadlines[index] = time + state.window.*;
            return;
        }
    }

    event: File_Change;
    event.full_path = .{ path.count, cast(*u8) state.owner.proc(.ALLOCATE, path.count, 0, null, state.owner.data) };
    B.assert(event.full_path.data != null, "watcher path allocation failed");
    memcpy(event.full_path.data, path.data, path.count);
    event.events = events;
    event.time_of_last_change = cast(float) time;
    B.array_add(state.changes, event);
    B.array_add(*state.deadlines, time + state.window.*);
}
