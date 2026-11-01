// Reading a TCP stream a line at a time. The server reads requests this way and the client reads
// replies this way.

// The longest line accepted: a SET with the largest key and value, plus its verb.
MAX_LINE :: MAX_KEY_SIZE + MAX_VALUE_SIZE + 16;

Line_Reader :: struct {
    stream: Io.Tcp_Stream;
    buffer: [..] u8;
    // The line being read starts here; `scanned` bytes after it hold no newline yet.
    start: s64;
    scanned: s64;
}

free_line_reader :: (reader: *Line_Reader) {
    array_reset(*reader.buffer);
    reader.* = .{};
}

// The next line without its line ending, or the error that ended the stream: `.END_OF_FILE` when
// the peer closed, `.INVALID` for a line longer than `MAX_LINE`. The line is valid until the next
// call. Between lines the reader does not wait for anything; a request pipelined behind another
// is found in the data that is already buffered.
read_line :: (reader: *Line_Reader) -> string, Io.Io_Error {
    while true {
        pending := reader.buffer.count - reader.start;

        while reader.scanned < pending {
            if reader.buffer[reader.start + reader.scanned] == #char "\n" {
                line := string.{ reader.scanned, reader.buffer.data + reader.start };

                reader.start += reader.scanned + 1;
                reader.scanned = 0;

                if line.count > 0 && line[line.count - 1] == #char "\r" line.count -= 1;

                return line, .NONE;
            }

            reader.scanned += 1;
        }

        if reader.scanned > MAX_LINE return "", .INVALID;

        // Move what is left of the last read to the front, then read into the spare room. The
        // buffer grows by doubling, so a long line is not copied again for every read.
        if reader.start > 0 {
            for 0..pending - 1 reader.buffer[it] = reader.buffer[reader.start + it];

            reader.buffer.count = pending;
            reader.start = 0;
        }

        if reader.buffer.allocated < pending + 4096 array_reserve(*reader.buffer, 2 * pending + 4096);

        room := ([] u8).{ reader.buffer.allocated - pending, reader.buffer.data + pending };
        count, error := Io.read(reader.stream, room);

        if error != .NONE return "", error;
        if count == 0 return "", .END_OF_FILE;

        reader.buffer.count += count;
    }

    return "", .INVALID;
}

// Writes `text` and a line ending.
write_line :: (stream: Io.Tcp_Stream, text: string) -> Io.Io_Error {
    // Not `tprint`: a fiber has little temporary storage and a line can hold a megabyte.
    line := alloc_string(text.count + 1);

    defer free(line);

    memcpy(line.data, text.data, text.count);
    line[text.count] = #char "\n";

    _, error := Io.write(stream, line);

    return error;
}
