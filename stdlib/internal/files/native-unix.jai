File :: struct {
    handle: *P.FILE;
}

file_open :: (name: string, for_writing := false, keep_existing_content := false, log_errors := true) -> File, bool {
    cname := to_c_string(name);

    defer free(cname);
    mode := "rb";

    if for_writing {
        if keep_existing_content mode = "r+b";
        else mode = "wb";
    }

    cmode := to_c_string(mode);

    defer free(cmode);
    handle := P.fopen(cname, cmode);
    if !handle && for_writing && keep_existing_content {
        fallback := to_c_string("w+b");
        defer free(fallback);
        handle = P.fopen(cname, fallback);
    }

    if !handle && log_errors context.logger("file_open failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    return File.{ handle }, handle != null;
}

file_close :: (file: *File) {
    if file.handle {
        result := P.fclose(file.handle);
        if result != 0 context.logger("file_close failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
    }

    file.handle = null;
}

is_valid :: (file: File) -> bool {
    return file.handle != null;
}

file_read :: (f: File, vdata: *void, bytes_to_read: s64) -> (success: bool, total_read: s64) {
    if bytes_to_read <= 0 || !vdata || !f.handle return false, 0;
    count := P.fread(vdata, 1, cast(u64) bytes_to_read, f.handle);
    return P.ferror(f.handle) == 0, cast(s64) count;
}

file_current_position :: (f: File) -> s64, bool {
    if !f.handle return 0, false;
    position := P.ftello(f.handle);
    return position, position >= 0;
}

file_set_position :: (f: File, pos: s64) -> bool {
    return f.handle != null && pos >= 0 && P.fseeko(f.handle, pos, P.SEEK_SET) == 0;
}

file_seek :: (file: File, pos: s64, type: Seek_Type) -> (success: bool) {
    if !file.handle return false;

    origin: s32;
    if type == .START origin = P.SEEK_SET;
    else if type == .CURRENT origin = P.SEEK_CUR;
    else if type == .END origin = P.SEEK_END;
    else return false;

    return P.fseeko(file.handle, pos, origin) == 0;
}

file_length :: (file: File) -> (length: s64, success: bool) {
    current, ok := file_current_position(file);
    if !ok return 0, false;
    if P.fseeko(file.handle, 0, P.SEEK_END) != 0 return 0, false;

    length, end_ok := file_current_position(file);
    restored := file_set_position(file, current);

    return length, end_ok && restored;
}

file_flush :: (file: File) -> bool {
    return file.handle != null && P.fflush(file.handle) == 0;
}

file_write :: (file: *File, data: *void, length: s64) -> bool {
    if !file.handle || length < 0 || (!data && length > 0) return false;
    if length == 0 return true;
    count := P.fwrite(data, 1, cast(u64) length, file.handle);
    return count == cast(u64) length && P.ferror(file.handle) == 0;
}

file_move :: (name_old: string, name_new: string) -> bool {
    old := to_c_string(name_old);

    defer free(old);
    new := to_c_string(name_new);

    defer free(new);

    return P.rename(old, new) == 0;
}

file_delete :: (filename: string) -> bool {
    name := to_c_string(filename);
    defer free(name);
    return P.unlink(name) == 0;
}

file_link :: (linkpath: string, filepath: string) -> bool {
    link := to_c_string(linkpath);

    defer free(link);
    target := to_c_string(filepath);

    defer free(target);

    return P.link(target, link) == 0;
}

delete_directory :: (dirname: string) -> bool {
    name := to_c_string(dirname);
    defer free(name);
    return P.rmdir(name) == 0;
}

make_directory_if_it_does_not_exist :: (name: string, recursive := false) -> directory_now_exists: bool {
    if recursive {
        parent := Strings.path_strip_filename(name);
        if parent.count > 1 && parent != name {
            parent = Strings.trim_right(parent, "/\\");
            if parent.count > 0 && !make_directory_if_it_does_not_exist(parent, true) return false;
        }
    }

    cname := to_c_string(name);

    defer free(cname);
    if P.mkdir(cname, cast(P.mode_t) 511) == 0 return true;

    #if OS == .WASM {
        directory := P.opendir(cname);
        if !directory return false;
        return P.closedir(directory) == 0;
    } else {
        stat: P.stat_t;
        return P.stat(cname, *stat) == 0 && P.S_ISDIR(stat.st_mode);
    }
}

get_absolute_path :: inline (relative_path: string) -> string, success: bool {
    name := to_c_string(relative_path);

    defer free(name);
    resolved := P.realpath(name, null);
    if !resolved return "", false;

    result := copy_string(to_string(resolved));

    c_free(resolved);

    return result, true;
}

#scope_file
#if OS == .WASM P :: #import "internal/files/wasi-libc";
else P :: #import "POSIX";

libc :: #system_library "libc";
c_free :: (memory: *void) #foreign libc "free";
