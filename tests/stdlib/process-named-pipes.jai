// Windows only: a child process started with `init_named_pipe_pair` pipes as its standard input
// and output. Input is written to the child through the overlapped parent end, the parent end is
// closed so the child sees end of input, and the child's sorted output is read back through the
// other overlapped pair. Other operating systems have no such pipes, so the program only
// prints ok there.
#import "Basic";
#import "Process";

#if OS == .WINDOWS {
    Windows :: #import "Windows";
    Utf8 :: #import "Windows_Utf8";

    // Waits for an overlapped operation that was started on `pipe`. Returns false at the end of
    // the stream (the other side closed).
    finish_io :: (pipe: *Process.Pipe, started: bool, immediate: u32) -> bool, int {
        if !started {
            error := cast(u32) Windows.GetLastError();
            if error == Windows.ERROR_BROKEN_PIPE return false, 0;
            assert(error == Windows.ERROR_IO_PENDING, "unexpected error %", error);
            transferred: s32;
            if !Windows.GetOverlappedResult(pipe.handle, pipe.overlapped, *transferred, 1) {
                error = cast(u32) Windows.GetLastError();
                if error == Windows.ERROR_BROKEN_PIPE return false, 0;
                assert(false, "overlapped result failed: %", error);
            }

            return true, transferred;
        }

        return true, cast(int) immediate;
    }

    write_all :: (pipe: *Process.Pipe, text: string) {
        written := 0;
        while written < text.count {
            amount: u32;
            started := Windows.WriteFile(pipe.handle, text.data + written, cast(u32) (text.count - written), *amount, pipe.overlapped) != 0;
            ok, count := finish_io(pipe, started, amount);

            assert(ok, "the child closed its input early");
            written += count;
        }
    }

    read_all :: (pipe: *Process.Pipe) -> string {
        builder: String_Builder;
        buffer: [4096] u8;

        while true {
            amount: u32;
            started := Windows.ReadFile(pipe.handle, buffer.data, buffer.count, *amount, pipe.overlapped) != 0;
            ok, count := finish_io(pipe, started, amount);

            if !ok break;
            append(*builder, string.{ count, buffer.data });
        }

        return builder_to_string(*builder);
    }

    main :: () {
        input: Process.Pipe;
        output: Process.Pipe;
        ok_in, child_in := init_named_pipe_pair(*input, true);
        ok_out, child_out := init_named_pipe_pair(*output, false);

        assert(ok_in && ok_out, "could not make the pipe pairs");
        assert(input.overlapped && input.overlapped.hEvent && !input.eof, "the parent end is set up");
        assert(output.overlapped && output.overlapped.hEvent && input.handle != output.handle);

        startup: Windows.STARTUPINFOW;
        startup.cb = size_of(Windows.STARTUPINFOW);
        startup.dwFlags = Windows.STARTF_USESTDHANDLES;
        startup.hStdInput = child_in;
        startup.hStdOutput = child_out;
        startup.hStdError = child_out;
        info: Windows.PROCESS_INFORMATION;
        command := Utf8.utf8_to_wide("sort");
        launched := Windows.CreateProcessW(null, command, null, null, 1, 0, null, null, *startup, *info) != 0;

        assert(launched, "could not start sort");
        // The child holds its own copies now; closing ours is what lets end of input and end of
        // output reach us.
        Windows.CloseHandle(child_in);
        Windows.CloseHandle(child_out);

        write_all(*input, "pear\r\napple\r\nfig\r\n");
        deinit(*input);
        assert(input.eof && input.overlapped == null, "deinit releases the parent end");

        text := read_all(*output);

        assert(text == "apple\r\nfig\r\npear\r\n", "sorted output was '%'", text);
        deinit(*output);

        Windows.WaitForSingleObject(info.hProcess, Windows.INFINITE);
        code: u32;

        assert(Windows.GetExitCodeProcess(info.hProcess, *code) != 0 && code == 0, "sort exit code %", code);
        Windows.CloseHandle(info.hThread);
        Windows.CloseHandle(info.hProcess);
        print("ok\n");
    }
} else {
    main :: () {
        print("ok\n");
    }
}
