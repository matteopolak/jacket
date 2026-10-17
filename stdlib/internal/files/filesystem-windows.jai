File_Visit_Info :: Visit.File_Visit_Info;

file_exists :: (file_path: string) -> bool {
    _, success := is_directory(file_path);
    return success;
}

is_directory :: (path: string) -> bool, success: bool {
    name := Text.to_utf16(path);
    if !name return false, false;

    defer free(name);
    attributes := W.GetFileAttributesW(name);
    if attributes == W.INVALID_FILE_ATTRIBUTES return false, false;

    return (attributes & W.FILE_ATTRIBUTE_DIRECTORY) != 0, true;
}

file_modtime_and_size :: (file_path: string) -> (modtime: Apollo_Time, size: s64, success: bool) {
    name := Text.to_utf16(file_path);
    if !name return APOLLO_TIME_INVALID, 0, false;

    defer free(name);
    data: ABI.Attribute_Data;
    if ABI.get_attributes(name, 0, *data) == 0 return APOLLO_TIME_INVALID, 0, false;

    size := ((cast(u64) data.size_high) << 32) | cast(u64) data.size_low;
    if size > 0x7fffffffffffffff return APOLLO_TIME_INVALID, 0, false;

    return filetime_to_apollo(data.written), cast(s64) size, true;
}

file_modtime :: (file_path: string) -> (modtime: Apollo_Time, success: bool) {
    time, _, success := file_modtime_and_size(file_path);
    return time, success;
}

add_modtime_and_size_if_missing :: inline (info: *File_Visit_Info) -> bool {
    if info.modification_time.low != APOLLO_TIME_INVALID.low || info.modification_time.high != APOLLO_TIME_INVALID.high return !info.had_error;

    time, size, success := file_modtime_and_size(info.full_name);

    info.modification_time = time;
    info.size = size;
    info.had_error = !success;

    return success;
}

copy_file :: (source: string, dest: string, silent := false) -> bool {
    from := Text.to_utf16(source);
    if !from return false;

    defer free(from);
    to := Text.to_utf16(dest);
    if !to return false;

    defer free(to);
    success := W.CopyFileW(from, to, 0) != 0;
    if !success && !silent context.logger("CopyFileW failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    return success;
}

atomic_file_replace_or_just_rename_if_destination_file_does_not_exist :: (old_filename: string, new_filename: string) -> bool {
    from := Text.to_utf16(old_filename);
    if !from return false;

    defer free(from);
    to := Text.to_utf16(new_filename);
    if !to return false;

    defer free(to);

    return ABI.replace_rename(from, to, 1) != 0;
}

visit_files :: (dir_name: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), visit_files := true, visit_directories := false, visit_symlinks := true, follow_directory_symlinks := true) -> bool {
    ancestors: [..] ABI.File_Identity;
    defer array_free(ancestors);
    options := Visit_Options.{ visit_files, visit_directories, visit_symlinks, follow_directory_symlinks };
    return walk_directory(dir_name, recursive, user_data, proc, options, *ancestors);
}

file_list :: (path: string, recursive := false, follow_directory_symlinks := true) -> [] string {
    result: [..] string;
    collector :: (info: *File_Visit_Info, result: *[..] string) {
        if !info.is_directory && !info.had_error array_add(result, copy_string(info.full_name));
    }

    success := visit_files(path, recursive, *result, collector, follow_directory_symlinks = follow_directory_symlinks);

    if !success context.logger("file_list encountered a filesystem error\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    return result;
}

copy_directory :: (source: string, dest: string) -> bool {
    source_id, valid := directory_identity(source);
    if !valid || dest.count == 0 return false;

    destination, resolved := absolute_path(dest);
    if !resolved return false;

    defer free(destination);
    ancestor := destination;
    while ancestor.count > 0 {
        if file_exists(ancestor) {
            identity, ok := directory_identity(ancestor);
            if !ok || same_identity(source_id, identity) return false;
        }

        parent := parent_path(ancestor);
        if parent.count >= ancestor.count break;

        ancestor = parent;
    }

    ancestors: [..] ABI.File_Identity;

    defer array_free(ancestors);

    return copy_tree(source, destination, *ancestors);
}

how_much_of_path_exists_on_the_local_filesystem :: (path: Paths.Path) -> (fully_exists: bool, how_much_exists: s32) {
    found: s32;

    for i: 1..path.words.count {
        prefix := Paths.path_to_string(path, i);
        exists := file_exists(prefix);

        free(prefix);
        if !exists return false, found;

        found = cast(s32) i;
    }

    return true, found;
}

#scope_file
#import "Basic";
W :: #import "Windows";
Text :: #import "internal/files/windows-text";
Visit :: #import "internal/files/visit-info";
ABI :: #import "internal/files/windows-abi";
S :: #import "String";
Paths :: #import "internal/files/paths";

Visit_Options :: struct {
    files: bool;
    directories: bool;
    symlinks: bool;
    follow: bool;
}

same_identity :: (a: ABI.File_Identity, b: ABI.File_Identity) -> bool {
    if a.volume != b.volume return false;
    for i: 0..15 if a.id[i] != b.id[i] return false;
    return true;
}

directory_identity :: (path: string) -> ABI.File_Identity, bool {
    directory, exists := is_directory(path);
    if !directory || !exists return .{}, false;

    name := Text.to_utf16(path);
    if !name return .{}, false;

    defer free(name);
    handle := W.CreateFileW(name, 0, W.FILE_SHARE_READ | W.FILE_SHARE_WRITE | W.FILE_SHARE_DELETE, flags_and_attributes = W.FILE_FLAG_BACKUP_SEMANTICS);
    if handle == W.INVALID_HANDLE_VALUE return .{}, false;

    identity: ABI.File_Identity;
    success := W.GetFileInformationByHandleEx(handle, W.FileIdInfo, *identity, size_of(ABI.File_Identity)) != 0;

    if W.CloseHandle(handle) == 0 success = false;

    return identity, success;
}

walk_directory :: (path: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), options: Visit_Options, ancestors: *[..] ABI.File_Identity) -> bool {
    identity, valid := directory_identity(path);
    if !valid return false;
    for ancestor: ancestors.* if same_identity(identity, ancestor) return true;

    array_add(ancestors, identity);
    defer ancestors.count -= 1;
    pattern := S.path_join(path, "*");

    defer free(pattern);
    name := Text.to_utf16(pattern);
    if !name return false;

    defer free(name);
    entry: W.WIN32_FIND_DATAW;
    handle := W.FindFirstFileW(name, *entry);

    if handle == W.INVALID_HANDLE_VALUE return W.GetLastError() == cast(W.OS_Error_Code) W.ERROR_FILE_NOT_FOUND;

    success := true;

    while true {
        short, converted := Text.from_utf16(entry.cFileName.data);
        if !converted success = false;
        else {
            defer free(short);

            if short != "." && short != ".." {
                full := S.path_join(path, short, separator = #char "/");

                defer free(full);
                info: File_Visit_Info;
                info.short_name = short;
                info.full_name = full;
                info.modification_time = filetime_to_apollo(entry.ftLastWriteTime);
                size := ((cast(u64) entry.nFileSizeHigh) << 32) | cast(u64) entry.nFileSizeLow;
                if size > 0x7fffffffffffffff {
                    info.had_error = true;
                    success = false;
                } else info.size = cast(s64) size;

                info.is_directory = (entry.dwFileAttributes & W.FILE_ATTRIBUTE_DIRECTORY) != 0;
                // Reparse points also include non-link facilities; recognize link/junction tags.
                info.is_symlink = entry.dwReserved0 == 0xa000000c || entry.dwReserved0 == 0xa0000003;
                can_descend := recursive && info.is_directory && (!info.is_symlink || options.follow);

                info.descend_into_directory = can_descend;
                selected := ifx info.is_symlink then options.symlinks else ifx info.is_directory then options.directories else options.files;
                if selected proc(*info, user_data);

                if can_descend && info.descend_into_directory && !info.had_error {
                    if !walk_directory(full, true, user_data, proc, options, ancestors) success = false;
                }
            }
        }

        if W.FindNextFileW(handle, *entry) == 0 {
            if W.GetLastError() != cast(W.OS_Error_Code) 18 success = false;
            break;
        }
    }

    if W.FindClose(handle) == 0 success = false;

    return success;
}

absolute_path :: (path: string) -> string, bool {
    name := Text.to_utf16(path);
    if !name return "", false;

    defer free(name);
    capacity: u32 = 256;

    while true {
        buffer := cast(*u16) alloc(cast(s64) capacity * 2);
        if !buffer return "", false;

        count := W.GetFullPathNameW(name, capacity, buffer, null);
        if count == 0 {
            free(buffer);
            return "", false;
        }

        if count < capacity {
            result, ok := Text.from_utf16(buffer, cast(s32) count);
            free(buffer);
            return result, ok;
        }

        free(buffer);
        if count > 0x7fffffff return "", false;

        capacity = count;
    }

    return "", false;
}

parent_path :: (path: string) -> string {
    trimmed := path;
    while trimmed.count > 0 && S.is_any(trimmed[trimmed.count - 1], "/\\") {
        if trimmed.count == 3 && trimmed[1] == 58 break;
        trimmed.count -= 1;
    }

    return S.path_strip_filename(trimmed);
}

make_directory :: (path: string) -> bool {
    directory, exists := is_directory(path);
    if exists return directory;

    parent := parent_path(path);
    if parent.count > 0 && parent.count < path.count {
        if !make_directory(parent) return false;
    }

    name := Text.to_utf16(path);
    if !name return false;

    defer free(name);
    if W.CreateDirectoryW(name, null) != 0 return true;

    directory, exists = is_directory(path);

    return exists && directory;
}

Copy_State :: struct {
    destination: string;
    ancestors: *[..] ABI.File_Identity;
    success := true;
}

copy_tree :: (source: string, dest: string, ancestors: *[..] ABI.File_Identity) -> bool {
    identity, valid := directory_identity(source);
    if !valid return false;
    for ancestor: ancestors.* if same_identity(identity, ancestor) return false;

    array_add(ancestors, identity);
    defer ancestors.count -= 1;
    if !make_directory(dest) return false;

    state := Copy_State.{ destination = dest, ancestors = ancestors };
    copier :: (info: *File_Visit_Info, state: *Copy_State) {
        if !state.success return;

        if info.had_error {
            state.success = false;
            return;
        }

        destination := S.path_join(state.destination, info.short_name);

        defer free(destination);
        if info.is_directory state.success = copy_tree(info.full_name, destination, state.ancestors);
        else state.success = copy_file(info.full_name, destination);
    }

    visited := visit_files(source, false, *state, copier, visit_directories = true);

    return visited && state.success;
}

filetime_to_apollo :: (time: W.FILETIME) -> Apollo_Time #no_aoc {
    ticks := ((cast(u64) time.dwHighDateTime) << 32) | cast(u64) time.dwLowDateTime;
    seconds := seconds_to_apollo(cast(s64) (ticks / 10000000) - 11644473600);
    fraction := nanoseconds_to_apollo(cast(s64) (ticks % 10000000) * 100);
    low := seconds.low + fraction.low;
    high := seconds.high + fraction.high;

    if low < seconds.low high += 1;

    epoch_low := low + APOLLO_TIME_FOR_JAN_1_1970.low;

    high += APOLLO_TIME_FOR_JAN_1_1970.high;
    if epoch_low < low high += 1;

    return .{ epoch_low, high };
}
