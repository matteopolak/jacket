init :: (watcher: *$T/File_Watcher, callback: T.File_Watcher_Callback, user_data: *T.User_Data = null, events_to_watch: File_Change_Event = .ALL_EVENTS, merge_window_seconds: float = 0.1, watch_recursively := true, verbose := false, event_buffer_size: s32 = DEFAULT_BUFFER_SIZE) -> success: bool {
    if !watcher || watcher.pool {
        B.assert(false, "watcher is null or already initialized");
        return false;
    }

    if !callback || !Math.is_finite(merge_window_seconds) || merge_window_seconds < 0 || event_buffer_size <= 0 return false;

    state := B.New(State);
    if !state return false;

    state.owner = context.allocator;
    state.watcher = watcher;
    state.changes = *watcher.changes;
    state.events = *watcher.events_to_watch;
    state.window = *watcher.merge_window_seconds;
    state.deadlines.allocator = state.owner;
    state.roots.allocator = state.owner;
    watcher.pool = B.New(Pool.Pool);
    if !watcher.pool {
        B.free(state);
        return false;
    }

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
    if !backend_init(watcher, state) {
        Pool.release(watcher.pool);
        B.free(watcher.pool);
        watcher.pool = null;
        B.free(state);

        return false;
    }

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
    if !state || state.watcher != watcher || state.closing || state.failed {
        B.assert(false, "uninitialized, relocated or failed watcher");
        return false;
    }

    push_context {
        context.allocator = state.owner;
        for path: dir_paths if !backend_add_directory(watcher, state, path) return false;
    }

    return true;
}

process_changes :: (using watcher: *File_Watcher) -> changed: bool, needs_wait: bool, wait_seconds: float64 {
    state := find_state(watcher.pool);
    if !state || state.watcher != watcher || state.closing || state.delivering || state.failed {
        B.assert(false, "uninitialized, relocated, recursively polled or failed watcher");
        return false, false, 0;
    }

    if !Math.is_finite(watcher.merge_window_seconds) || watcher.merge_window_seconds < 0 {
        B.assert(false, "invalid watcher merge window");
        return false, false, 0;
    }

    push_context {
        context.allocator = state.owner;

        if !backend_drain(watcher, state) {
            state.failed = true;
            return false, false, 0;
        }
    }

    now := B.seconds_since_init();
    if !Math.is_finite(now) {
        state.failed = true;
        return false, false, 0;
    }

    B.assert(state.deadlines.count == watcher.changes.count, "watcher pending arrays changed externally");
    retained: int;
    delivered := false;
    waiting := backend_active(watcher);
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
    waiting = waiting || retained > 0;

    return delivered, waiting, ifx waiting then delay else 0;
}

deinit :: (using watcher: $T/File_Watcher) {
    if !watcher.pool return;

    state := find_state(watcher.pool);
    if !state || state.closing || state.delivering {
        B.assert(false, "invalid or recursively invoked watcher deinit");
        return;
    }

    state.closing = true;

    push_context {
        context.allocator = state.owner;

        if !backend_deinit(*watcher, state) {
            state.closing = false;
            state.failed = true;
            return;
        }
    }

    lock_states();
    link := *states;
    while link.* && link.* != state link = *link.*.next;

    B.assert(link.* == state);
    link.* = state.next;
    unlock_states();
    for event: watcher.changes state.owner.proc(.FREE, 0, event.full_path.count, event.full_path.data, state.owner.data);

    B.array_free(watcher.changes);
    B.array_free(state.deadlines);
    B.array_free(state.roots);
    Pool.release(state.pool);
    state.owner.proc(.FREE, 0, size_of(Pool.Pool), state.pool, state.owner.data);
    owner := state.owner;

    owner.proc(.FREE, 0, size_of(State), state, owner.data);
}

#scope_module
B :: #import "Basic";
Math :: #import "Math";
Pool :: #import "Pool";

State :: struct {
    next: *State;
    owner: Allocator;
    watcher: *void;
    pool: *Pool.Pool;
    changes: *[..] File_Change;
    events: *File_Change_Event;
    window: *float;
    deadlines: [..] float64;
    roots: [..] string;
    dispatch: (state: *State, change: *File_Change);
    failed: bool;
    delivering: bool;
    closing: bool;
    #if OS == .LINUX {
        descriptor: s32 = -1;
        read_buffer: *void;
        read_capacity: s32;
        path_storage: *Pool.Pool;
        moves: [..] Pending_Move;
    }
    #if OS == .WINDOWS spare_directory: *Directory_Info;
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

report_error :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    context.logger("\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
}

queue_event :: (state: *State, path: string, events: File_Change_Event) -> bool {
    if (events & state.events.*) == 0 return true;

    now := B.seconds_since_init();
    if !Math.is_finite(now) return false;

    B.assert(state.changes.count == state.deadlines.count);

    for *change, index: state.changes.* {
        if change.full_path == path {
            change.events |= events;
            change.time_of_last_change = cast(float) now;
            state.deadlines[index] = now + state.window.*;
            return true;
        }
    }

    text: string = .{ path.count, cast(*u8) state.owner.proc(.ALLOCATE, path.count, 0, null, state.owner.data) };
    if !text.data {
        report_error("File_Watcher path allocation failed");
        return false;
    }

    memcpy(text.data, path.data, path.count);
    B.array_add(state.changes, File_Change.{ full_path = text, events = events, time_of_last_change = cast(float) now });
    B.array_add(*state.deadlines, now + state.window.*);

    return true;
}

pooled_path :: (pool: *Pool.Pool, path: string) -> string {
    data := cast(*u8) Pool.get(pool, path.count);
    if !data return "";
    memcpy(data, path.data, path.count);
    return .{ path.count, data };
}

join_path :: (root: string, name: string) -> string {
    if root.count && root[root.count - 1] == #char "/" return B.sprint("%%", root, name);
    return B.sprint("%/%", root, name);
}
