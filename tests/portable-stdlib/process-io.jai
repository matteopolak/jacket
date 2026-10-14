#import "Basic";
C :: #import "Process";

main :: () {
    process: C.Process;

    assert(C.create_process(*process, "/bin/sh", "-c", "exec 0<&-; printf ready; sleep 0.02", capture_and_return_output = true));
    bytes: [64] u8;
    errors: [64] u8;
    success, count, error_count := C.read_from_process(*process, bytes, errors, 2000);

    assert(success && count == 5 && to_string(bytes.data, count) == "ready");
    data: [1] u8 = .[97];
    wrote, total := C.write_to_process(*process, data);

    assert(!wrote && total == 0);
    ok, result := C.get_process_result(*process, 2000);

    assert(ok && result.type == .EXITED && result.exit_code == 0);
    C.deinit(*process);
    assert(C.create_process(*process, "/bin/sh", "-c", "printf '%s' $$", capture_and_return_output = true));
    success, count, error_count = C.read_from_process(*process, bytes, errors, 2000);
    assert(success && count > 0);
    pid, parsed, tail := to_integer(to_string(bytes.data, count));

    assert(parsed && tail.count == 0 && pid == process.pid);
    ok, result = C.get_process_result(*process, 2000);
    assert(ok && result.type == .EXITED);
    C.deinit(*process);
    print("ok\n");
}
