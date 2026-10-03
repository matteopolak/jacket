File_Visit_Info :: Visit.File_Visit_Info;

posix_is_directory :: (path: string) -> bool, success: bool {
    if !Native_Path.is_valid(path) return false, false;

    name := to_c_string(path);

    defer free(name);
    stat: P.stat_t;
    if P.stat(name, *stat) != 0 return false, false;

    return P.S_ISDIR(stat.st_mode), true;
}

posix_add_modtime_and_size_if_missing :: (info: *File_Visit_Info) -> bool {
    if info.modification_time.low != APOLLO_TIME_INVALID.low || info.modification_time.high != APOLLO_TIME_INVALID.high return !info.had_error;

    time, size, ok := file_modtime_and_size(info.full_name);

    info.modification_time = time;
    info.size = size;
    info.had_error = !ok;

    return ok;
}

posix_visit_files :: (dir_name: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), visit_files := true, visit_directories := false, visit_symlinks := true, follow_directory_symlinks := true) -> bool {
    ancestors: [..] Directory_Identity;
    defer array_free(ancestors);
    options := Visit_Options.{ visit_files, visit_directories, visit_symlinks, follow_directory_symlinks };
    return walk_directory(dir_name, recursive, user_data, proc, options, *ancestors);
}

file_exists :: (path: string) -> bool {
    if !Native_Path.is_valid(path) return false;

    name := to_c_string(path);

    defer free(name);
    stat: P.stat_t;

    return P.stat(name, *stat) == 0;
}

file_modtime_and_size :: (file_path: string) -> (modtime: Apollo_Time, size: s64, file_exists: bool) {
    if !Native_Path.is_valid(file_path) return APOLLO_TIME_INVALID, 0, false;

    name := to_c_string(file_path);

    defer free(name);
    stat: P.stat_t;
    if P.stat(name, *stat) != 0 return APOLLO_TIME_INVALID, 0, false;

    return stat_timestamp(stat), stat.st_size, true;
}

file_modtime :: (file_path: string) -> (modtime: Apollo_Time, file_exists: bool) {
    time, _, ok := file_modtime_and_size(file_path);
    return time, ok;
}

copy_file :: (source: string, dest: string) -> bool {
    if !Native_Path.is_valid(source) || !Native_Path.is_valid(dest) return false;

    source_name := to_c_string(source);

    defer free(source_name);
    dest_name := to_c_string(dest);

    defer free(dest_name);
    source_stat: P.stat_t;
    dest_stat: P.stat_t;

    if P.stat(source_name, *source_stat) != 0 return false;
    if P.S_ISDIR(source_stat.st_mode) return false;
    if P.stat(dest_name, *dest_stat) == 0 && source_stat.st_dev == dest_stat.st_dev && source_stat.st_ino == dest_stat.st_ino return false;

    input, ok := F.file_open(source);
    if !ok return false;

    defer F.file_close(*input);
    output, valid := F.file_open(dest, true);
    if !valid return false;

    defer F.file_close(*output);
    buffer: [65536] u8;

    while true {
        read_ok, count := F.file_read(input, buffer.data, buffer.count);
        if !read_ok return false;
        if count == 0 break;
        if !F.file_write(*output, buffer.data, count) return false;
    }

    return F.file_flush(output);
}

atomic_file_replace_or_just_rename_if_destination_file_does_not_exist :: (old_filename: string, new_filename: string) -> bool {
    if !Native_Path.is_valid(old_filename) || !Native_Path.is_valid(new_filename) return false;
    return F.file_move(old_filename, new_filename);
}

file_list :: (path: string, recursive := false, follow_directory_symlinks := true) -> [] string {
    result: [..] string;
    collector :: (info: *File_Visit_Info, result: *[..] string) {
        if !info.is_directory && !info.had_error array_add(result, copy_string(info.full_name));
    }

    ok := posix_visit_files(path, recursive, *result, collector, follow_directory_symlinks = follow_directory_symlinks);

    if !ok context.logger("file_list encountered a filesystem error\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    return result;
}

copy_directory :: (source: string, dest: string) -> bool {
    if !Native_Path.is_valid(source) || !Native_Path.is_valid(dest) return false;

    canonical, ok := F.get_absolute_path(source);
    if !ok return false;

    defer free(canonical);
    destination, valid := canonical_destination(dest);
    if !valid return false;

    defer free(destination);
    prefix := S.join(canonical, "/");

    defer free(prefix);
    if destination == canonical || canonical == "/" || S.begins_with(destination, prefix) return false;

    ancestors: [..] Directory_Identity;

    defer array_free(ancestors);

    // Walk directories directly, preserving empty directories too.
    return copy_tree(source, dest, *ancestors);
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
P :: #import "POSIX";
F :: #import "File";
S :: #import "String";
Paths :: #import "internal/files/paths";
Visit :: #import "internal/files/visit-info";
Native_Path :: #import "internal/native-path";

Directory_Identity :: struct {
    device: u64;
    inode: u64;
}

Visit_Options :: struct {
    files: bool;
    directories: bool;
    symlinks: bool;
    follow: bool;
}

walk_directory :: (path: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), options: Visit_Options, ancestors: *[..] Directory_Identity) -> bool {
    if !Native_Path.is_valid(path) return false;

    name := to_c_string(path);

    defer free(name);
    parent_stat: P.stat_t;
    if P.stat(name, *parent_stat) != 0 || !P.S_ISDIR(parent_stat.st_mode) return false;

    identity := Directory_Identity.{ cast(u64) parent_stat.st_dev, cast(u64) parent_stat.st_ino };
    // Only ancestor identities suppress descent; distinct paths to a directory remain visitable.
    for ancestor: ancestors.* if ancestor.device == identity.device && ancestor.inode == identity.inode return true;

    array_add(ancestors, identity);
    defer ancestors.count -= 1;
    directory := P.opendir(name);
    if !directory return false;

    success := true;

    while true {
        P.set_errno(cast(P.OS_Error_Code) 0);
        entry := P.readdir(directory);
        if !entry {
            if P.errno() != 0 success = false;
            break;
        }

        short := to_string(entry.d_name.data);
        if short == "." || short == ".." continue;

        full := S.path_join(path, short);

        defer free(full);
        cname := to_c_string(full);

        defer free(cname);
        stat: P.stat_t;
        info: File_Visit_Info;
        info.short_name = short;
        info.full_name = full;
        info.modification_time = APOLLO_TIME_INVALID;

        if P.lstat(cname, *stat) != 0 {
            info.had_error = true;
            success = false;
        } else {
            info.is_symlink = P.S_ISLNK(stat.st_mode);
            if info.is_symlink {
                target: P.stat_t;
                if P.stat(cname, *target) == 0 stat = target;
                else if P.errno() != cast(P.OS_Error_Code) P.ENOENT {
                    info.had_error = true;
                    success = false;
                }
            }

            info.is_directory = P.S_ISDIR(stat.st_mode);
            info.size = stat.st_size;
            info.modification_time = stat_timestamp(stat);
        }

        can_descend := recursive && info.is_directory && (!info.is_symlink || options.follow);

        info.descend_into_directory = can_descend;
        selected := ifx info.is_symlink then options.symlinks else ifx info.is_directory then options.directories else options.files;
        if selected proc(*info, user_data);

        if can_descend && info.descend_into_directory && !info.had_error {
            if !walk_directory(full, true, user_data, proc, options, ancestors) success = false;
        }
    }

    if P.closedir(directory) != 0 success = false;

    return success;
}

canonical_destination :: (path: string) -> string, bool {
    if path.count == 0 return "", false;

    remaining := S.trim_right(path, "/\\");
    if remaining.count == 0 remaining = "/";

    suffix: [..] string;

    defer array_free(suffix);

    while true {
        resolved, ok := F.get_absolute_path(remaining);
        if ok {
            builder: String_Builder;

            defer free_buffers(*builder);
            append(*builder, resolved);
            free(resolved);
            i := suffix.count;
            while i > 0 {
                i -= 1;
                append(*builder, cast(u8) 47);
                append(*builder, suffix[i]);
            }

            combined := builder_to_string(*builder);

            defer free(combined);
            normalized := Paths.parse_path(combined);

            defer array_free(normalized.words);

            return Paths.path_to_string(normalized), true;
        }

        if remaining == "." || remaining == "/" return "", false;

        leaf := S.path_filename(remaining);

        array_add(*suffix, leaf);
        parent := S.trim_right(S.path_strip_filename(remaining), "/\\");
        if parent.count > 0 remaining = parent;
        else if S.is_absolute_path(remaining) remaining = "/";
        else remaining = ".";
    }

    return "", false;
}

stat_timestamp :: (stat: P.stat_t) -> Apollo_Time #no_aoc {
    seconds := seconds_to_apollo(stat.st_mtime.tv_sec);
    nanos := nanoseconds_to_apollo(stat.st_mtime.tv_nsec);
    low := seconds.low + nanos.low;
    high := seconds.high + nanos.high;

    if low < seconds.low high += 1;

    next := low + APOLLO_TIME_FOR_JAN_1_1970.low;

    high += APOLLO_TIME_FOR_JAN_1_1970.high;
    if next < low high += 1;

    return Apollo_Time.{ next, high };
}

push_directory :: (path: string, ancestors: *[..] Directory_Identity) -> bool {
    name := to_c_string(path);

    defer free(name);
    stat: P.stat_t;
    if P.stat(name, *stat) != 0 || !P.S_ISDIR(stat.st_mode) return false;

    device := cast(u64) stat.st_dev;
    inode := cast(u64) stat.st_ino;

    for identity: ancestors.* if identity.device == device && identity.inode == inode return false;

    array_add(ancestors, Directory_Identity.{ device, inode });

    return true;
}

copy_tree :: (source: string, dest: string, ancestors: *[..] Directory_Identity) -> bool {
    if !push_directory(source, ancestors) return false;

    defer ancestors.count -= 1;
    if !F.make_directory_if_it_does_not_exist(dest, true) return false;

    name := to_c_string(source);

    defer free(name);
    directory := P.opendir(name);
    if !directory return false;

    copied := copy_directory_entries(directory, source, dest, ancestors);
    closed := P.closedir(directory) == 0;

    return copied && closed;
}

copy_directory_entries :: (directory: *P.DIR, source: string, dest: string, ancestors: *[..] Directory_Identity) -> bool {
    while true {
        P.set_errno(cast(P.OS_Error_Code) 0);
        entry := P.readdir(directory);
        if !entry return P.errno() == 0;

        short := to_string(entry.d_name.data);
        if short == "." || short == ".." continue;

        from := S.path_join(source, short);

        defer free(from);
        to := S.path_join(dest, short);

        defer free(to);
        cname := to_c_string(from);

        defer free(cname);
        stat: P.stat_t;
        if P.stat(cname, *stat) != 0 return false;

        if P.S_ISDIR(stat.st_mode) {
            if !copy_tree(from, to, ancestors) return false;
        } else if !copy_file(from, to) return false;
    }

    return false;
}
