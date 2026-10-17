File :: struct {
    handle: W.HANDLE;
}

File_Map_Info :: struct {
    file: File = .{ W.INVALID_HANDLE_VALUE };
    mapping: W.HANDLE = null;
    data: string;
}

#assert size_of(File) == 8;
#assert size_of(File_Map_Info) == 32;

CSIDL_APPDATA :: 0x001a;
KF_FLAG_CREATE :: 0x00008000;
FOLDERID_SavedGames_uid_string :: "4C5C32FF-BB9D-43b0-B5B4-2D72E54EAAA4";

file_open :: (name: string, for_writing := false, keep_existing_content := false, log_errors := false) -> File, bool {
    path := Text.utf16_path(name);

    defer free(path);
    file: File = .{ W.INVALID_HANDLE_VALUE };

    if path {
        access: u32 = W.GENERIC_READ;
        creation: u32 = W.OPEN_EXISTING;

        if for_writing {
            access |= W.GENERIC_WRITE;
            creation = W.CREATE_ALWAYS;
            if keep_existing_content creation = W.OPEN_ALWAYS;
        }

        file.handle = W.CreateFileW(path, access, W.FILE_SHARE_READ | W.FILE_SHARE_WRITE | W.FILE_SHARE_DELETE,
                                   null, creation, W.FILE_ATTRIBUTE_NORMAL, null);
    }

    ok := is_valid(file);
    if !ok && log_errors context.logger("file_open failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    return file, ok;
}

is_valid :: (file: File) -> bool {
    return file.handle != null && file.handle != W.INVALID_HANDLE_VALUE;
}

file_close :: (file: *File) {
    if is_valid(file.*) {
        if W.CloseHandle(file.handle) == 0 {
            context.logger("file_close failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            return;
        }
    }

    file.handle = W.INVALID_HANDLE_VALUE;
} @Releases

file_read :: (f: File, vdata: *void, bytes_to_read: s64) -> (success: bool, total_read: s64) {
    if !is_valid(f) || !vdata || bytes_to_read <= 0 return false, 0;

    total: s64;
    while total < bytes_to_read {
        chunk := cast(u32) min(bytes_to_read - total, 0x7fffffff);
        received: u32;
        ok := W.ReadFile(f.handle, cast(*u8) vdata + total, chunk, *received, null);

        total += received;

        if ok == 0 {
            error := W.GetLastError();
            // Synchronous pipes report their closed writer as BROKEN_PIPE;
            // HANDLE_EOF is the native explicit end-of-file status.
            if error == 109 || error == 38 return true, total;

            return false, total;
        }

        if received < chunk break;
    }

    return true, total;
}

file_write :: (file: *File, data: *void, length: s64) -> bool {
    if !is_valid(file.*) || length < 0 || (!data && length > 0) return false;

    total: s64;
    while total < length {
        chunk := cast(u32) min(length - total, 0x7fffffff);
        written: u32;
        ok := W.WriteFile(file.handle, cast(*u8) data + total, chunk, *written, null);

        total += written;
        if ok == 0 || written == 0 return false;
    }

    return true;
}

file_flush :: (file: File) -> bool {
    return is_valid(file) && W.FlushFileBuffers(file.handle) != 0;
}

file_length :: (file: File) -> s64, bool {
    if !is_valid(file) return 0, false;
    length: W.LARGE_INTEGER;
    ok := W.GetFileSizeEx(file.handle, *length) != 0;
    return length.QuadPart, ok;
}

file_current_position :: (f: File) -> s64, bool {
    if !is_valid(f) return 0, false;

    distance: W.LARGE_INTEGER;
    position: W.LARGE_INTEGER;
    ok := W.SetFilePointerEx(f.handle, distance, *position, .FILE_CURRENT) != 0;

    return position.QuadPart, ok;
}

file_seek :: (file: File, pos: s64, type: Seek_Type) -> (success: bool) {
    if !is_valid(file) return false;

    origin: W.SetFilePointer_MoveMethod;
    if type == .START origin = .FILE_BEGIN;
    else if type == .CURRENT origin = .FILE_CURRENT;
    else if type == .END origin = .FILE_END;
    else return false;

    distance: W.LARGE_INTEGER;
    distance.QuadPart = pos;

    return W.SetFilePointerEx(file.handle, distance, null, origin) != 0;
}

file_set_position :: (f: File, pos: s64) -> bool {
    return pos >= 0 && file_seek(f, pos, .START);
}

file_delete :: (filename: string) -> bool {
    path := Text.utf16_path(filename);
    defer free(path);
    return path != null && W.DeleteFileW(path) != 0;
}

file_move :: (name_old: string, name_new: string) -> bool {
    old := Text.utf16_path(name_old);

    defer free(old);
    new := Text.utf16_path(name_new);

    defer free(new);

    return old != null && new != null && W.MoveFileW(old, new) != 0;
}

file_link :: (link: string, filepath: string) -> bool {
    name := Text.utf16_path(link);

    defer free(name);
    target := Text.utf16_path(filepath);

    defer free(target);

    return name != null && target != null && W.CreateHardLinkW(name, target, null) != 0;
}

delete_directory :: (dirname: string) -> bool {
    path := Text.utf16_path(dirname);
    defer free(path);
    return path != null && W.RemoveDirectoryW(path) != 0;
}

make_directory_if_it_does_not_exist :: (name: string, recursive := false) -> directory_now_exists: bool {
    path := Text.utf16_path(name);

    defer free(path);
    if !path return false;
    if W.CreateDirectoryW(path, null) != 0 return true;

    attributes := W.GetFileAttributesW(path);

    if attributes != 0xffffffff return (attributes & W.FILE_ATTRIBUTE_DIRECTORY) != 0;
    if !recursive return false;

    // Existing native roots stop the walk before any attempt to create them.
    parent := Strings.trim_right(Strings.path_strip_filename(name), "/\\");
    if parent.count == 0 || parent == name return false;
    if !make_directory_if_it_does_not_exist(parent, true) return false;
    if W.CreateDirectoryW(path, null) != 0 return true;

    attributes = W.GetFileAttributesW(path);

    return attributes != 0xffffffff && (attributes & W.FILE_ATTRIBUTE_DIRECTORY) != 0;
}

get_absolute_path :: inline (relative_path: string) -> string, success: bool {
    path := Text.utf16_path(relative_path);

    defer free(path);
    if !path return "", false;

    capacity := W.GetFullPathNameW(path, 0, null, null);
    while capacity > 0 {
        buffer := cast(*u16) alloc(cast(s64) capacity * 2);

        defer free(buffer);
        count := W.GetFullPathNameW(path, capacity, buffer, null);
        if count == 0 return "", false;
        if count < capacity return Text.utf8_text(buffer, count);

        capacity = count;
    }

    return "", false;
}

file_get_canonical_path :: (file: File) -> string {
    if !is_valid(file) return "";

    capacity := W.GetFinalPathNameByHandleW(file.handle, null, 0, 0);
    while capacity > 0 {
        buffer := cast(*u16) alloc(cast(s64) capacity * 2);

        defer free(buffer);
        count := W.GetFinalPathNameByHandleW(file.handle, buffer, capacity, 0);
        if count == 0 return "";

        if count < capacity {
            result, ok := Text.utf8_text(buffer, count);
            if !ok return "";
            return result;
        }

        capacity = count;
    }

    return "";
}

file_get_canonical_path :: (filepath: string) -> string {
    path := Text.utf16_path(filepath);

    defer free(path);
    if !path return "";

    file := File.{ W.CreateFileW(path, 0, W.FILE_SHARE_READ | W.FILE_SHARE_WRITE | W.FILE_SHARE_DELETE,
                                null, W.OPEN_EXISTING, W.FILE_FLAG_BACKUP_SEMANTICS, null) };
    if !is_valid(file) return "";

    defer file_close(*file);

    return file_get_canonical_path(file);
}

map_file :: (name: string, log_errors := true) -> File_Map_Info, bool {
    result: File_Map_Info;
    opened: bool;
    result.file, opened = file_open(name, log_errors = log_errors);
    if !opened return result, false;

    length, sized := file_length(result.file);
    if sized && length == 0 return result, true;

    if sized && length > 0 {
        result.mapping = W.CreateFileMappingW(result.file.handle, null, W.PAGE_READONLY, 0, 0, null);
        if result.mapping {
            data := W.MapViewOfFile(result.mapping, W.FILE_MAP_READ, 0, 0, 0);
            if data {
                result.data = .{ length, cast(*u8) data };
                return result, true;
            }
        }
    }

    if log_errors context.logger("map_file failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    cleaned := munmap_file(*result);
    if !cleaned context.logger("map_file cleanup failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    return result, false;
}

munmap_file :: (file_map: *File_Map_Info) -> bool {
    if file_map.data.data {
        if W.UnmapViewOfFile(file_map.data.data) == 0 return false;
        file_map.data = "";
    }

    if file_map.mapping {
        if W.CloseHandle(file_map.mapping) == 0 return false;
        file_map.mapping = null;
    }

    if is_valid(file_map.file) {
        if W.CloseHandle(file_map.file.handle) == 0 return false;
        file_map.file.handle = W.INVALID_HANDLE_VALUE;
    }

    return true;
}

get_path_os_program_data :: () -> string {
    id := W.GUID.{ 0x3eb685db, 0x65f9, 0x4cf6, .[0xa0, 0x3a, 0xe3, 0xef, 0x65, 0x72, 0x9f, 0x3d] };
    return known_folder_path(*id);
}

get_path_os_saved_games :: () -> string {
    id := W.GUID.{ 0x4c5c32ff, 0xbb9d, 0x43b0, .[0xb5, 0xb4, 0x2d, 0x72, 0xe5, 0x4e, 0xaa, 0xa4] };
    return known_folder_path(*id);
}

#scope_file
W :: #import "Windows";
Text :: #import "internal/files/windows-text";

shell32 :: #system_library "shell32";
ole32 :: #system_library "ole32";
SHGetKnownFolderPath :: (id: *W.GUID, flags: u32, token: W.HANDLE, path: **u16) -> s32 #foreign shell32;
CoTaskMemFree :: (memory: *void) #foreign ole32;

known_folder_path :: (id: *W.GUID) -> string {
    path: *u16;
    status := SHGetKnownFolderPath(id, KF_FLAG_CREATE, null, *path);

    defer if path CoTaskMemFree(path);
    if status < 0 || !path return "";

    count: s64;
    while path[count] != 0 count += 1;

    result, ok := Text.utf8_text(path, count);
    if !ok return "";

    return result;
}
