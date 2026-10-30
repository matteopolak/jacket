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

// Waits until `read` would not have to wait: a byte is there, the source ended, or it failed (the
// `read` that follows says which). Nothing is consumed, so it needs no buffer.
Wait_Proc :: #type (data: *void) -> Io_Error;

// Closes the source. `now` asks for a close that does not wait, for code that cannot block.
Close_Proc :: #type (data: *void, now: bool) -> Io_Error;

// A source of bytes: a pointer to its data and the procedures to use on it. `reader_of` makes one for a file, a TCP stream or
// a pipe end, and a struct of your own can fill in `read_proc` (and optionally `close_proc` and `wait_proc`). A `Reader` is a
// handle: copy it freely, it owns nothing, and the source must stay alive while it is used. `close` on it closes the source.
//
// ```jai
// Counter :: struct {
//     next: u8;
// }
//
// count_up :: (data: *void, buffer: [] u8) -> s64, Io_Error {
//     counter := cast(*Counter) data;
//
//     for 0..buffer.count - 1 {
//         buffer[it] = counter.next;
//         counter.next += 1;
//     }
//
//     return buffer.count, .NONE;
// }
//
// main :: () {
//     counter: Counter;
//     source := Reader.{ data = *counter, read_proc = count_up };
//
//     bytes: [4] u8;
//     count, error := read(source, bytes);
//
//     assert(error == .NONE && count == 4);
//     assert(bytes[0] == 0 && bytes[3] == 3);
// }
// ```
Reader :: struct {
    data: *void;
    read_proc: Read_Proc;
    // Null for a source that has nothing to close.
    close_proc: Close_Proc;
    // Null for a source that cannot say when a read would not wait (`wait_readable` then returns
    // `.UNSUPPORTED`). A TCP stream can.
    wait_proc: Wait_Proc;
}

// A sink for bytes, the counterpart of `Reader`: a pointer to its data, a `write_proc` and an optional `close_proc`. `writer_of`
// makes one for a file, a TCP stream or a pipe end. It is a handle that owns nothing; `close` on it closes the sink.
Writer :: struct {
    data: *void;
    write_proc: Write_Proc;
    close_proc: Close_Proc;
}

// Reading

// Waits for bytes and reads what is there, up to `buffer.count`. 0 means the end of the source.
read :: (reader: Reader, buffer: [] u8) -> s64, Io_Error {
    if buffer.count == 0 return 0, .NONE;
    if !reader.read_proc return 0, .UNSUPPORTED;

    return reader.read_proc(reader.data, buffer);
}

// Waits until a `read` would return at once: bytes are there, the source ended, or it failed (the
// `read` then reports which). It takes no buffer and consumes nothing, so a connection can sit idle
// without holding memory and allocate only when something arrives. Deadlines and `cancel` end it
// like any other wait. `.UNSUPPORTED` for a reader that cannot tell (a file, a pipe, a source of
// your own without `wait_proc`): `can_wait_readable` says beforehand.
wait_readable :: (reader: Reader) -> Io_Error {
    if !reader.wait_proc return .UNSUPPORTED;

    return reader.wait_proc(reader.data);
}

// Whether `wait_readable` works on `reader`, which is when its source supplies a `wait_proc` (a TCP stream does; a file or a pipe
// end does not). When it is false, `wait_readable` returns `.UNSUPPORTED`.
//
// ```jai
// Io :: #import "Extensions/Io";
//
// main :: () {
//     Io.io_run(() {
//         near, far := Io.make_pipe();
//         defer Io.close(near);
//         defer Io.close(far);
//
//         assert(!Io.can_wait_readable(Io.reader_of(near)));
//     });
// }
// ```
can_wait_readable :: (reader: Reader) -> bool {
    return reader.wait_proc != null;
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

// The `write` for a `string`: sends all of `text` and returns how many bytes went out.
write :: (writer: Writer, text: string) -> s64, Io_Error {
    return write(writer, ([] u8).{ text.count, text.data });
}

// Closing

// Closes the source behind a reader. With `now`, nothing waits: use it where the code cannot
// block (a `Local_Drop`); fibers still waiting on the source are not woken, so prefer the plain
// call unless the scheduler is ending.
close :: (reader: Reader, now := false) -> Io_Error {
    if !reader.close_proc return .NONE;

    return reader.close_proc(reader.data, now);
} @FreesArgument

// Closes the sink behind a writer. A TCP stream or pipe end has a reader and a writer over one connection, so close only one
// of them.
close :: (writer: Writer, now := false) -> Io_Error {
    if !writer.close_proc return .NONE;

    return writer.close_proc(writer.data, now);
} @FreesArgument

// The built-in sources

// The file as a `Reader`. Reading advances the file's position. The reader points at the `Io_File`, which must outlive it.
reader_of :: (file: *Io_File) -> Reader {
    return .{ file, file_read_proc, file_close_proc, null };
}

// The TCP stream as a `Reader`; it can `wait_readable`. The reader carries the socket handle, so closing the reader closes the
// stream.
reader_of :: (stream: Tcp_Stream) -> Reader {
    return .{ cast(*void) stream.handle.fd, tcp_read_proc, tcp_close_proc, tcp_wait_proc };
}

// One end of a pipe from `make_pipe` as a `Reader`. The reader points at the pipe end, which must outlive it.
reader_of :: (stream: *Pipe_Stream) -> Reader {
    return .{ stream, pipe_read_proc, pipe_close_proc, null };
}

// The file as a `Writer`. Writing advances the file's position. The writer points at the `Io_File`, which must outlive it.
writer_of :: (file: *Io_File) -> Writer {
    return .{ file, file_write_proc, file_close_proc };
}

// The TCP stream as a `Writer`. `write` through it waits as long as it takes for the peer to take the bytes. Closing the writer
// closes the stream.
//
// ```jai
// Io :: #import "Extensions/Io";
//
// main :: () {
//     Io.io_run(() {
//         listener, _ := Io.tcp_listen("127.0.0.1:0");
//         defer Io.close(listener);
//
//         address, _ := Io.local_address(listener);
//         client, _ := Io.connect(address);
//         server, _ := Io.accept(listener);
//
//         Io.write(Io.writer_of(client), "hello");
//
//         buffer: [8] u8;
//         count, error := Io.read(Io.reader_of(server), buffer);
//         assert(error == .NONE && string.{ count, buffer.data } == "hello");
//
//         Io.close(client);
//         Io.close(server);
//     });
// }
// ```
writer_of :: (stream: Tcp_Stream) -> Writer {
    return .{ cast(*void) stream.handle.fd, tcp_write_proc, tcp_close_proc };
}

// One end of a pipe from `make_pipe` as a `Writer`. The writer points at the pipe end, which must outlive it.
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

tcp_wait_proc :: (data: *void) -> Io_Error {
    return wait_readable(tcp_of(data));
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
