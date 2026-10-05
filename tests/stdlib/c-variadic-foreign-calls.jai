// C variadic foreign calls (`args: ..Any` on a #foreign procedure): Apple arm64 passes the
// variadic arguments on the stack, so they must not be treated as fixed register arguments.
// Also built natively by crates/jaic-cli/tests/native.rs.
#import "Basic";

libc :: #system_library "libc";
snprintf :: (buf: *u8, n: u64, fmt: *u8, args: ..Any) -> s32 #foreign libc;
#if OS != .WINDOWS {
    fcntl :: (fd: s32, cmd: s32, args: ..Any) -> s32 #foreign libc;
    pipe :: (fds: *[2] s32) -> s32 #foreign libc;
    close :: (fd: s32) -> s32 #foreign libc;
}

main :: () {
    buf: [128] u8;
    n := snprintf(buf.data, buf.count, "%d %s %.2f %ld %c", cast(s32) 42, "hi".data, 1.5, cast(s64) -7, cast(s32) #char "x");
    text := to_string(buf.data, n);
    assert(text == "42 hi 1.50 -7 x", "snprintf gave '%'", text);

    // Win64 passes variadic doubles in both an XMM and the matching integer register, and
    // the fifth argument onwards on the stack.
    n = snprintf(buf.data, buf.count, "%.1f %.1f %d %.3f %s", 0.5, 2.4, cast(s32) 9, 3.125, "end".data);
    text = to_string(buf.data, n);
    assert(text == "0.5 2.4 9 3.125 end", "snprintf gave '%'", text);

    // fcntl's third argument is variadic: FD_CLOEXEC must actually be set.
    #if OS != .WINDOWS {
        F_GETFD :: 1;
        F_SETFD :: 2;
        FD_CLOEXEC :: 1;
        fds: [2] s32;
        assert(pipe(*fds) == 0);
        assert(fcntl(fds[0], F_SETFD, cast(s32) FD_CLOEXEC) == 0);
        assert(fcntl(fds[0], F_GETFD) & FD_CLOEXEC != 0, "FD_CLOEXEC was not set");
        close(fds[0]);
        close(fds[1]);
    }
    print("ok\n");
}
