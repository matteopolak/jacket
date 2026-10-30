// What happened to a file, as flags that can be combined: `change.events & .MODIFIED != 0` asks whether one is set.
//
// - `ADDED`: the file or folder appeared, by creation or by being moved in.
// - `MODIFIED`: its content changed.
// - `MOVED`: it was renamed or moved. `MOVED_FROM` marks the old name and `MOVED_TO` the new one, where the system tells the two apart.
// - `ATTRIBUTES_CHANGED`: permissions, owner or times changed.
// - `REMOVED`: it is gone, by deletion or by being moved out.
// - `SCAN_CHILDREN`: the system cannot say exactly what changed below this folder (a new folder whose contents were not seen
//   yet, a full event queue, an unmounted file system); look at the folder's contents again. It comes with `MODIFIED`.
// - `ALL_EVENTS` is `ADDED`, `MODIFIED`, `MOVED`, `ATTRIBUTES_CHANGED` and `REMOVED` together, the default for `init`'s `events_to_watch`.
//
// The events that the system reports for one path within the merge window are combined into one `File_Change`.
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

// One report from a watcher: the `full_path` that changed, the `events` that happened to it, and `time_of_last_change`.
//
// `full_path` is the watched folder's path joined with the relative one, and may be spelled with links resolved (on macOS
// `/tmp/x` can show as `/private/tmp/x`). The string belongs to the watcher and is freed when the callback returns:
// copy it to keep it. `time_of_last_change` is a time in seconds from a clock that only goes forward; only differences between
// two are meaningful.
File_Change :: struct {
    full_path: string;
    events: File_Change_Event;
    time_of_last_change := 0.0;
}

DEFAULT_BUFFER_SIZE :: 8000;

// Watches folders for changes and calls a callback for them. `User_Data` is the type of the pointer handed to the callback; the
// default `void` is for a watcher with none.
//
// Use it in four steps: make an empty one (`watcher: File_Watcher;`), `init` it, `add_directories` for the folders to watch, and call
// `process_changes` regularly, for instance once per frame or loop; the callbacks run inside that call, on the calling thread. `deinit` releases it.
// The watcher must stay at the same address from `init` to `deinit`, because its native state points back at it, and `process_changes`
// must not be called from two threads at once or from inside a callback.
//
// The fields `callback`, `user_data`, `events_to_watch`, `merge_window_seconds`, `watch_recursively` and `buffer_size` are the
// settings `init` was given; `events_to_watch` and `merge_window_seconds` may be changed later. `changes`, `verbose` and the
// platform fields are the module's own. Memory comes from the `context.allocator` in force at `init`. Needs Linux, macOS or Windows
// (inotify, FSEvents and `ReadDirectoryChangesW` underneath); other targets stop at compile time.
//
// ```jai,no_run
// #import "Extensions/Environment";
// #import "File";
// #import "String";
//
// Seen :: struct {
//     found: bool;
// }
//
// on_change :: (watcher: *File_Watcher(Seen), change: *File_Change, seen: *Seen) {
//     if path_filename(change.full_path) == "jai-doc-watched.txt" && (change.events & (.ADDED | .MODIFIED)) != 0 seen.found = true;
// }
//
// main :: () {
//     base := environment_variable("TMPDIR");
//     if base == "" base = "/tmp";
//     root := tprint("%/jai-doc-file-watcher", base);
//     file := tprint("%/jai-doc-watched.txt", root);
//     assert(make_directory_if_it_does_not_exist(root));
//     defer {
//         file_delete(file);
//         delete_directory(root);
//     }
//
//     seen: Seen;
//     watcher: File_Watcher(Seen);
//     assert(init(*watcher, on_change, *seen));
//     defer deinit(watcher);
//     assert(add_directories(*watcher, root));
//
//     assert(write_entire_file(file, "hello"));
//     for 0..249 { // up to five seconds for the system to report it
//         process_changes(*watcher);
//         if seen.found break;
//         sleep_milliseconds(20);
//     }
//
//     assert(seen.found);
// }
// ```
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
    #if OS == .WINDOWS directories: [..] *Directory_Info;
    #if OS == .LINUX {
        inotify_instance: s32 = -1;
        inotify_data: *void;
        inotify_bytes_read: s64;
        inotify_cursor: s32;
        path_pool: *Pool.Pool;
        watch_paths: HT.Table(s32, string);
    }
}

#scope_file
Pool :: #import "Pool";
HT :: #import "Hash_Table";
