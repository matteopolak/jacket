// test: skip_on(.WASM, reason = "needs Extensions/Io fibers, which wasm cannot run (no stack switching)")
// Io.Line_Reader: lines split across reads, several lines in one read, `\r\n`, a last line with no
// ending, empty lines, the length limit and errors from the source.
#import "Basic";
Io :: #import "Extensions/Io";

// A source that hands out `text` in pieces of at most `piece` bytes, then fails with `failure`
// (or ends, when that is `.NONE`).
Chunks :: struct {
    text: string;
    piece: s64;
    position: s64;
    failure: Io.Io_Error;
}

next_chunk :: (data: *void, buffer: [] u8) -> s64, Io.Io_Error {
    chunks := cast(*Chunks) data;
    left := chunks.text.count - chunks.position;

    if left == 0 return 0, chunks.failure;

    count := min(min(left, chunks.piece), buffer.count);

    memcpy(buffer.data, chunks.text.data + chunks.position, count);
    chunks.position += count;

    return count, .NONE;
}

// Every line of `text`, read in pieces of `piece` bytes, joined with `|`, and the error that ended it.
collect :: (text: string, piece: s64, limit: s64, failure := Io.Io_Error.NONE) -> string, Io.Io_Error {
    chunks := Chunks.{ text = text, piece = piece, failure = failure };
    lines := Io.make_line_reader(Io.Reader.{ data = *chunks, read_proc = next_chunk }, limit);

    defer Io.free_line_reader(*lines);

    joined: String_Builder;
    first := true;

    while true {
        line, error := Io.read_line(*lines);

        if error != .NONE {
            return builder_to_string(*joined), error;
        }

        if !first append(*joined, "|");

        first = false;
        append(*joined, line);

        // The unterminated tail is marked with `~`.
        if lines.unterminated append(*joined, "~");
    }

    return "", .NONE;
}

// The same text at every piece size must give the same lines: split anywhere, even inside "\r\n".
check :: (text: string, want: string, want_error := Io.Io_Error.END_OF_FILE, limit := 1024) {
    for piece: 1..text.count + 1 {
        got, error := collect(text, piece, limit);

        assert(got == want && error == want_error, "% in pieces of %: got [%] %, want [%] %", text, piece, got, error, want, want_error);
    }
}

main :: () {
    check("", "");
    check("one\n", "one");
    check("one\ntwo\n", "one|two");
    check("one\r\ntwo\r\n", "one|two");
    check("one\ntwo", "one|two~");
    check("a\n\nb\n", "a||b");
    check("\n", "");
    check("\r\n", "");
    check("only", "only~");
    check("trailing\r", "trailing\r~");

    // The limit counts the line without its ending; `\r` is not part of it.
    check("12345\n", "12345", limit = 5);
    check("12345\r\n", "12345", limit = 5);
    check("123456\n", "", .INVALID, limit = 5);
    check("ok\n123456\nlater\n", "ok", .INVALID, limit = 5);
    check("1234567890", "", .INVALID, limit = 5);

    // The same limit holds for a tail with no ending, which is one byte over at six bytes.
    check("12345", "12345~", limit = 5);
    check("123456", "", .INVALID, limit = 5);
    check("ok\n12345\r", "ok", .INVALID, limit = 5);

    // A source error comes back after the lines before it.
    got, error := collect("a\nb\nc", 2, 1024, failure = .CONNECTION_RESET);

    assert(got == "a|b" && error == .CONNECTION_RESET, "failure: [%] %", got, error);

    // A long line over a small piece size.
    big := alloc_string(300000);

    defer free(big);
    memset(big.data, #char "x", big.count);
    big[big.count - 1] = #char "\n";

    chunks := Chunks.{ text = big, piece = 4096 };
    lines := Io.make_line_reader(Io.Reader.{ data = *chunks, read_proc = next_chunk }, 400000);

    defer Io.free_line_reader(*lines);

    line, big_error := Io.read_line(*lines);

    assert(big_error == .NONE && line.count == 299999, "big line: % %", line.count, big_error);

    // Over a pipe, on a fiber.
    Io.io_run(() {
        near, far := Io.make_pipe();

        defer Io.close(near);
        defer Io.close(far);

        Io.write(Io.writer_of(far), "GET a\r\nGET b\nrest");
        Io.shutdown_write(far);

        piped := Io.make_line_reader(Io.reader_of(near));

        defer Io.free_line_reader(*piped);

        // Each line is checked before the next call: it is only valid until then.
        first, e1 := Io.read_line(*piped);

        assert(e1 == .NONE && first == "GET a" && !piped.unterminated, "pipe 1: %", first);

        second, e2 := Io.read_line(*piped);

        assert(e2 == .NONE && second == "GET b", "pipe 2: %", second);

        third, e3 := Io.read_line(*piped);

        assert(e3 == .NONE && third == "rest" && piped.unterminated, "pipe 3: %", third);

        _, e4 := Io.read_line(*piped);

        assert(e4 == .END_OF_FILE, "pipe end: %", e4);
    });

    print("ok\n");
}
