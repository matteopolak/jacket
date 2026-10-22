# File_Watcher

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `File_Change :: struct`
- `File_Change_Event :: enum_flags`
- `DEFAULT_BUFFER_SIZE :: 8000`
- `File_Watcher :: struct (User_Data: Type = void)`
- `init :: (watcher: *$T/File_Watcher, callback: T.File_Watcher_Callback, user_data: *T.User_Data = null, events_to_watch: File_Change_Event = .ALL_EVENTS, merge_window_seconds: float = 0.1, watch_recursively := true, verbose := false, event_buffer_size: s32 = DEFAULT_BUFFER_SIZE) -> success: bool`
- `add_directories :: (using watcher: *$T/File_Watcher, dir_paths: ..string) -> bool`
- `deinit :: (using watcher: $T/File_Watcher)`
- `process_changes :: (using watcher: *File_Watcher) -> changed: bool, needs_wait: bool, wait_seconds: float64`

## Public type members

#### `File_Change` (struct)

- `full_path:  string`
- `events: File_Change_Event`
- `time_of_last_change := 0.0`

#### `File_Change_Event` (enum_flags)

- `ADDED`
- `MODIFIED`
- `MOVED`
- `MOVED_FROM`
- `MOVED_TO`
- `ATTRIBUTES_CHANGED`
- `REMOVED`
- `SCAN_CHILDREN`
- `ALL_EVENTS :: ADDED | MODIFIED | MOVED | ATTRIBUTES_CHANGED | REMOVED`

#### `File_Watcher` (struct)

- `File_Watcher_Callback :: #type (watcher: *File_Watcher(User_Data), change: *File_Change, user_data: *User_Data)`
- `callback: File_Watcher_Callback`
- `user_data: *User_Data`
- `events_to_watch: File_Change_Event`
- `merge_window_seconds: float`
- `watch_recursively: bool`
- `verbose: bool`
- `buffer_size: s32`
- `pool: *Pool`
- `changes: [..] File_Change`
- [OS == .WINDOWS] `directories: [..] *Directory_Info`
- [OS_HAS_INOTIFY] `inotify_instance: s32 = -1`
- [OS_HAS_INOTIFY] `inotify_data: *void`
- [OS_HAS_INOTIFY] `inotify_bytes_read: s64`
- [OS_HAS_INOTIFY] `inotify_cursor: s32`
- [OS_HAS_INOTIFY] `path_pool: *Pool`
- [OS_HAS_INOTIFY] `watch_paths: Table(s32, string)`
- [OS == .MACOS] `event_streams: [..] FSEventStreamRef`

## Behavioral contract and constraints

Call `init` to establish watcher state, `add_directories` to watch a directory tree, and `deinit` to stop watching. The public target variants are Windows (`directories: [..] *Directory_Info`), targets defining `OS_HAS_INOTIFY` (`inotify_instance: s32 = -1`, `inotify_data: *void`, `inotify_bytes_read: s64`, `inotify_cursor: s32`, `path_pool: *Pool`, and `watch_paths: Table(s32, string)`), macOS (`event_streams: [..] FSEventStreamRef`), and empty PS5/NN_SWITCH2 variants. Other targets hit the module's unsupported-target assertion. `Directory_Info` has module scope: qualified `W.Directory_Info` access is available, while plain imports and `using W` do not expose its name. The Windows `directories` field exposes pointers to these backend records; their layout is an implementation detail.

Event contracts: a rename may arrive as two `MOVED` events, one for each path; `MOVED_FROM` and `MOVED_TO` are modifiers when the backend can identify the old and new side. macOS may report only `MOVED`, and a move across the watched-tree boundary may report only one side. `SCAN_CHILDREN` tells the caller to inspect a directory's children; child changes are not automatically emitted merely because a directory was added or moved. `events_to_watch` is a request, but some platforms may still deliver event categories that were not requested. The event buffer is allocated once on Linux, once per directory on Windows, and is unused on macOS.

`File_Change.time_of_last_change` is a value measured in elapsed seconds from `Basic.seconds_since_init()`, not a wall-clock timestamp. `process_changes` returns `(changed, needs_wait, wait_seconds)`. The exact `changed` predicate is not specified. For polling use on Linux, the caller may poll `inotify_instance`; when `needs_wait` is true, it should call again after `wait_seconds`, even if polling produced no new event. Callers that invoke `process_changes` periodically may ignore those two wait fields. Overflow recovery behavior, cancellation of pending OS notifications, and callback guarantees during or after `deinit` are not specified by the public contract. `deinit` stops watching; the API does not specify whether the by-value watcher is reset or reusable after the call. Thread-safety guarantees are not specified.
