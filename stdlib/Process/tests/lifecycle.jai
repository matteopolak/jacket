#import "Basic";
P :: #import "Process";

main :: () {
    untouched: P.Process;
    success, initial_status := P.get_process_result(*untouched, 0);

    assert(success && initial_status.type == .UNSTARTED);

    #if OS == .WINDOWS {
        assert(untouched.job_handle == cast(*void) -1);
        assert(untouched.input.handle == cast(*void) -1 && untouched.input.overlapped == null);
        args := get_command_line_arguments();

        // The platform I/O driver supplies the child program; the generic runner passes none.
        if args.count == 2 {
            status, output, errors, timed_out := P.run_command(args[1], "emit", "雪😀 quote\" slash\\", capture_and_return_output = true);

            assert(status.type == .EXITED && status.exit_code == 0 && !timed_out);
            assert(output == "雪😀 quote\" slash\\" && errors == "stderr\n");
            free(output);
            free(errors);
            status, output, errors, timed_out = P.run_command(args[1], "sleep", capture_and_return_output = true, timeout_ms = 10);
            assert(status.type == .EXITED && timed_out);
            free(output);
            free(errors);
        }
    } else {
        status, output, errors, timed_out := P.run_command("/bin/sh", "-c", "printf 'out'; printf 'err' >&2", capture_and_return_output = true);

        assert(status.type == .EXITED && status.exit_code == 0 && !timed_out);
        assert(output == "out" && errors == "err");
        free(output);
        free(errors);
    }

    print("ok\n");
}
