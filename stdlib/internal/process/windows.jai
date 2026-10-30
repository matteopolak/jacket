Process :: Types.Process;
Process_Platform :: Types.Process_Platform;
Pipe_Platform :: Types.Pipe_Platform;
Process_Result :: Types.Process_Result;
Process_Result_Type :: Types.Process_Result_Type;
Process_Argument_Quoting :: Types.Process_Argument_Quoting;

#load "arguments.jai";
#load "run.jai";

// Sets up `pipe` as one end of a new named pipe and returns the other end for the child, as `(success, client_handle)`.
// `is_input` says which way the data flows: `true` for a pipe the parent writes. The child's handle is inheritable; the
// caller passes it to the child and closes it. Windows only; `create_process` uses it, and callers rarely need it.
init_named_pipe_pair :: (pipe: *Process.Pipe, is_input: bool) -> success: bool, client_handle: W.HANDLE = W.INVALID_HANDLE_VALUE {
    return Engine.initialize_pipe(pipe, is_input);
}

// Starts the program `args[0]` with the arguments `args[1..]` and returns `true` once it is running; the result comes later from
// `get_process_result`. `process` must be a fresh `Process`.
//
// The arguments become one command line, quoted as `arg_quoting` says, with a limit of 32766 UTF-16 units. It returns `false`
// when there is no program name, when `process` is in use, when the command line is too long or the program cannot be started.
// The program is searched for the way `CreateProcess` does, so give a full path or a name with `.exe`.
//
// - `working_directory`, if not empty, becomes the child's current directory.
// - With `capture_and_return_output` the child's output and error are pipes you read from the `Process`; without it the child
//   shares this program's own. Its input is always a pipe.
// - With `kill_process_if_parent_exits` (the default) the child is in a job object that kills it if this program ends; turn it
//   off for a child that should live on.
//
// A child that fills an output pipe nobody reads blocks until someone does, so read with `read_from_process` while it runs.
create_process :: (process: *Process, args: ..string, working_directory := "", capture_and_return_output := false, arg_quoting := Process_Argument_Quoting.QUOTE_IF_NEEDED, kill_process_if_parent_exits := true) -> success: bool {
    return Engine.launch(process, args, working_directory, capture_and_return_output, kill_process_if_parent_exits, arg_quoting != .NEVER_QUOTE);
}

// Releases a `Process`: closes its pipes and handles and leaves it as a fresh record. Unlike on Unix it does not wait for the
// child. A child started with `kill_process_if_parent_exits` (the default) is killed when its job handle closes here; one
// started without it keeps running. Always call it once per successful `create_process`.
deinit :: (process: *Process) {
    Engine.destroy(process);
} @FreesArgument

// Closes one pipe of a process, for instance `process.input` so that a child reading its standard input sees the end of the file.
// The pipe is marked at its end. Safe on a pipe that is already closed.
deinit :: (pipe: *Process.Pipe) {
    Engine.destroy_pipe(pipe);
} @FreesArgument

// Writes `data` to the child's standard input and returns `(success, bytes_written)`. It waits until the pipe has taken everything.
// It fails with the number of bytes that did go in when the pipe is closed, or the child has closed its end. Empty data succeeds with `0`.
write_to_process :: (process: *Process, data: [] u8) -> success: bool, bytes_written: int {
    return Engine.write_input(process, data);
}

// Waits up to `timeout_ms` for the child's output or error and reads what arrived into the two buffers: `(success, output_bytes, error_bytes)`.
//
// A `timeout_ms` of `0` only looks, and `-1` (the default) waits for ever. Nothing arriving in time is a success with `0` and `0`.
// Each call reads at most one buffer's worth from each pipe, so loop until `process.output.eof` and `process.error.eof` for all of
// it, and give the loop a time limit of its own.
read_from_process :: (process: *Process, output_buffer: [] u8, error_buffer: [] u8, timeout_ms := -1) -> success: bool, output_bytes: int, error_bytes: int {
    return Engine.read_pair(process, output_buffer, error_buffer, timeout_ms);
}

// Terminates the child with exit code `1` and returns whether that was done: `false` for a process that was never started, and `true`
// without doing anything for one that has already ended. The result then reads `EXITED`; collect it with `get_process_result`.
kill_process :: (process: *Process) -> bool {
    return Engine.terminate(process);
}

// The state of the child as `(success, result)`, waiting up to `timeout_ms` for it to finish. `0` only checks and `-1` (the
// default) waits for ever. While it runs the result is `STILL_RUNNING`; afterwards `EXITED` with the exit code. A process never
// started is `UNSTARTED`. `success` is `false` when the system could not tell.
get_process_result :: (process: *Process, timeout_ms := -1) -> success: bool, Process_Result {
    if !process.info.hProcess return true, .{};

    ok, done, code := Engine.wait(process, timeout_ms);
    if !ok return false, .{};
    if !done return true, .{ type = .STILL_RUNNING };

    return true, .{ type = .EXITED, exit_code = cast,no_check(s32) code };
}

// Reads what is available now from one pipe of a process into `buffer`, and returns `(success, bytes_read)`.
// `0` bytes with `success` means nothing is there yet, or the end: check `pipe.eof`. It fails with a closed pipe or an empty buffer.
// Use `read_from_process` to wait for data on both pipes.
read_pipe :: (pipe: *Process.Pipe, buffer: [] u8) -> success: bool, bytes_read: int {
    return Engine.read_pipe(pipe, buffer);
}

#scope_module
#import "Basic";
Types :: #import "internal/process/windows-types";
Engine :: #import "internal/platform-io/windows-process";
W :: #import "internal/bindings/windows-api";

close_pipe :: (pipe: *Process.Pipe) {
    deinit(pipe);
} @FreesArgument

cancel_or_report :: (process: *Process) {
    if !kill_process(process) context.logger("process termination failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
}

clock_milliseconds :: () -> s64, bool #no_context {
    value := GetTickCount64();
    if value > 0x7fffffffffffffff return 0, false;
    return cast(s64) value, true;
}

#scope_file
kernel32 :: #system_library "kernel32"; @native(external)
GetTickCount64 :: () -> u64 #foreign kernel32;
