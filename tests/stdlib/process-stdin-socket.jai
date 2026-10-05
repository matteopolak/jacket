// A captured child's stdin is a stream socket (children use it to message their parent, as
// rluba/cluster does), and read_pipe reports end of input as (true, 0) without closing the
// handle: callers watching the handles themselves decide when to mark them done.
#import "Basic";
#import "Process";
#import "POSIX";

main :: () {
    #if OS == .WINDOWS { print("ok\n"); return; }

    process: Process;
    ok := create_process(*process, "sh", "-c", "printf hi; printf done >&0", capture_and_return_output = true);
    assert(ok);
    defer deinit(*process);

    info: stat_t;
    assert(fstat(process.input.handle, *info) == 0);
    assert((info.st_mode & S_IFMT) == S_IFSOCK);

    // The child wrote into its stdin socket; the parent reads it there.
    buffer: [16] u8;
    got := read(process.input.handle, buffer.data, 16);
    while got < 0 && errno() == EINTR  got = read(process.input.handle, buffer.data, 16);
    assert(to_string(buffer.data, got) == "done");

    output: [..] u8;
    while true {
        success, count := read_pipe(*process.output, buffer);
        assert(success);
        if count == 0 break;
        array_add(*output, ..array_view(buffer, 0, count));
    }
    assert(cast(string) output == "hi");
    assert(process.output.handle >= 0);

    success, result := get_process_result(*process);
    assert(success && result.type == .EXITED && result.exit_code == 0);
    print("ok\n");
}
