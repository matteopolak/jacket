File_Visit_Info :: Visit.File_Visit_Info;

// Whether `path` is a directory, as `(is_directory, success)`. A link is followed. A path that cannot be examined (missing, no
// permission, or a NUL byte in it) gives `(false, false)`, so `success` tells a file from nothing at all.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Environment";
// #import "File";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// root := tprint("%/jai-doc-is-directory", base);
// file := tprint("%/file", root);
// defer {
//     file_delete(file);
//     delete_directory(root);
// }
//
// assert(make_directory_if_it_does_not_exist(root));
// assert(write_entire_file(file, "x"));
//
// directory, success := is_directory(root);
// assert(directory && success);
// directory, success = is_directory(file);
// assert(!directory && success);
// directory, success = is_directory(tprint("%/missing", root));
// assert(!directory && !success);
// assert(file_exists(file) && !file_exists(tprint("%/missing", root)));
// ```
is_directory :: (path: string) -> bool, success: bool {
    if !Native_Path.is_valid(path) return false, false;

    name := to_c_string(path);

    defer free(name);
    stat: P.stat_t;
    if P.stat(name, *stat) != 0 return false, false;

    return P.S_ISDIR(stat.st_mode), true;
}

// Fills in `modification_time` and `size` of `info` from `info.full_name` when the time is still `APOLLO_TIME_INVALID`.
// Returns whether the details are known: `false`, with `had_error` set, when they cannot be read. A record that already has
// a time is left alone and the answer is `!had_error`. `visit_files` fills these in itself on this target.
add_modtime_and_size_if_missing :: (info: *File_Visit_Info) -> bool {
    if info.modification_time.low != APOLLO_TIME_INVALID.low || info.modification_time.high != APOLLO_TIME_INVALID.high return !info.had_error;

    time, size, ok := file_modtime_and_size(info.full_name);

    info.modification_time = time;
    info.size = size;
    info.had_error = !ok;

    return ok;
}

// Calls `proc` with `user_data` for each entry of the directory `dir_name`, and returns `true` if every entry could be examined.
// With `recursive` the walk goes into sub-directories too.
//
// - The order is the file system's and is not sorted. `.` and `..` are skipped.
// - `visit_files`, `visit_directories` and `visit_symlinks` pick what `proc` hears about. A link counts as a link whatever it
//   points at, so with `visit_symlinks` off it is never reported as a file or a directory.
// - A directory is entered whether or not it is reported. `proc` can stop that by setting `info.descend_into_directory` to
//   `false`, which needs `visit_directories` so that `proc` sees the directory.
// - `follow_directory_symlinks` decides whether links to directories are entered. A directory that is already being walked
//   further up is not entered again, so link cycles end. Two different paths to one directory are both walked.
// - `info` and its strings are valid only during the call. A link whose target is missing is reported as a link, with the details of the link itself.
//
// A missing or unreadable `dir_name` returns `false` without calling `proc`. An entry whose details cannot be read is reported
// with `had_error` set and makes the result `false`, but the walk goes on.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Environment";
// #import "File";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// root := tprint("%/jai-doc-visit-files", base);
// defer {
//     file_delete(tprint("%/top", root));
//     file_delete(tprint("%/sub/leaf", root));
//     delete_directory(tprint("%/sub", root));
//     delete_directory(root);
// }
//
// assert(make_directory_if_it_does_not_exist(tprint("%/sub", root), recursive = true));
// assert(write_entire_file(tprint("%/top", root), "top"));
// assert(write_entire_file(tprint("%/sub/leaf", root), "leaf"));
//
// Count :: struct {
//     files: int;
//     directories: int;
//     bytes: s64;
// }
// count: Count;
// tally :: (info: *File_Visit_Info, count: *Count) {
//     if info.is_directory count.directories += 1;
//     else {
//         count.files += 1;
//         count.bytes += info.size;
//     }
// }
//
// assert(visit_files(root, true, *count, tally, visit_directories = true));
// assert(count.files == 2 && count.directories == 1 && count.bytes == 7);
//
// count = .{};
// assert(visit_files(root, false, *count, tally)); // top level only
// assert(count.files == 1);
// assert(!visit_files(tprint("%/missing", root), false, *count, tally));
// ```
visit_files :: (dir_name: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), visit_files := true, visit_directories := false, visit_symlinks := true, follow_directory_symlinks := true) -> bool {
    ancestors: [..] Directory_Identity;
    defer array_free(ancestors);
    options := Visit_Options.{ visit_files, visit_directories, visit_symlinks, follow_directory_symlinks };
    return walk_directory(dir_name, recursive, user_data, proc, options, *ancestors);
}

// Whether something exists at `path`: a file, a directory or anything else the system can examine. A link is followed,
// so a link whose target is missing does not count. `false` also for an empty path or one with a NUL byte.
file_exists :: (path: string) -> bool {
    if !Native_Path.is_valid(path) return false;

    name := to_c_string(path);

    defer free(name);
    stat: P.stat_t;

    return P.stat(name, *stat) == 0;
}

// When the file was last written, how many bytes it holds, and whether it exists: `(modtime, size, file_exists)`.
//
// For a missing file or a path with a NUL byte it returns `APOLLO_TIME_INVALID`, `0` and `false`. A link is followed. The
// time keeps the file system's resolution (nanoseconds on most Unix targets); compare two with `compare_apollo_times`.
// A directory's size is whatever the system reports for it.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Environment";
// #import "File";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-modtime.txt", base);
// defer file_delete(path);
//
// assert(write_entire_file(path, "copper"));
// time, size, found := file_modtime_and_size(path);
// assert(found && size == 6);
// same, still_found := file_modtime(path);
// assert(still_found && compare_apollo_times(time, same) == 0);
//
// _, _, found = file_modtime_and_size(tprint("%-missing", path));
// assert(!found);
// ```
file_modtime_and_size :: (file_path: string) -> (modtime: Apollo_Time, size: s64, file_exists: bool) {
    if !Native_Path.is_valid(file_path) return APOLLO_TIME_INVALID, 0, false;

    name := to_c_string(file_path);

    defer free(name);
    stat: P.stat_t;
    if P.stat(name, *stat) != 0 return APOLLO_TIME_INVALID, 0, false;

    return stat_timestamp(stat), stat.st_size, true;
}

// When the file was last written, and whether it exists; the same as `file_modtime_and_size` without the size.
file_modtime :: (file_path: string) -> (modtime: Apollo_Time, file_exists: bool) {
    time, _, ok := file_modtime_and_size(file_path);
    return time, ok;
}

// Copies the content of the file `source` to `dest`, which is created or, if it exists, replaced. Returns whether it all worked.
//
// The parent folder of `dest` must exist. Fails when `source` is missing or is a directory, when `dest` is the same file as
// `source` (it would be emptied first), or when either path has a NUL byte. If the copy fails half-way, `dest` keeps what was
// written. Permissions and times are not copied. `silent` has no effect here; on
// Windows a failure is logged unless it is `true`, and passing it keeps code the same on every target.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Environment";
// #import "File";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// source := tprint("%/jai-doc-copy-source.txt", base);
// dest := tprint("%/jai-doc-copy-dest.txt", base);
// defer {
//     file_delete(source);
//     file_delete(dest);
// }
//
// assert(write_entire_file(source, "copper"));
// assert(write_entire_file(dest, "a longer file that gets replaced"));
// assert(copy_file(source, dest));
// text, ok := read_entire_file(dest);
// assert(ok && text == "copper");
// free(text);
//
// assert(!copy_file(source, source, silent = true)); // onto itself
// assert(!copy_file(tprint("%-missing", source), dest, silent = true));
// ```
copy_file :: (source: string, dest: string, silent := false) -> bool {
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

// Moves `old_filename` to `new_filename`, replacing a file already there. Returns whether it worked.
//
// On Unix targets this is `rename`, which readers see as either the old file or the new one and never a mix; on Windows
// it is a replacing move. Both names must be on one file system. To change a file safely, write the new content to a
// second file in the same folder and move it over the old one.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Environment";
// #import "File";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// current := tprint("%/jai-doc-replace.txt", base);
// fresh := tprint("%/jai-doc-replace.new", base);
// defer {
//     file_delete(current);
//     file_delete(fresh);
// }
//
// assert(write_entire_file(current, "old"));
// assert(write_entire_file(fresh, "new"));
// assert(atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(fresh, current));
// assert(!file_exists(fresh));
// text, ok := read_entire_file(current);
// assert(ok && text == "new");
// free(text);
// ```
atomic_file_replace_or_just_rename_if_destination_file_does_not_exist :: (old_filename: string, new_filename: string) -> bool {
    if !Native_Path.is_valid(old_filename) || !Native_Path.is_valid(new_filename) return false;
    return F.file_move(old_filename, new_filename);
}

// The files below the folder `path`, as full paths: `path` joined with each name with `/`. Directories are not listed;
// links to files and links with a missing target are. With `recursive` it also lists the files of every sub-folder.
// `follow_directory_symlinks` is as in `visit_files`.
//
// The order is the file system's. The array and every string in it are new allocations from `context.allocator`: free each
// string and then the array. A missing folder or a read error is logged, and the files found before it are still returned.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Environment";
// #import "File";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// root := tprint("%/jai-doc-file-list", base);
// defer {
//     file_delete(tprint("%/a.txt", root));
//     file_delete(tprint("%/sub/b.txt", root));
//     delete_directory(tprint("%/sub", root));
//     delete_directory(root);
// }
//
// assert(make_directory_if_it_does_not_exist(tprint("%/sub", root), recursive = true));
// assert(write_entire_file(tprint("%/a.txt", root), "a"));
// assert(write_entire_file(tprint("%/sub/b.txt", root), "b"));
//
// top := file_list(root);
// assert(top.count == 1);
// for name: top free(name);
// array_free(top);
//
// all := file_list(root, recursive = true);
// assert(all.count == 2);
// for name: all free(name);
// array_free(all);
// ```
file_list :: (path: string, recursive := false, follow_directory_symlinks := true) -> [] string {
    result: [..] string;
    collector :: (info: *File_Visit_Info, result: *[..] string) {
        if !info.is_directory && !info.had_error array_add(result, copy_string(info.full_name));
    }

    ok := visit_files(path, recursive, *result, collector, follow_directory_symlinks = follow_directory_symlinks);

    if !ok context.logger("file_list encountered a filesystem error\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    return result;
}

// Copies the folder `source` and everything in it to `dest`, including empty sub-folders. Returns whether it all worked.
//
// `dest` and any missing parents are created; an existing `dest` is merged into, and files with the same names are replaced.
// It refuses (`false`, nothing copied) when `dest` is `source` or inside it, since that would never end. Links are followed
// and copied as what they point to, with link cycles cut. As with `copy_file`, permissions and times are not copied. A failure
// part-way leaves what was copied so far.
//
// ```jai,skip_on(wasm)
// #import "Extensions/Environment";
// #import "File";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// source := tprint("%/jai-doc-copy-tree", base);
// dest := tprint("%/jai-doc-copy-tree-copy", base);
// defer {
//     file_delete(tprint("%/sub/leaf", source));
//     delete_directory(tprint("%/sub", source));
//     delete_directory(source);
//     file_delete(tprint("%/sub/leaf", dest));
//     delete_directory(tprint("%/sub", dest));
//     delete_directory(dest);
// }
//
// assert(make_directory_if_it_does_not_exist(tprint("%/sub", source), recursive = true));
// assert(write_entire_file(tprint("%/sub/leaf", source), "copper"));
//
// assert(!copy_directory(source, tprint("%/inside", source))); // into itself
// assert(copy_directory(source, dest));
// text, ok := read_entire_file(tprint("%/sub/leaf", dest));
// assert(ok && text == "copper");
// free(text);
// ```
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

// How many leading words of `path` exist: `(fully_exists, how_much_exists)`. It checks `a`, then `a/b`, then `a/b/c` and so on, and
// stops at the first that is missing. The count is `0` when even the first is missing and the number of words when all of them exist.
// An empty path is `(true, 0)`.
//
// ```jai,skip_on(wasm)
// path := parse_path("jai-doc-no-such-folder/inner/file");
// defer array_free(path.words);
// exists, count := how_much_of_path_exists_on_the_local_filesystem(path);
// assert(!exists && count == 0);
// ```
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
P :: #import "internal/bindings/posix-api";
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
