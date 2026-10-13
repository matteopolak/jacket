// Extensions/Io readers and writers: the same five operations (`read`, `read_full`, `write`,
// `write_some`, `close`) over a file, a TCP connection and a pipe, directly and through
// `Io.Reader` / `Io.Writer`; a source made of a pointer and procedures; `read_all`; and the deadline
// primitive (`with_deadline`) on a reader.
#load "lib/suite.jai";
#import "Extensions/Json";

// ---------------------------------------------------------------------------------------------
// A generic copy: it knows nothing about where the bytes come from or go

copy_all :: (source: Io.Reader, sink: Io.Writer) -> s64, Io.Io_Error {
    buffer: [7] u8;
    total := 0;

    while true {
        count, error := Io.read(source, buffer);
        if error != .NONE return total, error;
        if count == 0 break;

        _, write_error := Io.write(sink, .{ count, buffer.data });
        if write_error != .NONE return total, write_error;

        total += count;
    }

    return total, .NONE;
}

bytes_of :: (text: string) -> [] u8 {
    return .{ text.count, text.data };
}

PAYLOAD :: "0123456789abcdefghijklmnopqrstuvwxyz";

// ---------------------------------------------------------------------------------------------
// Files

test_file_streams :: () {
    make_scratch();

    path := tprint("%/streams.txt", SCRATCH);

    file, error := Io.open_file(path, .WRITE);

    assert(error == .NONE, "open: %", error);

    sink := Io.writer_of(*file);

    // write_some takes at least one byte and moves the position by what it took.
    count, write_error := Io.write_some(sink, bytes_of("abcdef"));

    assert(write_error == .NONE && count >= 1 && count <= 6, "write_some: % %", count, write_error);
    assert(file.position == count, "write_some moved the position to % for % bytes", file.position, count);

    rest := string.{ 6 - count, "abcdef".data + count };

    count, write_error = Io.write(sink, rest);
    assert(write_error == .NONE && count == rest.count, "write: % %", count, write_error);

    // The direct forms are the same operations.
    count, write_error = Io.write_some(*file, bytes_of("ghij"));
    assert(write_error == .NONE && count >= 1);

    count, write_error = Io.write(*file, string.{ 4 - count, "ghij".data + count });
    assert(write_error == .NONE);

    count, write_error = Io.write(*file, "KL");
    assert(write_error == .NONE && count == 2);
    assert(file.position == 12, "position %", file.position);

    assert(Io.close(sink) == .NONE);

    // Read it back through a reader.
    file, error = Io.open_file(path, .READ);
    assert(error == .NONE);

    source := Io.reader_of(*file);
    buffer: [32] u8;

    count, error = Io.read_full(source, .{ 5, buffer.data });
    assert(error == .NONE && count == 5 && string.{ 5, buffer.data } == "abcde", "read_full: % %", count, error);

    count, error = Io.read_full(*file, .{ 3, buffer.data });
    assert(error == .NONE && count == 3 && string.{ 3, buffer.data } == "fgh", "direct read_full: % %", count, error);

    // read_full stops at the end of the file and says so, with what it got.
    count, error = Io.read_full(*file, buffer);
    assert(error == .END_OF_FILE && count == 4 && string.{ 4, buffer.data } == "ijKL", "read_full at the end: % %", count, error);

    count, error = Io.read(*file, buffer);
    assert(error == .NONE && count == 0, "read at the end: % %", count, error);

    // read_all from the start.
    file.position = 0;

    everything, error2 := Io.read_all(source);

    assert(error2 == .NONE && string.{ everything.count, everything.data } == "abcdefghijKL", "read_all: %", error2);

    free(everything.data);

    // Appending ignores the position.
    assert(Io.close(*file) == .NONE);

    file, error = Io.open_file(path, .APPEND);
    assert(error == .NONE);

    count, write_error = Io.write_some(Io.writer_of(*file), bytes_of("!"));
    assert(write_error == .NONE && count == 1);

    assert(Io.close(*file, now = true) == .NONE);

    whole, read_error := Io.read_file(path);

    assert(read_error == .NONE && string.{ whole.count, whole.data } == "abcdefghijKL!", "read_file: %", read_error);

    free(whole.data);

    // A big file goes through a copy of any size.
    big: [..] u8;

    for 0..29_999 array_add(*big, cast(u8) ((it * 13 + it / 7) & 0xff));

    assert(Io.write_file(tprint("%/streams-big.bin", SCRATCH), big) == .NONE);

    from, error3 := Io.open_file(tprint("%/streams-big.bin", SCRATCH), .READ);

    assert(error3 == .NONE);

    to, error4 := Io.open_file(tprint("%/streams-copy.bin", SCRATCH), .WRITE);

    assert(error4 == .NONE);

    copied, copy_error := copy_all(Io.reader_of(*from), Io.writer_of(*to));

    assert(copy_error == .NONE && copied == big.count, "copied % of %: %", copied, big.count, copy_error);

    Io.close(*from);
    Io.close(*to);

    again, again_error := Io.read_file(tprint("%/streams-copy.bin", SCRATCH));

    assert(again_error == .NONE && again.count == big.count && memcmp(again.data, big.data, big.count) == 0, "the copy differs");

    free(again.data);
    array_free(big);
}

// ---------------------------------------------------------------------------------------------
// TCP

test_tcp_streams :: () {
    pair := make_pair();

    sink := Io.writer_of(pair.client);
    source := Io.reader_of(pair.server);

    count, error := Io.write_some(sink, bytes_of("hello "));

    assert(error == .NONE && count >= 1 && count <= 6, "write_some: % %", count, error);

    rest := string.{ 6 - count, "hello ".data + count };

    _, error = Io.write(sink, rest);
    assert(error == .NONE);

    _, error = Io.write(pair.client, "world");
    assert(error == .NONE);

    buffer: [32] u8;

    count, error = Io.read_full(source, .{ 11, buffer.data });
    assert(error == .NONE && count == 11 && string.{ 11, buffer.data } == "hello world", "read_full: % %", count, error);

    // Direct and wrapped reads are the same operation.
    Io.write(pair.client, "abc");
    count, error = Io.read(pair.server, buffer);
    assert(error == .NONE && count == 3, "read: % %", count, error);

    // A reader and a writer on the same end see each other's data.
    Io.write(Io.writer_of(pair.server), "back");
    count, error = Io.read(Io.reader_of(pair.client), buffer);
    assert(error == .NONE && count == 4 && string.{ 4, buffer.data } == "back");

    // The peer closing is the end of the stream: read says 0, read_full says .END_OF_FILE.
    Io.write(pair.client, "xy");
    assert(Io.close(sink) == .NONE);

    count, error = Io.read_full(source, buffer);
    assert(error == .END_OF_FILE && count == 2 && string.{ 2, buffer.data } == "xy", "read_full after close: % %", count, error);

    count, error = Io.read(source, buffer);
    assert(error == .NONE && count == 0, "read after close: % %", count, error);

    // read_all collects everything up to the close.
    other := make_pair();

    Io.spawn((stream: Io.Tcp_Stream) {
        for 1..20 Io.write(stream, "chunk of text. ");
        Io.close(stream);
    }, other.client);

    all, all_error := Io.read_all(Io.reader_of(other.server));

    assert(all_error == .NONE && all.count == 20 * 15, "read_all: % bytes, %", all.count, all_error);

    free(all.data);

    Io.close(Io.reader_of(pair.server), now = true);
    Io.close(pair.listener);
    Io.close(other.server);
    Io.close(other.listener);
}

// ---------------------------------------------------------------------------------------------
// Pipes

test_pipe_streams :: () {
    near, far := Io.make_pipe();

    sink := Io.writer_of(near);
    source := Io.reader_of(far);

    // A pipe never waits, so write_some takes everything.
    count, error := Io.write_some(sink, bytes_of("pipe data"));

    assert(error == .NONE && count == 9, "write_some: % %", count, error);

    count, error = Io.write(sink, "!");
    assert(error == .NONE && count == 1);

    buffer: [16] u8;

    count, error = Io.read_full(source, .{ 10, buffer.data });
    assert(error == .NONE && count == 10 && string.{ 10, buffer.data } == "pipe data!", "read_full: % %", count, error);

    Io.write(near, "ab");
    count, error = Io.read(source, buffer);
    assert(error == .NONE && count == 2);

    Io.write_some(near, bytes_of("cd"));

    // Closing the writing end is the end of the stream.
    Io.write(near, "ef");
    Io.close(sink);

    all, all_error := Io.read_all(source);

    assert(all_error == .NONE && string.{ all.count, all.data } == "cdef", "read_all: %", all_error);

    free(all.data);

    count, error = Io.read_full(source, buffer);
    assert(error == .END_OF_FILE && count == 0, "read_full at the end: % %", count, error);

    Io.close(far);

    // Writing to a closed peer fails.
    a, b := Io.make_pipe();

    Io.close(b);

    _, error = Io.write(Io.writer_of(a), "x");
    assert(error == .BROKEN_PIPE, "write to a closed pipe: %", error);

    Io.close(a);
}

// ---------------------------------------------------------------------------------------------
// A source of your own

Trickle :: struct {
    text: string;
    at: int;
    // How many bytes a read or write may move.
    step: int;
    closed: int;
    sunk: [..] u8;
}

trickle_read :: (data: *void, buffer: [] u8) -> s64, Io.Io_Error {
    source := cast(*Trickle) data;

    count := min(min(source.step, buffer.count), source.text.count - source.at);

    memcpy(buffer.data, source.text.data + source.at, count);

    source.at += count;

    return count, .NONE;
}

trickle_write :: (data: *void, bytes: [] u8) -> s64, Io.Io_Error {
    sink := cast(*Trickle) data;

    count := min(sink.step, bytes.count);

    for 0..count - 1 array_add(*sink.sunk, bytes[it]);

    return count, .NONE;
}

trickle_close :: (data: *void, now: bool) -> Io.Io_Error {
    (cast(*Trickle) data).closed += ifx now then 10 else 1;

    return .NONE;
}

test_custom_source :: () {
    trickle: Trickle;

    trickle.text = PAYLOAD;
    trickle.step = 5;

    source := Io.Reader.{ *trickle, trickle_read, trickle_close };
    buffer: [64] u8;

    // read gives some bytes, read_full all of them, whatever the source's step.
    count, error := Io.read(source, buffer);

    assert(error == .NONE && count == 5, "read: % %", count, error);

    count, error = Io.read_full(source, .{ 12, buffer.data });
    assert(error == .NONE && count == 12, "read_full: % %", count, error);

    rest, rest_error := Io.read_all(source);

    assert(rest_error == .NONE && rest.count == PAYLOAD.count - 17, "read_all: % %", rest.count, rest_error);

    free(rest.data);

    count, error = Io.read_full(source, buffer);
    assert(error == .END_OF_FILE && count == 0, "read_full at the end: % %", count, error);

    // The same value works as the input of the generic copy.
    trickle.at = 0;

    sink: Trickle;

    sink.step = 3;

    copied, copy_error := copy_all(source, Io.Writer.{ *sink, trickle_write, trickle_close });

    assert(copy_error == .NONE && copied == PAYLOAD.count, "copy: % %", copied, copy_error);
    assert(string.{ sink.sunk.count, sink.sunk.data } == PAYLOAD, "the copy differs");

    // write loops over write_some.
    sink.sunk.count = 0;

    sunk, write_error := Io.write(Io.Writer.{ *sink, trickle_write, null }, PAYLOAD);

    assert(write_error == .NONE && sunk == PAYLOAD.count && sink.sunk.count == PAYLOAD.count);

    assert(Io.close(source) == .NONE);
    assert(Io.close(source, now = true) == .NONE);
    assert(trickle.closed == 11, "close_proc was called with now = %", trickle.closed);

    // A writer or reader without the procedure says so.
    nothing: Io.Reader;

    count, error = Io.read(nothing, buffer);
    assert(error == .UNSUPPORTED, "an empty reader: %", error);
    assert(Io.close(nothing) == .NONE);

    array_free(sink.sunk);
}

// ---------------------------------------------------------------------------------------------
// Deadlines on a reader

test_deadline_on_readers :: () {
    near, far := Io.make_pipe();

    buffer: [8] u8;
    count: s64;
    error: Io.Io_Error;

    started := now_ms();

    Io.with_deadline(30, #code {
        count, error = Io.read(Io.reader_of(far), buffer);
    });
    assert(error == .TIMED_OUT && count == 0 && now_ms() - started >= 25, "pipe reader: %", error);

    // NO_DEADLINE adds no limit; an enclosing one still applies.
    Io.spawn((end: *Io.Pipe_Stream) {
        Io.sleep(40);
        Io.write(end, "late");
    }, near);

    Io.with_deadline(Io.NO_DEADLINE, #code {
        count, error = Io.read(Io.reader_of(far), buffer);
    });
    assert(error == .NONE && count == 4, "no deadline: % %", count, error);

    Io.with_deadline(30, #code {
        Io.with_deadline(Io.NO_DEADLINE, #code {
            count, error = Io.read(Io.reader_of(far), buffer);
        });
    });
    assert(error == .TIMED_OUT, "the outer deadline survives NO_DEADLINE: %", error);

    // Zero has already passed, and the deadline is gone after the block.
    Io.write(near, "ready");

    Io.with_deadline(0, #code {
        count, error = Io.read(Io.reader_of(far), buffer);
    });
    assert(error == .TIMED_OUT, "an expired deadline fails the call: %", error);

    count, error = Io.read(Io.reader_of(far), buffer);
    assert(error == .NONE && count == 5, "no deadline after the block: % %", count, error);

    // The same on a TCP reader.
    pair := make_pair();

    Io.with_deadline(30, #code {
        count, error = Io.read(Io.reader_of(pair.client), buffer);
    });
    assert(error == .TIMED_OUT, "tcp reader: %", error);

    Io.close(near);
    Io.close(far);
    close_pair(pair);
}

// ---------------------------------------------------------------------------------------------
// JSON from a stream

Person :: struct {
    name: string;
    age: int;
    tags: [..] string;
}

JSON_TEXT :: "{\"name\": \"Zo\\u00eb \\\"Z\\\" \xc3\xa9\", \"age\": 31, \"tags\": [\"a\", \"bb\", \"ccc\"]}";

// Writes `text` in pieces of `step` bytes, giving the reader a turn after each, then closes.
Dribble :: struct {
    end: *Io.Pipe_Stream;
    text: string;
    step: int;
}

dribble :: (job: Dribble) {
    at := 0;

    while at < job.text.count {
        count := min(job.step, job.text.count - at);

        Io.write(job.end, string.{ count, job.text.data + at });
        Io.yield();

        at += count;
    }

    Io.close(job.end);
}

check_person :: (person: Person, what: string) {
    assert(person.name == "Zo\xc3\xab \"Z\" \xc3\xa9" && person.age == 31 && person.tags.count == 3 && person.tags[2] == "ccc", "%: % % %", what, person.name, person.age, person.tags.count);
}

test_read_json :: () {
    // A body split into one-byte, three-byte and big writes decodes to the same value, with a
    // multi-byte character and an escape cut anywhere.
    for step: int.[1, 3, 7, 1000] {
        near, far := Io.make_pipe();

        Io.spawn(dribble, Dribble.{ near, JSON_TEXT, step });

        person, error := Io.read_json(Io.reader_of(far), Person);

        assert(error.kind == .NONE, "step %: %", step, error_text(error));

        check_person(person, tprint("step %", step));

        free_json(*person);
        Io.close(far);
    }

    // The same bytes through from_json give the same value.
    direct, direct_error := from_json(Person, JSON_TEXT);

    assert(direct_error.kind == .NONE);
    check_person(direct, "from_json");
    free_json(*direct);

    // A file.
    make_scratch();

    path := tprint("%/streams.json", SCRATCH);

    assert(Io.write_file(path, JSON_TEXT) == .NONE);

    file, open_error := Io.open_file(path, .READ);

    assert(open_error == .NONE);

    from_file, file_error := Io.read_json(Io.reader_of(*file), Person);

    assert(file_error.kind == .NONE, "file: %", error_text(file_error));
    check_person(from_file, "file");

    free_json(*from_file);
    Io.close(*file);

    // A socket.
    pair := make_pair();

    Io.spawn((stream: Io.Tcp_Stream) {
        Io.write(stream, JSON_TEXT);
        Io.close(stream);
    }, pair.client);

    from_socket, socket_error := Io.read_json(Io.reader_of(pair.server), Person);

    assert(socket_error.kind == .NONE, "socket: %", error_text(socket_error));
    check_person(from_socket, "socket");

    free_json(*from_socket);
    Io.close(pair.server);
    Io.close(pair.listener);

    // Bad JSON is the same error as from_json gives, with the same position.
    bad := "{\"name\": \"x\", \"age\": true}";

    near, far := Io.make_pipe();

    Io.spawn(dribble, Dribble.{ near, bad, 2 });

    _, streamed_error := Io.read_json(Io.reader_of(far), Person);
    _, text_error := from_json(Person, bad);

    assert(streamed_error.kind == .TYPE && streamed_error.kind == text_error.kind && streamed_error.offset == text_error.offset && streamed_error.path == text_error.path, "a type error: %", error_text(streamed_error));

    free_json(*streamed_error);
    free_json(*text_error);
    Io.close(far);

    // A reader that fails: a read that times out is a .READ error that says why, and nothing leaks.
    near, far = Io.make_pipe();

    Io.write(near, "{\"name\": ");

    failed: Person;
    read_error: Json_Error;

    Io.with_deadline(20, #code {
        failed, read_error = Io.read_json(Io.reader_of(far), Person);
    });
    assert(read_error.kind == .READ && read_error.message == "reading failed: TIMED_OUT", "a failing reader: % %", read_error.kind, read_error.message);

    free_json(*read_error);
    Io.close(near);
    Io.close(far);

    // The document size limit applies to the stream.
    near, far = Io.make_pipe();

    Io.spawn(dribble, Dribble.{ near, JSON_TEXT, 5 });

    _, limit_error := Io.read_json(Io.reader_of(far), Person, Json_Options.{ max_size = 20 });

    assert(limit_error.kind == .LIMIT, "the size limit: %", error_text(limit_error));

    free_json(*limit_error);
    Io.close(far);

    // An empty stream is a syntax error, not a crash.
    near, far = Io.make_pipe();
    Io.close(near);

    _, empty_error := Io.read_json(Io.reader_of(far), Person);

    assert(empty_error.kind == .SYNTAX, "an empty stream: %", error_text(empty_error));

    free_json(*empty_error);
    Io.close(far);
}

test_all :: () {
    last_backend = Io.backend_name();

    test_file_streams();
    test_tcp_streams();
    test_pipe_streams();
    test_custom_source();
    test_deadline_on_readers();
    test_read_json();
}

main :: () {
    Io.io_run(test_all);

    print("ok (backend %)\n", last_backend);
}
