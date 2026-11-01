// test: skip_on(.WINDOWS, reason = "POSIX stream paths")
// test: skip_on(.WASM, reason = "fibers require native stacks")
#import "Basic";
#import "File";
P :: #import "POSIX";
Io :: #import "Extensions/Io";

feed :: (fd: s32) {
    Io.sleep(20);
    assert(P.write(fd, "hello\n".data, 6) == 6);
}

main :: () {
    Io.io_run(() {
        path := tprint("%/chat-fifo-%", ".", to_nanoseconds(current_time_monotonic()));

        assert(P.mkfifo(temp_c_string(path), 384) == 0);
        defer file_delete(path);
        fd := P.open(temp_c_string(path), P.O_RDWR | P.O_NONBLOCK);

        assert(fd >= 0);
        defer P.close(fd);
        input, error := Io.open_stream_file(path);

        assert(error == .NONE);
        defer Io.close(*input);
        lines := Io.make_line_reader(Io.reader_of(*input));

        defer Io.free_line_reader(*lines);
        Io.with_deadline(5, #code {
            _, timed := Io.read_line(*lines);
            assert(timed == .TIMED_OUT);
        });
        Io.spawn(feed, fd);
        line, received := Io.read_line(*lines);

        assert(received == .NONE && line == "hello");
        _, absent := Io.open_stream_file(tprint("%-absent", path));

        assert(absent == .NOT_FOUND);

        // On macOS /dev/fd duplicates a blocking descriptor and ignores open flags.
        // Check the mode before attempting IO, so a regression fails rather than hanging.
        fds: [2] s32;

        assert(P.pipe(*fds) == 0);
        defer P.close(fds[0]);
        defer P.close(fds[1]);
        previous := P.fcntl(fds[0], P.F_GETFL);
        duplicate, duplicated := Io.open_stream_file(tprint("/dev/fd/%", fds[0]));

        assert(duplicated == .NONE);
        flags := P.fcntl(cast(s32) duplicate.handle.fd, P.F_GETFL);

        assert((flags & P.O_NONBLOCK) != 0);
        buffer: [1] u8;

        Io.with_deadline(5, #code {
            _, timed := Io.read(*duplicate, buffer);
            assert(timed == .TIMED_OUT);
        });
        assert(Io.close(*duplicate) == .NONE);
        assert(P.fcntl(fds[0], P.F_GETFL) == previous);
    });
}
