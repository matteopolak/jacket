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
