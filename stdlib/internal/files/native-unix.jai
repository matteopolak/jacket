// An open file. It holds the operating system's handle (a C `FILE *` here, a `HANDLE` on Windows) and nothing else, so a
// copy of a `File` is the same open file, not a second one. Get one from `file_open`, and give it back with `file_close`
// exactly once; closing through a copy leaves the other copy dangling.
//
// A zero `File` is not open: `is_valid` is `false` for it and every procedure fails on it. Reads and writes go through the
// C library's buffer, so written bytes can still be in memory until `file_flush` or `file_close`.
File :: struct {
    handle: *P.FILE;
}

// Opens the file `name` and returns it with `true`, or a closed file and `false` when that fails.
//
// - By default it opens an existing file for reading; a missing file is a failure.
// - `for_writing` opens for writing and creates the file when missing; an existing one is cut to nothing, unless
//   `keep_existing_content` is set, which keeps its bytes and starts at the first one (it does not append: use
//   `file_seek(file, 0, .END)` first). Without `for_writing` that flag does nothing.
// - A truncating open (`for_writing` alone) cannot be read back; a `keep_existing_content` open can. Seek when switching
//   between reading and writing on one file.
//
// `name` is a path in bytes: the C library takes them as they are. A name with a NUL byte in it is refused. A failure
// (a missing directory, no permission, a directory where a file is wanted for writing) is logged as an error through
// `context.logger` unless `log_errors` is `false`.
//
// ```jai
// #import "Extensions/Environment";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-file-open.txt", base);
// defer file_delete(path);
//
// _, found := file_open(path, log_errors = false);
// assert(!found); // reading needs an existing file
//
// file, ok := file_open(path, for_writing = true);
// assert(ok && file_write(*file, "hello"));
// file_close(*file);
//
// // Keeping the content starts at the first byte, so this overwrites it.
// file, ok = file_open(path, for_writing = true, keep_existing_content = true);
// assert(ok && file_write(*file, "J"));
// file_close(*file);
//
// text, read_ok := read_entire_file(path);
// assert(read_ok && text == "Jello");
// free(text);
// ```
file_open :: (name: string, for_writing := false, keep_existing_content := false, log_errors := true) -> File, bool {
    if !Native_Path.is_valid(name) {
        if log_errors context.logger("file_open failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return File.{}, false;
    }

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
        // The file does not exist yet: update mode cannot create it, so create it empty.
        create_mode := to_c_string("w+b");
        defer free(create_mode);
        handle = P.fopen(cname, create_mode);
    }

    if !handle && log_errors context.logger("file_open failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

    return File.{ handle }, handle != null;
} @Opens

// Closes `file` and sets it to the closed state. A file that is already closed, or was never opened, is left alone, so closing
// twice is safe. Buffered output is written first; a failure to do so is logged, and the file is closed all the same.
file_close :: (file: *File) {
    if file.handle {
        result := P.fclose(file.handle);
        if result != 0 context.logger("file_close failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
    }

    file.handle = null;
} @FreesArgument @Closes

// Whether `file` is open: `false` for a zero `File`, for one `file_open` failed to open, and for one `file_close` closed.
is_valid :: (file: File) -> bool {
    return file.handle != null;
}

// Reads up to `bytes_to_read` bytes from the current position into `vdata`, and returns `(success, total_read)`.
//
// `success` is `true` and `total_read` smaller than asked at the end of the file, and `0` right at the end: that is how a loop
// knows to stop. A closed file, a `null` buffer or a count of `0` or less fails with `0` bytes. A read error gives `false`
// with the bytes that did arrive. The position moves by `total_read`.
//
// ```jai
// #import "Extensions/Environment";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-file-read.bin", base);
// defer file_delete(path);
//
// assert(write_entire_file(path, "abcdef"));
// file, ok := file_open(path);
// assert(ok);
// defer file_close(*file);
//
// buffer: [4] u8;
// success, count := file_read(file, buffer.data, buffer.count);
// assert(success && count == 4 && buffer[0] == #char "a");
// success, count = file_read(file, buffer.data, buffer.count);
// assert(success && count == 2); // short: the end of the file
// success, count = file_read(file, buffer.data, buffer.count);
// assert(success && count == 0); // at the end
// ```
file_read :: (f: File, vdata: *void, bytes_to_read: s64) -> (success: bool, total_read: s64) {
    if bytes_to_read <= 0 || !vdata || !f.handle return false, 0;
    count := P.fread(vdata, 1, cast(u64) bytes_to_read, f.handle);
    return P.ferror(f.handle) == 0, cast(s64) count;
}

// The position of the next read or write, in bytes from the start of the file, and whether it could be read.
// A closed file gives `(0, false)`. See `file_tell` for the same answer with the results swapped.
file_current_position :: (f: File) -> s64, bool {
    if !f.handle return 0, false;
    position := P.ftello(f.handle);
    return position, position >= 0;
}

// Moves to byte `pos` counted from the start of the file. Returns `false` for a closed file or a negative `pos`.
// A position past the end is allowed; a write there fills the gap with zero bytes on most file systems.
file_set_position :: (f: File, pos: s64) -> bool {
    return f.handle != null && pos >= 0 && P.fseeko(f.handle, pos, P.SEEK_SET) == 0;
}

// Moves by `pos` bytes relative to `.START`, `.CURRENT` or `.END` of the file. A negative `pos` goes backwards. Returns
// `false` for a closed file or when the result would be before the start. Going past the end is allowed, as in `file_set_position`.
//
// ```jai
// #import "Extensions/Environment";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-file-seek.bin", base);
// defer file_delete(path);
//
// assert(write_entire_file(path, "abcdef"));
// file, ok := file_open(path);
// assert(ok);
// defer file_close(*file);
//
// assert(file_seek(file, -2, .END));
// tail, tail_ok := read_entire_file(file);
// assert(tail_ok && tail == "ef");
// free(tail);
//
// assert(file_seek(file, 1, .START) && file_seek(file, 2, .CURRENT));
// position, position_ok := file_current_position(file);
// assert(position_ok && position == 3);
// assert(!file_seek(file, -1, .START));
// ```
file_seek :: (file: File, pos: s64, type: Seek_Type) -> (success: bool) {
    if !file.handle return false;

    origin: s32;
    if type == .START origin = P.SEEK_SET;
    else if type == .CURRENT origin = P.SEEK_CUR;
    else if type == .END origin = P.SEEK_END;
    else return false;

    return P.fseeko(file.handle, pos, origin) == 0;
}

// The size of the file in bytes and whether it could be read. The position is put back where it was.
// It reports the size of a regular file; a pipe or terminal has none and fails.
file_length :: (file: File) -> (length: s64, success: bool) {
    current, ok := file_current_position(file);
    if !ok return 0, false;
    if P.fseeko(file.handle, 0, P.SEEK_END) != 0 return 0, false;

    length, end_ok := file_current_position(file);
    restored := file_set_position(file, current);

    return length, end_ok && restored;
}

// Hands buffered output to the operating system; returns `false` for a closed file or on an error. This makes the bytes visible
// to other readers of the file but does not force them to disk. `file_close` flushes too.
file_flush :: (file: File) -> bool {
    return file.handle != null && P.fflush(file.handle) == 0;
}

// Writes `length` bytes at `data` at the current position. Returns `false` for a closed file, a negative `length`, `null`
// `data` with a positive length, or a write error or short write (then an unknown part of the bytes may be in the file).
// A `length` of `0` succeeds. The bytes are buffered; see `file_flush`.
file_write :: (file: *File, data: *void, length: s64) -> bool {
    if !file.handle || length < 0 || (!data && length > 0) return false;
    if length == 0 return true;
    count := P.fwrite(data, 1, cast(u64) length, file.handle);
    return count == cast(u64) length && P.ferror(file.handle) == 0;
}

// Renames the file or directory `name_old` to `name_new`. Returns `false` if that fails.
//
// The rename replaces an existing file at `name_new` in one step, and fails across file systems or when `name_new`
// is a non-empty directory. On Windows it fails when `name_new` exists. For a replacement that works the same
// everywhere, see `File_Utilities.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist`.
file_move :: (name_old: string, name_new: string) -> bool {
    if !Native_Path.is_valid(name_old) || !Native_Path.is_valid(name_new) return false;

    old := to_c_string(name_old);

    defer free(old);
    new := to_c_string(name_new);

    defer free(new);

    return P.rename(old, new) == 0;
}

// Deletes the file `filename`; `false` if it does not exist or is a directory (see `delete_directory`).
// Other open handles on the file keep working until closed on Unix targets.
file_delete :: (filename: string) -> bool {
    if !Native_Path.is_valid(filename) return false;

    name := to_c_string(filename);
    defer free(name);
    return P.unlink(name) == 0;
}

// Creates the hard link `linkpath` to the existing file `filepath`, so both names are one file. Note the order: the new
// name comes first. Fails if `linkpath` exists, if `filepath` is a directory, or across file systems.
file_link :: (linkpath: string, filepath: string) -> bool {
    if !Native_Path.is_valid(linkpath) || !Native_Path.is_valid(filepath) return false;

    link := to_c_string(linkpath);

    defer free(link);
    target := to_c_string(filepath);

    defer free(target);

    return P.link(target, link) == 0;
}

// Removes the directory `dirname` if it is empty. Returns `false` if it is missing, not empty, or not a directory.
delete_directory :: (dirname: string) -> bool {
    if !Native_Path.is_valid(dirname) return false;

    name := to_c_string(dirname);
    defer free(name);
    return P.rmdir(name) == 0;
}

// Makes sure the directory `name` exists and returns whether it does afterwards: an existing directory is a success, a file
// of that name or a missing parent is a failure. With `recursive`, missing parents are created too, so the whole path
// exists on success.
//
// New directories get the permissions `0777` less the process mask on Unix targets. Both `/` and, on Windows, `\` work as
// separators. A failure part-way through a recursive call leaves the parents it made.
//
// ```jai
// #import "Extensions/Environment";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// root := tprint("%/jai-doc-make-directory", base);
// leaf := tprint("%/a/b", root);
// defer {
//     delete_directory(leaf);
//     delete_directory(tprint("%/a", root));
//     delete_directory(root);
// }
//
// assert(!make_directory_if_it_does_not_exist(leaf)); // the parents are missing
// assert(make_directory_if_it_does_not_exist(leaf, recursive = true));
// assert(make_directory_if_it_does_not_exist(leaf)); // already there
// ```
make_directory_if_it_does_not_exist :: (name: string, recursive := false) -> directory_now_exists: bool {
    if !Native_Path.is_valid(name) return false;

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

// The absolute form of `relative_path`, in a new string from `context.allocator` that the caller frees, and whether it worked.
//
// Here the path must exist: symbolic links in it are resolved, `.` and `..` are removed, and a missing path fails with
// `("", false)`. On Windows the path need not exist and links are not followed; the result uses `/` there too.
get_absolute_path :: inline (relative_path: string) -> string, success: bool {
    if !Native_Path.is_valid(relative_path) return "", false;

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
else P :: #import "internal/bindings/posix-api";

#import "internal/libc";
