// Readers and writers: one way to read and write bytes whatever they come from. A file, a TCP
// connection, an in-memory pipe, or a source of your own, all reached through the same five
// operations:
//
//     file, error := open_file("log.txt", .READ);
//     defer close(*file);
//
//     source := reader_of(*file);
//
//     header: [16] u8;
//     count, error = read_full(source, header);      // exactly 16 bytes, or .END_OF_FILE
//     rest, error  = read_all(source);               // everything that is left
//
// `read` (some bytes), `read_full` (all of a buffer), `write` (all of the bytes), `write_some`
// (at least one byte) and `close` exist for `*Io_File`, `Tcp_Stream` and `*Pipe_Stream` directly,
// and for `Reader` and `Writer`, which make code that does not care where the bytes go (a parser
// that wants a stream, an HTTP connection) work on every source.
//
// A `Reader` or `Writer` is a handle: a pointer to the source's own data and the procedures to
// use on it. Copy it freely; it does not own the source. Whatever has a position or other state
// (`Io_File`) is passed by pointer, and the handle points at it, so keep it alive while the
// reader or writer is in use. A TCP connection is a handle itself, and its reader and writer
// carry the descriptor in `data`. `close` on either one closes the source, once: a TCP stream or
// pipe end has a reader and a writer over the same connection, so close just one of them.
//
// A source of your own is a struct with a pointer and the procedures:
//
//     counter_read :: (data: *void, buffer: [] u8) -> s64, Io_Error { ... }
//
//     source := Io.Reader.{ data = *mine, read_proc = counter_read };
//
// Any `Io_Error` other than `.NONE` comes with 0 bytes (or, for `read_full` and `write`, with the
// number of bytes that did go through).

// Reads what is there, up to `buffer.count` bytes, waiting for at least one: 0 means the end.
Read_Proc :: #type (data: *void, buffer: [] u8) -> s64, Io_Error;

// Writes at least one byte of `bytes` (all of it for a sink that never waits) and returns how many.
Write_Proc :: #type (data: *void, bytes: [] u8) -> s64, Io_Error;

// Closes the source. `now` asks for a close that does not wait, for code that cannot block.
Close_Proc :: #type (data: *void, now: bool) -> Io_Error;

Reader :: struct {
    data: *void;
    read_proc: Read_Proc;
    // Null for a source that has nothing to close.
    close_proc: Close_Proc;
}

Writer :: struct {
    data: *void;
    write_proc: Write_Proc;
    close_proc: Close_Proc;
}

// ---------------------------------------------------------------------------------------------
// Reading

// Waits for bytes and reads what is there, up to `buffer.count`. 0 means the end of the source.
read :: (reader: Reader, buffer: [] u8) -> s64, Io_Error {
    if buffer.count == 0 return 0, .NONE;
    if !reader.read_proc return 0, .UNSUPPORTED;

    return reader.read_proc(reader.data, buffer);
}

// Reads until `buffer` is full. `.END_OF_FILE` if the source ended first, with the bytes read.
read_full :: (reader: Reader, buffer: [] u8) -> s64, Io_Error {
    total := 0;

    while total < buffer.count {
        count, error := read(reader, .{ buffer.count - total, buffer.data + total });
        if error != .NONE return total, error;
        if count == 0 return total, .END_OF_FILE;

        total += count;
    }

    return total, .NONE;
}

// Reads to the end of the source into memory from `context.allocator` (free it with `free`).
// `size_hint` is the expected length, when it is known. On any error the bytes read so far are
// freed.
read_all :: (reader: Reader, size_hint := 0) -> [] u8, Io_Error {
    capacity := max(size_hint + 1, 4096);

    data := cast(*u8) alloc(capacity);
    used := 0;

    while true {
        if used == capacity {
            capacity *= 2;
            data = cast(*u8) realloc(data, capacity, used);
        }

        count, error := read(reader, .{ capacity - used, data + used });

        if error != .NONE {
            free(data);

            return .[], error;
        }

        if count == 0 break;

        used += count;
    }

    return .{ used, data }, .NONE;
}

// ---------------------------------------------------------------------------------------------
// Writing

// Sends at least one byte of `data` (unless it is empty) and returns how many.
write_some :: (writer: Writer, data: [] u8) -> s64, Io_Error {
    if data.count == 0 return 0, .NONE;
    if !writer.write_proc return 0, .UNSUPPORTED;

    return writer.write_proc(writer.data, data);
}

// Sends all of `data`, waiting as often as it takes. On an error it returns how many bytes went out.
write :: (writer: Writer, data: [] u8) -> s64, Io_Error {
    total := 0;

    while total < data.count {
        count, error := write_some(writer, .{ data.count - total, data.data + total });
        if error != .NONE return total, error;
        if count == 0 return total, .NO_SPACE;

        total += count;
    }

    return total, .NONE;
}

write :: (writer: Writer, text: string) -> s64, Io_Error {
    return write(writer, .{ text.count, text.data });
}

// ---------------------------------------------------------------------------------------------
// Closing

// Closes the source behind a reader. With `now`, nothing waits: use it where the code cannot
// block (a `Local_Drop`); fibers still waiting on the source are not woken, so prefer the plain
// call unless the scheduler is ending.
close :: (reader: Reader, now := false) -> Io_Error {
    if !reader.close_proc return .NONE;

    return reader.close_proc(reader.data, now);
}

close :: (writer: Writer, now := false) -> Io_Error {
    if !writer.close_proc return .NONE;

    return writer.close_proc(writer.data, now);
}

// ---------------------------------------------------------------------------------------------
// The built-in sources

reader_of :: (file: *Io_File) -> Reader {
    return .{ file, file_read_proc, file_close_proc };
}

reader_of :: (stream: Tcp_Stream) -> Reader {
    return .{ cast(*void) stream.handle.fd, tcp_read_proc, tcp_close_proc };
}

reader_of :: (stream: *Pipe_Stream) -> Reader {
    return .{ stream, pipe_read_proc, pipe_close_proc };
}

writer_of :: (file: *Io_File) -> Writer {
    return .{ file, file_write_proc, file_close_proc };
}

writer_of :: (stream: Tcp_Stream) -> Writer {
    return .{ cast(*void) stream.handle.fd, tcp_write_proc, tcp_close_proc };
}

writer_of :: (stream: *Pipe_Stream) -> Writer {
    return .{ stream, pipe_write_proc, pipe_close_proc };
}

#scope_module

file_read_proc :: (data: *void, buffer: [] u8) -> s64, Io_Error {
    return read(cast(*Io_File) data, buffer);
}

file_write_proc :: (data: *void, bytes: [] u8) -> s64, Io_Error {
    return write_some(cast(*Io_File) data, bytes);
}

file_close_proc :: (data: *void, now: bool) -> Io_Error {
    return close(cast(*Io_File) data, now);
}

tcp_of :: (data: *void) -> Tcp_Stream {
    stream: Tcp_Stream;

    stream.handle = .{ cast(s64) data, .SOCKET };

    return stream;
}

tcp_read_proc :: (data: *void, buffer: [] u8) -> s64, Io_Error {
    return read(tcp_of(data), buffer);
}

tcp_write_proc :: (data: *void, bytes: [] u8) -> s64, Io_Error {
    return write_some(tcp_of(data), bytes);
}

tcp_close_proc :: (data: *void, now: bool) -> Io_Error {
    return close(tcp_of(data), now);
}

pipe_read_proc :: (data: *void, buffer: [] u8) -> s64, Io_Error {
    return read(cast(*Pipe_Stream) data, buffer);
}

pipe_write_proc :: (data: *void, bytes: [] u8) -> s64, Io_Error {
    return write_some(cast(*Pipe_Stream) data, bytes);
}

pipe_close_proc :: (data: *void, now: bool) -> Io_Error {
    return close(cast(*Pipe_Stream) data, now);
}
