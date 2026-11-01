// Files for fibers. On io_uring the reads and writes run in the kernel and only the calling fiber
// waits. On epoll and kqueue, which cannot report a regular file as "not ready", they run at once
// on the scheduler's thread (blocking-with-readiness): a slow disk stalls every fiber of that
// thread for the length of the call. Opening a file is always a plain call.

// How [open_file] opens a file.
File_Mode :: enum u8 {
    // An existing file, read only.
    READ;
    // Created if missing, emptied if present, write only.
    WRITE;
    // Created if missing, writes go to the end.
    APPEND;
    // Created if missing, kept as it is, read and write.
    READ_WRITE;
}

// An open file. `read` and `write` use and advance `position`; `read_at` and `write_at` do not. It has a position, so every procedure takes it by pointer.
Io_File :: struct {
    handle: Handle;
    position: s64;
    // Opened with `.APPEND`: every write goes to the end, whatever the offset.
    append: bool;
    // `open_stream_file` restores these POSIX status flags before closing a duplicated stream.
    restore_status_flags: s32 = -1;
}

// Opens `path`. Close the file with [close].
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     file, error := Io.open_file("/tmp/jacket-io-open.txt", .WRITE);
//     assert(error == .NONE);
//     Io.write(*file, "hello");
//     Io.close(*file);
//
//     _, error = Io.open_file("/tmp/jacket-io-missing/none.txt");
//     assert(error == .NOT_FOUND);
// });
// file_delete("/tmp/jacket-io-open.txt");
// ```
open_file :: (path: string, mode := File_Mode.READ) -> Io_File, Io_Error {
    current_fiber();

    flags: s32;

    if mode == {
        case .READ; flags = O_RDONLY;
        case .WRITE; flags = O_WRONLY | O_CREAT | O_TRUNC;
        case .APPEND; flags = O_WRONLY | O_CREAT | O_APPEND;
        case .READ_WRITE; flags = O_RDWR | O_CREAT;
    }

    fd := sys_open(temp_c_string(path), flags, 0x1a4);
    if fd < 0 return .{}, error_from_result(fd);

    file: Io_File;
    file.handle = .{ fd, .FILE };
    file.append = mode == .APPEND;

    return file, .NONE;
}

// Opens a POSIX byte stream (a terminal, FIFO, or `/dev/stdin`) for non-blocking reads.
// Unlike `open_file`, reads use the current position rather than pread. Close with `close`.
// Windows returns `.UNSUPPORTED`. Closing leaves stdin open and restores its status flags when
// the path duplicated a descriptor (`/dev/fd` on macOS). While open, that shared descriptor is
// non-blocking too; do not read it elsewhere concurrently.
//
// ```jai,no_run
// input, error := open_stream_file("/dev/stdin");
// if error == .NONE {
//     defer close(*input);
//     lines := make_line_reader(reader_of(*input));
//     defer free_line_reader(*lines);
//     line, read_error := read_line(*lines);
// }
// ```
open_stream_file :: (path: string) -> Io_File, Io_Error {
    current_fiber();

    #if OS == .WINDOWS {
        return .{}, .UNSUPPORTED;
    } else {
        fd := sys_open(temp_c_string(path), O_RDONLY | O_NONBLOCK, 0);
        if fd < 0 return .{}, error_from_result(fd);

        // /dev/fd paths may duplicate a descriptor and ignore open's O_NONBLOCK flag.
        flags := c_fcntl(cast(s32) fd, F_GETFL);
        prepared := sys_nonblocking(cast(s32) fd);

        if flags < 0 || prepared < 0 {
            if flags >= 0 c_fcntl(cast(s32) fd, F_SETFL, flags);
            sys_close(fd);
            return .{}, .OTHER;
        }

        return .{ handle = .{ fd, .STREAM }, restore_status_flags = flags }, .NONE;
    }
}

// Reads up to `buffer.count` bytes at the file's position. 0 at the end of the file.
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     Io.write_file("/tmp/jacket-io-read.txt", "hello");
//
//     file, _ := Io.open_file("/tmp/jacket-io-read.txt");
//     defer Io.close(*file);
//
//     buffer: [16] u8;
//     count, error := Io.read(*file, buffer);
//     assert(error == .NONE && string.{ count, buffer.data } == "hello");
// });
// file_delete("/tmp/jacket-io-read.txt");
// ```
read :: (file: *Io_File, buffer: [] u8) -> s64, Io_Error {
    count, error := read_at(file, buffer, file.position);
    file.position += count;

    return count, error;
}

// Reads until `buffer` is full. `.END_OF_FILE` if the file ended first, with the bytes read.
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     Io.write_file("/tmp/jacket-io-full.txt", "abc");
//
//     file, _ := Io.open_file("/tmp/jacket-io-full.txt");
//     defer Io.close(*file);
//
//     buffer: [8] u8;
//     count, error := Io.read_full(*file, buffer);
//     assert(error == .END_OF_FILE && count == 3);
// });
// file_delete("/tmp/jacket-io-full.txt");
// ```
read_full :: (file: *Io_File, buffer: [] u8) -> s64, Io_Error {
    return read_full(reader_of(file), buffer);
}

// Reads at `offset`; the file's position does not move.
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     Io.write_file("/tmp/jacket-io-at.txt", "0123456789");
//
//     file, _ := Io.open_file("/tmp/jacket-io-at.txt");
//     defer Io.close(*file);
//
//     buffer: [3] u8;
//     Io.read_at(*file, buffer, 4);
//     assert(string.{ 3, buffer.data } == "456");
// });
// file_delete("/tmp/jacket-io-at.txt");
// ```
read_at :: (file: *Io_File, buffer: [] u8, offset: s64) -> s64, Io_Error {
    if buffer.count == 0 return 0, .NONE;

    op := begin_op(.READ, file.handle);
    op.buffer = buffer.data;
    op.length = buffer.count;
    op.offset = offset;

    error := await_op(op);
    count := op.result;

    end_op(op);

    if error != .NONE return 0, error;

    return count, .NONE;
}

// Writes some of `data` at the file's position (at the end, for `.APPEND`) and moves the
// position past it. At least one byte unless `data` is empty; returns how many. Use [write] to
// write everything.
write_some :: (file: *Io_File, data: [] u8) -> s64, Io_Error {
    if data.count == 0 return 0, .NONE;

    op := begin_op(.WRITE, file.handle);
    op.buffer = data.data;
    op.length = data.count;
    op.offset = ifx file.append then -1 else file.position;

    error := await_op(op);
    count := op.result;

    end_op(op);

    if error != .NONE return 0, error;

    file.position += count;

    return count, .NONE;
}

// Writes all of `data` at the file's position (at the end, for `.APPEND`).
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     file, _ := Io.open_file("/tmp/jacket-io-write.txt", .WRITE);
//     defer Io.close(*file);
//
//     count, error := Io.write(*file, u8.[1, 2, 3]);
//     assert(error == .NONE && count == 3);
// });
// file_delete("/tmp/jacket-io-write.txt");
// ```
write :: (file: *Io_File, data: [] u8) -> s64, Io_Error {
    return write(writer_of(file), data);
}

// The `write` for a `string`.
write :: (file: *Io_File, text: string) -> s64, Io_Error {
    return write(file, ([] u8).{ text.count, text.data });
}

// Writes all of `data` at `offset`; the file's position does not move.
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     file, _ := Io.open_file("/tmp/jacket-io-write-at.txt", .READ_WRITE);
//     defer Io.close(*file);
//
//     Io.write_at(*file, "abcdef", 0);
//     Io.write_at(*file, "XY", 2);
//
//     data, _ := Io.read_file("/tmp/jacket-io-write-at.txt");
//     assert(cast(string) data == "abXYef");
//     free(data.data);
// });
// file_delete("/tmp/jacket-io-write-at.txt");
// ```
write_at :: (file: *Io_File, data: [] u8, offset: s64) -> s64, Io_Error {
    total := 0;

    while total < data.count {
        op := begin_op(.WRITE, file.handle);
        op.buffer = data.data + total;
        op.length = data.count - total;
        op.offset = ifx file.append then -1 else offset + total;

        error := await_op(op);
        count := op.result;

        end_op(op);

        if error != .NONE return total, error;
        if count == 0 return total, .NO_SPACE;

        total += count;
    }

    return total, .NONE;
}

// The `write_at` for a `string`.
write_at :: (file: *Io_File, text: string, offset: s64) -> s64, Io_Error {
    return write_at(file, ([] u8).{ text.count, text.data }, offset);
}

// The size of the file in bytes.
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     Io.write_file("/tmp/jacket-io-size.txt", "12345");
//
//     file, _ := Io.open_file("/tmp/jacket-io-size.txt");
//     defer Io.close(*file);
//
//     size, error := Io.file_size(*file);
//     assert(error == .NONE && size == 5);
// });
// file_delete("/tmp/jacket-io-size.txt");
// ```
file_size :: (file: *Io_File) -> s64, Io_Error {
    current_fiber();

    size := sys_file_size(file.handle.fd);
    if size < 0 return 0, error_from_result(size);

    return size, .NONE;
}

// Closes the file. With `now`, without going through the engine (see `close` for a reader).
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     file, _ := Io.open_file("/tmp/jacket-io-close.txt", .WRITE);
//     assert(Io.close(*file) == .NONE);
// });
// file_delete("/tmp/jacket-io-close.txt");
// ```
close :: (file: *Io_File, now := false) -> Io_Error {
    #if OS != .WINDOWS {
        if file.restore_status_flags >= 0 {
            c_fcntl(cast(s32) file.handle.fd, F_SETFL, file.restore_status_flags);
            file.restore_status_flags = -1;
        }
    }

    if now {
        close_without_waiting(file.handle);

        return .NONE;
    }

    return close_handle(file.handle);
} @FreesArgument

// The whole content of a file, in memory from `context.allocator` (free it with `free`).
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     Io.write_file("/tmp/jacket-io-whole.txt", "contents");
//
//     data, error := Io.read_file("/tmp/jacket-io-whole.txt");
//     assert(error == .NONE && cast(string) data == "contents");
//     free(data.data);
// });
// file_delete("/tmp/jacket-io-whole.txt");
// ```
read_file :: (path: string) -> [] u8, Io_Error {
    file, error := open_file(path, .READ);
    if error != .NONE return .[], error;

    defer close(*file);

    size, size_error := file_size(*file);
    if size_error != .NONE return .[], size_error;

    return read_all(reader_of(*file), size);
}

// Creates or replaces a file with `data`.
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// Io.io_run(() {
//     assert(Io.write_file("/tmp/jacket-io-wf.txt", "hello") == .NONE);
// });
// file_delete("/tmp/jacket-io-wf.txt");
// ```
write_file :: (path: string, data: [] u8) -> Io_Error {
    file, error := open_file(path, .WRITE);
    if error != .NONE return error;

    defer close(*file);

    _, write_error := write(*file, data);

    return write_error;
}

// The `write_file` for a `string`.
write_file :: (path: string, text: string) -> Io_Error {
    return write_file(path, ([] u8).{ text.count, text.data });
}
