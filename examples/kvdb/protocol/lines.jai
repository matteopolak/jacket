// Lines on a TCP stream. Reading is `Io.Line_Reader`; the server reads requests that way and the
// client reads replies that way.

// The longest line accepted: a SET with the largest key and value, plus its verb.
MAX_LINE :: MAX_KEY_SIZE + MAX_VALUE_SIZE + 16;

// A line reader over `stream` that accepts lines of up to `MAX_LINE` bytes.
line_reader :: (stream: Io.Tcp_Stream) -> Io.Line_Reader {
    return Io.make_line_reader(Io.reader_of(stream), MAX_LINE);
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
