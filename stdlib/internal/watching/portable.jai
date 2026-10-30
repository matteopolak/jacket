// Sets up `watcher` and returns whether that worked. Call it on an empty watcher, once; afterwards add folders with `add_directories`.
//
// - `callback` is called by `process_changes` for each change, as `callback(watcher, change, user_data)`. It may not be `null`.
// - `user_data` is passed on to the callback.
// - `events_to_watch` selects the events worth reporting (default `.ALL_EVENTS`); others are dropped.
// - `merge_window_seconds` (default `0.1`) is how long a path must be quiet before its changes are reported, so a burst of writes
//   to one file gives one report. `0` reports at the next `process_changes`.
// - `watch_recursively` (default `true`) includes the sub-folders of watched folders, also ones made later.
// - `event_buffer_size` sizes the system's event buffer for each watched folder (default 8000); on Linux a burst of changes that
//   overflows it is reported as `SCAN_CHILDREN`.
// - `verbose` is kept in the watcher; the current backends do not use it.
//
// It returns `false`, and leaves the watcher empty, for a `null` callback, a negative or non-finite window, a buffer size of `0` or
// less, or when the system refuses. Calling it on a watcher that is already set up fails an assertion. Pair each successful call with `deinit`.
//
// ```jai
// on_change :: (watcher: *File_Watcher(void), change: *File_Change, data: *void) {}
//
// watcher: File_Watcher;
// assert(!init(*watcher, on_change, merge_window_seconds = -1));
// assert(!init(*watcher, on_change, event_buffer_size = 0));
// deinit(watcher); // nothing was set up, so nothing to release
// ```
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

// Starts watching each folder in `dir_paths` and returns `true` when all of them were added. It stops at the first one that
// cannot be watched (it is missing, is not a folder, or the system is out of watches) and returns `false`; the ones before it stay
// watched. The failure is logged through `context.logger`.
//
// With `watch_recursively` the sub-folders are watched too. Changes made before the call are not reported. Paths are as the
// file procedures take them; relative ones are resolved when added. Fails an assertion on a watcher that was not set up.
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

// Collects what the system has reported, calls the callback for every change whose merge window has passed, and returns
// `(changed, needs_wait, wait_seconds)`: whether any callback ran, whether more may come, and how long to wait before the next
// call in the worst case.
//
// It never blocks; call it from your own loop and sleep for `wait_seconds` (at most `0.1`) when `needs_wait` is `true`, as a game
// loop would rather call it every frame. `needs_wait` is `true` for as long as the watcher watches anything. The `change` handed to
// the callback is valid only during the call. The callback must not call `process_changes` or `deinit` on the same watcher.
// Fails an assertion on a watcher that was not set up, moved, or is being polled recursively.
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

// Stops watching and frees everything the watcher holds, including changes that were found but not yet delivered. Call it once
// for each successful `init`, and do not use the watcher afterwards: it takes the watcher by value, as `deinit(watcher)` without a
// `*`, so the caller's copy still points at what was freed. It is safe on a watcher that was never set up.
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
} @FreesArgument

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
