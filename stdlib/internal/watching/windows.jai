#scope_module
W :: #import "Windows";
F :: #import "File";
Text :: #import "internal/files/windows-text";
R :: #import "internal/watching/event-records";

// Opaque public pointers refer to these actual native operation records.
Directory_Info :: struct {
    next: *Directory_Info;
    path: string;
    handle: W.HANDLE = W.INVALID_HANDLE_VALUE;
    event: W.HANDLE;
    operation: W.OVERLAPPED;
    buffer: *u8;
    capacity: s32;
    pending: bool;
}

// WinError.h values and signature are verified against official Microsoft SDK
// documentation; these private bindings do not extend the Windows facade.
ERROR_OPERATION_ABORTED :: 995;
ERROR_IO_INCOMPLETE :: 996;
ERROR_NOTIFY_ENUM_DIR :: 1022;
ERROR_NOT_FOUND :: 1168;
watch_kernel32 :: #library,system "kernel32";
cancel_io :: (handle: W.HANDLE, operation: *W.OVERLAPPED) -> W.BOOL #foreign watch_kernel32 "CancelIoEx";

backend_init :: (watcher: *$T/File_Watcher, state: *State) -> bool {
    watcher.directories.allocator = state.owner;
    return true; // No OS operation exists until a directory is registered.
}

report_windows :: (operation: string, error: u32) {
    message := B.sprint("File_Watcher % failed (Win32 %)", operation, error);
    defer B.free(message);
    report_error(message);
}

arm_directory :: (watcher: *$T/File_Watcher, info: *Directory_Info) -> bool {
    B.assert(!info.pending);
    info.operation = .{};
    info.operation.hEvent = info.event;
    mask: s32 = W.FILE_NOTIFY_CHANGE_FILE_NAME | W.FILE_NOTIFY_CHANGE_DIR_NAME | W.FILE_NOTIFY_CHANGE_ATTRIBUTES | W.FILE_NOTIFY_CHANGE_SIZE | W.FILE_NOTIFY_CHANGE_LAST_WRITE | W.FILE_NOTIFY_CHANGE_SECURITY;
    returned: s32;
    result := W.ReadDirectoryChangesW(info.handle, info.buffer, info.capacity, ifx watcher.watch_recursively then 1 else 0, mask, *returned, *info.operation, null);

    if !result && cast(u32) W.GetLastError() != W.ERROR_IO_PENDING {
        report_windows("ReadDirectoryChangesW", cast(u32) W.GetLastError());
        return false;
    }

    info.pending = true;

    return true;
}

retire_directory :: (info: *Directory_Info) -> bool {
    if info.pending {
        if !cancel_io(info.handle, *info.operation) {
            error := cast(u32) W.GetLastError();
            if error != ERROR_NOT_FOUND {
                report_windows("CancelIoEx", error);
                return false;
            }
        }

        transferred: s32;
        if !W.GetOverlappedResult(info.handle, *info.operation, *transferred, 1) {
            error := cast(u32) W.GetLastError();
            if error == ERROR_IO_INCOMPLETE {
                report_windows("GetOverlappedResult cancellation incomplete", error);
                return false;
            }

            if error != ERROR_OPERATION_ABORTED report_windows("GetOverlappedResult canceled operation", error);
        }

        info.pending = false;
    }

    if info.handle != W.INVALID_HANDLE_VALUE {
        if !W.CloseHandle(info.handle) {
            report_windows("CloseHandle directory", cast(u32) W.GetLastError());
            return false;
        }

        info.handle = W.INVALID_HANDLE_VALUE;
    }

    if info.event {
        if !W.CloseHandle(info.event) {
            report_windows("CloseHandle event", cast(u32) W.GetLastError());
            return false;
        }

        info.event = null;
    }

    return true;
}

backend_add_directory :: (watcher: *$T/File_Watcher, state: *State, path: string) -> bool {
    absolute, valid := F.get_absolute_path(path);
    if !valid return false;

    defer B.free(absolute);
    for *byte: absolute if byte.* == #char "\\" byte.* = #char "/";
    for existing: watcher.directories if existing.path == absolute return true;

    native := Text.to_utf16(absolute);
    if !native return false;

    defer B.free(native);
    info := state.spare_directory;
    if info state.spare_directory = info.next;
    else info = cast(*Directory_Info) Pool.get(state.pool, size_of(Directory_Info));
    if !info return false;

    info.* = .{};
    info.path = B.copy_string(absolute);
    if !info.path {
        info.next = state.spare_directory;
        state.spare_directory = info;
        return false;
    }

    // Publish ownership before acquiring handles; a failed native close can then
    // retain this record for deinit retry without losing an OS operation.
    B.array_add(*watcher.directories, info);
    info.capacity = watcher.buffer_size;
    info.buffer = cast(*u8) state.owner.proc(.ALLOCATE, info.capacity, 0, null, state.owner.data);
    if !info.buffer {
        release_failed_directory(watcher, state, info);
        return false;
    }

    info.handle = W.CreateFileW(native, W.FILE_LIST_DIRECTORY, W.FILE_SHARE_READ | W.FILE_SHARE_WRITE | W.FILE_SHARE_DELETE, null, W.OPEN_EXISTING, W.FILE_FLAG_BACKUP_SEMANTICS | W.FILE_FLAG_OVERLAPPED, null);
    if info.handle == W.INVALID_HANDLE_VALUE {
        report_windows("CreateFileW directory", cast(u32) W.GetLastError());
        release_failed_directory(watcher, state, info);
        return false;
    }

    info.event = W.CreateEventW(null, 1, 0, null);
    if !info.event || !arm_directory(watcher, info) {
        if !info.event report_windows("CreateEventW", cast(u32) W.GetLastError());
        release_failed_directory(watcher, state, info);
        return false;
    }

    return true;
}

release_failed_directory :: (watcher: *$T/File_Watcher, state: *State, info: *Directory_Info) {
    if !retire_directory(info) {
        state.failed = true;
        return;
    }

    if info.buffer state.owner.proc(.FREE, 0, info.capacity, info.buffer, state.owner.data);

    info.buffer = null;
    B.free(info.path);
    info.path = "";
    B.assert(watcher.directories.count > 0 && watcher.directories[watcher.directories.count - 1] == info);
    watcher.directories.count -= 1;
    info.next = state.spare_directory;
    state.spare_directory = info;
} @Releases

classify_windows :: (action: u32) -> File_Change_Event {
    if action == W.FILE_ACTION_ADDED return .ADDED;
    if action == W.FILE_ACTION_REMOVED return .REMOVED;
    if action == W.FILE_ACTION_MODIFIED return .MODIFIED | .ATTRIBUTES_CHANGED;
    if action == W.FILE_ACTION_RENAMED_OLD_NAME return .MOVED | .MOVED_FROM;
    if action == W.FILE_ACTION_RENAMED_NEW_NAME return .MOVED | .MOVED_TO;

    return .SCAN_CHILDREN | .MODIFIED;
}

backend_drain :: (watcher: *$T/File_Watcher, state: *State) -> bool {
    for info: watcher.directories {
        if !info.pending {
            report_error("File_Watcher directory has no active read");
            return false;
        }

        transferred: s32;
        result := W.GetOverlappedResult(info.handle, *info.operation, *transferred, 0);
        error := ifx result then cast(u32) 0 else cast(u32) W.GetLastError();

        if !result && error == ERROR_IO_INCOMPLETE continue;

        info.pending = false;

        if !result && error != ERROR_NOTIFY_ENUM_DIR {
            report_windows("GetOverlappedResult", error);
            return false;
        }

        if !result || transferred == 0 {
            if !queue_event(state, info.path, .SCAN_CHILDREN | .MODIFIED) return false;
        } else {
            if transferred < 0 || transferred > info.capacity {
                report_error("File_Watcher invalid Windows notification extent");
                return false;
            }

            data: [] u8 = .{ cast(s64) transferred, info.buffer };
            cursor: s64;

            while cursor < data.count {
                record, next, valid := R.windows_record(data, cursor);
                if !valid {
                    report_error("File_Watcher malformed Windows notification");
                    return false;
                }

                cursor = next;
                name, converted := Text.from_utf16(cast(*u16) record.name.data, cast(s32) (record.name.count / 2));
                if !converted {
                    report_windows("notification UTF-16 conversion", cast(u32) W.GetLastError());
                    return false;
                }

                defer B.free(name);
                for *byte: name if byte.* == #char "\\" byte.* = #char "/";

                path := join_path(info.path, name);

                defer B.free(path);
                if !queue_event(state, path, classify_windows(record.flags)) return false;
            }
        }

        if !arm_directory(watcher, info) return false;
    }

    return true;
}

backend_active :: (watcher: *$T/File_Watcher) -> bool {
    return watcher.directories.count > 0;
}

backend_deinit :: (watcher: *$T/File_Watcher, state: *State) -> bool {
    released := true;

    for info: watcher.directories {
        if !retire_directory(info) {
            released = false;
            continue;
        }

        if info.buffer state.owner.proc(.FREE, 0, info.capacity, info.buffer, state.owner.data);

        info.buffer = null;
        B.free(info.path);
        info.path = "";
    }

    if !released return false;

    B.array_free(watcher.directories);

    return true;
} @Releases
