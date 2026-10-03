Process :: Types.Process;
Process_Platform :: Types.Process_Platform;
Pipe_Platform :: Types.Pipe_Platform;
Process_Result :: Types.Process_Result;
Process_Result_Type :: Types.Process_Result_Type;
Process_Argument_Quoting :: Types.Process_Argument_Quoting;

#load "arguments.jai";
#load "run.jai";

init_named_pipe_pair :: (pipe: *Process.Pipe, is_input: bool) -> success: bool, client_handle: W.HANDLE = W.INVALID_HANDLE_VALUE {
    return Engine.initialize_pipe(pipe, is_input);
}

create_process :: (process: *Process, args: ..string, working_directory := "", capture_and_return_output := false, arg_quoting := Process_Argument_Quoting.QUOTE_IF_NEEDED, kill_process_if_parent_exits := true) -> success: bool {
    return Engine.launch(process, args, working_directory, capture_and_return_output, kill_process_if_parent_exits, arg_quoting != .NEVER_QUOTE);
}

deinit :: (process: *Process) {
    Engine.destroy(process);
}

deinit :: (pipe: *Process.Pipe) {
    Engine.destroy_pipe(pipe);
}

write_to_process :: (process: *Process, data: [] u8) -> success: bool, bytes_written: int {
    return Engine.write_input(process, data);
}

read_from_process :: (process: *Process, output_buffer: [] u8, error_buffer: [] u8, timeout_ms := -1) -> success: bool, output_bytes: int, error_bytes: int {
    return Engine.read_pair(process, output_buffer, error_buffer, timeout_ms);
}

kill_process :: (process: *Process) -> bool {
    return Engine.terminate(process);
}

get_process_result :: (process: *Process, timeout_ms := -1) -> success: bool, Process_Result {
    if !process.info.hProcess return true, .{};

    ok, done, code := Engine.wait(process, timeout_ms);
    if !ok return false, .{};
    if !done return true, .{ type = .STILL_RUNNING };

    return true, .{ type = .EXITED, exit_code = cast,no_check(s32) code };
}

read_pipe :: (pipe: *Process.Pipe, buffer: [] u8) -> success: bool, bytes_read: int {
    return Engine.read_pipe(pipe, buffer);
}

#scope_module
#import "Basic";
Types :: #import "internal/process/windows-types";
Engine :: #import "internal/platform-io/windows-process";
W :: #import "Windows";

close_pipe :: (pipe: *Process.Pipe) {
    deinit(pipe);
}

cancel_or_report :: (process: *Process) {
    if !kill_process(process) context.logger("process termination failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
}

clock_milliseconds :: () -> s64, bool #no_context {
    value := GetTickCount64();
    if value > 0x7fffffffffffffff return 0, false;
    return cast(s64) value, true;
}

#scope_file
kernel32 :: #system_library "kernel32";
GetTickCount64 :: () -> u64 #foreign kernel32;
