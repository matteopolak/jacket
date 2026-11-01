#if OS == .MACOS || OS == .LINUX {
    #import "Basic";
    #import "String";
    #import "Extensions/Testing";
    #import "Extensions/Pty";
    #import "System";

    real_pty_output_and_resize :: () {
        s, ok := open_session("/bin/sh", "stty size; printf '\\033[31mpty-ok\\033[0m'", 37, 9);

        require(ok, "PTY opens");
        defer close_session(*s);
        require(resize_session(*s, 37, 9));
        output: String_Builder;
        buffer: [1024] u8;
        ended := false;

        for 0..499 {
            n := read_session(*s, buffer);
            if n > 0 append(*output, buffer.data, n);

            if n < 0 {
                ended = true;
                break;
            }

            sleep_milliseconds(2);
        }

        text := builder_to_string(*output);

        defer free(text);
        expect(ended, "child closes output within one second");
        expect(contains(text, "9 37"), text);
        expect(contains(text, "\x1b[31mpty-ok\x1b[0m"), text);

        for 0..99 {
            if poll_exit(*s) break;
            sleep_milliseconds(2);
        }

        expect(s.exited && s.status == 0);
    } @test
}

#if OS == .MACOS || OS == .LINUX {
    collect_until :: (s: *Session, needle: string) -> string {
        bytes: [..] u8;

        defer array_free(bytes);
        buffer: [1024] u8;

        for 0..999 {
            n := read_session(s, buffer);
            if n < 0 break;
            if n > 0 for i: 0..n - 1 array_add(*bytes, buffer[i]);

            text := string.{ bytes.count, bytes.data };
            if contains(text, needle) return copy_string(text);

            sleep_milliseconds(2);
        }

        return copy_string(.{ bytes.count, bytes.data });
    }

    input_and_foreground_resize_signal :: () {
        s, ok := open_session("/bin/sh", "stty -echo; printf 'ready:%s\\n' \"$TERM\"; read line; printf 'input:%s\\n' \"$line\"; trap 'stty size; exit 0' WINCH; printf armed; while :; do sleep 0.02; done", 20, 4);

        require(ok);
        defer close_session(*s);
        ready := collect_until(*s, "ready:xterm-256color");

        defer free(ready);
        require(contains(ready, "ready:xterm-256color"), ready);
        expect_equal(write_session(*s, "hello\n"), 6);
        armed := collect_until(*s, "armed");

        defer free(armed);
        require(contains(armed, "input:hello"), armed);
        require(contains(armed, "armed"), armed);
        require(resize_session(*s, 61, 13));
        resized := collect_until(*s, "13 61");

        defer free(resized);
        expect(contains(resized, "13 61"), resized);
    } @test

    rejects_bad_sizes_without_opening_a_child :: () {
        s, ok := open_session("/bin/sh", "exit", 0, 24);
        expect(!ok && s.fd == -1 && s.pid == -1);
        close_session(*s);
        close_session(*s);
    } @test
}
