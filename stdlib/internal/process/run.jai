// Runs a program to its end and returns how it ended: `(process_result, output_string, error_string, timeout_reached)`.
//
// The first of `args` is the program: on Unix targets it is searched for on `PATH` unless it has a slash, and the rest are its
// arguments. No shell is involved, so `*`, `$` and `>` are just text; run `/bin/sh -c "..."` for those. The program starts in
// `working_directory` when one is given. Its standard input is closed at once, so a program that reads it sees end of file.
//
// - A program that cannot be started gives `FAILED_TO_LAUNCH` and empty strings.
// - With `capture_and_return_output` its standard output and error come back in `output_string` and `error_string`, new
//   strings from `context.allocator` that the caller frees, also when empty. Without it they are `""` literals, not to be freed,
//   and the child writes to this program's own output.
// - With `print_captured_output` the captured text is also written out as it arrives.
// - With `timeout_ms` of `0` or more the program is killed when that time has passed, and `timeout_reached` is `true`; the
//   result then says how the kill ended it (`SIGNALED` with `9` on Unix targets). `-1` waits for ever.
// - Output is read while the program runs, so a program that writes more than a pipe holds does not block.
//
// `arg_quoting` is for Windows, see `Process_Argument_Quoting`.
//
// ```jai,only(linux, macos)
// result, out, err, timed_out := run_command("/bin/sh", "-c", "printf copper; printf blue >&2; exit 7", capture_and_return_output = true, timeout_ms = 5000);
// assert(!timed_out && result.type == .EXITED && result.exit_code == 7);
// assert(out == "copper" && err == "blue");
// free(out);
// free(err);
//
// result, out, err, timed_out = run_command("/bin/sleep", "30", timeout_ms = 20);
// assert(timed_out && result.type == .SIGNALED && result.signal == 9);
//
// result, out, err, timed_out = run_command("/jai-doc-no-such-program");
// assert(result.type == .FAILED_TO_LAUNCH);
// ```
run_command :: (args: ..string, working_directory := "", capture_and_return_output := false, print_captured_output := false, timeout_ms := -1, arg_quoting := Process_Argument_Quoting.QUOTE_IF_NEEDED) -> (process_result: Process_Result, output_string := "", error_string := "", timeout_reached := false) {
    process: Process;
    if !create_process(*process, ..args, working_directory = working_directory, capture_and_return_output = capture_and_return_output, arg_quoting = arg_quoting)
        return Process_Result.{ type = .FAILED_TO_LAUNCH }, "", "", false;

    defer deinit(*process);
    close_pipe(*process.input);
    output: String_Builder;
    errors: String_Builder;

    defer free_buffers(*output);
    defer free_buffers(*errors);
    deadline: s64;

    if timeout_ms >= 0 {
        now, ok := clock_milliseconds();
        if !ok {
            cancel_or_report(*process);
            success, result := get_process_result(*process, -1);
            assert(success, "process status unavailable after clock failure");
            return result, "", "", false;
        }

        deadline = now + min(timeout_ms, 0x7fffffffffffffff - now);
    }

    timed_out := false;
    result: Process_Result;

    while true {
        success, current := get_process_result(*process, 0);
        if !success {
            context.logger("process result read failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            cancel_or_report(*process);
            break;
        }

        result = current;
        finished := result.type == .EXITED || result.type == .SIGNALED;
        if finished && (!capture_and_return_output || (process.output.eof && process.error.eof)) break;

        wait_time: s64 = 20;

        if timeout_ms >= 0 {
            now, ok := clock_milliseconds();
            if !ok {
                cancel_or_report(*process);
                break;
            }

            remaining := deadline - now;
            if remaining <= 0 {
                timed_out = true;

                if !finished {
                    cancel_or_report(*process);
                }

                break;
            }

            wait_time = min(wait_time, remaining);
        }

        if capture_and_return_output {
            stdout: [16384] u8;
            stderr: [16384] u8;
            read_ok, out_count, error_count := read_from_process(*process, stdout, stderr, wait_time);

            append(*output, stdout.data, out_count);
            append(*errors, stderr.data, error_count);

            if output.failed || errors.failed {
                cancel_or_report(*process);
                assert(false, "process output capture allocation failed");
            }

            if print_captured_output {
                write_string(string.{ out_count, stdout.data });
                write_string(string.{ error_count, stderr.data }, true);
            }

            if !read_ok {
                context.logger("process output read failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
                cancel_or_report(*process);
                break;
            }
        } else {
            success, current = get_process_result(*process, wait_time);
            if !success {
                cancel_or_report(*process);
                break;
            }

            result = current;
        }
    }

    success, terminal := get_process_result(*process, -1);

    assert(success, "process final status unavailable");
    result = terminal;
    if capture_and_return_output return result, builder_to_string(*output), builder_to_string(*errors), timed_out;

    return result, "", "", timed_out;
}
