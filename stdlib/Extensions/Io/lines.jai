// Reading a source a line at a time. A `Line_Reader` wraps any `Reader` and keeps what it has read
// but not yet handed out, so lines that arrive together cost one `read`, and a line that arrives in
// pieces is put back together.

// The default for `make_line_reader`'s `limit`: the longest line it accepts, in bytes.
DEFAULT_LINE_LIMIT :: 64 * 1024;

// Reads lines from `source`. Make one with `make_line_reader`, read with `read_line` and give its
// buffer back with `free_line_reader`. It does not close `source`.
Line_Reader :: struct {
    source: Reader;
    limit: s64;
    buffer: [..] u8;
    // The line being read starts here; `scanned` bytes after it are known to hold no newline.
    start: s64;
    scanned: s64;
    ended: bool;
    // Set by `read_line`: whether the line it just returned had no line ending.
    unterminated: bool;
}

// A line reader over `source` that accepts lines of up to `limit` bytes (not counting the line
// ending). The limit stops a peer that never sends a newline from filling memory.
//
// ```jai
// Io :: #import "Extensions/Io";
//
// Io.io_run(() {
//     near, far := Io.make_pipe();
//
//     defer Io.close(near);
//     defer Io.close(far);
//     Io.write(Io.writer_of(far), "one\r\ntwo\n");
//     Io.shutdown_write(far);
//
//     lines := Io.make_line_reader(Io.reader_of(near), limit = 1024);
//
//     defer Io.free_line_reader(*lines);
//
//     line, error := Io.read_line(*lines);
//
//     assert(error == .NONE && line == "one");
// });
// ```
make_line_reader :: (source: Reader, limit := DEFAULT_LINE_LIMIT) -> Line_Reader {
    reader: Line_Reader;

    reader.source = source;
    reader.limit = limit;

    return reader;
}

// Frees the reader's buffer. Lines it returned are invalid afterwards.
//
// ```jai
// Io :: #import "Extensions/Io";
//
// Io.io_run(() {
//     near, far := Io.make_pipe();
//
//     defer Io.close(near);
//     defer Io.close(far);
//
//     lines := Io.make_line_reader(Io.reader_of(near));
//
//     Io.free_line_reader(*lines);
// });
// ```
free_line_reader :: (reader: *Line_Reader) {
    array_reset(*reader.buffer);
    reader.* = .{};
}

// The next line without its line ending (`\n` or `\r\n`), or the error that ended the reading:
// `.END_OF_FILE` once the source has ended and every line was handed out, `.INVALID` for a line
// longer than the limit, or whatever the source's `read` reported. After an error, stop reading
// and free the reader.
//
// When the source ends in the middle of a line, that last piece is returned once, with
// `reader.unterminated` set (it is false after every line that had an ending). A protocol that
// acts on whole lines should drop it: the peer may have been cut off.
//
// The line points into the reader's buffer and is valid until the next call. Lines already
// buffered are returned without reading, so a request pipelined behind another costs no wait.
//
// ```jai
// Io :: #import "Extensions/Io";
//
// Io.io_run(() {
//     near, far := Io.make_pipe();
//
//     defer Io.close(near);
//     defer Io.close(far);
//     Io.write(Io.writer_of(far), "GET a\nGET b\nlast");
//     Io.shutdown_write(far);
//
//     lines := Io.make_line_reader(Io.reader_of(near));
//
//     defer Io.free_line_reader(*lines);
//
//     whole := 0;
//
//     while true {
//         line, error := Io.read_line(*lines);
//         if error != .NONE break;
//         if lines.unterminated break; // "last" was cut off: do not act on it
//         whole += 1;
//     }
//
//     assert(whole == 2);
// });
// ```
read_line :: (reader: *Line_Reader) -> string, Io_Error {
    reader.unterminated = false;

    while true {
        pending := reader.buffer.count - reader.start;

        while reader.scanned < pending {
            if reader.buffer[reader.start + reader.scanned] == #char "\n" {
                line := string.{ reader.scanned, reader.buffer.data + reader.start };

                reader.start += reader.scanned + 1;
                reader.scanned = 0;

                if line.count > 0 && line[line.count - 1] == #char "\r" line.count -= 1;
                if line.count > reader.limit return "", .INVALID;

                return line, .NONE;
            }

            reader.scanned += 1;

            // One byte over the limit may be the `\r` of a line that is exactly at it.
            if reader.scanned > reader.limit + 1 return "", .INVALID;
        }

        if reader.ended {
            if pending == 0 return "", .END_OF_FILE;

            line := string.{ pending, reader.buffer.data + reader.start };

            reader.start = reader.buffer.count;
            reader.scanned = 0;

            // No line ending, so a trailing `\r` is part of the text.
            if line.count > reader.limit return "", .INVALID;

            reader.unterminated = true;

            return line, .NONE;
        }

        // Move what is left of the last read to the front, then read into the spare room. The
        // buffer grows by doubling, so a long line is not copied again for every read.
        if reader.start > 0 {
            for 0..pending - 1 reader.buffer[it] = reader.buffer[reader.start + it];

            reader.buffer.count = pending;
            reader.start = 0;
        }

        if reader.buffer.allocated < pending + 4096 array_reserve(*reader.buffer, 2 * pending + 4096);

        room := ([] u8).{ reader.buffer.allocated - pending, reader.buffer.data + pending };
        count, error := read(reader.source, room);

        if error != .NONE return "", error;
        if count == 0 reader.ended = true;

        reader.buffer.count += count;
    }

    return "", .INVALID;
}
