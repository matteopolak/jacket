// Public Windows process lifetime, capture, timeout and pipe ownership regression.
#import "Basic";
P :: #import "Process";
W :: #import "Windows";

main :: () {
    args := get_command_line_arguments();
    if args.count == 4 {
        detached: P.Process;

        assert(args[2] == "parent-keep" || args[2] == "parent-kill");
        assert(P.create_process(*detached, ..string.[args[1], "marker", args[3]], kill_process_if_parent_exits = args[2] == "parent-kill"));

        // Deliberately exit with the job handle open: the kernel closes it on
        // parent exit. The runner checks termination versus the explicit opt-out.
        return;
    }

    assert(args.count == 2);
    assert(size_of(P.Pipe_Platform) == 16 && size_of(P.Process.Pipe) == 24);
    assert(size_of(P.Process_Platform) == 32 && size_of(P.Process) == 104);
    assert(type_info(P.Process).name == "Process" && type_info(P.Process.Pipe).name == "Pipe");
    assert(type_info(P.Process_Platform).name == "Process_Platform" && type_info(P.Pipe_Platform).name == "Pipe_Platform");
    standalone: P.Process.Pipe;

    assert(standalone.handle == W.INVALID_HANDLE_VALUE && standalone.overlapped == null && !standalone.eof);

    {
        push_allocator(Allocator.{ deny_allocations, null });
        allocated, failed_client := P.init_named_pipe_pair(*standalone, false);
        assert(!allocated && failed_client == W.INVALID_HANDLE_VALUE);
        assert(standalone.handle == W.INVALID_HANDLE_VALUE && standalone.overlapped == null);
    }

    pair_ok, client := P.init_named_pipe_pair(*standalone, false);

    assert(pair_ok && standalone.overlapped != null);
    sent_native: u32;

    assert(W.WriteFile(client, cast(*void) "pipe".data, 4, *sent_native, null) != 0 && sent_native == 4);
    pipe_buffer: [32] u8;
    pipe_ok, pipe_bytes := P.read_pipe(*standalone, pipe_buffer);

    assert(pipe_ok && pipe_bytes == 4);
    assert(W.CloseHandle(client) != 0);
    pipe_ok, pipe_bytes = P.read_pipe(*standalone, pipe_buffer);
    assert(pipe_ok && pipe_bytes == 0 && standalone.eof);
    P.deinit(*standalone);
    assert(standalone.overlapped == null && standalone.handle == W.INVALID_HANDLE_VALUE);
    child: P.Process;

    assert(!P.create_process(*child));
    failed, failed_output, failed_errors, failed_timeout := P.run_command("_platform_io_missing_executable_31eea0.exe", capture_and_return_output = true);

    assert(failed.type == .FAILED_TO_LAUNCH && failed_output.count == 0 && failed_errors.count == 0 && !failed_timeout);
    assert(P.create_process(*child, ..string.[args[1], "exit-high"]));
    exited, high := P.get_process_result(*child, 0x100000001);

    assert(exited && high.type == .EXITED && high.exit_code == -1);
    P.deinit(*child);

    assert(P.create_process(*child, ..string.[args[1], "emit", "雪😀 quote\" slash\\"], capture_and_return_output = true));
    assert(child.info.hProcess != null && child.info.hThread != null);
    assert(child.info.dwProcessId != 0 && child.info.dwThreadId != 0);
    assert(child.job_handle != W.INVALID_HANDLE_VALUE && child.output.overlapped != null);
    P.deinit(*child.input);
    output: String_Builder;
    errors: String_Builder;
    stdout: [512] u8;
    stderr: [512] u8;

    while !child.output.eof || !child.error.eof {
        ok, out_count, err_count := P.read_from_process(*child, stdout, stderr, 5000);
        assert(ok);
        append(*output, stdout.data, out_count);
        append(*errors, stderr.data, err_count);
    }

    ok, done, code := wait_child(*child, 5000);

    assert(ok && done && code == 0);
    out_text := builder_to_string(*output);
    err_text := builder_to_string(*errors);

    assert(out_text == "雪😀 quote\" slash\\" && err_text == "stderr\n");
    free(out_text);
    free(err_text);
    free_buffers(*output);
    free_buffers(*errors);
    P.deinit(*child);

    assert(P.create_process(*child, ..string.[args[1], "echo"], capture_and_return_output = true));
    sent, count := P.write_to_process(*child, cast([] u8) "binary\0\r\n雪");

    assert(sent && count == 12);
    P.deinit(*child.input);

    while !child.output.eof || !child.error.eof {
        read_ok, out_count, err_count := P.read_from_process(*child, stdout, stderr, 5000);
        assert(read_ok);
        append(*output, stdout.data, out_count);
        append(*errors, stderr.data, err_count);
    }

    ok, done, code = wait_child(*child, 5000);
    assert(ok && done && code == 0);
    out_text = builder_to_string(*output);
    assert(out_text == "binary\0\r\n雪");
    free(out_text);
    free_buffers(*output);
    free_buffers(*errors);
    P.deinit(*child);

    assert(P.create_process(*child, ..string.[args[1], "zero"], capture_and_return_output = true));
    P.deinit(*child.input);

    while !child.output.eof || !child.error.eof {
        read_ok, out_count, err_count := P.read_from_process(*child, stdout, stderr, 5000);
        assert(read_ok && err_count == 0);
        append(*output, stdout.data, out_count);
    }

    ok, done, code = wait_child(*child, 5000);
    assert(ok && done && code == 0);
    out_text = builder_to_string(*output);
    assert(out_text == "after-zero");
    free(out_text);
    free_buffers(*output);
    P.deinit(*child);

    assert(P.create_process(*child, ..string.[args[1], "sleep"], capture_and_return_output = true));
    timed_ok, timed_out_count, timed_err_count := P.read_from_process(*child, stdout, stderr, 1);

    assert(timed_ok && timed_out_count == 0 && timed_err_count == 0);
    ok, done, code = wait_child(*child, 0);
    assert(ok && !done);
    assert(P.kill_process(*child));
    ok, done, code = wait_child(*child, 5000);
    assert(ok && done && code != 0);
    P.deinit(*child);

    assert(P.create_process(*child, ..string.[args[1], "bulk"], capture_and_return_output = true));
    P.deinit(*child.input);
    total_output: s64;
    total_error: s64;

    while !child.output.eof || !child.error.eof {
        read_ok, out_count, err_count := P.read_from_process(*child, stdout, stderr, 5000);

        assert(read_ok);
        for i: 0..out_count - 1 assert(stdout[i] == 120);
        for i: 0..err_count - 1 assert(stderr[i] == 120);

        total_output += out_count;
        total_error += err_count;
    }

    ok, done, code = wait_child(*child, 5000);
    assert(ok && done && code == 0);
    assert(total_output == 262144 && total_error == 262144);
    P.deinit(*child);
    print("ok\n");
}

#scope_file
deny_allocations :: (_mode: Allocator_Mode, _size: s64, _old_size: s64, _old_memory: *void, _allocator_data: *void) -> *void {
    return null;
}

wait_child :: (child: *P.Process, timeout: s64) -> bool, bool, u32 {
    ok, result := P.get_process_result(child, timeout);
    return ok, result.type == .EXITED, cast,no_check(u32) result.exit_code;
}
