// Files for fibers. On io_uring the reads and writes run in the kernel and only the calling fiber
// waits. On epoll and kqueue, which cannot report a regular file as "not ready", they run at once
// on the scheduler's thread (blocking-with-readiness): a slow disk stalls every fiber of that
// thread for the length of the call. Opening a file is always a plain call.

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
}

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

// Reads up to `buffer.count` bytes at the file's position. 0 at the end of the file.
read :: (file: *Io_File, buffer: [] u8) -> s64, Io_Error {
    count, error := read_at(file, buffer, file.position);
    file.position += count;

    return count, error;
}

// Reads until `buffer` is full. `.END_OF_FILE` if the file ended first, with the bytes read.
read_full :: (file: *Io_File, buffer: [] u8) -> s64, Io_Error {
    return read_full(reader_of(file), buffer);
}

// Reads at `offset`; the file's position does not move.
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
// position past it. At least one byte unless `data` is empty; returns how many.
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
write :: (file: *Io_File, data: [] u8) -> s64, Io_Error {
    return write(writer_of(file), data);
}

write :: (file: *Io_File, text: string) -> s64, Io_Error {
    return write(file, .{ text.count, text.data });
}

// Writes all of `data` at `offset`; the file's position does not move.
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

write_at :: (file: *Io_File, text: string, offset: s64) -> s64, Io_Error {
    return write_at(file, .{ text.count, text.data }, offset);
}

file_size :: (file: *Io_File) -> s64, Io_Error {
    current_fiber();

    size := sys_file_size(file.handle.fd);
    if size < 0 return 0, error_from_result(size);

    return size, .NONE;
}

// Closes the file. With `now`, without going through the engine (see `close` for a reader).
close :: (file: *Io_File, now := false) -> Io_Error {
    if now {
        close_without_waiting(file.handle);

        return .NONE;
    }

    return close_handle(file.handle);
} @Releases

// The whole content of a file, in memory from `context.allocator` (free it with `free`).
read_file :: (path: string) -> [] u8, Io_Error {
    file, error := open_file(path, .READ);
    if error != .NONE return .[], error;

    defer close(*file);

    size, size_error := file_size(*file);
    if size_error != .NONE return .[], size_error;

    return read_all(reader_of(*file), size);
}

// Creates or replaces a file with `data`.
write_file :: (path: string, data: [] u8) -> Io_Error {
    file, error := open_file(path, .WRITE);
    if error != .NONE return error;

    defer close(*file);

    _, write_error := write(*file, data);

    return write_error;
}

write_file :: (path: string, text: string) -> Io_Error {
    return write_file(path, .{ text.count, text.data });
}
