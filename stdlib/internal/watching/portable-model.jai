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
