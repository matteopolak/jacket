// In-memory pipes (Io.make_pipe): bytes in order between two fibers, end of stream, half close,
// closing under a waiting reader, deadlines, cancellation and a transfer big enough to make the
// buffer compact. Nothing here touches the network.
#import "Basic";
Io :: #import "Extensions/Io";

pattern :: (position: int) -> u8 {
    return cast(u8) ((position * 7 + position / 251) & 0xff);
}

// Echoes everything it reads, upper-cased, until the other end closes.
shout :: (end: *Io.Pipe_Stream) {
    buffer: [64] u8;

    while true {
        count, error := Io.read(end, buffer);

        if error != .NONE || count == 0 break;

        for 0..count - 1 {
            if buffer[it] >= #char "a" && buffer[it] <= #char "z" buffer[it] -= 32;
        }

        Io.write(end, .{ count, buffer.data });
    }

    Io.close(end);
}

test_echo :: () {
    near, far := Io.make_pipe();

    Io.spawn(shout, far);

    buffer: [16] u8;

    Io.write(near, "ping");

    count, error := Io.read(near, buffer);

    assert(error == .NONE && count == 4 && string.{ count, buffer.data } == "PING", "echo: % %", count, error);

    // Two writes before the other side runs again arrive as one stream.
    Io.write(near, "ab");
    Io.write(near, "cd");

    count, error = Io.read_full(near, .{ 4, buffer.data });

    assert(error == .NONE && count == 4 && string.{ count, buffer.data } == "ABCD", "ordered: %", string.{ count, buffer.data });

    Io.close(near);
}

test_end_of_stream :: () {
    near, far := Io.make_pipe();

    Io.write(near, "last words");
    Io.close(near);

    buffer: [4] u8;
    text: [..] u8;

    while true {
        count, error := Io.read(far, buffer);

        assert(error == .NONE, "read after the peer closed: %", error);

        if count == 0 break;

        for 0..count - 1 array_add(*text, buffer[it]);
    }

    assert(string.{ text.count, text.data } == "last words", "bytes sent before the close are read first: %", string.{ text.count, text.data });

    count, error := Io.read(far, buffer);

    assert(count == 0 && error == .NONE, "the end stays the end");

    _, write_error := Io.write(far, "late");

    assert(write_error == .BROKEN_PIPE, "writing to a closed peer: %", write_error);

    Io.close(far);
}

test_half_close :: () {
    near, far := Io.make_pipe();

    Io.write(near, "request");
    Io.shutdown_write(near);

    buffer: [32] u8;

    count, error := Io.read(far, buffer);

    assert(count == 7 && error == .NONE, "the data before the shutdown");

    count, error = Io.read(far, buffer);

    assert(count == 0 && error == .NONE, "then the end");

    // The other direction still works.
    Io.write(far, "response");

    count, error = Io.read(near, buffer);

    assert(count == 8 && string.{ count, buffer.data } == "response", "the shut end can still read");

    _, write_error := Io.write(near, "more");

    assert(write_error == .BROKEN_PIPE, "writing after shutdown_write: %", write_error);

    Io.close(near);
    Io.close(far);
}

Closer :: struct {
    end: *Io.Pipe_Stream;
    error: Io.Io_Error;
    count: s64;
    finished: bool;
}

read_once :: (closer: *Closer) {
    buffer: [8] u8;

    closer.count, closer.error = Io.read(closer.end, buffer);
    closer.finished = true;
}

test_close_wakes_reader :: () {
    near, far := Io.make_pipe();

    // Its own end closed under a waiting read: CLOSED.
    mine: Closer;

    mine.end = near;

    Io.spawn(read_once, *mine);
    Io.sleep(5);

    assert(!mine.finished, "the read waits");

    Io.close(near);
    Io.sleep(5);

    assert(mine.finished && mine.error == .CLOSED, "closing the end under a read: %", mine.error);

    // The other end closed under a waiting read: the end of the stream.
    near, far = Io.make_pipe();

    theirs: Closer;

    theirs.end = far;

    Io.spawn(read_once, *theirs);
    Io.sleep(5);

    assert(!theirs.finished, "the read waits");

    Io.close(near);
    Io.sleep(5);

    assert(theirs.finished && theirs.error == .NONE && theirs.count == 0, "the other end closing is the end of the stream: % %", theirs.error, theirs.count);

    Io.close(far);
}

test_deadline_and_cancel :: () {
    near, far := Io.make_pipe();

    buffer: [8] u8;

    started := to_milliseconds(current_time_monotonic());

    previous := Io.set_deadline(30);
    count, error := Io.read(far, buffer);

    Io.restore_deadline(previous);

    elapsed := to_milliseconds(current_time_monotonic()) - started;

    assert(error == .TIMED_OUT && count == 0 && elapsed >= 25, "a read past its deadline: % after % ms", error, elapsed);

    // The timed-out reader left the wait list: a later write and read are normal.
    Io.write(near, "ok");

    count, error = Io.read(far, buffer);

    assert(error == .NONE && count == 2, "the pipe still works after a timeout");

    // A canceled reader returns .CANCELED.
    waiting: Closer;

    waiting.end = far;

    handle := Io.spawn(read_once, *waiting);

    Io.sleep(5);
    Io.cancel(handle);
    Io.sleep(5);

    assert(waiting.finished && waiting.error == .CANCELED, "a canceled read: %", waiting.error);

    Io.write(near, "x");

    count, error = Io.read(far, buffer);

    assert(error == .NONE && count == 1, "the pipe still works after a cancel");

    Io.close(near);
    Io.close(far);
}

Sender :: struct {
    end: *Io.Pipe_Stream;
    total: int;
}

send_pattern :: (sender: *Sender) {
    block: [1000] u8;
    position := 0;

    while position < sender.total {
        size := min(1000, sender.total - position);

        for 0..size - 1 block[it] = pattern(position + it);

        Io.write(sender.end, .{ size, block.data });
        Io.yield();

        position += size;
    }

    Io.close(sender.end);
}

test_large_transfer :: () {
    near, far := Io.make_pipe();

    sender := Sender.{ far, 400_000 };

    Io.spawn(send_pattern, *sender);

    buffer: [333] u8;
    position := 0;

    while true {
        count, error := Io.read(near, buffer);

        assert(error == .NONE, "large read: %", error);

        if count == 0 break;

        for 0..count - 1 {
            assert(buffer[it] == pattern(position + it), "byte % differs", position + it);
        }

        position += count;
    }

    assert(position == 400_000, "all bytes arrive: %", position);

    Io.close(near);
}

test_read_full_stops_at_the_end :: () {
    near, far := Io.make_pipe();

    Io.write(near, "abc");
    Io.close(near);

    buffer: [8] u8;

    count, error := Io.read_full(far, buffer);

    assert(count == 3 && error == .END_OF_FILE, "read_full at the end: % %", count, error);

    Io.close(far);
}

test_all :: () {
    assert(Io.in_scheduler(), "inside io_run");

    test_echo();
    test_end_of_stream();
    test_half_close();
    test_close_wakes_reader();
    test_deadline_and_cancel();
    test_large_transfer();
    test_read_full_stops_at_the_end();
}

main :: () {
    assert(!Io.in_scheduler(), "outside io_run");

    Io.io_run(test_all);

    assert(!Io.in_scheduler(), "after io_run");

    print("ok\n");
}
