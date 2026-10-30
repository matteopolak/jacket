#if OS == .WINDOWS #load "native-windows.jai";
else #load "native-unix.jai";

// Whether `map_entire_file_start` maps the file into memory (Windows) or reads it into allocated memory (every other target).
USE_MAP_FILE :: OS == .WINDOWS;

// A switch kept for programs that mention it; it is always `false` and nothing in the module reads it.
STORE_MAPPED_FILENAMES :: false;

// The largest `s64`, kept for programs that compare against it. No procedure of this module returns it. Unix targets only.
DUMB_LINUX_FAILURE_NUMBER :: 0x7fffffffffffffff;

// A whole file's bytes, as `map_entire_file_start` returns them: `data` is the content, and the record must go back to `map_entire_file_end`.
//
// On Windows the bytes are a read-only view of the file (`map_info` holds the mapping) and writing to them faults;
// elsewhere they are a copy in memory from `context.allocator`. Code should treat `data` as read-only on both.
Map_File_Info :: struct {
    data: string;
    #if USE_MAP_FILE map_info: File_Map_Info;
}

// The current position of `file` as `(success, position)`.
//
// The same answer as `file_current_position`, with the two results in the other order: that one returns the
// position first. Both orders are kept because programs rely on each.
file_tell :: (file: File) -> (success: bool, position: s64) {
    position, ok := file_current_position(file);
    return ok, position;
}

// Writes all of `data` to `file`. Returns `false` when the file is not open or the write fails or stops short.
//
// An empty string succeeds without touching the file. After a failure part of the data may have been written.
file_write :: inline (file: *File, data: string) -> bool {
    return file_write(file, data.data, data.count);
}

// Writes the content of `builder` to `file`, then resets the builder unless `do_reset` is `false`.
//
// Returns `false` when the builder has failed or a write fails; an empty builder succeeds. The builder is reset even
// on failure, and it is still the caller's to free with `free_buffers`.
file_write :: (file: *File, builder: *String_Builder, do_reset := true) -> bool {
    defer {
        if do_reset reset(builder);
    }

    if builder.failed return false;
    if !builder.initialized return true;

    buffer := get_base_buffer(builder);
    while buffer {
        if !file_write(file, get_buffer_data(buffer), buffer.count) return false;
        buffer = buffer.next;
    }

    return true;
}

// Reads the rest of an open file, from its current position to its end, into a new string.
//
// Returns `("", false)` when the position or length cannot be read, the position is past the end, memory runs out,
// or the read fails. With `zero_terminated`, one extra `0` byte follows the text and is not counted in `count`, so the
// result can go to C. The string comes from `context.allocator` and the caller frees it; the file stays open, at its end.
//
// ```jai
// #import "Extensions/Environment";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-read-rest.bin", base);
// defer file_delete(path);
//
// assert(write_entire_file(path, "abcdef"));
// file, ok := file_open(path);
// assert(ok);
// defer file_close(*file);
//
// assert(file_set_position(file, 2));
// rest, read_ok := read_entire_file(file, zero_terminated = true);
// assert(read_ok && rest == "cdef" && rest.data[rest.count] == 0);
// free(rest);
// ```
read_entire_file :: (f: File, zero_terminated := false) -> string, bool {
    position, ok := file_current_position(f);
    if !ok return "", false;

    length, valid := file_length(f);
    if !valid || length < position return "", false;

    count := length - position;
    allocated := count;

    if zero_terminated {
        if allocated == 0x7fffffffffffffff return "", false;
        allocated += 1;
    }

    memory := cast(*u8) alloc(allocated);
    if !memory && allocated > 0 return "", false;

    result := string.{ count, memory };

    if count > 0 {
        read_ok, total := file_read(f, result.data, count);
        if !read_ok {
            free(result);
            return "", false;
        }

        result.count = total;
    }

    if zero_terminated result.data[result.count] = 0;

    return result, true;
} @Allocates

// Reads the whole file `name` into a new string. Returns `("", false)` when it cannot be opened or read.
//
// The string comes from `context.allocator` and the caller frees it, also for an empty file, which gives an empty
// string with `true`. The content is bytes: it may hold `0`, line endings are not translated, and it is not checked
// for UTF-8. `zero_terminated` adds an uncounted `0` byte after the text. When the file cannot be opened,
// Nothing is printed for a missing file: the result says so. `log_errors = true` reports it through `context.logger`.
//
// ```jai
// #import "Extensions/Environment";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-read-entire.txt", base);
// defer file_delete(path);
//
// _, found := read_entire_file(path);
// assert(!found);
//
// assert(write_entire_file(path, "one\r\ntwo\0"));
// text, ok := read_entire_file(path);
// assert(ok && text.count == 9 && text[8] == 0);
// free(text);
// ```
read_entire_file :: (name: string, zero_terminated := false, log_errors := false) -> string, bool {
    file, ok := file_open(name, log_errors = log_errors);
    if !ok return "", false;
    defer file_close(*file);
    return read_entire_file(file, zero_terminated);
} @Allocates

// Replaces the file `name` with `data`: it is created if missing and cut to nothing otherwise. Returns `false` when it cannot be
// opened, written or flushed.
//
// On failure the file may be left empty or partly written; the old content is not kept. The name is a path in UTF-8.
// To change a file without ever exposing a partial result, write a second file and `file_move` it into place.
//
// ```jai
// #import "Extensions/Environment";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-write-entire.txt", base);
// defer file_delete(path);
//
// assert(write_entire_file(path, "a long first version"));
// assert(write_entire_file(path, "short"));
// text, ok := read_entire_file(path);
// assert(ok && text == "short");
// free(text);
// assert(!write_entire_file(tprint("%/jai-doc-no-such-dir/file", base), "x"));
// ```
write_entire_file :: inline (name: string, data: string) -> bool {
    return write_entire_file(name, data.data, data.count);
}

// Replaces the file `name` with `count` bytes at `data`, as the string form does. A `count` of `0` empties the file;
// `data` may then be `null`. A negative `count`, or `null` data with a positive count, fails without writing.
write_entire_file :: (name: string, data: *void, count: int) -> bool {
    file, ok := file_open(name, true);
    if !ok return false;
    defer file_close(*file);
    return file_write(*file, data, count) && file_flush(file);
}

// Replaces the file `name` with the content of `builder`, then resets the builder unless `do_reset` is `false`.
//
// The builder is reset even when the file cannot be opened or written, so the text is gone after a failure.
write_entire_file :: (name: string, builder: *String_Builder, do_reset := true) -> bool {
    file, ok := file_open(name, true);
    if !ok {
        if do_reset reset(builder);
        return false;
    }

    defer file_close(*file);

    return file_write(*file, builder, do_reset) && file_flush(file);
}

// Wraps text you already own as a `Map_File_Info`, so it can go through code that expects one.
//
// On Windows `map_entire_file_end` leaves such a wrapper's text alone; elsewhere it frees `data`, so the text must then
// have been allocated with `context.allocator`.
string_to_map_file_info :: (data: string) -> Map_File_Info {
    return .{ data = data };
}

// Gives a file's whole content as `Map_File_Info.data`; returns `ok` as `false` and empty data when the file cannot be read.
//
// The content is read-only and must be released with `map_entire_file_end`. On Windows it is mapped, so a large file costs
// no copy and the file stays open until the end call; elsewhere it is read into memory, as `read_entire_file` does.
// `log_errors = true` reports an open failure through `context.logger`.
//
// ```jai
// #import "Extensions/Environment";
// base := environment_variable("TMPDIR");
// if base == "" base = environment_variable("TEMP");
// if base == "" base = "/tmp";
// path := tprint("%/jai-doc-map-file.txt", base);
// defer file_delete(path);
//
// assert(write_entire_file(path, "mapped text"));
// info, ok := map_entire_file_start(path);
// assert(ok && info.data == "mapped text");
// map_entire_file_end(*info);
// assert(info.data.count == 0);
// ```
map_entire_file_start :: (name: string, log_errors := false) -> Map_File_Info, bool {
    #if USE_MAP_FILE {
        mapping, ok := map_file(name, log_errors);
        return .{ data = mapping.data, map_info = mapping }, ok;
    } else {
        data, ok := read_entire_file(name, log_errors = log_errors);
        return .{ data = data }, ok;
    }
}

// Releases what `map_entire_file_start` returned and leaves `data` empty: it unmaps the view on Windows and frees the
// text elsewhere. Do not use the old `data` afterwards. A failure to unmap is logged and `data` keeps its value.
map_entire_file_end :: (file_info: *Map_File_Info) {
    #if USE_MAP_FILE {
        if !munmap_file(*file_info.map_info) {
            context.logger("map_entire_file_end failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            return;
        }
    } else free(file_info.data);

    file_info.data = "";
}

#scope_module
Seek_Type :: enum {
    START :: 0;
    CURRENT :: 1;
    END :: 2;
}

#import "Basic";
Strings :: #import "String";
Native_Path :: #import "internal/native-path";
